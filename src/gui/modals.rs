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
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        let is_symlink = metadata.is_symlink();
        let modified_timestamp = metadata.modified().map_or(0, |t| {
            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64)
        });
        let created_timestamp = metadata.created().map_or(0, |t| {
            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64)
        });
        let accessed_timestamp = metadata.accessed().map_or(0, |t| {
            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64)
        });

        let name_id = string_pool.get_or_insert(name.as_bytes());
        let child_idx = cloned_nodes.len() as u32;

        if metadata.is_dir() {
            let dir_node = crate::arena::FileNode::new(
                name_id,
                Some(parent_idx),
                true,
                false,
                modified_timestamp,
                created_timestamp,
                accessed_timestamp,
            );
            cloned_nodes.push(dir_node);
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

            walk_dir(
                &path,
                child_idx,
                cloned_nodes,
                string_pool,
                last_child_map,
                traversal_stats,
                dir_idx,
            );
        } else {
            let size = metadata.len();
            let mut file_node = crate::arena::FileNode::new(
                name_id,
                Some(parent_idx),
                false,
                is_symlink,
                modified_timestamp,
                created_timestamp,
                accessed_timestamp,
            );
            file_node.size = size;
            cloned_nodes.push(file_node);
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

            traversal_stats.files_scanned.fetch_add(1, Ordering::SeqCst);
            traversal_stats
                .bytes_scanned
                .fetch_add(size as usize, Ordering::SeqCst);

            // Propagate size and count upwards through parent indices up to dir_idx
            let mut current_idx = Some(parent_idx);
            while let Some(idx) = current_idx {
                cloned_nodes[idx as usize].size += size;
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

    pub fn render_modals(&mut self, ctx: &egui::Context, snapshot: &FileArenaSnapshot) {
        // Render Permanent Deletion Modal Popup
        if self.active_modal == Some(ActiveModal::Delete) {
            let idx_opt = self.delete_node_idx;
            if let Some(idx) = idx_opt {
                let path_str = snapshot.get_full_path(idx);
                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(snapshot.nodes[idx as usize].size)
                    .to_string();

                let mut open = true;
                egui::Window::new("⚠ PERMANENT DELETION WARNING")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .frame(
                        egui::Frame::window(&ctx.global_style())
                            .stroke(egui::Stroke::new(2.0, theme::DELETION_BORDER))
                    ) // Thick red border outline
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            let path = std::path::Path::new(&path_str);
                            if path.exists() {
                                ui.heading(
                                    egui::RichText::new("⚠ Permanent Deletion Warning!")
                                        .color(theme::DELETION_WARNING)
                                        .strong()
                                );
                                ui.separator();

                                ui.label("You are about to permanently delete the following path:");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
                                ui.label(format!("Total Size: {size_str}"));
                                ui.separator();

                                ui.label("This is a recursive deletion. All files, folders, and subdirectories under this path will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).");
                                ui.add_space(8.0);

                                ui.checkbox(&mut self.delete_confirm_checked, "I understand that files will be permanently deleted and cannot be recovered.");
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Cancel").clicked() {
                                        self.active_modal = None;
                                    }

                                    // Red confirm button
                                    let confirm_btn = egui::Button::new(
                                        egui::RichText::new("🗑 Yes, Delete Permanently")
                                            .color(egui::Color32::WHITE)
                                            .strong()
                                    ).fill(theme::DELETION_BORDER);

                                    let confirm_res = ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                    if confirm_res.clicked() {
                                        let path = std::path::Path::new(&path_str);
                                        if path.exists() {
                                            let delete_result = if path.is_dir() {
                                                std::fs::remove_dir_all(path)
                                            } else {
                                                std::fs::remove_file(path)
                                            };

                                            if let Err(e) = delete_result {
                                                println!("Failed to delete path: {e}");
                                            } else {
                                                // Dynamic backpropagation updates the tree before dropping active choice
                                                self.remove_nodes_from_snapshot(&[idx]);
                                                self.selected_node_idx = None;
                                            }
                                        }
                                        self.active_modal = None;
                                    }
                                });
                            } else {
                                ui.heading(
                                    egui::RichText::new("❌ Path Does Not Exist!")
                                        .color(theme::DELETION_WARNING)
                                        .strong()
                                );
                                ui.separator();
                                ui.label("Error: The path you are trying to delete does not exist on disk.");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
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
        }

        // Render Move to Trash Modal Popup
        if self.active_modal == Some(ActiveModal::Trash) {
            let idx_opt = self.delete_node_idx;
            if let Some(idx) = idx_opt {
                let path_str = snapshot.get_full_path(idx);
                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(snapshot.nodes[idx as usize].size)
                    .to_string();

                let mut open = true;
                egui::Window::new("♻ MOVE TO TRASH")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .frame(
                        egui::Frame::window(&ctx.global_style())
                            .stroke(egui::Stroke::new(2.0, theme::TRASH_BORDER))
                    ) // Thick blue border outline
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            let path = std::path::Path::new(&path_str);
                            if path.exists() {
                                ui.heading(
                                    egui::RichText::new("♻ Move to Trash")
                                        .color(theme::TRASH_WARNING)
                                        .strong()
                                );
                                ui.separator();

                                ui.label("You are about to move the following path to the trash:");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
                                ui.label(format!("Total Size: {size_str}"));
                                ui.separator();

                                ui.label("This will move the selected path and all its contents to your system recycle bin/trash, where it can be recovered or permanently deleted later.");
                                ui.add_space(8.0);

                                ui.checkbox(&mut self.delete_confirm_checked, "I confirm that I want to move this to the trash.");
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Cancel").clicked() {
                                        self.active_modal = None;
                                    }

                                    // Blue confirm button
                                    let confirm_btn = egui::Button::new(
                                        egui::RichText::new("♻ Yes, Move to Trash")
                                            .color(egui::Color32::WHITE)
                                            .strong()
                                    ).fill(theme::TRASH_BORDER);

                                    let confirm_res = ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                    if confirm_res.clicked() {
                                        let path = std::path::Path::new(&path_str);
                                        if path.exists() {
                                            let trash_result = trash::delete(path);

                                            if let Err(e) = trash_result {
                                                println!("Failed to move path to trash: {e}");
                                            } else {
                                                // Dynamic backpropagation updates the tree before dropping active choice
                                                self.remove_nodes_from_snapshot(&[idx]);
                                                self.selected_node_idx = None;
                                            }
                                        }
                                        self.active_modal = None;
                                    }
                                });
                            } else {
                                ui.heading(
                                    egui::RichText::new("❌ Path Does Not Exist!")
                                        .color(theme::DELETION_WARNING)
                                        .strong()
                                );
                                ui.separator();
                                ui.label("Error: The path you are trying to trash does not exist on disk.");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
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
        }

        // Render Permanent Deduplication Modal Popup
        if self.active_modal == Some(ActiveModal::DeleteDuplicates) {
            let idxs = self.delete_duplicates_indices.clone();
            if idxs.is_empty() {
                self.active_modal = None;
            } else {
                let count = idxs.len();
                let total_size: u64 = idxs
                    .iter()
                    .map(|&idx| snapshot.nodes[idx as usize].size)
                    .sum();
                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(total_size)
                    .to_string();

                let mut open = true;
                egui::Window::new("⚠ PERMANENT DEDUPLICATION WARNING")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(true)
                    .default_width(500.0)
                    .open(&mut open)
                    .frame(
                        egui::Frame::window(&ctx.global_style())
                            .stroke(egui::Stroke::new(2.0, theme::DELETION_BORDER))
                    ) // Thick red border outline
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("⚠ Permanent Duplicate Deletion Warning!")
                                    .color(theme::DELETION_WARNING)
                                    .strong()
                            );
                            ui.separator();

                            ui.label(format!("You are about to permanently delete {count} duplicate files:"));
                            ui.colored_label(ui.visuals().strong_text_color(), format!("Total space to be reclaimed: {size_str}"));
                            ui.separator();

                            ui.label("Files to be deleted:");
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                for &idx in &idxs {
                                    let path_str = snapshot.get_full_path(idx);
                                    ui.small(&path_str);
                                }
                            });
                            ui.separator();

                            ui.label("All selected files will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).");
                            ui.add_space(8.0);

                            ui.checkbox(&mut self.delete_confirm_checked, "I understand that files will be permanently deleted and cannot be recovered.");
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.active_modal = None;
                                }

                                // Red confirm button
                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new("🗑 Yes, Delete Selected Permanently")
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                ).fill(theme::DELETION_BORDER);

                                    let confirm_res = ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                    if confirm_res.clicked() {
                                        let mut successfully_deleted = Vec::new();
                                        for &idx in &self.delete_duplicates_indices {
                                            let path_str = snapshot.get_full_path(idx);
                                            let path = std::path::Path::new(&path_str);
                                            if path.exists() {
                                                let delete_result = if path.is_dir() {
                                                    std::fs::remove_dir_all(path)
                                                } else {
                                                    std::fs::remove_file(path)
                                                };

                                                if let Err(e) = delete_result {
                                                    println!("Failed to delete path: {e}");
                                                } else {
                                                    successfully_deleted.push(idx);
                                                }
                                            } else {
                                                successfully_deleted.push(idx);
                                            }
                                        }

                                        // Prune the deleted files from the deduplicator in-memory results list
                                        {
                                            let mut results = self.deduplicator_results.write();
                                            for group in results.iter_mut() {
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
                                            results.retain(|group| group.nodes.len() >= 2);
                                        }

                                        // Clear selection and close modal
                                        self.selected_duplicates.retain(|idx| !successfully_deleted.contains(idx));

                                        // Execute structural adjustments over the cloned tree all at once
                                        self.remove_nodes_from_snapshot(&successfully_deleted);

                                        self.delete_duplicates_indices.clear();
                                        self.active_modal = None;
                                    }
                            });
                        });
                    });
                if !open {
                    self.active_modal = None;
                }
            }
        }

        // Render Move to Trash Deduplication Modal Popup
        if self.active_modal == Some(ActiveModal::TrashDuplicates) {
            let idxs = self.delete_duplicates_indices.clone();
            if idxs.is_empty() {
                self.active_modal = None;
            } else {
                let count = idxs.len();
                let total_size: u64 = idxs
                    .iter()
                    .map(|&idx| snapshot.nodes[idx as usize].size)
                    .sum();
                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(total_size)
                    .to_string();

                let mut open = true;
                egui::Window::new("♻ MOVE DUPLICATES TO TRASH")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(true)
                    .default_width(500.0)
                    .open(&mut open)
                    .frame(
                        egui::Frame::window(&ctx.global_style())
                            .stroke(egui::Stroke::new(2.0, theme::TRASH_BORDER)),
                    ) // Thick blue border outline
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("♻ Move Duplicates to Trash")
                                    .color(theme::TRASH_WARNING)
                                    .strong(),
                            );
                            ui.separator();

                            ui.label(format!(
                                "You are about to move {count} duplicate files to the trash:"
                            ));
                            ui.colored_label(
                                ui.visuals().strong_text_color(),
                                format!("Total space to be reclaimed: {size_str}"),
                            );
                            ui.separator();

                            ui.label("Files to be moved:");
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for &idx in &idxs {
                                        let path_str = snapshot.get_full_path(idx);
                                        ui.small(&path_str);
                                    }
                                });
                            ui.separator();

                            ui.label("All selected files will be moved to the recycle bin/trash.");
                            ui.add_space(8.0);

                            ui.checkbox(
                                &mut self.delete_confirm_checked,
                                "I confirm that I want to move these files to the trash.",
                            );
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.active_modal = None;
                                }

                                // Blue confirm button
                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new("♻ Yes, Move Selected to Trash")
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                )
                                .fill(theme::TRASH_BORDER);

                                let confirm_res =
                                    ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                if confirm_res.clicked() {
                                    let mut successfully_deleted = Vec::new();
                                    for &idx in &self.delete_duplicates_indices {
                                        let path_str = snapshot.get_full_path(idx);
                                        let path = std::path::Path::new(&path_str);
                                        if path.exists() {
                                            let trash_result = trash::delete(path);

                                            if let Err(e) = trash_result {
                                                println!("Failed to move path to trash: {e}");
                                            } else {
                                                successfully_deleted.push(idx);
                                            }
                                        } else {
                                            successfully_deleted.push(idx);
                                        }
                                    }

                                    // Prune the deleted files from the deduplicator in-memory results list
                                    {
                                        let mut results = self.deduplicator_results.write();
                                        for group in results.iter_mut() {
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
                                        results.retain(|group| group.nodes.len() >= 2);
                                    }

                                    // Clear selection and close modal
                                    self.selected_duplicates
                                        .retain(|idx| !successfully_deleted.contains(idx));

                                    // Execute structural adjustments over the cloned tree all at once
                                    self.remove_nodes_from_snapshot(&successfully_deleted);

                                    self.delete_duplicates_indices.clear();
                                    self.active_modal = None;
                                }
                            });
                        });
                    });
                if !open {
                    self.active_modal = None;
                }
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
                        ui.label("A modern, zero-copy, highly performant disk usage analyzer written in Rust.");
                        ui.label("Features dynamic work-stealing multithreaded directory walking, lazy explorer sibling sorting, zero-copy persistent memory mapping, HSL treemap gradients, and instant virtual rendering.");
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
        let current_snap = self.shared_state.current_snapshot.load();
        let path_str = current_snap.get_full_path(dir_idx);
        let path = std::path::PathBuf::from(path_str);
        if !path.exists() || !path.is_dir() {
            return;
        }

        let state = self.shared_state.clone();
        let traversal_stats = self.traversal_engine.stats().clone();

        state.is_scanning.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            let current_snap = state.current_snapshot.load();
            let mut cloned_nodes = (*current_snap.nodes).clone();
            let mut string_pool = (*current_snap.string_pool).clone();

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
