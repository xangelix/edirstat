use std::{collections::HashMap, sync::Arc};

use bytemuck::{Pod, Zeroable};

pub const NO_INDEX: u32 = u32::MAX;
pub const NO_EXTENSION: &str = "(no extension)";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(transparent)]
pub struct StringId(pub u32);

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C, align(8))]
pub struct FileNode {
    /// Index into the global `StringPool` for the entry's base name (e.g., "Cargo.toml")
    pub name_id: StringId,

    /// Arena index of the parent node. `u32::MAX` if none.
    pub parent: u32,

    /// Arena index of the first child node. `u32::MAX` if empty or file.
    pub first_child: u32,

    /// Arena index of the next sibling. `u32::MAX` if last sibling.
    pub next_sibling: u32,

    /// Cumulative size in bytes on disk.
    pub size: u64,

    /// Last modified timestamp (seconds since Unix Epoch)
    pub modified_timestamp: i64,

    /// Creation timestamp (seconds since Unix Epoch)
    pub created_timestamp: i64,

    /// Last access timestamp (seconds since Unix Epoch)
    pub accessed_timestamp: i64,

    /// Total number of files nested under this node (if directory).
    pub file_count: u32,

    /// Flags indicating node properties (bit 0: `is_directory`, bit 1: `is_symlink`).
    pub flags: u8,

    /// Explicit padding bytes to ensure no uninitialized memory and strict 8-byte alignment.
    _padding: [u8; 3],
}

impl FileNode {
    pub const FLAG_DIRECTORY: u8 = 1 << 0;
    pub const FLAG_SYMLINK: u8 = 1 << 1;

