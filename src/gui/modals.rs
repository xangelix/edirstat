use std::{
    borrow::Cow,
    sync::{Arc, atomic::Ordering},
};

use eframe::egui;
use fluent_zero::t;

use super::{GuiApp, theme};
use crate::arena::{FileArenaSnapshot, NodeStorage, precompute_dir_counts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    Delete,
    Trash,
    About,
    DeleteDuplicates,
    TrashDuplicates,
    HardlinkDuplicates,
    SoftlinkDuplicates,
    HowItWorks,
    AdminWarning,
}

fn count_nested_stats(
    nodes: &[crate::arena::FileNode],
    idx: u32,
    files: &mut usize,
    dirs: &mut usize,
) {
    if idx as usize >= nodes.len() {
        return;
    }
    let node = &nodes[idx as usize];
    if node.is_directory() {
        *dirs += 1;
        let mut curr = node.first_child;
        while curr != crate::arena::NO_INDEX {
            count_nested_stats(nodes, curr, files, dirs);
            curr = nodes[curr as usize].next_sibling;
        }
    } else {
        *files += 1;
    }
}

fn collect_descendants(nodes: &[crate::arena::FileNode], idx: u32, out: &mut Vec<u32>) {
    if idx as usize >= nodes.len() {
        return;
    }
    let mut curr = nodes[idx as usize].first_child;
    while curr != crate::arena::NO_INDEX {
        out.push(curr);
        collect_descendants(nodes, curr, out);
        curr = nodes[curr as usize].next_sibling;
    }
}

struct WalkCtx<'a> {
    cloned_nodes: &'a mut Vec<crate::arena::FileNode>,
    string_pool: &'a mut crate::arena::StringPool,
    last_child_map: &'a mut Vec<u32>,
    traversal_stats: &'a crate::engine::traversal::TraversalStats,
    ancestors: &'a mut smallvec::SmallVec<[(u64, u64); 16]>,
}

fn walk_dir(dir_path: &std::path::Path, parent_idx: u32, dir_idx: u32, ctx: &mut WalkCtx<'_>) {
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        return;
    };
    ctx.traversal_stats
        .dirs_scanned
        .fetch_add(1, Ordering::SeqCst);

    for entry_res in entries {
        let Ok(entry) = entry_res else {
            continue;
        };
        let Some(meta) = crate::arena::EntryMetadata::from_dir_entry(&entry) else {
            continue;
        };

        // Cycle Detection
        if meta.file_id != (0, 0) && ctx.ancestors.contains(&meta.file_id) {
            continue;
        }

        let name_id = ctx.string_pool.get_or_insert(meta.name.as_bytes());
        let child_idx = ctx.cloned_nodes.len() as u32;

        let node = crate::arena::FileNode::from_metadata(name_id, Some(parent_idx), &meta);
        ctx.cloned_nodes.push(node);
        ctx.last_child_map.push(crate::arena::NO_INDEX);

        // Connect to parent sibling chain
        let p_idx = parent_idx as usize;
        let last_child = ctx.last_child_map[p_idx];
        if last_child == crate::arena::NO_INDEX {
            ctx.cloned_nodes[p_idx].first_child = child_idx;
        } else {
            ctx.cloned_nodes[last_child as usize].next_sibling = child_idx;
        }
        ctx.last_child_map[p_idx] = child_idx;

        if meta.is_dir {
            if meta.file_id != (0, 0) {
                ctx.ancestors.push(meta.file_id);
            }
            walk_dir(&entry.path(), child_idx, dir_idx, ctx);
            if meta.file_id != (0, 0) {
                ctx.ancestors.pop();
            }
        } else {
            ctx.traversal_stats
                .files_scanned
                .fetch_add(1, Ordering::SeqCst);
            ctx.traversal_stats
                .bytes_scanned
                .fetch_add(meta.len as usize, Ordering::SeqCst);

            // Propagate size and count upwards through parent indices up to dir_idx
            let mut current_idx = Some(parent_idx);
            while let Some(idx) = current_idx {
                ctx.cloned_nodes[idx as usize].size += meta.len;
                ctx.cloned_nodes[idx as usize].file_count += 1;
                if idx == dir_idx {
                    break;
                }
                current_idx = ctx.cloned_nodes[idx as usize].parent_opt();
            }
        }
    }
}

impl GuiApp {
    /// Performs a zero-copy update to the active snapshot by unlinking deleted nodes,
    /// backtracking size weights up to the root, and swapping the updated tree structure.
    pub(crate) fn remove_nodes_from_snapshot(&mut self, target_indices: &[u32]) {
        if target_indices.is_empty() {
            return;
        }

        let current_snap = self.shared_state.current_snapshot.load();
        let mut cloned_nodes = current_snap.nodes.to_vec();

        let mut files_to_remove = 0;
        let mut dirs_to_remove = 0;
        let mut bytes_to_remove = 0u64;

        for &node_idx in target_indices {
            let idx = node_idx as usize;
            if idx >= cloned_nodes.len() {
                continue;
            }
            bytes_to_remove += cloned_nodes[idx].size;
            count_nested_stats(
                &cloned_nodes,
                node_idx,
                &mut files_to_remove,
                &mut dirs_to_remove,
            );
        }

        self.traversal_engine
            .stats()
            .files_scanned
            .fetch_sub(files_to_remove, Ordering::SeqCst);
        self.traversal_engine
            .stats()
            .dirs_scanned
            .fetch_sub(dirs_to_remove, Ordering::SeqCst);
        self.traversal_engine
            .stats()
            .bytes_scanned
            .fetch_sub(bytes_to_remove as usize, Ordering::SeqCst);

        // Clear chart caches to force re-computation
        self.size_dist_chart.cached_counts = None;
        self.dir_comp_chart.children_composition.clear();

        for &node_idx in target_indices {
            let node_idx = node_idx as usize;
            if node_idx >= cloned_nodes.len() {
                continue;
            }

            let node_size = cloned_nodes[node_idx].size;
            let parent_idx = cloned_nodes[node_idx].parent;
            let is_dir = cloned_nodes[node_idx].is_directory();

            // 1. Unlink the deleted item from its parent's sibling chain
            if parent_idx != crate::arena::NO_INDEX {
                let p_idx = parent_idx as usize;
                let mut prev_sibling: Option<u32> = None;
                let mut curr_sibling = cloned_nodes[p_idx].first_child;

                while curr_sibling != crate::arena::NO_INDEX {
                    if curr_sibling == node_idx as u32 {
                        let next_sib = cloned_nodes[node_idx].next_sibling;
                        if let Some(prev) = prev_sibling {
                            cloned_nodes[prev as usize].next_sibling = next_sib;
                        } else {
                            cloned_nodes[p_idx].first_child = next_sib;
                        }
                        break;
                    }
                    // Explicitly advance the pointer
                    prev_sibling = Some(curr_sibling);
                    curr_sibling = cloned_nodes[curr_sibling as usize].next_sibling;
                }
            }

            // 2. Roll back size metrics and file count up the ancestral line
            let mut current_parent = if parent_idx == crate::arena::NO_INDEX {
                None
            } else {
                Some(parent_idx)
            };
            while let Some(p_idx) = current_parent {
                let p_node = &mut cloned_nodes[p_idx as usize];
                p_node.size = p_node.size.saturating_sub(node_size);
                if !is_dir {
                    p_node.file_count = p_node.file_count.saturating_sub(1);
                }
                current_parent = p_node.parent_opt();
            }

            // 3. Isolate the node
            cloned_nodes[node_idx].size = 0;
            cloned_nodes[node_idx].file_count = 0;
            cloned_nodes[node_idx].first_child = crate::arena::NO_INDEX;
            cloned_nodes[node_idx].next_sibling = crate::arena::NO_INDEX;
        }

        let dir_counts = Arc::new(precompute_dir_counts(&cloned_nodes));
        let new_snapshot = crate::arena::FileArenaSnapshot {
            nodes: std::sync::Arc::new(NodeStorage::Owned(cloned_nodes)),
            string_pool: current_snap.string_pool.clone(),
            dir_counts,
        };
        self.shared_state
            .current_snapshot
            .store(std::sync::Arc::new(new_snapshot));
    }

