use std::collections::HashMap;

use egui_plot::{Bar, BarChart};

use crate::arena::{FileArenaSnapshot, FileNode, NO_INDEX, StringPool};

pub struct DirCompositionChart {
    pub parent_idx: u32,
    pub top_extensions: Vec<String>,
    // Holds (child_name, child_extension_map, total_bytes) for the top 8 children
    pub children_composition: Vec<(String, HashMap<String, u64, ahash::RandomState>, u64)>,
    pub last_snapshot_ptr: usize,
}

impl DirCompositionChart {
    #[must_use]
    pub const fn new(parent_idx: u32) -> Self {
        Self {
            parent_idx,
            top_extensions: Vec::new(),
            children_composition: Vec::new(),
            last_snapshot_ptr: 0,
        }
    }
}

impl Default for DirCompositionChart {
    fn default() -> Self {
        Self::new(0)
    }
}

impl super::StatsChart for DirCompositionChart {
    type Output = ();

    fn compute(&mut self, snapshot: &FileArenaSnapshot) -> Self::Output {
        self.top_extensions.clear();
        self.children_composition.clear();

        if snapshot.nodes.is_empty() || self.parent_idx as usize >= snapshot.nodes.len() {
            return;
        }

        let parent_node = &snapshot.nodes[self.parent_idx as usize];
        if !parent_node.is_directory() {
            return;
        }

        // 1. Gather all immediate children of the parent directory
        let mut immediate_children = Vec::new();
        let mut curr = parent_node.first_child;
        while curr != NO_INDEX {
            immediate_children.push(curr);
            curr = snapshot.nodes[curr as usize].next_sibling;
        }

        if immediate_children.is_empty() {
            return;
        }

        // Sort immediate children descending by size
        immediate_children.sort_by(|&a, &b| {
            snapshot.nodes[b as usize]
                .size
                .cmp(&snapshot.nodes[a as usize].size)
        });

        // Restrict to the top 8 largest children for clean, readable layout spacing
        immediate_children.truncate(8);

        // 2. Compute extension composition for each child
        let mut overall_ext_sizes: HashMap<String, u64> = HashMap::new();

        for &child_idx in &immediate_children {
            let child_node = &snapshot.nodes[child_idx as usize];
            let name = snapshot
                .string_pool
                .get(child_node.name_id)
                .unwrap_or("unknown")
                .to_string();

            let mut ext_map = HashMap::with_hasher(ahash::RandomState::new());
            if child_node.is_directory() {
                // Recursively gather file extension profiles of the subdirectory
                gather_dir_extensions(
                    &snapshot.nodes,
                    &snapshot.string_pool,
                    child_idx,
                    &mut ext_map,
                );
            } else {
                let ext = std::path::Path::new(&name).extension().map_or_else(
                    || "(no extension)".to_string(),
                    |s| s.to_string_lossy().to_ascii_lowercase(),
                );
                ext_map.insert(ext, child_node.size);
            }

            // Aggregate overall sizes to identify dominant extension groups
            for (ext, size) in &ext_map {
                *overall_ext_sizes.entry(ext.clone()).or_insert(0) += size;
            }

            self.children_composition
                .push((name, ext_map, child_node.size));
        }

        // Sort globally observed extensions to choose the top 5 largest formats
        let mut sorted_exts: Vec<(String, u64)> = overall_ext_sizes.into_iter().collect();
        sorted_exts.sort_by_key(|b| std::cmp::Reverse(b.1));
        sorted_exts.truncate(5);

        let top_exts: Vec<String> = sorted_exts.into_iter().map(|(ext, _)| ext).collect();
        self.top_extensions.clone_from(&top_exts);
    }
}

