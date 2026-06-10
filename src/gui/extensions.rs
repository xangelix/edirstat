use eframe::egui;

use super::{GuiApp, theme};

pub struct ExtensionStat {
    pub ext: String,
    pub total_size: u64,
    pub file_count: u32,
    pub color: egui::Color32,
}

impl GuiApp {
    pub fn draw_extensions_contents(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.vertical(|ui| {
            ui.heading(
                egui::RichText::new("📂 Extensions")
                    .strong()
                    .color(ui.visuals().strong_text_color()),
            );
            ui.separator();

            // 1. O(1) Pointer Cache Check: Only map and allocate when stats actually change
            let shared_ext_stats = self.shared_state.extension_stats.load();
            let current_ptr = std::sync::Arc::as_ptr(&shared_ext_stats) as usize;

            if self.last_extension_stats_ptr != current_ptr {
                self.extension_stats = shared_ext_stats
                    .iter()
                    .map(|(ext, total_size, file_count)| ExtensionStat {
                        ext: ext.clone(),
                        total_size: *total_size,
                        file_count: *file_count,
                        color: theme::get_color_for_extension(ext),
                    })
                    .collect();
                self.last_extension_stats_ptr = current_ptr;
            }

            if self.extension_stats.is_empty() {
                ui.label("No statistics gathered yet.");
            } else {
                // 2. Lazy Viewport Rendering: Render only the rows visible on screen
                let row_height = 20.0;
                let total_rows = self.extension_stats.len();

                egui::ScrollArea::vertical().show_rows(
                    ui,
                    row_height,
                    total_rows,
                    |ui, row_range| {
                        for idx in row_range {
                            let stat = &self.extension_stats[idx];
                            ui.horizontal(|ui| {
                                ui.set_min_height(row_height);

                                // Colored dot
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(rect.center(), 4.5, stat.color);

                                // Column alignment pre-calculations (avoids nesting heavy UI containers)
                                let available_width = ui.available_width();
                                let size_str = prettier_bytes::ByteFormatter::new()
                                    .format(stat.total_size)
                                    .to_string();

                                let size_width = 72.0;
                                let name_width = (available_width - size_width - 8.0).max(10.0);

                                // Left Column: Extension Name
                                ui.allocate_ui_with_layout(
                                    egui::vec2(name_width, row_height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.style_mut().wrap_mode =
                                            Some(egui::TextWrapMode::Truncate);
                                        ui.label(&stat.ext)
                                            .on_hover_text(format!("Files: {}", stat.file_count));
                                    },
                                );

                                // Right Column: Space Consumed
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), row_height),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(size_str);
                                    },
                                );
                            });
                        }
                    },
                );
            }
        });
    }

    pub fn render_extension_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("right_panel")
            .resizable(true)
            .size_range(80.0..=250.0)
            .default_size(210.0)
            .show_inside(ui, |ui| {
                self.draw_extensions_contents(ui);
            });
    }
}