    pub(crate) fn execute_deletion(
        &mut self,
        target_indices: &[u32],
        to_trash: bool,
        snapshot: &FileArenaSnapshot,
    ) {
        let mut successfully_deleted = Vec::new();
        let mut failures = Vec::new();
        for &idx in target_indices {
            let path_str = snapshot.get_full_path(idx);
            let path = std::path::Path::new(&path_str);
            if path.exists() {
                let result = if to_trash {
                    trash::delete(path).map_err(|e| (e.to_string(), is_permission_denied_trash(&e)))
                } else if path.is_dir() {
                    std::fs::remove_dir_all(path)
                        .map_err(|e| (e.to_string(), is_permission_denied_io(&e)))
                } else {
                    std::fs::remove_file(path)
                        .map_err(|e| (e.to_string(), is_permission_denied_io(&e)))
                };

                if let Err((err_msg, is_perm)) = result {
                    println!(
                        "Failed to delete/trash path {}: {}",
                        path.display(),
                        err_msg
                    );
                    failures.push((path_str, err_msg, is_perm));
                } else {
                    successfully_deleted.push(idx);
                }
            } else {
                successfully_deleted.push(idx);
            }
        }

        if to_trash {
            if !successfully_deleted.is_empty() {
                crate::gui::toast_success(format!(
                    "Moved {} item(s) to trash",
                    successfully_deleted.len()
                ));
            }
            if !failures.is_empty() {
                let perm_count = failures.iter().filter(|&(_, _, is_perm)| *is_perm).count();
                if perm_count == failures.len() {
                    crate::gui::toast_error(format!(
                        "Failed to move {} item(s) to trash (Permission Denied). Try running with elevated privileges.",
                        failures.len()
                    ));
                } else if perm_count > 0 {
                    crate::gui::toast_error(format!(
                        "Failed to move {} item(s) to trash ({} due to Permission Denied).",
                        failures.len(),
                        perm_count
                    ));
                } else {
                    crate::gui::toast_error(format!(
                        "Failed to move {} item(s) to trash",
                        failures.len()
                    ));
                }
            }
        } else {
            if !successfully_deleted.is_empty() {
                crate::gui::toast_success(format!(
                    "Permanently deleted {} item(s)",
                    successfully_deleted.len()
                ));
            }
            if !failures.is_empty() {
                let perm_count = failures.iter().filter(|&(_, _, is_perm)| *is_perm).count();
                if perm_count == failures.len() {
                    crate::gui::toast_error(format!(
                        "Failed to delete {} item(s) (Permission Denied). Try running with elevated privileges.",
                        failures.len()
                    ));
                } else if perm_count > 0 {
                    crate::gui::toast_error(format!(
                        "Failed to delete {} item(s) ({} due to Permission Denied).",
                        failures.len(),
                        perm_count
                    ));
                } else {
                    crate::gui::toast_error(format!("Failed to delete {} item(s)", failures.len()));
                }
            }
        }

        if !successfully_deleted.is_empty() {
            {
                let mut results = self.deduplicator_results.write();
                for group in &mut results.groups {
                    let mut i = 0;
                    while i < group.nodes.len() {
                        if successfully_deleted.contains(&group.nodes[i]) {
                            group.nodes.remove(i);
                            if i < group.file_ids.len() {
                                group.file_ids.remove(i);
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                results.groups.retain(|group| group.nodes.len() >= 2);
                results.rebuild_flat_rows(snapshot);
            }

            self.selected_duplicates
                .retain(|idx| !successfully_deleted.contains(idx));

            // Clean up selections inside RoaringBitmap
            for &idx in &successfully_deleted {
                self.table_state.selected_rows.remove(idx);
            }

            self.remove_nodes_from_snapshot(&successfully_deleted);
        }
    }

    pub(crate) fn execute_hardlinking(
        &mut self,
        target_indices: &[u32],
        snapshot: &FileArenaSnapshot,
    ) {
        let mut successfully_linked = Vec::new();
        let mut failures = Vec::new();
        let results_guard = self.deduplicator_results.read();

        for &idx in target_indices {
            let Some(group) = results_guard.groups.iter().find(|g| g.nodes.contains(&idx)) else {
                failures.push((
                    format!("Index {idx}"),
                    "Not found in any duplicate group".to_string(),
                    false,
                ));
                continue;
            };

            // Find a source node in the group that is NOT being replaced to link against
            let Some(&src_idx) = group.nodes.iter().find(|&&n| !target_indices.contains(&n)) else {
                failures.push((
                    format!("Index {idx}"),
                    "No remaining source file in group".to_string(),
                    false,
                ));
                continue;
            };

            let src_path_str = snapshot.get_full_path(src_idx);
            let dst_path_str = snapshot.get_full_path(idx);
            let src_path = std::path::Path::new(&src_path_str);
            let dst_path = std::path::Path::new(&dst_path_str);

            if src_path.exists() && dst_path.exists() {
                let temp_dst = dst_path.with_extension("tmp_hl_bak");
                match std::fs::rename(dst_path, &temp_dst) {
                    Err(e) => {
                        failures.push((dst_path_str, e.to_string(), is_permission_denied_io(&e)));
                    }
                    Ok(()) => {
                        match std::fs::hard_link(src_path, dst_path) {
                            Ok(()) => {
                                let _ = std::fs::remove_file(&temp_dst);
                                successfully_linked.push(idx);
                            }
                            Err(e) => {
                                // Restore backup on failure
                                let _ = std::fs::rename(&temp_dst, dst_path);
                                failures.push((
                                    dst_path_str,
                                    format!("Failed to create hard link: {e}"),
                                    is_permission_denied_io(&e),
                                ));
                            }
                        }
                    }
                }
            } else {
                failures.push((
                    dst_path_str,
                    "Source or destination path does not exist".to_string(),
                    false,
                ));
            }
        }

        if !successfully_linked.is_empty() {
            crate::gui::toast_success(format!(
                "Successfully replaced {} duplicate(s) with hardlinks",
                successfully_linked.len()
            ));
        }
        if !failures.is_empty() {
            let perm_count = failures.iter().filter(|&(_, _, is_perm)| *is_perm).count();
            if perm_count == failures.len() {
                crate::gui::toast_error(format!(
                    "Failed to hardlink {} duplicate(s) (Permission Denied).",
                    failures.len()
                ));
            } else if perm_count > 0 {
                crate::gui::toast_error(format!(
                    "Failed to hardlink {} duplicate(s) ({} due to Permission Denied).",
                    failures.len(),
                    perm_count
                ));
            } else {
                crate::gui::toast_error(format!(
                    "Failed to hardlink {} duplicate(s)",
                    failures.len()
                ));
            }
        }

        if !successfully_linked.is_empty() {
            drop(results_guard);
            {
                let mut results = self.deduplicator_results.write();
                for group in &mut results.groups {
                    let has_any = group.nodes.iter().any(|n| successfully_linked.contains(n));
                    if has_any {
                        for (i, &node_idx) in group.nodes.iter().enumerate() {
                            let path_str = snapshot.get_full_path(node_idx);
                            if let Ok(meta) = std::fs::metadata(path_str) {
                                let file_id = crate::engine::traversal::get_file_id(&meta);
                                if i < group.file_ids.len() {
                                    group.file_ids[i] = file_id;
                                }
                            }
                        }
                    }
                }
                results.rebuild_flat_rows(snapshot);
            }

            self.selected_duplicates
                .retain(|idx| !successfully_linked.contains(idx));
        }
    }

    pub(crate) fn execute_softlinking(
        &mut self,
        target_indices: &[u32],
        snapshot: &FileArenaSnapshot,
    ) {
        let mut successfully_linked = Vec::new();
        let mut failures = Vec::new();
        let results_guard = self.deduplicator_results.read();

        for &idx in target_indices {
            let Some(group) = results_guard.groups.iter().find(|g| g.nodes.contains(&idx)) else {
                failures.push((
                    format!("Index {idx}"),
                    "Not found in any duplicate group".to_string(),
                    false,
                ));
                continue;
            };

            // Find a source node in the group that is NOT being replaced to link against
            let Some(&src_idx) = group.nodes.iter().find(|&&n| !target_indices.contains(&n)) else {
                failures.push((
                    format!("Index {idx}"),
                    "No remaining source file in group".to_string(),
                    false,
                ));
                continue;
            };

            let src_path_str = snapshot.get_full_path(src_idx);
            let dst_path_str = snapshot.get_full_path(idx);
            let src_path = std::path::Path::new(&src_path_str);
            let dst_path = std::path::Path::new(&dst_path_str);

            if src_path.exists() && dst_path.exists() {
                let temp_dst = dst_path.with_extension("tmp_sl_bak");
                if let Err(e) = std::fs::rename(dst_path, &temp_dst) {
                    failures.push((dst_path_str, e.to_string(), is_permission_denied_io(&e)));
                } else {
                    let symlink_result = Self::symlink(src_path, dst_path);

                    match symlink_result {
                        Ok(()) => {
                            let _ = std::fs::remove_file(&temp_dst);
                            successfully_linked.push(idx);
                        }
                        Err(e) => {
                            // Restore backup on failure
                            let _ = std::fs::rename(&temp_dst, dst_path);
                            failures.push((
                                dst_path_str,
                                format!("Failed to create symlink: {e}"),
                                is_permission_denied_io(&e),
                            ));
                        }
                    }
                }
            } else {
                failures.push((
                    dst_path_str,
                    "Source or destination path does not exist".to_string(),
                    false,
                ));
            }
        }

        if !successfully_linked.is_empty() {
            crate::gui::toast_success(format!(
                "Successfully replaced {} duplicate(s) with softlinks",
                successfully_linked.len()
            ));
        }
        if !failures.is_empty() {
            let perm_count = failures.iter().filter(|&(_, _, is_perm)| *is_perm).count();
            if perm_count == failures.len() {
                crate::gui::toast_error(format!(
                    "Failed to softlink {} duplicate(s) (Permission Denied).",
                    failures.len()
                ));
            } else if perm_count > 0 {
                crate::gui::toast_error(format!(
                    "Failed to softlink {} duplicate(s) ({} due to Permission Denied).",
                    failures.len(),
                    perm_count
                ));
            } else {
                crate::gui::toast_error(format!(
                    "Failed to softlink {} duplicate(s)",
                    failures.len()
                ));
            }
        }

        if !successfully_linked.is_empty() {
            drop(results_guard);
            {
                let mut results = self.deduplicator_results.write();
                for group in &mut results.groups {
                    let mut i = 0;
                    while i < group.nodes.len() {
                        if successfully_linked.contains(&group.nodes[i]) {
                            group.nodes.remove(i);
                            if i < group.file_ids.len() {
                                group.file_ids.remove(i);
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                results.groups.retain(|group| group.nodes.len() >= 2);
                results.rebuild_flat_rows(snapshot);
            }

            self.selected_duplicates
                .retain(|idx| !successfully_linked.contains(idx));

            // Clean up selections inside RoaringBitmap
            for &idx in &successfully_linked {
                self.table_state.selected_rows.remove(idx);
            }

            self.remove_nodes_from_snapshot(&successfully_linked);
        }
    }

    #[cfg(target_family = "unix")]
    fn symlink(src_path: &std::path::Path, dst_path: &std::path::Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src_path, dst_path)
    }

    #[cfg(target_family = "windows")]
    fn symlink(src_path: &std::path::Path, dst_path: &std::path::Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(src_path, dst_path)
    }

    #[cfg(not(any(unix, windows)))]
    fn symlink(src_path: &std::path::Path, dst_path: &std::path::Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Symlinks not supported on this platform",
        ))
    }

    pub fn render_modals(&mut self, ctx: &egui::Context, snapshot: &FileArenaSnapshot) {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum DeletionAction {
            DeleteMultiple,
            TrashMultiple,
            DeleteDuplicates,
            TrashDuplicates,
            HardlinkDuplicates,
            SoftlinkDuplicates,
        }

        struct ModalConfig {
            title: Cow<'static, str>,
            border_color: egui::Color32,
            warning_color: egui::Color32,
            header: String,
            info_msg: String,
            warning_msg: Cow<'static, str>,
            checkbox_label: Cow<'static, str>,
            confirm_button_text: Cow<'static, str>,
            paths: Vec<String>,
            action: DeletionAction,
        }

        let modal_config = match self.active_modal {
            Some(ActiveModal::Delete) => {
                let idxs = &self.delete_node_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths: Vec<String> = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-delete-title"),
                        border_color: theme::DELETION_BORDER,
                        warning_color: theme::DELETION_WARNING,
                        header: t!("modal-delete-header").into_owned(),
                        info_msg: t!("modal-delete-info", { "size" => size_str.as_str() })
                            .into_owned(),
                        warning_msg: t!("modal-delete-warning"),
                        checkbox_label: t!("modal-delete-checkbox"),
                        confirm_button_text: t!("modal-delete-confirm"),
                        paths,
                        action: DeletionAction::DeleteMultiple,
                    })
                }
            }
            Some(ActiveModal::Trash) => {
                let idxs = &self.delete_node_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths: Vec<String> = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-trash-title"),
                        border_color: theme::TRASH_BORDER,
                        warning_color: theme::TRASH_WARNING,
                        header: t!("modal-trash-header").into_owned(),
                        info_msg: t!("modal-trash-info", { "size" => size_str.as_str() })
                            .into_owned(),
                        warning_msg: t!("modal-trash-warning"),
                        checkbox_label: t!("modal-trash-checkbox"),
                        confirm_button_text: t!("modal-trash-confirm"),
                        paths,
                        action: DeletionAction::TrashMultiple,
                    })
                }
            }
            Some(ActiveModal::DeleteDuplicates) => {
                let idxs = &self.delete_duplicates_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-delete-duplicates-title"),
                        border_color: theme::DELETION_BORDER,
                        warning_color: theme::DELETION_WARNING,
                        header: t!("modal-delete-duplicates-header").into_owned(),
                        info_msg:
                            t!("modal-delete-duplicates-info", { "size" => size_str.as_str() })
                                .into_owned(),
                        warning_msg: t!("modal-delete-duplicates-warning"),
                        checkbox_label: t!("modal-delete-duplicates-checkbox"),
                        confirm_button_text: t!("modal-delete-duplicates-confirm"),
                        paths,
                        action: DeletionAction::DeleteDuplicates,
                    })
                }
            }
            Some(ActiveModal::TrashDuplicates) => {
                let idxs = &self.delete_duplicates_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-trash-duplicates-title"),
                        border_color: theme::TRASH_BORDER,
                        warning_color: theme::TRASH_WARNING,
                        header: t!("modal-trash-duplicates-header").into_owned(),
                        info_msg:
                            t!("modal-trash-duplicates-info", { "size" => size_str.as_str() })
                                .into_owned(),
                        warning_msg: t!("modal-trash-duplicates-warning"),
                        checkbox_label: t!("modal-trash-duplicates-checkbox"),
                        confirm_button_text: t!("modal-trash-duplicates-confirm"),
                        paths,
                        action: DeletionAction::TrashDuplicates,
                    })
                }
            }
            Some(ActiveModal::HardlinkDuplicates) => {
                let idxs = &self.delete_duplicates_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-hardlink-duplicates-title"),
                        border_color: theme::BUTTON_ORANGE,
                        warning_color: theme::BUTTON_ORANGE_HOVER,
                        header: t!("modal-hardlink-duplicates-header").into_owned(),
                        info_msg: t!("modal-hardlink-duplicates-info", {
                            "count" => idxs.len(),
                            "size" => size_str.as_str()
                        })
                        .into_owned(),
                        warning_msg: t!("modal-hardlink-duplicates-warning"),
                        checkbox_label: t!("modal-hardlink-duplicates-checkbox"),
                        confirm_button_text: t!("modal-hardlink-duplicates-confirm"),
                        paths,
                        action: DeletionAction::HardlinkDuplicates,
                    })
                }
            }
            Some(ActiveModal::SoftlinkDuplicates) => {
                let idxs = &self.delete_duplicates_indices;
                if idxs.is_empty() {
                    None
                } else {
                    let total_size: u64 = idxs
                        .iter()
                        .map(|&idx| snapshot.nodes[idx as usize].size)
                        .sum();
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(total_size)
                        .to_string();
                    let paths = idxs
                        .iter()
                        .map(|&idx| {
                            crate::model::arena::clean_unc_path(&snapshot.get_full_path(idx))
                                .into_owned()
                        })
                        .collect();
                    Some(ModalConfig {
                        title: t!("modal-softlink-duplicates-title"),
                        border_color: theme::BUTTON_ORANGE,
                        warning_color: theme::BUTTON_ORANGE_HOVER,
                        header: t!("modal-softlink-duplicates-header").into_owned(),
                        info_msg: t!("modal-softlink-duplicates-info", {
                            "count" => idxs.len(),
                            "size" => size_str.as_str()
                        })
                        .into_owned(),
                        warning_msg: t!("modal-softlink-duplicates-warning"),
                        checkbox_label: t!("modal-softlink-duplicates-checkbox"),
                        confirm_button_text: t!("modal-softlink-duplicates-confirm"),
                        paths,
                        action: DeletionAction::SoftlinkDuplicates,
                    })
                }
            }
            _ => None,
        };

        if let Some(cfg) = modal_config {
            let mut open = true;
            egui::Window::new(cfg.title)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(!cfg.paths.is_empty())
                .default_width(550.0)
                .open(&mut open)
                .title_bar(false) // Disable default system title bar for uniform styling
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .fill(theme::BG_WINDOW_SLATE)
                        .stroke(egui::Stroke::new(
                            1.2f32,
                            egui::Color32::from_rgb(74, 85, 104),
                        )) // Bright, crisp slate border
                        .inner_margin(egui::Margin::ZERO) // Fill header completely to the borders
                        .corner_radius(8.0),
                )
                .show(ctx, |ui| {
                    // Custom Header Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new(&cfg.header)
                                        .color(ui.visuals().strong_text_color())
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.button("❌");
                                        if close_btn.clicked() {
                                            self.active_modal = None;
                                        }
                                    },
                                );
                            });
                        });

                    // Thin, subtle separator line matching normal panels
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.center().y,
                        egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE),
                    );

                    // Modal Content Frame
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                let path_exists = if cfg.paths.len() == 1 {
                                    let raw_path = match self.active_modal {
                                        Some(ActiveModal::Delete | ActiveModal::Trash) => {
                                            self.delete_node_indices.first().map_or_else(
                                                || cfg.paths[0].clone(),
                                                |&idx| snapshot.get_full_path(idx),
                                            )
                                        }
                                        Some(
                                            ActiveModal::DeleteDuplicates
                                            | ActiveModal::TrashDuplicates
                                            | ActiveModal::HardlinkDuplicates
                                            | ActiveModal::SoftlinkDuplicates,
                                        ) => self.delete_duplicates_indices.first().map_or_else(
                                            || cfg.paths[0].clone(),
                                            |&idx| snapshot.get_full_path(idx),
                                        ),
                                        _ => cfg.paths[0].clone(),
                                    };
                                    std::path::Path::new(&raw_path).exists()
                                } else {
                                    true
                                };

                                if path_exists {
                                    ui.label(if cfg.paths.len() > 1 {
                                        t!("modal-process-multiple", { "count" => cfg.paths.len() }).into_owned()
                                    } else {
                                        t!("modal-process-single").into_owned()
                                    });

                                    ui.add_space(8.0);

                                    // Display list of selected items inside a high-contrast container
                                    let path_bg = theme::BG_PANEL_SLATE;
                                    egui::Frame::new()
                                        .fill(path_bg)
                                        .stroke(egui::Stroke::new(
                                            1.0f32,
                                            theme::STROKE_BORDER_SLATE,
                                        ))
                                        .inner_margin(egui::Margin::same(12))
                                        .corner_radius(4.0)
                                        .show(ui, |ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Wrap);
                                            ui.colored_label(
                                                ui.visuals().strong_text_color(),
                                                &cfg.paths[0],
                                            );
                                            if cfg.paths.len() > 1 {
                                                ui.add_space(4.0);
                                                egui::ScrollArea::vertical()
                                                    .max_height(150.0)
                                                    .show(ui, |ui| {
                                                        for path in &cfg.paths[1..] {
                                                            ui.small(path);
                                                        }
                                                    });
                                            }
                                        });

                                    ui.add_space(8.0);

                                    ui.horizontal(|ui| {
                                        ui.label(t!("modal-details-label"));
                                        ui.label(egui::RichText::new(&cfg.info_msg).strong());
                                    });

                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);

                                    // Warning explanation text area
                                    ui.horizontal(|ui| {
                                        ui.colored_label(cfg.warning_color, "⚠");
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                        ui.label(egui::RichText::new(cfg.warning_msg));
                                    });

                                    ui.add_space(12.0);

                                    // Checkbox alignment
                                    ui.checkbox(
                                        &mut self.delete_confirm_checked,
                                        cfg.checkbox_label,
                                    );
                                    ui.add_space(8.0);

                                    if matches!(
                                        self.active_modal,
                                        Some(ActiveModal::Delete | ActiveModal::Trash)
                                    ) {
                                        ui.checkbox(
                                            &mut self.remember_confirmation,
                                            t!("modal-remember-confirmation"),
                                        );
                                        ui.add_space(16.0);
                                    } else {
                                        ui.add_space(8.0);
                                    }

                                    // Action Buttons
                                    ui.horizontal(|ui| {
                                        if ui.button(t!("modal-cancel-btn")).clicked() {
                                            self.active_modal = None;
                                        }

                                        let confirm_btn = egui::Button::new(
                                            egui::RichText::new(cfg.confirm_button_text)
                                                .color(theme::COLOR_WHITE)
                                                .strong(),
                                        )
                                        .fill(cfg.border_color);

                                        let confirm_res = ui
                                            .add_enabled(self.delete_confirm_checked, confirm_btn);
                                        if confirm_res.clicked() {
                                            match cfg.action {
                                                DeletionAction::DeleteMultiple => {
                                                    if self.remember_confirmation {
                                                        self.deletion_confirmation = false;
                                                    }
                                                    self.execute_deletion(
                                                        &self.delete_node_indices.clone(),
                                                        false,
                                                        snapshot,
                                                    );
                                                    self.delete_node_indices.clear();
                                                }
                                                DeletionAction::TrashMultiple => {
                                                    if self.remember_confirmation {
                                                        self.trash_confirmation = false;
                                                    }
                                                    self.execute_deletion(
                                                        &self.delete_node_indices.clone(),
                                                        true,
                                                        snapshot,
                                                    );
                                                    self.delete_node_indices.clear();
                                                }
                                                DeletionAction::DeleteDuplicates => {
                                                    self.execute_deletion(
                                                        &self.delete_duplicates_indices.clone(),
                                                        false,
                                                        snapshot,
                                                    );
                                                    self.delete_duplicates_indices.clear();
                                                }
                                                DeletionAction::TrashDuplicates => {
                                                    self.execute_deletion(
                                                        &self.delete_duplicates_indices.clone(),
                                                        true,
                                                        snapshot,
                                                    );
                                                    self.delete_duplicates_indices.clear();
                                                }
                                                DeletionAction::HardlinkDuplicates => {
                                                    self.execute_hardlinking(
                                                        &self.delete_duplicates_indices.clone(),
                                                        snapshot,
                                                    );
                                                    self.delete_duplicates_indices.clear();
                                                }
                                                DeletionAction::SoftlinkDuplicates => {
                                                    self.execute_softlinking(
                                                        &self.delete_duplicates_indices.clone(),
                                                        snapshot,
                                                    );
                                                    self.delete_duplicates_indices.clear();
                                                }
                                            }
                                            self.active_modal = None;
                                        }
                                    });
                                } else {
                                    ui.heading(
                                        egui::RichText::new(t!("modal-path-not-exist-title"))
                                            .color(theme::DELETION_WARNING)
                                            .strong(),
                                    );
                                    ui.separator();
                                    ui.label(t!("modal-path-not-exist-msg"));
                                    ui.colored_label(
                                        ui.visuals().strong_text_color(),
                                        &cfg.paths[0],
                                    );
                                    ui.add_space(16.0);
                                    if ui.button(t!("modal-close-btn")).clicked() {
                                        self.active_modal = None;
                                    }
                                }
                            });
                        });
                });
            if !open {
                self.active_modal = None;
            }
        }

        // Render the Admin Access Recommendation Modal
        if self.active_modal == Some(ActiveModal::AdminWarning) {
            let mut open = true;
            egui::Window::new(t!("modal-elevation-title"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .title_bar(false) // Disable default system title bar to match custom styles
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .fill(theme::BG_WINDOW_SLATE)
                        .stroke(egui::Stroke::new(
                            1.2f32,
                            egui::Color32::from_rgb(74, 85, 104),
                        ))
                        .inner_margin(egui::Margin::ZERO)
                        .corner_radius(8.0),
                )
                .show(ctx, |ui| {
                    // Header Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new(t!("modal-elevation-title"))
                                        .color(ui.visuals().strong_text_color())
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.button("❌");
                                        if close_btn.clicked() {
                                            self.active_modal = None;
                                        }
                                    },
                                );
                            });
                        });

                    // Separator line
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.center().y,
                        egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE),
                    );

                    // Content Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                ui.label(t!("modal-elevation-desc"));
                                ui.add_space(10.0);

                                // Warning highlight box
                                egui::Frame::new()
                                    .fill(theme::BG_PANEL_SLATE)
                                    .stroke(egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE))
                                    .inner_margin(egui::Margin::same(12))
                                    .corner_radius(4.0)
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.colored_label(theme::WARNING_RED, "⚡");
                                            ui.vertical(|ui| {
                                                ui.strong(t!("modal-elevation-mft-disabled"));
                                                ui.small(t!("modal-elevation-mft-desc"));
                                            });
                                        });
                                    });

                                ui.add_space(12.0);
                                ui.label(t!("modal-elevation-relaunch-prompt"));
                                ui.add_space(16.0);

                                // Modal actions footer
                                ui.horizontal(|ui| {
                                    if ui.button(t!("modal-elevation-continue-std")).clicked() {
                                        self.active_modal = None;
                                    }

                                    let relaunch_btn = egui::Button::new(
                                        egui::RichText::new(t!("modal-elevation-relaunch-btn"))
                                            .color(theme::COLOR_WHITE)
                                            .strong(),
                                    )
                                    .fill(theme::BUTTON_ORANGE);

                                    if ui.add(relaunch_btn).clicked() {
                                        #[cfg(target_os = "windows")]
                                        {
                                            let _ = cli_or_gui::relaunch_as_elevated();
                                        }
                                    }
                                });
                            });
                        });
                });
            if !open {
                self.active_modal = None;
            }
        }

        // Render Help -> About Modal Popup
        if self.active_modal == Some(ActiveModal::About) {
            let mut open = true;
            egui::Window::new(t!("modal-about-title"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .title_bar(false) // Disable default system title bar
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .fill(theme::BG_WINDOW_SLATE)
                        .stroke(egui::Stroke::new(
                            1.2f32,
                            egui::Color32::from_rgb(74, 85, 104),
                        )) // Matching bright slate border
                        .inner_margin(egui::Margin::ZERO)
                        .corner_radius(8.0),
                )
                .show(ctx, |ui| {
                    // Custom Header Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new(t!("modal-about-title"))
                                        .color(ui.visuals().strong_text_color())
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.button("❌");
                                        if close_btn.clicked() {
                                            self.active_modal = None;
                                        }
                                    },
                                );
                            });
                        });

                    // Thin, subtle separator line
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.center().y,
                        egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE),
                    );

                    // Content Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../../assets/img/logo-nosubtext-transparent.svg"
                                    ))
                                    .max_height(100.0),
                                );
                                ui.add_space(8.0);

                                ui.label(
                                    egui::RichText::new(t!("modal-about-version", {
                                        "version" => env!("CARGO_PKG_VERSION")
                                    }))
                                    .strong()
                                    .color(ui.visuals().strong_text_color()),
                                );
                                #[cfg(feature = "online")]
                                {
                                    ui.add_space(4.0);
                                    self.draw_update_check_ui(ui);
                                }
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.label(egui::RichText::new(t!("modal-about-author")));
                                ui.add_space(12.0);

                                let info_bg = theme::BG_PANEL_SLATE;
                                egui::Frame::new()
                                    .fill(info_bg)
                                    .stroke(egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE))
                                    .inner_margin(egui::Margin::same(12))
                                    .corner_radius(4.0)
                                    .show(ui, |ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                        ui.label(t!("modal-about-desc1"));
                                        ui.add_space(6.0);
                                        ui.label(t!("modal-about-desc2"));
                                        ui.add_space(6.0);
                                        ui.label(t!("modal-about-desc3"));
                                    });

                                ui.add_space(16.0);

                                if ui.button(t!("modal-about-licenses-btn")).clicked() {
                                    self.show_licenses = true;
                                }
                            });
                        });
                });
            if !open {
                self.active_modal = None;
            }
        }

        // Render "How Deduplication Works" Modal
        if self.active_modal == Some(ActiveModal::HowItWorks) {
            let mut open = true;
            egui::Window::new("ℹ How Deduplication Works")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .title_bar(false) // Disable default system title bar
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .fill(theme::BG_WINDOW_SLATE)
                        .stroke(egui::Stroke::new(
                            1.2f32,
                            egui::Color32::from_rgb(74, 85, 104),
                        ))
                        .inner_margin(egui::Margin::ZERO)
                        .corner_radius(8.0),
                )
                .show(ctx, |ui| {
                    // Custom Header Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new(t!("modal-how-dedup-title"))
                                        .color(ui.visuals().strong_text_color())
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.button("❌");
                                        if close_btn.clicked() {
                                            self.active_modal = None;
                                        }
                                    },
                                );
                            });
                        });

                    // Thin, subtle separator line
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.center().y,
                        egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE),
                    );

                    // Content Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(450.0)
                                    .auto_shrink([false, true]) // Lock scrollbar against the right edge
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                    )
                                    .content_margin(egui::Margin {
                                        left: 0,
                                        right: 14, // Clean separation padding before the scrollbar
                                        top: 0,
                                        bottom: 0,
                                    })
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Wrap);

                                            ui.label(t!("modal-how-dedup-desc1"));
                                            ui.add_space(10.0);

                                            ui.strong(t!("modal-how-dedup-pipeline-title"));
                                            ui.add_space(6.0);

                                            let steps = [
                                                (
                                                    t!("modal-how-dedup-step1-title"),
                                                    t!("modal-how-dedup-step1-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step2-title"),
                                                    t!("modal-how-dedup-step2-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step3-title"),
                                                    t!("modal-how-dedup-step3-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step4-title"),
                                                    t!("modal-how-dedup-step4-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step5-title"),
                                                    t!("modal-how-dedup-step5-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step6-title"),
                                                    t!("modal-how-dedup-step6-desc"),
                                                ),
                                                (
                                                    t!("modal-how-dedup-step7-title"),
                                                    t!("modal-how-dedup-step7-desc"),
                                                ),
                                            ];

                                            for (title, desc) in steps {
                                                egui::Frame::new()
                                                    .fill(theme::BG_PANEL_SLATE)
                                                    .stroke(egui::Stroke::new(
                                                        1.0f32,
                                                        theme::STROKE_BORDER_SLATE,
                                                    ))
                                                    .inner_margin(egui::Margin::same(10))
                                                    .corner_radius(4.0)
                                                    .show(ui, |ui| {
                                                        // Stretch step frame to align perfectly with content bounds
                                                        ui.set_min_width(ui.available_width());
                                                        ui.style_mut().wrap_mode =
                                                            Some(egui::TextWrapMode::Wrap);
                                                        ui.strong(title);
                                                        ui.add_space(2.0);
                                                        ui.small(desc);
                                                    });
                                                ui.add_space(6.0);
                                            }

                                            ui.add_space(10.0);
                                            ui.strong(t!("modal-how-dedup-why-title"));
                                            ui.add_space(4.0);
                                            ui.label(t!("modal-how-dedup-why-desc1"));
                                        });
                                    });

                                ui.add_space(16.0);
                                ui.vertical_centered(|ui| {
                                    if ui.button(t!("modal-close-btn")).clicked() {
                                        self.active_modal = None;
                                    }
                                });
                            });
                        });
                });
            if !open {
                self.active_modal = None;
            }
        }

        if self.show_licenses {
            let mut open = true;
            egui::Window::new(t!("modal-licenses-title"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(true)
                .default_width(650.0)
                .default_height(500.0)
                .open(&mut open)
                .title_bar(false)
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .fill(theme::BG_WINDOW_SLATE)
                        .stroke(egui::Stroke::new(
                            1.2f32,
                            egui::Color32::from_rgb(74, 85, 104),
                        ))
                        .inner_margin(egui::Margin::ZERO)
                        .corner_radius(8.0),
                )
                .show(ctx, |ui| {
                    // Custom Header Area
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(
                                    egui::RichText::new(t!("modal-licenses-title"))
                                        .color(ui.visuals().strong_text_color())
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.button("❌");
                                        if close_btn.clicked() {
                                            self.show_licenses = false;
                                        }
                                    },
                                );
                            });
                        });

                    // Thin, subtle separator line matching normal panels
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().hline(
                        rect.left()..=rect.right(),
                        rect.center().y,
                        egui::Stroke::new(1.0f32, theme::STROKE_BORDER_SLATE),
                    );

                    // Modal Content Frame
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(t!("modal-licenses-desc"));
                                ui.add_space(8.0);

                                let mut licenses_text = {
                                    #[cfg(target_os = "linux")]
                                    let bytes =
                                        include_packed::include_packed!("assets/licenses/linux.md");
                                    #[cfg(target_os = "windows")]
                                    let bytes = include_packed::include_packed!(
                                        "assets/licenses/windows.md"
                                    );
                                    #[cfg(target_os = "macos")]
                                    let bytes =
                                        include_packed::include_packed!("assets/licenses/macos.md");
                                    #[cfg(not(any(
                                        target_os = "linux",
                                        target_os = "windows",
                                        target_os = "macos"
                                    )))]
                                    let bytes =
                                        include_packed::include_packed!("assets/licenses/linux.md");

                                    String::from_utf8(bytes).unwrap_or_default()
                                };

                                egui::ScrollArea::vertical()
                                    .max_height(350.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut licenses_text)
                                                .font(egui::TextStyle::Monospace)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(18)
                                                .interactive(true),
                                        );
                                    });

                                ui.add_space(16.0);
                                ui.horizontal(|ui| {
                                    if ui.button(t!("modal-close-btn")).clicked() {
                                        self.show_licenses = false;
                                    }
                                });
                            });
                        });
                });
            if !open {
                self.show_licenses = false;
            }
        }
    }

    pub(crate) fn refresh_directory_subtree(&mut self, dir_idx: u32) {
        self.refresh_directory_subtrees(&[dir_idx]);
    }

    pub(crate) fn refresh_directory_subtrees(&mut self, dir_indices: &[u32]) {
        if dir_indices.is_empty() {
            return;
        }
        self.scan_start_time = Some(std::time::Instant::now());
        self.total_scan_duration = None;
        let current_snap = self.shared_state.current_snapshot.load();
        let mut valid_indices = Vec::new();
        for &idx in dir_indices {
            let path_str = current_snap.get_full_path(idx);
            let path = std::path::PathBuf::from(path_str);
            if path.exists() && path.is_dir() {
                valid_indices.push((idx, path));
            }
        }
        if valid_indices.is_empty() {
            return;
        }

        let state = self.shared_state.clone();
        let traversal_stats = self.traversal_engine.stats().clone();

        state.is_scanning.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            let current_snap = state.current_snapshot.load();
            let mut cloned_nodes = current_snap.nodes.to_vec();
            let mut string_pool = (*current_snap.string_pool).clone();

            for (dir_idx, path) in valid_indices {
                // 1. Collect and delete old descendants of dir_idx
                let mut descendants = Vec::new();
                collect_descendants(&cloned_nodes, dir_idx, &mut descendants);

                let old_size = cloned_nodes[dir_idx as usize].size;
                let old_file_count = cloned_nodes[dir_idx as usize].file_count;

                // Roll back ancestors size/counts
                let mut current_parent = cloned_nodes[dir_idx as usize].parent_opt();
                while let Some(p_idx) = current_parent {
                    let p_node = &mut cloned_nodes[p_idx as usize];
                    p_node.size = p_node.size.saturating_sub(old_size);
                    p_node.file_count = p_node.file_count.saturating_sub(old_file_count);
                    current_parent = p_node.parent_opt();
                }

                let mut files_removed = 0;
                let mut dirs_removed = 0;
                for &idx in &descendants {
                    let node = &cloned_nodes[idx as usize];
                    if node.is_directory() {
                        dirs_removed += 1;
                    } else {
                        files_removed += 1;
                    }
                }

                traversal_stats
                    .files_scanned
                    .fetch_sub(files_removed, Ordering::SeqCst);
                traversal_stats
                    .dirs_scanned
                    .fetch_sub(dirs_removed, Ordering::SeqCst);
                traversal_stats
                    .bytes_scanned
                    .fetch_sub(old_size as usize, Ordering::SeqCst);

                // Isolate old descendants
                for &idx in &descendants {
                    let idx = idx as usize;
                    cloned_nodes[idx].size = 0;
                    cloned_nodes[idx].file_count = 0;
                    cloned_nodes[idx].first_child = crate::arena::NO_INDEX;
                    cloned_nodes[idx].next_sibling = crate::arena::NO_INDEX;
                    // Do NOT set parent to NO_INDEX to avoid them being treated as ghost root nodes
                }

                cloned_nodes[dir_idx as usize].first_child = crate::arena::NO_INDEX;
                cloned_nodes[dir_idx as usize].size = 0;
                cloned_nodes[dir_idx as usize].file_count = 0;

                // 2. Scan the directory recursively on disk and append new nodes
                let mut last_child_map = vec![crate::arena::NO_INDEX; cloned_nodes.len()];

                // --- BUILD ANCESTORS FOR CYCLE DETECTION ---
                let mut ancestors: smallvec::SmallVec<[(u64, u64); 16]> = smallvec::smallvec![];
                for ancestor_path in path.ancestors() {
                    if let Ok(meta) = std::fs::metadata(ancestor_path) {
                        ancestors.push(crate::engine::traversal::get_file_id(&meta));
                    }
                }
                // Reverse so that the root ancestor is first and the target path is last
                ancestors.reverse();

                let mut walk_ctx = WalkCtx {
                    cloned_nodes: &mut cloned_nodes,
                    string_pool: &mut string_pool,
                    last_child_map: &mut last_child_map,
                    traversal_stats: &traversal_stats,
                    ancestors: &mut ancestors,
                };
                walk_dir(&path, dir_idx, dir_idx, &mut walk_ctx);

                // 3. Now propagate the new size/counts of dir_idx to all its ancestors!
                let new_size = cloned_nodes[dir_idx as usize].size;
                let new_file_count = cloned_nodes[dir_idx as usize].file_count;

                let mut current_parent = cloned_nodes[dir_idx as usize].parent_opt();
                while let Some(p_idx) = current_parent {
                    let p_node = &mut cloned_nodes[p_idx as usize];
                    p_node.size += new_size;
                    p_node.file_count += new_file_count;
                    current_parent = p_node.parent_opt();
                }
            }

            // 4. Swap snapshot
            let dir_counts = Arc::new(precompute_dir_counts(&cloned_nodes));
            let new_snapshot = FileArenaSnapshot {
                nodes: Arc::new(NodeStorage::Owned(cloned_nodes)),
                string_pool: Arc::new(string_pool),
                dir_counts,
            };
            state.current_snapshot.store(Arc::new(new_snapshot));

            state.is_scanning.store(false, Ordering::SeqCst);
        });
    }
}