impl super::StatComponent for DirCompositionChart {
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        context: &mut super::StatContext,
    ) {
        use super::StatsChart;
        // Bind composition to active tree folder, falling back to root (0)
        let active_dir = context
            .selected_nodes
            .iter()
            .copied()
            .find(|&idx| {
                idx < snapshot.nodes.len() as u32 && snapshot.nodes[idx as usize].is_directory()
            })
            .unwrap_or(0);

        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;
        let needs_rebuild = self.last_snapshot_ptr != snapshot_ptr
            || self.parent_idx != active_dir
            || self.children_composition.is_empty();

        if needs_rebuild {
            self.parent_idx = active_dir;
            self.compute(snapshot);
            self.last_snapshot_ptr = snapshot_ptr;
        }

        if self.children_composition.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    "Selected path has no nested subdirectories or files to display composition.",
                );
            });
            return;
        }

        let parent_name = snapshot
            .string_pool
            .get(snapshot.nodes[active_dir as usize].name_id)
            .unwrap_or("Root");
        ui.strong(format!("📁 Active Directory: {parent_name}"));
        ui.add_space(4.0);

        // Build stacked BarCharts on demand
        let mut unstacked_charts = Vec::new();

        // Create individual BarChart bars for each top extension
        for ext in &self.top_extensions {
            let mut bars = Vec::new();
            for (i, (_child_name, ext_map, _total_size)) in
                self.children_composition.iter().enumerate()
            {
                #[allow(clippy::cast_precision_loss)]
                let height = *ext_map.get(ext).unwrap_or(&0) as f64;
                #[allow(clippy::cast_precision_loss)]
                let index = i as f64;

                bars.push(Bar::new(index, height).name(ext));
            }
            let chart = BarChart::new(ext.clone(), bars)
                .width(0.5)
                .name(format!(".{ext} files"))
                .color(crate::colors::get_color_for_extension(ext));
            unstacked_charts.push(chart);
        }

        // Add remaining non-dominant extensions under the "Other" category
        let mut other_bars = Vec::new();
        for (i, (_child_name, ext_map, _total_size)) in self.children_composition.iter().enumerate()
        {
            let mut other_height = 0u64;
            for (ext, &size) in ext_map {
                if !self.top_extensions.contains(ext) {
                    other_height += size;
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let index = i as f64;
            #[allow(clippy::cast_precision_loss)]
            let height = other_height as f64;

            other_bars.push(Bar::new(index, height).name("Other"));
        }
        let other_chart = BarChart::new("Other".to_string(), other_bars)
            .width(0.5)
            .name("Other files")
            .color(crate::colors::TREEMAP_DIR_FALLBACK);
        unstacked_charts.push(other_chart);

        // 4. Transform unstacked series into a stacked vector
        let mut stacked_charts = Vec::new();
        for unstacked in unstacked_charts {
            let refs: Vec<&BarChart> = stacked_charts.iter().collect();
            let stacked = unstacked.stack_on(&refs);
            stacked_charts.push(stacked);
        }

        let children_count = self.children_composition.len();

        // Clone names to keep closure 'static
        let children_names: Vec<String> = self
            .children_composition
            .iter()
            .map(|(name, _, _)| name.clone())
            .collect();
        let x_formatter = move |mark: egui_plot::GridMark,
                                _range: &std::ops::RangeInclusive<f64>| {
            let val = mark.value.round() as usize;
            if val < children_count {
                children_names[val].clone()
            } else {
                String::new()
            }
        };

        let y_formatter = |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| {
            let val = mark.value;
            if val <= 0.0 {
                return String::new();
            }
            prettier_bytes::ByteFormatter::new()
                .format(val as u64)
                .to_string()
        };

        let x_grid = move |_input: egui_plot::GridInput| {
            let mut marks = vec![];
            for i in 0..children_count {
                #[allow(clippy::cast_precision_loss)]
                let value = i as f64;

                marks.push(egui_plot::GridMark {
                    value,
                    step_size: 1.0,
                });
            }
            marks
        };

        let x_axes = vec![
            egui_plot::AxisHints::new_x()
                .label("Direct Children")
                .formatter(x_formatter),
        ];
        let y_axes = vec![
            egui_plot::AxisHints::new_y()
                .label("Cumulative Space")
                .formatter(y_formatter),
        ];

        let plot = egui_plot::Plot::new("dir_composition_plot")
            .height(ui.available_height() - 30.0)
            .custom_x_axes(x_axes)
            .custom_y_axes(y_axes)
            .x_grid_spacer(x_grid)
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::RightTop))
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false);

        plot.show(ui, |plot_ui| {
            for chart in stacked_charts {
                plot_ui.bar_chart(chart);
            }
        });
    }
}

