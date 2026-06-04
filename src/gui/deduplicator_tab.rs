#![allow(clippy::cast_precision_loss)]

use std::sync::Arc;
use std::sync::atomic::Ordering;

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::arena::FileArenaSnapshot;
use crate::stats::deduplicator::run_deduplication;

/// Flat representation of a duplicate candidate for lazy table rendering
#[derive(Clone, Debug)]
pub struct DuplicateRow {
    pub node_idx: u32,
    pub group_idx: usize,
    pub filename: String,
    pub parent_path: String,
    pub size: u64,
    pub size_str: String,
    pub reclaimable_str: String,
    pub created_time_str: String,
    pub modified_time_str: String,
    pub is_original: bool,
    pub is_hardlink: bool,
}

/// Format epoch timestamp into YYYY-MM-DD HH:MM:SS format
fn format_timestamp(epoch_secs: i64) -> String {
    if epoch_secs <= 0 {
        return "Unknown".to_string();
    }
    let days = epoch_secs / 86400;
    let secs_in_day = epoch_secs % 86400;

    let mut year = 1970;
    let mut days_left = days;

    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if days_left < days_in_year {
            break;
        }
        days_left -= days_in_year;
        year += 1;
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    let mut day = days_left + 1;
    for &days_in_m in &month_days {
        if day <= days_in_m {
            break;
        }
        day -= days_in_m;
        month += 1;
    }

    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

impl super::GuiApp {
    pub(crate) fn render_deduplicator_tab(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        let progress_snap = self.deduplicator_progress.snapshot();
        let is_running = !progress_snap.finished && progress_snap.elapsed.is_some();

        if is_running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Determine if any duplicate group is fully selected (meaning the original and all copies are selected)
        let active_groups = self.deduplicator_results.read().clone();
        let mut fully_selected_groups_info = Vec::new();
        for group in &active_groups {
            let all_selected = group
                .nodes
                .iter()
                .all(|&idx| self.selected_duplicates.contains(&idx));
            if all_selected && let Some(&first_idx) = group.nodes.first() {
                let filename = snapshot
                    .string_pool
                    .get(snapshot.nodes[first_idx as usize].name_id)
                    .unwrap_or("unknown")
                    .to_string();
                fully_selected_groups_info.push((filename, group.nodes.clone()));
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Lock vertical footprint to exactly 28px to eliminate layout jitter on show/hide
                ui.set_height(28.0);

                ui.heading(
                    egui::RichText::new("👥 Duplicate File Finder")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );

                // Floating, right-aligned warning badge on the far edge of the central panel
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !fully_selected_groups_info.is_empty() {
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));

                    let time = ui.input(|i| i.time);
                    let pulse = 0.5f64.mul_add((time * 6.0).sin(), 0.5) as f32;
                    let alpha = 0.6f32.mul_add(pulse, 0.4);
                    let warning_red = egui::Color32::from_rgb(239, 68, 68);
                    let glow_color = warning_red.linear_multiply(alpha * 0.15);
                    let text_color = warning_red.linear_multiply(0.4f32.mul_add(pulse, 0.6));

                let frame = egui::Frame::new()
                    .fill(glow_color)
                    .stroke(egui::Stroke::new(1.0, warning_red.linear_multiply(alpha * 0.4)))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .corner_radius(4.0);

                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠ DATA LOSS WARNING").strong().color(text_color));
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Deleting all versions of {} file(s)",
                                        fully_selected_groups_info.len()
                                    ))
                                    .color(ui.visuals().text_color())
                                );
                            });
                        }).response;

                        response.on_hover_ui(|ui| {
                            ui.set_max_width(450.0);
                            ui.heading(
                                egui::RichText::new("No Original Copy Will Remain:")
                                    .color(egui::Color32::from_rgb(239, 68, 68))
                                    .strong()
                            );
                            ui.label("You have checked both the original and all duplicate copies for the files listed below. Deleting them will likely result in permanent data loss:");
                            ui.separator();

                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for (filename, nodes) in &fully_selected_groups_info {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(egui::Color32::from_rgb(239, 68, 68), "🔥");
                                            ui.strong(filename);
                                            ui.weak(format!("({} copies selected)", nodes.len()));
                                        });
                                        for &idx in nodes {
                                            let path = snapshot.get_full_path(idx);
                                            ui.small(format!("  - {path}"));
                                        }
                                        ui.add_space(4.0);
                                    });
                                }
                            });
                        });
                    }
                });
            });

            ui.label("Find and safely remove byte-for-byte identical files using an optimized 7-stage hashing pipeline.");
            ui.separator();

            self.draw_deduplicator_controls(ui, snapshot);
            ui.separator();

            self.draw_deduplicator_results(ui, snapshot);
        });
    }

    fn draw_deduplicator_controls(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        let progress_snap = self.deduplicator_progress.snapshot();
        let is_running = !progress_snap.finished && progress_snap.elapsed.is_some();

        if is_running {
            ui.horizontal_wrapped(|ui| {
                // Interactive Spinner: turns into a red cancel X on hover with zero size shift
                let spinner_size = 18.0;
                let (rect, mut response) = ui.allocate_exact_size(
                    egui::vec2(spinner_size, spinner_size),
                    egui::Sense::click(),
                );

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    response = response.on_hover_text("Click to Cancel Scan");

                    let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(239, 68, 68));
                    let inset = 4.0;
                    ui.painter().line_segment(
                        [
                            rect.left_top() + egui::vec2(inset, inset),
                            rect.right_bottom() - egui::vec2(inset, inset),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            rect.right_top() + egui::vec2(-inset, inset),
                            rect.left_bottom() - egui::vec2(-inset, inset),
                        ],
                        stroke,
                    );

                    if response.clicked() {
                        self.deduplicator_cancel.store(true, Ordering::SeqCst);
                    }
                } else {
                    ui.put(rect, egui::Spinner::new().size(spinner_size));
                }

                ui.label(progress_snap.name.as_str());

                if progress_snap.total > 0 {
                    let progress_val =
                        (progress_snap.position as f64 / progress_snap.total.max(1) as f64) as f32;
                    ui.add(egui::ProgressBar::new(progress_val).text(format!(
                        "{}/{}",
                        progress_snap.position, progress_snap.total
                    )));
                }

                if !progress_snap.item.is_empty() {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    ui.weak(format!("Current: {}", progress_snap.item));
                }
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.label("Min File Size:");
                let mut min_kb = self.deduplicator_config.min_size / 1024;
                if ui
                    .add(
                        egui::DragValue::new(&mut min_kb)
                            .range(0..=1024 * 1024)
                            .suffix(" KB"),
                    )
                    .changed()
                {
                    self.deduplicator_config.min_size = min_kb * 1024;
                }

                ui.separator();
                ui.checkbox(
                    &mut self.deduplicator_config.ignore_system,
                    "Ignore System Files",
                );
                ui.checkbox(
                    &mut self.deduplicator_config.ignore_hidden,
                    "Ignore Hidden Files",
                );

                ui.separator();
                let can_scan = !snapshot.nodes.is_empty();
                let scan_btn =
                    ui.add_enabled(can_scan, egui::Button::new("⚡ Start Deduplication Scan"));

                if scan_btn.clicked() {
                    self.selected_duplicates.clear();
                    self.deduplicator_cancel.store(false, Ordering::SeqCst);

                    let snapshot_clone = Arc::new(FileArenaSnapshot {
                        nodes: snapshot.nodes.clone(),
                        string_pool: snapshot.string_pool.clone(),
                    });

                    self.deduplicator_progress = atomic_progress::ProgressBuilder::new_spinner(
                        "Phase 1/7: Grouping all scanned files by size...",
                    )
                    .with_start_time_now()
                    .build();

                    *self.deduplicator_results.write() = Vec::new();

                    let progress_clone = self.deduplicator_progress.clone();
                    let results_clone = self.deduplicator_results.clone();
                    let cancel_clone = self.deduplicator_cancel.clone();
                    let config_clone = self.deduplicator_config;

                    std::thread::spawn(move || {
                        run_deduplication(
                            snapshot_clone,
                            progress_clone,
                            results_clone,
                            cancel_clone,
                            config_clone,
                        );
                    });
                }

                if !can_scan {
                    ui.small("Please scan a directory first.");
                }
            });

            if progress_snap.finished {
                if let Some(err) = &progress_snap.error {
                    ui.colored_label(egui::Color32::YELLOW, format!("Scan was cancelled: {err}"));
                } else if !progress_snap.name.is_empty() && progress_snap.name != "Deduplicator" {
                    ui.colored_label(
                        crate::colors::COLOR_SCAN_COMPLETE,
                        progress_snap.name.as_str(),
                    );
                }
            }
        }
    }

    fn check_rebuild_flat_rows(
        &mut self,
        snapshot: &FileArenaSnapshot,
        active_groups: &[crate::stats::deduplicator::DuplicateGroup],
    ) {
        let current_total_files: usize = active_groups.iter().map(|g| g.nodes.len()).sum();
        let current_sig = (active_groups.len(), current_total_files);

        let mut flat_rows = Vec::with_capacity(current_total_files);
        for (g_idx, group) in active_groups.iter().enumerate() {
            // Pair each node with its file ID
            let mut paired: Vec<(u32, (u64, u64))> = group.nodes.iter().copied()
                .zip(group.file_ids.iter().copied())
                .collect();
            if paired.len() < group.nodes.len() {
                paired = group.nodes.iter().copied()
                    .map(|idx| (idx, (0, 0)))
                    .collect();
            }

            // Sort chronologically (oldest modified first), using shorter filename length as a tie-breaker
            paired.sort_by(|a, b| {
                let node_a = &snapshot.nodes[a.0 as usize];
                let node_b = &snapshot.nodes[b.0 as usize];

                let mod_cmp = node_a.modified_timestamp.cmp(&node_b.modified_timestamp);
                if mod_cmp != std::cmp::Ordering::Equal {
                    return mod_cmp;
                }

                let cre_cmp = node_a.created_timestamp.cmp(&node_b.created_timestamp);
                if cre_cmp != std::cmp::Ordering::Equal {
                    return cre_cmp;
                }

                let name_a = snapshot.string_pool.get(node_a.name_id).unwrap_or("");
                let name_b = snapshot.string_pool.get(node_b.name_id).unwrap_or("");
                name_a.len().cmp(&name_b.len())
            });

            // Calculate total reclaimable space for the group based on unique inodes
            let unique_inodes_count = {
                let mut ids: Vec<(u64, u64)> = paired.iter().map(|p| p.1).filter(|&id| id != (0, 0)).collect();
                ids.sort_unstable();
                ids.dedup();
                ids.len()
            };
            let total_reclaimable = if unique_inodes_count > 0 {
                group.size * (unique_inodes_count as u64 - 1)
            } else {
                group.size * (paired.len().saturating_sub(1) as u64)
            };

            for (f_idx, &(node_idx, file_id)) in paired.iter().enumerate() {
                let full_path = snapshot.get_full_path(node_idx);
                let path = std::path::Path::new(&full_path);

                let filename = path
                    .file_name()
                    .map_or_else(String::new, |s| s.to_string_lossy().into_owned());

                let parent_path = path
                    .parent()
                    .map_or_else(String::new, |s| s.to_string_lossy().into_owned());

                let node = &snapshot.nodes[node_idx as usize];

                let is_original = f_idx == 0;

                // A node is a hardlink if it shares its file ID with another node in the group
                let is_hardlink = file_id != (0, 0)
                    && paired.iter().any(|other| other.0 != node_idx && other.1 == file_id);

                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(group.size)
                    .to_string();

                let reclaimable_str = if is_original {
                    prettier_bytes::ByteFormatter::new()
                        .format(total_reclaimable)
                        .to_string()
                } else {
                    let individual_reclaimable = if is_hardlink {
                        0
                    } else {
                        group.size
                    };
                    prettier_bytes::ByteFormatter::new()
                        .format(individual_reclaimable)
                        .to_string()
                };

                let created_time_str = format_timestamp(node.created_timestamp);
                let modified_time_str = format_timestamp(node.modified_timestamp);

                flat_rows.push(DuplicateRow {
                    node_idx,
                    group_idx: g_idx,
                    filename,
                    parent_path,
                    size: group.size,
                    size_str,
                    reclaimable_str,
                    created_time_str,
                    modified_time_str,
                    is_original,
                    is_hardlink,
                });
            }
        }
        self.deduplicator_flat_rows = flat_rows;
        self.deduplicator_last_sig = current_sig;
    }

    fn draw_deduplicator_results(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        let progress_snap = self.deduplicator_progress.snapshot();
        let is_running = !progress_snap.finished && progress_snap.elapsed.is_some();

        let active_groups = self.deduplicator_results.read().clone();
        let active_groups_len = active_groups.len();

        let current_total_files: usize = active_groups.iter().map(|g| g.nodes.len()).sum();
        let current_sig = (active_groups.len(), current_total_files);
        if self.deduplicator_last_sig != current_sig {
            self.check_rebuild_flat_rows(snapshot, &active_groups);
        }

        if active_groups_len == 0 && self.deduplicator_flat_rows.is_empty() {
            ui.centered_and_justified(|ui| {
                if is_running {
                    ui.label("Analyzing files...");
                } else {
                    ui.label("No duplicate groups found. Try reducing the Minimum File Size or scanning a different folder.");
                }
            });
            return;
        }

        ui.horizontal_wrapped(|ui| {
            if ui.button("🎯 Select All But Oldest").clicked() {
                self.selected_duplicates.clear();
                for group in &active_groups {
                    let mut oldest_node: Option<(u32, i64)> = None;
                    for &idx in &group.nodes {
                        let mod_time = snapshot.nodes[idx as usize].modified_timestamp;
                        match oldest_node {
                            None => oldest_node = Some((idx, mod_time)),
                            Some((_, oldest_time)) => {
                                if mod_time < oldest_time {
                                    oldest_node = Some((idx, mod_time));
                                }
                            }
                        }
                    }
                    if let Some((oldest_idx, _)) = oldest_node {
                        for &idx in &group.nodes {
                            if idx != oldest_idx {
                                self.selected_duplicates.insert(idx);
                            }
                        }
                    }
                }
            }

            if ui.button("🎯 Select All But Newest").clicked() {
                self.selected_duplicates.clear();
                for group in &active_groups {
                    let mut newest_node: Option<(u32, i64)> = None;
                    for &idx in &group.nodes {
                        let mod_time = snapshot.nodes[idx as usize].modified_timestamp;
                        match newest_node {
                            None => newest_node = Some((idx, mod_time)),
                            Some((_, newest_time)) => {
                                if mod_time > newest_time {
                                    newest_node = Some((idx, mod_time));
                                }
                            }
                        }
                    }
                    if let Some((newest_idx, _)) = newest_node {
                        for &idx in &group.nodes {
                            if idx != newest_idx {
                                self.selected_duplicates.insert(idx);
                            }
                        }
                    }
                }
            }

            if ui.button("🎯 Select All But Shortest Path").clicked() {
                self.selected_duplicates.clear();
                for group in &active_groups {
                    let mut best_node: Option<(u32, usize)> = None;
                    for &idx in &group.nodes {
                        let path_len = snapshot.get_full_path(idx).len();
                        match best_node {
                            None => best_node = Some((idx, path_len)),
                            Some((_, best_len)) => {
                                if path_len < best_len {
                                    best_node = Some((idx, path_len));
                                }
                            }
                        }
                    }
                    if let Some((kept_idx, _)) = best_node {
                        for &idx in &group.nodes {
                            if idx != kept_idx {
                                self.selected_duplicates.insert(idx);
                            }
                        }
                    }
                }
            }

            if ui.button("🎯 Select All But Root-most").clicked() {
                self.selected_duplicates.clear();
                for group in &active_groups {
                    let mut best_node: Option<(u32, usize)> = None;
                    for &idx in &group.nodes {
                        let mut depth = 0;
                        let mut curr = idx;
                        while let Some(parent) = snapshot.nodes.get(curr as usize).and_then(|n| n.parent_opt()) {
                            depth += 1;
                            curr = parent;
                        }
                        match best_node {
                            None => best_node = Some((idx, depth)),
                            Some((_, best_depth)) => {
                                if depth < best_depth {
                                    best_node = Some((idx, depth));
                                }
                            }
                        }
                    }
                    if let Some((kept_idx, _)) = best_node {
                        for &idx in &group.nodes {
                            if idx != kept_idx {
                                self.selected_duplicates.insert(idx);
                            }
                        }
                    }
                }
            }

            if ui.button("❌ Clear Selection").clicked() {
                self.selected_duplicates.clear();
            }

            ui.separator();

            let has_selection = !self.selected_duplicates.is_empty();
            let total_selected_size: u64 = self
                .selected_duplicates
                .iter()
                .map(|&idx| snapshot.nodes[idx as usize].size)
                .sum();
            let reclaim_str = prettier_bytes::ByteFormatter::new()
                .format(total_selected_size)
                .to_string();

            let trash_btn = ui.add_enabled(
                has_selection,
                egui::Button::new(
                    egui::RichText::new(format!(
                        "♻ Move Selected to Trash ({} files, {})",
                        self.selected_duplicates.len(),
                        reclaim_str
                    ))
                    .color(egui::Color32::WHITE)
                    .strong(),
                )
                .fill(crate::colors::TRASH_BORDER),
            );

            if trash_btn.clicked() {
                self.delete_duplicates_indices = self.selected_duplicates.iter().copied().collect();
                self.delete_confirm_checked = false;
                self.active_modal = Some(crate::gui::ActiveModal::TrashDuplicates);
            }

            let delete_btn = ui.add_enabled(
                has_selection,
                egui::Button::new(
                    egui::RichText::new(format!(
                        "🗑 Delete Selected ({} files, {})",
                        self.selected_duplicates.len(),
                        reclaim_str
                    ))
                    .color(egui::Color32::WHITE)
                    .strong(),
                )
                .fill(crate::colors::DELETION_BORDER),
            );

            if delete_btn.clicked() {
                self.delete_duplicates_indices = self.selected_duplicates.iter().copied().collect();
                self.delete_confirm_checked = false;
                self.active_modal = Some(crate::gui::ActiveModal::DeleteDuplicates);
            }
        });

        ui.add_space(6.0);

        let toggled_node = std::cell::Cell::new(None);
        let flat_rows = &self.deduplicator_flat_rows;
        let selected = &self.selected_duplicates;
        let monospace_paths = self.monospace_paths;
        let is_scan_running = is_running;

        let max_width = ui.available_width();
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_width(max_width)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .id_salt("deduplicator_table")
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto().at_least(30.0)) // Checkbox
                    .column(Column::initial(180.0).range(100.0..=500.0).clip(true)) // Filename
                    .column(Column::initial(300.0).range(100.0..=1000.0).clip(true)) // Folder Path
                    .column(Column::initial(80.0)) // Size
                    .column(Column::initial(90.0)) // Reclaimable Space
                    .column(Column::initial(130.0)) // Created Time
                    .column(Column::initial(130.0)) // Modified Time
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("[     ]");
                        });
                        header.col(|ui| {
                            ui.strong("Filename");
                        });
                        header.col(|ui| {
                            ui.strong("Folder");
                        });
                        header.col(|ui| {
                            ui.strong("Size");
                        });
                        header.col(|ui| {
                            ui.strong("Reclaimable");
                        });
                        header.col(|ui| {
                            ui.strong("Created");
                        });
                        header.col(|ui| {
                            ui.strong("Modified");
                        });
                    })
                    .body(|body| {
                        body.rows(22.0, flat_rows.len(), |mut row| {
                            let r_idx = row.index();
                            let row_data = &flat_rows[r_idx];

                            row.col(|ui| {
                                ui.add_enabled_ui(!is_scan_running, |ui| {
                                    let mut is_checked = selected.contains(&row_data.node_idx);
                                    if ui.checkbox(&mut is_checked, "").changed() {
                                        toggled_node.set(Some((row_data.node_idx, is_checked)));
                                    }
                                });
                            });

                            row.col(|ui| {
                                let column_width = ui.available_width();
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.style_mut().wrap_mode =
                                            Some(egui::TextWrapMode::Truncate);
                                        ui.set_max_width(column_width);
                                        let name_rich = if row_data.is_original {
                                            egui::RichText::new(format!("⭐ {}", row_data.filename))
                                                .color(ui.visuals().text_color())
                                        } else {
                                            egui::RichText::new(format!(
                                                "      >> {}",
                                                row_data.filename
                                            ))
                                            .strong()
                                            .color(egui::Color32::from_rgb(245, 158, 11)) // Orange for duplicate
                                        };
                                        ui.label(name_rich).on_hover_text(&row_data.filename);
                                        if row_data.is_hardlink {
                                            ui.add_space(4.0);
                                            let frame = egui::Frame::new()
                                                .fill(ui.visuals().selection.bg_fill.linear_multiply(0.15))
                                                .stroke(egui::Stroke::new(1.0, ui.visuals().selection.stroke.color.linear_multiply(0.5)))
                                                .inner_margin(egui::Margin::symmetric(4, 2))
                                                .corner_radius(3.0);
                                            frame.show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new("Hardlink")
                                                        .size(10.0)
                                                        .strong()
                                                        .color(ui.visuals().selection.stroke.color),
                                                );
                                            });
                                        }
                                    },
                                );
                            });

                            row.col(|ui| {
                                let column_width = ui.available_width();
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.style_mut().wrap_mode =
                                            Some(egui::TextWrapMode::Truncate);
                                        ui.set_max_width(column_width);
                                        let mut path_rich =
                                            egui::RichText::new(&row_data.parent_path).weak();
                                        if monospace_paths {
                                            path_rich = path_rich.monospace();
                                        }

                                        // Make folder label clickable to open in file explorer
                                        let response = ui.add(
                                            egui::Label::new(path_rich).sense(egui::Sense::click()),
                                        );

                                        if response.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if response.clicked() {
                                            let _ = open::that(&row_data.parent_path);
                                        }
                                    },
                                );
                            });

                            row.col(|ui| {
                                ui.label(&row_data.size_str);
                            });

                            row.col(|ui| {
                                if row_data.is_original {
                                    // Display the original group-sum reclaimable value in standard green
                                    ui.colored_label(
                                        crate::colors::COLOR_SCAN_COMPLETE,
                                        &row_data.reclaimable_str,
                                    );
                                } else {
                                    // Display the duplicate rows in a lighter pastel mint-green
                                    let light_green = egui::Color32::from_rgb(134, 239, 172);
                                    ui.colored_label(light_green, &row_data.reclaimable_str);
                                }
                            });

                            row.col(|ui| {
                                ui.label(&row_data.created_time_str);
                            });

                            row.col(|ui| {
                                ui.label(&row_data.modified_time_str);
                            });
                        });
                    });
            });

        if let Some((node_idx, is_checked)) = toggled_node.get() {
            if is_checked {
                self.selected_duplicates.insert(node_idx);
            } else {
                self.selected_duplicates.remove(&node_idx);
            }
        }
    }
}
