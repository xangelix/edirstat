use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use eframe::egui;
use rfd::FileDialog;

use super::{
    arena::{FileArenaSnapshot, NO_EXTENSION},
    coordinator::SharedState,
    persistence::{load_snapshot, save_snapshot},
    stats::{self, StatComponent as _},
    traversal::TraversalEngine,
};

pub mod deduplicator_tab;
pub mod explorer;
pub mod extensions;
pub mod modals;
pub mod theme;

pub use extensions::ExtensionStat;
pub use modals::ActiveModal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisMode {
    Treemap,
    Plots,
    Deduplicator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotType {
    SizeDistribution,
    AgeSizeScatter,
    DirComposition,
    ExtensionBoxplot,
    TemporalTimeline,
}

#[allow(clippy::struct_excessive_bools)]
pub struct GuiApp {
    pub(crate) shared_state: Arc<SharedState>,
    pub(crate) traversal_engine: Arc<TraversalEngine>,

    // UI state
    pub(crate) selected_node_idx: Option<u32>,
    pub(crate) expanded_nodes: HashSet<u32>,
    pub(crate) search_query: String,
    pub(crate) monospace_paths: bool,
    pub(crate) left_panel_collapsed: bool,
    pub(crate) right_panel_collapsed: bool,

    // Visualization tabs
    pub(crate) vis_mode: VisMode,
    pub(crate) plot_type: PlotType,

    // Analytics components
    pub(crate) treemap_chart: stats::treemap::TreemapChart,
    pub(crate) size_dist_chart: stats::size_distribution::SizeDistributionChart,
    pub(crate) scatter_chart: stats::scatter_plot::FileAgeSizeScatterChart,
    pub(crate) dir_comp_chart: stats::dir_composition::DirCompositionChart,
    pub(crate) boxplot_chart: stats::extension_boxplot::ExtensionBoxplotChart,
    pub(crate) timeline_chart: stats::temporal_timeline::TemporalTimelineChart,

    // Modal states
    pub(crate) delete_confirm_checked: bool,
    pub(crate) delete_node_idx: Option<u32>,
    pub(crate) active_modal: Option<ActiveModal>,

    // Saved scan parameters
    pub(crate) current_scan_path: Option<PathBuf>,
    pub(crate) scan_start_time: Option<Instant>,
    pub(crate) total_scan_duration: Option<Duration>,

    // Extension breakdown stats
    pub(crate) extension_stats: Vec<ExtensionStat>,
    pub(crate) last_extension_update: Option<Instant>,

    // Single-use trigger to automatically scroll the list view to the target row
    pub(crate) scroll_to_selected: bool,

    // Deduplicator states
    pub(crate) deduplicator_config: crate::stats::deduplicator::DeduplicatorConfig,
    pub(crate) deduplicator_progress: atomic_progress::Progress,
    pub(crate) deduplicator_results:
        Arc<parking_lot::RwLock<Vec<crate::stats::deduplicator::DuplicateGroup>>>,
    pub(crate) deduplicator_cancel: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) selected_duplicates: HashSet<u32>,
    pub(crate) delete_duplicates_indices: Vec<u32>,
    pub(crate) deduplicator_flat_rows: Vec<crate::gui::deduplicator_tab::DuplicateRow>,
    pub(crate) deduplicator_last_sig: (usize, usize),
}

impl GuiApp {
    #[must_use]
    pub fn new(shared_state: Arc<SharedState>, traversal_engine: Arc<TraversalEngine>) -> Self {
        Self {
            shared_state,
            traversal_engine,
            selected_node_idx: None,
            expanded_nodes: HashSet::new(),
            search_query: String::new(),
            monospace_paths: false,
            left_panel_collapsed: false,
            right_panel_collapsed: false,
            vis_mode: VisMode::Treemap,
            plot_type: PlotType::SizeDistribution,
            treemap_chart: stats::treemap::TreemapChart::new(),
            size_dist_chart: stats::size_distribution::SizeDistributionChart::new(),
            scatter_chart: stats::scatter_plot::FileAgeSizeScatterChart::new(),
            dir_comp_chart: stats::dir_composition::DirCompositionChart::new(0),
            boxplot_chart: stats::extension_boxplot::ExtensionBoxplotChart::new(),
            timeline_chart: stats::temporal_timeline::TemporalTimelineChart::new(),
            delete_confirm_checked: false,
            delete_node_idx: None,
            active_modal: None,
            current_scan_path: None,
            scan_start_time: None,
            total_scan_duration: None,
            extension_stats: Vec::new(),
            last_extension_update: None,
            scroll_to_selected: false,
            deduplicator_config: crate::stats::deduplicator::DeduplicatorConfig::default(),
            deduplicator_progress: atomic_progress::Progress::new_spinner("Deduplicator"),
            deduplicator_results: Arc::new(parking_lot::RwLock::new(Vec::new())),
            deduplicator_cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            selected_duplicates: HashSet::new(),
            delete_duplicates_indices: Vec::new(),
            deduplicator_flat_rows: Vec::new(),
            deduplicator_last_sig: (0, 0),
        }
    }

