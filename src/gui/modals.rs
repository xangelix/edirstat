use std::sync::{Arc, atomic::Ordering};

use eframe::egui;

use super::{GuiApp, theme};
use crate::arena::FileArenaSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    Delete,
    Trash,
    About,
    DeleteDuplicates,
    TrashDuplicates,
    HardlinkDuplicates,
    SoftlinkDuplicates,
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

fn walk_dir(
    dir_path: &std::path::Path,
    parent_idx: u32,
    cloned_nodes: &mut Vec<crate::arena::FileNode>,
    string_pool: &mut crate::arena::StringPool,
    last_child_map: &mut Vec<u32>,
    traversal_stats: &crate::engine::traversal::TraversalStats,
    dir_idx: u32,
) {
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        return;
    };
    traversal_stats.dirs_scanned.fetch_add(1, Ordering::SeqCst);

    for entry_res in entries {
        let Ok(entry) = entry_res else {
            continue;
        };
        let Some(meta) = crate::arena::EntryMetadata::from_dir_entry(&entry) else {
            continue;
        };

        let name_id = string_pool.get_or_insert(meta.name.as_bytes());
        let child_idx = cloned_nodes.len() as u32;

        let node = crate::arena::FileNode::from_metadata(name_id, Some(parent_idx), &meta);
        cloned_nodes.push(node);
        last_child_map.push(crate::arena::NO_INDEX);

        // Connect to parent sibling chain
        let p_idx = parent_idx as usize;
        let last_child = last_child_map[p_idx];
        if last_child == crate::arena::NO_INDEX {
            cloned_nodes[p_idx].first_child = child_idx;
        } else {
            cloned_nodes[last_child as usize].next_sibling = child_idx;
        }
        last_child_map[p_idx] = child_idx;

        if meta.is_dir {
            walk_dir(
                &entry.path(),
                child_idx,
                cloned_nodes,
                string_pool,
                last_child_map,
                traversal_stats,
                dir_idx,
            );
        } else {
            traversal_stats.files_scanned.fetch_add(1, Ordering::SeqCst);
            traversal_stats
                .bytes_scanned
                .fetch_add(meta.len as usize, Ordering::SeqCst);

            // Propagate size and count upwards through parent indices up to dir_idx
            let mut current_idx = Some(parent_idx);
            while let Some(idx) = current_idx {
                cloned_nodes[idx as usize].size += meta.len;
                cloned_nodes[idx as usize].file_count += 1;
                if idx == dir_idx {
                    break;
                }
                current_idx = cloned_nodes[idx as usize].parent_opt();
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
        let mut cloned_nodes = (*current_snap.nodes).clone();

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

        let new_snapshot = crate::arena::FileArenaSnapshot {
            nodes: std::sync::Arc::new(cloned_nodes),
            string_pool: current_snap.string_pool.clone(),
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
        for &idx in target_indices {
            let path_str = snapshot.get_full_path(idx);
            let path = std::path::Path::new(&path_str);
            if path.exists() {
                let result = if to_trash {
                    trash::delete(path).map_err(|e| e.to_string())
                } else if path.is_dir() {
                    std::fs::remove_dir_all(path).map_err(|e| e.to_string())
                } else {
                    std::fs::remove_file(path).map_err(|e| e.to_string())
                };

                if let Err(e) = result {
                    println!("Failed to delete/trash path {}: {}", path.display(), e);
                } else {
                    successfully_deleted.push(idx);
                }
            } else {
                successfully_deleted.push(idx);
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
            self.selected_nodes
                .retain(|idx| !successfully_deleted.contains(idx));
            self.remove_nodes_from_snapshot(&successfully_deleted);

            if self.selected_nodes.len() == 1 {
                self.selected_node_idx = self.selected_nodes.iter().next().copied();
            } else {
                self.selected_node_idx = None;
            }
        }
    }

    pub(crate) fn execute_hardlinking(
        &mut self,
        target_indices: &[u32],
        snapshot: &FileArenaSnapshot,
    ) {
        let mut successfully_linked = Vec::new();
        let results_guard = self.deduplicator_results.read();

        for &idx in target_indices {
            let Some(group) = results_guard.groups.iter().find(|g| g.nodes.contains(&idx)) else {
                continue;
            };

            // Find a source node in the group that is NOT being replaced to link against
            let Some(&src_idx) = group.nodes.iter().find(|&&n| !target_indices.contains(&n)) else {
                continue;
            };

            let src_path_str = snapshot.get_full_path(src_idx);
            let dst_path_str = snapshot.get_full_path(idx);
            let src_path = std::path::Path::new(&src_path_str);
            let dst_path = std::path::Path::new(&dst_path_str);

            if src_path.exists() && dst_path.exists() {
                let temp_dst = dst_path.with_extension("tmp_hl_bak");
                if std::fs::rename(dst_path, &temp_dst).is_ok() {
                    if std::fs::hard_link(src_path, dst_path).is_ok() {
                        let _ = std::fs::remove_file(&temp_dst);
                        successfully_linked.push(idx);
                    } else {
                        // Restore backup on failure
                        let _ = std::fs::rename(&temp_dst, dst_path);
                    }
                }
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
        let results_guard = self.deduplicator_results.read();

        for &idx in target_indices {
            let Some(group) = results_guard.groups.iter().find(|g| g.nodes.contains(&idx)) else {
                continue;
            };

            // Find a source node in the group that is NOT being replaced to link against
            let Some(&src_idx) = group.nodes.iter().find(|&&n| !target_indices.contains(&n)) else {
                continue;
            };

            let src_path_str = snapshot.get_full_path(src_idx);
            let dst_path_str = snapshot.get_full_path(idx);
            let src_path = std::path::Path::new(&src_path_str);
            let dst_path = std::path::Path::new(&dst_path_str);

            if src_path.exists() && dst_path.exists() {
                let temp_dst = dst_path.with_extension("tmp_sl_bak");
                if std::fs::rename(dst_path, &temp_dst).is_ok() {
                    let symlink_result = {
                        #[cfg(unix)]
                        {
                            std::os::unix::fs::symlink(src_path, dst_path)
                        }
                        #[cfg(windows)]
                        {
                            std::os::windows::fs::symlink_file(src_path, dst_path)
                        }
                        #[cfg(not(any(unix, windows)))]
                        {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Unsupported,
                                "Symlinks not supported on this platform",
                            ))
                        }
                    };

                    if symlink_result.is_ok() {
                        let _ = std::fs::remove_file(&temp_dst);
                        successfully_linked.push(idx);
                    } else {
                        // Restore backup on failure
                        let _ = std::fs::rename(&temp_dst, dst_path);
                    }
                }
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
            self.selected_nodes
                .retain(|idx| !successfully_linked.contains(idx));
            self.remove_nodes_from_snapshot(&successfully_linked);

            if self.selected_nodes.len() == 1 {
                self.selected_node_idx = self.selected_nodes.iter().next().copied();
            } else {
                self.selected_node_idx = None;
            }
        }
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
            title: &'static str,
            border_color: egui::Color32,
            warning_color: egui::Color32,
            header: String,
            info_msg: String,
            warning_msg: &'static str,
            checkbox_label: &'static str,
            confirm_button_text: &'static str,
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "⚠ PERMANENT DELETION WARNING",
                        border_color: theme::DELETION_BORDER,
                        warning_color: theme::DELETION_WARNING,
                        header: "⚠ Permanent Deletion Warning!".to_string(),
                        info_msg: format!("Total Size: {size_str}"),
                        warning_msg: "This is a recursive deletion. All files, folders, and subdirectories under the selected path(s) will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).",
                        checkbox_label: "I understand that files will be permanently deleted and cannot be recovered.",
                        confirm_button_text: "🗑 Yes, Delete Permanently",
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "♻ MOVE TO TRASH",
                        border_color: theme::TRASH_BORDER,
                        warning_color: theme::TRASH_WARNING,
                        header: "♻ Move to Trash".to_string(),
                        info_msg: format!("Total Size: {size_str}"),
                        warning_msg: "This will move the selected path(s) and all their contents to your system recycle bin/trash, where they can be recovered or permanently deleted later.",
                        checkbox_label: "I confirm that I want to move this to the trash.",
                        confirm_button_text: "♻ Yes, Move to Trash",
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "⚠ PERMANENT DEDUPLICATION WARNING",
                        border_color: theme::DELETION_BORDER,
                        warning_color: theme::DELETION_WARNING,
                        header: "⚠ Permanent Duplicate Deletion Warning!".to_string(),
                        info_msg: format!("Total space to be reclaimed: {size_str}"),
                        warning_msg: "All selected files will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).",
                        checkbox_label: "I understand that files will be permanently deleted and cannot be recovered.",
                        confirm_button_text: "🗑 Yes, Delete Selected Permanently",
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "♻ MOVE DUPLICATES TO TRASH",
                        border_color: theme::TRASH_BORDER,
                        warning_color: theme::TRASH_WARNING,
                        header: "♻ Move Duplicates to Trash".to_string(),
                        info_msg: format!("Total space to be reclaimed: {size_str}"),
                        warning_msg: "All selected files will be moved to the recycle bin/trash.",
                        checkbox_label: "I confirm that I want to move these files to the trash.",
                        confirm_button_text: "♻ Yes, Move Selected to Trash",
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "🔗 REPLACE DUPLICATES WITH HARDLINKS",
                        border_color: theme::BUTTON_ORANGE,
                        warning_color: theme::BUTTON_ORANGE_HOVER,
                        header: "🔗 Replace Duplicates with Hardlinks".to_string(),
                        info_msg: format!(
                            "Total files to process: {}. Cumulative virtual size: {}",
                            idxs.len(),
                            size_str
                        ),
                        warning_msg: "This will delete the selected duplicate files and replace them with filesystem-level hardlinks pointing to the remaining original file in each group. This retains files visually while freeing up actual physical storage.",
                        checkbox_label: "I confirm that I want to replace selected files with hardlinks.",
                        confirm_button_text: "🔗 Yes, Replace with Hardlinks",
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
                        .map(|&idx| snapshot.get_full_path(idx))
                        .collect();
                    Some(ModalConfig {
                        title: "🔗 REPLACE DUPLICATES WITH SOFTLINKS",
                        border_color: theme::BUTTON_ORANGE,
                        warning_color: theme::BUTTON_ORANGE_HOVER,
                        header: "🔗 Replace Duplicates with Softlinks".to_string(),
                        info_msg: format!(
                            "Total files to process: {}. Cumulative virtual size: {}",
                            idxs.len(),
                            size_str
                        ),
                        warning_msg: "This will delete the selected duplicate files and replace them with filesystem-level softlinks (symbolic links) pointing to the remaining original file in each group. This retains files visually while freeing up actual physical storage.",
                        checkbox_label: "I confirm that I want to replace selected files with softlinks.",
                        confirm_button_text: "🔗 Yes, Replace with Softlinks",
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
                .default_width(500.0)
                .open(&mut open)
                .frame(
                    egui::Frame::window(&ctx.global_style())
                        .stroke(egui::Stroke::new(2.0, cfg.border_color)),
                )
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        let path_exists = if cfg.paths.len() == 1 {
                            std::path::Path::new(&cfg.paths[0]).exists()
                        } else {
                            true
                        };

                        if path_exists {
                            ui.heading(
                                egui::RichText::new(&cfg.header)
                                    .color(cfg.warning_color)
                                    .strong(),
                            );
                            ui.separator();

                            ui.label(if cfg.paths.len() > 1 {
                                format!(
                                    "You are about to delete/trash {} duplicate files/items:",
                                    cfg.paths.len()
                                )
                            } else {
                                "You are about to delete/trash the following path:".to_string()
                            });

                            ui.colored_label(ui.visuals().strong_text_color(), &cfg.paths[0]);
                            if cfg.paths.len() > 1 {
                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for path in &cfg.paths[1..] {
                                            ui.small(path);
                                        }
                                    });
                            }
                            ui.label(&cfg.info_msg);
                            ui.separator();

                            ui.label(cfg.warning_msg);
                            ui.add_space(8.0);

                            ui.checkbox(&mut self.delete_confirm_checked, cfg.checkbox_label);
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.active_modal = None;
                                }

                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new(cfg.confirm_button_text)
                                        .color(theme::COLOR_WHITE)
                                        .strong(),
                                )
                                .fill(cfg.border_color);

                                let confirm_res =
                                    ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                if confirm_res.clicked() {
                                    match cfg.action {
                                        DeletionAction::DeleteMultiple => {
                                            self.execute_deletion(
                                                &self.delete_node_indices.clone(),
                                                false,
                                                snapshot,
                                            );
                                            self.delete_node_indices.clear();
                                        }
                                        DeletionAction::TrashMultiple => {
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
                                egui::RichText::new("❌ Path Does Not Exist!")
                                    .color(theme::DELETION_WARNING)
                                    .strong(),
                            );
                            ui.separator();
                            ui.label(
                                "Error: The path you are trying to delete does not exist on disk.",
                            );
                            ui.colored_label(ui.visuals().strong_text_color(), &cfg.paths[0]);
                            ui.add_space(8.0);
                            if ui.button("Close").clicked() {
                                self.active_modal = None;
                            }
                        }
                    });
                });
            if !open {
                self.active_modal = None;
            }
        }

        // Render Help -> About Modal Popup
        if self.active_modal == Some(ActiveModal::About) {
            let mut open = true;
            egui::Window::new("ℹ About eDirStat")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("eDirStat 👷")
                                .strong()
                                .color(ui.visuals().strong_text_color())
                        );
                        ui.label(concat!("v", env!("CARGO_PKG_VERSION")));
                        ui.separator();
                        ui.label("By: Cody Wyatt Neiman (xangelix) <".to_owned() + "neiman" + "@" + "cody.to>");
                        ui.add_space(8.0);
                        ui.label("A high-performance disk space analyzer and deduplication toolkit built in Rust.");
                        ui.add_space(4.0);
                        ui.label("Features parallel, work-stealing directory traversal, zero-copy memory-mapped file structures, and responsive, interactive treemaps.");
                        ui.add_space(4.0);
                        ui.label("The integrated deduplicator runs a multi-stage cryptographic hashing pipeline to safely isolate duplicate groups, calculate reclaimable space, and respect system-level hardlinks.");
                        ui.add_space(8.0);
                        if ui.button("Close").clicked() {
                            self.active_modal = None;
                        }
                    });
                });
            if !open {
                self.active_modal = None;
            }
        }
    }

    pub(crate) fn refresh_directory_subtree(&self, dir_idx: u32) {
        self.refresh_directory_subtrees(&[dir_idx]);
    }

    pub(crate) fn refresh_directory_subtrees(&self, dir_indices: &[u32]) {
        if dir_indices.is_empty() {
            return;
        }
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
            let mut cloned_nodes = (*current_snap.nodes).clone();
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
                    cloned_nodes[idx].parent = crate::arena::NO_INDEX;
                }

                cloned_nodes[dir_idx as usize].first_child = crate::arena::NO_INDEX;
                cloned_nodes[dir_idx as usize].size = 0;
                cloned_nodes[dir_idx as usize].file_count = 0;

                // 2. Scan the directory recursively on disk and append new nodes
                let mut last_child_map = vec![crate::arena::NO_INDEX; cloned_nodes.len()];

                walk_dir(
                    &path,
                    dir_idx,
                    &mut cloned_nodes,
                    &mut string_pool,
                    &mut last_child_map,
                    &traversal_stats,
                    dir_idx,
                );

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
            let new_snapshot = FileArenaSnapshot {
                nodes: Arc::new(cloned_nodes),
                string_pool: Arc::new(string_pool),
            };
            state.current_snapshot.store(Arc::new(new_snapshot));

            state.is_scanning.store(false, Ordering::SeqCst);
        });
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

        let mut app = GuiApp::new(shared_state, engine);

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
}
