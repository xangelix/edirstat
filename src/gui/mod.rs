use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    str::FromStr as _,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use compact_str::{CompactString, ToCompactString as _};
use eframe::egui;
use fluent_zero::t;
use rfd::FileDialog;
use strum::IntoEnumIterator as _;

use super::{
    arena::FileArenaSnapshot,
    coordinator::SharedState,
    persistence::snapshot::{load_snapshot, save_snapshot},
    stats::{self, StatComponent as _},
    traversal::TraversalEngine,
};
use crate::arena::precompute_dir_counts;

pub mod deduplicator;
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
    pub(crate) focus_node_idx: Option<u32>,
    pub(crate) delete_node_indices: Vec<u32>,
    pub(crate) search_query: String,
    pub(crate) monospace_paths: bool,
    pub(crate) left_panel_collapsed: bool,
    pub(crate) right_panel_collapsed: bool,

    pub(crate) filter_case_sensitive: bool,
    pub(crate) filter_regex: bool,
    pub(crate) time_format: crate::model::time_utils::TimeFormat,

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
    pub(crate) show_licenses: bool,

    // Saved scan parameters
    pub(crate) current_scan_path: Option<PathBuf>,
    pub(crate) scan_start_time: Option<Instant>,
    pub(crate) total_scan_duration: Option<Duration>,

    // Extension breakdown stats
    pub(crate) extension_stats: Vec<ExtensionStat>,
    pub(crate) last_extension_update: Option<Instant>,

    /// Tracked preference delta to safely batch config saves
    pub(crate) last_saved_preferences: crate::model::persistence::preferences::UserPreferences,

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

    // Defer initial CLI scan to the first render frame
    pub(crate) pending_initial_path: Option<PathBuf>,

    pub(crate) highlight_duplicates: bool,
    pub(crate) deletion_confirmation: bool,
    pub(crate) trash_confirmation: bool,
    pub(crate) remember_confirmation: bool,
    pub(crate) hovered_node_idx: Option<u32>,
    pub(crate) last_rendered_snapshot_ptr: usize,
    pub(crate) last_extension_stats_ptr: usize,

    /// Caches (Node Index, User String, Group String, Permissions String)
    pub(crate) unix_metadata_cache: Option<(u32, String, String, String)>,

    pub(crate) locale: Locale,

    #[cfg(feature = "online")]
    pub(crate) update_checker: egui_async::Bind<Option<String>, String>,
}

