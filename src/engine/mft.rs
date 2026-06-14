use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
};

#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt as _;

use compact_str::CompactString;
use crossbeam::channel::{Sender, bounded};
use smallvec::SmallVec;

#[cfg(target_os = "windows")]
use windows::{Win32::Storage::FileSystem::GetVolumeInformationW, core::PCWSTR};

use super::traversal::{LocalId, ScanEvent, TraversalStats};

const MFT_RECORD_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16MB Reusable Staging Buffer

// Windows kernel storage flags for raw sector-aligned direct I/O
#[cfg(target_os = "windows")]
const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
#[cfg(target_os = "windows")]
const FILE_FLAG_SEQUENTIAL_SCAN: u32 = 0x0800_0000;

/// Strictly aligned 4096-byte memory page required to satisfy Windows Direct I/O (DMA) alignment
#[repr(C, align(4096))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct AlignedPage {
    data: [u8; 4096],
}

#[repr(C, align(8))]
struct MftEntry {
    size: u64,
    parent_record_id: u64,
    modified_timestamp: i64,
    created_timestamp: i64,
    accessed_timestamp: i64,
    name_id: u32,
    flags: u8, // bit 0: is_dir, bit 1: is_symlink, bit 2: has_attr_list
    _padding: [u8; 3],
}

struct TraversalFrame {
    record_id: u64,
    parent_local_id: LocalId,
}

#[derive(Clone)]
struct DataRun {
    length_clusters: u64,
    lcn: Option<i64>, // None means sparse
}

#[derive(Clone)]
struct ExtractedLink {
    parent_ref: u64,
    name: CompactString,
    size: u64,
    has_attr_list: bool,
}

struct IngestionChunk {
    buffer: Vec<AlignedPage>,
    start_record_id: u64,
    bytes_read: usize,
}

struct TlCacheEntry {
    hash: u64,
    val: CompactString,
    id: crate::arena::StringId,
}

thread_local! {
    static TL_INTERN_CACHE: std::cell::RefCell<[Option<TlCacheEntry>; 2048]> = const { std::cell::RefCell::new(
        const { [const { None }; 2048] }
    ) };
}

/// Sharded parallel interner utilizing `RwLock` with a Thread-Local L1 Cache for lock-free resolutions
pub struct ShardedStringPool {
    shards: Vec<parking_lot::RwLock<crate::arena::StringPool>>,
}

impl ShardedStringPool {
    fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(parking_lot::RwLock::new(crate::arena::StringPool::new()));
        }
        Self { shards }
    }

    #[inline]
    fn get_or_insert(&self, s_bytes: &[u8]) -> crate::arena::StringId {
        let hash = crc32_hash(s_bytes);
        let cache_idx = (hash % 2048) as usize;

        // 1. Thread-Local L1 Cache Check (Zero locks, zero atomic instructions)
        let hit = TL_INTERN_CACHE.with(|cache| {
            let cache_ref = cache.borrow();
            if let Some(entry) = &cache_ref[cache_idx]
                && entry.hash == hash
                && entry.val.as_bytes() == s_bytes
            {
                return Some(entry.id);
            }
            None
        });

        if let Some(id) = hit {
            return id;
        }

        // 2. Fallback to Global Shard Lookup
        let id = self.global_get_or_insert(s_bytes, hash);

        // 3. Update Thread-Local L1 Cache on Miss
        if let Ok(s_str) = std::str::from_utf8(s_bytes) {
            TL_INTERN_CACHE.with(|cache| {
                let mut cache_mut = cache.borrow_mut();
                cache_mut[cache_idx] = Some(TlCacheEntry {
                    hash,
                    val: CompactString::new(s_str),
                    id,
                });
            });
        }

        id
    }

    #[inline]
    fn global_get_or_insert(&self, s_bytes: &[u8], hash: u64) -> crate::arena::StringId {
        let shard_idx = (hash % self.shards.len() as u64) as usize;
        let s_str = std::str::from_utf8(s_bytes).unwrap_or("");

        {
            let shard_read = self.shards[shard_idx].read();
            if let Ok(Some(local_id)) = shard_read.interner.lookup_handle(s_str) {
                let global_id = ((shard_idx as u32) << 24) | local_id;
                return crate::arena::StringId(global_id);
            }
        }

        let local_id = self.shards[shard_idx].write().get_or_insert(s_bytes);
        let global_id = ((shard_idx as u32) << 24) | local_id.0;
        crate::arena::StringId(global_id)
    }

    fn get_str(&self, global_id: u32) -> Option<String> {
        let shard_idx = (global_id >> 24) as usize;
        let local_id = global_id & 0x00FF_FFFF;
        if shard_idx < self.shards.len() {
            let shard = self.shards[shard_idx].read();
            shard
                .get(crate::arena::StringId(local_id))
                .map(std::string::ToString::to_string)
        } else {
            None
        }
    }
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

/// Portably computes a fast CRC32 hash utilizing hardware acceleration
/// across x86, ARM, and software fallbacks.
#[inline]
fn crc32_hash(bytes: &[u8]) -> u64 {
    crc_fast::crc32_iso_hdlc(bytes) as u64
}

