use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use eframe::egui;
use rfd::FileDialog;

use crate::arena::precompute_dir_counts;

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
pub mod operations;
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
    DeduplicatorWaste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Classic,
    WinDirStat,
}

#[allow(clippy::struct_excessive_bools)]
pub struct GuiApp {
    pub(crate) shared_state: Arc<SharedState>,
    pub(crate) traversal_engine: Arc<TraversalEngine>,

    // Unified egui-table-kit State
    pub(crate) table_state: egui_table_kit::state::TableState,

    // Command listener channels for decoupled operations
    pub(crate) command_rx: std::sync::mpsc::Receiver<crate::gui::operations::AppCommand>,

    // Unified TableOperations set
    pub(crate) operations: egui_table_kit::operations::TableOperations,

    // UI state
    pub(crate) selected_node_idx: Option<u32>,
    pub(crate) focus_node_idx: Option<u32>,
    pub(crate) delete_node_indices: Vec<u32>,
    pub(crate) search_query: String,
    pub(crate) monospace_paths: bool,
    pub(crate) left_panel_collapsed: bool,
    pub(crate) right_panel_collapsed: bool,

    pub(crate) filter_case_sensitive: bool,
    pub(crate) filter_regex: bool,

    // Caching layer for tree search matches
    pub(crate) query_coordinator: crate::gui::explorer::QueryCoordinator,

    // Visualization tabs
    pub(crate) vis_mode: VisMode,
    pub(crate) plot_type: PlotType,
    pub(crate) layout_mode: LayoutMode,

    // Analytics components
    pub(crate) treemap_chart: stats::treemap::TreemapChart,
    pub(crate) size_dist_chart: stats::size_distribution::SizeDistributionChart,
    pub(crate) scatter_chart: stats::scatter_plot::FileAgeSizeScatterChart,
    pub(crate) dir_comp_chart: stats::dir_composition::DirCompositionChart,
    pub(crate) boxplot_chart: stats::extension_boxplot::ExtensionBoxplotChart,
    pub(crate) timeline_chart: stats::temporal_timeline::TemporalTimelineChart,
    pub(crate) duplicate_waste_chart: stats::duplicate_waste::DuplicateWasteChart,

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
        Arc<parking_lot::RwLock<crate::stats::deduplicator::DeduplicationResults>>,
    pub(crate) deduplicator_cancel: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) selected_duplicates: HashSet<u32>,
    pub(crate) delete_duplicates_indices: Vec<u32>,
    pub(crate) deduplicator_dir_filter: String,

    pub(crate) highlight_duplicates: bool,
    pub(crate) hovered_node_idx: Option<u32>,
}

impl GuiApp {
    #[must_use]
    pub fn new(
        shared_state: Arc<SharedState>,
        traversal_engine: Arc<TraversalEngine>,
        initial_path: Option<PathBuf>,
    ) -> Self {
        // Initialize the command queue channels
        let (command_tx, command_rx) = std::sync::mpsc::channel();

        // Assemble our consolidated TableOperations collection
        let operations = egui_table_kit::operations::TableOperations::new()
            .with_group(vec![
                Box::new(crate::gui::operations::UpOneLevelOp::new(
                    shared_state.clone(),
                    command_tx.clone(),
                )),
                Box::new(crate::gui::operations::RefreshDirectoryOp::new(
                    shared_state.clone(),
                    command_tx.clone(),
                )),
            ])
            .with_group(vec![
                Box::new(crate::gui::operations::OpenFileManagerOp::new(
                    shared_state.clone(),
                )),
                Box::new(crate::gui::operations::OpenTerminalOp::new(
                    shared_state.clone(),
                )),
            ])
            .with_group(vec![
                Box::new(crate::gui::operations::CopyNameOp::new(
                    shared_state.clone(),
                )),
                Box::new(crate::gui::operations::CopyPathOp::new(
                    shared_state.clone(),
                )),
            ])
            .with_group(vec![
                Box::new(crate::gui::operations::TrashSelectedOp::new(
                    command_tx.clone(),
                )),
                Box::new(crate::gui::operations::DeleteSelectedOp::new(command_tx)),
            ]);

        let mut app = Self {
            shared_state,
            traversal_engine,
            table_state: egui_table_kit::state::TableState::new("edirstat_hierarchical_table", 0),
            command_rx,
            operations,
            selected_node_idx: None,
            focus_node_idx: None,
            delete_node_indices: Vec::new(),
            search_query: String::new(),
            monospace_paths: false,
            left_panel_collapsed: false,
            right_panel_collapsed: false,
            filter_case_sensitive: false,
            filter_regex: false,
            query_coordinator: crate::gui::explorer::QueryCoordinator::new(),
            vis_mode: VisMode::Treemap,
            plot_type: PlotType::SizeDistribution,
            layout_mode: LayoutMode::WinDirStat,
            treemap_chart: stats::treemap::TreemapChart::new(),
            size_dist_chart: stats::size_distribution::SizeDistributionChart::new(),
            scatter_chart: stats::scatter_plot::FileAgeSizeScatterChart::new(),
            dir_comp_chart: stats::dir_composition::DirCompositionChart::new(0),
            boxplot_chart: stats::extension_boxplot::ExtensionBoxplotChart::new(),
            timeline_chart: stats::temporal_timeline::TemporalTimelineChart::new(),
            duplicate_waste_chart: stats::duplicate_waste::DuplicateWasteChart::new(),
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
            deduplicator_results: Arc::new(parking_lot::RwLock::new(
                crate::stats::deduplicator::DeduplicationResults::default(),
            )),
            deduplicator_cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            selected_duplicates: HashSet::new(),
            delete_duplicates_indices: Vec::new(),
            deduplicator_dir_filter: String::new(),

            highlight_duplicates: false,
            hovered_node_idx: None,
        };

        if let Some(path) = initial_path {
            if path.exists() {
                if path.is_dir() {
                    app.start_scan(path);
                } else if path.is_file()
                    && let Err(e) = app.load_snapshot_file(path.clone())
                {
                    eprintln!("Error loading snapshot file {}: {}", path.display(), e);
                }
            } else {
                eprintln!("Error: Path does not exist: {}", path.display());
            }
        }

        app
    }

