use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom},
    os::windows::ffi::OsStrExt as _,
    os::windows::fs::OpenOptionsExt as _,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use compact_str::CompactString;
use crossbeam::channel::Sender;
use windows::{Win32::Storage::FileSystem::GetVolumeInformationW, core::PCWSTR};

use super::traversal::{LocalId, ScanEvent, TraversalStats};

const MFT_RECORD_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16MB Reusable Staging Buffer

struct MftEntry {
    name: CompactString,
    size: u64,
    parent_record_id: u64,
    modified_timestamp: i64,
    created_timestamp: i64,
    accessed_timestamp: i64,
    is_dir: bool,
    is_symlink: bool,
    has_attr_list: bool,
}

struct TraversalFrame {
    record_id: u64,
    parent_local_id: LocalId,
}

struct DataRun {
    length_clusters: u64,
    lcn: Option<i64>, // None means sparse
}

struct ExtractedLink {
    parent_ref: u64,
    name: CompactString,
    size: u64,
    has_attr_list: bool,
}

/// Converts Windows NT 100-nanosecond intervals to standard Unix epoch seconds.
#[inline]
const fn nt_time_to_unix(nt_time: u64) -> i64 {
    if nt_time == 0 {
        0
    } else {
        (nt_time / 10_000_000).saturating_sub(11_644_473_600) as i64
    }
}

/// Applies the NTFS Sector Update Sequence Array (USA) fixup in-place to guarantee sector integrity.
fn apply_fixup(buffer: &mut [u8]) -> bool {
    if buffer.len() < 24 {
        return false;
    }
    if &buffer[0..4] != b"FILE" && &buffer[0..4] != b"INDX" {
        return false;
    }
    let update_seq_offset = u16::from_le_bytes(buffer[4..6].try_into().unwrap_or([0; 2])) as usize;
    let update_seq_count = u16::from_le_bytes(buffer[6..8].try_into().unwrap_or([0; 2])) as usize;

    if update_seq_offset + update_seq_count * 2 > buffer.len() {
        return false;
    }

    let usn = [buffer[update_seq_offset], buffer[update_seq_offset + 1]];
    let mut array_pos = update_seq_offset + 2;
    let mut sector_pos = 512 - 2;

    for _ in 1..update_seq_count {
        if sector_pos + 2 > buffer.len() {
            break;
        }
        if [buffer[sector_pos], buffer[sector_pos + 1]] != usn {
            return false;
        }
        buffer[sector_pos] = buffer[array_pos];
        buffer[sector_pos + 1] = buffer[array_pos + 1];
        array_pos += 2;
        sector_pos += 512;
    }
    true
}

/// Decodes non-resident data runs mapping the sequential cluster layout.
fn decode_data_runs(mut data_runs_bytes: &[u8]) -> Vec<DataRun> {
    let mut runs = Vec::new();
    let mut previous_lcn = 0i64;

    while !data_runs_bytes.is_empty() {
        let header = data_runs_bytes[0];
        if header == 0 {
            break;
        }
        data_runs_bytes = &data_runs_bytes[1..];

        let len_bytes_count = (header & 0x0F) as usize;
        let offset_bytes_count = ((header & 0xF0) >> 4) as usize;

        if data_runs_bytes.len() < len_bytes_count + offset_bytes_count {
            break;
        }

        let mut len_buf = [0u8; 8];
        len_buf[..len_bytes_count].copy_from_slice(&data_runs_bytes[..len_bytes_count]);
        let length_clusters = u64::from_le_bytes(len_buf);
        data_runs_bytes = &data_runs_bytes[len_bytes_count..];

        let lcn = if offset_bytes_count > 0 {
            let mut off_buf = [0u8; 8];
            off_buf[..offset_bytes_count].copy_from_slice(&data_runs_bytes[..offset_bytes_count]);
            let mut offset = i64::from_le_bytes(off_buf);
            let unused_bits = (8 - offset_bytes_count) * 8;
            offset = (offset << unused_bits) >> unused_bits;
            data_runs_bytes = &data_runs_bytes[offset_bytes_count..];

            let current_lcn = previous_lcn + offset;
            previous_lcn = current_lcn;
            Some(current_lcn)
        } else {
            None
        };

        runs.push(DataRun {
            length_clusters,
            lcn,
        });
    }
    runs
}

struct AttributeHeader<'a> {
    ty: u32,
    is_non_resident: bool,
    name_length: usize,
    value_length: u64,
    payload: &'a [u8],
}