/// Dual-lane memory-safe UTF-16 to ASCII decoder utilizing SSE and AVX2 register channels
#[inline]
#[allow(clippy::cast_ptr_alignment)]
fn decode_utf16_simd(name_raw: &[u8]) -> Option<CompactString> {
    #[cfg(target_arch = "x86_64")]
    {
        let byte_len = name_raw.len();
        let len = byte_len / 2;

        if len == 0 {
            return None;
        }

        if is_x86_feature_detected!("avx2") {
            if byte_len >= 32 && len <= 16 {
                // 256-bit AVX2 registers for medium-length names
                unsafe {
                    use std::arch::x86_64::{
                        __m256i, _mm256_and_si256, _mm256_cmpeq_epi16, _mm256_loadu_si256,
                        _mm256_movemask_epi8, _mm256_packus_epi16, _mm256_permute4x64_epi64,
                        _mm256_set1_epi16, _mm256_setzero_si256, _mm256_storeu_si256,
                    };
                    let vec = _mm256_loadu_si256(name_raw.as_ptr().cast::<__m256i>());
                    let mask = _mm256_set1_epi16(0xFF00u16 as i16);
                    let test = _mm256_and_si256(vec, mask);

                    let zero = _mm256_setzero_si256();
                    let cmp = _mm256_cmpeq_epi16(test, zero);

                    let move_mask = _mm256_movemask_epi8(cmp);
                    if move_mask == -1 {
                        let packed = _mm256_packus_epi16(vec, zero);
                        let permuted = _mm256_permute4x64_epi64(packed, 0xD8);

                        let mut ascii_buf = [0u8; 32];
                        _mm256_storeu_si256(ascii_buf.as_mut_ptr().cast::<__m256i>(), permuted);

                        return Some(CompactString::new(std::str::from_utf8_unchecked(
                            &ascii_buf[..len],
                        )));
                    }
                }
            } else if byte_len >= 16 && len <= 8 {
                // 128-bit SSE registers to process short file names safely
                unsafe {
                    use std::arch::x86_64::{
                        __m128i, _mm_and_si128, _mm_cmpeq_epi16, _mm_loadu_si128,
                        _mm_movemask_epi8, _mm_packus_epi16, _mm_set1_epi16, _mm_setzero_si128,
                        _mm_storeu_si128,
                    };
                    let vec = _mm_loadu_si128(name_raw.as_ptr().cast::<__m128i>());
                    let mask = _mm_set1_epi16(0xFF00u16 as i16);
                    let test = _mm_and_si128(vec, mask);

                    let zero = _mm_setzero_si128();
                    let cmp = _mm_cmpeq_epi16(test, zero);

                    let move_mask = _mm_movemask_epi8(cmp);
                    if move_mask == 0xFFFF {
                        let packed = _mm_packus_epi16(vec, zero);

                        let mut ascii_buf = [0u8; 16];
                        _mm_storeu_si128(ascii_buf.as_mut_ptr().cast::<__m128i>(), packed);

                        return Some(CompactString::new(std::str::from_utf8_unchecked(
                            &ascii_buf[..len],
                        )));
                    }
                }
            }
        }
    }
    None
}

/// Decodes UTF-16 filenames using AVX2/SSE SIMD registers as the primary path,
/// falling back to scalar stack buffers and wide conversion only if wide characters are present.
#[inline]
fn decode_utf16_name_to_compact_string(name_raw: &[u8]) -> CompactString {
    let len = name_raw.len() / 2;
    if len == 0 {
        return CompactString::new("");
    }

    // 1. Explicit SIMD Fast Paths (Safe SSE/AVX2)
    if let Some(s) = decode_utf16_simd(name_raw) {
        return s;
    }

    // 2. Fallback Scalar ASCII path (Avoiding any heap allocation for short ASCII names)
    if len <= 64 {
        let mut ascii_buf = [0u8; 64];
        let mut is_ascii = true;
        for i in 0..len {
            let c1 = name_raw[i * 2];
            let c2 = name_raw[i * 2 + 1];
            if c2 != 0 {
                is_ascii = false;
                break;
            }
            ascii_buf[i] = c1;
        }
        if is_ascii && let Ok(ascii_str) = std::str::from_utf8(&ascii_buf[..len]) {
            return CompactString::new(ascii_str);
        }
    }

    // 3. Fallback wide/Unicode path
    let u16_chars: Vec<u16> = name_raw
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    String::from_utf16(&u16_chars).map_or_else(|_| CompactString::new(""), CompactString::new)
}

