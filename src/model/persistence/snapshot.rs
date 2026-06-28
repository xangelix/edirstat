use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};

use super::super::{
    arena::{FileNode, StringPool},
    varint::{
        read_i64_zigzag, read_u64_varint, u8_slice_to_u32_vec, write_i64_zigzag, write_u64_varint,
    },
};

pub const FILE_VERSION_V2: u16 = 2;
pub const FILE_VERSION_V3: u16 = 3;
pub const ZSTD_COMPRESSION_LEVEL: i32 = 3;

// Control byte bit flags for Columnar optimizations. Bits 0-2 live on `FileNode`
// (directory / symlink / no-permission); bits 3-4 are the timestamp shortcuts.
const FLAG_CREATED_EQ_MODIFIED: u8 = 1 << 3;
const FLAG_MODIFIED_EQ_PARENT: u8 = 1 << 4;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C, align(8))]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    _padding: u16,
    pub uncompressed_size: u64,
    pub node_count: u64,
    pub string_pool_offset: u64,
    pub string_pool_length: u64,
    pub reserved: [u64; 4], // 32 bytes of padding for future backward compatibility
}

impl FileHeader {
    /// Converts all numeric fields in the header to little-endian representation.
    /// On little-endian hosts, this compiles to an empty operation (no-op).
    #[must_use]
    pub const fn to_le(self) -> Self {
        Self {
            magic: self.magic,
            version: self.version.to_le(),
            _padding: self._padding.to_le(),
            uncompressed_size: self.uncompressed_size.to_le(),
            node_count: self.node_count.to_le(),
            string_pool_offset: self.string_pool_offset.to_le(),
            string_pool_length: self.string_pool_length.to_le(),
            reserved: [
                self.reserved[0].to_le(),
                self.reserved[1].to_le(),
                self.reserved[2].to_le(),
                self.reserved[3].to_le(),
            ],
        }
    }

    /// Converts little-endian fields in the header back to host-endian representation.
    #[must_use]
    pub const fn from_le(self) -> Self {
        Self {
            magic: self.magic,
            version: u16::from_le(self.version),
            _padding: u16::from_le(self._padding),
            uncompressed_size: u64::from_le(self.uncompressed_size),
            node_count: u64::from_le(self.node_count),
            string_pool_offset: u64::from_le(self.string_pool_offset),
            string_pool_length: u64::from_le(self.string_pool_length),
            reserved: [
                u64::from_le(self.reserved[0]),
                u64::from_le(self.reserved[1]),
                u64::from_le(self.reserved[2]),
                u64::from_le(self.reserved[3]),
            ],
        }
    }
}

#[derive(Debug)]
pub struct PersistentArena {
    nodes: Vec<FileNode>,
}

impl PersistentArena {
    #[must_use]
    pub const fn new(nodes: Vec<FileNode>) -> Self {
        Self { nodes }
    }

    #[must_use]
    #[inline]
    pub fn nodes(&self) -> &[FileNode] {
        &self.nodes
    }

    #[inline]
    pub fn nodes_mut(&mut self) -> &mut [FileNode] {
        &mut self.nodes
    }
}

// =============================================================================
// Helper Binary Bit-Packing, Endian, & Varint Functions
// =============================================================================

/// On-disk size of a legacy Version-2 node record.
///
/// V2 was a raw memory dump of the pre-u32 `FileNode`: four `u32` pointers, a
/// `u64` size, **three `i64` timestamps** (modified, created, accessed), a `u32`
/// file count, a `u8` flags byte, and 3 padding bytes. That layout no longer
/// matches the current `FileNode` (which uses `u32` timestamps and stores no
/// accessed time), so V2 files must be decoded record-by-record rather than cast.
const V2_NODE_SIZE: usize = 56;

/// Decodes one 56-byte Version-2 node record into the current [`FileNode`].
///
/// Fields are read with explicit little-endian reads so the code is both
/// alignment-safe (records are not 8-byte aligned within the payload) and
/// endian-portable (no `cfg(target_endian)` branch). The accessed time is read
/// only to advance past it — it is intentionally discarded.
fn decode_v2_legacy_node(chunk: &[u8]) -> FileNode {
    // Helper closures keep the field-by-field reads readable and bounds-checked.
    let u32_at = |offset: usize| {
        u32::from_le_bytes(chunk[offset..offset + 4].try_into().unwrap_or([0; 4]))
    };
    let u64_at = |offset: usize| {
        u64::from_le_bytes(chunk[offset..offset + 8].try_into().unwrap_or([0; 8]))
    };
    let i64_at = |offset: usize| {
        i64::from_le_bytes(chunk[offset..offset + 8].try_into().unwrap_or([0; 8]))
    };

    let name_id = crate::arena::StringId(u32_at(0));
    let parent = u32_at(4);
    let first_child = u32_at(8);
    let next_sibling = u32_at(12);
    let size = u64_at(16);
    // Timestamps were i64 in V2; clamp pre-epoch values to the 0 "unknown" sentinel.
    let modified = i64_at(24).max(0) as u32;
    let created = i64_at(32).max(0) as u32;
    // accessed (i64 at offset 40) is discarded.
    let file_count = u32_at(48);
    let flags = chunk[52];

    let mut node = FileNode::new(
        name_id,
        (parent != crate::arena::NO_INDEX).then_some(parent),
        (flags & FileNode::FLAG_DIRECTORY) != 0,
        (flags & FileNode::FLAG_SYMLINK) != 0,
        modified,
        created,
    );
    node.parent = parent;
    node.first_child = first_child;
    node.next_sibling = next_sibling;
    node.size = size;
    node.file_count = file_count;
    // The dir/symlink/no-permission bit layout is unchanged, so carry the byte as-is.
    node.flags = flags;
    node
}

