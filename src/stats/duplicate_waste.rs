#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;

use egui_plot::{Bar, BarChart};

use crate::arena::FileArenaSnapshot;

pub struct DuplicateWasteChart {
    pub top_extensions: Vec<(String, u64)>, // (ext, wasted_bytes)
    pub last_snapshot_ptr: usize,
    pub last_results_count: usize,
}

impl DuplicateWasteChart {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            top_extensions: Vec::new(),
            last_snapshot_ptr: 0,
            last_results_count: usize::MAX,
        }
    }

    pub fn compute_waste(
        &mut self,
        snapshot: &FileArenaSnapshot,
        results: &crate::stats::deduplicator::DeduplicationResults,
    ) {
        self.top_extensions.clear();

        if results.groups.is_empty() {
            return;
        }

        let mut ext_waste: HashMap<String, u64> = HashMap::new();

        for group in &results.groups {
            if group.nodes.is_empty() {
                continue;
            }

            // Hardlink-aware wasted bytes calculation
            let unique_inodes_count = {
                let mut ids: Vec<(u64, u64)> = group.file_ids.clone();
                ids.sort_unstable();
                ids.dedup();
                ids.retain(|&id| id != (0, 0));
                ids.len()
            };

            let wasted_bytes = if unique_inodes_count > 0 {
                group.size * (unique_inodes_count as u64 - 1)
            } else {
                group.size * (group.nodes.len().saturating_sub(1) as u64)
            };

            if wasted_bytes == 0 {
                continue;
            }

            // Use the extension of the first file in the group as the representative extension
            if let Some(&first_node_idx) = group.nodes.first() {
                let first_node = &snapshot.nodes[first_node_idx as usize];
                if let Some(name) = snapshot.string_pool.get(first_node.name_id) {
                    let ext = std::path::Path::new(name).extension().map_or_else(
                        || "(no extension)".to_string(),
                        |s| s.to_string_lossy().to_ascii_lowercase(),
                    );
                    *ext_waste.entry(ext).or_insert(0) += wasted_bytes;
                }
            }
        }

        // Sort descending by wasted bytes and limit to the top 8 extensions
        let mut sorted: Vec<(String, u64)> = ext_waste.into_iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
        sorted.truncate(8);

        self.top_extensions = sorted;
    }
}

impl Default for DuplicateWasteChart {
    fn default() -> Self {
        Self::new()
    }
}

impl super::StatsChart for DuplicateWasteChart {
    type Output = ();

    fn compute(&mut self, _snapshot: &crate::arena::FileArenaSnapshot) -> Self::Output {
        // Computation requires both the snapshot and the deduplicator results, so we use compute_waste directly
    }
}

