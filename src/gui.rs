use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use eframe::egui;
use rfd::FileDialog;
use smallvec::SmallVec;

use super::{
    arena::{FileArenaSnapshot, FileNode, NO_INDEX, StringPool},
    coordinator::SharedState,
    persistence::{load_snapshot, save_snapshot},
    traversal::TraversalEngine,
};

const NO_EXTENSION: &str = "(no extension)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveModal {
    Delete,
    About,
}

pub struct GuiApp {
    shared_state: Arc<SharedState>,
    traversal_engine: Arc<TraversalEngine>,

    // UI state
    selected_node_idx: Option<u32>,
    expanded_nodes: HashSet<u32>,
    search_query: String,
    monospace_paths: bool,

    // Modal states
    delete_confirm_checked: bool,
    delete_node_idx: Option<u32>,
    active_modal: Option<ActiveModal>,

    // Saved scan parameters
    current_scan_path: Option<PathBuf>,
    scan_start_time: Option<Instant>,
    total_scan_duration: Option<Duration>,

    // Extension breakdown stats
    extension_stats: Vec<ExtensionStat>,
    last_extension_update: Option<Instant>,

    // Layout caching fields
    cached_blocks: Vec<TreemapBlock>,
    last_snapshot_ptr: usize,
    last_rect: egui::Rect,
}

struct ExtensionStat {
    ext: String,
    total_size: u64,
    file_count: u32,
    color: egui::Color32,
}

impl GuiApp {
    pub fn new(shared_state: Arc<SharedState>, traversal_engine: Arc<TraversalEngine>) -> Self {
        Self {
            shared_state,
            traversal_engine,
            selected_node_idx: None,
            expanded_nodes: HashSet::new(),
            search_query: String::new(),
            monospace_paths: false,
            delete_confirm_checked: false,
            delete_node_idx: None,
            active_modal: None,
            current_scan_path: None,
            scan_start_time: None,
            total_scan_duration: None,
            extension_stats: Vec::new(),
            last_extension_update: None,
            cached_blocks: Vec::new(),
            last_snapshot_ptr: 0,
            last_rect: egui::Rect::NOTHING,
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
        self.traversal_engine.stats().reset();
        self.cached_blocks.clear();
        self.last_snapshot_ptr = 0;
        self.last_rect = egui::Rect::NOTHING;
    }

    /// Renders the shared "File" actions used in both the top toolbar and node context menus.
    fn draw_file_menu_contents(&mut self, ui: &mut egui::Ui, snapshot: &FileArenaSnapshot) {
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
        // Fetch current snapshot
        let snapshot = self.shared_state.current_snapshot.load();
        let is_scanning = self.shared_state.is_scanning.load(Ordering::SeqCst);

        // Repaint continuously during scan to show live progress
        if is_scanning {
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        // Apply dark, premium glassmorphism-inspired style
        setup_custom_style(&ctx);

        // Top Control Panel
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("eDirStat 👷")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
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
                    ui.colored_label(egui::Color32::from_rgb(139, 92, 246), "Scanning Disk...");
                } else if self.current_scan_path.is_some() {
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), "Scan Complete");
                } else {
                    ui.label("Idle");
                }

