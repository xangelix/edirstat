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
    last_extension_update: Option<Instant>, // tracking for live throttled updates
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
            last_extension_update: None, // Added initialization
        }
    }

    fn reset_state(&mut self) {
        self.selected_node_idx = None;
        self.expanded_nodes.clear();
        self.extension_stats.clear();
        self.last_extension_update = None; // Added reset
        self.delete_confirm_checked = false;
        self.delete_node_idx = None;
        self.active_modal = None;
        self.traversal_engine.stats().reset();
    }

    fn update_extension_stats(&mut self, snapshot: &FileArenaSnapshot) {
        let mut ext_map: HashMap<String, (u64, u32)> = HashMap::new();

        for node in snapshot.nodes.iter() {
            if node.is_directory() {
                continue;
            }
            if let Some(name) = snapshot.string_pool.get(node.name_id) {
                let ext = Path::new(name).extension().map_or_else(
                    || NO_EXTENSION.to_string(),
                    |s| s.to_string_lossy().to_ascii_lowercase(),
                );

                let entry = ext_map.entry(ext).or_insert((0, 0));
                entry.0 += node.size;
                entry.1 += 1;
            }
        }

        let mut stats: Vec<ExtensionStat> = ext_map
            .into_iter()
            .map(|(ext, (total_size, file_count))| {
                let color = get_color_for_extension(&ext);
                ExtensionStat {
                    ext,
                    total_size,
                    file_count,
                    color,
                }
            })
            .collect();

        // Sort by total size descending
        stats.sort_by_key(|b| std::cmp::Reverse(b.total_size));
        self.extension_stats = stats;
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

                if is_scanning {
                    if let Some(start) = self.scan_start_time {
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

                    // Periodically update extension stats during active scanning
                    let should_update = self
                        .last_extension_update
                        .is_none_or(|last| last.elapsed() >= Duration::from_millis(250));
                    if should_update && !snapshot.nodes.is_empty() {
                        self.update_extension_stats(&snapshot);
                        self.last_extension_update = Some(Instant::now());
                    }
                } else if let Some(ref _path) = self.current_scan_path {
                    // Run one final sync on completion or if loading a static snapshot
                    if self.last_extension_update.is_some() || self.extension_stats.is_empty() {
                        if !snapshot.nodes.is_empty() {
                            self.update_extension_stats(&snapshot);
                        }
                        self.last_extension_update = None; // Reset tracker post-scan
                    }
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
                            let has_selection = self.selected_node_idx.is_some();
                            let open_btn = ui.add_enabled(
                                has_selection,
                                egui::Button::new("🗁 Open in File Manager"),
                            );
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
                            }

                            let delete_btn = ui.add_enabled(
                                has_selection,
                                egui::Button::new("🗑 Delete (Permanent)"),
                            );
                            if delete_btn.clicked() {
                                self.active_modal = Some(ActiveModal::Delete);
                                self.delete_confirm_checked = false;
                                self.delete_node_idx = self.selected_node_idx;
                            }
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

                    let mut blocks = Vec::new();
                    let config = TreemapConfig {
                        nodes: &snapshot.nodes,
                        string_pool: &snapshot.string_pool,
                        max_depth: 5, // depth limit for visual clarity and performance
                    };
                    build_treemap(&config, 0, rect, 0, &mut blocks);

                    let painter = ui.painter_at(rect);
                    let mut hovered_block = None;

                    for block in &blocks {
                        // Draw block rectangle
                        let mut fill_color = block.color;
                        let mut border_color = egui::Color32::TRANSPARENT;
                        let mut border_width = 0.0;

                        let is_hovered = response
                            .hover_pos()
                            .is_some_and(|pos| block.rect.contains(pos));
                        if is_hovered {
                            fill_color = fill_color.linear_multiply(1.3);
                            border_color = egui::Color32::WHITE;
                            border_width = 1.5;
                            hovered_block = Some(block);
                        }

                        // Check if block selected
                        if self.selected_node_idx == Some(block.node_idx) {
                            border_color = egui::Color32::from_rgb(139, 92, 246); // Accent purple border
                            border_width = 2.5;
                        }

                        let stroke = if border_width > 0.0 {
                            egui::Stroke::new(border_width, border_color)
                        } else {
                            egui::Stroke::NONE
                        };

                        // Draw vertical HSL/HSV visual gradient with egui::Mesh
                        let color_light = fill_color.linear_multiply(1.15);
                        let color_dark = fill_color.linear_multiply(0.75);

                        let mut mesh = egui::Mesh::default();
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.left_top(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_light,
                        });
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.right_top(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_light,
                        });
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.right_bottom(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_dark,
                        });
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: block.rect.left_bottom(),
                            uv: egui::epaint::WHITE_UV,
                            color: color_dark,
                        });
                        mesh.add_triangle(0, 1, 2);
                        mesh.add_triangle(0, 2, 3);
                        painter.add(mesh);

                        // Draw outline border if hovered or selected
                        if border_width > 0.0 {
                            painter.rect(
                                block.rect,
                                0.0,
                                egui::Color32::TRANSPARENT,
                                stroke,
                                egui::StrokeKind::Inside,
                            );
                        }
                    }

                    // Click event to select node
                    if response.clicked() {
                        let pointer_pos = response.interact_pointer_pos();
                        if let Some(pos) = pointer_pos {
                            for block in &blocks {
                                if block.rect.contains(pos) {
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
                                    break;
                                }
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

        let response = ui
            .horizontal(|ui| {
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

                // Node Name / Selectable label with automatic left-aligned truncation
                let is_selected = self.selected_node_idx == Some(node_idx);
                let mut rich_name = egui::RichText::new(name);
                if self.monospace_paths {
                    rich_name = rich_name.monospace();
                }

                // Allocate exactly the remaining width minus space for the size column (72px subtracted for a beautifully balanced gap)
                let name_width = (ui.available_width() - 72.0).max(50.0);

                ui.allocate_ui(egui::vec2(name_width, ui.spacing().interact_size.y), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    let name_label = ui.selectable_label(is_selected, rich_name);
                    if name_label.clicked() {
                        self.selected_node_idx = Some(node_idx);
                    }
                });

                // Muted size details (far right aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        prettier_bytes::ByteFormatter::new()
                            .format(node.size)
                            .to_string(),
                    );
                });
            })
            .response;

        // Select the path first when right-clicked
        if response.clicked_by(egui::PointerButton::Secondary) {
            self.selected_node_idx = Some(node_idx);
        }

        // Render context menu on right-click
        response.context_menu(|ui| {
            let has_selection = self.selected_node_idx.is_some();
            let open_btn = ui.add_enabled(has_selection, egui::Button::new("Open in File Manager"));
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
                ui.close_kind(egui::UiKind::Menu);
            }

            let delete_btn =
                ui.add_enabled(has_selection, egui::Button::new("🗑 Delete (Permanent)"));
            if delete_btn.clicked() {
                self.active_modal = Some(ActiveModal::Delete);
                self.delete_confirm_checked = false;
                self.delete_node_idx = self.selected_node_idx;
                ui.close_kind(egui::UiKind::Menu);
            }
        });

        // Draw vertical indentation guidelines to visually track nested containers
        let rect = response.rect;
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