// =============================================================================
// Serialization APIs
// =============================================================================

/// Serializes nodes using the Version 3 format. If `compress` is enabled, the
/// output is wrapped inside a standard Zstandard container.
pub fn save_snapshot(
    nodes: &[FileNode],
    string_pool: &StringPool,
    path: &Path,
    compress: bool,
) -> Result<(), crate::EdirstatError> {
    save_snapshot_v3(nodes, string_pool, path, compress)
}

/// Serializes the snapshot in the Version 3 format.
pub fn save_snapshot_v3(
    nodes: &[FileNode],
    string_pool: &StringPool,
    path: &Path,
    compress: bool,
) -> Result<(), crate::EdirstatError> {
    let (arena_string, offsets) = string_pool.export_for_save()?;

    // 1. Sort nodes in memory into strict DFS pre-order using an explicit stack
    let mut dfs_order = Vec::with_capacity(nodes.len());
    let mut stack = vec![0u32];
    let mut children_buf = Vec::new();

    while let Some(idx) = stack.pop() {
        dfs_order.push(idx);

        let mut curr = nodes[idx as usize].first_child;
        while curr != crate::arena::NO_INDEX {
            children_buf.push(curr);
            curr = nodes[curr as usize].next_sibling;
        }
        stack.extend(children_buf.iter().copied().rev());
        children_buf.clear();
    }

    // Create 7 homogeneous column buffers
    let mut col_control = Vec::with_capacity(nodes.len());
    let mut col_name_id = Vec::with_capacity(nodes.len() * 2);
    let mut col_size = Vec::with_capacity(nodes.len() * 2);
    let mut col_mod_delta = Vec::with_capacity(nodes.len() * 2);
    let mut col_cre_delta = Vec::with_capacity(nodes.len() * 2);
    let mut col_file_count = Vec::with_capacity(nodes.len() * 2);
    let mut col_child_count = Vec::with_capacity(nodes.len() * 2);

    for &old_idx in &dfs_order {
        let node = &nodes[old_idx as usize];

        // Compute delta offsets of fields relative to original parent lookup.
        // Deltas are computed in i64 so negative deltas (created before modified,
        // or a node older than its parent) do not underflow; the underlying values
        // are u32 epoch seconds.
        let modified = node.modified_timestamp as i64;
        let created = node.created_timestamp as i64;
        let (mod_delta, cre_delta) = if node.parent == crate::arena::NO_INDEX {
            (modified, created - modified)
        } else {
            let parent_node = &nodes[node.parent as usize];
            (modified - parent_node.modified_timestamp as i64, created - modified)
        };

        let mod_eq_parent = node.parent != crate::arena::NO_INDEX && mod_delta == 0;
        let cre_eq_mod = cre_delta == 0;

        // 1. Pack directory, symlink, and permission flags into the control byte
        let mut control = 0u8;
        if node.is_directory() {
            control |= FileNode::FLAG_DIRECTORY;
        }
        if node.is_symlink() {
            control |= FileNode::FLAG_SYMLINK;
        }
        if node.has_no_permission() {
            control |= FileNode::FLAG_NO_PERMISSION;
        }
        if mod_eq_parent {
            control |= FLAG_MODIFIED_EQ_PARENT;
        }
        if cre_eq_mod {
            control |= FLAG_CREATED_EQ_MODIFIED;
        }
        col_control.push(control);

        // 2. Write name StringID (Varint)
        write_u64_varint(&mut col_name_id, node.name_id.0 as u64);

        // 3. Write size (Varint)
        write_u64_varint(&mut col_size, node.size);

        // 4. Write timestamps contiguously to their columns ONLY if they differ
        if !mod_eq_parent {
            write_i64_zigzag(&mut col_mod_delta, mod_delta);
        }
        if !cre_eq_mod {
            write_i64_zigzag(&mut col_cre_delta, cre_delta);
        }

        // 5. Write file count & immediate child count ONLY if directory
        if node.is_directory() {
            let mut child_count = 0u32;
            let mut curr = node.first_child;
            while curr != crate::arena::NO_INDEX {
                child_count += 1;
                curr = nodes[curr as usize].next_sibling;
            }

            write_u64_varint(&mut col_file_count, node.file_count as u64);
            write_u64_varint(&mut col_child_count, child_count as u64);
        }
    }

    // 6. Write StringPool in a highly compact, offsets-free sequential format
    let mut sp_buf = Vec::new();
    let string_count = offsets.len() - 1;
    write_u64_varint(&mut sp_buf, string_count as u64);

    for i in 0..string_count {
        let offset = offsets[i] as usize;
        let end = offsets[i + 1] as usize;
        let s_bytes = &arena_string.as_bytes()[offset..end];

        write_u64_varint(&mut sp_buf, s_bytes.len() as u64);
        sp_buf.extend_from_slice(s_bytes);
    }

    // Define column segments metadata
    let col_lengths = [
        col_control.len() as u32,
        col_name_id.len() as u32,
        col_size.len() as u32,
        col_mod_delta.len() as u32,
        col_cre_delta.len() as u32,
        col_file_count.len() as u32,
        col_child_count.len() as u32,
    ];

    let meta_header_size = col_lengths.len() * std::mem::size_of::<u32>(); // 7 * 4 = 28 bytes
    let nodes_size = meta_header_size + col_lengths.iter().sum::<u32>() as usize;
    let string_pool_length = sp_buf.len();
    let uncompressed_size = nodes_size + string_pool_length;

    let header = FileHeader {
        magic: *b"EDST",
        version: FILE_VERSION_V3,
        _padding: 0,
        uncompressed_size: uncompressed_size as u64,
        node_count: nodes.len() as u64,
        string_pool_offset: nodes_size as u64,
        string_pool_length: string_pool_length as u64,
        reserved: [0; 4],
    };

    // Serialize uncompressed payload into a memory buffer
    let mut edst_bytes = Vec::with_capacity(72 + uncompressed_size);
    let le_header = header.to_le();
    edst_bytes.write_all(bytemuck::bytes_of(&le_header))?;

    // Convert metadata header column lengths to little-endian representation
    let col_lengths_le: [u32; 7] = col_lengths.map(u32::to_le);
    edst_bytes.write_all(bytemuck::cast_slice(&col_lengths_le))?;

    // Write contiguous column blocks sequentially
    edst_bytes.write_all(&col_control)?;
    edst_bytes.write_all(&col_name_id)?;
    edst_bytes.write_all(&col_size)?;
    edst_bytes.write_all(&col_mod_delta)?;
    edst_bytes.write_all(&col_cre_delta)?;
    edst_bytes.write_all(&col_file_count)?;
    edst_bytes.write_all(&col_child_count)?;

    // Append string pool
    edst_bytes.write_all(&sp_buf)?;

    let mut file = File::create(path)?;
    if compress {
        let compressed = zstd::encode_all(&edst_bytes[..], ZSTD_COMPRESSION_LEVEL)?;
        file.write_all(&compressed)?;
    } else {
        file.write_all(&edst_bytes)?;
    }

    file.sync_all()?;
    Ok(())
}