/// Applies the NTFS Sector Update Sequence Array (USA) fixup in-place to guarantee sector integrity.
fn apply_fixup(buffer: &mut [u8]) -> bool {
    if buffer.len() < 24 {
        return false;
    }
    if &buffer[0..4] != b"FILE" && &buffer[0..4] != b"INDX" {
        return false;
    }
    let update_seq_offset = u16::from_le_bytes([buffer[4], buffer[5]]) as usize;
    let update_seq_count = u16::from_le_bytes([buffer[6], buffer[7]]) as usize;

    // Hardened bounds checks to prevent overflows and out-of-bounds indexing
    if update_seq_count == 0 || update_seq_offset + 2 > buffer.len() {
        return false;
    }
    if update_seq_offset + update_seq_count * 2 > buffer.len() {
        return false;
    }

    let usn = [buffer[update_seq_offset], buffer[update_seq_offset + 1]];
    let mut array_pos = update_seq_offset + 2;
    let mut sector_pos = 512 - 2;

    for _ in 1..update_seq_count {
        if sector_pos + 2 > buffer.len() || array_pos + 2 > buffer.len() {
            return false;
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

        // Hardening against integer-overflow on stack allocation sizes
        if len_bytes_count > 8 || offset_bytes_count > 8 {
            break;
        }
        if len_bytes_count == 0 && offset_bytes_count == 0 {
            break; // Avoid infinite loops on malformed run headers
        }

        if data_runs_bytes.len() < len_bytes_count + offset_bytes_count {
            break;
        }

        let mut len_buf = [0u8; 8];
        if let Some(slice) = data_runs_bytes.get(..len_bytes_count) {
            len_buf[..len_bytes_count].copy_from_slice(slice);
        } else {
            break;
        }
        let length_clusters = u64::from_le_bytes(len_buf);
        data_runs_bytes = &data_runs_bytes[len_bytes_count..];

        let lcn = if offset_bytes_count > 0 {
            let mut off_buf = [0u8; 8];
            if let Some(slice) = data_runs_bytes.get(..offset_bytes_count) {
                off_buf[..offset_bytes_count].copy_from_slice(slice);
            } else {
                break;
            }
            let mut offset = i64::from_le_bytes(off_buf);
            let unused_bits = (8 - offset_bytes_count) * 8;
            offset = (offset << unused_bits) >> unused_bits;
            data_runs_bytes = &data_runs_bytes[offset_bytes_count..];

            let current_lcn = previous_lcn.wrapping_add(offset);
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
fn parse_attributes(record_data: &[u8]) -> SmallVec<[AttributeHeader<'_>; 6]> {
    let mut attrs = SmallVec::new();
    if record_data.len() < 24 {
        return attrs;
    }
    let first_attr_offset = u16::from_le_bytes([record_data[20], record_data[21]]) as usize;

    if first_attr_offset >= record_data.len() {
        return attrs;
    }
    let mut offset = first_attr_offset;

    while offset + 8 <= record_data.len() {
        let ty = u32::from_le_bytes([
            record_data[offset],
            record_data[offset + 1],
            record_data[offset + 2],
            record_data[offset + 3],
        ]);
        if ty == 0xFFFF_FFFF {
            break;
        }
        let length = u32::from_le_bytes([
            record_data[offset + 4],
            record_data[offset + 5],
            record_data[offset + 6],
            record_data[offset + 7],
        ]) as usize;

        // Hardening: Verify attributes are at least 16 bytes and do not extend past file bounds
        if length < 16 || offset + length > record_data.len() {
            break;
        }

        let is_non_resident = record_data[offset + 8] != 0;
        let name_length = record_data[offset + 9] as usize;

        let value_length = if is_non_resident {
            if length >= 56 {
                u64::from_le_bytes([
                    record_data[offset + 48],
                    record_data[offset + 49],
                    record_data[offset + 50],
                    record_data[offset + 51],
                    record_data[offset + 52],
                    record_data[offset + 53],
                    record_data[offset + 54],
                    record_data[offset + 55],
                ])
            } else {
                0
            }
        } else if length >= 20 {
            u32::from_le_bytes([
                record_data[offset + 16],
                record_data[offset + 17],
                record_data[offset + 18],
                record_data[offset + 19],
            ]) as u64
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

/// Extracts standard info timestamps (created, modified, accessed) from an attribute payload.
#[inline]
fn parse_standard_information_timestamps(payload: &[u8]) -> Option<(i64, i64, i64)> {
    if payload.len() < 24 {
        return None;
    }
    let val_offset = u16::from_le_bytes([payload[20], payload[21]]) as usize;
    if val_offset + 32 <= payload.len() {
        let std_info = &payload[val_offset..val_offset + 32];
        let created = nt_time_to_unix(u64::from_le_bytes([
            std_info[0],
            std_info[1],
            std_info[2],
            std_info[3],
            std_info[4],
            std_info[5],
            std_info[6],
            std_info[7],
        ]));
        let modified = nt_time_to_unix(u64::from_le_bytes([
            std_info[8],
            std_info[9],
            std_info[10],
            std_info[11],
            std_info[12],
            std_info[13],
            std_info[14],
            std_info[15],
        ]));
        let accessed = nt_time_to_unix(u64::from_le_bytes([
            std_info[24],
            std_info[25],
            std_info[26],
            std_info[27],
            std_info[28],
            std_info[29],
            std_info[30],
            std_info[31],
        ]));
        Some((created, modified, accessed))
    } else {
        None
    }
}

/// Extracts all unique directory links from a file record, deduplicating local namespace variants.
fn extract_all_links_from_record(attrs: &[AttributeHeader<'_>]) -> SmallVec<[ExtractedLink; 1]> {
    let mut links = SmallVec::<[(u64, CompactString, i32); 2]>::new();
    let mut unnamed_data_size = None;
    let mut has_attr_list = false;
    let mut fallback_size = 0u64;

    for attr in attrs {
        match attr.ty {
            0x30 if !attr.is_non_resident && attr.payload.len() >= 24 => {
                // Resident FileName Attribute
                let val_offset = u16::from_le_bytes([attr.payload[20], attr.payload[21]]) as usize;
                let val_len = u32::from_le_bytes([
                    attr.payload[16],
                    attr.payload[17],
                    attr.payload[18],
                    attr.payload[19],
                ]) as usize;

                if val_offset + val_len <= attr.payload.len() && val_len >= 66 {
                    let val = &attr.payload[val_offset..val_offset + val_len];

                    let parent_ref = u64::from_le_bytes([
                        val[0], val[1], val[2], val[3], val[4], val[5], val[6], val[7],
                    ]) & 0x0000_ffff_ffff_ffff;

                    let namespace = val[65];
                    let prio = match namespace {
                        1 => 3, // Win32
                        3 => 2, // Win32AndDos
                        2 => 1, // Dos
                        _ => 0, // Posix
                    };

                    let name_len = val[64] as usize;
                    if 66 + name_len * 2 <= val.len() {
                        let mut existing_idx = None;
                        for (idx, item) in links.iter().enumerate() {
                            if item.0 == parent_ref {
                                existing_idx = Some(idx);
                                break;
                            }
                        }

                        let should_decode = existing_idx.is_none_or(|idx| prio > links[idx].2);

                        if should_decode && let Some(name_raw) = val.get(66..66 + name_len * 2) {
                            let compact_name = decode_utf16_name_to_compact_string(name_raw);
                            if let Some(idx) = existing_idx {
                                links[idx] = (parent_ref, compact_name, prio);
                            } else {
                                links.push((parent_ref, compact_name, prio));
                            }
                        }
                    }

                    if fallback_size == 0 {
                        fallback_size = u64::from_le_bytes([
                            val[48], val[49], val[50], val[51], val[52], val[53], val[54], val[55],
                        ]);
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

    let mut out = SmallVec::new();
    for (parent_ref, name, _) in links {
        out.push(ExtractedLink {
            parent_ref,
            name,
            size: actual_size,
            has_attr_list,
        });
    }
    out
}

/// Reconstructs the absolute path of an MFT record purely in-memory by walking up parent references.
fn reconstruct_path(
    mft_entries: &[Option<MftEntry>],
    record_id: u64,
    root_record_id: u64,
    root_path: &Path,
    sharded_pool: &ShardedStringPool,
) -> PathBuf {
    let mut path = PathBuf::new();
    let mut curr = record_id;

    // Use stack-allocated SmallVec arrays instead of heap-allocated Vec and HashSet
    let mut segments = SmallVec::<[String; 16]>::new();
    let mut visited = SmallVec::<[u64; 16]>::new();

    while curr != root_record_id {
        // Prevent infinite loops on corrupted/circular directory reference networks on disk
        if visited.contains(&curr) {
            break;
        }
        visited.push(curr);

        if let Some(Some(entry)) = mft_entries.get(curr as usize) {
            if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                segments.push(name_str);
            }
            curr = entry.parent_record_id;
        } else {
            break;
        }
    }

    path.push(root_path);
    for segment in segments.into_iter().rev() {
        path.push(&segment);
    }
    path
}

/// Sweeps the in-memory tree starting from the Root record to locate the target directory using flat links.
fn find_target_record_in_memory_flat(
    mft_entries: &[Option<MftEntry>],
    first_child: &[u32],
    next_sibling: &[u32],
    root_path: &Path,
    sharded_pool: &ShardedStringPool,
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
        let mut child_record = first_child[current_record as usize];
        while child_record != crate::arena::NO_INDEX {
            if let Some(Some(entry)) = mft_entries.get(child_record as usize)
                && entry.flags & 1 != 0
            {
                // is_dir check
                if let Some(name_str) = sharded_pool.get_str(entry.name_id)
                    && name_str.to_ascii_lowercase() == segment
                {
                    current_record = child_record as u64;
                    found = true;
                    break;
                }
            }
            child_record = next_sibling[child_record as usize];
        }
        if !found {
            return 5; // Fallback to root directory if the nested hierarchy cannot be mapped
        }
    }
    current_record
}

/// Helper function to parse an exported, raw $MFT metadata file sequentially on any platform.
fn scan_mft_file_sequential(
    file_path: &Path,
    event_tx: &Sender<Vec<ScanEvent>>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    let file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    let max_records = file_len / MFT_RECORD_SIZE as u64;

    if max_records == 0 || max_records > 50_000_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MFT file size descriptor is invalid or excessively large",
        )
        .into());
    }

    let mut mft_entries: Vec<Option<MftEntry>> = Vec::new();
    mft_entries.resize_with(max_records as usize, || None);

    // Spawns a background thread to read file chunks sequentially
    let (filled_tx, filled_rx) = bounded::<IngestionChunk>(4);
    let mut file_clone = file.try_clone()?;
    let pages_per_chunk = CHUNK_SIZE / 4096;

    let (empty_tx, empty_rx) = bounded::<Vec<AlignedPage>>(4);
    for _ in 0..4 {
        let _ = empty_tx.send(vec![AlignedPage { data: [0; 4096] }; pages_per_chunk]);
    }

    std::thread::spawn(move || {
        let mut record_id_counter = 0u64;

        while let Ok(mut chunk_buffer) = empty_rx.recv() {
            let active_bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut chunk_buffer);
            match file_clone.read(active_bytes) {
                Ok(n) if n > 0 => {
                    let chunk = IngestionChunk {
                        buffer: chunk_buffer,
                        start_record_id: record_id_counter,
                        bytes_read: n,
                    };
                    record_id_counter += (n / MFT_RECORD_SIZE) as u64;
                    if filled_tx.send(chunk).is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    let sharded_pool = Arc::new(ShardedStringPool::new(16));
    let side_channel_links = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Consume chunks and parse records concurrently using Rayon threads
    rayon::scope(|scope| {
        let mft_entries_ptr = mft_entries.as_mut_ptr() as usize;

        while let Ok(chunk) = filled_rx.recv() {
            let records_count = chunk.bytes_read / MFT_RECORD_SIZE;
            let start_idx = chunk.start_record_id as usize;

            let side_channel_links_clone = side_channel_links.clone();
            let sharded_pool_clone = sharded_pool.clone();
            let empty_tx_clone = empty_tx.clone();

            scope.spawn(move |_| {
                // Assert safe slice bounds to prevent any possible memory overflow
                let safe_records_count = records_count.min(max_records as usize - start_idx);
                let target_slice = unsafe {
                    std::slice::from_raw_parts_mut(
                        (mft_entries_ptr as *mut Option<MftEntry>).add(start_idx),
                        safe_records_count,
                    )
                };

                let mut chunk = chunk;
                let chunk_bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut chunk.buffer);
                let mut local_links = Vec::new();

                for (i, entry_slot) in target_slice.iter_mut().enumerate() {
                    let offset = i * MFT_RECORD_SIZE;
                    if offset + MFT_RECORD_SIZE <= chunk.bytes_read {
                        let record_buffer = &mut chunk_bytes[offset..offset + MFT_RECORD_SIZE];

                        // Match raw pointer verification safely via standard arrays
                        if record_buffer[..4] == *b"FILE" && apply_fixup(record_buffer) {
                            let flags = u16::from_le_bytes([record_buffer[22], record_buffer[23]]);
                            if (flags & 1) != 0 {
                                let base_file_ref = u64::from_le_bytes([
                                    record_buffer[32],
                                    record_buffer[33],
                                    record_buffer[34],
                                    record_buffer[35],
                                    record_buffer[36],
                                    record_buffer[37],
                                    record_buffer[38],
                                    record_buffer[39],
                                ]);
                                let base_record_id = base_file_ref & 0x0000_ffff_ffff_ffff;

                                if base_record_id == 0 {
                                    let attrs = parse_attributes(record_buffer);
                                    let extracted_links = extract_all_links_from_record(&attrs);
                                    let is_dir = (flags & 2) != 0;

                                    let mut modified = 0i64;
                                    let mut created = 0i64;
                                    let mut accessed = 0i64;

                                    for attr in &attrs {
                                        if attr.ty == 0x10
                                            && !attr.is_non_resident
                                            && let Some((cre, mod_t, acc)) =
                                                parse_standard_information_timestamps(attr.payload)
                                        {
                                            created = cre;
                                            modified = mod_t;
                                            accessed = acc;
                                            break;
                                        }
                                    }

                                    if let Some(first) = extracted_links.first() {
                                        let name_id =
                                            sharded_pool_clone.get_or_insert(first.name.as_bytes());
                                        let size = if is_dir { 0 } else { first.size };

                                        let mut entry_flags = 0u8;
                                        if is_dir {
                                            entry_flags |= 1;
                                        }
                                        if first.has_attr_list {
                                            entry_flags |= 4;
                                        }

                                        *entry_slot = Some(MftEntry {
                                            size,
                                            parent_record_id: first.parent_ref,
                                            modified_timestamp: modified,
                                            created_timestamp: created,
                                            accessed_timestamp: accessed,
                                            name_id: name_id.0,
                                            flags: entry_flags,
                                            _padding: [0; 3],
                                        });

                                        if extracted_links.len() > 1 {
                                            local_links.extend(extracted_links[1..].to_vec());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !local_links.is_empty() {
                    side_channel_links_clone.lock().extend(local_links);
                }
                let _ = empty_tx_clone.send(chunk.buffer);
            });

            let current_id = start_idx + records_count;
            if current_id.is_multiple_of(20000) {
                stats.files_scanned.store(current_id, Ordering::Relaxed);
                stats
                    .bytes_scanned
                    .store(current_id * MFT_RECORD_SIZE, Ordering::Relaxed);
            }
        }
    });

    // 1. Compile the total size of our structural mapping array (including secondary hardlinks)
    let total_entries = mft_entries.len() + side_channel_links.lock().len();
    let mut first_child = vec![crate::arena::NO_INDEX; total_entries];
    let mut next_sibling = vec![crate::arena::NO_INDEX; total_entries];

    // 2. Build structural child mappings sequentially on the main thread for the base entries
    for record_id in 0..mft_entries.len() {
        if let Some(entry) = &mft_entries[record_id] {
            let parent_id = entry.parent_record_id as usize;
            if parent_id < total_entries {
                next_sibling[record_id] = first_child[parent_id];
                first_child[parent_id] = record_id as u32;
            }
        }
    }

    // 3. Append secondary virtual links (hardlinks) safely on the single main thread and build structural sibling links
    for link in side_channel_links.lock().drain(..) {
        let virt_id = mft_entries.len() as u64;
        let name_id = sharded_pool.get_or_insert(link.name.as_bytes());

        mft_entries.push(Some(MftEntry {
            size: link.size,
            parent_record_id: link.parent_ref,
            modified_timestamp: 0,
            created_timestamp: 0,
            accessed_timestamp: 0,
            name_id: name_id.0,
            flags: 0, // Hardlinks are files
            _padding: [0; 3],
        }));

        let parent_id = link.parent_ref as usize;
        if parent_id < total_entries {
            next_sibling[virt_id as usize] = first_child[parent_id];
            first_child[parent_id] = virt_id as u32;
        }
    }

    let target_record = find_target_record_in_memory_flat(
        &mft_entries,
        &first_child,
        &next_sibling,
        file_path,
        &sharded_pool,
    );
    stats.reset();

    // 4. Perform a Hierarchical Depth-First Traversal and stream event batches of 1024
    let mut local_id_counter = 1u32;
    let mut stack = vec![TraversalFrame {
        record_id: target_record,
        parent_local_id: LocalId(0),
    }];

    // Allocate a compact bit-vector (1 bit per entry)
    let mut visited = vec![0u64; mft_entries.len().div_ceil(64)];

    // Mark target_record as visited
    {
        let idx = target_record as usize;
        visited[idx / 64] |= 1u64 << (idx % 64);
    }

    let mut buffered_events = Vec::with_capacity(1024 * 1024);

    while let Some(frame) = stack.pop() {
        let mut child_record = first_child[frame.record_id as usize];
        while child_record != crate::arena::NO_INDEX {
            let child_record_idx = child_record as usize;
            if child_record_idx >= mft_entries.len() {
                child_record = next_sibling[child_record as usize];
                continue;
            }

            let is_dir = mft_entries[child_record_idx]
                .as_ref()
                .is_some_and(|e| e.flags & 1 != 0);

            let Some(entry) = mft_entries[child_record_idx].as_ref() else {
                child_record = next_sibling[child_record as usize];
                continue;
            };

            if is_dir {
                // Bitset check and mark
                let word_idx = child_record_idx / 64;
                let bit_idx = child_record_idx % 64;
                let bit_mask = 1u64 << bit_idx;

                if (visited[word_idx] & bit_mask) != 0 {
                    child_record = next_sibling[child_record as usize];
                    continue;
                }
                visited[word_idx] |= bit_mask;

                stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

                let child_local_id = LocalId(local_id_counter);
                local_id_counter = local_id_counter.saturating_add(1);

                if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                    buffered_events.push(ScanEvent::DirDiscovered {
                        parent_worker_id: 0,
                        child_worker_id: 0,
                        local_parent_id: frame.parent_local_id,
                        local_child_id: child_local_id,
                        name: CompactString::new(name_str),
                        modified_timestamp: entry.modified_timestamp,
                        created_timestamp: entry.created_timestamp,
                        accessed_timestamp: entry.accessed_timestamp,
                        no_permission: false,
                    });
                }

                stack.push(TraversalFrame {
                    record_id: child_record as u64,
                    parent_local_id: child_local_id,
                });
            } else {
                // Resolve sizes for placeholder system files containing attribute lists
                let mut actual_size = entry.size;
                if actual_size == 0 && (entry.flags & 4) != 0 {
                    let mut full_path = reconstruct_path(
                        &mft_entries,
                        entry.parent_record_id,
                        target_record,
                        file_path,
                        &sharded_pool,
                    );
                    if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                        full_path.push(&name_str);
                    }
                    if let Ok(meta) = std::fs::metadata(&full_path) {
                        actual_size = meta.len();
                    }
                }

                stats.files_scanned.fetch_add(1, Ordering::Relaxed);
                stats
                    .bytes_scanned
                    .fetch_add(actual_size as usize, Ordering::Relaxed);

                if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                    buffered_events.push(ScanEvent::FileDiscovered {
                        parent_worker_id: 0,
                        local_parent_id: frame.parent_local_id,
                        name: CompactString::new(name_str),
                        size: actual_size,
                        is_symlink: entry.flags & 2 != 0,
                        modified_timestamp: entry.modified_timestamp,
                        created_timestamp: entry.created_timestamp,
                        accessed_timestamp: entry.accessed_timestamp,
                        no_permission: false,
                    });
                }
            }

            child_record = next_sibling[child_record as usize];
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

/// Primary entry point for raw MFT scanning.
/// Parses either the raw drive partition on Windows or a standalone MFT copy on any platform.
pub fn try_scan_mft(
    root_path: &Path,
    event_tx: &Sender<Vec<ScanEvent>>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    if root_path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("$mft"))
    {
        return scan_mft_file_sequential(root_path, event_tx, stats);
    }

    let volume_path = get_volume_path(root_path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unable to determine partition drive path from target path",
        )
    })?;

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt as _;

        options.share_mode(7);
        // Direct non-buffered sequential I/O
        options.custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_SEQUENTIAL_SCAN);
    }

    let raw_disk = options.open(&volume_path)?;

    // 1. Read Boot Sector (Synchronously via raw sector read)
    let mut aligned_boot_buffer = vec![AlignedPage { data: [0; 4096] }; 1];
    let boot_sector: &mut [u8] = bytemuck::cast_slice_mut(&mut aligned_boot_buffer);

    // Perform raw drive sector read
    {
        let mut raw_disk_sync = OpenOptions::new();
        raw_disk_sync.read(true);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::OpenOptionsExt as _;

            raw_disk_sync.share_mode(7);
        }
        let mut f = raw_disk_sync.open(&volume_path)?;
        f.read_exact(&mut boot_sector[..512])?;
    }

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

    let cluster_size = sector_size
        .checked_mul(sectors_per_cluster)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Cluster size calculation overflowed",
            )
        })?;

    let mft_lcn = u64::from_le_bytes(boot_sector[0x30..0x38].try_into().unwrap_or([0; 8]));

    let mft_start_offset = mft_lcn.checked_mul(cluster_size).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MFT start offset calculation overflowed",
        )
    })?;

    // 2. Read MFT Record 0 ($MFT) (Synchronously via sector read)
    let mut aligned_mft_zero_buffer = vec![AlignedPage { data: [0; 4096] }; 1];
    let mft_record_zero: &mut [u8] = bytemuck::cast_slice_mut(&mut aligned_mft_zero_buffer);
    {
        let mut raw_disk_sync = OpenOptions::new();
        raw_disk_sync.read(true);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::OpenOptionsExt as _;

            raw_disk_sync.share_mode(7);
        }
        let mut f = raw_disk_sync.open(&volume_path)?;
        f.seek(SeekFrom::Start(mft_start_offset))?;
        f.read_exact(&mut mft_record_zero[..MFT_RECORD_SIZE])?;
    }

    if !apply_fixup(&mut mft_record_zero[..MFT_RECORD_SIZE]) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed record fixup for $MFT Record 0",
        )
        .into());
    }

    // 3. Parse Attributes and Data Runs of $MFT Record 0 to map the MFT sequential payload
    let mft_zero_attrs = parse_attributes(&mft_record_zero[..MFT_RECORD_SIZE]);
    let mut mft_runs = Vec::new();
    let mut mft_allocated_len = 0u64;

    for attr in mft_zero_attrs {
        if attr.ty == 0x80 && attr.is_non_resident {
            if attr.payload.len() < 34 {
                continue;
            }
            let data_runs_offset =
                u16::from_le_bytes(attr.payload[32..34].try_into().unwrap_or([0; 2])) as usize;
            if data_runs_offset < attr.payload.len() {
                let run_bytes = &attr.payload[data_runs_offset..];
                mft_runs = decode_data_runs(run_bytes);
                mft_allocated_len = attr.value_length;
            }
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

    // Hardening: Protect against OOM allocation panics on corrupted volume descriptors
    if max_records == 0 || max_records > 50_000_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MFT allocation length is invalid or excessively large",
        )
        .into());
    }

    let mut mft_entries: Vec<Option<MftEntry>> = Vec::new();
    mft_entries.resize_with(max_records as usize, || None);

    // Setup pipelined channels with aligned page slots
    let num_buffers = 4;
    let (empty_tx, empty_rx) = bounded::<Vec<AlignedPage>>(num_buffers);
    let (filled_tx, filled_rx) = bounded::<IngestionChunk>(num_buffers);

    let pages_per_chunk = CHUNK_SIZE / 4096;
    for _ in 0..num_buffers {
        let _ = empty_tx.send(vec![AlignedPage { data: [0; 4096] }; pages_per_chunk]);
    }

    let raw_disk_clone = raw_disk.try_clone()?;
    let mft_runs_clone = mft_runs;

    // Spawns background thread to ingest disk sectors sequentially. On Windows,
    // this utilizes sector-aligned direct unbuffered hardware reads.
    std::thread::spawn(move || {
        let mut raw_disk_clone = raw_disk_clone;
        let mut current_record_id = 0u64;
        let pages_per_chunk = CHUNK_SIZE / 4096;

        for run in mft_runs_clone {
            let length_bytes = run.length_clusters * cluster_size;
            let mut bytes_left = length_bytes;

            if let Some(lcn) = run.lcn {
                let start_byte = lcn as u64 * cluster_size;
                if raw_disk_clone.seek(SeekFrom::Start(start_byte)).is_ok() {
                    while bytes_left > 0 && current_record_id < max_records {
                        let actual_needed = (bytes_left as usize).min(CHUNK_SIZE);

                        // Windows Direct I/O Safeguard: Round up the read size
                        // to the nearest 4096-byte sector boundary.
                        let sector_aligned_read_size = (actual_needed + 4095) & !4095;

                        // Fetch or allocate an aligned direct-I/O buffer
                        let mut chunk_buffer = empty_rx.recv().unwrap_or_else(|_| {
                            vec![AlignedPage { data: [0; 4096] }; pages_per_chunk]
                        });

                        let active_bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut chunk_buffer);

                        // Read the rounded-up aligned size to satisfy Windows DMA rules
                        if raw_disk_clone
                            .read_exact(&mut active_bytes[..sector_aligned_read_size])
                            .is_err()
                        {
                            break;
                        }

                        // Send the chunk, instructing processing to only parse up to actual_needed
                        let chunk = IngestionChunk {
                            buffer: chunk_buffer,
                            start_record_id: current_record_id,
                            bytes_read: actual_needed,
                        };
                        current_record_id += (actual_needed / MFT_RECORD_SIZE) as u64;

                        if filled_tx.send(chunk).is_err() {
                            break;
                        }

                        bytes_left -= actual_needed as u64;
                    }
                }
            } else {
                let sparse_records = (run.length_clusters * cluster_size) / MFT_RECORD_SIZE as u64;
                current_record_id = current_record_id.saturating_add(sparse_records);
            }
        }
    });

    let sharded_pool = Arc::new(ShardedStringPool::new(16));
    let side_channel_links = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Parse records concurrently with Rayon
    rayon::scope(|scope| {
        let mft_entries_ptr = mft_entries.as_mut_ptr() as usize;

        while let Ok(chunk) = filled_rx.recv() {
            let records_count = chunk.bytes_read / MFT_RECORD_SIZE;
            let start_idx = chunk.start_record_id as usize;

            let side_channel_links_clone = side_channel_links.clone();
            let sharded_pool_clone = sharded_pool.clone();
            let empty_tx_clone = empty_tx.clone();

            scope.spawn(move |_| {
                // Assert safe slice bounds to prevent any possible memory overflow
                let safe_records_count = records_count.min(max_records as usize - start_idx);
                let target_slice = unsafe {
                    std::slice::from_raw_parts_mut(
                        (mft_entries_ptr as *mut Option<MftEntry>).add(start_idx),
                        safe_records_count,
                    )
                };

                let mut chunk = chunk;
                let chunk_bytes: &mut [u8] = bytemuck::cast_slice_mut(&mut chunk.buffer);

                for (i, entry_slot) in target_slice.iter_mut().enumerate() {
                    let offset = i * MFT_RECORD_SIZE;
                    if offset + MFT_RECORD_SIZE <= chunk.bytes_read {
                        let record_buffer = &mut chunk_bytes[offset..offset + MFT_RECORD_SIZE];

                        if record_buffer[..4] == *b"FILE" && apply_fixup(record_buffer) {
                            let flags = u16::from_le_bytes([record_buffer[22], record_buffer[23]]);
                            if (flags & 1) != 0 {
                                let base_file_ref = u64::from_le_bytes([
                                    record_buffer[32],
                                    record_buffer[33],
                                    record_buffer[34],
                                    record_buffer[35],
                                    record_buffer[36],
                                    record_buffer[37],
                                    record_buffer[38],
                                    record_buffer[39],
                                ]);
                                let base_record_id = base_file_ref & 0x0000_ffff_ffff_ffff;

                                if base_record_id == 0 {
                                    let attrs = parse_attributes(record_buffer);
                                    let extracted_links = extract_all_links_from_record(&attrs);
                                    let is_dir = (flags & 2) != 0;

                                    let mut modified = 0i64;
                                    let mut created = 0i64;
                                    let mut accessed = 0i64;

                                    for attr in &attrs {
                                        if attr.ty == 0x10
                                            && !attr.is_non_resident
                                            && let Some((cre, mod_t, acc)) =
                                                parse_standard_information_timestamps(attr.payload)
                                        {
                                            created = cre;
                                            modified = mod_t;
                                            accessed = acc;
                                            break;
                                        }
                                    }

                                    if let Some(first) = extracted_links.first() {
                                        let name_id =
                                            sharded_pool_clone.get_or_insert(first.name.as_bytes());
                                        let size = if is_dir { 0 } else { first.size };

                                        let mut entry_flags = 0u8;
                                        if is_dir {
                                            entry_flags |= 1;
                                        }
                                        if first.has_attr_list {
                                            entry_flags |= 4;
                                        }

                                        *entry_slot = Some(MftEntry {
                                            size,
                                            parent_record_id: first.parent_ref,
                                            modified_timestamp: modified,
                                            created_timestamp: created,
                                            accessed_timestamp: accessed,
                                            name_id: name_id.0,
                                            flags: entry_flags,
                                            _padding: [0; 3],
                                        });

                                        if extracted_links.len() > 1 {
                                            side_channel_links_clone
                                                .lock()
                                                .extend(extracted_links[1..].to_vec());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = empty_tx_clone.send(chunk.buffer);
            });

            let current_id = start_idx + records_count;
            if current_id.is_multiple_of(20000) {
                stats.files_scanned.store(current_id, Ordering::Relaxed);
                stats
                    .bytes_scanned
                    .store(current_id * MFT_RECORD_SIZE, Ordering::Relaxed);
            }
        }
    });

    // 1. Compile the total size of our structural mapping array (including secondary hardlinks)
    let total_entries = mft_entries.len() + side_channel_links.lock().len();
    let mut first_child = vec![crate::arena::NO_INDEX; total_entries];
    let mut next_sibling = vec![crate::arena::NO_INDEX; total_entries];

    // 2. Build structural child mappings sequentially on the main thread for the base entries
    for record_id in 0..mft_entries.len() {
        if let Some(entry) = &mft_entries[record_id] {
            let parent_id = entry.parent_record_id as usize;
            if parent_id < total_entries {
                next_sibling[record_id] = first_child[parent_id];
                first_child[parent_id] = record_id as u32;
            }
        }
    }

    // 3. Append secondary virtual links (hardlinks) safely on the single main thread and build structural sibling links
    for link in side_channel_links.lock().drain(..) {
        let virt_id = mft_entries.len() as u64;
        let name_id = sharded_pool.get_or_insert(link.name.as_bytes());

        mft_entries.push(Some(MftEntry {
            size: link.size,
            parent_record_id: link.parent_ref,
            modified_timestamp: 0,
            created_timestamp: 0,
            accessed_timestamp: 0,
            name_id: name_id.0,
            flags: 0, // Hardlinks are files
            _padding: [0; 3],
        }));

        let parent_id = link.parent_ref as usize;
        if parent_id < total_entries {
            next_sibling[virt_id as usize] = first_child[parent_id];
            first_child[parent_id] = virt_id as u32;
        }
    }

    let target_record = find_target_record_in_memory_flat(
        &mft_entries,
        &first_child,
        &next_sibling,
        root_path,
        &sharded_pool,
    );
    stats.reset();

    // 4. Perform a Hierarchical Depth-First Traversal and stream event batches of 1024
    let mut local_id_counter = 1u32;
    let mut stack = vec![TraversalFrame {
        record_id: target_record,
        parent_local_id: LocalId(0),
    }];

    // Allocate a compact bit-vector (1 bit per entry)
    let mut visited = vec![0u64; mft_entries.len().div_ceil(64)];

    // Mark target_record as visited
    {
        let idx = target_record as usize;
        visited[idx / 64] |= 1u64 << (idx % 64);
    }

    let mut buffered_events = Vec::with_capacity(1024 * 1024);

    while let Some(frame) = stack.pop() {
        let mut child_record = first_child[frame.record_id as usize];
        while child_record != crate::arena::NO_INDEX {
            let child_record_idx = child_record as usize;
            if child_record_idx >= mft_entries.len() {
                child_record = next_sibling[child_record as usize];
                continue;
            }

            let is_dir = mft_entries[child_record_idx]
                .as_ref()
                .is_some_and(|e| e.flags & 1 != 0);

            let Some(entry) = mft_entries[child_record_idx].as_ref() else {
                child_record = next_sibling[child_record as usize];
                continue;
            };

            if is_dir {
                // Bitset check and mark
                let word_idx = child_record_idx / 64;
                let bit_idx = child_record_idx % 64;
                let bit_mask = 1u64 << bit_idx;

                if (visited[word_idx] & bit_mask) != 0 {
                    child_record = next_sibling[child_record as usize];
                    continue;
                }
                visited[word_idx] |= bit_mask;

                stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

                let child_local_id = LocalId(local_id_counter);
                local_id_counter = local_id_counter.saturating_add(1);

                if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                    buffered_events.push(ScanEvent::DirDiscovered {
                        parent_worker_id: 0,
                        child_worker_id: 0,
                        local_parent_id: frame.parent_local_id,
                        local_child_id: child_local_id,
                        name: CompactString::new(name_str),
                        modified_timestamp: entry.modified_timestamp,
                        created_timestamp: entry.created_timestamp,
                        accessed_timestamp: entry.accessed_timestamp,
                        no_permission: false,
                    });
                }

                stack.push(TraversalFrame {
                    record_id: child_record as u64,
                    parent_local_id: child_local_id,
                });
            } else {
                // Resolve sizes for placeholder system files containing attribute lists
                let mut actual_size = entry.size;
                if actual_size == 0 && (entry.flags & 4) != 0 {
                    let mut full_path = reconstruct_path(
                        &mft_entries,
                        entry.parent_record_id,
                        target_record,
                        root_path,
                        &sharded_pool,
                    );
                    if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                        full_path.push(&name_str);
                    }
                    if let Ok(meta) = std::fs::metadata(&full_path) {
                        actual_size = meta.len();
                    }
                }

                stats.files_scanned.fetch_add(1, Ordering::Relaxed);
                stats
                    .bytes_scanned
                    .fetch_add(actual_size as usize, Ordering::Relaxed);

                if let Some(name_str) = sharded_pool.get_str(entry.name_id) {
                    buffered_events.push(ScanEvent::FileDiscovered {
                        parent_worker_id: 0,
                        local_parent_id: frame.parent_local_id,
                        name: CompactString::new(name_str),
                        size: actual_size,
                        is_symlink: entry.flags & 2 != 0,
                        modified_timestamp: entry.modified_timestamp,
                        created_timestamp: entry.created_timestamp,
                        accessed_timestamp: entry.accessed_timestamp,
                        no_permission: false,
                    });
                }
            }

            child_record = next_sibling[child_record as usize];
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
#[cfg(target_os = "windows")]
fn get_volume_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let trimmed = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    let mut chars = trimmed.chars();
    if let Some(drive) = chars.next()
        && chars.next() == Some(':')
        && drive.is_ascii_alphabetic()
    {
        Some(format!("\\\\.\\{}:", drive.to_ascii_uppercase()))
    } else {
        None
    }
}

/// Resolves partition paths on non-Windows targets.
#[cfg(not(target_os = "windows"))]
#[allow(clippy::unnecessary_wraps)]
fn get_volume_path(path: &Path) -> Option<String> {
    Some(path.to_string_lossy().into_owned())
}

/// Checks the root of the specified directory path to verify if the file system is NTFS.
#[cfg(target_os = "windows")]
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

/// Fallback fs type check for non-Windows systems.
#[cfg(not(target_os = "windows"))]
#[must_use]
pub const fn get_fs_type(_path: &Path) -> Option<String> {
    None
}
