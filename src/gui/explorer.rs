use std::sync::atomic::Ordering;

use eframe::egui;
use smallvec::SmallVec;

use super::{ActiveModal, GuiApp, theme};
use crate::arena::{FileArenaSnapshot, NO_INDEX};

pub struct QueryCoordinator {
    pub cached_node_matches: Vec<bool>,
    last_search_query: String,
    last_filter_case_sensitive: bool,
    last_filter_regex: bool,
    last_highlight_duplicates: bool,
    last_search_snapshot_ptr: usize,
    last_selected_duplicates_len: usize,
}

impl QueryCoordinator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_node_matches: Vec::new(),
            last_search_query: String::new(),
            last_filter_case_sensitive: false,
            last_filter_regex: false,
            last_highlight_duplicates: false,
            last_search_snapshot_ptr: 0,
            last_selected_duplicates_len: 0,
        }
    }

    pub fn update(
        &mut self,
        snapshot: &FileArenaSnapshot,
        search_query: &str,
        filter_case_sensitive: bool,
        filter_regex: bool,
        highlight_duplicates: bool,
        selected_duplicates: &std::collections::HashSet<u32>,
    ) {
        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;

        let needs_rebuild = search_query != self.last_search_query
            || filter_case_sensitive != self.last_filter_case_sensitive
            || filter_regex != self.last_filter_regex
            || highlight_duplicates != self.last_highlight_duplicates
            || selected_duplicates.len() != self.last_selected_duplicates_len
            || snapshot_ptr != self.last_search_snapshot_ptr
            || self.cached_node_matches.is_empty();

        if !needs_rebuild {
            return;
        }

        self.last_search_query = search_query.to_string();
        self.last_filter_case_sensitive = filter_case_sensitive;
        self.last_filter_regex = filter_regex;
        self.last_highlight_duplicates = highlight_duplicates;
        self.last_selected_duplicates_len = selected_duplicates.len();
        self.last_search_snapshot_ptr = snapshot_ptr;

        self.cached_node_matches.clear();
        self.cached_node_matches.resize(snapshot.nodes.len(), false);

        if !search_query.is_empty() && !snapshot.nodes.is_empty() {
            let regex_matcher = if filter_regex {
                let mut builder = regex::RegexBuilder::new(search_query);
                builder.case_insensitive(!filter_case_sensitive);
                builder.build().ok()
            } else {
                None
            };

            // Single-pass O(N) reverse propagation of matched subtrees
            let search_query_lower = search_query.to_lowercase();
            for idx in (0..snapshot.nodes.len()).rev() {
                let node = &snapshot.nodes[idx];
                let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");

                let self_matches = regex_matcher.as_ref().map_or_else(
                    || {
                        if filter_case_sensitive {
                            name.contains(search_query)
                        } else {
                            crate::arena::contains_case_insensitive(name, &search_query_lower)
                        }
                    },
                    |re| re.is_match(name),
                );

                if self_matches {
                    self.cached_node_matches[idx] = true;
                }

                // If this node matches, flag its parent recursively up to the root
                if self.cached_node_matches[idx]
                    && let Some(parent) = node.parent_opt()
                    && (parent as usize) < self.cached_node_matches.len()
                {
                    self.cached_node_matches[parent as usize] = true;
                }
            }
        }

        if highlight_duplicates {
            for &idx in selected_duplicates {
                if (idx as usize) < self.cached_node_matches.len() {
                    self.cached_node_matches[idx as usize] = true;

                    // Propagate match to parents to keep them visible in the tree
                    let mut curr = Some(idx);
                    while let Some(c_idx) = curr {
                        let node = &snapshot.nodes[c_idx as usize];
                        if let Some(parent) = node.parent_opt() {
                            self.cached_node_matches[parent as usize] = true;
                            curr = Some(parent);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl Default for QueryCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiApp {
    pub fn handle_node_click(
        &mut self,
        clicked_node_idx: u32,
        modifiers: egui::Modifiers,
        visible_nodes: &[(u32, usize)],
    ) {
        if modifiers.command || modifiers.ctrl {
            // Toggle selection
            if self.selected_nodes.contains(&clicked_node_idx) {
                self.selected_nodes.remove(&clicked_node_idx);
            } else {
                self.selected_nodes.insert(clicked_node_idx);
                self.focus_node_idx = Some(clicked_node_idx);
            }
        } else if modifiers.shift && self.focus_node_idx.is_some() {
            if let Some(focus) = self.focus_node_idx {
                // Range selection in visible_nodes
                let pos_focus = visible_nodes.iter().position(|&(idx, _)| idx == focus);
                let pos_clicked = visible_nodes
                    .iter()
                    .position(|&(idx, _)| idx == clicked_node_idx);
                if let (Some(idx_a), Some(idx_b)) = (pos_focus, pos_clicked) {
                    let start = idx_a.min(idx_b);
                    let end = idx_a.max(idx_b);
                    for &(idx, _) in visible_nodes.iter().take(end + 1).skip(start) {
                        self.selected_nodes.insert(idx);
                    }
                }
            }
        } else {
            // Normal click: select only this node
            self.selected_nodes.clear();
            self.selected_nodes.insert(clicked_node_idx);
            self.focus_node_idx = Some(clicked_node_idx);
        }

        // Sync back selected_node_idx
        if self.selected_nodes.len() == 1 {
            self.selected_node_idx = self.selected_nodes.iter().next().copied();
        } else {
            self.selected_node_idx = None;
        }
    }

    pub fn flatten_visible_tree(
        &mut self,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
        indent_level: usize,
        out: &mut Vec<(u32, usize)>,
    ) {
        self.query_coordinator.update(
            snapshot,
            &self.search_query,
            self.filter_case_sensitive,
            self.filter_regex,
            self.highlight_duplicates,
            &self.selected_duplicates,
        );

        // Shared borrow of immutable self components (safely bypasses simultaneous borrow limits)
        self.flatten_visible_tree_impl(
            snapshot,
            node_idx,
            indent_level,
            out,
            &self.query_coordinator.cached_node_matches,
        );
    }

    fn flatten_visible_tree_impl(
        &self,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
        indent_level: usize,
        out: &mut Vec<(u32, usize)>,
        node_matches: &[bool],
    ) {
        let node = &snapshot.nodes[node_idx as usize];

        // Filter search query: O(1) matching subtree lookup
        if !self.search_query.is_empty()
            && (node_idx as usize) < node_matches.len()
            && !node_matches[node_idx as usize]
        {
            return;
        }

        out.push((node_idx, indent_level));

        let is_expanded = self.expanded_nodes.contains(&node_idx);
        let has_children = node.is_directory() && node.first_child != NO_INDEX;

        if is_expanded && has_children {
            let mut sorted_child_indices = SmallVec::<[u32; 16]>::new();
            let mut curr = node.first_child;
            while curr != NO_INDEX {
                sorted_child_indices.push(curr);
                curr = snapshot.nodes[curr as usize].next_sibling;
            }
            // Sort immediate children by size descending dynamically for correct tree views
            sorted_child_indices.sort_by(|&a, &b| {
                snapshot.nodes[b as usize]
                    .size
                    .cmp(&snapshot.nodes[a as usize].size)
            });

            for &child_idx in &sorted_child_indices {
                self.flatten_visible_tree_impl(
                    snapshot,
                    child_idx,
                    indent_level + 1,
                    out,
                    node_matches,
                );
            }
        }
    }

    pub fn render_tree_node_row(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
        indent_level: usize,
        visible_nodes: &[(u32, usize)],
    ) {
        let is_duplicate =
            self.highlight_duplicates && self.selected_duplicates.contains(&node_idx);

        let node = &snapshot.nodes[node_idx as usize];
        let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");

        let is_expanded = self.expanded_nodes.contains(&node_idx);
        let has_children = node.is_directory() && node.first_child != NO_INDEX;
        let is_selected = self.selected_nodes.contains(&node_idx);

        let horizontal_res = ui.horizontal(|ui| {
            // Indent padding
            #[allow(clippy::cast_precision_loss)]
            ui.add_space(indent_level as f32 * 16.0);

            // Icon & Expand Arrow
            let icon_text = if node.is_symlink() {
                "🔗"
            } else if node.is_directory() {
                "📁"
            } else {
                "📄"
            };

            if has_children {
                let arrow = if is_expanded { "[-]" } else { "[+]" };
                let rich_arrow = egui::RichText::new(arrow).monospace();
                let label = ui.selectable_label(is_expanded, rich_arrow);
                if label.clicked() {
                    if is_expanded {
                        self.expanded_nodes.remove(&node_idx);
                    } else {
                        self.expanded_nodes.insert(node_idx);
                    }
                }
            } else {
                ui.add_space(22.0); // Arrow placeholder alignment space matching "[+]"
            }

            ui.label(icon_text);

            // Node Name / Label with automatic left-aligned truncation
            let mut rich_name = egui::RichText::new(name);
            if self.monospace_paths {
                rich_name = rich_name.monospace();
            }
            if is_selected {
                rich_name = rich_name
                    .strong()
                    .color(ui.visuals().selection.stroke.color);
            } else if is_duplicate {
                rich_name = rich_name.color(crate::colors::GLOW_INNER_CORE);
            }

            // Allocate exactly the remaining width minus space for the size column (72px subtracted)
            let name_width = (ui.available_width() - 72.0).max(50.0);

            ui.allocate_ui(egui::vec2(name_width, ui.spacing().interact_size.y), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                ui.label(rich_name);
            });

            // Muted size details (far right aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    prettier_bytes::ByteFormatter::new()
                        .format(node.size)
                        .to_string(),
                );
            });
        });

        // Get the bounding box of the whole row
        let rect = horizontal_res.response.rect;

        // --- Offset the interaction hitbox strictly to the right of the expand button ---
        let mut interactive_rect = rect;
        #[allow(clippy::cast_precision_loss)]
        let expand_button_width = (indent_level as f32).mul_add(16.0, 24.0);
        interactive_rect.min.x += expand_button_width;

        let row_id = ui.id().with(("tree_row", node_idx));
        let response = ui.interact(interactive_rect, row_id, egui::Sense::click());

        // Draw professional background selection / hover highlights over the FULL row (for seamless visual style)
        if is_selected {
            let fill_color = ui.visuals().selection.bg_fill.linear_multiply(0.12);
            ui.painter().rect_filled(rect, 4.0, fill_color);
        } else if response.hovered() {
            let hover_color = ui.visuals().widgets.hovered.bg_fill.linear_multiply(0.04);
            ui.painter().rect_filled(rect, 4.0, hover_color);
        }

        // Handle selection on Left-Click or Right-Click (only outside of the expand button)
        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);
            self.handle_node_click(node_idx, modifiers, visible_nodes);
        } else if response.secondary_clicked() && !self.selected_nodes.contains(&node_idx) {
            self.selected_nodes.clear();
            self.selected_nodes.insert(node_idx);
            self.selected_node_idx = Some(node_idx);
            self.focus_node_idx = Some(node_idx);
        }

        // Render the context menu on Right-Click
        response.context_menu(|ui| {
            self.draw_file_menu_contents(ui, snapshot);
        });

        // Draw vertical indentation guidelines to visually track nested guidelines
        let painter = ui.painter();
        let stroke = egui::Stroke::new(1.0, theme::INDENT_GUIDELINE);
        for i in 0..indent_level {
            #[allow(clippy::cast_precision_loss)]
            let x = (i as f32).mul_add(16.0, rect.min.x) + 8.0;

            // Draw a dashed vertical line
            let dash_length = 2.0;
            let gap_length = 2.0;
            let step = dash_length + gap_length;
            let total_height = rect.max.y - rect.min.y;
            if total_height > 0.0 {
                let num_steps = (total_height / step).ceil() as usize;
                for step_idx in 0..num_steps {
                    #[allow(clippy::cast_precision_loss)]
                    let segment_y = (step_idx as f32).mul_add(step, rect.min.y);

                    let next_y = (segment_y + dash_length).min(rect.max.y);
                    painter.line_segment([egui::pos2(x, segment_y), egui::pos2(x, next_y)], stroke);
                }
            }
        }
    }

    pub fn render_hierarchical_table(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        if snapshot.nodes.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Click 'Scan Directory' to explore disk usage.");
            });
            return;
        }

        // Query modifier keys here before the outer `ui` is borrowed by TableBuilder
        let modifiers = ui.input(|i| i.modifiers);

        // Auto-expand root
        if self.expanded_nodes.is_empty() {
            self.expanded_nodes.insert(0);
        }

        let mut visible_nodes = Vec::new();
        self.flatten_visible_tree(snapshot, 0, 0, &mut visible_nodes);

        let row_height = 24.0;
        let available_height = ui.available_height();
        let visuals = ui.visuals().clone();

        let mut next_hovered = None;

        egui_extras::TableBuilder::new(ui)
            .id_salt("hierarchical_file_table")
            .sense(egui::Sense::click())
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::remainder().clip(true)) // Name
            .column(egui_extras::Column::initial(140.0).range(80.0..=400.0)) // Percentage
            .column(egui_extras::Column::initial(90.0).range(60.0..=200.0)) // Size
            .column(egui_extras::Column::initial(70.0).range(40.0..=150.0)) // Items
            .column(egui_extras::Column::initial(70.0).range(40.0..=150.0)) // Files
            .column(egui_extras::Column::initial(70.0).range(40.0..=150.0)) // Subdirs
            .column(egui_extras::Column::initial(150.0).range(100.0..=300.0)) // Last Modified
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(24.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Percentage");
                });
                header.col(|ui| {
                    ui.strong("Size");
                });
                header.col(|ui| {
                    ui.strong("Items");
                });
                header.col(|ui| {
                    ui.strong("Files");
                });
                header.col(|ui| {
                    ui.strong("Subdirs");
                });
                header.col(|ui| {
                    ui.strong("Last Modified");
                });
            })
            .body(|body| {
                body.rows(row_height, visible_nodes.len(), |mut row| {
                    let mut arrow_hovered = false;
                    let r_idx = row.index();
                    let (node_idx, indent) = visible_nodes[r_idx];
                    let node = &snapshot.nodes[node_idx as usize];
                    let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");
                    let is_selected = self.selected_nodes.contains(&node_idx);
                    let is_hovered = self.hovered_node_idx == Some(node_idx);
                    let is_expanded = self.expanded_nodes.contains(&node_idx);
                    let has_children = node.is_directory() && node.first_child != NO_INDEX;
                    let is_duplicate =
                        self.highlight_duplicates && self.selected_duplicates.contains(&node_idx);

                    let files_count = if node.is_directory() {
                        node.file_count
                    } else {
                        0
                    };
                    let subdirs_count = if node.is_directory() {
                        *snapshot.dir_counts.get(node_idx as usize).unwrap_or(&0)
                    } else {
                        0
                    };
                    let items_count = files_count + subdirs_count;

                    // Row-level click/hover state variables
                    let mut row_clicked = false;
                    let mut row_secondary_clicked = false;
                    let mut row_hovered_by_mouse = false;

                    let paint_bg = |ui: &mut egui::Ui, col_idx: usize| {
                        let mut cell_rect = ui.max_rect();
                        cell_rect.set_height(row_height);
                        let spacing = ui.spacing().item_spacing.x;
                        let mut highlight_rect = cell_rect;
                        if col_idx > 0 {
                            highlight_rect.min.x -= spacing / 2.0;
                        } else {
                            highlight_rect.min.x = ui.clip_rect().min.x;
                        }
                        if col_idx < 6 {
                            highlight_rect.max.x += spacing / 2.0;
                        } else {
                            highlight_rect.max.x = ui.clip_rect().max.x;
                        }
                        if is_selected {
                            let fill_color = visuals.selection.bg_fill.linear_multiply(0.20);
                            ui.painter().rect_filled(highlight_rect, 0.0, fill_color);
                        } else if is_hovered {
                            let hover_color = visuals.widgets.hovered.bg_fill.linear_multiply(0.08);
                            ui.painter().rect_filled(highlight_rect, 0.0, hover_color);
                        }
                    };

                    // Render Name Column
                    row.col(|ui| {
                        paint_bg(ui, 0);

                        // Indent space
                        #[allow(clippy::cast_precision_loss)]
                        ui.add_space(indent as f32 * 16.0);

                        // Collapse/expand arrow with exact width of 16.0
                        let (arrow_rect, arrow_resp) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
                        if has_children {
                            arrow_hovered = arrow_resp.hovered();
                            let arrow = if is_expanded { "⏷" } else { "⏵" };

                            let arrow_color = if arrow_resp.hovered() {
                                egui::Color32::from_rgb(59, 130, 246) // Bright blue hover indication
                            } else {
                                ui.visuals().widgets.inactive.text_color()
                            };

                            ui.painter().text(
                                arrow_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                arrow,
                                egui::FontId::monospace(12.0),
                                arrow_color,
                            );
                            if arrow_resp.clicked() {
                                if is_expanded {
                                    self.expanded_nodes.remove(&node_idx);
                                } else {
                                    self.expanded_nodes.insert(node_idx);
                                }
                            }
                            if arrow_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        }

                        // Icon
                        let icon_text = if node.is_symlink() {
                            "🔗"
                        } else if node.is_directory() {
                            "📁"
                        } else {
                            "📄"
                        };
                        ui.label(icon_text);

                        // Name label
                        let mut rich_name = egui::RichText::new(name);
                        if self.monospace_paths {
                            rich_name = rich_name.monospace();
                        }
                        if is_selected {
                            rich_name = rich_name
                                .strong()
                                .color(ui.visuals().selection.stroke.color);
                        } else if is_duplicate {
                            rich_name = rich_name.color(theme::GLOW_INNER_CORE);
                        }

                        let name_width = ui.available_width().max(50.0);
                        ui.allocate_ui(
                            egui::vec2(name_width, ui.spacing().interact_size.y),
                            |ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                ui.label(rich_name);
                            },
                        );

                        // Render vertical dashed guidelines for tree structure
                        let painter = ui.painter();
                        let stroke = egui::Stroke::new(1.0, theme::INDENT_GUIDELINE);
                        let mut cell_rect_guide = ui.max_rect();
                        cell_rect_guide.set_height(row_height);
                        for i in 0..indent {
                            #[allow(clippy::cast_precision_loss)]
                            let x = (i as f32).mul_add(16.0, cell_rect_guide.min.x) + 8.0;

                            let dash_length = 2.0;
                            let gap_length = 2.0;
                            let step = dash_length + gap_length;
                            let total_height = cell_rect_guide.max.y - cell_rect_guide.min.y;
                            if total_height > 0.0 {
                                let num_steps = (total_height / step).ceil() as usize;
                                for step_idx in 0..num_steps {
                                    #[allow(clippy::cast_precision_loss)]
                                    let segment_y =
                                        (step_idx as f32).mul_add(step, cell_rect_guide.min.y);
                                    let next_y =
                                        (segment_y + dash_length).min(cell_rect_guide.max.y);
                                    painter.line_segment(
                                        [egui::pos2(x, segment_y), egui::pos2(x, next_y)],
                                        stroke,
                                    );
                                }
                            }
                        }

                        // Interact with the cell strictly to the right of the expand arrow
                        let mut interact_rect = ui.max_rect();
                        #[allow(clippy::cast_precision_loss)]
                        let offset_width = (indent as f32).mul_add(16.0, 16.0);
                        interact_rect.min.x =
                            (interact_rect.min.x + offset_width).min(interact_rect.max.x);

                        let cell_resp = ui.interact(
                            interact_rect,
                            ui.id().with(("cell_interact", 0)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() && !arrow_hovered {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() && !arrow_hovered {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() && !arrow_hovered {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Percentage Column
                    row.col(|ui| {
                        paint_bg(ui, 1);

                        let mut cell_rect = ui.max_rect();
                        cell_rect.set_height(row_height);

                        let parent_idx = node.parent;
                        let parent_size = if parent_idx == crate::arena::NO_INDEX {
                            node.size.max(1)
                        } else {
                            snapshot.nodes[parent_idx as usize].size.max(1)
                        };
                        #[allow(clippy::cast_precision_loss)]
                        let pct = (node.size as f32 / parent_size as f32).clamp(0.0, 1.0);

                        let cell_width = cell_rect.width();
                        #[allow(clippy::cast_precision_loss)]
                        let inset_x = (indent as f32 * 4.0).min(cell_width * 0.3);
                        let mut bar_rect = cell_rect;
                        bar_rect.min.x += inset_x;

                        // Center the progress bar vertically
                        let bar_height = 14.0;
                        let vertical_margin = (row_height - bar_height) / 2.0;
                        bar_rect.min.y += vertical_margin;
                        bar_rect.max.y -= vertical_margin;

                        let colored_width = bar_rect.width() * pct;
                        let mut colored_rect = bar_rect;
                        colored_rect.max.x = colored_rect.min.x + colored_width;

                        // Draw unfilled background portion (no rounding)
                        let bg_color = ui.visuals().widgets.noninteractive.bg_fill;
                        ui.painter().rect_filled(bar_rect, 0.0, bg_color);

                        // Draw filled portion with extension-matching gradient (no rounding)
                        if pct > 0.0 {
                            let ext_color = if node.is_directory() {
                                egui::Color32::from_rgb(110, 120, 135) // Neutral cool slate gray for directories
                            } else {
                                let ext = std::path::Path::new(name)
                                    .extension()
                                    .map_or_else(String::new, |s| {
                                        s.to_string_lossy().to_ascii_lowercase()
                                    });
                                theme::get_color_for_extension(&ext)
                            };
                            let left_color = ext_color;
                            let right_color = ext_color.linear_multiply(0.75);
                            paint_gradient_rect(
                                ui.painter(),
                                colored_rect,
                                left_color,
                                right_color,
                            );
                        }

                        // Draw text centered
                        let text = format!("{:.1}%", pct * 100.0);
                        let text_color = ui.visuals().widgets.active.text_color();
                        ui.painter().text(
                            bar_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::monospace(10.0),
                            text_color,
                        );

                        // Allocate space
                        ui.allocate_rect(cell_rect, egui::Sense::empty());

                        // Interact with the entire cell
                        let cell_resp = ui.interact(
                            ui.max_rect(),
                            ui.id().with(("cell_interact", 1)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Size Column
                    row.col(|ui| {
                        paint_bg(ui, 2);

                        ui.label(
                            prettier_bytes::ByteFormatter::new()
                                .format(node.size)
                                .to_string(),
                        );

                        // Interact with the entire cell
                        let cell_resp = ui.interact(
                            ui.max_rect(),
                            ui.id().with(("cell_interact", 2)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Items Column
                    row.col(|ui| {
                        paint_bg(ui, 3);

                        if node.is_directory() {
                            ui.label(format!("{items_count}"));
                        } else {
                            ui.label("-");
                        }

                        // Interact with the entire cell
                        let cell_resp = ui.interact(
                            ui.max_rect(),
                            ui.id().with(("cell_interact", 3)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Files Column
                    row.col(|ui| {
                        paint_bg(ui, 4);

                        if node.is_directory() {
                            ui.label(format!("{files_count}"));
                        } else {
                            ui.label("-");
                        }

                        // Interact with the entire cell
                        let cell_resp = ui.interact(
                            ui.max_rect(),
                            ui.id().with(("cell_interact", 4)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Subdirs Column
                    row.col(|ui| {
                        paint_bg(ui, 5);

                        if node.is_directory() {
                            ui.label(format!("{subdirs_count}"));
                        } else {
                            ui.label("-");
                        }

                        // Interact with the entire cell
                        let cell_resp = ui.interact(
                            ui.max_rect(),
                            ui.id().with(("cell_interact", 5)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Render Last Modified Column
                    row.col(|col_ui| {
                        paint_bg(col_ui, 6);

                        col_ui.label(crate::model::time_utils::format_epoch(
                            node.modified_timestamp,
                            true,
                        ));

                        // Interact with the entire cell
                        let cell_resp = col_ui.interact(
                            col_ui.max_rect(),
                            col_ui.id().with(("cell_interact", 6)),
                            egui::Sense::click(),
                        );
                        if cell_resp.hovered() {
                            row_hovered_by_mouse = true;
                        }
                        if cell_resp.clicked() {
                            row_clicked = true;
                        }
                        if cell_resp.secondary_clicked() {
                            row_secondary_clicked = true;
                        }
                        cell_resp.context_menu(|ui| {
                            self.draw_file_menu_contents(ui, snapshot);
                        });
                    });

                    // Handle selection/hover updates at the end of the row columns loop
                    if row_clicked {
                        self.handle_node_click(node_idx, modifiers, &visible_nodes);
                    } else if row_secondary_clicked && !self.selected_nodes.contains(&node_idx) {
                        self.selected_nodes.clear();
                        self.selected_nodes.insert(node_idx);
                        self.selected_node_idx = Some(node_idx);
                        self.focus_node_idx = Some(node_idx);
                    }

                    if row_hovered_by_mouse {
                        next_hovered = Some(node_idx);
                    }
                });
            });

        self.hovered_node_idx = next_hovered;
    }

    pub fn render_multi_file_detail_list(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        let count = self.selected_nodes.len();

        // Find selection roots to avoid double-counting nested folders
        let roots =
            crate::stats::treemap::get_selection_roots(&snapshot.nodes, &self.selected_nodes);

        let mut total_size = 0u64;
        for &root_idx in &roots {
            if (root_idx as usize) < snapshot.nodes.len() {
                total_size += snapshot.nodes[root_idx as usize].size;
            }
        }

        let mut files = 0;
        let mut directories = 0;
        let mut stack = Vec::new();
        for &root_idx in &roots {
            stack.push(root_idx);
        }

        while let Some(idx) = stack.pop() {
            if (idx as usize) < snapshot.nodes.len() {
                let node = &snapshot.nodes[idx as usize];
                if node.is_directory() {
                    directories += 1;
                    let mut curr = node.first_child;
                    while curr != crate::arena::NO_INDEX {
                        stack.push(curr);
                        curr = snapshot.nodes[curr as usize].next_sibling;
                    }
                } else {
                    files += 1;
                }
            }
        }

        let total_size_str = prettier_bytes::ByteFormatter::new()
            .format(total_size)
            .to_string();

        ui.vertical(|ui| {
            // Header with Close Button
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ℹ Details")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").on_hover_text("Deselect items").clicked() {
                        self.selected_nodes.clear();
                        self.selected_node_idx = None;
                        self.focus_node_idx = None;
                    }
                });
            });
            ui.separator();

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(format!("{count} Selected Items"))
                        .strong()
                        .size(16.0),
                );
                ui.add_space(8.0);

                ui.label(format!("Total Size: {total_size_str}"));
                ui.label(format!("Files: {files}"));
                ui.label(format!("Directories: {directories}"));
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.strong("Actions");
                ui.add_space(4.0);

                // File operations
                ui.weak("Operations:");
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    let is_scanning = self.shared_state.is_scanning.load(Ordering::SeqCst);
                    let is_dir_selected = directories > 0 && !is_scanning;

                    if ui
                        .add_enabled(is_dir_selected, egui::Button::new("🔄 Refresh Directory"))
                        .on_hover_text("Refresh all selected directory subtrees")
                        .clicked()
                    {
                        let dirs: Vec<u32> = self
                            .selected_nodes
                            .iter()
                            .copied()
                            .filter(|&idx| {
                                (idx as usize) < snapshot.nodes.len()
                                    && snapshot.nodes[idx as usize].is_directory()
                            })
                            .collect();
                        self.refresh_directory_subtrees(&dirs);
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_enabled(!is_scanning, egui::Button::new("♻ Move to Trash"))
                        .clicked()
                    {
                        self.active_modal = Some(ActiveModal::Trash);
                        self.delete_confirm_checked = false;
                        self.delete_node_indices = self.selected_nodes.iter().copied().collect();
                    }
                    ui.add_space(4.0);

                    if ui
                        .add_enabled(!is_scanning, egui::Button::new("🗑 Permanently delete"))
                        .clicked()
                    {
                        self.active_modal = Some(ActiveModal::Delete);
                        self.delete_confirm_checked = false;
                        self.delete_node_indices = self.selected_nodes.iter().copied().collect();
                    }
                });
            });
        });
    }

    pub fn render_file_detail_list(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
    ) {
        if node_idx as usize >= snapshot.nodes.len() {
            return;
        }
        let node = &snapshot.nodes[node_idx as usize];
        let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");
        let is_dir = node.is_directory();
        let is_sym = node.is_symlink();

        ui.vertical(|ui| {
            // Header with Close Button
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ℹ Details")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").on_hover_text("Deselect item").clicked() {
                        self.selected_nodes.clear();
                        self.selected_node_idx = None;
                        self.focus_node_idx = None;
                    }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    // Large Icon and Name
                    ui.horizontal(|ui| {
                        let icon = if is_sym {
                            "🔗"
                        } else if is_dir {
                            "📁"
                        } else {
                            "📄"
                        };
                        ui.label(egui::RichText::new(icon).size(24.0));
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.label(egui::RichText::new(name).strong().size(14.0));
                    });
                    ui.add_space(8.0);

                    let files_count = if is_dir { node.file_count } else { 0 };
                    let subdirs_count = if is_dir {
                        *snapshot.dir_counts.get(node_idx as usize).unwrap_or(&0)
                    } else {
                        0
                    };
                    let items_count = files_count + subdirs_count;

                    // Grid layout for fields to align nicely
                    egui::Grid::new("file_details_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Type field
                            ui.weak("Type:");
                            let type_str = if is_sym {
                                "Symbolic Link"
                            } else if is_dir {
                                "Directory"
                            } else {
                                "File"
                            };
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(type_str);
                                },
                            );
                            ui.end_row();

                            // Size field
                            ui.weak("Size:");
                            let formatted_size = prettier_bytes::ByteFormatter::new()
                                .format(node.size)
                                .to_string();
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(formatted_size);
                                },
                            );
                            ui.end_row();

                            // Bytes field
                            ui.weak("Bytes:");
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(format_with_commas(node.size));
                                },
                            );
                            ui.end_row();

                            // Items
                            ui.weak("Items:");
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if is_dir {
                                        ui.label(format!("{items_count}"));
                                    } else {
                                        ui.label("-");
                                    }
                                },
                            );
                            ui.end_row();

                            // Files
                            ui.weak("Files:");
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if is_dir {
                                        ui.label(format!("{files_count}"));
                                    } else {
                                        ui.label("-");
                                    }
                                },
                            );
                            ui.end_row();

                            // Subdirs
                            ui.weak("Subdirs:");
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if is_dir {
                                        ui.label(format!("{subdirs_count}"));
                                    } else {
                                        ui.label("-");
                                    }
                                },
                            );
                            ui.end_row();

                            // Unix-only details (User, Group, Permissions)
                            #[cfg(unix)]
                            {
                                let full_path = snapshot.get_full_path(node_idx);
                                if let Some((user, group, perm)) = get_unix_metadata(&full_path) {
                                    ui.weak("User:");
                                    ui.allocate_ui(
                                        egui::vec2(
                                            ui.available_width(),
                                            ui.spacing().interact_size.y,
                                        ),
                                        |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(user);
                                        },
                                    );
                                    ui.end_row();

                                    ui.weak("Group:");
                                    ui.allocate_ui(
                                        egui::vec2(
                                            ui.available_width(),
                                            ui.spacing().interact_size.y,
                                        ),
                                        |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(group);
                                        },
                                    );
                                    ui.end_row();

                                    ui.weak("Permissions:");
                                    ui.allocate_ui(
                                        egui::vec2(
                                            ui.available_width(),
                                            ui.spacing().interact_size.y,
                                        ),
                                        |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.label(perm);
                                        },
                                    );
                                    ui.end_row();
                                }
                            }
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Actions
                    ui.strong("Actions");
                    ui.add_space(4.0);

                    // Full Path display and copy
                    ui.weak("Full Path:");
                    let full_path = snapshot.get_full_path(node_idx);
                    ui.horizontal(|ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.label(
                            egui::RichText::new(&full_path)
                                .monospace()
                                .weak()
                                .size(10.0),
                        );
                    });
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy Path").clicked() {
                            ui.ctx().copy_text(full_path.clone());
                        }
                        if ui.button("🗁 Open Manager").clicked() {
                            let path = std::path::Path::new(&full_path);
                            let dir_to_open = if path.is_dir() {
                                path
                            } else {
                                path.parent().map_or(path, |p| p)
                            };
                            let _ = open::that(dir_to_open);
                        }
                    });

                    ui.add_space(12.0);

                    // File operations
                    ui.weak("Operations:");
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        let is_dir_selected = is_dir
                            && !self
                                .shared_state
                                .is_scanning
                                .load(std::sync::atomic::Ordering::SeqCst);
                        if ui
                            .add_enabled(is_dir_selected, egui::Button::new("🔄 Refresh Subtree"))
                            .clicked()
                        {
                            self.refresh_directory_subtree(node_idx);
                        }
                        ui.add_space(4.0);

                        if ui.button("♻ Move to Trash").clicked() {
                            self.active_modal = Some(ActiveModal::Trash);
                            self.delete_confirm_checked = false;
                            self.delete_node_indices = vec![node_idx];
                        }
                        ui.add_space(4.0);

                        if ui.button("🗑 Delete Permanently").clicked() {
                            self.active_modal = Some(ActiveModal::Delete);
                            self.delete_confirm_checked = false;
                            self.delete_node_indices = vec![node_idx];
                        }
                    });
                });
            });
        });
    }
}

