use std::sync::{Arc, atomic::Ordering};

use eframe::egui;
use egui_table_kit::{
    error::TableError,
    filter::Filter,
    operations::{HeaderIter, RowCallback, RowHierarchy},
    state::TableState,
};
use fixedbitset::FixedBitSet;
use fluent_zero::t;
use smallvec::SmallVec;

use super::{ActiveModal, GuiApp, theme};
use crate::{
    arena::{FileArenaSnapshot, NO_INDEX},
    colors::{AppTheme, get_current_theme},
};

pub struct TableProviderWrapper<'a> {
    snapshot: &'a FileArenaSnapshot,
    time_format: crate::time_utils::TimeFormat,
}

impl<'a> TableProviderWrapper<'a> {
    #[must_use]
    pub const fn new(
        snapshot: &'a FileArenaSnapshot,
        time_format: crate::time_utils::TimeFormat,
    ) -> Self {
        Self {
            snapshot,
            time_format,
        }
    }
}

impl egui_table_kit::operations::TableProvider for TableProviderWrapper<'_> {
    fn column_count(&self) -> usize {
        8
    }

    fn header(&self, index: usize) -> Option<std::borrow::Cow<'_, str>> {
        let key = match index {
            0 => "explorer-hdr-name",
            1 => "explorer-hdr-percentage",
            2 => "explorer-hdr-size",
            3 => "explorer-hdr-items",
            4 => "explorer-hdr-files",
            5 => "explorer-hdr-subdirs",
            6 => "explorer-hdr-created",
            7 => "explorer-hdr-modified",
            _ => return None,
        };
        Some(t!(key))
    }

    fn headers(&self) -> HeaderIter<'_> {
        HeaderIter::new(self)
    }

    fn row_count(&self) -> usize {
        self.snapshot.nodes.len()
    }

    fn cell_at(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Result<Option<egui_table_kit::operations::TableCell<'_>>, TableError> {
        if row_index >= self.snapshot.nodes.len() {
            return Ok(None);
        }
        let node = &self.snapshot.nodes[row_index];
        let name = self.snapshot.string_pool.get(node.name_id).unwrap_or("");

        let val = match col_index {
            0 => {
                if node.parent_opt().is_none() {
                    crate::arena::clean_unc_path(name)
                } else {
                    std::borrow::Cow::Borrowed(name)
                }
            }
            1 => {
                let parent_idx = node.parent;
                let parent_size = if parent_idx == crate::arena::NO_INDEX {
                    node.size.max(1)
                } else {
                    self.snapshot.nodes[parent_idx as usize].size.max(1)
                };
                #[allow(clippy::cast_precision_loss)]
                let pct = (node.size as f32 / parent_size as f32).clamp(0.0, 1.0);
                std::borrow::Cow::Owned(format!("{:.1}%", pct * 100.0))
            }
            2 => std::borrow::Cow::Owned(
                prettier_bytes::ByteFormatter::new()
                    .format(node.size)
                    .to_string(),
            ),
            3 => {
                if node.is_directory() {
                    let files_count = node.file_count;
                    let subdirs_count = *self.snapshot.dir_counts.get(row_index).unwrap_or(&0);
                    std::borrow::Cow::Owned(format!("{}", files_count + subdirs_count))
                } else {
                    std::borrow::Cow::Borrowed("-")
                }
            }
            4 => {
                if node.is_directory() {
                    std::borrow::Cow::Owned(format!("{}", node.file_count))
                } else {
                    std::borrow::Cow::Borrowed("-")
                }
            }
            5 => {
                if node.is_directory() {
                    let subdirs_count = *self.snapshot.dir_counts.get(row_index).unwrap_or(&0);
                    std::borrow::Cow::Owned(format!("{subdirs_count}"))
                } else {
                    std::borrow::Cow::Borrowed("-")
                }
            }
            6 => {
                if node.has_no_permission() {
                    std::borrow::Cow::Owned(t!("no-permission").into_owned())
                } else {
                    std::borrow::Cow::Owned(crate::time_utils::format_epoch(
                        node.created_timestamp,
                        &self.time_format,
                    ))
                }
            }
            7 => {
                if node.has_no_permission() {
                    std::borrow::Cow::Owned(t!("no-permission").into_owned())
                } else {
                    std::borrow::Cow::Owned(crate::time_utils::format_epoch(
                        node.modified_timestamp,
                        &self.time_format,
                    ))
                }
            }
            _ => return Ok(None),
        };

        Ok(Some((val, None)))
    }

    fn row_at(
        &self,
        index: usize,
    ) -> Result<Option<egui_table_kit::operations::OwnedRow>, TableError> {
        if index >= self.snapshot.nodes.len() {
            return Ok(None);
        }
        let mut cells = Vec::with_capacity(8);
        for col_idx in 0..8 {
            if let Some((val, hover)) = self.cell_at(index, col_idx)? {
                cells.push((
                    compact_str::CompactString::from(val.as_ref()),
                    hover.map(|h| compact_str::CompactString::from(h.as_ref())),
                ));
            } else {
                cells.push((compact_str::CompactString::default(), None));
            }
        }
        Ok(Some(egui_table_kit::operations::OwnedRow { cells }))
    }

    fn for_selected_rows(
        &self,
        state: &TableState,
        f: &mut RowCallback<'_>,
    ) -> Result<(), TableError> {
        for row_idx in &state.selected_rows {
            if (row_idx as usize) < self.snapshot.nodes.len() {
                let row = egui_table_kit::operations::BorrowedRow {
                    provider: self,
                    row_index: row_idx as usize,
                };
                f(&row)?;
            }
        }
        Ok(())
    }

    fn for_all_rows(&self, f: &mut RowCallback<'_>) -> Result<(), TableError> {
        for row_idx in 0..self.snapshot.nodes.len() {
            let row = egui_table_kit::operations::BorrowedRow {
                provider: self,
                row_index: row_idx,
            };
            f(&row)?;
        }
        Ok(())
    }

    fn row_hierarchy(&self, state: &TableState, row_index: usize) -> Option<RowHierarchy> {
        let node = &self.snapshot.nodes[row_index];
        let has_children = node.is_directory() && node.first_child != NO_INDEX;
        let is_expanded = state.expanded_rows.contains(row_index as u32);

        let mut indent_level = 0;
        let mut curr = node.parent;
        while curr != NO_INDEX {
            indent_level += 1;
            curr = self.snapshot.nodes[curr as usize].parent;
        }

        Some(RowHierarchy {
            indent_level,
            has_children,
            is_expanded,
        })
    }

    fn is_tree(&self) -> bool {
        true
    }

    // Resolve parent global index
    fn row_parent(&self, row_index: usize) -> Option<usize> {
        let parent = self.snapshot.nodes[row_index].parent;
        if parent == NO_INDEX {
            None
        } else {
            Some(parent as usize)
        }
    }

    // Resolve immediate directory child indices
    fn row_children(&self, row_index: usize) -> Vec<usize> {
        let node = &self.snapshot.nodes[row_index];
        let mut children = Vec::new();
        let mut curr = node.first_child;
        while curr != NO_INDEX {
            children.push(curr as usize);
            curr = self.snapshot.nodes[curr as usize].next_sibling;
        }

        // Sort initial list by size descending (matching classic Windirstat visual default)
        children.sort_by(|&a, &b| {
            self.snapshot.nodes[b]
                .size
                .cmp(&self.snapshot.nodes[a].size)
        });

        children
    }

    // Match string names against active filters
    fn row_matches(
        &self,
        _state: &TableState,
        row_index: usize,
        filters: &[(usize, Filter)],
        highlight: Option<u8>,
    ) -> bool {
        let node = &self.snapshot.nodes[row_index];
        let name = self.snapshot.string_pool.get(node.name_id).unwrap_or("");

        for &(col_idx, ref filter) in filters {
            let cell_text = match col_idx {
                0 => {
                    let cleaned_name = if node.parent_opt().is_none() {
                        crate::arena::clean_unc_path(name)
                    } else {
                        std::borrow::Cow::Borrowed(name)
                    };
                    cleaned_name.into_owned()
                }
                1 => {
                    // Percentage
                    let parent_idx = node.parent;
                    let parent_size = if parent_idx == crate::arena::NO_INDEX {
                        node.size.max(1)
                    } else {
                        self.snapshot.nodes[parent_idx as usize].size.max(1)
                    };
                    #[allow(clippy::cast_precision_loss)]
                    let pct = (node.size as f32 / parent_size as f32).clamp(0.0, 1.0);
                    format!("{:.1}%", pct * 100.0)
                }
                2 => {
                    // Size (e.g. "1.2 GB" or "4.5 MB")
                    prettier_bytes::ByteFormatter::new()
                        .format(node.size)
                        .to_string()
                }
                3 => {
                    // Items count
                    if node.is_directory() {
                        let files_count = node.file_count;
                        let subdirs_count = *self.snapshot.dir_counts.get(row_index).unwrap_or(&0);
                        format!("{}", files_count + subdirs_count)
                    } else {
                        "-".to_string()
                    }
                }
                4 => {
                    // Files count
                    if node.is_directory() {
                        format!("{}", node.file_count)
                    } else {
                        "-".to_string()
                    }
                }
                5 => {
                    // Subdirectories count
                    if node.is_directory() {
                        let subdirs_count = *self.snapshot.dir_counts.get(row_index).unwrap_or(&0);
                        format!("{subdirs_count}")
                    } else {
                        "-".to_string()
                    }
                }
                6 => {
                    // Created date string
                    if node.has_no_permission() {
                        t!("no-permission").into_owned()
                    } else {
                        crate::time_utils::format_epoch(node.created_timestamp, &self.time_format)
                    }
                }
                7 => {
                    // Last Modified date string
                    if node.has_no_permission() {
                        t!("no-permission").into_owned()
                    } else {
                        crate::time_utils::format_epoch(node.modified_timestamp, &self.time_format)
                    }
                }
                _ => String::new(),
            };

            if !filter.matches(&cell_text, highlight) {
                return false;
            }
        }
        true
    }

    fn sort_active_rows(
        &self,
        active_rows: &mut Vec<usize>,
        col_index: usize,
        ascending: bool,
    ) -> Result<(), TableError> {
        active_rows.sort_by(|&a, &b| {
            let cmp = compare_nodes_by_column(self.snapshot, col_index, a, b);
            if ascending { cmp } else { cmp.reverse() }
        });

        Ok(())
    }
}