/// Returns a collection of raw attributes contained inside the MFT record.
fn parse_attributes(record_data: &[u8]) -> Vec<AttributeHeader<'_>> {
    let mut attrs = Vec::new();
    if record_data.len() < 24 {
        return attrs;
    }
    let first_attr_offset =
        u16::from_le_bytes(record_data[20..22].try_into().unwrap_or([0; 2])) as usize;
    let mut offset = first_attr_offset;

    while offset + 8 <= record_data.len() {
        let ty = u32::from_le_bytes(record_data[offset..offset + 4].try_into().unwrap_or([0; 4]));
        if ty == 0xFFFF_FFFF {
            break;
        }
        let length = u32::from_le_bytes(
            record_data[offset + 4..offset + 8]
                .try_into()
                .unwrap_or([0; 4]),
        ) as usize;
        if length == 0 || offset + length > record_data.len() {
            break;
        }

        let is_non_resident = record_data[offset + 8] != 0;
        let name_length = record_data[offset + 9] as usize;

        let value_length = if is_non_resident {
            if length >= 56 {
                u64::from_le_bytes(
                    record_data[offset + 48..offset + 56]
                        .try_into()
                        .unwrap_or([0; 8]),
                )
            } else {
                0
            }
        } else if length >= 20 {
            u32::from_le_bytes(
                record_data[offset + 16..offset + 20]
                    .try_into()
                    .unwrap_or([0; 4]),
            ) as u64
        } else {
            0
        };

        attrs.push(AttributeHeader {
            ty,
            is_non_resident,
            name_length,
            value_length,
            payload: &record_data[offset..offset + length],
        });

        offset += length;
    }
    attrs
}

/// Extracts all unique directory links from a file record, deduplicating local namespace variants.
fn extract_all_links_from_record(record_buffer: &[u8]) -> Vec<ExtractedLink> {
    let mut links: HashMap<u64, (CompactString, i32)> = HashMap::new();
    let mut unnamed_data_size = None;
    let mut has_attr_list = false;
    let mut fallback_size = 0u64;

    let attrs = parse_attributes(record_buffer);

    for attr in &attrs {
        match attr.ty {
            0x30 if !attr.is_non_resident && attr.payload.len() >= 24 => {
                // Resident FileName Attribute
                let val_offset =
                    u16::from_le_bytes(attr.payload[20..22].try_into().unwrap_or([0; 2])) as usize;
                let val_len =
                    u32::from_le_bytes(attr.payload[16..20].try_into().unwrap_or([0; 4])) as usize;
                if val_offset + val_len <= attr.payload.len() && val_len >= 66 {
                    let val = &attr.payload[val_offset..val_offset + val_len];
                    let parent_ref = u64::from_le_bytes(val[0..8].try_into().unwrap_or([0; 8]))
                        & 0x0000_ffff_ffff_ffff;
                    let namespace = val[65];
                    let prio = match namespace {
                        1 => 3, // Win32
                        3 => 2, // Win32AndDos
                        2 => 1, // Dos
                        _ => 0, // Posix
                    };

                    let name_len = val[64] as usize;
                    if 66 + name_len * 2 <= val.len() {
                        let name_raw = &val[66..66 + name_len * 2];
                        let u16_chars: Vec<u16> = name_raw
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();

                        if let Ok(decoded_name) = String::from_utf16(&u16_chars) {
                            let compact_name = CompactString::new(&decoded_name);
                            let entry = links
                                .entry(parent_ref)
                                .or_insert_with(|| (CompactString::new(""), -1));
                            if prio > entry.1 {
                                *entry = (compact_name, prio);
                            }
                        }
                    }

                    if fallback_size == 0 {
                        fallback_size =
                            u64::from_le_bytes(val[48..56].try_into().unwrap_or([0; 8]));
                    }
                }
            }
            0x80 if attr.name_length == 0 => {
                // $DATA Attribute
                unnamed_data_size = Some(attr.value_length);
            }
            0x20 => {
                // $ATTRIBUTE_LIST Attribute
                has_attr_list = true;
            }
            _ => {}
        }
    }

    // Trust the unnamed $DATA stream size if found (greater than 0),
    // otherwise fallback to the filename data_size (catches WOF-compressed/placeholder system files).
    let actual_size = match unnamed_data_size {
        Some(size) if size > 0 => size,
        _ => fallback_size,
    };

    links
        .into_iter()
        .map(|(parent_ref, (name, _))| ExtractedLink {
            parent_ref,
            name,
            size: actual_size,
            has_attr_list,
        })
        .collect()
}