    fn reset_state(&mut self) {
        self.selected_node_idx = None;
        self.expanded_nodes.clear();
        self.extension_stats.clear();
        self.last_extension_update = None;
        self.delete_confirm_checked = false;
        self.delete_node_idx = None;
        self.active_modal = None;
        self.selected_duplicates.clear();
        self.delete_duplicates_indices.clear();
        self.deduplicator_flat_rows.clear();
        self.deduplicator_last_sig = (0, 0);
        self.deduplicator_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.deduplicator_progress = atomic_progress::Progress::new_spinner("Deduplicator");
        *self.deduplicator_results.write() = Vec::new();
        self.traversal_engine.stats().reset();
        self.treemap_chart = stats::treemap::TreemapChart::default();
        self.size_dist_chart = stats::size_distribution::SizeDistributionChart::default();
        self.scatter_chart = stats::scatter_plot::FileAgeSizeScatterChart::default();
        self.dir_comp_chart = stats::dir_composition::DirCompositionChart::default();
        self.boxplot_chart = stats::extension_boxplot::ExtensionBoxplotChart::default();
        self.timeline_chart = stats::temporal_timeline::TemporalTimelineChart::default();

        self.scroll_to_selected = false;
    }

    /// Renders the shared "File" actions used in both the top toolbar and node context menus.
    pub(crate) fn draw_file_menu_contents(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        let has_selection = self.selected_node_idx.is_some();

        let open_btn = ui.add_enabled(has_selection, egui::Button::new("🗁 Open in File Manager"));
        if open_btn.clicked() {
            let idx_opt = self.selected_node_idx;
            if let Some(idx) = idx_opt {
                let path_str = snapshot.get_full_path(idx);
                let path = std::path::Path::new(&path_str);
                let dir_to_open = if path.is_dir() {
                    path
                } else {
                    path.parent().map_or(path, |p| p)
                };
                let _ = open::that(dir_to_open);
            }
            ui.close_kind(egui::UiKind::Menu); // Closes the active menu/context-menu
        }

        let delete_btn = ui.add_enabled(has_selection, egui::Button::new("🗑 Delete (Permanent)"));
        if delete_btn.clicked() {
            self.active_modal = Some(ActiveModal::Delete);
            self.delete_confirm_checked = false;
            self.delete_node_idx = self.selected_node_idx;
            ui.close_kind(egui::UiKind::Menu); // Closes the active menu/context-menu
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F9)) {
            self.left_panel_collapsed = !self.left_panel_collapsed;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.right_panel_collapsed = !self.right_panel_collapsed;
        }

        // Fetch current snapshot
        let snapshot = self.shared_state.current_snapshot.load();
        let is_scanning = self.shared_state.is_scanning.load(Ordering::SeqCst);