#[cfg(feature = "online")]
impl GuiApp {
    fn draw_update_check_ui(&mut self, ui: &mut egui::Ui) {
        let check_result = self.update_checker.read_or_request(|| async {
            let client = reqwest::Client::builder()
                .user_agent("edirstat-update-checker")
                .build()
                .map_err(|e| e.to_string())?;

            let response = client
                .get("https://github.com/xangelix/edirstat/releases/latest")
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let final_url = response.url();
            let tag = final_url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .ok_or_else(|| "Invalid release URL structure".to_string())?;

            let latest_version_str = tag.trim_start_matches('v');
            let latest_version = semver::Version::parse(latest_version_str)
                .map_err(|e| format!("Failed to parse latest version '{tag}': {e}"))?;

            let current_version_str = env!("CARGO_PKG_VERSION");
            let current_version = semver::Version::parse(current_version_str).map_err(|e| {
                format!("Failed to parse current version '{current_version_str}': {e}")
            })?;

            if latest_version > current_version && latest_version.pre.is_empty() {
                Ok(Some(tag.to_string()))
            } else {
                Ok(None)
            }
        });

        match check_result {
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.small(t!("update-checking"));
                });
            }
            Some(Ok(Some(new_version))) => {
                ui.horizontal(|ui| {
                    ui.colored_label(theme::BUTTON_ORANGE, "✨");
                    ui.hyperlink_to(
                        egui::RichText::new(t!("update-available", { "version" => new_version.as_str() }))
                            .color(theme::BUTTON_ORANGE)
                            .strong(),
                        "https://github.com/xangelix/edirstat/releases/latest",
                    );
                });
            }
            Some(Ok(None)) => {
                ui.weak(t!("update-up-to-date"));
            }
            Some(Err(err)) => {
                ui.weak(t!("update-failed", { "error" => err.as_str() }));
            }
        }
    }
}