/// Accumulates file sizes of a directory subtree in a safe, stack-based non-recursive layout,
/// capped at 20,000 files to guarantee lightning-fast visual updates.
pub fn gather_dir_extensions(
    nodes: &[FileNode],
    string_pool: &StringPool,
    start_idx: u32,
    ext_sizes: &mut HashMap<String, u64, ahash::RandomState>,
) {
    let mut stack = vec![start_idx];
    let mut visited_count = 0;

    while let Some(idx) = stack.pop() {
        visited_count += 1;
        if visited_count > 20000 {
            break;
        }

        let node = &nodes[idx as usize];
        if node.is_directory() {
            let mut curr = node.first_child;
            while curr != NO_INDEX {
                stack.push(curr);
                curr = nodes[curr as usize].next_sibling;
            }
        } else {
            let name = string_pool.get(node.name_id).unwrap_or("");
            let ext = std::path::Path::new(name).extension().map_or_else(
                || "(no extension)".to_string(),
                |s| s.to_string_lossy().to_ascii_lowercase(),
            );
            *ext_sizes.entry(ext).or_insert(0) += node.size;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        arena::{FileArenaSnapshot, FileNode, NodeStorage, StringPool, precompute_dir_counts},
        stats::StatsChart,
    };

    #[test]
    fn test_dir_composition_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let mut chart = DirCompositionChart::new(0);
        chart.compute(&snapshot);
        assert!(chart.top_extensions.is_empty());
        assert!(chart.children_composition.is_empty());
    }

    #[test]
    fn test_dir_composition_no_children() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");

        let nodes = vec![FileNode::new(r_id, None, true, false, 0, 0)];
        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = DirCompositionChart::new(0);
        chart.compute(&snapshot);
        assert!(chart.top_extensions.is_empty());
        assert!(chart.children_composition.is_empty());
    }

    #[test]
    fn test_dir_composition_standard() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");
        let f2_id = pool.get_or_insert(b"f2.txt");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0),
            FileNode::new(f2_id, Some(0), false, false, 0, 0),
        ];
        nodes[0].first_child = 1;
        nodes[1].next_sibling = 2;
        nodes[1].size = 500;
        nodes[2].size = 300;

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = DirCompositionChart::new(0);
        chart.compute(&snapshot);

        assert_eq!(chart.top_extensions.len(), 2);
        assert_eq!(chart.top_extensions[0], "png");
        assert_eq!(chart.top_extensions[1], "txt");

        assert_eq!(chart.children_composition.len(), 2);
        assert_eq!(chart.children_composition[0].0, "f1.png");
        assert_eq!(chart.children_composition[0].2, 500);
        assert_eq!(chart.children_composition[1].0, "f2.txt");
        assert_eq!(chart.children_composition[1].2, 300);
    }

    #[test]
    fn test_gather_dir_extensions() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let d1_id = pool.get_or_insert(b"dir1");
        let f1_id = pool.get_or_insert(b"f1.png");
        let f2_id = pool.get_or_insert(b"f2.txt");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(d1_id, Some(0), true, false, 0, 0),
            FileNode::new(f1_id, Some(1), false, false, 0, 0),
            FileNode::new(f2_id, Some(1), false, false, 0, 0),
        ];
        nodes[0].first_child = 1;
        nodes[1].first_child = 2;
        nodes[2].next_sibling = 3;
        nodes[2].size = 100;
        nodes[3].size = 200;

        let mut ext_sizes = HashMap::with_hasher(ahash::RandomState::new());
        gather_dir_extensions(&nodes, &pool, 1, &mut ext_sizes);

        assert_eq!(ext_sizes.len(), 2);
        assert_eq!(ext_sizes.get("png"), Some(&100));
        assert_eq!(ext_sizes.get("txt"), Some(&200));
    }
}