    #[must_use]
    #[inline]
    pub fn new(
        name_id: StringId,
        parent: Option<u32>,
        is_dir: bool,
        is_symlink: bool,
        modified_timestamp: i64,
        created_timestamp: i64,
        accessed_timestamp: i64,
    ) -> Self {
        let mut flags = 0u8;
        if is_dir {
            flags |= Self::FLAG_DIRECTORY;
        }
        if is_symlink {
            flags |= Self::FLAG_SYMLINK;
        }
        Self {
            name_id,
            parent: parent.unwrap_or(NO_INDEX),
            first_child: NO_INDEX,
            next_sibling: NO_INDEX,
            size: 0,
            modified_timestamp,
            created_timestamp,
            accessed_timestamp,
            file_count: 0,
            flags,
            _padding: [0; 3],
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_directory(&self) -> bool {
        (self.flags & Self::FLAG_DIRECTORY) != 0
    }

    #[must_use]
    #[inline]
    pub const fn is_symlink(&self) -> bool {
        (self.flags & Self::FLAG_SYMLINK) != 0
    }

    #[must_use]
    #[inline]
    pub const fn parent_opt(&self) -> Option<u32> {
        if self.parent == NO_INDEX {
            None
        } else {
            Some(self.parent)
        }
    }

    #[must_use]
    #[inline]
    pub const fn first_child_opt(&self) -> Option<u32> {
        if self.first_child == NO_INDEX {
            None
        } else {
            Some(self.first_child)
        }
    }

    #[must_use]
    #[inline]
    pub const fn next_sibling_opt(&self) -> Option<u32> {
        if self.next_sibling == NO_INDEX {
            None
        } else {
            Some(self.next_sibling)
        }
    }

    #[must_use]
    #[inline]
    pub fn from_metadata(name_id: StringId, parent: Option<u32>, meta: &EntryMetadata) -> Self {
        let mut node = Self::new(
            name_id,
            parent,
            meta.is_dir,
            meta.is_symlink,
            meta.modified_timestamp,
            meta.created_timestamp,
            meta.accessed_timestamp,
        );
        if !meta.is_dir {
            node.size = meta.len;
        }
        node
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringPool {
    /// Tightly packed byte array containing all string data sequentially
    pub data: Vec<u8>,
    /// Offsets and lengths of strings, indexed by `StringId`
    pub offsets: Vec<StringOffset>,
    /// Hash map for deduplicating newly encountered strings during scan
    pub lookup: HashMap<Vec<u8>, StringId>,
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct StringOffset {
    pub offset: u32,
    pub len: u32,
}

impl StringPool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_insert(&mut self, s: &[u8]) -> StringId {
        if let Some(&id) = self.lookup.get(s) {
            return id;
        }

        let offset = self.data.len() as u32;
        let len = s.len() as u32;
        self.data.extend_from_slice(s);

        let id = StringId(self.offsets.len() as u32);
        self.offsets.push(StringOffset { offset, len });
        self.lookup.insert(s.to_vec(), id);
        id
    }

    #[must_use]
    pub fn get(&self, id: StringId) -> Option<&str> {
        let offset_info = self.offsets.get(id.0 as usize)?;
        let start = offset_info.offset as usize;
        let end = start + offset_info.len as usize;
        if end <= self.data.len() {
            std::str::from_utf8(&self.data[start..end]).ok()
        } else {
            None
        }
    }
}

pub struct FileArenaSnapshot {
    /// Read-only snapshot of the nodes
    pub nodes: Arc<Vec<FileNode>>,
    /// Read-only snapshot of the string pool
    pub string_pool: Arc<StringPool>,
}

impl FileArenaSnapshot {
    /// Reconstruct the full path of a node by walking up parent indices
    #[must_use]
    pub fn get_full_path(&self, node_idx: u32) -> String {
        let mut parts = Vec::new();
        let mut curr = Some(node_idx);
        while let Some(idx) = curr {
            if let Some(node) = self.nodes.get(idx as usize) {
                if let Some(name) = self.string_pool.get(node.name_id) {
                    // Avoid duplicating empty or root names inappropriately
                    if !name.is_empty() {
                        parts.push(name);
                    }
                }
                curr = node.parent_opt();
            } else {
                break;
            }
        }
        parts.reverse();

        // Handle Unix vs Windows root correctly
        if parts.is_empty() {
            return "/".to_string();
        }

        // If the first part starts with a Windows drive letter or "/", join carefully
        let first = parts[0];
        if first.starts_with('/') || first.contains(':') {
            let mut path = first.to_string();
            for part in &parts[1..] {
                if !path.ends_with('/') {
                    path.push('/');
                }
                path.push_str(part);
            }
            path
        } else {
            parts.join("/")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new();
        let id1 = pool.get_or_insert(b"Cargo.toml");
        let id2 = pool.get_or_insert(b"src");
        let id3 = pool.get_or_insert(b"Cargo.toml"); // duplicate

        assert_eq!(id1, id3); // must deduplicate duplicate string
        assert_ne!(id1, id2); // distinct strings must have distinct IDs

        assert_eq!(pool.get(id1), Some("Cargo.toml"));
        assert_eq!(pool.get(id2), Some("src"));
    }

    #[test]
    fn test_path_reconstruction() {
        let mut pool = StringPool::new();
        let root_id = pool.get_or_insert(b"/home/tux");
        let dir_id = pool.get_or_insert(b"Documents");
        let file_id = pool.get_or_insert(b"test.rs");

        // Construct tree
        // Node 0: Root (/home/tux)
        // Node 1: Dir (Documents), parent=0
        // Node 2: File (test.rs), parent=1
        let nodes = vec![
            FileNode::new(root_id, None, true, false, 0, 0, 0),
            FileNode::new(dir_id, Some(0), true, false, 0, 0, 0),
            FileNode::new(file_id, Some(1), false, false, 0, 0, 0),
        ];

        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(nodes),
            string_pool: Arc::new(pool),
        };

        assert_eq!(snapshot.get_full_path(0), "/home/tux");
        assert_eq!(snapshot.get_full_path(1), "/home/tux/Documents");
        assert_eq!(snapshot.get_full_path(2), "/home/tux/Documents/test.rs");
    }
}

#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub len: u64,
    pub modified_timestamp: i64,
    pub created_timestamp: i64,
    pub accessed_timestamp: i64,
    pub file_id: (u64, u64),
}

impl EntryMetadata {
    pub fn from_dir_entry(entry: &std::fs::DirEntry) -> Option<Self> {
        let metadata = entry.metadata().ok()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = metadata.is_dir();
        let is_symlink = metadata.is_symlink();
        let len = metadata.len();

        let modified_timestamp = metadata
            .modified()
            .map_or(0, crate::model::time_utils::system_time_to_unix_timestamp);
        let created_timestamp = metadata
            .created()
            .map_or(0, crate::model::time_utils::system_time_to_unix_timestamp);
        let accessed_timestamp = metadata
            .accessed()
            .map_or(0, crate::model::time_utils::system_time_to_unix_timestamp);

        #[cfg(unix)]
        let file_id = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };

        #[cfg(windows)]
        let file_id = {
            use std::os::windows::fs::MetadataExt;
            (
                metadata.volume_serial_number().unwrap_or(0) as u64,
                metadata.file_index().unwrap_or(0),
            )
        };

        #[cfg(not(any(unix, windows)))]
        let file_id = (0, 0);

        Some(Self {
            name,
            is_dir,
            is_symlink,
            len,
            modified_timestamp,
            created_timestamp,
            accessed_timestamp,
            file_id,
        })
    }
}