impl super::StatComponent for DuplicateWasteChart {
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        context: &mut super::StatContext,
    ) {
        let Some(results_lock) = context.deduplicator_results else {
            ui.centered_and_justified(|ui| {
                ui.label("Deduplicator results are unavailable in this context.");
            });
            return;
        };

        let results_count = results_lock.read().groups.len();

        if results_count == 0 {
            ui.centered_and_justified(|ui| {
                ui.label("No duplicate data available. Run a 'Duplicate File Finder' scan under the Deduplicator tab to generate duplicate waste analytics.");
            });
            return;
        }

        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;
        let needs_rebuild = self.last_snapshot_ptr != snapshot_ptr
            || self.last_results_count != results_count
            || self.top_extensions.is_empty();

        if needs_rebuild {
            self.compute_waste(snapshot, &results_lock.read());
            self.last_snapshot_ptr = snapshot_ptr;
            self.last_results_count = results_count;
        }

        if self.top_extensions.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No space-wasting duplicates detected (all duplicates may be hardlinks or single copies).");
            });
            return;
        }

        let total_wasted_bytes: u64 = self.top_extensions.iter().map(|(_, bytes)| bytes).sum();
        let formatted_total = prettier_bytes::ByteFormatter::new()
            .format(total_wasted_bytes)
            .to_string();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("👥 Duplicate Waste by Extension");
                ui.separator();
                ui.label("Total Wasted Space:");
                ui.colored_label(crate::colors::WARNING_RED, formatted_total);
            });
            ui.add_space(6.0);

            let count = self.top_extensions.len();

            let bars: Vec<Bar> = self
                .top_extensions
                .iter()
                .enumerate()
                .map(|(i, (ext, wasted_bytes))| {
                    let index = i as f64;
                    let height = *wasted_bytes as f64;
                    Bar::new(index, height)
                        .name(format!(".{ext}"))
                        .fill(crate::colors::get_color_for_extension(ext))
                })
                .collect();

            let bar_chart = BarChart::new("Duplicate Waste", bars).width(0.6);

            let ext_names: Vec<String> = self
                .top_extensions
                .iter()
                .map(|(ext, _)| ext.clone())
                .collect();

            let x_formatter =
                move |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| {
                    let val_f = mark.value.round();
                    if val_f >= 0.0 && val_f < ext_names.len() as f64 {
                        let val = val_f as usize;
                        format!(".{}", ext_names[val])
                    } else {
                        String::new()
                    }
                };

            let y_formatter =
                |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| {
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
                for i in 0..count {
                    marks.push(egui_plot::GridMark {
                        value: i as f64,
                        step_size: 1.0,
                    });
                }
                marks
            };

            let x_axes = vec![
                egui_plot::AxisHints::new_x()
                    .label("File Extension")
                    .formatter(x_formatter),
            ];
            let y_axes = vec![
                egui_plot::AxisHints::new_y()
                    .label("Wasted Disk Space")
                    .formatter(y_formatter),
            ];

            let plot = egui_plot::Plot::new("duplicate_waste_plot")
                .height(ui.available_height() - 10.0)
                .custom_x_axes(x_axes)
                .custom_y_axes(y_axes)
                .x_grid_spacer(x_grid)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false);

            plot.show(ui, |plot_ui| {
                plot_ui.bar_chart(bar_chart);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        arena::{FileArenaSnapshot, FileNode, NodeStorage, StringPool, precompute_dir_counts},
        stats::deduplicator::{DeduplicationResults, DuplicateGroup},
    };

    #[test]
    fn test_duplicate_waste_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let results = DeduplicationResults::default();
        let mut chart = DuplicateWasteChart::new();
        chart.compute_waste(&snapshot, &results);
        assert!(chart.top_extensions.is_empty());
    }

    #[test]
    fn test_duplicate_waste_with_groups() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"file1.png");
        let f2_id = pool.get_or_insert(b"file2.png");
        let f3_id = pool.get_or_insert(b"file3.txt");
        let f4_id = pool.get_or_insert(b"file4.txt");

        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f2_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f3_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f4_id, Some(0), false, false, 0, 0, 0),
        ];

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let results = DeduplicationResults {
            groups: vec![
                DuplicateGroup {
                    size: 1000,
                    nodes: vec![1, 2],
                    file_ids: vec![(0, 0), (0, 0)],
                },
                DuplicateGroup {
                    size: 500,
                    nodes: vec![3, 4],
                    file_ids: vec![(0, 0), (0, 0)],
                },
            ],
            flat_rows: vec![],
        };

        let mut chart = DuplicateWasteChart::new();
        chart.compute_waste(&snapshot, &results);

        assert_eq!(chart.top_extensions.len(), 2);
        assert_eq!(chart.top_extensions[0], ("png".to_string(), 1000));
        assert_eq!(chart.top_extensions[1], ("txt".to_string(), 500));
    }

    #[test]
    fn test_duplicate_waste_with_hardlinks() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"file1.png");
        let f2_id = pool.get_or_insert(b"file2.png");

        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0, 0),
            FileNode::new(f2_id, Some(0), false, false, 0, 0, 0),
        ];

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let results = DeduplicationResults {
            groups: vec![DuplicateGroup {
                size: 1000,
                nodes: vec![1, 2],
                file_ids: vec![(1, 1), (1, 1)],
            }],
            flat_rows: vec![],
        };

        let mut chart = DuplicateWasteChart::new();
        chart.compute_waste(&snapshot, &results);

        assert!(chart.top_extensions.is_empty());
    }
}
