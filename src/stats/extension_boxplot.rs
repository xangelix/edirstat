use std::collections::HashMap;

use egui_plot::BoxSpread;

use crate::arena::FileArenaSnapshot;

pub struct ExtensionBoxplotChart {
    pub top_extensions: Vec<String>,
    pub computed_spreads: Vec<(String, BoxSpread)>, // (ext, spread)
    pub last_snapshot_ptr: usize,
}

impl ExtensionBoxplotChart {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            top_extensions: Vec::new(),
            computed_spreads: Vec::new(),
            last_snapshot_ptr: 0,
        }
    }
}

impl Default for ExtensionBoxplotChart {
    fn default() -> Self {
        Self::new()
    }
}

impl super::StatsChart for ExtensionBoxplotChart {
    type Output = ();

    fn compute(&mut self, snapshot: &FileArenaSnapshot) -> Self::Output {
        self.top_extensions.clear();
        self.computed_spreads.clear();

        if snapshot.nodes.is_empty() {
            return;
        }

        // 1. Map extension names to log10(sizes) vector
        let mut ext_files: HashMap<String, Vec<f64>> = HashMap::new();

        for node in snapshot.nodes.iter() {
            if node.is_directory() {
                continue;
            }
            if let Some(name) = snapshot.string_pool.get(node.name_id) {
                let ext = std::path::Path::new(name).extension().map_or_else(
                    || "(no extension)".to_string(),
                    |s| s.to_string_lossy().to_ascii_lowercase(),
                );
                if node.size > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let log_size = (node.size as f64).log10();

                    ext_files.entry(ext).or_default().push(log_size);
                }
            }
        }

        if ext_files.is_empty() {
            return;
        }

        // 2. Sort the list of extensions descending by file sample counts to ensure statistical significance
        let mut sorted_exts: Vec<(String, Vec<f64>)> = ext_files.into_iter().collect();
        sorted_exts.sort_by_key(|b| std::cmp::Reverse(b.1.len()));
        sorted_exts.truncate(6);

        // 3. Compute box spread parameters (min, Q1, median, Q3, max)
        for (ext, mut sizes) in sorted_exts {
            if sizes.len() < 4 {
                // Ignore categories with fewer than 4 files to guarantee box structural integrity
                continue;
            }
            sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let len = sizes.len();
            let min = sizes[0];
            let q1 = sizes[len / 4];
            let median = sizes[len / 2];
            let q3 = sizes[(len * 3) / 4];
            let max = sizes[len - 1];

            let spread = BoxSpread::new(min, q1, median, q3, max);
            self.top_extensions.push(ext.clone());
            self.computed_spreads.push((ext, spread));
        }
    }
}

impl super::StatComponent for ExtensionBoxplotChart {
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        _context: &mut super::StatContext,
    ) {
        use super::StatsChart;
        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;
        let needs_rebuild =
            self.last_snapshot_ptr != snapshot_ptr || self.computed_spreads.is_empty();

        if needs_rebuild {
            self.compute(snapshot);
            self.last_snapshot_ptr = snapshot_ptr;
        }

        if self.computed_spreads.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    "Not enough file data in any single extension category to generate box plots.",
                );
            });
            return;
        }

        let spreads_count = self.computed_spreads.len();

        // Clone names to keep closure 'static
        let ext_names: Vec<String> = self
            .computed_spreads
            .iter()
            .map(|(ext, _)| ext.clone())
            .collect();
        let x_formatter = move |mark: egui_plot::GridMark,
                                _range: &std::ops::RangeInclusive<f64>| {
            let val = mark.value.round() as usize;
            if val < ext_names.len() {
                format!(".{}", ext_names[val])
            } else {
                String::new()
            }
        };

        let y_formatter = |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| {
            let val = mark.value;
            if val < 0.0 {
                return String::new();
            }
            let bytes = 10.0f64.powf(val);
            if bytes >= 1.0 {
                prettier_bytes::ByteFormatter::new()
                    .format(bytes as u64)
                    .to_string()
            } else {
                String::new()
            }
        };

        let x_grid = move |_input: egui_plot::GridInput| {
            let mut marks = vec![];
            for i in 0..spreads_count {
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
                .label("Top Extensions (by file count)")
                .formatter(x_formatter),
        ];
        let y_axes = vec![
            egui_plot::AxisHints::new_y()
                .label("File Size Distribution")
                .formatter(y_formatter),
        ];

        let plot = egui_plot::Plot::new("boxplot_plot")
            .height(ui.available_height() - 10.0)
            .custom_x_axes(x_axes)
            .custom_y_axes(y_axes)
            .x_grid_spacer(x_grid)
            .legend(egui_plot::Legend::default().position(egui_plot::Corner::RightTop))
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false);

        plot.show(ui, |plot_ui| {
            for (i, (ext, spread)) in self.computed_spreads.iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let index = i as f64;

                let elem =
                    egui_plot::BoxElem::new(index, spread.clone()).name(format!(".{ext} sizes"));
                let box_plot = egui_plot::BoxPlot::new(ext.clone(), vec![elem])
                    .color(crate::colors::get_color_for_extension(ext));
                plot_ui.box_plot(box_plot);
            }
        });
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        arena::{FileArenaSnapshot, FileNode, NodeStorage, StringPool, precompute_dir_counts},
        stats::StatsChart,
    };

    #[test]
    fn test_extension_boxplot_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let mut chart = ExtensionBoxplotChart::new();
        chart.compute(&snapshot);
        assert!(chart.top_extensions.is_empty());
        assert!(chart.computed_spreads.is_empty());
    }

    #[test]
    fn test_extension_boxplot_too_few_files() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");
        let f2_id = pool.get_or_insert(b"f2.png");
        let f3_id = pool.get_or_insert(b"f3.png");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f2_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f3_id, Some(0), false, false, 0, 0, 0),
        ];
        nodes[1].size = 100;
        nodes[2].size = 200;
        nodes[3].size = 300;

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = ExtensionBoxplotChart::new();
        chart.compute(&snapshot);

        assert!(chart.computed_spreads.is_empty());
    }

    #[test]
    fn test_extension_boxplot_sufficient_files() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");
        let f2_id = pool.get_or_insert(b"f2.png");
        let f3_id = pool.get_or_insert(b"f3.png");
        let f4_id = pool.get_or_insert(b"f4.png");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f2_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f3_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f4_id, Some(0), false, false, 0, 0, 0),
        ];
        nodes[1].size = 10;
        nodes[2].size = 100;
        nodes[3].size = 1000;
        nodes[4].size = 10000;

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = ExtensionBoxplotChart::new();
        chart.compute(&snapshot);

        assert_eq!(chart.computed_spreads.len(), 1);
        let (ext, spread) = &chart.computed_spreads[0];
        assert_eq!(ext, "png");

        assert_eq!(spread.lower_whisker, 1.0);
        assert_eq!(spread.quartile1, 2.0);
        assert_eq!(spread.median, 3.0);
        assert_eq!(spread.quartile3, 4.0);
        assert_eq!(spread.upper_whisker, 4.0);
    }
}