// Slice-and-dice partitioning treemap algorithm
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

fn build_treemap(
    config: &TreemapConfig,
    node_idx: u32,
    rect: egui::Rect,
    depth: usize,
    blocks: &mut Vec<TreemapBlock>,
) {
    let node = &config.nodes[node_idx as usize];
    if node.size == 0 || rect.width() < 2.0 || rect.height() < 2.0 {
        return;
    }

    // Leaf files or max depth limit
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

    // Collect children
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

    // Sort by size descending
    children.sort_by(|&a, &b| {
        config.nodes[b as usize]
            .size
            .cmp(&config.nodes[a as usize].size)
    });

    #[allow(clippy::cast_precision_loss)]
    let total_size = children
        .iter()
        .map(|&idx| config.nodes[idx as usize].size)
        .sum::<u64>() as f64;

    if total_size == 0.0 {
        return;
    }

    // Find the last child index that is actually drawn (size > 0)
    let last_drawn_idx = children
        .iter()
        .copied()
        .rev()
        .find(|&idx| config.nodes[idx as usize].size > 0);

    let mut remaining_rect = rect;
    let mut remaining_size = total_size;

    for &child_idx in &children {
        let child = &config.nodes[child_idx as usize];
        if child.size == 0 {
            continue;
        }

        // Dynamically slice along the longer axis of the remaining space
        let vertical = remaining_rect.height() > remaining_rect.width();
        let is_last = Some(child_idx) == last_drawn_idx;

        #[allow(clippy::cast_precision_loss)]
        let child_ratio = child.size as f64 / remaining_size;

        let child_rect;
        if is_last {
            child_rect = remaining_rect;
        } else {
            if vertical {
                let height = (remaining_rect.height() as f64 * child_ratio) as f32;
                child_rect = egui::Rect::from_min_max(
                    remaining_rect.min,
                    egui::pos2(remaining_rect.max.x, remaining_rect.min.y + height),
                );
                remaining_rect.min.y += height;
            } else {
                let width = (remaining_rect.width() as f64 * child_ratio) as f32;
                child_rect = egui::Rect::from_min_max(
                    remaining_rect.min,
                    egui::pos2(remaining_rect.min.x + width, remaining_rect.max.y),
                );
                remaining_rect.min.x += width;
            }

            #[allow(clippy::cast_precision_loss)]
            let size_acc = child.size as f64;

            remaining_size -= size_acc;
        }

        // Recurse down one level
        let start_blocks_len = blocks.len();

        #[allow(clippy::cast_precision_loss)]
        let child_ratio_of_total = child.size as f64 / total_size;

        let next_depth = if child_ratio_of_total < 0.005 {
            config.max_depth
        } else {
            depth + 1
        };
        build_treemap(config, child_idx, child_rect, next_depth, blocks);

        // Fallback: If the subdirectory did not draw any children (due to pruning/filtering),
        // draw the subdirectory itself as a single solid colored block to guarantee zero gaps!
        if blocks.len() == start_blocks_len {
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
        }
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