#[derive(Default, PartialEq, strum::EnumIter)]
pub(crate) enum Locale {
    #[default]
    EnUs,
    EsEs,
    DeDe,
    NlNl,
    FrFr,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnUs => write!(f, "en-US"),
            Self::EsEs => write!(f, "es-ES"),
            Self::DeDe => write!(f, "de-DE"),
            Self::NlNl => write!(f, "nl-NL"),
            Self::FrFr => write!(f, "fr-FR"),
        }
    }
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
                Box::new(crate::gui::operations::RefreshRootOp::new(
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

        #[cfg(not(test))]
        let pending_initial_path = initial_path;
        #[cfg(test)]
        let pending_initial_path = None;

        #[cfg(target_os = "windows")]
        let active_modal = if cli_or_gui::is_elevated() {
            None
        } else {
            Some(ActiveModal::AdminWarning)
        };

        #[cfg(not(target_os = "windows"))]
        let active_modal = None;

        let prefs = crate::model::persistence::preferences::load_preferences();

        let app = Self {
            shared_state,
            traversal_engine,
            table_state: egui_table_kit::state::TableState::new("edirstat_hierarchical_table", 0),
            command_rx,
            operations,
            focus_node_idx: None,
            delete_node_indices: Vec::new(),
            search_query: String::new(),
            monospace_paths: prefs.monospace_paths,
            left_panel_collapsed: false,
            right_panel_collapsed: false,
            filter_case_sensitive: false,
            filter_regex: false,
            time_format: prefs.time_format.clone(),
            last_saved_preferences: prefs.clone(),
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
            active_modal,
            show_licenses: false,
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

            pending_initial_path,

            highlight_duplicates: false,
            deletion_confirmation: prefs.deletion_confirmation,
            trash_confirmation: prefs.trash_confirmation,
            remember_confirmation: false,
            hovered_node_idx: None,
            last_rendered_snapshot_ptr: 0,
            last_extension_stats_ptr: 0,

            unix_metadata_cache: None,

            locale: Locale::default(),

            #[cfg(feature = "online")]
            update_checker: egui_async::Bind::default(),
        };

        #[cfg(test)]
        let mut app = app;

        // Immediate execution for unit tests only
        #[cfg(test)]
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
        self.show_licenses = false;
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
        self.last_rendered_snapshot_ptr = 0;
        self.last_extension_stats_ptr = 0;

        self.unix_metadata_cache = None;

        #[cfg(feature = "online")]
        self.update_checker.clear();
    }

    /// Safely retrieves the single selected node index (if exactly one is selected)
    #[must_use]
    #[inline]
    pub fn selected_node_idx(&self) -> Option<u32> {
        if self.table_state.selected_rows.len() == 1 {
            self.table_state.selected_rows.iter().next()
        } else {
            None
        }
    }

    pub(crate) fn start_scan(&mut self, mut path: PathBuf) {
        if let Ok(abs_path) = std::fs::canonicalize(&path) {
            path = abs_path;
        }

        self.reset_state();

        // Select the root row by default
        self.table_state.selected_rows.insert(0);
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
        self.focus_node_idx = Some(0);

        // Keep the nodes in the memory map zero-copy
        let loaded_snapshot = FileArenaSnapshot {
            dir_counts: Arc::new(precompute_dir_counts(arena.nodes())),
            nodes: Arc::new(crate::arena::NodeStorage::Mmapped(arena)),
            string_pool: Arc::new(string_pool),
        };
        self.shared_state
            .current_snapshot
            .store(Arc::new(loaded_snapshot));
        self.current_scan_path = Some(path);
        self.scan_start_time = None;

        // Rebuild extension stats and accumulate total stats in a single pass
        let mut ext_map: HashMap<CompactString, (u64, u32), ahash::RandomState> =
            HashMap::with_hasher(ahash::RandomState::new());

        let mut total_files = 0;
        let mut total_dirs = 0;
        let mut total_bytes = 0u64;

        let snapshot = self.shared_state.current_snapshot.load();
        for node in snapshot.nodes.iter() {
            if node.is_directory() {
                total_dirs += 1;
                continue;
            }

            total_files += 1;
            total_bytes += node.size;

            if let Some(name) = snapshot.string_pool.get(node.name_id) {
                let ext_slice = super::arena::get_ext_slice(name);
                super::arena::with_lowercase_ext(ext_slice, |ext_lowercased| {
                    let ext = CompactString::new(ext_lowercased);
                    let entry = ext_map.entry(ext).or_insert((0, 0));
                    entry.0 += node.size;
                    entry.1 += 1;
                });
            }
        }

        // Update the traversal engine stats so the bottom status bar displays the totals
        self.traversal_engine
            .stats()
            .files_scanned
            .store(total_files, Ordering::SeqCst);
        self.traversal_engine
            .stats()
            .dirs_scanned
            .store(total_dirs, Ordering::SeqCst);
        self.traversal_engine
            .stats()
            .bytes_scanned
            .store(total_bytes as usize, Ordering::SeqCst);

        let mut stats: Vec<(CompactString, u64, u32)> = ext_map
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
        let provider =
            crate::gui::explorer::TableProviderWrapper::new(snapshot, self.time_format.clone());
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
                        egui::RichText::new(t!("vis-mode-treemap"))
                            .strong()
                            .color(ui.visuals().strong_text_color()),
                    );
                }
                VisMode::Plots => {
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new(t!("vis-mode-plots"))
                                .strong()
                                .color(ui.visuals().strong_text_color()),
                        );
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(t!("select-plot-label"));

                        let plot_combo_id = if self.layout_mode == LayoutMode::Classic {
                            "plot_type_combo"
                        } else {
                            "plot_type_combo_windirstat"
                        };

                        egui::ComboBox::from_id_salt(plot_combo_id)
                            .selected_text(match self.plot_type {
                                PlotType::SizeDistribution => t!("plot-size-distribution"),
                                PlotType::AgeSizeScatter => t!("plot-age-size"),
                                PlotType::DirComposition => t!("plot-dir-composition"),
                                PlotType::ExtensionBoxplot => t!("plot-extension-boxplot"),
                                PlotType::TemporalTimeline => t!("plot-temporal-timeline"),
                                PlotType::DeduplicatorWaste => t!("plot-deduplicator-waste"),
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::SizeDistribution,
                                    t!("plot-size-distribution"),
                                );
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::AgeSizeScatter,
                                    t!("plot-age-size"),
                                );
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::DirComposition,
                                    t!("plot-dir-composition"),
                                );
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::ExtensionBoxplot,
                                    t!("plot-extension-boxplot"),
                                );
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::TemporalTimeline,
                                    t!("plot-temporal-timeline"),
                                );
                                ui.selectable_value(
                                    &mut self.plot_type,
                                    PlotType::DeduplicatorWaste,
                                    t!("plot-deduplicator-waste"),
                                );
                            });
                    });
                }
                VisMode::Deduplicator => {
                    ui.heading(
                        egui::RichText::new(t!("vis-mode-deduplicator"))
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
                            .stroke(egui::Stroke::new(1.0f32, warning_red.linear_multiply(alpha * 0.4)))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .corner_radius(4.0);

                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(t!("dedup-warning-title")).strong().color(text_color));
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(t!(
                                        "dedup-warning-desc",
                                        { "count" => fully_selected_groups_info.len() }
                                    ))
                                    .color(ui.visuals().text_color())
                                );
                            });
                        }).response;

                        response.on_hover_ui(|ui| {
                            ui.set_max_width(450.0);
                            ui.heading(
                                egui::RichText::new(t!("dedup-warning-no-original"))
                                    .color(theme::WARNING_RED)
                                    .strong()
                            );
                            ui.label(t!("dedup-warning-details"));
                            ui.separator();

                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for (filename, nodes) in &fully_selected_groups_info {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(theme::WARNING_RED, "🔥");
                                            ui.strong(filename);
                                            ui.weak(t!(
                                                "dedup-copies-selected",
                                                { "count" => nodes.len() }
                                            ));
                                        });
                                        for &idx in nodes {
                                            let path = snapshot.get_full_path(idx);
                                            let cleaned_path = crate::model::arena::clean_unc_path(&path);
                                            ui.small(format!("  - {cleaned_path}"));
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
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(feature = "online")]
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();

        egui_extras::install_image_loaders(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Process any deferred command line paths on the first draw pass
        if let Some(path) = self.pending_initial_path.take() {
            if path.exists() {
                if path.is_dir() {
                    self.start_scan(path);
                } else if path.is_file()
                    && let Err(e) = self.load_snapshot_file(path.clone())
                {
                    eprintln!("Error loading snapshot file {}: {e}", path.display());
                }
            } else {
                eprintln!("Error: Path does not exist: {}", path.display());
            }
        }

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
        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;

        if self.last_rendered_snapshot_ptr != snapshot_ptr {
            self.table_state.filter_cache_dirty = true;
            self.last_rendered_snapshot_ptr = snapshot_ptr;
        }

        // Delete keyboard shortcuts (Delete / Shift + Delete)
        if !is_scanning
            && !ctx.egui_wants_keyboard_input()
            && !self.table_state.selected_rows.is_empty()
            && ctx.input(|i| i.key_pressed(egui::Key::Delete))
        {
            let shift = ctx.input(|i| i.modifiers.shift);
            self.delete_node_indices = self.table_state.selected_rows.iter().collect();
            if shift {
                if self.deletion_confirmation {
                    self.active_modal = Some(ActiveModal::Delete);
                    self.delete_confirm_checked = false;
                    self.remember_confirmation = false;
                } else {
                    self.execute_deletion(&self.delete_node_indices.clone(), false, &snapshot);
                    self.delete_node_indices.clear();
                }
            } else {
                if self.trash_confirmation {
                    self.active_modal = Some(ActiveModal::Trash);
                    self.delete_confirm_checked = false;
                    self.remember_confirmation = false;
                } else {
                    self.execute_deletion(&self.delete_node_indices.clone(), true, &snapshot);
                    self.delete_node_indices.clear();
                }
            }
        }

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
                    self.delete_node_indices = nodes;
                    if self.trash_confirmation {
                        self.active_modal = Some(ActiveModal::Trash);
                        self.delete_confirm_checked = false;
                        self.remember_confirmation = false;
                    } else {
                        self.execute_deletion(&self.delete_node_indices.clone(), true, &snapshot);
                        self.delete_node_indices.clear();
                    }
                }
                crate::gui::operations::AppCommand::ShowDeleteModal(nodes) => {
                    self.delete_node_indices = nodes;
                    if self.deletion_confirmation {
                        self.active_modal = Some(ActiveModal::Delete);
                        self.delete_confirm_checked = false;
                        self.remember_confirmation = false;
                    } else {
                        self.execute_deletion(&self.delete_node_indices.clone(), false, &snapshot);
                        self.delete_node_indices.clear();
                    }
                }
            }
        }

        // Background Modal polling processing for custom TableOperations
        for op_group in &mut self.operations.groups {
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
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("eDirStat")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.add(
                    egui::Image::new(egui::include_image!(
                        "../../assets/img/icon-transparent.svg"
                    ))
                    .max_height(24.0),
                );
                ui.separator();

                // Temporarily disable button frames to make top-level menus flat & clean
                let saved_button_frame = ui.visuals().button_frame;
                ui.style_mut().visuals.button_frame = false;

                // Top menu buttons (File / View / Help)
                ui.menu_button(t!("file"), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    self.draw_file_menu_contents(ui, &snapshot);
                });
                ui.menu_button(t!("view"), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                    // Aligned emoji checkbox layout
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let mut checked = self.monospace_paths;
                        if ui.checkbox(&mut checked, "").changed() {
                            self.monospace_paths = checked;
                        }
                        let response = ui
                            .horizontal(|ui| {
                                ui.label(egui::RichText::new("🅰").size(12.0));
                                ui.label(
                                    egui::RichText::new(t!("monospace-paths"))
                                        .color(ui.visuals().widgets.inactive.text_color()),
                                );
                            })
                            .response;

                        let label_click = ui.interact(
                            response.rect,
                            ui.id().with("monospace_label"),
                            egui::Sense::click(),
                        );
                        if label_click.clicked() {
                            self.monospace_paths = !self.monospace_paths;
                        }
                    });

                    ui.checkbox(&mut self.highlight_duplicates, t!("highlight-duplicates"));
                    ui.checkbox(&mut self.deletion_confirmation, t!("deletion-confirmation"));
                    ui.checkbox(&mut self.trash_confirmation, t!("trash-confirmation"));

                    ui.menu_button(t!("time-format"), |ui| {
                        for format in crate::model::time_utils::CommonTimeFormat::ALL {
                            let is_selected = self.time_format.0 == format.as_str();
                            if ui.selectable_label(is_selected, format.label()).clicked() {
                                self.time_format = crate::model::time_utils::TimeFormat(
                                    format.as_str().to_string(),
                                );
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        }
                    });

                    ui.menu_button(t!("language"), |ui| {
                        for locale in Locale::iter() {
                            let is_selected = self.locale == locale;
                            let locale_str = locale.to_compact_string();
                            if ui
                                .selectable_label(is_selected, locale_str.as_str())
                                .clicked()
                            {
                                if let Ok(lang) =
                                    fluent_zero::LanguageIdentifier::from_str(&locale_str)
                                {
                                    fluent_zero::set_lang(lang);
                                }

                                self.locale = locale;
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        }
                    });

                    ui.separator();
                    ui.label(t!("layout-mode"));
                    ui.radio_value(
                        &mut self.layout_mode,
                        LayoutMode::Classic,
                        t!("classic-layout"),
                    );
                    ui.radio_value(
                        &mut self.layout_mode,
                        LayoutMode::WinDirStat,
                        t!("windirstat-layout"),
                    );

                    ui.separator();

                    let is_classic = self.layout_mode == LayoutMode::Classic;
                    if is_classic {
                        let left_label = t!("toggle-left-panel", {
                            "collapsed" => self.left_panel_collapsed.to_string()
                        });
                        if ui.button(left_label).clicked() {
                            self.left_panel_collapsed = !self.left_panel_collapsed;
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    }

                    let right_label = t!("toggle-right-panel", {
                        "collapsed" => self.right_panel_collapsed.to_string(),
                        "is_classic" => is_classic.to_string()
                    });
                    if ui.button(right_label).clicked() {
                        self.right_panel_collapsed = !self.right_panel_collapsed;
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    ui.separator();
                    if ui.button(t!("collapse-all")).clicked() {
                        self.table_state.expanded_rows.clear();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
                ui.menu_button(t!("help"), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    if ui.button(t!("about")).clicked() {
                        self.active_modal = Some(ActiveModal::About);
                    }
                });

                ui.separator();

                let should_pulse = !is_scanning && snapshot.nodes.is_empty();
                let scan_btn_text = t!("scan-directory");
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
                        ui.style_mut().visuals.widgets.inactive.bg_stroke =
                            egui::Stroke::new(1.0f32, border_color);
                        ui.style_mut().visuals.widgets.inactive.fg_stroke =
                            egui::Stroke::new(1.0f32, text_color);

                        // Hovered state (bright purple highlight)
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                            theme::COLOR_SCANNING.linear_multiply(0.25);
                        ui.style_mut().visuals.widgets.hovered.bg_stroke =
                            egui::Stroke::new(1.0f32, theme::COLOR_SCANNING);
                        ui.style_mut().visuals.widgets.hovered.fg_stroke =
                            egui::Stroke::new(1.0f32, theme::COLOR_WHITE);

                        // Active state (clicked)
                        ui.style_mut().visuals.widgets.active.weak_bg_fill =
                            theme::COLOR_SCANNING.linear_multiply(0.35);
                        ui.style_mut().visuals.widgets.active.bg_stroke =
                            egui::Stroke::new(1.0f32, theme::COLOR_SCANNING);
                        ui.style_mut().visuals.widgets.active.fg_stroke =
                            egui::Stroke::new(1.0f32, theme::COLOR_WHITE);

                        ui.button(egui::RichText::new(scan_btn_text).strong())
                    })
                    .inner
                } else {
                    ui.button(scan_btn_text)
                };

                if scan_btn.clicked() {
                    let folder_opt = FileDialog::new().pick_folder();
                    if let Some(path) = folder_opt {
                        self.start_scan(path);
                    }
                }

                ui.add_space(10.0);

                if ui.button(t!("save-snapshot")).clicked() && !snapshot.nodes.is_empty() {
                    let file_opt = FileDialog::new()
                        .add_filter("eDirStat Compressed Snapshot (*.edst.zst)", &["edst.zst"])
                        .add_filter("eDirStat Uncompressed Snapshot (*.edst)", &["edst"])
                        .save_file();
                    if let Some(path) = file_opt {
                        let compress = path
                            .extension()
                            .is_none_or(|ext| ext.eq_ignore_ascii_case("zst"));
                        match save_snapshot(&snapshot.nodes, &snapshot.string_pool, &path, compress)
                        {
                            Ok(()) => {}
                            Err(e) => {
                                println!("Failed to save snapshot: {e}");
                            }
                        }
                    }
                }

                ui.add_space(10.0);

                if ui.button(t!("load-snapshot")).clicked() {
                    let file_opt = FileDialog::new()
                        .add_filter("eDirStat Snapshot", &["edst.zst", "edst"])
                        .pick_file();
                    if let Some(path) = file_opt
                        && let Err(e) = self.load_snapshot_file(path)
                    {
                        println!("Failed to load snapshot: {e}");
                    }
                }

                ui.separator();

                // Live status display
                if is_scanning {
                    ui.spinner();
                    ui.colored_label(theme::COLOR_SCANNING, t!("scanning-disk"));
                } else if self.current_scan_path.is_some() {
                    ui.colored_label(theme::COLOR_SCAN_COMPLETE, t!("scan-complete"));
                } else {
                    ui.label(t!("idle"));
                }

                if let Some(ref path) = self.current_scan_path {
                    ui.separator();
                    let path_lossy = path.to_string_lossy();
                    let cleaned_path = crate::model::arena::clean_unc_path(&path_lossy);
                    ui.label(t!("path-label", {
                        "path" => cleaned_path.as_ref()
                    }));
                }

                // --- Right-Aligned Concurrency Badge ---
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let threads = self.traversal_engine.num_threads();
                    let badge_text = t!("worker-threads", {
                        "count" => threads
                    });
                    ui.colored_label(theme::GLOW_INNER_CORE, badge_text)
                        .on_hover_text(t!("worker-threads-hover"));
                });

                ui.style_mut().visuals.button_frame = saved_button_frame; // Restore default button frames
            });
        });

        // Bottom Stats Panel
        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
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

                ui.label(t!("directories-count", {
                    "count" => dir_count
                }));
                ui.separator();
                ui.label(t!("files-count", {
                    "count" => file_count
                }));
                ui.separator();
                ui.label(t!("total-size", {
                    "size" => prettier_bytes::ByteFormatter::new().format(bytes as u64).to_string()
                }));

                if is_scanning && let Some(start) = self.scan_start_time {
                    let elapsed = start.elapsed();

                    #[allow(clippy::cast_precision_loss)]
                    let speed = bytes as f64 / elapsed.as_secs_f64();

                    ui.separator();
                    ui.label(t!("elapsed-time", {
                        "time" => format!("{:.3}s", elapsed.as_secs_f64())
                    }));
                    ui.separator();
                    ui.label(t!("scan-speed", {
                        "speed" => prettier_bytes::ByteFormatter::new().format(speed as u64).to_string()
                    }));
                } else if !is_scanning && let Some(duration) = self.total_scan_duration {
                    ui.separator();
                    ui.label(t!("elapsed-time", {
                        "time" => format!("{:.3}s", duration.as_secs_f64())
                    }));
                    ui.separator();
                    #[allow(clippy::cast_precision_loss)]
                    let speed = if duration.as_secs_f64() > 0.0 {
                        bytes as f64 / duration.as_secs_f64()
                    } else {
                        0.0
                    };
                    ui.label(t!("scan-speed", {
                        "speed" => prettier_bytes::ByteFormatter::new().format(speed as u64).to_string()
                    }));
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
                            let cleaned_path = crate::model::arena::clean_unc_path(&path_str);
                            ui.label(t!("selection-path", {
                                "path" => cleaned_path.as_ref()
                            }));
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
                        ui.label(t!("selection-items", {
                            "count" => self.table_state.selected_rows.len()
                        }));
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
                    .show(ui, |ui| {
                        self.render_classic_left_panel(ui, &snapshot);
                    });
            }

            // Right Panel - Extension statistics
            if !self.right_panel_collapsed {
                self.render_extension_panel(ui);
            }

            // Central Panel - Canvas visual Treemap / Plot Panel
            egui::CentralPanel::default().show(ui, |ui| {
                self.render_classic_central_panel(ui, &snapshot);
            });
        } else {
            self.render_windirstat_layout(ui, &snapshot);
        }

        // Render any active modals
        self.render_modals(&ctx, &snapshot);

        // Show toast notifications
        show_toasts(&ctx);

        #[cfg(feature = "profile-tracy")]
        {
            ui.ctx().request_repaint();
            tracy_client::frame_mark();
        }

        // Batched preference saving evaluated at frame exit
        let current_prefs = crate::model::persistence::preferences::UserPreferences {
            monospace_paths: self.monospace_paths,
            highlight_duplicates: self.highlight_duplicates,
            time_format: self.time_format.clone(),
            deletion_confirmation: self.deletion_confirmation,
            trash_confirmation: self.trash_confirmation,
        };

        if current_prefs != self.last_saved_preferences {
            crate::model::persistence::preferences::save_preferences(&current_prefs);
            self.last_saved_preferences = current_prefs;
        }
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
            if !snapshot.nodes.is_empty() {
                self.draw_custom_operations_toolbar(ui, snapshot);
                ui.add_space(4.0);
            }
            ui.separator();

            if snapshot.nodes.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(t!("explorer-empty-state"));
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
                    if let Some(selected_idx) = self.selected_node_idx() {
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
                            ui.label(t!("placeholder-treemap"));
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

                        // Content-Aware Sync (Selections)
                        let selection_changed = selected_nodes_set.len()
                            != self.table_state.selected_rows.len() as usize
                            || selected_nodes_set
                                .iter()
                                .any(|&idx| !self.table_state.selected_rows.contains(idx));

                        if selection_changed {
                            self.table_state.selected_rows.clear();
                            self.table_state
                                .selected_rows
                                .extend(selected_nodes_set.iter());
                        }

                        // Content-Aware Sync (Expansions)
                        let expansion_changed = expanded_nodes_set.len()
                            != self.table_state.expanded_rows.len() as usize
                            || expanded_nodes_set
                                .iter()
                                .any(|&idx| !self.table_state.expanded_rows.contains(idx));

                        if expansion_changed {
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
                            ui.label(t!("placeholder-plots"));
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

                        // Content-Aware Sync (Selections)
                        let selection_changed = selected_nodes_set.len()
                            != self.table_state.selected_rows.len() as usize
                            || selected_nodes_set
                                .iter()
                                .any(|&idx| !self.table_state.selected_rows.contains(idx));

                        if selection_changed {
                            self.table_state.selected_rows.clear();
                            self.table_state
                                .selected_rows
                                .extend(selected_nodes_set.iter());
                        }

                        // Content-Aware Sync (Expansions)
                        let expansion_changed = expanded_nodes_set.len()
                            != self.table_state.expanded_rows.len() as usize
                            || expanded_nodes_set
                                .iter()
                                .any(|&idx| !self.table_state.expanded_rows.contains(idx));

                        if expansion_changed {
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
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // Render operations directly as a flat row of toolbar buttons
                    ui.spacing_mut().item_spacing.x = 8.0;

                    self.draw_custom_operations_toolbar(ui, snapshot);

                    // Separator between operations and the search/filter box
                    ui.separator();

                    // Filter search input
                    ui.label(t!("search-filter-label"));
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
                        .show(ui, |ui| {
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
                egui::CentralPanel::default().frame(frame).show(ui, |ui| {
                    self.render_hierarchical_table(ui, snapshot);
                });
            });

        // The remaining area becomes the bottom section
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical(|ui| {
                self.draw_central_panel_header(ui, snapshot);
                ui.separator();

                // Right Extensions Panel inside the bottom section if not collapsed
                if !self.right_panel_collapsed && self.vis_mode != VisMode::Deduplicator {
                    egui::Panel::right("windirstat_extensions_panel")
                        .resizable(true)
                        .default_size(210.0)
                        .size_range(80.0..=250.0)
                        .show(ui, |ui| {
                            self.draw_extensions_contents(ui);
                        });
                }

                // Rest of the space for visualizers (rendered directly without nested CentralPanel to avoid vertical spacing gaps)
                match self.vis_mode {
                    VisMode::Treemap => {
                        if snapshot.nodes.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(t!("placeholder-treemap"));
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

                            // Content-Aware Sync (Selections)
                            let selection_changed = selected_nodes_set.len()
                                != self.table_state.selected_rows.len() as usize
                                || selected_nodes_set
                                    .iter()
                                    .any(|&idx| !self.table_state.selected_rows.contains(idx));

                            if selection_changed {
                                self.table_state.selected_rows.clear();
                                self.table_state
                                    .selected_rows
                                    .extend(selected_nodes_set.iter());
                            }

                            // Content-Aware Sync (Expansions)
                            let expansion_changed = expanded_nodes_set.len()
                                != self.table_state.expanded_rows.len() as usize
                                || expanded_nodes_set
                                    .iter()
                                    .any(|&idx| !self.table_state.expanded_rows.contains(idx));

                            if expansion_changed {
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
                                        ui.label(t!("placeholder-plots"));
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

                                    // Content-Aware Sync (Selections)
                                    let selection_changed = selected_nodes_set.len()
                                        != self.table_state.selected_rows.len() as usize
                                        || selected_nodes_set.iter().any(|&idx| {
                                            !self.table_state.selected_rows.contains(idx)
                                        });

                                    if selection_changed {
                                        self.table_state.selected_rows.clear();
                                        self.table_state
                                            .selected_rows
                                            .extend(selected_nodes_set.iter());
                                    }

                                    // Content-Aware Sync (Expansions)
                                    let expansion_changed = expanded_nodes_set.len()
                                        != self.table_state.expanded_rows.len() as usize
                                        || expanded_nodes_set.iter().any(|&idx| {
                                            !self.table_state.expanded_rows.contains(idx)
                                        });

                                    if expansion_changed {
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

    pub(crate) fn draw_custom_operations_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let provider =
                crate::gui::explorer::TableProviderWrapper::new(snapshot, self.time_format.clone());

            let _ = self.operations.gui_custom(
                ui,
                &provider,
                &mut self.table_state,
                false,
                |ui, op, enabled, reason, _| {
                    render_custom_op_button(ui, op.icon(), op.name().as_ref(), enabled, reason)
                },
            );
        });
    }
}

fn get_op_hover_color(op_name: &str) -> egui::Color32 {
    match op_name {
        "Up One Level" => egui::Color32::from_rgb(59, 130, 246), // Blue
        "Refresh Entire Scan" => egui::Color32::from_rgb(16, 185, 129), // Emerald Green
        "Refresh Directory" => egui::Color32::from_rgb(34, 197, 94), // Green
        "Open in File Manager" => egui::Color32::from_rgb(245, 158, 11), // Orange/Amber
        "Open Terminal Here" => egui::Color32::from_rgb(6, 182, 212), // Cyan/Teal
        "Copy Path" | "Copy Name" => egui::Color32::from_rgb(139, 92, 246), // Purple
        "Move to Trash" => egui::Color32::from_rgb(234, 179, 8), // Yellow/Orange
        "Permanently Delete" => egui::Color32::from_rgb(239, 68, 68), // Red
        _ => egui::Color32::from_rgb(96, 165, 250),              // Default light blue
    }
}

fn render_custom_op_button(
    ui: &mut egui::Ui,
    icon: &str,
    name: &str,
    enabled: bool,
    reason: &str,
) -> egui::Response {
    let hover_color = get_op_hover_color(name);

    ui.add_enabled_ui(enabled, |ui| {
        let mut response = ui
            .scope(|ui| {
                ui.style_mut().visuals.button_frame = true;

                // Inactive (subtle tint, extremely faded border)
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    hover_color.linear_multiply(0.04);
                ui.style_mut().visuals.widgets.inactive.bg_stroke =
                    egui::Stroke::new(1.0f32, hover_color.linear_multiply(0.12));
                ui.style_mut().visuals.widgets.inactive.fg_stroke =
                    egui::Stroke::new(1.0f32, ui.visuals().widgets.inactive.text_color());

                // Hovered (soft fill, subtle stroke, full hover color for icon)
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    hover_color.linear_multiply(0.12);
                ui.style_mut().visuals.widgets.hovered.bg_stroke =
                    egui::Stroke::new(1.0f32, hover_color.linear_multiply(0.4));
                ui.style_mut().visuals.widgets.hovered.fg_stroke =
                    egui::Stroke::new(1.0f32, hover_color);

                // Active (pressed)
                ui.style_mut().visuals.widgets.active.weak_bg_fill =
                    hover_color.linear_multiply(0.24);
                ui.style_mut().visuals.widgets.active.bg_stroke =
                    egui::Stroke::new(1.0f32, hover_color.linear_multiply(0.6));
                ui.style_mut().visuals.widgets.active.fg_stroke =
                    egui::Stroke::new(1.0f32, hover_color);

                // Set button padding to make it a nice square
                ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);

                ui.button(egui::RichText::new(icon).size(15.0))
            })
            .inner;

        if enabled {
            response = response.on_hover_text(name);
        } else {
            response = response.on_disabled_hover_text(format!("{name}\n({reason})"));
        }

        response
    })
    .inner
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

pub static TOASTS: std::sync::LazyLock<parking_lot::Mutex<egui_notify::Toasts>> =
    std::sync::LazyLock::new(|| {
        parking_lot::Mutex::new(
            egui_notify::Toasts::new()
                .with_anchor(egui_notify::Anchor::BottomRight)
                .with_margin(egui::vec2(10.0, 30.0)),
        )
    });

pub fn toast_success(message: impl Into<egui::WidgetText>) {
    TOASTS
        .lock()
        .success(message)
        .duration(Some(Duration::from_secs(4)));
}

pub fn toast_info(message: impl Into<egui::WidgetText>) {
    TOASTS
        .lock()
        .info(message)
        .duration(Some(Duration::from_secs(4)));
}

pub fn toast_warning(message: impl Into<egui::WidgetText>) {
    TOASTS
        .lock()
        .warning(message)
        .duration(Some(Duration::from_secs(8)));
}

pub fn toast_error(message: impl Into<egui::WidgetText>) {
    TOASTS
        .lock()
        .error(message)
        .duration(Some(Duration::from_secs(16)));
}

pub fn show_toasts(ctx: &egui::Context) {
    TOASTS.lock().show(ctx);
}