pub struct QueryCoordinator {
    pub cached_node_matches: FixedBitSet,
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
            cached_node_matches: FixedBitSet::new(),
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

        // Reuse the existing allocation when the node count is unchanged (e.g. only the
        // search query changed); otherwise rebuild to the exact new length.
        if self.cached_node_matches.len() == snapshot.nodes.len() {
            self.cached_node_matches.clear();
        } else {
            self.cached_node_matches = FixedBitSet::with_capacity(snapshot.nodes.len());
        }

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
                let cleaned_name = if node.parent_opt().is_none() {
                    crate::arena::clean_unc_path(name)
                } else {
                    std::borrow::Cow::Borrowed(name)
                };

                let self_matches = regex_matcher.as_ref().map_or_else(
                    || {
                        if filter_case_sensitive {
                            cleaned_name.contains(search_query)
                        } else {
                            crate::arena::contains_case_insensitive(
                                &cleaned_name,
                                &search_query_lower,
                            )
                        }
                    },
                    |re| re.is_match(&cleaned_name),
                );

                if self_matches {
                    self.cached_node_matches.insert(idx);
                }

                // If this node matches, flag its parent recursively up to the root
                if self.cached_node_matches.contains(idx)
                    && let Some(parent) = node.parent_opt()
                    && (parent as usize) < self.cached_node_matches.len()
                {
                    self.cached_node_matches.insert(parent as usize);
                }
            }
        }

        if highlight_duplicates {
            for &idx in selected_duplicates {
                if (idx as usize) < self.cached_node_matches.len() {
                    self.cached_node_matches.insert(idx as usize);

                    // Propagate match to parents to keep them visible in the tree
                    let mut curr = Some(idx);
                    while let Some(c_idx) = curr {
                        let node = &snapshot.nodes[c_idx as usize];
                        if let Some(parent) = node.parent_opt() {
                            self.cached_node_matches.insert(parent as usize);
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
        let selected_rows = &mut self.table_state.selected_rows;

        if modifiers.command || modifiers.ctrl {
            // Toggle selection
            if selected_rows.contains(clicked_node_idx) {
                selected_rows.remove(clicked_node_idx);
            } else {
                selected_rows.insert(clicked_node_idx);
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
                        selected_rows.insert(idx);
                    }
                }
            }
        } else if selected_rows.len() == 1 && selected_rows.contains(clicked_node_idx) {
            // Normal click: if already selected singly, deselect it. Otherwise, select only this node.
            selected_rows.clear();
            self.focus_node_idx = None;
        } else {
            selected_rows.clear();
            selected_rows.insert(clicked_node_idx);
            self.focus_node_idx = Some(clicked_node_idx);
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
        node_matches: &FixedBitSet,
    ) {
        let node = &snapshot.nodes[node_idx as usize];

        // Filter search query: O(1) matching subtree lookup
        if !self.search_query.is_empty()
            && (node_idx as usize) < node_matches.len()
            && !node_matches.contains(node_idx as usize)
        {
            return;
        }

        out.push((node_idx, indent_level));

        let is_expanded = self.table_state.expanded_rows.contains(node_idx);
        let has_children = node.is_directory() && node.first_child != NO_INDEX;

        if is_expanded && has_children {
            let mut sorted_child_indices = SmallVec::<[u32; 16]>::new();
            let mut curr = node.first_child;
            while curr != NO_INDEX {
                sorted_child_indices.push(curr);
                curr = snapshot.nodes[curr as usize].next_sibling;
            }

            // Sync sort column with active traversal structure
            let sort_state = self.table_state.get_sort_state();
            if let Some((sort_col, sort_up)) = sort_state {
                sorted_child_indices.sort_by(|&a, &b| {
                    let cmp = compare_nodes_by_column(snapshot, sort_col, a as usize, b as usize);
                    if sort_up { cmp } else { cmp.reverse() }
                });
            } else {
                // Default: Sort immediate children by size descending dynamically for correct tree views
                sorted_child_indices.sort_by(|&a, &b| {
                    snapshot.nodes[b as usize]
                        .size
                        .cmp(&snapshot.nodes[a as usize].size)
                });
            }

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

        let is_expanded = self.table_state.expanded_rows.contains(node_idx);
        let has_children = node.is_directory() && node.first_child != NO_INDEX;
        let is_selected = self.table_state.selected_rows.contains(node_idx);

        let horizontal_res = ui.horizontal(|ui| {
            // Indent padding
            #[allow(clippy::cast_precision_loss)]
            ui.add_space(indent_level as f32 * 22.0);

            // Icon & Expand Arrow
            let icon_text = if node.is_symlink() {
                "🔗"
            } else if node.is_directory() {
                "📁"
            } else {
                "📄"
            };

            ui.scope(|ui| {
                ui.spacing_mut().interact_size.x = 0.0;
                ui.spacing_mut().button_padding = egui::vec2(2.0, 2.0);
                ui.spacing_mut().item_spacing.x = 4.0;

                if has_children {
                    let arrow = if is_expanded { "⏷" } else { "⏵" };

                    // Allocate an exact interactive rectangle
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::click());

                    // Resolve responsive colors based on interaction states
                    let arrow_color = if response.hovered() {
                        if is_expanded {
                            ui.visuals().warn_fg_color.linear_multiply(0.9)
                        } else {
                            ui.visuals().widgets.hovered.text_color()
                        }
                    } else if is_expanded {
                        ui.visuals().widgets.active.text_color()
                    } else {
                        ui.visuals()
                            .widgets
                            .inactive
                            .text_color()
                            .linear_multiply(0.5)
                    };

                    // Draw the glyph centered inside the allocated rectangle
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        arrow,
                        egui::FontId::proportional(11.0),
                        arrow_color,
                    );

                    if response.clicked() {
                        if is_expanded {
                            self.table_state.expanded_rows.remove(node_idx);
                        } else {
                            self.table_state.expanded_rows.insert(node_idx);
                        }
                    }
                } else {
                    let dummy_arrow = egui::RichText::new("⏵").color(egui::Color32::TRANSPARENT);
                    ui.add_enabled_ui(false, |ui| {
                        let _ = ui.selectable_label(false, dummy_arrow);
                    });
                }
            });

            ui.label(icon_text);

            // Node Name / Label with automatic left-aligned truncation
            let cleaned_name = if node.parent_opt().is_none() {
                crate::arena::clean_unc_path(name)
            } else {
                std::borrow::Cow::Borrowed(name)
            };
            let mut rich_name = egui::RichText::new(&*cleaned_name);
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
        let expand_button_width = (indent_level as f32).mul_add(22.0, 26.0);
        interactive_rect.min.x += expand_button_width;

        let row_id = ui.id().with(("tree_row", node_idx));
        let response = ui.interact(interactive_rect, row_id, egui::Sense::click());

        if is_selected {
            let fill_color = match get_current_theme() {
                AppTheme::HighContrast => egui::Color32::from_rgb(80, 80, 0),
                AppTheme::Light => egui::Color32::from_rgb(204, 229, 255),
                AppTheme::Dark => ui.visuals().selection.bg_fill.linear_multiply(0.12),
            };
            ui.painter().rect_filled(rect, 4.0, fill_color);
        } else if response.hovered() {
            let hover_color = match get_current_theme() {
                AppTheme::HighContrast => egui::Color32::from_rgb(65, 65, 65),
                AppTheme::Light => egui::Color32::from_rgb(225, 238, 254),
                AppTheme::Dark => ui.visuals().widgets.hovered.bg_fill.linear_multiply(0.04),
            };
            ui.painter().rect_filled(rect, 4.0, hover_color);
        }

        // Handle selection on Left-Click or Right-Click (only outside of the expand button)
        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);
            self.table_state
                .handle_row_selection(modifiers, node_idx as usize);
        } else if response.secondary_clicked() && !self.table_state.selected_rows.contains(node_idx)
        {
            self.table_state.selected_rows.clear();
            self.table_state.selected_rows.insert(node_idx);
            self.focus_node_idx = Some(node_idx);
        }

        // Render the context menu on Right-Click
        response.context_menu(|ui| {
            self.draw_file_menu_contents(ui, snapshot);
        });

        // Draw vertical indentation guidelines to visually track nested guidelines
        let painter = ui.painter();
        let stroke = egui::Stroke::new(1.0f32, theme::get_indent_guideline());
        for i in 0..indent_level {
            #[allow(clippy::cast_precision_loss)]
            let x = (i as f32).mul_add(22.0, rect.min.x) + 11.0;

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
                ui.label(t!("explorer-empty-state"));
            });
            return;
        }

        // Automatically expand the root node by default if state is empty
        if self.table_state.expanded_rows.is_empty() && !snapshot.nodes.is_empty() {
            self.table_state.expanded_rows.insert(0);
        }

        // Sync global self.search_query to TableState's Column 0 (Name) filter search text
        if self.table_state.columns.len() < 8 {
            self.table_state
                .columns
                .resize_with(8, egui_table_kit::header::ColumnState::default);
        }

        let name_col = &mut self.table_state.columns[0];
        if name_col.response.filtering.search.text() != self.search_query {
            name_col
                .response
                .filtering
                .search
                .set_text(self.search_query.clone());
            if self.search_query.is_empty() {
                name_col.response.filtering.search.clear();
            } else {
                name_col.response.filtering.search.open();
            }
            self.table_state.filter_cache_dirty = true; // Mark cache dirty to trigger a rebuild
        }

        let row_height = 28.0;

        let provider = TableProviderWrapper::new(snapshot, self.time_format.clone());

        self.table_state.flatten_tree(&provider);

        let columns = vec![
            egui_table_kit::layout::Column::new(320.0).resizable(true),
            egui_table_kit::layout::Column::new(140.0)
                .range(80.0..=400.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(90.0)
                .range(60.0..=200.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(70.0)
                .range(40.0..=150.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(70.0)
                .range(40.0..=150.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(70.0)
                .range(40.0..=150.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(150.0)
                .range(100.0..=300.0)
                .resizable(true),
            egui_table_kit::layout::Column::new(150.0)
                .range(100.0..=300.0)
                .resizable(true),
        ];

        let mut table = egui_table_kit::layout::Table::new()
            .id_salt("hierarchical_file_table")
            .num_rows(self.table_state.active_rows.len() as u64)
            .columns(columns)
            .headers([egui_table_kit::layout::HeaderRow::new(row_height)]); // Aligned header row to 28.0

        if self.scroll_to_selected {
            if let Some(selected_idx) = self.selected_node_idx()
                && let Some(row_index) = self
                    .table_state
                    .active_rows
                    .iter()
                    .position(|&node_idx| node_idx == selected_idx as usize)
            {
                table = table.scroll_to_row(row_index as u64, Some(egui::Align::Center));
            }
            self.scroll_to_selected = false;
        }

        let snapshot_nodes = Arc::clone(&snapshot.nodes);
        let snapshot_string_pool = Arc::clone(&snapshot.string_pool);
        let active_rows = self.table_state.active_rows.clone();
        let selected_rows = self.table_state.selected_rows.clone();
        let highlight_duplicates = self.highlight_duplicates;
        let selected_duplicates = self.selected_duplicates.clone();
        let monospace_paths = self.monospace_paths;
        let current_theme = get_current_theme();

        let active_rows_count = active_rows.len();

        let indent_levels: Vec<usize> = active_rows
            .iter()
            .map(|&row_idx| {
                let mut indent_level = 0;
                let mut curr = snapshot.nodes[row_idx].parent;
                while curr != crate::arena::NO_INDEX {
                    indent_level += 1;
                    curr = snapshot.nodes[curr as usize].parent;
                }
                indent_level
            })
            .collect();

        let custom_cell_ui = Box::new(
            move |ui: &mut egui::Ui,
                  cell_info: &egui_table_kit::layout::CellInfo,
                  row_data: &dyn egui_table_kit::operations::Row,
                  text_color: egui::Color32| {
                if cell_info.col_nr == 0 {
                    let row_idx = active_rows[cell_info.row_nr as usize];
                    let node = &snapshot_nodes[row_idx];
                    let name = snapshot_string_pool.get(node.name_id).unwrap_or("unknown");

                    let is_duplicate =
                        highlight_duplicates && selected_duplicates.contains(&(row_idx as u32));
                    let is_selected = selected_rows.contains(row_idx as u32);

                    let icon_text = if node.is_symlink() {
                        "🔗"
                    } else if node.is_directory() {
                        "📁"
                    } else {
                        "📄"
                    };

                    let cleaned_name = if node.parent_opt().is_none() {
                        crate::arena::clean_unc_path(name)
                    } else {
                        std::borrow::Cow::Borrowed(name)
                    };
                    let mut rich_name = egui::RichText::new(&*cleaned_name);
                    if monospace_paths {
                        rich_name = rich_name.monospace();
                    }
                    if is_selected {
                        rich_name = rich_name.strong().color(text_color);
                    } else if is_duplicate {
                        rich_name = rich_name.color(theme::GLOW_INNER_CORE);
                    }

                    let response = ui
                        .horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.label(icon_text);
                            let name_width = ui.available_width().max(50.0);
                            ui.allocate_ui(
                                egui::vec2(name_width, ui.spacing().interact_size.y),
                                |ui| {
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                    ui.label(rich_name);
                                },
                            );
                        })
                        .response;

                    return Some(response);
                }

                if cell_info.col_nr == 1 {
                    let row_idx = active_rows[cell_info.row_nr as usize];
                    let node = &snapshot_nodes[row_idx];

                    let parent_idx = node.parent;
                    let parent_size = if parent_idx == crate::arena::NO_INDEX {
                        node.size.max(1)
                    } else {
                        snapshot_nodes[parent_idx as usize].size.max(1)
                    };
                    #[allow(clippy::cast_precision_loss)]
                    let pct = (node.size as f32 / parent_size as f32).clamp(0.0, 1.0);

                    // Get the actual full cell rect directly from the cell's Ui
                    let cell_rect = ui.max_rect();

                    let cell_width = cell_rect.width();
                    let indent = indent_levels[cell_info.row_nr as usize];
                    #[allow(clippy::cast_precision_loss)]
                    let inset_x = (indent as f32 * 4.0).min(cell_width * 0.3);
                    let mut bar_rect = cell_rect;
                    bar_rect.min.x += inset_x;

                    // Symmetric vertical centering math
                    let bar_height = 14.0;
                    let vertical_margin = (row_height - bar_height) / 2.0;
                    bar_rect.min.y += vertical_margin;
                    bar_rect.max.y = bar_rect.min.y + bar_height;

                    let colored_width = bar_rect.width() * pct;
                    let mut colored_rect = bar_rect;
                    colored_rect.max.x = colored_rect.min.x + colored_width;

                    let bg_color = match current_theme {
                        AppTheme::HighContrast => egui::Color32::from_rgb(45, 45, 45),
                        _ => ui.visuals().widgets.noninteractive.bg_fill,
                    };
                    if current_theme == AppTheme::HighContrast {
                        ui.painter().rect(
                            bar_rect,
                            0.0,
                            bg_color,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(235, 235, 235)),
                            egui::StrokeKind::Outside,
                        );
                    } else {
                        ui.painter().rect_filled(bar_rect, 0.0, bg_color);
                    }

                    if pct > 0.0 {
                        let ext_color = if node.is_directory() {
                            egui::Color32::from_rgb(110, 120, 135)
                        } else {
                            let name = snapshot_string_pool.get(node.name_id).unwrap_or("unknown");
                            let ext = std::path::Path::new(name)
                                .extension()
                                .map_or_else(String::new, |s| {
                                    s.to_string_lossy().to_ascii_lowercase()
                                });
                            theme::get_color_for_extension(&ext)
                        };
                        paint_gradient_rect(
                            ui.painter(),
                            colored_rect,
                            ext_color,
                            ext_color.linear_multiply(0.75),
                        );
                    }

                    let text = format!("{:.1}%", pct * 100.0);
                    ui.painter().text(
                        bar_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::monospace(10.0),
                        ui.visuals().widgets.active.text_color(),
                    );

                    // Allocate the rect to let egui know we used the area and support hover tooltips
                    let response = ui.allocate_rect(cell_rect, egui::Sense::hover());

                    return Some(response);
                }

                if cell_info.col_nr >= 2
                    && let Some((val, _)) = row_data.cell(cell_info.col_nr)
                {
                    let response = ui
                        .horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(val.as_ref()).color(text_color),
                                )
                                .selectable(false)
                                .wrap_mode(egui::TextWrapMode::Truncate),
                            );
                        })
                        .response;

                    return Some(response);
                }

                None
            },
        );

        let org_colors = &[];
        let user_colors = &mut [];
        let mut collected_responses = Vec::new();
        let mut halt_error = None;
        let mut item_clicked = None;
        let mut secondary_clicked = None;

        let mut delegate = egui_table_kit::delegate::TableKitDelegate::new(
            &provider,
            &mut self.table_state,
            org_colors,
            user_colors,
            &mut collected_responses,
            &mut halt_error,
            Some(custom_cell_ui),
            &mut item_clicked,
            &mut secondary_clicked,
        );

        delegate.header_menu_anchor = egui_table_kit::header::HeaderMenuAnchor::Cursor;
        delegate.striped = true;
        delegate.striping_color = Some(match current_theme {
            AppTheme::HighContrast => egui::Color32::from_rgb(30, 30, 30),
            AppTheme::Light => egui::Color32::from_rgb(235, 235, 240),
            AppTheme::Dark => egui::Color32::from_rgb(32, 36, 48),
        });

        let bright_gray = egui::Color32::from_rgb(180, 180, 180);
        ui.visuals_mut().widgets.noninteractive.bg_stroke.color = bright_gray;

        delegate.row_height = row_height;

        #[allow(clippy::pedantic)]
        let total_height = (active_rows_count + 1) as f32 * delegate.row_height;
        let table_height = total_height.min(ui.available_height());

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), table_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                table.show(ui, &mut delegate);
            },
        );

        drop(delegate);

        if let Some(row_idx) = item_clicked {
            self.focus_node_idx = Some(row_idx as u32);
        }

        let mut show_menu = false;
        let mut menu_pos = egui::Pos2::ZERO;
        if let Some(row_idx) = secondary_clicked {
            if !self.table_state.selected_rows.contains(row_idx as u32) {
                self.table_state.selected_rows.clear();
                self.table_state.selected_rows.insert(row_idx as u32);
                self.focus_node_idx = Some(row_idx as u32);
            }
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                menu_pos = pos;
                show_menu = true;
            }
        }

        let popup_id = ui.make_persistent_id("row_context_menu");
        if show_menu {
            ui.data_mut(|d| d.insert_temp(popup_id.with("pos"), menu_pos));
            egui::Popup::toggle_id(ui.ctx(), popup_id);
        }

        let saved_pos = ui.data(|d| d.get_temp::<egui::Pos2>(popup_id.with("pos")));
        if let Some(pos) = saved_pos {
            let anchor_rect = egui::Rect::from_center_size(pos, egui::Vec2::splat(1.0));
            let dummy_response = ui.interact(
                anchor_rect,
                popup_id.with("dummy_anchor"),
                egui::Sense::hover(),
            );

            egui::Popup::menu(&dummy_response)
                .id(popup_id)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    self.draw_file_menu_contents(ui, snapshot);
                });
        }

        let popup_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        if !popup_open {
            ui.data_mut(|d| {
                d.remove::<egui::Pos2>(popup_id.with("pos"));
            });
        }

        let _ = self
            .table_state
            .process_responses(&provider, collected_responses);

        let name_filter_text = self.table_state.columns[0].response.filtering.search.text();
        if name_filter_text != self.search_query {
            self.search_query = name_filter_text.to_string();
        }
    }

    pub fn render_multi_file_detail_list(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        let count = self.table_state.selected_rows.len();

        let selected_set: std::collections::HashSet<u32> =
            self.table_state.selected_rows.iter().collect();
        let roots = crate::stats::treemap::get_selection_roots(&snapshot.nodes, &selected_set);

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
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(t!("explorer-details-header"))
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let deselect_btn = ui
                        .scope(|ui| {
                            ui.style_mut().visuals.button_frame = false;
                            ui.style_mut().visuals.widgets.hovered.fg_stroke =
                                egui::Stroke::new(1.0f32, egui::Color32::from_rgb(239, 68, 68));
                            ui.button("❌")
                        })
                        .inner
                        .on_hover_text(t!("explorer-deselect-hover"));
                    if deselect_btn.clicked() {
                        self.table_state.selected_rows.clear();
                        self.focus_node_idx = None;
                    }
                });
            });
            ui.separator();

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(t!("explorer-selected-items-count", {
                        "count" => count
                    }))
                    .strong()
                    .size(16.0),
                );
                ui.add_space(8.0);

                ui.label(t!("explorer-total-size", {
                    "size" => total_size_str.as_str()
                }));
                ui.label(t!("explorer-files", {
                    "count" => files
                }));
                ui.label(t!("explorer-directories", {
                    "count" => directories
                }));
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.strong(t!("explorer-actions-title"));
                ui.add_space(4.0);

                // Filesystem-mutating operations are native-only
                if crate::IS_NATIVE || !crate::HIDE_NA_UI {
                    let res = ui
                        .add_enabled_ui(crate::IS_NATIVE, |ui| {
                            ui.weak(t!("explorer-actions-operations"));
                            ui.add_space(4.0);
                            ui.vertical(|ui| {
                                let is_scanning =
                                    self.shared_state.is_scanning.load(Ordering::SeqCst);
                                let is_dir_selected = directories > 0 && !is_scanning;

                                let refresh_btn = draw_action_button(
                                    ui,
                                    &t!("explorer-action-refresh-directory"),
                                    egui::Color32::from_rgb(34, 197, 94), // Green
                                    is_dir_selected,
                                )
                                .on_hover_text(t!("explorer-action-refresh-hover"));
                                if refresh_btn.clicked() {
                                    let dirs: Vec<u32> = self
                                        .table_state
                                        .selected_rows
                                        .iter()
                                        .filter(|&idx| {
                                            (idx as usize) < snapshot.nodes.len()
                                                && snapshot.nodes[idx as usize].is_directory()
                                        })
                                        .collect();
                                    self.refresh_directory_subtrees(&dirs);
                                }
                                ui.add_space(4.0);

                                let trash_btn = draw_action_button(
                                    ui,
                                    "♻ Move to Trash",
                                    egui::Color32::from_rgb(234, 179, 8), // Yellow/Orange
                                    !is_scanning,
                                );
                                if trash_btn.clicked() {
                                    self.delete_node_indices =
                                        self.table_state.selected_rows.iter().collect();
                                    if self.trash_confirmation {
                                        self.active_modal = Some(ActiveModal::Trash);
                                        self.delete_confirm_checked = false;
                                        self.remember_confirmation = false;
                                    } else {
                                        self.execute_deletion(
                                            &self.delete_node_indices.clone(),
                                            true,
                                            ui.ctx(),
                                        );
                                        self.delete_node_indices.clear();
                                    }
                                }
                                ui.add_space(4.0);

                                let delete_btn = draw_action_button(
                                    ui,
                                    "🗑 Permanently delete",
                                    egui::Color32::from_rgb(239, 68, 68), // Red
                                    !is_scanning,
                                );
                                if delete_btn.clicked() {
                                    self.delete_node_indices =
                                        self.table_state.selected_rows.iter().collect();
                                    if self.deletion_confirmation {
                                        self.active_modal = Some(ActiveModal::Delete);
                                        self.delete_confirm_checked = false;
                                        self.remember_confirmation = false;
                                    } else {
                                        self.execute_deletion(
                                            &self.delete_node_indices.clone(),
                                            false,
                                            ui.ctx(),
                                        );
                                        self.delete_node_indices.clear();
                                    }
                                }
                            });
                        })
                        .response;
                    if !crate::IS_NATIVE {
                        res.on_disabled_hover_text(t!("web-not-available"));
                    }
                }
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

        let cleaned_name = if node.parent_opt().is_none() {
            crate::arena::clean_unc_path(name)
        } else {
            std::borrow::Cow::Borrowed(name)
        };

        ui.vertical(|ui| {
            // Header with Close Button
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(t!("explorer-details-header"))
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let deselect_btn = ui
                        .scope(|ui| {
                            ui.style_mut().visuals.button_frame = false;
                            ui.style_mut().visuals.widgets.hovered.fg_stroke =
                                egui::Stroke::new(1.0f32, egui::Color32::from_rgb(239, 68, 68));
                            ui.button("❌")
                        })
                        .inner
                        .on_hover_text(t!("explorer-deselect-single-hover"));
                    if deselect_btn.clicked() {
                        self.table_state.selected_rows.clear();
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
                        ui.label(egui::RichText::new(&*cleaned_name).strong().size(14.0));
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
                            ui.weak(t!("explorer-grid-type"));
                            let type_str = if is_sym {
                                t!("type-symlink")
                            } else if is_dir {
                                t!("type-directory")
                            } else {
                                t!("type-file")
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
                            ui.weak(t!("explorer-grid-size"));
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
                            ui.weak(t!("explorer-grid-bytes"));
                            ui.allocate_ui(
                                egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(format_with_commas(node.size));
                                },
                            );
                            ui.end_row();

                            // Items
                            ui.weak(t!("explorer-grid-items"));
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
                            ui.weak(t!("explorer-grid-files"));
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
                            ui.weak(t!("explorer-grid-subdirs"));
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
                                // Retrieve or populate cache
                                let needs_update = self
                                    .unix_metadata_cache
                                    .as_ref()
                                    .is_none_or(|(cached_idx, _, _, _)| *cached_idx != node_idx);

                                if needs_update {
                                    let full_path = snapshot.get_full_path(node_idx);
                                    if let Some((user, group, perm)) = get_unix_metadata(&full_path)
                                    {
                                        self.unix_metadata_cache =
                                            Some((node_idx, user, group, perm));
                                    } else {
                                        self.unix_metadata_cache = None;
                                    }
                                }

                                if let Some((_, user, group, perm)) = &self.unix_metadata_cache {
                                    ui.weak(t!("explorer-grid-user"));
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

                                    ui.weak(t!("explorer-grid-group"));
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

                                    ui.weak(t!("explorer-grid-permissions"));
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
                    ui.strong(t!("explorer-actions-title"));
                    ui.add_space(4.0);

                    // Full Path display and copy
                    ui.weak(t!("explorer-grid-path"));
                    let full_path = snapshot.get_full_path(node_idx);
                    let cleaned_path = crate::arena::clean_unc_path(&full_path);
                    ui.horizontal(|ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.label(
                            egui::RichText::new(&*cleaned_path)
                                .monospace()
                                .weak()
                                .size(10.0),
                        );
                    });
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let copy_btn = draw_action_button(
                            ui,
                            &t!("explorer-action-copy-path"),
                            egui::Color32::from_rgb(139, 92, 246), // Violet/Purple
                            true,
                        );
                        if copy_btn.clicked() {
                            ui.ctx().copy_text(cleaned_path.into_owned());
                        }

                        // Opening the system file manager is native-only
                        if crate::IS_NATIVE || !crate::HIDE_NA_UI {
                            let mut open_btn = ui
                                .add_enabled_ui(crate::IS_NATIVE, |ui| {
                                    draw_action_button(
                                        ui,
                                        &t!("explorer-action-open-manager"),
                                        egui::Color32::from_rgb(245, 158, 11), // Amber/Orange
                                        true,
                                    )
                                })
                                .inner;
                            if !crate::IS_NATIVE {
                                open_btn = open_btn.on_disabled_hover_text(t!("web-not-available"));
                            }
                            if open_btn.clicked() {
                                #[cfg(not(target_family = "wasm"))]
                                {
                                    let path = std::path::Path::new(&full_path);
                                    let dir_to_open = if path.is_dir() {
                                        path
                                    } else {
                                        path.parent().map_or(path, |p| p)
                                    };
                                    let _ = open::that(dir_to_open);
                                }
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // File operations (native-only: need a live filesystem)
                    if crate::IS_NATIVE || !crate::HIDE_NA_UI {
                        let res = ui
                            .add_enabled_ui(crate::IS_NATIVE, |ui| {
                                ui.weak(t!("explorer-actions-operations"));
                                ui.add_space(4.0);
                                ui.vertical(|ui| {
                                    let is_dir_selected = is_dir
                                        && !self
                                            .shared_state
                                            .is_scanning
                                            .load(std::sync::atomic::Ordering::SeqCst);
                                    let refresh_btn = draw_action_button(
                                        ui,
                                        &t!("explorer-action-refresh-subtree"),
                                        egui::Color32::from_rgb(34, 197, 94), // Green
                                        is_dir_selected,
                                    );
                                    if refresh_btn.clicked() {
                                        self.refresh_directory_subtree(node_idx);
                                    }
                                    ui.add_space(4.0);

                                    let trash_btn = draw_action_button(
                                        ui,
                                        &t!("explorer-action-move-trash"),
                                        egui::Color32::from_rgb(234, 179, 8), // Yellow/Orange
                                        true,
                                    );
                                    if trash_btn.clicked() {
                                        self.delete_node_indices = vec![node_idx];
                                        if self.trash_confirmation {
                                            self.active_modal = Some(ActiveModal::Trash);
                                            self.delete_confirm_checked = false;
                                            self.remember_confirmation = false;
                                        } else {
                                            self.execute_deletion(
                                                &self.delete_node_indices.clone(),
                                                true,
                                                ui.ctx(),
                                            );
                                            self.delete_node_indices.clear();
                                        }
                                    }
                                    ui.add_space(4.0);

                                    let delete_btn = draw_action_button(
                                        ui,
                                        &t!("explorer-action-delete-permanently"),
                                        egui::Color32::from_rgb(239, 68, 68), // Red
                                        true,
                                    );
                                    if delete_btn.clicked() {
                                        self.delete_node_indices = vec![node_idx];
                                        if self.deletion_confirmation {
                                            self.active_modal = Some(ActiveModal::Delete);
                                            self.delete_confirm_checked = false;
                                            self.remember_confirmation = false;
                                        } else {
                                            self.execute_deletion(
                                                &self.delete_node_indices.clone(),
                                                false,
                                                ui.ctx(),
                                            );
                                            self.delete_node_indices.clear();
                                        }
                                    }
                                });
                            })
                            .response;
                        if !crate::IS_NATIVE {
                            res.on_disabled_hover_text(t!("web-not-available"));
                        }
                    }
                });
            });
        });
    }
}