        // Repaint during scan to show live progress, or continuously while selected to drive the glow animation
        if is_scanning {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else if self.selected_node_idx.is_some() {
            ctx.request_repaint_after(Duration::from_millis(8)); // ~120fps smooth animation loop
        }

        // Apply dark, premium glassmorphism-inspired style
        theme::setup_custom_style(&ctx);

        // Top Control Panel
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("eDirStat 👷")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.separator();

                // Temporarily disable button frames to make top-level menus flat & clean
                let saved_button_frame = ui.visuals().button_frame;
                ui.style_mut().visuals.button_frame = false;

               // Top menu buttons (File / View / Help)
                ui.menu_button("File", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    self.draw_file_menu_contents(ui, &snapshot);
                });
                ui.menu_button("View", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                    // Aligned emoji checkbox layout
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let mut checked = self.monospace_paths;
                        if ui.checkbox(&mut checked, "").changed() {
                            self.monospace_paths = checked;
                        }
                        let response = ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🅰").size(12.0));
                            ui.label("Monospace Paths");
                        }).response;

                        let label_click = ui.interact(response.rect, ui.id().with("monospace_label"), egui::Sense::click());
                        if label_click.clicked() {
                            self.monospace_paths = !self.monospace_paths;
                        }
                    });

                    ui.separator();

                    let left_label = if self.left_panel_collapsed { "▶ Show Left Panel (F9)" } else { "◀ Hide Left Panel (F9)" };
                    if ui.button(left_label).clicked() {
                        self.left_panel_collapsed = !self.left_panel_collapsed;
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    let right_label = if self.right_panel_collapsed { "◀ Show Right Panel (F11)" } else { "▶ Hide Right Panel (F11)" };
                    if ui.button(right_label).clicked() {
                        self.right_panel_collapsed = !self.right_panel_collapsed;
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    ui.separator();
                    if ui.button("🗂 Collapse All").clicked() {
                        self.expanded_nodes.clear();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    if ui.button("ℹ About").clicked() {
                        self.active_modal = Some(ActiveModal::About);
                    }
                });

                ui.separator();

                if ui.button("📁 Scan Directory").clicked() {
                    let folder_opt = FileDialog::new().pick_folder();
                    if let Some(path) = folder_opt {
                        self.reset_state();
                        self.current_scan_path = Some(path.clone());
                        self.scan_start_time = Some(Instant::now());
                        self.total_scan_duration = None;

                        // Start traversal and coordinator
                        let (tx, rx) = crossbeam::channel::unbounded();
                        let traversal = self.traversal_engine.clone();
                        let state = self.shared_state.clone();

                        // Launch Traversal Engine in background
                        match traversal.start_traversal(path.clone(), tx) {
                            Ok(_) => {
                                // Launch Coordinator in background
                                let mut coordinator =
                                    crate::coordinator::Coordinator::new(rx, state);
                                std::thread::spawn(move || {
                                    coordinator.run_coordinator_loop(&path.to_string_lossy());
                                });
                            }
                            Err(e) => {
                                println!("Failed to start traversal: {e}");
                            }
                        }
                    }
                }

                ui.add_space(10.0);

                if ui.button("💾 Save Snapshot").clicked() && !snapshot.nodes.is_empty() {
                    let file_opt = FileDialog::new()
                        .add_filter("eDirStat Snapshot", &["edst"])
                        .save_file();
                    if let Some(path) = file_opt {
                        match save_snapshot(&snapshot.nodes, &snapshot.string_pool, &path) {
                            Ok(()) => {}
                            Err(e) => {
                                println!("Failed to save snapshot: {e}");
                            }
                        }
                    }
                }

                ui.add_space(10.0);

                if ui.button("📖 Load Snapshot").clicked() {
                    let file_opt = FileDialog::new()
                        .add_filter("eDirStat Snapshot", &["edst"])
                        .pick_file();
                    if let Some(path) = file_opt {
                        match load_snapshot(&path) {
                            Ok((arena, string_pool)) => {
                                self.reset_state();
                                let loaded_snapshot = FileArenaSnapshot {
                                    nodes: Arc::new(arena.nodes().to_vec()),
                                    string_pool: Arc::new(string_pool),
                                };
                                self.shared_state
                                    .current_snapshot
                                    .store(Arc::new(loaded_snapshot));
                                self.current_scan_path = Some(path);
                                self.scan_start_time = None;

                                // Rebuild extension stats exactly once in the background upon load
                                let mut ext_map: HashMap<String, (u64, u32)> = HashMap::new();
                                for node in self.shared_state.current_snapshot.load().nodes.iter() {
                                    if node.is_directory() {
                                        continue;
                                    }
                                    if let Some(name) = self
                                        .shared_state
                                        .current_snapshot
                                        .load()
                                        .string_pool
                                        .get(node.name_id)
                                    {
                                        let ext = Path::new(name).extension().map_or_else(
                                            || NO_EXTENSION.to_string(),
                                            |s| s.to_string_lossy().to_ascii_lowercase(),
                                        );
                                        let entry = ext_map.entry(ext).or_insert((0, 0));
                                        entry.0 += node.size;
                                        entry.1 += 1;
                                    }
                                }
                                let mut stats: Vec<(String, u64, u32)> = ext_map
                                    .into_iter()
                                    .map(|(ext, (total_size, file_count))| {
                                        (ext, total_size, file_count)
                                    })
                                    .collect();
                                stats.sort_by_key(|b| std::cmp::Reverse(b.1));
                                self.shared_state.extension_stats.store(Arc::new(stats));
                            }
                            Err(e) => {
                                println!("Failed to load snapshot: {e}");
                            }
                        }
                    }
                }

                ui.separator();

                // Live status display
                if is_scanning {
                    ui.spinner();
                    ui.colored_label(theme::COLOR_SCANNING, "Scanning Disk...");
                } else if self.current_scan_path.is_some() {
                    ui.colored_label(theme::COLOR_SCAN_COMPLETE, "Scan Complete");
                } else {
                    ui.label("Idle");
                }

                if let Some(ref path) = self.current_scan_path {
                    ui.separator();
                    ui.label(format!("Path: {}", path.display()));
                }

                // --- Right-Aligned Concurrency Badge ---
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let threads = self.traversal_engine.num_threads();
                    let badge_text = format!("⚡ {threads} Worker Threads");
                    ui.colored_label(theme::GLOW_INNER_CORE, badge_text)
                        .on_hover_text("The number of parallel, work-stealing CPU cores allocated for directory traversal.");
                });

                ui.style_mut().visuals.button_frame = saved_button_frame; // Restore default button frames
            });
        });

        // Bottom Stats Panel
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let file_count = self
                    .traversal_engine
                    .stats()
                    .files_scanned
                    .load(Ordering::Relaxed);
                let dir_count = self
                    .traversal_engine
                    .stats()
                    .dirs_scanned
                    .load(Ordering::Relaxed);
                let bytes = self
                    .traversal_engine
                    .stats()
                    .bytes_scanned
                    .load(Ordering::Relaxed);

                ui.label(format!("📁 Directories: {dir_count}"));
                ui.separator();
                ui.label(format!("📄 Files: {file_count}"));
                ui.separator();
                ui.label(format!(
                    "💾 Total Size: {}",
                    prettier_bytes::ByteFormatter::new().format(bytes as u64)
                ));

                if is_scanning && let Some(start) = self.scan_start_time {
                    let elapsed = start.elapsed();

                    #[allow(clippy::cast_precision_loss)]
                    let speed = bytes as f64 / elapsed.as_secs_f64();

                    ui.separator();
                    ui.label(format!("⏱ Time: {:.1}s", elapsed.as_secs_f64()));
                    ui.separator();
                    ui.label(format!(
                        "⚡ Speed: {}/s",
                        prettier_bytes::ByteFormatter::new().format(speed as u64)
                    ));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(idx) = self.selected_node_idx {
                        let size_str = prettier_bytes::ByteFormatter::new()
                            .format(snapshot.nodes[idx as usize].size)
                            .to_string();
                        ui.strong(size_str);
                        let path_str = snapshot.get_full_path(idx);
                        ui.label(format!("Selection: {path_str}"));
                    }
                });
            });
        });

        // Left Panel - Directory Tree Explorer
        if !self.left_panel_collapsed {
            egui::Panel::left("left_panel")
                .resizable(true)
                .default_size(300.0)
                .show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("🔍 Filter:");
                            let clear_btn_width = if self.search_query.is_empty() {
                                0.0
                            } else {
                                26.0
                            };
                            let desired_width = ui.available_width() - clear_btn_width - 8.0;
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .desired_width(desired_width.max(10.0)),
                            );
                            if !self.search_query.is_empty() && ui.button("❌").clicked() {
                                self.search_query.clear();
                            }
                        });
                        ui.add_space(4.0);
                        ui.separator();

                        if snapshot.nodes.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("Click 'Scan Directory' to explore disk usage.");
                            });
                        } else {
                            // Auto-expand the root node (0) if expanded_nodes is empty
                            if self.expanded_nodes.is_empty() {
                                self.expanded_nodes.insert(0);
                            }

                            let mut visible_nodes = Vec::new();
                            self.flatten_visible_tree(&snapshot, 0, 0, &mut visible_nodes);

                            // Fetch the exact layout spacing variables
                            let row_height = ui.spacing().interact_size.y;
                            let spacing_y = ui.spacing().item_spacing.y;
                            let row_stride = row_height + spacing_y; // Actual pixel gap per item index
                            let available_height = ui.available_height(); // Height of the left panel

                            // --- Mathematically Correct Programmatic Scrolling ---
                            let mut scroll_area = egui::ScrollArea::vertical();
                            if self.scroll_to_selected {
                                if let Some(selected_idx) = self.selected_node_idx {
                                    // Find the index of the selected item in the flat visible list
                                    if let Some(row_index) = visible_nodes
                                        .iter()
                                        .position(|&(node_idx, _)| node_idx == selected_idx)
                                    {
                                        #[allow(clippy::cast_precision_loss)]
                                        let target_y = (row_index as f32) * row_stride;

                                        // Calculate center offset relative to the available height of the viewport
                                        let center_offset = (available_height - row_height) / 2.0;
                                        let offset = (target_y - center_offset).max(0.0);

                                        scroll_area = scroll_area.vertical_scroll_offset(offset);
                                    }
                                }
                                self.scroll_to_selected = false; // Reset the scroll trigger
                            }

                            scroll_area.show_rows(
                                ui,
                                row_height,
                                visible_nodes.len(),
                                |ui, row_range| {
                                    for idx in row_range {
                                        let (node_idx, indent) = visible_nodes[idx];
                                        self.render_tree_node_row(ui, &snapshot, node_idx, indent);
                                    }
                                },
                            );
                        }
                    });
                });
        }

        // Right Panel - Extension statistics
        if !self.right_panel_collapsed {
            self.render_extension_panel(ui);
        }

        // Central Panel - Canvas visual Treemap / Plot Panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.vis_mode, VisMode::Treemap, "🗺 Treemap");
                    ui.selectable_value(&mut self.vis_mode, VisMode::Plots, "📈 Plots");
                    ui.selectable_value(
                        &mut self.vis_mode,
                        VisMode::Deduplicator,
                        "👥 Deduplicator",
                    );
                });

                ui.add_space(5.0);

                match self.vis_mode {
                    VisMode::Treemap => {
                        ui.heading(
                            egui::RichText::new("📊 Treemap Visualization")
                                .strong()
                                .color(ui.visuals().strong_text_color()),
                        );
                        ui.separator();

                        if snapshot.nodes.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    "Scanned filesystem will be visualized as a treemap here.",
                                );
                            });
                        } else {
                            let mut context = stats::StatContext {
                                selected_node_idx: &mut self.selected_node_idx,
                                expanded_nodes: &mut self.expanded_nodes,
                                scroll_to_selected: &mut self.scroll_to_selected,
                            };
                            self.treemap_chart.render(ui, &snapshot, &mut context);
                        }
                    }
                    VisMode::Plots => {
                        // Plots rendering block
                        ui.horizontal(|ui| {
                            ui.label("Select Plot:");
                            egui::ComboBox::from_id_salt("plot_type_combo")
                                .selected_text(match self.plot_type {
                                    PlotType::SizeDistribution => "📊 File Size Distribution",
                                    PlotType::AgeSizeScatter => "🌌 File Age vs. File Size",
                                    PlotType::DirComposition => "🍰 Directory Composition",
                                    PlotType::ExtensionBoxplot => "📦 File Sizes by Extension",
                                    PlotType::TemporalTimeline => "⏱ Linked Temporal Timelines",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.plot_type,
                                        PlotType::SizeDistribution,
                                        "📊 File Size Distribution",
                                    );
                                    ui.selectable_value(
                                        &mut self.plot_type,
                                        PlotType::AgeSizeScatter,
                                        "🌌 File Age vs. File Size",
                                    );
                                    ui.selectable_value(
                                        &mut self.plot_type,
                                        PlotType::DirComposition,
                                        "🍰 Directory Composition",
                                    );
                                    ui.selectable_value(
                                        &mut self.plot_type,
                                        PlotType::ExtensionBoxplot,
                                        "📦 File Sizes by Extension",
                                    );
                                    ui.selectable_value(
                                        &mut self.plot_type,
                                        PlotType::TemporalTimeline,
                                        "⏱ Linked Temporal Timelines",
                                    );
                                });
                        });
                        ui.separator();

                        if snapshot.nodes.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("Scanned filesystem will be plotted here.");
                            });
                        } else {
                            let mut context = stats::StatContext {
                                selected_node_idx: &mut self.selected_node_idx,
                                expanded_nodes: &mut self.expanded_nodes,
                                scroll_to_selected: &mut self.scroll_to_selected,
                            };
                            match self.plot_type {
                                PlotType::SizeDistribution => {
                                    self.size_dist_chart.render(ui, &snapshot, &mut context);
                                }
                                PlotType::AgeSizeScatter => {
                                    self.scatter_chart.render(ui, &snapshot, &mut context);
                                }
                                PlotType::DirComposition => {
                                    self.dir_comp_chart.render(ui, &snapshot, &mut context);
                                }
                                PlotType::ExtensionBoxplot => {
                                    self.boxplot_chart.render(ui, &snapshot, &mut context);
                                }
                                PlotType::TemporalTimeline => {
                                    self.timeline_chart.render(ui, &snapshot, &mut context);
                                }
                            }
                        }
                    }
                    VisMode::Deduplicator => {
                        self.render_deduplicator_tab(ui, &snapshot);
                    }
                }
            });
        });

        // Render any active modals
        self.render_modals(&ctx, &snapshot);
    }
}
