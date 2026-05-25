use eframe::egui;

use super::{GuiApp, theme};
use crate::arena::FileArenaSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    Delete,
    About,
    DeleteDuplicates,
}

impl GuiApp {
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
                                                // Reset selection upon deletion
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
                                            group.nodes.retain(|idx| !successfully_deleted.contains(idx));
                                        }
                                        results.retain(|group| group.nodes.len() >= 2);
                                    }

                                    // Clear selection and close modal
                                    self.selected_duplicates.retain(|idx| !successfully_deleted.contains(idx));
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
}