    fn reset_state(&mut self) {
        self.selected_node_idx = None;
        self.table_state.selected_rows.clear();
        self.table_state.expanded_rows.clear();
        self.table_state.active_rows.clear();
        self.table_state.sorted_children_cache.clear();
        self.focus_node_idx = None;
        self.delete_node_indices.clear();
        self.extension_stats.clear();
        self.last_extension_update = None;
        self.delete_confirm_checked = false;
        self.delete_node_idx = None;
        self.active_modal = None;
        self.selected_duplicates.clear();
        self.delete_duplicates_indices.clear();
        self.deduplicator_dir_filter.clear();
        self.scan_start_time = None;
        self.total_scan_duration = None;
        self.deduplicator_cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.deduplicator_progress = atomic_progress::Progress::new_spinner("Deduplicator");
        *self.deduplicator_results.write() =
            crate::stats::deduplicator::DeduplicationResults::default();
        self.query_coordinator = crate::gui::explorer::QueryCoordinator::new();
        self.traversal_engine.stats().reset();
        self.treemap_chart = stats::treemap::TreemapChart::default();
        self.size_dist_chart = stats::size_distribution::SizeDistributionChart::default();
        self.scatter_chart = stats::scatter_plot::FileAgeSizeScatterChart::default();
        self.dir_comp_chart = stats::dir_composition::DirCompositionChart::default();
        self.boxplot_chart = stats::extension_boxplot::ExtensionBoxplotChart::default();
        self.timeline_chart = stats::temporal_timeline::TemporalTimelineChart::default();
        self.duplicate_waste_chart = stats::duplicate_waste::DuplicateWasteChart::default();

        self.scroll_to_selected = false;
    }