#[cfg(unix)]
fn get_unix_metadata(path_str: &str) -> Option<(String, String, String)> {
    use std::{fs, os::unix::fs::MetadataExt as _};

    let metadata = fs::symlink_metadata(path_str).ok()?;
    let uid = metadata.uid();
    let gid = metadata.gid();
    let mode = metadata.mode();

    // Query UID natively and safely
    let user = uzers::get_user_by_uid(uid).map_or_else(
        || uid.to_string(),
        |u| u.name().to_string_lossy().into_owned(),
    );

    // Query GID natively and safely
    let group = uzers::get_group_by_gid(gid).map_or_else(
        || gid.to_string(),
        |g| g.name().to_string_lossy().into_owned(),
    );

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

fn draw_action_button(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        ui.scope(|ui| {
            ui.style_mut().visuals.button_frame = true;

            // Inactive (transparent/very subtle background border)
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = color.linear_multiply(0.04);
            ui.style_mut().visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0f32, color.linear_multiply(0.2));
            ui.style_mut().visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0f32, ui.visuals().widgets.inactive.text_color());

            // Hovered (soft fill, solid border)
            ui.style_mut().visuals.widgets.hovered.weak_bg_fill = color.linear_multiply(0.12);
            ui.style_mut().visuals.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0f32, color.linear_multiply(0.45));
            ui.style_mut().visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0f32, color);

            // Active (pressed)
            ui.style_mut().visuals.widgets.active.weak_bg_fill = color.linear_multiply(0.22);
            ui.style_mut().visuals.widgets.active.bg_stroke =
                egui::Stroke::new(1.0f32, color.linear_multiply(0.65));
            ui.style_mut().visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0f32, color);

            ui.button(label)
        })
        .inner
    })
    .inner
}