// =============================================================================
// Deserialization API (Supporting both Version 2 and Version 3)
// =============================================================================

pub fn load_snapshot(path: &Path) -> Result<(PersistentArena, StringPool), crate::EdirstatError> {
    let mut file = File::open(path)?;
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)?;

    // Transparent layer: Decompress standard Zstd container if detected (magic: 0x28B52FFD)
    if file_bytes.len() >= 4 && file_bytes[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
        file_bytes = zstd::decode_all(&file_bytes[..])
            .map_err(|e| crate::EdirstatError::Zstd(e.to_string()))?;
    }

    if file_bytes.len() < 72 {
        return Err(crate::EdirstatError::HeaderTooSmall);
    }

    // Allocate an aligned FileHeader directly on the stack to prevent alignment panics
    let mut header = FileHeader::zeroed();
    let header_bytes = bytemuck::bytes_of_mut(&mut header);
    header_bytes.copy_from_slice(&file_bytes[0..72]);

    // Restore correct host representation
    header = header.from_le();

    if header.magic != *b"EDST" {
        return Err(crate::EdirstatError::InvalidMagic);
    }
    if header.version != FILE_VERSION_V2 && header.version != FILE_VERSION_V3 {
        return Err(crate::EdirstatError::UnsupportedVersion(header.version));
    }

    // Validate that the declared payload size fits in the host address space
    // before any arithmetic that would silently truncate it on 32-bit targets.
    let uncompressed_size = usize::try_from(header.uncompressed_size).map_err(|_| {
        crate::EdirstatError::OutOfRange("uncompressed_size exceeds addressable memory")
    })?;

    // Extract payload data
    let decompressed_data = if header.version == FILE_VERSION_V2 {
        // Legacy V2 uses internal bulk compression for the payload block
        let compressed_payload = &file_bytes[72..];
        zstd::bulk::decompress(compressed_payload, uncompressed_size)
            .map_err(|e| crate::EdirstatError::Zstd(e.to_string()))?
    } else {
        // V3 payload is written raw (with any compression handled at the transparent file wrapper layer)
        let payload_end =
            72_usize
                .checked_add(uncompressed_size)
                .ok_or(crate::EdirstatError::OutOfRange(
                    "payload end offset overflow",
                ))?;
        if file_bytes.len() < payload_end {
            return Err(crate::EdirstatError::TruncatedNodes);
        }
        file_bytes[72..payload_end].to_vec()
    };

    // 1. Reconstruct the StringPool first
    let sp_start = usize::try_from(header.string_pool_offset).map_err(|_| {
        crate::EdirstatError::OutOfRange("string_pool_offset exceeds addressable memory")
    })?;
    let sp_length = usize::try_from(header.string_pool_length).map_err(|_| {
        crate::EdirstatError::OutOfRange("string_pool_length exceeds addressable memory")
    })?;
    let sp_end = sp_start
        .checked_add(sp_length)
        .ok_or(crate::EdirstatError::OutOfRange(
            "string pool end offset overflow",
        ))?;
    if decompressed_data.len() < sp_end {
        return Err(crate::EdirstatError::TruncatedStringPool);
    }

    let sp_slice = &decompressed_data[sp_start..sp_end];

    let string_pool = if header.version == FILE_VERSION_V2 {
        // Need at least the 8-byte string-count prefix before reading anything.
        if sp_slice.len() < 8 {
            return Err(crate::EdirstatError::TruncatedStringPool);
        }
        let mut offset_count_bytes = [0u8; 8];
        offset_count_bytes.copy_from_slice(&sp_slice[0..8]);
        let offsets_count =
            usize::try_from(u64::from_le_bytes(offset_count_bytes)).map_err(|_| {
                crate::EdirstatError::OutOfRange(
                    "string pool offset count exceeds addressable memory",
                )
            })?;

        let offsets_start: usize = 8;
        let offsets_end = offsets_start
            .checked_add(
                offsets_count
                    .checked_mul(std::mem::size_of::<u32>())
                    .ok_or(crate::EdirstatError::OutOfRange(
                        "string pool offset table size overflow",
                    ))?,
            )
            .ok_or(crate::EdirstatError::OutOfRange(
                "string pool offset table end overflow",
            ))?;
        // The 8-byte raw-bytes count prefix immediately follows the offset table.
        let counts_end = offsets_end
            .checked_add(8)
            .ok_or(crate::EdirstatError::OutOfRange(
                "string pool prefix end overflow",
            ))?;
        if sp_slice.len() < counts_end {
            return Err(crate::EdirstatError::TruncatedStringPool);
        }
        let offsets_bytes = &sp_slice[offsets_start..offsets_end];
        let mut offsets = u8_slice_to_u32_vec(offsets_bytes);

        // Convert slice values on-the-fly only when compiled on big-endian architectures
        if cfg!(target_endian = "big") {
            for val in &mut offsets {
                *val = u32::from_le(*val);
            }
        }

        let mut bytes_count_bytes = [0u8; 8];
        bytes_count_bytes.copy_from_slice(&sp_slice[offsets_end..counts_end]);
        let bytes_count = usize::try_from(u64::from_le_bytes(bytes_count_bytes)).map_err(|_| {
            crate::EdirstatError::OutOfRange("string pool byte count exceeds addressable memory")
        })?;

        let raw_bytes_start = counts_end;
        let raw_bytes_end =
            raw_bytes_start
                .checked_add(bytes_count)
                .ok_or(crate::EdirstatError::OutOfRange(
                    "string pool raw bytes end overflow",
                ))?;
        if sp_slice.len() < raw_bytes_end {
            return Err(crate::EdirstatError::TruncatedStringPool);
        }
        let raw_bytes = &sp_slice[raw_bytes_start..raw_bytes_end];

        let arena_data: Arc<str> = Arc::from(
            std::str::from_utf8(raw_bytes).map_err(|_| crate::EdirstatError::InvalidUtf8)?,
        );

        // Validate that the offsets are monotonic and in-bounds so a corrupt
        // table cannot underflow or slice OOB when frozen handles are resolved.
        let upper_bound = u32::try_from(raw_bytes.len()).map_err(|_| {
            crate::EdirstatError::OutOfRange("string pool exceeds u32 addressable range")
        })?;
        for pair in offsets.windows(2) {
            let offset = pair[0];
            let next = pair[1];
            if next < offset || next > upper_bound {
                return Err(crate::EdirstatError::Corrupt(
                    "string pool offsets are not monotonic or out of bounds",
                ));
            }
        }
        StringPool::frozen(arena_data, offsets)
    } else {
        // Version 3: Sequentially rebuild offsets and the StringPool without storage tables
        let mut sp_cursor = 0;
        let string_count =
            usize::try_from(read_u64_varint(sp_slice, &mut sp_cursor)?).map_err(|_| {
                crate::EdirstatError::OutOfRange("string count exceeds addressable memory")
            })?;

        // The capacity hint is bounded by the slice length so a maliciously large
        // count cannot trigger a huge pre-allocation before we discover the stream
        // is actually short.
        let mut offsets = Vec::with_capacity(string_count.min(sp_slice.len()).saturating_add(1));
        offsets.push(0u32);

        let mut decoded_strings_count = 0;
        let mut current_offset = 0u32;
        let mut string_bytes = Vec::new();

        while decoded_strings_count < string_count {
            let raw_len = read_u64_varint(sp_slice, &mut sp_cursor)?;
            let len = u32::try_from(raw_len)
                .map_err(|_| crate::EdirstatError::OutOfRange("string length exceeds u32 range"))?;
            let len_usize = usize::try_from(len).map_err(|_| {
                crate::EdirstatError::OutOfRange("string length exceeds addressable memory")
            })?;
            let end = sp_cursor
                .checked_add(len_usize)
                .ok_or(crate::EdirstatError::OutOfRange(
                    "string offset end overflow",
                ))?;
            if end > sp_slice.len() {
                return Err(crate::EdirstatError::TruncatedStringPool);
            }
            string_bytes.extend_from_slice(&sp_slice[sp_cursor..end]);
            sp_cursor = end;
            current_offset =
                current_offset
                    .checked_add(len)
                    .ok_or(crate::EdirstatError::OutOfRange(
                        "cumulative string offset overflow",
                    ))?;
            offsets.push(current_offset);
            decoded_strings_count += 1;
        }

        let arena_data: Arc<str> = Arc::from(
            std::str::from_utf8(&string_bytes).map_err(|_| crate::EdirstatError::InvalidUtf8)?,
        );
        StringPool::frozen(arena_data, offsets)
    };

    // 2. Decode file nodes based on format version
    let node_count = usize::try_from(header.node_count)
        .map_err(|_| crate::EdirstatError::OutOfRange("node count exceeds addressable memory"))?;
    let decoded_nodes = if header.version == FILE_VERSION_V2 {
        // Legacy V2 stored nodes as a raw memory dump of the pre-u32 `FileNode`
        // layout: `V2_NODE_SIZE` bytes per record, with three i64 timestamps.
        // Decode each record field-by-field into the current `FileNode`; the
        // accessed time (read only to skip past it) is intentionally discarded.
        let expected_nodes_size = node_count
            .checked_mul(V2_NODE_SIZE)
            .ok_or(crate::EdirstatError::OutOfRange("node table size overflow"))?;
        if decompressed_data.len() < expected_nodes_size {
            return Err(crate::EdirstatError::TruncatedNodes);
        }
        decompressed_data[..expected_nodes_size]
            .chunks_exact(V2_NODE_SIZE)
            .map(decode_v2_legacy_node)
            .collect()
    } else {
        // Version 3: Reconstruct nodes from the 7 parallel column streams
        let mut decoded =
            vec![FileNode::new(crate::arena::StringId(0), None, false, false, 0, 0); node_count];

        // The 28-byte (7 * 4) column-length prefix must be present before we read it.
        if decompressed_data.len() < 28 {
            return Err(crate::EdirstatError::TruncatedNodes);
        }
        let mut col_lengths = u8_slice_to_u32_vec(&decompressed_data[0..28]);

        // Convert metadata sizes back to host-endian
        if cfg!(target_endian = "big") {
            for len in &mut col_lengths {
                *len = u32::from_le(*len);
            }
        }

        // Segment the column boundaries with overflow- and bounds-checked math
        // so a corrupt header cannot index past the available data.
        let mut start = 28;
        let (col_control_slice, next) = take_column(&decompressed_data, start, col_lengths[0])?;
        start = next;
        let (col_name_id_slice, next) = take_column(&decompressed_data, start, col_lengths[1])?;
        start = next;
        let (col_size_slice, next) = take_column(&decompressed_data, start, col_lengths[2])?;
        start = next;
        let (col_mod_delta_slice, next) = take_column(&decompressed_data, start, col_lengths[3])?;
        start = next;
        let (col_cre_delta_slice, next) = take_column(&decompressed_data, start, col_lengths[4])?;
        start = next;
        let (col_file_count_slice, next) = take_column(&decompressed_data, start, col_lengths[5])?;
        start = next;
        let (col_child_count_slice, _next) =
            take_column(&decompressed_data, start, col_lengths[6])?;

        // Track cursor positions sequentially per column

        let mut cursor_name_id = 0;
        let mut cursor_size = 0;
        let mut cursor_mod_delta = 0;
        let mut cursor_cre_delta = 0;
        let mut cursor_file_count = 0;
        let mut cursor_child_count = 0;

        // Parent tracking stack: (parent_idx, remaining_immediate_children_to_process)
        let mut parent_stack: Vec<(u32, u32)> = Vec::new();

        // Guard the directly-indexed control column: every node reads one byte
        // from it, so it must be at least `node_count` long or we would panic.
        if col_control_slice.len() < node_count {
            return Err(crate::EdirstatError::Corrupt(
                "control column is shorter than the declared node count",
            ));
        }

        for idx in 0..node_count {
            let control = col_control_slice[idx];

            let is_dir = (control & FileNode::FLAG_DIRECTORY) != 0;
            let is_symlink = (control & FileNode::FLAG_SYMLINK) != 0;
            let no_permission = (control & FileNode::FLAG_NO_PERMISSION) != 0;
            let mod_eq_parent = (control & FLAG_MODIFIED_EQ_PARENT) != 0;
            let cre_eq_mod = (control & FLAG_CREATED_EQ_MODIFIED) != 0;

            let name_id_val = read_u64_varint(col_name_id_slice, &mut cursor_name_id)? as u32;
            let size = read_u64_varint(col_size_slice, &mut cursor_size)?;

            // Reconstruct modification timestamp relative to parent directory flag
            let mod_delta = if mod_eq_parent {
                0
            } else {
                read_i64_zigzag(col_mod_delta_slice, &mut cursor_mod_delta)?
            };

            // Reconstruct creation timestamp relative to modified flag (0 bytes read if matching modified time)
            let cre_delta = if cre_eq_mod {
                0
            } else {
                read_i64_zigzag(col_cre_delta_slice, &mut cursor_cre_delta)?
            };

            // Reconstruct file_count & child_count only if directory
            let (file_count, children_count) = if is_dir {
                (
                    read_u64_varint(col_file_count_slice, &mut cursor_file_count)? as u32,
                    read_u64_varint(col_child_count_slice, &mut cursor_child_count)? as u32,
                )
            } else {
                (0, 0)
            };

            // Reconstruct absolute parent pointer implicitly using the DFS pre-order stack
            let parent = parent_stack.last().map(|&(parent_idx, _)| parent_idx);

            // Reconstruct absolute timestamps as u32. Deltas were encoded as i64, so
            // the arithmetic stays exact even when created precedes modified or a
            // node is older than its parent. The originals are u32, so `as u32` is lossless.
            let (modified, created) = parent.map_or_else(
                || (mod_delta as u32, (cre_delta + mod_delta) as u32),
                |p| {
                    let parent_node = &decoded[p as usize];
                    let absolute_mod = parent_node.modified_timestamp as i64 + mod_delta;
                    (absolute_mod as u32, (cre_delta + absolute_mod) as u32)
                },
            );

            let mut node = FileNode::new(
                crate::arena::StringId(name_id_val),
                parent,
                is_dir,
                is_symlink,
                modified,
                created,
            );
            node.size = size;
            node.file_count = file_count;
            if no_permission {
                node.flags |= FileNode::FLAG_NO_PERMISSION;
            }

            decoded[idx] = node;

            // Decrement the active parent's remaining children count
            if !parent_stack.is_empty() {
                let last_idx = parent_stack.len() - 1;
                parent_stack[last_idx].1 -= 1;

                // Recursively pop completed directories off the stack
                while let Some(last) = parent_stack.last() {
                    if last.1 == 0 {
                        parent_stack.pop();
                    } else {
                        break;
                    }
                }
            }

            // If the current node is a directory containing child nodes, push it onto the active stack
            if is_dir && children_count > 0 {
                parent_stack.push((idx as u32, children_count));
            }
        }

        // Reconstruct first_child and next_sibling structural links in O(1)
        let mut last_child = vec![crate::arena::NO_INDEX; node_count];
        for child_idx in 1..node_count {
            let parent_idx = decoded[child_idx].parent;
            if parent_idx != crate::arena::NO_INDEX {
                let p = parent_idx as usize;
                let prev_sibling = last_child[p];
                if prev_sibling == crate::arena::NO_INDEX {
                    decoded[p].first_child = child_idx as u32;
                } else {
                    decoded[prev_sibling as usize].next_sibling = child_idx as u32;
                }
                last_child[p] = child_idx as u32;
            }
        }

        decoded
    };

    let arena = PersistentArena::new(decoded_nodes);
    Ok((arena, string_pool))
}