                if let Some(ref path) = self.current_scan_path {
                    ui.separator();
                    ui.label(format!("Path: {}", path.display()));
                }
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
        egui::Panel::left("left_panel")
            .resizable(true)
            .default_size(450.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    // Toolbar above the root tree
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            self.draw_file_menu_contents(ui, &snapshot);
                        });
                        ui.menu_button("View", |ui| {
                            ui.checkbox(&mut self.monospace_paths, "🅰 Monospace Paths");
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("ℹ About").clicked() {
                                self.active_modal = Some(ActiveModal::About);
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("🔍 Filter:");
                        ui.text_edit_singleline(&mut self.search_query);
                        if !self.search_query.is_empty() && ui.button("❌").clicked() {
                            self.search_query.clear();
                        }
                    });
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

                        let row_height = ui.spacing().interact_size.y;
                        egui::ScrollArea::vertical().show_rows(
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

        // Right Panel - Extension statistics
        egui::Panel::right("right_panel")
            .resizable(true)
            .size_range(80.0..=250.0)
            .default_size(210.0)
            .show_inside(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                ui.vertical(|ui| {
                    ui.heading(
                        egui::RichText::new("📂 Extensions")
                            .strong()
                            .color(ui.visuals().strong_text_color()),
                    );
                    ui.separator();

                    // Map the pre-computed/pre-sorted stats vector from our background thread
                    let shared_ext_stats = self.shared_state.extension_stats.load();
                    if !shared_ext_stats.is_empty() {
                        self.extension_stats = shared_ext_stats
                            .iter()
                            .map(|(ext, total_size, file_count)| ExtensionStat {
                                ext: ext.clone(),
                                total_size: *total_size,
                                file_count: *file_count,
                                color: get_color_for_extension(ext),
                            })
                            .collect();
                    }

                    if self.extension_stats.is_empty() {
                        ui.label("No statistics gathered yet.");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for stat in &self.extension_stats {
                                ui.horizontal(|ui| {
                                    // Colored dot
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(10.0, 10.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().circle_filled(rect.center(), 5.0, stat.color);

                                    // Allocate name width and truncate it
                                    let name_width = (ui.available_width() - 65.0).max(10.0);
                                    ui.allocate_ui(
                                        egui::vec2(name_width, ui.spacing().interact_size.y),
                                        |ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Truncate);

                                            // Render the label and attach a hover tooltip showing file count
                                            ui.label(&stat.ext).on_hover_text(format!(
                                                "Files: {}",
                                                stat.file_count
                                            ));
                                        },
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                prettier_bytes::ByteFormatter::new()
                                                    .format(stat.total_size)
                                                    .to_string(),
                                            );
                                        },
                                    );
                                });
                            }
                        });
                    }
                });
            });

        // Central Panel - Canvas visual Treemap
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("📊 Treemap Visualization")
                        .strong()
                        .color(ui.visuals().strong_text_color()),
                );
                ui.separator();

                if snapshot.nodes.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("Scanned filesystem will be visualized as a treemap here.");
                    });
                } else {
                    let available_rect = ui.available_rect_before_wrap();
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(available_rect.width(), available_rect.height() - 20.0),
                        egui::Sense::click_and_drag(),
                    );

                    // --- Layout Cache Check ---
                    let snapshot_ptr = Arc::as_ptr(&snapshot.nodes) as usize;
                    let needs_rebuild = self.cached_blocks.is_empty()
                        || snapshot_ptr != self.last_snapshot_ptr
                        || rect != self.last_rect;

                    if needs_rebuild {
                        let mut blocks = Vec::new();
                        let config = TreemapConfig {
                            nodes: &snapshot.nodes,
                            string_pool: &snapshot.string_pool,
                            max_depth: 20, // High depth is safe due to visual density checking
                        };
                        build_treemap(&config, 0, rect, 0, &mut blocks);
                        self.cached_blocks = blocks;
                        self.last_snapshot_ptr = snapshot_ptr;
                        self.last_rect = rect;
                    }

                    let painter = ui.painter_at(rect);
                    let mut hovered_block = None;
                    let hover_pos = response.hover_pos();

                    // Look up hovered block (O(M) linear search is fast on layout blocks in Rust)
                    if let Some(pos) = hover_pos {
                        for block in &self.cached_blocks {
                            if block.rect.contains(pos) {
                                hovered_block = Some(block);
                                break;
                            }
                        }
                    }

                    // GPU Batching: Consolidate static blocks into exactly ONE single mesh submission to the GPU
                    let mut combined_mesh = egui::Mesh::default();
                    for block in &self.cached_blocks {
                        let fill_color = block.color;
                        let color_light = fill_color.linear_multiply(1.15);
                        let color_dark = fill_color.linear_multiply(0.75);

                        let base_vertex_idx = combined_mesh.vertices.len() as u32;

                        combined_mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.left_top(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_light,
                        });
                        combined_mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.right_top(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_light,
                        });
                        combined_mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.right_bottom(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_dark,
                        });
                        combined_mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.left_bottom(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_dark,
                        });

                        combined_mesh.add_triangle(
                            base_vertex_idx,
                            base_vertex_idx + 1,
                            base_vertex_idx + 2,
                        );
                        combined_mesh.add_triangle(
                            base_vertex_idx,
                            base_vertex_idx + 2,
                            base_vertex_idx + 3,
                        );
                    }

                    painter.add(combined_mesh);

                    // Dynamic overlays for highlights
                    if let Some(block) = hovered_block {
                        let stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
                        painter.rect(
                            block.rect,
                            0.0,
                            egui::Color32::TRANSPARENT,
                            stroke,
                            egui::StrokeKind::Inside,
                        );
                    }

                    if let Some(selected_idx) = self.selected_node_idx
                        && let Some(block) = self
                            .cached_blocks
                            .iter()
                            .find(|b| b.node_idx == selected_idx)
                    {
                        let accent_purple = egui::Color32::from_rgb(139, 92, 246);
                        let stroke = egui::Stroke::new(2.5, accent_purple);
                        painter.rect(
                            block.rect,
                            0.0,
                            egui::Color32::TRANSPARENT,
                            stroke,
                            egui::StrokeKind::Inside,
                        );
                    }

                    // Click event to select node
                    if response.clicked()
                        && let Some(block) = hovered_block
                    {
                        self.selected_node_idx = Some(block.node_idx);

                        // Auto expand parents so it shows up in tree view
                        let mut curr = Some(block.node_idx);
                        while let Some(idx) = curr {
                            if let Some(node) = snapshot.nodes.get(idx as usize) {
                                if node.is_directory() {
                                    self.expanded_nodes.insert(idx);
                                }
                                curr = node.parent_opt();
                            } else {
                                break;
                            }
                        }
                    }

                    // Draw tooltip
                    if let Some(block) = hovered_block {
                        let path_str = snapshot.get_full_path(block.node_idx);
                        let size_str = prettier_bytes::ByteFormatter::new()
                            .format(snapshot.nodes[block.node_idx as usize].size)
                            .to_string();
                        egui::Tooltip::always_open(
                            ctx.clone(),
                            ui.layer_id(),
                            egui::Id::new("treemap_tooltip"),
                            egui::PopupAnchor::Pointer,
                        )
                        .show(|ui| {
                            ui.label(format!("📁 {path_str}"));
                            ui.label(format!("💾 Size: {size_str}"));
                        });
                    }
                }
            });
        });

        // Render Permanent Deletion Modal Popup
        if self.active_modal == Some(ActiveModal::Delete) {
            let idx_opt = self.delete_node_idx;
            if let Some(idx) = idx_opt {
                let path_str = snapshot.get_full_path(idx);
                let size_str = prettier_bytes::ByteFormatter::new()
                    .format(snapshot.nodes[idx as usize].size)
                    .to_string();

                let mut open = true;
                egui::Window::new("⚠ PERMANENT DELETION WARNING")
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .frame(egui::Frame::window(ui.style()).stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 38, 38)))) // Thick red border outline
                    .show(&ctx, |ui| {
                        ui.vertical(|ui| {
                            let path = std::path::Path::new(&path_str);
                            if path.exists() {
                                ui.heading(
                                    egui::RichText::new("⚠ Permanent Deletion Warning!")
                                        .color(egui::Color32::from_rgb(239, 68, 68))
                                        .strong()
                                );
                                ui.separator();

                                ui.label("You are about to permanently delete the following path:");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
                                ui.label(format!("Total Size: {size_str}"));
                                ui.separator();

                                ui.label("This is a recursive deletion. All files, folders, and subdirectories under this path will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).");
                                ui.add_space(8.0);

                                ui.checkbox(&mut self.delete_confirm_checked, "I understand that files will be permanently deleted and cannot be recovered.");
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Cancel").clicked() {
                                        self.active_modal = None;
                                    }

                                    // Red confirm button
                                    let confirm_btn = egui::Button::new(
                                        egui::RichText::new("🗑 Yes, Delete Permanently")
                                            .color(egui::Color32::WHITE)
                                            .strong()
                                    ).fill(egui::Color32::from_rgb(220, 38, 38));

                                    let confirm_res = ui.add_enabled(self.delete_confirm_checked, confirm_btn);
                                    if confirm_res.clicked() {
                                        let path = std::path::Path::new(&path_str);
                                        if path.exists() {
                                            let delete_result = if path.is_dir() {
                                                std::fs::remove_dir_all(path)
                                            } else {
                                                std::fs::remove_file(path)
                                            };

                                            if let Err(e) = delete_result {
                                                println!("Failed to delete path: {e}");
                                            } else {
                                                // Reset selection upon deletion
                                                self.selected_node_idx = None;
                                            }
                                        }
                                        self.active_modal = None;
                                    }
                                });
                            } else {
                                ui.heading(
                                    egui::RichText::new("❌ Path Does Not Exist!")
                                        .color(egui::Color32::from_rgb(239, 68, 68))
                                        .strong()
                                );
                                ui.separator();
                                ui.label("Error: The path you are trying to delete does not exist on disk.");
                                ui.colored_label(ui.visuals().strong_text_color(), &path_str);
                                ui.add_space(8.0);
                                if ui.button("Close").clicked() {
                                    self.active_modal = None;
                                }
                            }
                        });
                    });
                if !open {
                    self.active_modal = None;
                }
            }
        }

        // Render Help -> About Modal Popup
        if self.active_modal == Some(ActiveModal::About) {
            let mut open = true;
            egui::Window::new("ℹ About eDirStat")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(&ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("eDirStat 👷")
                                .strong()
                                .color(ui.visuals().strong_text_color())
                        );
                        ui.label("v1.0.0");
                        ui.separator();
                        ui.label("By: Cody Wyatt Neiman (xangelix) <".to_owned() + "neiman" + "@" + "cody.to>");
                        ui.add_space(8.0);
                        ui.label("A modern, zero-copy, highly performant disk usage analyzer written in Rust.");
                        ui.label("Features dynamic work-stealing multithreaded directory walking, lazy explorer sibling sorting, zero-copy persistent memory mapping, HSL treemap gradients, and instant virtual rendering.");
                        ui.add_space(8.0);
                        if ui.button("Close").clicked() {
                            self.active_modal = None;
                        }
                    });
                });
            if !open {
                self.active_modal = None;
            }
        }
    }
}

