use std::{fs::File, io::Write, path::Path, sync::Arc};

use bytemuck::{Pod, Zeroable};
use memmap2::MmapOptions;

use super::arena::{FileNode, StringPool};

pub const FILE_VERSION: u16 = 2;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    _padding: u16,
    pub node_count: u64,
    pub string_pool_offset: u64,
    pub string_pool_length: u64,
}

pub struct PersistentArena {
    /// Underlying memory-mapped file (mapped copy-on-write)
    mmap: memmap2::MmapMut,
    node_count: usize,
}

impl PersistentArena {
    #[must_use]
    pub const fn new(mmap: memmap2::MmapMut, node_count: usize) -> Self {
        Self { mmap, node_count }
    }

    #[must_use]
    pub fn nodes(&self) -> &[FileNode] {
        let start = 32;
        let end = start + self.node_count * std::mem::size_of::<FileNode>();
        let bytes = &self.mmap[start..end];
        bytemuck::cast_slice(bytes)
    }

    pub fn nodes_mut(&mut self) -> &mut [FileNode] {
        let start = 32;
        let end = start + self.node_count * std::mem::size_of::<FileNode>();
        let bytes = &mut self.mmap[start..end];
        bytemuck::cast_slice_mut(bytes)
    }
}

pub fn save_snapshot(
    nodes: &[FileNode],
    string_pool: &StringPool,
    path: &Path,
) -> Result<(), crate::EdirstatError> {
    let mut file = File::create(path)?;

    // Calculate offsets by exporting the interner
    let (arena_string, offsets) = string_pool.interner.clone().export_arena().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Interner handle overflow during export",
        )
    })?;

    // Calculate offsets
    let header_size = 32u64;
    let nodes_size = std::mem::size_of_val(nodes) as u64;
    let string_pool_offset = header_size + nodes_size;

    // We will write the string pool as:
    // [ offsets_count: u64 ] [ u32 start offsets array ] [ bytes_count: u64 ] [ raw string pool bytes ]
    let offsets_count = offsets.len() as u64;
    let offsets_size = (offsets.len() * std::mem::size_of::<u32>()) as u64;
    let bytes_count = arena_string.len() as u64;
    let string_pool_length = 8 + offsets_size + 8 + bytes_count;

    // Create header with version 2
    let header = FileHeader {
        magic: *b"EDST",
        version: FILE_VERSION,
        _padding: 0,
        node_count: nodes.len() as u64,
        string_pool_offset,
        string_pool_length,
    };

    // Write header
    file.write_all(bytemuck::bytes_of(&header))?;

    // Write nodes
    file.write_all(bytemuck::cast_slice(nodes))?;

    // Write string pool components
    file.write_all(&offsets_count.to_le_bytes())?;
    file.write_all(bytemuck::cast_slice(&offsets))?;
    file.write_all(&bytes_count.to_le_bytes())?;
    file.write_all(arena_string.as_bytes())?;

    file.sync_all()?;
    Ok(())
}