/// Bounds-checked extraction of a length-prefixed column slice. Returns the
/// slice and the cursor advanced past it. Prevents out-of-bounds panics when a
/// corrupt header advertises a column that extends beyond the available data.
fn take_column(
    data: &[u8],
    start: usize,
    length: u32,
) -> Result<(&[u8], usize), crate::EdirstatError> {
    let len = usize::try_from(length).map_err(|_| {
        crate::EdirstatError::OutOfRange("column length exceeds addressable memory")
    })?;
    let end = start
        .checked_add(len)
        .ok_or(crate::EdirstatError::OutOfRange(
            "column end offset overflow",
        ))?;
    if end > data.len() {
        return Err(crate::EdirstatError::TruncatedNodes);
    }
    Ok((&data[start..end], end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncompressed_roundtrip() -> Result<(), crate::EdirstatError> {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0),
        ];
        nodes[0].first_child = 1;
        nodes[0].size = 1000;
        nodes[1].size = 1000;

        let temp_dir = std::env::current_dir()?.join("target");
        let path_uncompressed = temp_dir.join("test_uncompressed.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Save as uncompressed
        save_snapshot(&nodes, &pool, &path_uncompressed, false)?;

        // Load back
        let (loaded_arena, pool_loaded) = load_snapshot(&path_uncompressed)?;
        let loaded_nodes = loaded_arena.nodes();

        assert_eq!(loaded_nodes.len(), 2);
        assert_eq!(loaded_nodes[0].size, 1000);
        assert_eq!(loaded_nodes[1].size, 1000);
        assert_eq!(pool_loaded.get(loaded_nodes[1].name_id), Some("f1.png"));

        let _ = std::fs::remove_file(&path_uncompressed);
        Ok(())
    }

    #[test]
    fn test_load_snapshot_header_too_small() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_small.edst");
        let _ = std::fs::create_dir_all(&temp_dir);
        std::fs::write(&test_path, b"too_small")?;

        let res = load_snapshot(&test_path);
        assert!(matches!(res, Err(crate::EdirstatError::HeaderTooSmall)));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_snapshot_invalid_magic() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_invalid_magic.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        let header = FileHeader {
            magic: *b"BAD!",
            version: FILE_VERSION_V3,
            _padding: 0,
            uncompressed_size: 0,
            node_count: 0,
            string_pool_offset: 72,
            string_pool_length: 0,
            reserved: [0; 4],
        };
        std::fs::write(&test_path, bytemuck::bytes_of(&header))?;

        let res = load_snapshot(&test_path);
        assert!(matches!(res, Err(crate::EdirstatError::InvalidMagic)));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_snapshot_unsupported_version() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_unsupported_version.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        let header = FileHeader {
            magic: *b"EDST",
            version: 99,
            _padding: 0,
            uncompressed_size: 0,
            node_count: 0,
            string_pool_offset: 72,
            string_pool_length: 0,
            reserved: [0; 4],
        };
        std::fs::write(&test_path, bytemuck::bytes_of(&header))?;

        let res = load_snapshot(&test_path);
        assert!(matches!(
            res,
            Err(crate::EdirstatError::UnsupportedVersion(99))
        ));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_v2_zero_string_offsets_no_panic() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_v2_zero_offsets.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Hand-build a V2 payload whose string pool declares zero offsets, then
        // zstd-compress it so the loader takes the V2 branch. Previously the
        // `offsets.len() - 1` underflowed and panicked; it must now succeed
        // (returning an empty snapshot) instead of aborting.
        let mut raw_payload: Vec<u8> = Vec::new();
        raw_payload.extend_from_slice(&0u64.to_le_bytes()); // offsets_count = 0
        raw_payload.extend_from_slice(&0u64.to_le_bytes()); // bytes_count = 0

        let compressed = zstd::encode_all(&raw_payload[..], ZSTD_COMPRESSION_LEVEL)?;

        let header = FileHeader {
            magic: *b"EDST",
            version: FILE_VERSION_V2,
            _padding: 0,
            uncompressed_size: raw_payload.len() as u64,
            node_count: 0,
            string_pool_offset: 0,
            string_pool_length: raw_payload.len() as u64,
            reserved: [0; 4],
        };

        let mut file_bytes = bytemuck::bytes_of(&header.to_le()).to_vec();
        file_bytes.extend_from_slice(&compressed);
        std::fs::write(&test_path, &file_bytes)?;

        let res = load_snapshot(&test_path);
        assert!(res.is_ok(), "loading must not panic/abort: {res:?}");

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_v3_invalid_utf8_string_pool() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_v3_bad_utf8.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // 32 zero bytes of column metadata (node_count = 0) followed by a V3
        // string pool describing a single 1-byte string whose lone byte (0xFF)
        // is not valid UTF-8. This used to silently turn the whole pool into an
        // empty string via `unwrap_or("")`; it must now report InvalidUtf8.
        let mut payload = vec![0u8; 32];
        payload.extend_from_slice(&[0x01, 0x01, 0xFF]); // count=1, len=1, 0xFF

        let header = FileHeader {
            magic: *b"EDST",
            version: FILE_VERSION_V3,
            _padding: 0,
            uncompressed_size: payload.len() as u64,
            node_count: 0,
            string_pool_offset: 32,
            string_pool_length: 3,
            reserved: [0; 4],
        };

        let mut file_bytes = bytemuck::bytes_of(&header.to_le()).to_vec();
        file_bytes.extend_from_slice(&payload);
        std::fs::write(&test_path, &file_bytes)?;

        let res = load_snapshot(&test_path);
        assert!(matches!(res, Err(crate::EdirstatError::InvalidUtf8)));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_v3_control_column_shorter_than_node_count() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_v3_short_control.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Column metadata: control column claims 2 bytes, all others 0. The
        // node count (5) exceeds the control length, which must be rejected
        // rather than indexing out of bounds.
        let col_lengths: [u32; 8] = [2, 0, 0, 0, 0, 0, 0, 0];
        let mut payload: Vec<u8> = Vec::new();
        for &l in &col_lengths {
            payload.extend_from_slice(&l.to_le_bytes());
        }
        payload.extend_from_slice(&[0u8, 0u8]); // control column bytes
        payload.push(0u8); // V3 string pool: zero strings

        let header = FileHeader {
            magic: *b"EDST",
            version: FILE_VERSION_V3,
            _padding: 0,
            uncompressed_size: payload.len() as u64,
            node_count: 5,
            string_pool_offset: 34,
            string_pool_length: 1,
            reserved: [0; 4],
        };

        let mut file_bytes = bytemuck::bytes_of(&header.to_le()).to_vec();
        file_bytes.extend_from_slice(&payload);
        std::fs::write(&test_path, &file_bytes)?;

        let res = load_snapshot(&test_path);
        assert!(matches!(res, Err(crate::EdirstatError::Corrupt(_))));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    #[test]
    fn test_load_v3_truncated_column() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_v3_truncated_column.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // The control column claims 100 bytes but only 2 are present, so the
        // bounds-checked column extraction must fail instead of slicing OOB.
        let col_lengths: [u32; 8] = [100, 0, 0, 0, 0, 0, 0, 0];
        let mut payload: Vec<u8> = Vec::new();
        for &l in &col_lengths {
            payload.extend_from_slice(&l.to_le_bytes());
        }
        payload.extend_from_slice(&[0u8, 0u8]); // only 2 control bytes exist
        payload.push(0u8); // V3 string pool: zero strings

        let header = FileHeader {
            magic: *b"EDST",
            version: FILE_VERSION_V3,
            _padding: 0,
            uncompressed_size: payload.len() as u64,
            node_count: 5,
            string_pool_offset: 34,
            string_pool_length: 1,
            reserved: [0; 4],
        };

        let mut file_bytes = bytemuck::bytes_of(&header.to_le()).to_vec();
        file_bytes.extend_from_slice(&payload);
        std::fs::write(&test_path, &file_bytes)?;

        let res = load_snapshot(&test_path);
        assert!(matches!(res, Err(crate::EdirstatError::TruncatedNodes)));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    /// Builds one 56-byte Version-2 legacy node record in the pre-u32 layout:
    /// four `u32` pointers, `u64` size, **three `i64` timestamps**, `u32`
    /// `file_count`, `u8` flags, 3 padding bytes.
    #[allow(clippy::too_many_arguments)]
    fn legacy_v2_record(
        name_id: u32,
        parent: u32,
        first_child: u32,
        next_sibling: u32,
        size: u64,
        modified: i64,
        created: i64,
        accessed: i64,
        file_count: u32,
        flags: u8,
    ) -> [u8; V2_NODE_SIZE] {
        let mut b = [0u8; V2_NODE_SIZE];
        b[0..4].copy_from_slice(&name_id.to_le_bytes());
        b[4..8].copy_from_slice(&parent.to_le_bytes());
        b[8..12].copy_from_slice(&first_child.to_le_bytes());
        b[12..16].copy_from_slice(&next_sibling.to_le_bytes());
        b[16..24].copy_from_slice(&size.to_le_bytes());
        b[24..32].copy_from_slice(&modified.to_le_bytes());
        b[32..40].copy_from_slice(&created.to_le_bytes());
        b[40..48].copy_from_slice(&accessed.to_le_bytes());
        b[48..52].copy_from_slice(&file_count.to_le_bytes());
        b[52] = flags;
        // bytes 53..56 stay zero (padding)
        b
    }

    /// A Version-2 snapshot (raw 56-byte legacy records with three i64 timestamps)
    /// must still load: modified/created are recovered as u32, the accessed time is
    /// discarded, and pre-epoch i64 values clamp to the 0 "unknown" sentinel.
    #[test]
    fn test_load_v2_legacy_timestamps() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_v2_legacy_timestamps.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Nodes (DFS order): root dir, a file, and a file whose i64 times are pre-1970.
        let nodes_buf: Vec<u8> = [
            legacy_v2_record(0, u32::MAX, 1, u32::MAX, 0, 1_000_000_000, 900_000_000, 2_000_000_000, 1, 1),
            legacy_v2_record(1, 0, u32::MAX, u32::MAX, 500, 1_100_000_000, 950_000_000, 2_100_000_000, 0, 0),
            legacy_v2_record(2, 0, u32::MAX, u32::MAX, 0, -50, -100, 5_000_000_000, 0, 0),
        ]
        .concat();

        // String pool (V2 layout): offsets_count, offsets[], bytes_count, raw bytes.
        let strings = ["root", "f1.png", "old"];
        let mut offsets: Vec<u32> = vec![0];
        let mut arena = String::new();
        for s in strings {
            arena.push_str(s);
            offsets.push(arena.len() as u32);
        }
        let mut sp_buf: Vec<u8> = Vec::new();
        sp_buf.extend_from_slice(&(offsets.len() as u64).to_le_bytes());
        for &o in &offsets {
            sp_buf.extend_from_slice(&o.to_le_bytes());
        }
        sp_buf.extend_from_slice(&(arena.len() as u64).to_le_bytes());
        sp_buf.extend_from_slice(arena.as_bytes());

        let mut raw_payload = Vec::new();
        raw_payload.extend_from_slice(&nodes_buf);
        raw_payload.extend_from_slice(&sp_buf);

        let nodes_size = nodes_buf.len();
        let string_pool_length = sp_buf.len();
        let uncompressed_size = raw_payload.len();

        let header = FileHeader {
            magic: *b"EDST",
            version: FILE_VERSION_V2,
            _padding: 0,
            uncompressed_size: uncompressed_size as u64,
            node_count: 3,
            string_pool_offset: nodes_size as u64,
            string_pool_length: string_pool_length as u64,
            reserved: [0; 4],
        };

        let compressed = zstd::encode_all(&raw_payload[..], ZSTD_COMPRESSION_LEVEL)?;
        let mut file_bytes = bytemuck::bytes_of(&header.to_le()).to_vec();
        file_bytes.extend_from_slice(&compressed);
        std::fs::write(&test_path, &file_bytes)?;

        let (arena_obj, pool_loaded) = load_snapshot(&test_path)?;
        let n = arena_obj.nodes();

        assert_eq!(n.len(), 3);
        // modified/created recovered as u32; accessed (2e9/2.1e9) discarded entirely.
        assert_eq!(n[0].modified_timestamp, 1_000_000_000);
        assert_eq!(n[0].created_timestamp, 900_000_000);
        assert!(n[0].is_directory());
        assert_eq!(pool_loaded.get(n[0].name_id), Some("root"));

        assert_eq!(n[1].modified_timestamp, 1_100_000_000);
        assert_eq!(n[1].created_timestamp, 950_000_000);
        assert_eq!(n[1].size, 500);
        assert_eq!(pool_loaded.get(n[1].name_id), Some("f1.png"));

        // Pre-epoch i64 timestamps clamp to 0.
        assert_eq!(n[2].modified_timestamp, 0);
        assert_eq!(n[2].created_timestamp, 0);
        assert_eq!(pool_loaded.get(n[2].name_id), Some("old"));

        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }

    /// Round-trips `v3` nodes whose timestamps require negative deltas: a child
    /// newer in creation than modification (negative `cre_delta`), and a child
    /// older than its parent (negative `mod_delta`). The `i64` delta math must
    /// reconstruct the exact u32 values.
    #[test]
    fn test_v3_negative_delta_roundtrip() -> Result<(), crate::EdirstatError> {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f_id = pool.get_or_insert(b"child.txt");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 1_000_000_000, 900_000_000),
            FileNode::new(f_id, Some(0), false, false, 800_000_000, 850_000_000),
        ];
        nodes[0].first_child = 1;

        let temp_dir = std::env::current_dir()?.join("target");
        let path = temp_dir.join("test_v3_negative_delta.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        save_snapshot(&nodes, &pool, &path, false)?;
        let (arena_obj, _) = load_snapshot(&path)?;
        let n = arena_obj.nodes();

        assert_eq!(n.len(), 2);
        assert_eq!(n[0].modified_timestamp, 1_000_000_000);
        assert_eq!(n[0].created_timestamp, 900_000_000);
        assert_eq!(n[1].modified_timestamp, 800_000_000);
        assert_eq!(n[1].created_timestamp, 850_000_000);

        let _ = std::fs::remove_file(&path);
        Ok(())
    }
}