impl GuiApp {
    fn flatten_visible_tree(
        &mut self,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
        indent_level: usize,
        out: &mut Vec<(u32, usize)>,
    ) {
        let node = &snapshot.nodes[node_idx as usize];
        let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");

        // Filter search query
        if !self.search_query.is_empty() {
            let matches_query = name
                .to_lowercase()
                .contains(&self.search_query.to_lowercase());
            // If it's a file and doesn't match, skip
            if !node.is_directory() && !matches_query {
                return;
            }
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
            // Sort immediate children by size descending dynamically for 100% correct tree views
            sorted_child_indices.sort_by(|&a, &b| {
                snapshot.nodes[b as usize]
                    .size
                    .cmp(&snapshot.nodes[a as usize].size)
            });

            for &child_idx in &sorted_child_indices {
                self.flatten_visible_tree(snapshot, child_idx, indent_level + 1, out);
            }
        }
    }

    fn render_tree_node_row(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &FileArenaSnapshot,
        node_idx: u32,
        indent_level: usize,
    ) {
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

        // Draw vertical indentation guidelines to visually track nested containers
        let painter = ui.painter();
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(65));
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

// Custom Glassmorphic Dark styling settings
fn setup_custom_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Background Slate Color
    visuals.panel_fill = egui::Color32::from_rgb(18, 20, 28);
    visuals.window_fill = egui::Color32::from_rgb(26, 29, 38);