fn is_permission_denied_io(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::PermissionDenied
}

fn is_permission_denied_trash(err: &trash::Error) -> bool {
    match err {
        trash::Error::CouldNotAccess { .. } => true,
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "ios"),
            not(target_os = "android")
        ))]
        trash::Error::FileSystem { source, .. } => {
            source.kind() == std::io::ErrorKind::PermissionDenied
        }
        trash::Error::Os { description, .. } | trash::Error::Unknown { description } => {
            let desc_lower = description.to_lowercase();
            desc_lower.contains("permission")
                || desc_lower.contains("access is denied")
                || desc_lower.contains("denied")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::coordinator::{Coordinator, SharedState};
    use crate::engine::traversal::TraversalEngine;

    #[test]
    fn test_execute_softlinking() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_gui_softlinking");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean old
        std::fs::create_dir_all(&temp_dir)?;

        let file1_path = temp_dir.join("file1.txt");
        let file2_path = temp_dir.join("file2.txt");

        let content = b"hello identical softlink content! hello identical softlink content!";
        std::fs::write(&file1_path, content)?;
        std::fs::write(&file2_path, content)?;

        let shared_state = Arc::new(SharedState::new());
        let engine = Arc::new(TraversalEngine::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        let handle = engine.start_traversal(temp_dir.clone(), tx)?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());

        let mut app = GuiApp::new(shared_state, engine, None);

        // Scan duplicates using run_deduplication
        let progress = atomic_progress::Progress::new_spinner("Deduplicator");
        let config = crate::stats::deduplicator::DeduplicatorConfig {
            min_size: 1,
            ignore_system: false,
            ignore_hidden: false,
        };
        crate::stats::deduplicator::run_deduplication(
            snapshot.clone(),
            progress,
            app.deduplicator_results.clone(),
            app.deduplicator_cancel.clone(),
            config,
        );

        // Verify we found a duplicate group
        assert_eq!(app.deduplicator_results.read().groups.len(), 1);
        assert_eq!(app.deduplicator_results.read().groups[0].nodes.len(), 2);

        // We want to replace the second file (node index 2 or 1, let's find the one that is not original/first)
        let target_node_idx = {
            let results_guard = app.deduplicator_results.read();
            // The flat_rows sorted items: first is original, second is duplicate to be replaced.
            assert_eq!(results_guard.flat_rows.len(), 2);
            assert!(!results_guard.flat_rows[1].is_original);
            results_guard.flat_rows[1].node_idx
        };

        // Execute softlinking
        app.execute_softlinking(&[target_node_idx], &snapshot);

        // Verify the softlink exists and points to the first file
        let target_path = snapshot.get_full_path(target_node_idx);
        let link_path = std::path::Path::new(&target_path);
        assert!(link_path.exists());
        assert!(link_path.is_symlink());

        // Verify that the softlinked node has been removed from the results
        assert!(app.deduplicator_results.read().groups.is_empty());
        assert!(app.deduplicator_results.read().flat_rows.is_empty());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_gui_app_new_with_initial_path() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_gui_app_initial_path");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        let test_file = temp_dir.join("test.txt");
        std::fs::write(&test_file, b"hello world")?;

        let shared_state = Arc::new(SharedState::new());
        let engine = Arc::new(TraversalEngine::new());

        // Test scanning a directory
        let app = GuiApp::new(shared_state.clone(), engine, Some(temp_dir.clone()));

        // Wait for the background scan to start
        let mut attempts = 0;
        while !shared_state.is_scanning.load(Ordering::SeqCst) && attempts < 200 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            attempts += 1;
        }

        // Wait for the background scan to complete
        attempts = 0;
        while shared_state.is_scanning.load(Ordering::SeqCst) && attempts < 200 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            attempts += 1;
        }

        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());
        assert_eq!(app.current_scan_path, Some(temp_dir.clone()));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_gui_app_new_with_snapshot_file() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_gui_app_snapshot_file");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        let mut pool = crate::arena::StringPool::new();
        let name_root = pool.get_or_insert(b"/");
        let nodes = vec![crate::arena::FileNode::new(
            name_root, None, true, false, 0, 0,
        )];
        let test_file = temp_dir.join("test_snapshot.edst");
        crate::persistence::snapshot::save_snapshot(&nodes, &pool, &test_file, false)?; // Save as raw uncompressed

        let shared_state = Arc::new(SharedState::new());
        let engine = Arc::new(TraversalEngine::new());

        // Test loading snapshot file
        let app = GuiApp::new(shared_state.clone(), engine, Some(test_file.clone()));

        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());
        assert_eq!(app.current_scan_path, Some(test_file));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_gui_root_name_clean_unc() -> Result<(), egui_table_kit::error::TableError> {
        use egui_table_kit::operations::TableProvider as _;

        let mut pool = crate::arena::StringPool::new();
        // A Windows UNC path as the root node name
        let name_root = pool.get_or_insert(b"\\\\?\\C:\\TestFolder");
        let nodes = vec![crate::arena::FileNode::new(
            name_root, None, true, false, 0, 0,
        )];
        let dir_counts = crate::arena::precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(crate::arena::NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        // 1. Verify TableProviderWrapper for_selected_rows and for_all_rows
        let provider = crate::gui::explorer::TableProviderWrapper::new(
            &snapshot,
            crate::model::time_utils::TimeFormat::default(),
        );
        let mut state = egui_table_kit::state::TableState::new("test_table", 0);
        state.selected_rows.insert(0);

        let mut row_names = Vec::new();
        provider.for_selected_rows(&state, &mut |row| {
            if let Some(cell) = row.cell(0) {
                row_names.push(cell.0.to_string());
            }
            Ok(())
        })?;
        assert_eq!(row_names, vec!["C:\\TestFolder"]);

        let mut all_row_names = Vec::new();
        provider.for_all_rows(&mut |row| {
            if let Some(cell) = row.cell(0) {
                all_row_names.push(cell.0.to_string());
            }
            Ok(())
        })?;
        assert_eq!(all_row_names, vec!["C:\\TestFolder"]);

        // 2. Verify row_matches
        let mut filter = egui_table_kit::filter::Filter::default();
        filter.search.set_text("Folder");
        filter.search.open();
        assert!(provider.row_matches(&state, 0, &[(0, filter.clone())], None));

        // Searching for the UNC prefix should not match if it's stripped
        let mut filter_unc = egui_table_kit::filter::Filter::default();
        filter_unc.search.set_text(r"\\?\");
        filter_unc.search.open();
        assert!(!provider.row_matches(&state, 0, &[(0, filter_unc)], None));

        Ok(())
    }
}
