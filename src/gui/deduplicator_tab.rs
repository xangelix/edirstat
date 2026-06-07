#![allow(clippy::cast_precision_loss)]

use std::sync::Arc;
use std::sync::atomic::Ordering;

use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::arena::FileArenaSnapshot;
use crate::stats::deduplicator::run_deduplication;

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

        ui.vertical(|ui| {
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

                    let stroke = egui::Stroke::new(2.0, crate::colors::WARNING_RED);
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
                        dir_counts: snapshot.dir_counts.clone(),
                    });

                    self.deduplicator_progress = atomic_progress::ProgressBuilder::new_spinner(
                        "Phase 1/7: Grouping all scanned files by size...",
                    )
                    .with_start_time_now()
                    .build();

                    *self.deduplicator_results.write() =
                        crate::stats::deduplicator::DeduplicationResults::default();

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
                    ui.colored_label(
                        crate::colors::COLOR_WARNING_YELLOW,
                        format!("Scan was cancelled: {err}"),
                    );
                } else if !progress_snap.name.is_empty() && progress_snap.name != "Deduplicator" {
                    ui.colored_label(
                        crate::colors::COLOR_SCAN_COMPLETE,
                        progress_snap.name.as_str(),
                    );
                }
            }
        }
    }

    fn draw_deduplicator_results(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        let progress_snap = self.deduplicator_progress.snapshot();
        let is_running = !progress_snap.finished && progress_snap.elapsed.is_some();

        let results_lock = Arc::clone(&self.deduplicator_results);

        {
            let results_guard = results_lock.read();

            if results_guard.groups.is_empty() && results_guard.flat_rows.is_empty() {
                ui.centered_and_justified(|ui| {
                    if is_running {
                        ui.label("Analyzing files...");
                    } else {
                        ui.label("No duplicate groups found. Try reducing the Minimum File Size or scanning a different folder.");
                    }
                });
                return;
            }
        }

        ui.horizontal_wrapped(|ui| {
            ui.scope(|ui| {
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = crate::colors::BUTTON_BLUE;
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    crate::colors::BUTTON_BLUE_HOVER;
                ui.style_mut().visuals.widgets.active.weak_bg_fill = crate::colors::BUTTON_BLUE;
                ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

                let select_label = egui::RichText::new("🎯 Select items...")
                    .color(crate::colors::COLOR_WHITE)
                    .strong();

                let menu_config = egui::containers::menu::MenuConfig::default()
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);

                egui::containers::menu::MenuButton::new(select_label)
                    .config(menu_config)
                    .ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.style_mut().visuals.widgets =
                            ui.ctx().global_style().visuals.widgets.clone();

                        if ui.button("🎯 All But Oldest").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
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
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🎯 All But Newest").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
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
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🎯 All But Shortest Path").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
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
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🎯 All But Root-most").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
                                let mut best_node: Option<(u32, usize)> = None;
                                for &idx in &group.nodes {
                                    let mut depth = 0;
                                    let mut curr = idx;
                                    while let Some(parent) = snapshot
                                        .nodes
                                        .get(curr as usize)
                                        .and_then(crate::arena::FileNode::parent_opt)
                                    {
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
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🎯 All But Longest Path").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
                                let mut best_node: Option<(u32, usize)> = None;
                                for &idx in &group.nodes {
                                    let path_len = snapshot.get_full_path(idx).len();
                                    match best_node {
                                        None => best_node = Some((idx, path_len)),
                                        Some((_, max_len)) => {
                                            if path_len > max_len {
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
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Preferred Directory Pattern:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.deduplicator_dir_filter)
                                    .hint_text("e.g. /home/user/Archive")
                                    .desired_width(200.0),
                            );
                        });

                        if ui.button("🎯 All But Preferred Directory").clicked() {
                            self.selected_duplicates.clear();
                            for group in &results_lock.read().groups {
                                let mut preferred_idx: Option<u32> = None;
                                for &idx in &group.nodes {
                                    let path_str = snapshot.get_full_path(idx);
                                    if !self.deduplicator_dir_filter.is_empty()
                                        && path_str.contains(&self.deduplicator_dir_filter)
                                    {
                                        preferred_idx = Some(idx);
                                        break;
                                    }
                                }
                                let kept_idx = preferred_idx
                                    .unwrap_or_else(|| group.nodes.first().copied().unwrap_or(0));
                                for &idx in &group.nodes {
                                    if idx != kept_idx {
                                        self.selected_duplicates.insert(idx);
                                    }
                                }
                            }
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        ui.separator();

                        if ui.button("❌ Clear Selection").clicked() {
                            self.selected_duplicates.clear();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
            });

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

            ui.add_enabled_ui(has_selection, |ui| {
                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        crate::colors::BUTTON_ORANGE;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        crate::colors::BUTTON_ORANGE_HOVER;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill =
                        crate::colors::BUTTON_ORANGE;
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

                    let link_button_text = if has_selection {
                        format!("🔗 Link... ({} files)", self.selected_duplicates.len())
                    } else {
                        "🔗 Link...".to_string()
                    };

                    let link_label = egui::RichText::new(link_button_text)
                        .color(crate::colors::COLOR_WHITE)
                        .strong();
                    ui.menu_button(link_label, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                        if ui.button("🔗 Replace Selected with Hardlinks").clicked() {
                            self.delete_duplicates_indices =
                                self.selected_duplicates.iter().copied().collect();
                            self.delete_confirm_checked = false;
                            self.active_modal = Some(crate::gui::ActiveModal::HardlinkDuplicates);
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🔗 Replace Selected with Softlinks").clicked() {
                            self.delete_duplicates_indices =
                                self.selected_duplicates.iter().copied().collect();
                            self.delete_confirm_checked = false;
                            self.active_modal = Some(crate::gui::ActiveModal::SoftlinkDuplicates);
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                });
            });

            // Combined drop-down menu button styled with deletion red hues
            ui.add_enabled_ui(has_selection, |ui| {
                ui.scope(|ui| {
                    // Custom styles to make the dropdown host button look prominently Red (Deletion)
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        crate::colors::DELETION_BORDER;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        crate::colors::DELETION_WARNING;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill =
                        crate::colors::DELETION_BORDER;
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

                    let button_text = if has_selection {
                        format!(
                            "🗑 Remove... ({} files, {})",
                            self.selected_duplicates.len(),
                            reclaim_str
                        )
                    } else {
                        "🗑 Remove...".to_string()
                    };

                    let remove_label = egui::RichText::new(button_text)
                        .color(crate::colors::COLOR_WHITE)
                        .strong();
                    ui.menu_button(remove_label, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                        if ui.button("♻ Move Selected to Trash").clicked() {
                            self.delete_duplicates_indices =
                                self.selected_duplicates.iter().copied().collect();
                            self.delete_confirm_checked = false;
                            self.active_modal = Some(crate::gui::ActiveModal::TrashDuplicates);
                            ui.close_kind(egui::UiKind::Menu);
                        }

                        if ui.button("🗑 Delete Selected Permanently").clicked() {
                            self.delete_duplicates_indices =
                                self.selected_duplicates.iter().copied().collect();
                            self.delete_confirm_checked = false;
                            self.active_modal = Some(crate::gui::ActiveModal::DeleteDuplicates);
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                });
            });
        });

        ui.add_space(6.0);

        let toggled_node = std::cell::Cell::new(None);
        let selected = &self.selected_duplicates;
        let monospace_paths = self.monospace_paths;
        let is_scan_running = is_running;

        let max_width = ui.available_width();
        let row_len = results_lock.read().flat_rows.len();
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_width(max_width)
            .show(ui, |ui| {
                let available_height = ui.available_height();
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
                    .min_scrolled_height(0.0)
                    .max_scroll_height(available_height)
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
                        body.rows(22.0, row_len, |mut row| {
                            let r_idx = row.index();
                            let row_data = &results_lock.read().flat_rows[r_idx];

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
                                            .color(crate::colors::COLOR_DUPLICATE_ORANGE) // Orange for duplicate
                                        };
                                        ui.label(name_rich).on_hover_text(&row_data.filename);
                                        if row_data.is_hardlink {
                                            ui.add_space(4.0);
                                            let frame = egui::Frame::new()
                                                .fill(
                                                    ui.visuals()
                                                        .selection
                                                        .bg_fill
                                                        .linear_multiply(0.15),
                                                )
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    ui.visuals()
                                                        .selection
                                                        .stroke
                                                        .color
                                                        .linear_multiply(0.5),
                                                ))
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
                                    let light_green = crate::colors::COLOR_LIGHT_GREEN;
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
