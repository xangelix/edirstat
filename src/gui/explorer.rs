use eframe::egui;
use smallvec::SmallVec;

use super::{GuiApp, theme};
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
            for idx in (0..snapshot.nodes.len()).rev() {
                let node = &snapshot.nodes[idx];
                let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");

                let self_matches = regex_matcher.as_ref().map_or_else(
                    || {
                        if filter_case_sensitive {
                            name.contains(search_query)
                        } else {
                            name.to_lowercase().contains(&search_query.to_lowercase())
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
    ) {
        let is_duplicate =
            self.highlight_duplicates && self.selected_duplicates.contains(&node_idx);

        let node = &snapshot.nodes[node_idx as usize];
        let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");

        let is_expanded = self.expanded_nodes.contains(&node_idx);
        let has_children = node.is_directory() && node.first_child != NO_INDEX;
        let is_selected = self.selected_node_idx == Some(node_idx);

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
        if response.clicked() || response.secondary_clicked() {
            self.selected_node_idx = Some(node_idx);
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
}