#[cfg(unix)]
fn get_unix_metadata(path_str: &str) -> Option<(String, String, String)> {
    use std::fs;
    use std::os::unix::fs::MetadataExt;

    let metadata = fs::symlink_metadata(path_str).ok()?;
    let uid = metadata.uid();
    let gid = metadata.gid();
    let mode = metadata.mode();

    let user = std::process::Command::new("id")
        .args(["-nu", &uid.to_string()])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| uid.to_string());

    let group = std::process::Command::new("getent")
        .args(["group", &gid.to_string()])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout);
                s.split(':').next().map(|name| name.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| gid.to_string());

    let file_type_char = if metadata.is_dir() {
        'd'
    } else if metadata.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };

    let rwx = |val: u32| {
        let r = if val & 4 != 0 { 'r' } else { '-' };
        let w = if val & 2 != 0 { 'w' } else { '-' };
        let x = if val & 1 != 0 { 'x' } else { '-' };
        format!("{r}{w}{x}")
    };

    let u_perm = rwx((mode >> 6) & 7);
    let g_perm = rwx((mode >> 3) & 7);
    let o_perm = rwx(mode & 7);
    let perm_str = format!("{file_type_char}{u_perm}{g_perm}{o_perm}");

    Some((user, group, perm_str))
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn paint_gradient_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    left_color: egui::Color32,
    right_color: egui::Color32,
) {
    let mut mesh = egui::Mesh::default();
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.left_top(),
        uv: egui::epaint::WHITE_UV,
        color: left_color,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.right_top(),
        uv: egui::epaint::WHITE_UV,
        color: right_color,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.right_bottom(),
        uv: egui::epaint::WHITE_UV,
        color: right_color,
    });
    mesh.vertices.push(egui::epaint::Vertex {
        pos: rect.left_bottom(),
        uv: egui::epaint::WHITE_UV,
        color: left_color,
    });
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}