    pub(crate) fn start_scan(&mut self, path: PathBuf) {
        self.reset_state();

        // Select the root row by default
        self.table_state.selected_rows.insert(0);
        self.selected_node_idx = Some(0);
        self.focus_node_idx = Some(0);

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
                let mut coordinator = crate::coordinator::Coordinator::new(rx, state);
                std::thread::spawn(move || {
                    coordinator.run_coordinator_loop(&path.to_string_lossy());
                });
            }
            Err(e) => {
                println!("Failed to start traversal: {e}");
            }
        }
    }

    pub fn load_snapshot_file(&mut self, path: PathBuf) -> Result<(), crate::EdirstatError> {
        let (arena, string_pool) = load_snapshot(&path)?;
        self.reset_state();

        // Select the root row by default
        self.table_state.selected_rows.insert(0);
        self.selected_node_idx = Some(0);
        self.focus_node_idx = Some(0);

        let loaded_snapshot = FileArenaSnapshot {
            nodes: Arc::new(arena.nodes().to_vec()),
            string_pool: Arc::new(string_pool),
            dir_counts: Arc::new(precompute_dir_counts(arena.nodes())),
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
            .map(|(ext, (total_size, file_count))| (ext, total_size, file_count))
            .collect();
        stats.sort_by_key(|b| std::cmp::Reverse(b.1));
        self.shared_state.extension_stats.store(Arc::new(stats));

        Ok(())
    }

    /// Delegates render operations entirely to our registered `TableOperations` suite.
    pub(crate) fn draw_file_menu_contents(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        let provider = crate::gui::explorer::TableProviderWrapper::new(snapshot);
        let _ = self
            .operations
            .gui(ui, &provider, &mut self.table_state, true);
    }

    /// Renders a unified top row controls bar inside visualizer panel viewports.
    pub(crate) fn draw_central_panel_header(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        ui.horizontal(|ui| {
            // Left side: Active mode title or layout controls
            match self.vis_mode {
                VisMode::Treemap => {
                    ui.heading(
                        egui::RichText::new("📊 Treemap")
                            .strong()
                            .color(ui.visuals().strong_text_color()),
                    );
                }
                VisMode::Plots => {
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("📈 Plots")
                                .strong()
                                .color(ui.visuals().strong_text_color()),
                        );
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label("Select Plot:");

                        let plot_combo_id = if self.layout_mode == LayoutMode::Classic {
                            "plot_type_combo"
                        } else {
                            "plot_type_combo_windirstat"
                        };

                        egui::ComboBox::from_id_salt(plot_combo_id)
                            .selected_text(match self.plot_type {
                                PlotType::SizeDistribution => "📊 File Size Distribution",
                                PlotType::AgeSizeScatter => "🌌 File Age vs. File Size",
                                PlotType::DirComposition => "🍰 Directory Composition",
                                PlotType::ExtensionBoxplot => "📦 File Sizes by Extension",
                                PlotType::TemporalTimeline => "⏱ Linked Temporal Timelines",
                                PlotType::DeduplicatorWaste => "👥 Duplicate Waste by Extension",
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
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::DeduplicatorWaste,
                                    "👥 Duplicate Waste by Extension",
                                );
                            });
                    });
                }
                VisMode::Deduplicator => {
                    ui.heading(
                        egui::RichText::new("👥 Duplicate File Finder")
                            .strong()
                            .color(ui.visuals().strong_text_color()),
                    );

                    // Determine if any duplicate group is fully selected (meaning the original and all copies are selected)
                    let mut fully_selected_groups_info = Vec::new();
                    {
                        let guard = self.deduplicator_results.read();
                        for group in &guard.groups {
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
                    }

                    if !fully_selected_groups_info.is_empty() {
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));

                        let time = ui.input(|i| i.time);
                        #[allow(clippy::cast_possible_truncation)]
                        let pulse = 0.5f64.mul_add((time * 6.0).sin(), 0.5) as f32;
                        let alpha = 0.6f32.mul_add(pulse, 0.4);
                        let warning_red = theme::WARNING_RED;
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
                                    .color(theme::WARNING_RED)
                                    .strong()
                            );
                            ui.label("You have checked both the original and all duplicate copies for the files listed below. Deleting them will likely result in permanent data loss:");
                            ui.separator();

                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for (filename, nodes) in &fully_selected_groups_info {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(theme::WARNING_RED, "🔥");
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
                }
            }

            // Right side: Active Visualizer Modes
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.selectable_value(&mut self.vis_mode, VisMode::Deduplicator, "👥 Deduplicator");
                ui.selectable_value(&mut self.vis_mode, VisMode::Plots, "📈 Plots");
                ui.selectable_value(&mut self.vis_mode, VisMode::Treemap, "🗺 Treemap");
            });
        });
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle keyboard shortcuts
        if self.layout_mode == LayoutMode::Classic && ctx.input(|i| i.key_pressed(egui::Key::F9)) {
            self.left_panel_collapsed = !self.left_panel_collapsed;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.right_panel_collapsed = !self.right_panel_collapsed;
        }

        // Fetch current snapshot
        let snapshot = self.shared_state.current_snapshot.load();
        let is_scanning = self.shared_state.is_scanning.load(Ordering::SeqCst);

        // --- Handle Table commands sent from standard and context-menu operations ---
        while let Ok(command) = self.command_rx.try_recv() {
            match command {
                crate::gui::operations::AppCommand::ScrollToSelected => {
                    self.scroll_to_selected = true;
                }
                crate::gui::operations::AppCommand::RefreshSubtrees(dirs) => {
                    self.refresh_directory_subtrees(&dirs);
                }
                crate::gui::operations::AppCommand::ShowTrashModal(nodes) => {
                    // Corrected variant
                    self.delete_node_indices = nodes;
                    self.active_modal = Some(ActiveModal::Trash);
                    self.delete_confirm_checked = false; // Renders custom confirmation checkbox in your dialog!
                }
                crate::gui::operations::AppCommand::ShowDeleteModal(nodes) => {
                    // Corrected variant
                    self.delete_node_indices = nodes;
                    self.active_modal = Some(ActiveModal::Delete);
                    self.delete_confirm_checked = false; // Renders custom confirmation checkbox in your dialog!
                }
            }
        }

        // Background Modal polling processing for custom TableOperations
        for op_group in &mut self.operations.0 {
            for op in op_group {
                if op.is_modal_open() {
                    let _ = op.poll(ui, &mut self.table_state);
                }
            }
        }

        if !is_scanning && let Some(start) = self.scan_start_time {
            self.total_scan_duration = Some(start.elapsed());
            self.scan_start_time = None;
        }

        // Repaint during scan to show live progress, or continuously while selected to drive the glow animation
        if is_scanning {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else if !self.table_state.selected_rows.is_empty() {
            ctx.request_repaint();
        } else if snapshot.nodes.is_empty() {
            // Animating the Scan Directory button when no scan is active and no snapshot is open
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        // Apply dark style
        theme::setup_custom_style(&ctx);

        // Top Control Panel
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("eDirStat")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/img/icon-transparent.svg"))
                        .max_height(24.0)
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
                            ui.label(
                                egui::RichText::new("Monospace Paths")
                                    .color(ui.visuals().widgets.inactive.text_color()),
                            );
                        }).response;

                        let label_click = ui.interact(response.rect, ui.id().with("monospace_label"), egui::Sense::click());
                        if label_click.clicked() {
                            self.monospace_paths = !self.monospace_paths;
                        }
                    });

                    ui.checkbox(&mut self.highlight_duplicates, "✨ Highlight Duplicates");  

                    ui.separator();
                    ui.label("Layout Mode:");
                    ui.radio_value(&mut self.layout_mode, LayoutMode::Classic, "Classic Layout");
                    ui.radio_value(&mut self.layout_mode, LayoutMode::WinDirStat, "WinDirStat Layout");

                    ui.separator();

                    let is_classic = self.layout_mode == LayoutMode::Classic;
                    if is_classic {
                        let left_label = if self.left_panel_collapsed { "▶ Show Left Panel (F9)" } else { "◀ Hide Left Panel (F9)" };
                        if ui.button(left_label).clicked() {
                            self.left_panel_collapsed = !self.left_panel_collapsed;
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    }

                    let right_label = if self.right_panel_collapsed {
                        if is_classic { "◀ Show Right Panel (F11)" } else { "▶ Show Extensions Panel (F11)" }
                    } else {
                        if is_classic { "▶ Hide Right Panel (F11)" } else { "◀ Hide Extensions Panel (F11)" }
                    };
                    if ui.button(right_label).clicked() {
                        self.right_panel_collapsed = !self.right_panel_collapsed;
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    ui.separator();
                    if ui.button("⏏ Collapse All").clicked() {
                        self.table_state.expanded_rows.clear();
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

                let should_pulse = !is_scanning && snapshot.nodes.is_empty();
                let scan_btn = if should_pulse {
                    let time = ui.input(|i| i.time);
                    #[allow(clippy::cast_possible_truncation)]
                    let pulse = 0.5f64.mul_add((time * 3.0).sin(), 0.5) as f32; // gentle pulsing between 0.0 and 1.0

                    // Pulsing background and border with theme's scanning color
                    let fill_color = theme::COLOR_SCANNING.linear_multiply(pulse * 0.12 + 0.04);
                    let border_color = theme::COLOR_SCANNING.linear_multiply(pulse * 0.35 + 0.15);
                    let text_color = theme::COLOR_WHITE.linear_multiply(pulse * 0.15 + 0.85);

                    ui.scope(|ui| {
                        ui.style_mut().visuals.button_frame = true;

                        // Inactive state (pulsing)
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = fill_color;
                        ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_color);
                        ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_color);

                        // Hovered state (bright purple highlight)
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = theme::COLOR_SCANNING.linear_multiply(0.25);
                        ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme::COLOR_SCANNING);
                        ui.style_mut().visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, theme::COLOR_WHITE);

                        // Active state (clicked)
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = theme::COLOR_SCANNING.linear_multiply(0.35);
                        ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme::COLOR_SCANNING);
                        ui.style_mut().visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, theme::COLOR_WHITE);

                        ui.button(egui::RichText::new("📁 Scan Directory").strong())
                    }).inner
                } else {
                    ui.button("📁 Scan Directory")
                };

                if scan_btn.clicked() {
                    let folder_opt = FileDialog::new().pick_folder();
                    if let Some(path) = folder_opt {
                        self.start_scan(path);
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
                    if let Some(path) = file_opt && let Err(e) = self.load_snapshot_file(path) {
                        println!("Failed to load snapshot: {e}");
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
                } else if !is_scanning && let Some(duration) = self.total_scan_duration {
                    ui.separator();
                    ui.label(format!("⏱ Time: {:.1}s", duration.as_secs_f64()));
                    ui.separator();
                    #[allow(clippy::cast_precision_loss)]
                    let speed = if duration.as_secs_f64() > 0.0 {
                        bytes as f64 / duration.as_secs_f64()
                    } else {
                        0.0
                    };
                    ui.label(format!(
                        "⚡ Speed: {}/s",
                        prettier_bytes::ByteFormatter::new().format(speed as u64)
                    ));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.table_state.selected_rows.len() == 1 {
                        if let Some(idx) = self.table_state.selected_rows.iter().next()
                            && (idx as usize) < snapshot.nodes.len()
                        {
                            let size_str = prettier_bytes::ByteFormatter::new()
                                .format(snapshot.nodes[idx as usize].size)
                                .to_string();
                            ui.strong(size_str);
                            let path_str = snapshot.get_full_path(idx);
                            ui.label(format!("Selection: {path_str}"));
                        }
                    } else if !self.table_state.selected_rows.is_empty() {
                        let total_size: u64 = self
                            .table_state
                            .selected_rows
                            .iter()
                            .map(|idx| snapshot.nodes[idx as usize].size)
                            .sum();
                        let size_str = prettier_bytes::ByteFormatter::new()
                            .format(total_size)
                            .to_string();
                        ui.strong(size_str);
                        ui.label(format!(
                            "Selection: {} items",
                            self.table_state.selected_rows.len()
                        ));
                    }
                });
            });
        });

        if self.layout_mode == LayoutMode::Classic {
            // Left Panel - Directory Tree Explorer
            if !self.left_panel_collapsed {
                egui::Panel::left("left_panel")
                    .resizable(true)
                    .default_size(300.0)
                    .show_inside(ui, |ui| {
                        self.render_classic_left_panel(ui, &snapshot);
                    });
            }

            // Right Panel - Extension statistics
            if !self.right_panel_collapsed {
                self.render_extension_panel(ui);
            }

            // Central Panel - Canvas visual Treemap / Plot Panel
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.render_classic_central_panel(ui, &snapshot);
            });
        } else {
            self.render_windirstat_layout(ui, &snapshot);
        }

        // Render any active modals
        self.render_modals(&ctx, &snapshot);
    }
}