/// Compares two arena nodes based on the selected column index and metadata.
#[inline]
#[must_use]
pub fn compare_nodes_by_column(
    snapshot: &FileArenaSnapshot,
    col_index: usize,
    a: usize,
    b: usize,
) -> std::cmp::Ordering {
    let node_a = &snapshot.nodes[a];
    let node_b = &snapshot.nodes[b];

    match col_index {
        0 => {
            // Sort by Name
            let name_a = snapshot.string_pool.get(node_a.name_id).unwrap_or("");
            let name_b = snapshot.string_pool.get(node_b.name_id).unwrap_or("");
            name_a.cmp(name_b)
        }
        2 => {
            // Sort by Size
            node_a.size.cmp(&node_b.size)
        }
        3 => {
            // Sort by Items (Files + Subdirectories)
            let items_a = if node_a.is_directory() {
                node_a.file_count + *snapshot.dir_counts.get(a).unwrap_or(&0)
            } else {
                0
            };
            let items_b = if node_b.is_directory() {
                node_b.file_count + *snapshot.dir_counts.get(b).unwrap_or(&0)
            } else {
                0
            };
            items_a.cmp(&items_b)
        }
        4 => {
            // Sort by Files count
            let files_a = if node_a.is_directory() {
                node_a.file_count
            } else {
                0
            };
            let files_b = if node_b.is_directory() {
                node_b.file_count
            } else {
                0
            };
            files_a.cmp(&files_b)
        }
        5 => {
            // Sort by Subdirectories count
            let subdirs_a = if node_a.is_directory() {
                *snapshot.dir_counts.get(a).unwrap_or(&0)
            } else {
                0
            };
            let subdirs_b = if node_b.is_directory() {
                *snapshot.dir_counts.get(b).unwrap_or(&0)
            } else {
                0
            };
            subdirs_a.cmp(&subdirs_b)
        }
        6 => {
            // Sort by Created timestamp
            node_a.created_timestamp.cmp(&node_b.created_timestamp)
        }
        7 => {
            // Sort by Last Modified timestamp
            node_a.modified_timestamp.cmp(&node_b.modified_timestamp)
        }
        _ => {
            // Fallback (Percentage, etc.)
            node_a.size.cmp(&node_b.size)
        }
    }
}