    // Borders
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(26, 29, 38);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 43, 56));

    ctx.set_visuals(visuals);
}

// Squarified partitioning treemap algorithm (Bruls, Huizing, and van Wijk)
struct TreemapBlock {
    rect: egui::Rect,
    node_idx: u32,
    color: egui::Color32,
}

struct TreemapConfig<'a> {
    nodes: &'a [FileNode],
    string_pool: &'a StringPool,
    max_depth: usize,
}

fn worst_aspect_ratio(row: &[f64], w: f64) -> f64 {
    if row.is_empty() || w <= 0.0 {
        return f64::INFINITY;
    }
    let sum: f64 = row.iter().sum();
    if sum <= 0.0 {
        return f64::INFINITY;
    }
    let sum_sq = sum * sum;
    let w_sq = w * w;

    let mut max_ratio = 0.0;
    for &area in row {
        if area <= 0.0 {
            continue;
        }
        let ratio1 = (w_sq * area) / sum_sq;
        let ratio2 = sum_sq / (w_sq * area);
        let ratio = ratio1.max(ratio2);
        if ratio > max_ratio {
            max_ratio = ratio;
        }
    }
    max_ratio
}

fn recurse_child(
    config: &TreemapConfig,
    child_idx: u32,
    child_rect: egui::Rect,
    depth: usize,
    blocks: &mut Vec<TreemapBlock>,
) {
    // Minimum visual dimension constraint
    const MIN_PIXEL_DIM: f32 = 12.0;

    if child_rect.width() <= 0.0 || child_rect.height() <= 0.0 {
        return;
    }

    let child = &config.nodes[child_idx as usize];

    let is_leaf_or_too_small = !child.is_directory()
        || depth >= config.max_depth
        || child_rect.width() < MIN_PIXEL_DIM
        || child_rect.height() < MIN_PIXEL_DIM;

    if is_leaf_or_too_small {
        let name = config.string_pool.get(child.name_id).unwrap_or("");
        let ext = Path::new(name).extension().map_or_else(
            || NO_EXTENSION.to_string(),
            |s| s.to_string_lossy().to_ascii_lowercase(),
        );
        let color = get_color_for_extension(&ext);
        blocks.push(TreemapBlock {
            rect: child_rect,
            node_idx: child_idx,
            color,
        });
        return;
    }

    build_treemap(config, child_idx, child_rect, depth + 1, blocks);
}

