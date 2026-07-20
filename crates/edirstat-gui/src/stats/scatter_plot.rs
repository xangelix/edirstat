pub struct FileAgeSizeScatterChart {
    pub top_files: Vec<(u32, u64)>, // (node_idx, size)
    pub max_timestamp: u32,
    pub last_snapshot_ptr: usize,
}

impl FileAgeSizeScatterChart {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            top_files: Vec::new(),
            max_timestamp: 0,
            last_snapshot_ptr: 0,
        }
    }
}

impl Default for FileAgeSizeScatterChart {
    fn default() -> Self {
        Self::new()
    }
}

impl super::StatsChart for FileAgeSizeScatterChart {
    type Output = ();

    fn compute(&mut self, snapshot: &crate::arena::FileArenaSnapshot) -> Self::Output {
        if snapshot.nodes.is_empty() {
            self.top_files.clear();
            self.max_timestamp = 0;
            return;
        }

        // 1. Establish the modern baseline using the most recent modification time
        let mut max_time = 0u32;
        for node in snapshot.nodes.iter() {
            if !node.is_directory() && node.modified_timestamp > max_time {
                max_time = node.modified_timestamp;
            }
        }
        self.max_timestamp = max_time;

        // 2. Gather all leaf nodes with a physical size
        let mut files: Vec<(u32, u64)> = snapshot
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.is_directory() && node.size > 0)
            .map(|(idx, node)| (idx as u32, node.size))
            .collect();

        // 3. Sort descending to isolate the top 5,000 items
        files.sort_by_key(|b| std::cmp::Reverse(b.1));
        files.truncate(5000);

        self.top_files = files;
    }
}

impl super::StatComponent for FileAgeSizeScatterChart {
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        _context: &mut super::StatContext,
    ) {
        use super::StatsChart;
        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;
        if self.last_snapshot_ptr != snapshot_ptr || self.top_files.is_empty() {
            self.compute(snapshot);
            self.last_snapshot_ptr = snapshot_ptr;
        }

        if self.top_files.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No file data available to plot.");
            });
            return;
        }

        let max_time = self.max_timestamp;

        // Populate log-scale coordinates [Age, Size]
        let plot_points: Vec<[f64; 2]> = self
            .top_files
            .iter()
            .map(|&(idx, size)| {
                let node = &snapshot.nodes[idx as usize];

                #[allow(clippy::cast_precision_loss)]
                let age_days = if max_time > node.modified_timestamp {
                    (max_time - node.modified_timestamp) as f64 / 86400.0
                } else {
                    0.0
                };
                #[allow(clippy::cast_precision_loss)]
                let size_log = (size as f64).log10();

                [age_days, size_log]
            })
            .collect();

        // Format grid boundaries log10 values into pretty byte readouts
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

        let x_axes = vec![egui_plot::AxisHints::new_x().label("File Age (Days unmodified)")];
        let y_axes = vec![
            egui_plot::AxisHints::new_y()
                .label("File Size (Logarithmic)")
                .formatter(y_formatter),
        ];

        let points = egui_plot::Points::new("Top 5,000 Space Hogs", plot_points)
            .radius(2.0f32)
            .color(crate::colors::COLOR_SCANNING);

        let plot = egui_plot::Plot::new("age_size_scatter_plot")
            .height(ui.available_height() - 10.0)
            .custom_x_axes(x_axes)
            .custom_y_axes(y_axes)
            .show_background(true);

        let egui_plot::PlotResponse {
            inner: (pointer_coordinate, bounds),
            ..
        } = plot.show(ui, |plot_ui| {
            plot_ui.points(points);
            (plot_ui.pointer_coordinate(), plot_ui.plot_bounds())
        });

        // Hover coordinates check for rendering on-demand details
        if let Some(coord) = pointer_coordinate {
            let bounds_width = bounds.width();
            let bounds_height = bounds.height();

            if bounds_width > 0.0 && bounds_height > 0.0 {
                let mut closest_node_idx = None;
                let mut min_dist_sq = f64::INFINITY;

                for &(idx, size) in &self.top_files {
                    let node = &snapshot.nodes[idx as usize];

                    #[allow(clippy::cast_precision_loss)]
                    let age_days = if max_time > node.modified_timestamp {
                        (max_time - node.modified_timestamp) as f64 / 86400.0
                    } else {
                        0.0
                    };

                    #[allow(clippy::cast_precision_loss)]
                    let size_log = (size as f64).log10();

                    // Standardize aspect ratio coordinate scaling
                    let dx = (coord.x - age_days) / bounds_width;
                    let dy = (coord.y - size_log) / bounds_height;
                    let dist_sq = dy.mul_add(dy, dx * dx);

                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                        closest_node_idx = Some(idx);
                    }
                }

                // Tooltip displays if within visual vicinity (0.02 screen-radius bounds)
                if min_dist_sq < 0.0004
                    && let Some(node_idx) = closest_node_idx
                {
                    let node = &snapshot.nodes[node_idx as usize];
                    let path_str = snapshot.get_full_path(node_idx);
                    let size_str = prettier_bytes::ByteFormatter::new()
                        .format(node.size)
                        .to_string();

                    let age_days = if max_time > node.modified_timestamp {
                        (max_time - node.modified_timestamp) / 86400
                    } else {
                        0
                    };

                    eframe::egui::Tooltip::always_open(
                        ui.ctx().clone(),
                        ui.layer_id(),
                        eframe::egui::Id::new("scatter_tooltip"),
                        eframe::egui::PopupAnchor::Pointer,
                    )
                    .show(|ui| {
                        ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                        let cleaned_path = crate::arena::clean_unc_path(&path_str);
                        ui.label(format!("📄 Path: {cleaned_path}"));
                        ui.label(format!("💾 Size: {size_str}"));
                        ui.label(format!("⏳ Age: {age_days} days unmodified"));
                    });
                }
            }
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
    fn test_scatter_plot_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let mut chart = FileAgeSizeScatterChart::new();
        chart.compute(&snapshot);
        assert!(chart.top_files.is_empty());
        assert_eq!(chart.max_timestamp, 0);
    }

    #[test]
    fn test_scatter_plot_standard() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");
        let f2_id = pool.get_or_insert(b"f2.txt");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 1000, 0),
            FileNode::new(f2_id, Some(0), false, false, 2000, 0),
        ];
        nodes[1].size = 500;
        nodes[2].size = 1000;

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = FileAgeSizeScatterChart::new();
        chart.compute(&snapshot);

        assert_eq!(chart.max_timestamp, 2000);
        assert_eq!(chart.top_files.len(), 2);
        assert_eq!(chart.top_files[0], (2, 1000));
        assert_eq!(chart.top_files[1], (1, 500));
    }
}