pub fn load_snapshot(path: &Path) -> Result<(PersistentArena, StringPool), crate::EdirstatError> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.len() < 32 {
        return Err(crate::EdirstatError::HeaderTooSmall);
    }

    // Map the file privately copy-on-write
    let mmap = unsafe { MmapOptions::new().map_copy(&file)? };

    // Cast the header
    let header: &FileHeader = bytemuck::from_bytes(&mmap[0..32]);
    if header.magic != *b"EDST" {
        return Err(crate::EdirstatError::InvalidMagic);
    }
    // Only accept the current `FILE_VERSION`
    if header.version != FILE_VERSION {
        return Err(crate::EdirstatError::UnsupportedVersion(header.version));
    }

    let node_count = header.node_count as usize;
    let expected_size = 32 + node_count * std::mem::size_of::<FileNode>();
    if mmap.len() < expected_size {
        return Err(crate::EdirstatError::TruncatedNodes);
    }

    // Read StringPool
    let sp_start = header.string_pool_offset as usize;
    let sp_end = sp_start + header.string_pool_length as usize;
    if mmap.len() < sp_end {
        return Err(crate::EdirstatError::TruncatedStringPool);
    }

    let sp_slice = &mmap[sp_start..sp_end];

    // Parse offset count (8 bytes)
    let mut offset_count_bytes = [0u8; 8];
    offset_count_bytes.copy_from_slice(&sp_slice[0..8]);
    let offsets_count = u64::from_le_bytes(offset_count_bytes) as usize;

    // Parse u32 offsets array
    let offsets_start = 8;
    let offsets_end = offsets_start + offsets_count * std::mem::size_of::<u32>();
    let offsets_bytes = &sp_slice[offsets_start..offsets_end];
    let offsets: &[u32] = bytemuck::cast_slice(offsets_bytes);

    // Parse bytes count (8 bytes)
    let mut bytes_count_bytes = [0u8; 8];
    bytes_count_bytes.copy_from_slice(&sp_slice[offsets_end..offsets_end + 8]);
    let bytes_count = u64::from_le_bytes(bytes_count_bytes) as usize;

    // Parse raw bytes
    let raw_bytes_start = offsets_end + 8;
    let raw_bytes_end = raw_bytes_start + bytes_count;
    let raw_bytes = &sp_slice[raw_bytes_start..raw_bytes_end];

    // Reconstruct the interner allocation-free
    let arena_data: Arc<str> = Arc::from(std::str::from_utf8(raw_bytes).unwrap_or(""));
    let mut interner = xgx_intern::Interner::new(ahash::RandomState::new());

    for i in 0..offsets.len() - 1 {
        let offset = offsets[i];
        let len = offsets[i + 1] - offset;
        let shared_str = xgx_intern::ArenaString::Shared {
            arena: arena_data.clone(),
            offset,
            len,
        };
        // Rebuilds the lookup map without allocating any heap memory for the keys
        let _ = interner.intern_owned(shared_str);
    }

    let string_pool = StringPool { interner };
    let arena = PersistentArena::new(mmap, node_count);

    Ok((arena, string_pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_serialization_and_mmap_load() -> Result<(), crate::EdirstatError> {
        let mut pool = StringPool::new();
        let name_root = pool.get_or_insert(b"/");
        let name_dir = pool.get_or_insert(b"target");
        let name_file = pool.get_or_insert(b"lib.rs");

        let mut nodes = vec![
            FileNode::new(name_root, None, true, false, 0, 0, 0),
            FileNode::new(name_dir, Some(0), true, false, 0, 0, 0),
            FileNode::new(name_file, Some(1), false, false, 0, 0, 0),
        ];

        // Connect nodes
        nodes[0].first_child = 1;
        nodes[1].first_child = 2;
        nodes[1].size = 12345;
        nodes[1].file_count = 1;
        nodes[2].size = 12345;

        // Use a temporary test file path inside the workspace
        let temp_dir = std::env::current_dir()?.join("target");
        let test_path = temp_dir.join("test_snapshot.edst");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Save snapshot
        save_snapshot(&nodes, &pool, &test_path)?;

        // Load snapshot via mmap copy-on-write
        let (mut loaded_arena, loaded_pool) = load_snapshot(&test_path)?;
        let loaded_nodes = loaded_arena.nodes();

        // Validate structure size & elements
        assert_eq!(loaded_nodes.len(), 3);
        assert_eq!(loaded_nodes[0].name_id, name_root);
        assert_eq!(loaded_nodes[1].name_id, name_dir);
        assert_eq!(loaded_nodes[2].name_id, name_file);

        assert_eq!(loaded_nodes[0].first_child, 1);
        assert_eq!(loaded_nodes[1].first_child, 2);
        assert_eq!(loaded_nodes[1].size, 12345);
        assert_eq!(loaded_nodes[2].size, 12345);

        // Validate string pool contents
        assert_eq!(loaded_pool.get(name_root), Some("/"));
        assert_eq!(loaded_pool.get(name_dir), Some("target"));
        assert_eq!(loaded_pool.get(name_file), Some("lib.rs"));

        // Validate Mutability (Copy-On-Write):
        // Confirm that the loaded nodes can be modified in-memory (e.g. for on-demand lazy sorting)
        let loaded_nodes_mut = loaded_arena.nodes_mut();
        loaded_nodes_mut[1].next_sibling = 999;
        assert_eq!(loaded_nodes_mut[1].next_sibling, 999);

        // Clean up temporary file
        let _ = std::fs::remove_file(&test_path);
        Ok(())
    }
}