fn build_treemap(
    config: &TreemapConfig,
    node_idx: u32,
    rect: egui::Rect,
    depth: usize,
    blocks: &mut Vec<TreemapBlock>,
) {
    const MIN_AVG_CHILD_AREA: f64 = 16.0; // Corresponds to a 4x4 screen box per child

    let node = &config.nodes[node_idx as usize];
    if node.size == 0 || rect.width() < 2.0 || rect.height() < 2.0 {
        return;
    }

    // Leaf files or max depth limit reached
    if !node.is_directory() || depth >= config.max_depth {
        let name = config.string_pool.get(node.name_id).unwrap_or("");
        let ext = Path::new(name).extension().map_or_else(
            || NO_EXTENSION.to_string(),
            |s| s.to_string_lossy().to_ascii_lowercase(),
        );
        let color = get_color_for_extension(&ext);

        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    // Collect directory children
    let mut children = SmallVec::<[u32; 16]>::new();
    let mut curr = node.first_child;
    while curr != NO_INDEX {
        children.push(curr);
        curr = config.nodes[curr as usize].next_sibling;
    }

    if children.is_empty() {
        let color = egui::Color32::from_gray(100);
        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    // --- Dense Directory Area Cutoff Optimization ---
    // If a directory contains more children than can visually be resolved cleanly,
    // draw the parent directory itself as a solid block.
    // This removes layout gaps and saves 99% of processing on large, deep structures.
    let area = (rect.width() * rect.height()) as f64;

    #[allow(clippy::cast_precision_loss)]
    let avg_area_per_child = area / children.len() as f64;

    if avg_area_per_child < MIN_AVG_CHILD_AREA {
        let name = config.string_pool.get(node.name_id).unwrap_or("");
        let ext = Path::new(name).extension().map_or_else(
            || NO_EXTENSION.to_string(),
            |s| s.to_string_lossy().to_ascii_lowercase(),
        );
        let color = get_color_for_extension(&ext);
        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    // Sort descending
    children.sort_by(|&a, &b| {
        config.nodes[b as usize]
            .size
            .cmp(&config.nodes[a as usize].size)
    });

    // Filter out items with 0 size
    let active_children: Vec<u32> = children
        .into_iter()
        .filter(|&idx| config.nodes[idx as usize].size > 0)
        .collect();

    if active_children.is_empty() {
        return;
    }

    #[allow(clippy::cast_precision_loss)]
    let total_size = active_children
        .iter()
        .map(|&idx| config.nodes[idx as usize].size)
        .sum::<u64>() as f64;

    if total_size == 0.0 {
        return;
    }

    // Map sizes to pixel areas
    let total_area = (rect.width() * rect.height()) as f64;
    let child_areas: Vec<f64> = active_children
        .iter()
        .map(|&idx| {
            #[allow(clippy::cast_precision_loss)]
            let size = config.nodes[idx as usize].size as f64;
            (size / total_size) * total_area
        })
        .collect();

    let mut remaining_rect = rect;
    let mut i = 0;

    while i < active_children.len() {
        let w = (remaining_rect.width().min(remaining_rect.height())) as f64;
        if w <= 0.0 {
            break;
        }

        let mut current_row = Vec::new();
        current_row.push(child_areas[i]);
        let mut j = i + 1;

        while j < active_children.len() {
            let next_area = child_areas[j];
            let mut test_row = current_row.clone();
            test_row.push(next_area);

            let worst_before = worst_aspect_ratio(&current_row, w);
            let worst_after = worst_aspect_ratio(&test_row, w);

            if worst_after <= worst_before {
                current_row.push(next_area);
                j += 1;
            } else {
                break;
            }
        }

        let row_sum: f64 = current_row.iter().sum();
        let vertical_layout = remaining_rect.width() >= remaining_rect.height();

        if vertical_layout {
            let h = remaining_rect.height() as f64;
            let thickness = if h > 0.0 { row_sum / h } else { 0.0 };
            let mut current_y = remaining_rect.min.y;

            for (k, &area) in current_row.iter().enumerate() {
                let child_idx = active_children[i + k];
                let item_height = if row_sum > 0.0 {
                    h * (area / row_sum)
                } else {
                    0.0
                };

                let child_rect = egui::Rect::from_min_max(
                    egui::pos2(remaining_rect.min.x, current_y),
                    egui::pos2(
                        (remaining_rect.min.x + thickness as f32).min(remaining_rect.max.x),
                        (current_y + item_height as f32).min(remaining_rect.max.y),
                    ),
                );

                recurse_child(config, child_idx, child_rect, depth, blocks);
                current_y += item_height as f32;
            }

            remaining_rect.min.x =
                (remaining_rect.min.x + thickness as f32).min(remaining_rect.max.x);
        } else {
            let width = remaining_rect.width() as f64;
            let thickness = if width > 0.0 { row_sum / width } else { 0.0 };
            let mut current_x = remaining_rect.min.x;

            for (k, &area) in current_row.iter().enumerate() {
                let child_idx = active_children[i + k];
                let item_width = if row_sum > 0.0 {
                    width * (area / row_sum)
                } else {
                    0.0
                };

                let child_rect = egui::Rect::from_min_max(
                    egui::pos2(current_x, remaining_rect.min.y),
                    egui::pos2(
                        (current_x + item_width as f32).min(remaining_rect.max.x),
                        (remaining_rect.min.y + thickness as f32).min(remaining_rect.max.y),
                    ),
                );

                recurse_child(config, child_idx, child_rect, depth, blocks);
                current_x += item_width as f32;
            }

            remaining_rect.min.y =
                (remaining_rect.min.y + thickness as f32).min(remaining_rect.max.y);
        }

        i = j;
    }
}

// Harmonious custom HSL colors for extensions
fn get_color_for_extension(ext: &str) -> egui::Color32 {
    match ext {
        "rs" => egui::Color32::from_rgb(239, 68, 68), // Rust red
        "toml" => egui::Color32::from_rgb(59, 130, 246), // Toml blue
        "git" | "gitignore" => egui::Color32::from_rgb(107, 114, 128), // Git gray
        "js" | "ts" => egui::Color32::from_rgb(234, 179, 8), // JS yellow
        "json" | "yaml" => egui::Color32::from_rgb(168, 85, 247), // Purple config
        "html" | "css" => egui::Color32::from_rgb(249, 115, 22), // HTML/CSS orange
        "py" => egui::Color32::from_rgb(16, 185, 129), // Python green
        "c" | "cpp" | "h" => egui::Color32::from_rgb(6, 182, 212), // C/C++ cyan
        "zip" | "tar" | "gz" => egui::Color32::from_rgb(236, 72, 153), // Compressed pink
        "mp3" | "wav" | "flac" => egui::Color32::from_rgb(14, 165, 233), // Audio sky-blue
        "mp4" | "mkv" | "avi" => egui::Color32::from_rgb(20, 184, 166), // Video teal
        "png" | "jpg" | "jpeg" | "gif" => egui::Color32::from_rgb(244, 63, 94), // Image rose
        NO_EXTENSION => egui::Color32::from_rgb(75, 85, 99), // Muted dark gray
        _ => {
            // Hash the extension to generate a stable, beautiful pseudo-random color
            let mut hash: u32 = 5381;
            for c in ext.bytes() {
                hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u32);
            }
            // Hue from hash, Saturation ~75%, Lightness ~55%
            #[allow(clippy::cast_precision_loss)]
            let hue = (hash % 360) as f32 / 360.0;

            let color = egui::epaint::Hsva::new(hue, 0.75, 0.55, 1.0);
            egui::Color32::from(color)
        }
    }
}