impl GuiApp {
    fn render_classic_left_panel(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.strong("🔍 Filter:");

                // Lay out control elements from right-to-left to prevent layout feedback loops
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 1. Far right: Regular Expression matching (.*)
                    let reg = self.filter_regex;
                    let reg_btn = ui
                        .selectable_label(reg, egui::RichText::new(".*").strong())
                        .on_hover_text("Use Regular Expression (Regex)");
                    if reg_btn.clicked() {
                        self.filter_regex = !reg;
                    }

                    // 2. Middle-right: Match Case Sensitivity (Aa)
                    let case_sens = self.filter_case_sensitive;
                    let case_btn = ui
                        .selectable_label(case_sens, egui::RichText::new("Aa").strong())
                        .on_hover_text("Match Case (Case Sensitive)");
                    if case_btn.clicked() {
                        self.filter_case_sensitive = !case_sens;
                    }

                    // 3. Clear button
                    if !self.search_query.is_empty() && ui.button("❌").clicked() {
                        self.search_query.clear();
                    }

                    // 4. Remaining middle-left: TextEdit box (safe, non-recursive width assignment)
                    let remaining_width = ui.available_width();
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .id_salt("filter_text_edit")
                            .desired_width(remaining_width.max(10.0)),
                    );
                });
            });
            ui.add_space(4.0);
            ui.separator();

            if snapshot.nodes.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Click 'Scan Directory' to explore disk usage.");
                });
            } else {
                // Auto-expand the root node (0) if expanded_rows is empty
                if self.table_state.expanded_rows.is_empty() {
                    self.table_state.expanded_rows.insert(0);
                }

                let mut visible_nodes = Vec::new();
                self.flatten_visible_tree(snapshot, 0, 0, &mut visible_nodes);

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

                scroll_area.show_rows(ui, row_height, visible_nodes.len(), |ui, row_range| {
                    for idx in row_range {
                        let (node_idx, indent) = visible_nodes[idx];
                        self.render_tree_node_row(ui, snapshot, node_idx, indent);
                    }
                });
            }
        });
    }

    fn render_classic_central_panel(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        ui.vertical(|ui| {
            self.draw_central_panel_header(ui, snapshot);
            ui.separator();

            match self.vis_mode {
                VisMode::Treemap => {
                    if snapshot.nodes.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("Scanned filesystem will be visualized as a treemap here.");
                        });
                    } else {
                        // Gather temporary HashSets compatible with the treemap API
                        let mut selected_nodes_set: HashSet<u32> =
                            self.table_state.selected_rows.iter().collect();
                        let mut expanded_nodes_set: HashSet<u32> =
                            self.table_state.expanded_rows.iter().collect();

                        let mut context = stats::StatContext {
                            selected_nodes: &mut selected_nodes_set,
                            expanded_nodes: &mut expanded_nodes_set,
                            scroll_to_selected: &mut self.scroll_to_selected,
                            deduplicator_results: Some(&self.deduplicator_results),
                        };
                        self.treemap_chart.render(ui, snapshot, &mut context);

                        // Sync selections
                        if selected_nodes_set.len() != self.table_state.selected_rows.len() as usize
                        {
                            self.table_state.selected_rows.clear();
                            self.table_state
                                .selected_rows
                                .extend(selected_nodes_set.iter());
                            if self.table_state.selected_rows.len() == 1 {
                                self.selected_node_idx =
                                    self.table_state.selected_rows.iter().next();
                            } else {
                                self.selected_node_idx = None;
                            }
                        }
                        // Sync expansions
                        if expanded_nodes_set.len() != self.table_state.expanded_rows.len() as usize
                        {
                            self.table_state.expanded_rows.clear();
                            self.table_state
                                .expanded_rows
                                .extend(expanded_nodes_set.iter());
                        }
                    }
                }
                VisMode::Plots => {
                    ui.add_space(8.0);
                    if snapshot.nodes.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("Scanned filesystem will be plotted here.");
                        });
                    } else {
                        let mut selected_nodes_set: HashSet<u32> =
                            self.table_state.selected_rows.iter().collect();
                        let mut expanded_nodes_set: HashSet<u32> =
                            self.table_state.expanded_rows.iter().collect();

                        let mut context = stats::StatContext {
                            selected_nodes: &mut selected_nodes_set,
                            expanded_nodes: &mut expanded_nodes_set,
                            scroll_to_selected: &mut self.scroll_to_selected,
                            deduplicator_results: Some(&self.deduplicator_results),
                        };
                        match self.plot_type {
                            PlotType::SizeDistribution => {
                                self.size_dist_chart.render(ui, snapshot, &mut context);
                            }
                            PlotType::AgeSizeScatter => {
                                self.scatter_chart.render(ui, snapshot, &mut context);
                            }
                            PlotType::DirComposition => {
                                self.dir_comp_chart.render(ui, snapshot, &mut context);
                            }
                            PlotType::ExtensionBoxplot => {
                                self.boxplot_chart.render(ui, snapshot, &mut context);
                            }
                            PlotType::TemporalTimeline => {
                                self.timeline_chart.render(ui, snapshot, &mut context);
                            }
                            PlotType::DeduplicatorWaste => {
                                self.duplicate_waste_chart
                                    .render(ui, snapshot, &mut context);
                            }
                        }

                        // Sync selections
                        if selected_nodes_set.len() != self.table_state.selected_rows.len() as usize
                        {
                            self.table_state.selected_rows.clear();
                            self.table_state
                                .selected_rows
                                .extend(selected_nodes_set.iter());
                            if self.table_state.selected_rows.len() == 1 {
                                self.selected_node_idx =
                                    self.table_state.selected_rows.iter().next();
                            } else {
                                self.selected_node_idx = None;
                            }
                        }
                        // Sync expansions
                        if expanded_nodes_set.len() != self.table_state.expanded_rows.len() as usize
                        {
                            self.table_state.expanded_rows.clear();
                            self.table_state
                                .expanded_rows
                                .extend(expanded_nodes_set.iter());
                        }
                    }
                }
                VisMode::Deduplicator => {
                    self.render_deduplicator_tab(ui, snapshot);
                }
            }
        });
    }

    fn render_windirstat_layout(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
        // We want a top panel and a bottom panel (Central Panel space).
        egui::Panel::top("windirstat_top_panel")
            .resizable(true)
            .default_size(380.0)
            .size_range(150.0..=600.0)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // Render operations directly as a flat row of toolbar buttons
                    ui.spacing_mut().item_spacing.x = 8.0;

                    let provider = crate::gui::explorer::TableProviderWrapper::new(snapshot);
                    let _ = self
                        .operations
                        .gui(ui, &provider, &mut self.table_state, false);

                    // Separator between operations and the search/filter box
                    ui.separator();

                    // Filter search input
                    ui.label("🔍 Filter:");
                    if !self.search_query.is_empty() && ui.button("❌").clicked() {
                        self.search_query.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let reg = self.filter_regex;
                        let reg_btn = ui
                            .selectable_label(reg, egui::RichText::new(".*").strong())
                            .on_hover_text("Use Regular Expression (Regex)");
                        if reg_btn.clicked() {
                            self.filter_regex = !reg;
                        }

                        let case_sens = self.filter_case_sensitive;
                        let case_btn = ui
                            .selectable_label(case_sens, egui::RichText::new("Aa").strong())
                            .on_hover_text("Match Case (Case Sensitive)");
                        if case_btn.clicked() {
                            self.filter_case_sensitive = !case_sens;
                        }

                        let text_width = ui.available_width() - 8.0;
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .id_salt("windirstat_filter_text_edit")
                                .desired_width(text_width.max(50.0)),
                        );
                    });
                });
                ui.separator();

                // If selection exists, pop out detail panel on the right of the top section
                if !self.table_state.selected_rows.is_empty() {
                    egui::Panel::right("windirstat_detail_panel")
                        .resizable(true)
                        .default_size(260.0)
                        .size_range(160.0..=450.0)
                        .show_inside(ui, |ui| {
                            if self.table_state.selected_rows.len() == 1 {
                                if let Some(selected_idx) =
                                    self.table_state.selected_rows.iter().next()
                                {
                                    self.render_file_detail_list(ui, snapshot, selected_idx);
                                }
                            } else {
                                self.render_multi_file_detail_list(ui, snapshot);
                            }
                        });
                }

                // The rest is the table view
                let mut frame = egui::Frame::central_panel(ui.style());
                frame.inner_margin.top = 2; // Shrink top padding above the table
                egui::CentralPanel::default()
                    .frame(frame)
                    .show_inside(ui, |ui| {
                        self.render_hierarchical_table(ui, snapshot);
                    });
            });

        // The remaining area becomes the bottom section
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                self.draw_central_panel_header(ui, snapshot);
                ui.separator();

                // Right Extensions Panel inside the bottom section if not collapsed
                if !self.right_panel_collapsed && self.vis_mode != VisMode::Deduplicator {
                    egui::Panel::right("windirstat_extensions_panel")
                        .resizable(true)
                        .default_size(210.0)
                        .size_range(80.0..=250.0)
                        .show_inside(ui, |ui| {
                            self.draw_extensions_contents(ui);
                        });
                }

                // Rest of the space for visualizers (rendered directly without nested CentralPanel to avoid vertical spacing gaps)
                match self.vis_mode {
                    VisMode::Treemap => {
                        if snapshot.nodes.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    "Scanned filesystem will be visualized as a treemap here.",
                                );
                            });
                        } else {
                            // Gather temporary HashSets compatible with the treemap API
                            let mut selected_nodes_set: HashSet<u32> =
                                self.table_state.selected_rows.iter().collect();
                            let mut expanded_nodes_set: HashSet<u32> =
                                self.table_state.expanded_rows.iter().collect();

                            let mut context = stats::StatContext {
                                selected_nodes: &mut selected_nodes_set,
                                expanded_nodes: &mut expanded_nodes_set,
                                scroll_to_selected: &mut self.scroll_to_selected,
                                deduplicator_results: Some(&self.deduplicator_results),
                            };
                            self.treemap_chart.render(ui, snapshot, &mut context);

                            // Sync selections
                            if selected_nodes_set.len()
                                != self.table_state.selected_rows.len() as usize
                            {
                                self.table_state.selected_rows.clear();
                                self.table_state
                                    .selected_rows
                                    .extend(selected_nodes_set.iter());
                                if self.table_state.selected_rows.len() == 1 {
                                    self.selected_node_idx =
                                        self.table_state.selected_rows.iter().next();
                                } else {
                                    self.selected_node_idx = None;
                                }
                            }
                            // Sync expansions
                            if expanded_nodes_set.len()
                                != self.table_state.expanded_rows.len() as usize
                            {
                                self.table_state.expanded_rows.clear();
                                self.table_state
                                    .expanded_rows
                                    .extend(expanded_nodes_set.iter());
                            }
                        }
                    }
                    VisMode::Plots => {
                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 6,
                                right: 6,
                                top: 6,
                                bottom: 0,
                            })
                            .show(ui, |ui| {
                                if snapshot.nodes.is_empty() {
                                    ui.centered_and_justified(|ui| {
                                        ui.label("Scanned filesystem will be plotted here.");
                                    });
                                } else {
                                    let mut selected_nodes_set: HashSet<u32> =
                                        self.table_state.selected_rows.iter().collect();
                                    let mut expanded_nodes_set: HashSet<u32> =
                                        self.table_state.expanded_rows.iter().collect();

                                    let mut context = stats::StatContext {
                                        selected_nodes: &mut selected_nodes_set,
                                        expanded_nodes: &mut expanded_nodes_set,
                                        scroll_to_selected: &mut self.scroll_to_selected,
                                        deduplicator_results: Some(&self.deduplicator_results),
                                    };
                                    match self.plot_type {
                                        PlotType::SizeDistribution => {
                                            self.size_dist_chart.render(ui, snapshot, &mut context);
                                        }
                                        PlotType::AgeSizeScatter => {
                                            self.scatter_chart.render(ui, snapshot, &mut context);
                                        }
                                        PlotType::DirComposition => {
                                            self.dir_comp_chart.render(ui, snapshot, &mut context);
                                        }
                                        PlotType::ExtensionBoxplot => {
                                            self.boxplot_chart.render(ui, snapshot, &mut context);
                                        }
                                        PlotType::TemporalTimeline => {
                                            self.timeline_chart.render(ui, snapshot, &mut context);
                                        }
                                        PlotType::DeduplicatorWaste => {
                                            self.duplicate_waste_chart.render(
                                                ui,
                                                snapshot,
                                                &mut context,
                                            );
                                        }
                                    }

                                    // Sync selections
                                    if selected_nodes_set.len()
                                        != self.table_state.selected_rows.len() as usize
                                    {
                                        self.table_state.selected_rows.clear();
                                        self.table_state
                                            .selected_rows
                                            .extend(selected_nodes_set.iter());
                                        if self.table_state.selected_rows.len() == 1 {
                                            self.selected_node_idx =
                                                self.table_state.selected_rows.iter().next();
                                        } else {
                                            self.selected_node_idx = None;
                                        }
                                    }
                                    // Sync expansions
                                    if expanded_nodes_set.len()
                                        != self.table_state.expanded_rows.len() as usize
                                    {
                                        self.table_state.expanded_rows.clear();
                                        self.table_state
                                            .expanded_rows
                                            .extend(expanded_nodes_set.iter());
                                    }
                                }
                            });
                    }
                    VisMode::Deduplicator => {
                        self.render_deduplicator_tab(ui, snapshot);
                    }
                }
            });
        });
    }
}

fn open_terminal_at(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "cmd"])
            .current_dir(path)
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(path)
            .spawn()?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let emulators = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "kitty",
            "alacritty",
            "xterm",
        ];
        let mut last_err = None;
        for &emulator in &emulators {
            let mut cmd = std::process::Command::new(emulator);
            if emulator == "gnome-terminal" {
                cmd.arg(format!("--working-directory={}", path.display()));
            } else {
                cmd.current_dir(path);
            }
            match cmd.spawn() {
                Ok(_) => return Ok(()),
                Err(e) => last_err = Some(e),
            }
        }
        last_err.map_or_else(
            || {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No terminal emulator found",
                ))
            },
            Err,
        )
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported platform",
        ))
    }
}