/// Reconstructs the absolute path of an MFT record purely in-memory by walking up parent references.
fn reconstruct_path(
    mft_entries: &[Option<MftEntry>],
    record_id: u64,
    root_record_id: u64,
    root_path: &Path,
) -> PathBuf {
    let mut path = PathBuf::new();
    let mut curr = record_id;
    let mut segments = Vec::new();

    while curr != root_record_id {
        if let Some(Some(entry)) = mft_entries.get(curr as usize) {
            segments.push(entry.name.as_str());
            curr = entry.parent_record_id;
        } else {
            break;
        }
    }

    path.push(root_path);
    for segment in segments.into_iter().rev() {
        path.push(segment);
    }
    path
}

/// Sweeps the in-memory tree starting from the Root record to locate the target directory.
fn find_target_record_in_memory(
    mft_entries: &[Option<MftEntry>],
    children_map: &HashMap<u64, Vec<u64>, ahash::RandomState>,
    root_path: &Path,
) -> u64 {
    let mut current_record = 5u64; // Root folder constant
    let mut components = Vec::new();
    for component in root_path.components() {
        if let std::path::Component::Normal(name) = component {
            components.push(name.to_string_lossy().to_ascii_lowercase());
        }
    }

    for segment in components {
        let mut found = false;
        if let Some(children) = children_map.get(&current_record) {
            for &child_id in children {
                if let Some(Some(entry)) = mft_entries.get(child_id as usize)
                    && entry.is_dir
                    && entry.name.to_ascii_lowercase() == segment
                {
                    current_record = child_id;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return 5; // Fallback to root directory if the nested hierarchy cannot be mapped
        }
    }
    current_record
}

/// Primary entry point for raw Windows MFT scanning.
/// Parses the raw drive partition, builds the structural index, and streams events hierarchically.
pub fn try_scan_mft(
    root_path: &Path,
    event_tx: &Sender<Vec<ScanEvent>>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    let volume_path = get_volume_path(root_path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unable to determine partition drive letter from target path",
        )
    })?;

    let mut options = OpenOptions::new();
    options.read(true);
    options.share_mode(7);

    let mut raw_disk = options.open(&volume_path)?;

    // 1. Read Boot Sector
    let mut boot_sector = [0u8; 512];
    raw_disk.read_exact(&mut boot_sector)?;

    if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid boot sector signature",
        )
        .into());
    }

    let sector_size =
        u16::from_le_bytes(boot_sector[0x0B..0x0D].try_into().unwrap_or([0; 2])) as u64;
    let sectors_per_cluster = boot_sector[0x0D] as u64;
    let cluster_size = sector_size * sectors_per_cluster;
    let mft_lcn = u64::from_le_bytes(boot_sector[0x30..0x38].try_into().unwrap_or([0; 8]));
    let mft_start_offset = mft_lcn * cluster_size;

    // 2. Read MFT Record 0 ($MFT)
    let mut mft_record_zero = vec![0u8; MFT_RECORD_SIZE];
    raw_disk.seek(SeekFrom::Start(mft_start_offset))?;
    raw_disk.read_exact(&mut mft_record_zero)?;

    if !apply_fixup(&mut mft_record_zero) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed record fixup for $MFT Record 0",
        )
        .into());
    }

    // 3. Parse Attributes and Data Runs of $MFT Record 0 to map the MFT sequential payload
    let mft_zero_attrs = parse_attributes(&mft_record_zero);
    let mut mft_runs = Vec::new();
    let mut mft_allocated_len = 0u64;

    for attr in mft_zero_attrs {
        if attr.ty == 0x80 && attr.is_non_resident {
            let data_runs_offset =
                u16::from_le_bytes(attr.payload[32..34].try_into().unwrap_or([0; 2])) as usize;
            let run_bytes = &attr.payload[data_runs_offset..];
            mft_runs = decode_data_runs(run_bytes);
            mft_allocated_len = attr.value_length;
            break;
        }
    }

    if mft_runs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not resolve $MFT sequential data runs map",
        )
        .into());
    }

    let max_records = mft_allocated_len / MFT_RECORD_SIZE as u64;
    let mut mft_entries: Vec<Option<MftEntry>> = Vec::new();
    mft_entries.resize_with(max_records as usize, || None);

    let mut children_map: HashMap<u64, Vec<u64>, ahash::RandomState> =
        HashMap::with_capacity_and_hasher(max_records as usize / 10, ahash::RandomState::new());

    // Allocate a large reusable staging buffer for chunked ingestion
    let mut run_buffer = vec![0u8; CHUNK_SIZE];
    let mut current_record_id = 0u64;

    // --- Pass 1: Linear ingest phase - completely in-memory, zero random seeks ---
    for run in mft_runs {
        let length_bytes = run.length_clusters * cluster_size;
        let mut bytes_left = length_bytes;

        if let Some(lcn) = run.lcn {
            let start_byte = lcn as u64 * cluster_size;
            raw_disk.seek(SeekFrom::Start(start_byte))?;

            while bytes_left > 0 && current_record_id < max_records {
                let to_read = (bytes_left as usize).min(CHUNK_SIZE);
                let active_slice = &mut run_buffer[..to_read];

                // Graceful reading: If raw partition bounds are exceeded, break instead of failing the scan.
                if raw_disk.read_exact(active_slice).is_err() {
                    break;
                }

                let mut offset = 0;
                while offset + MFT_RECORD_SIZE <= to_read && current_record_id < max_records {
                    let record_buffer = &mut run_buffer[offset..offset + MFT_RECORD_SIZE];
                    offset += MFT_RECORD_SIZE;

                    let record_id = current_record_id;
                    current_record_id += 1;

                    if !apply_fixup(record_buffer) {
                        continue; // Skip corrupted record signatures
                    }

                    let flags =
                        u16::from_le_bytes(record_buffer[22..24].try_into().unwrap_or([0; 2]));
                    // Skip deleted/inactive records immediately
                    if (flags & 1) == 0 {
                        continue;
                    }

                    // Read base record reference from MFT Record Header (Offset 32)
                    let base_file_ref =
                        u64::from_le_bytes(record_buffer[32..40].try_into().unwrap_or([0; 8]));
                    let base_record_id = base_file_ref & 0x0000_ffff_ffff_ffff;

                    if base_record_id == 0 {
                        let extracted_links = extract_all_links_from_record(record_buffer);
                        let is_dir = (flags & 2) != 0;

                        // Extract timestamps from standard information (always resident, always first)
                        let mut modified = 0i64;
                        let mut created = 0i64;
                        let mut accessed = 0i64;

                        let attrs = parse_attributes(record_buffer);
                        for attr in &attrs {
                            if attr.ty == 0x10 && !attr.is_non_resident && attr.payload.len() >= 24
                            {
                                let val_offset = u16::from_le_bytes(
                                    attr.payload[20..22].try_into().unwrap_or([0; 2]),
                                ) as usize;
                                if val_offset + 32 <= attr.payload.len() {
                                    let std_info = &attr.payload[val_offset..val_offset + 32];
                                    created = nt_time_to_unix(u64::from_le_bytes(
                                        std_info[0..8].try_into().unwrap_or([0; 8]),
                                    ));
                                    modified = nt_time_to_unix(u64::from_le_bytes(
                                        std_info[8..16].try_into().unwrap_or([0; 8]),
                                    ));
                                    accessed = nt_time_to_unix(u64::from_le_bytes(
                                        std_info[24..32].try_into().unwrap_or([0; 8]),
                                    ));
                                    break;
                                }
                            }
                        }

                        for (link_idx, link) in extracted_links.into_iter().enumerate() {
                            let size = if is_dir { 0 } else { link.size };

                            let target_id = if link_idx == 0 {
                                record_id
                            } else {
                                // Allocate a virtual MFT record ID for secondary links (hardlinks)
                                let virt_id = mft_entries.len() as u64;
                                mft_entries.push(None); // Grow container
                                virt_id
                            };

                            mft_entries[target_id as usize] = Some(MftEntry {
                                name: link.name,
                                size,
                                parent_record_id: link.parent_ref,
                                modified_timestamp: modified,
                                created_timestamp: created,
                                accessed_timestamp: accessed,
                                is_dir,
                                is_symlink: false,
                                has_attr_list: link.has_attr_list,
                            });

                            children_map
                                .entry(link.parent_ref)
                                .or_default()
                                .push(target_id);
                        }
                    }
                }

                bytes_left -= to_read as u64;

                // Update ingestion progress bar telemetry smoothly during sequential ingestion phase (Pass 1)
                if current_record_id.is_multiple_of(10000) {
                    stats
                        .files_scanned
                        .store(current_record_id as usize, Ordering::Relaxed);
                    stats.bytes_scanned.store(
                        (current_record_id * MFT_RECORD_SIZE as u64) as usize,
                        Ordering::Relaxed,
                    );
                }
            }
        } else {
            // Sparse data run - simply advance indices
            let sparse_records = (run.length_clusters * cluster_size) / MFT_RECORD_SIZE as u64;
            current_record_id += sparse_records;
        }
    }

    // Flush final Pass 1 ingestion progress metrics
    stats
        .files_scanned
        .store(current_record_id as usize, Ordering::Relaxed);
    stats.bytes_scanned.store(
        (current_record_id * MFT_RECORD_SIZE as u64) as usize,
        Ordering::Relaxed,
    );

    // Resolve target subdirectory in-memory
    let target_record = find_target_record_in_memory(&mft_entries, &children_map, root_path);

    // --- Reset Atomics Before Starting Traversed-Tree Resolution (Pass 2) ---
    // Clears the Pass 1 ingestion counts so that the status bar exactly matches the root directory.
    stats.reset();

    // Transactional Local Event Buffer:
    // Prevents partial event flushing on MFT scanning failures.
    let mut buffered_events = Vec::with_capacity(1024 * 1024);

    // Hierarchical streaming phase
    let mut local_id_counter = 1u32;
    let mut visited = HashSet::with_capacity(max_records as usize / 5);
    visited.insert(target_record);

    let mut stack = vec![TraversalFrame {
        record_id: target_record,
        parent_local_id: LocalId(0),
    }];

    while let Some(frame) = stack.pop() {
        let Some(children) = children_map.get(&frame.record_id) else {
            continue;
        };

        for &child_record in children {
            if child_record as usize >= mft_entries.len() {
                continue;
            }
            let Some(entry) = &mft_entries[child_record as usize] else {
                continue;
            };

            if entry.is_dir {
                if !visited.insert(child_record) {
                    continue;
                }

                let child_local_id = LocalId(local_id_counter);
                local_id_counter += 1;

                // Safely update traversed directory atomic counters
                stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

                buffered_events.push(ScanEvent::DirDiscovered {
                    parent_worker_id: 0,
                    child_worker_id: 0,
                    local_parent_id: frame.parent_local_id,
                    local_child_id: child_local_id,
                    name: entry.name.clone(),
                    modified_timestamp: entry.modified_timestamp,
                    created_timestamp: entry.created_timestamp,
                    accessed_timestamp: entry.accessed_timestamp,
                });

                stack.push(TraversalFrame {
                    record_id: child_record,
                    parent_local_id: child_local_id,
                });
            } else {
                // Resolve true sizes for heavily fragmented or locked files on-demand.
                // Triggers an OS query ONLY for files containing an Attribute List but reporting 0 size.
                let mut file_size = entry.size;
                if file_size == 0 && entry.has_attr_list {
                    let full_path =
                        reconstruct_path(&mft_entries, child_record, target_record, root_path);
                    if let Ok(meta) = std::fs::metadata(&full_path) {
                        file_size = meta.len();
                    }
                }

                // Safely update traversed file and logical size atomic counters exactly once per file
                stats.files_scanned.fetch_add(1, Ordering::Relaxed);
                stats
                    .bytes_scanned
                    .fetch_add(file_size as usize, Ordering::Relaxed);

                buffered_events.push(ScanEvent::FileDiscovered {
                    parent_worker_id: 0,
                    local_parent_id: frame.parent_local_id,
                    name: entry.name.clone(),
                    size: file_size,
                    is_symlink: entry.is_symlink,
                    modified_timestamp: entry.modified_timestamp,
                    created_timestamp: entry.created_timestamp,
                    accessed_timestamp: entry.accessed_timestamp,
                });
            }
        }
    }

    // --- Flush Transactional Event Buffer to the Coordinator ---
    let mut batch = Vec::with_capacity(1024);
    for event in buffered_events {
        batch.push(event);
        if batch.len() >= 1024 {
            let _ = event_tx.send(std::mem::replace(&mut batch, Vec::with_capacity(1024)));
        }
    }
    if !batch.is_empty() {
        let _ = event_tx.send(batch);
    }

    Ok(())
}

/// Resolves the Windows partition volume path from a standard file path
fn get_volume_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let mut chars = path_str.chars();
    if let Some(drive) = chars.next()
        && chars.next() == Some(':')
        && drive.is_ascii_alphabetic()
    {
        Some(format!("\\\\.\\{}:", drive.to_ascii_uppercase()))
    } else {
        None
    }
}

/// Checks the root of the specified directory path to verify if the file system is NTFS.
#[must_use]
pub fn get_fs_type(path: &Path) -> Option<String> {
    let root_path = path.ancestors().last()?;
    let mut root_w: Vec<u16> = root_path.as_os_str().encode_wide().collect();
    if !root_w.ends_with(&[0]) {
        root_w.push(0);
    }

    let mut fs_name_buf = [0u16; 256];

    let success = unsafe {
        GetVolumeInformationW(
            PCWSTR::from_raw(root_w.as_ptr()),
            None,
            None,
            None,
            None,
            Some(&mut fs_name_buf),
        )
    };

    if success.is_ok() {
        let len = fs_name_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(fs_name_buf.len());
        String::from_utf16(&fs_name_buf[..len]).ok()
    } else {
        None
    }
}
