use std::cell::RefCell;

use eframe::egui::{self, Color32};

// --- Custom Theme Palette Constants ---

// UI Status Indicators
pub const COLOR_SCANNING: Color32 = Color32::from_rgb(139, 92, 246); // Purple/violet
pub const COLOR_SCAN_COMPLETE: Color32 = Color32::from_rgb(34, 197, 94); // Green

// Treemap Highlight Overlays
pub const GLOW_OUTER_BASE: Color32 = Color32::from_rgb(246, 92, 92); // Gentle red
pub const GLOW_INNER_CORE: Color32 = Color32::from_rgb(253, 181, 181); // Even lighter red
pub const TREEMAP_DIR_FALLBACK: Color32 = Color32::from_gray(100);
pub const TREEMAP_BORDER_COLOR: Color32 = Color32::from_rgb(22, 24, 28);

// Deletion Modal
pub const DELETION_BORDER: Color32 = Color32::from_rgb(220, 38, 38); // Dark red
pub const DELETION_WARNING: Color32 = Color32::from_rgb(239, 68, 68); // Bright red

// Trash Modal
pub const TRASH_BORDER: Color32 = Color32::from_rgb(37, 99, 235); // Dark blue/indigo
pub const TRASH_WARNING: Color32 = Color32::from_rgb(96, 165, 250); // Light blue

// Directory Tree Guidelines
pub const INDENT_GUIDELINE: Color32 = Color32::from_gray(65);

// Glassmorphic Custom Canvas Styling
pub const BG_PANEL_SLATE: Color32 = Color32::from_rgb(18, 20, 28);
pub const BG_WINDOW_SLATE: Color32 = Color32::from_rgb(26, 29, 38);
pub const STROKE_BORDER_SLATE: Color32 = Color32::from_rgb(38, 43, 56);

// Extension Groupings (Harmonious Palette)
pub const EXT_RUST: Color32 = Color32::from_rgb(239, 68, 68); // Rust red
pub const EXT_TOML: Color32 = Color32::from_rgb(59, 130, 246); // Toml blue
pub const EXT_GIT: Color32 = Color32::from_rgb(107, 114, 128); // Git gray
pub const EXT_JS_TS: Color32 = Color32::from_rgb(234, 179, 8); // JS yellow
pub const EXT_CONFIG: Color32 = Color32::from_rgb(168, 85, 247); // Purple config
pub const EXT_WEB: Color32 = Color32::from_rgb(249, 115, 22); // HTML/CSS orange
pub const EXT_PYTHON: Color32 = Color32::from_rgb(16, 185, 129); // Python green
pub const EXT_CPP: Color32 = Color32::from_rgb(6, 182, 212); // C/C++ cyan
pub const EXT_COMPRESSED: Color32 = Color32::from_rgb(236, 72, 153); // Compressed pink
pub const EXT_AUDIO: Color32 = Color32::from_rgb(14, 165, 233); // Audio sky-blue
pub const EXT_VIDEO: Color32 = Color32::from_rgb(20, 184, 166); // Video teal
pub const EXT_IMAGE: Color32 = Color32::from_rgb(244, 63, 94); // Image rose
pub const EXT_NONE: Color32 = Color32::from_rgb(75, 85, 99); // Muted dark gray

// Bright blue button styling (for select items button)
pub const BUTTON_BLUE: Color32 = Color32::from_rgb(59, 130, 246); // Toml blue / bright blue
pub const BUTTON_BLUE_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // Light blue for hover

// Orange button styling (for hardlink button)
pub const BUTTON_ORANGE: Color32 = Color32::from_rgb(217, 119, 6); // Amber-600
pub const BUTTON_ORANGE_HOVER: Color32 = Color32::from_rgb(245, 158, 11); // Amber-500

// General warnings & indicators
pub const WARNING_RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const COLOR_WARNING_YELLOW: Color32 = Color32::YELLOW;
pub const COLOR_DUPLICATE_ORANGE: Color32 = Color32::from_rgb(245, 158, 11);
pub const COLOR_LIGHT_GREEN: Color32 = Color32::from_rgb(134, 239, 172);

// Standard utility colors to avoid direct egui::Color32 references
pub const COLOR_WHITE: Color32 = Color32::WHITE;
pub const COLOR_TRANSPARENT: Color32 = Color32::TRANSPARENT;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AppTheme {
    #[default]
    Dark,
    HighContrast,
    Light,
}

thread_local! {
    static CURRENT_THEME: std::cell::Cell<AppTheme> = const { std::cell::Cell::new(AppTheme::Dark) };

    /// Caches calculated colors for custom extensions to avoid expensive
    /// Hsva -> Rgba -> Color32 (powf) conversions in the render loop.
    static COLOR_CACHE: RefCell<ahash::HashMap<String, egui::Color32>> =
        RefCell::new(ahash::HashMap::default());
}

pub fn get_current_theme() -> AppTheme {
    CURRENT_THEME.with(std::cell::Cell::get)
}

pub fn set_current_theme(theme: AppTheme) {
    CURRENT_THEME.with(|t| t.set(theme));
}

pub fn clear_color_cache() {
    COLOR_CACHE.with(|cache| cache.borrow_mut().clear());
}

#[must_use]
pub fn get_bg_panel() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => BG_PANEL_SLATE,
        AppTheme::HighContrast => Color32::from_rgb(0, 0, 0),
        AppTheme::Light => Color32::from_rgb(245, 245, 247),
    }
}

#[must_use]
pub fn get_bg_window() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => BG_WINDOW_SLATE,
        AppTheme::HighContrast => Color32::from_rgb(0, 0, 0),
        AppTheme::Light => Color32::from_rgb(255, 255, 255),
    }
}

#[must_use]
pub fn get_stroke_border() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => STROKE_BORDER_SLATE,
        AppTheme::HighContrast => Color32::from_rgb(235, 235, 235), // Soft off-white
        AppTheme::Light => Color32::from_rgb(220, 220, 224),
    }
}

#[must_use]
pub fn get_indent_guideline() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => INDENT_GUIDELINE,
        AppTheme::HighContrast => Color32::from_gray(180),
        AppTheme::Light => Color32::from_gray(200),
    }
}

#[must_use]
pub fn get_color_scanning() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(255, 0, 255),
        _ => COLOR_SCANNING,
    }
}

#[must_use]
pub fn get_color_scan_complete() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(0, 255, 0),
        _ => COLOR_SCAN_COMPLETE,
    }
}

#[must_use]
pub fn get_glow_outer_base() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => GLOW_OUTER_BASE,
        AppTheme::HighContrast => Color32::from_rgb(0, 255, 255),
        AppTheme::Light => Color32::from_rgb(0, 120, 255),
    }
}

#[must_use]
pub fn get_glow_inner_core() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => GLOW_INNER_CORE,
        AppTheme::HighContrast => Color32::from_rgb(235, 235, 235), // Soft off-white
        AppTheme::Light => Color32::from_rgb(0, 80, 200),
    }
}

#[must_use]
pub fn get_treemap_border_color() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark | AppTheme::HighContrast => TREEMAP_BORDER_COLOR,
        AppTheme::Light => Color32::BLACK, // Black borders for Light theme
    }
}

#[must_use]
pub fn get_deletion_border() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(255, 0, 0),
        _ => DELETION_BORDER,
    }
}

#[must_use]
pub fn get_deletion_warning() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(255, 50, 50),
        _ => DELETION_WARNING,
    }
}

#[must_use]
pub fn get_trash_border() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(0, 0, 255),
        _ => TRASH_BORDER,
    }
}

#[must_use]
pub fn get_trash_warning() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(100, 150, 255),
        _ => TRASH_WARNING,
    }
}

#[must_use]
pub fn get_warning_red() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(255, 0, 0),
        _ => WARNING_RED,
    }
}

#[must_use]
pub fn get_button_blue() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(0, 255, 255),
        _ => BUTTON_BLUE,
    }
}

#[must_use]
pub fn get_button_blue_hover() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(235, 235, 235), // Soft off-white
        _ => BUTTON_BLUE_HOVER,
    }
}

#[must_use]
pub fn get_button_orange() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(255, 165, 0),
        _ => BUTTON_ORANGE,
    }
}

#[must_use]
pub fn get_button_orange_hover() -> Color32 {
    match get_current_theme() {
        AppTheme::HighContrast => Color32::from_rgb(235, 235, 235), // Soft off-white
        _ => BUTTON_ORANGE_HOVER,
    }
}

#[must_use]
pub fn get_treemap_dir_fallback() -> Color32 {
    match get_current_theme() {
        AppTheme::Dark => TREEMAP_DIR_FALLBACK,
        AppTheme::HighContrast => Color32::from_gray(150),
        AppTheme::Light => Color32::from_gray(180),
    }
}

#[must_use]
pub fn get_color_for_extension(ext: &str) -> egui::Color32 {
    let mut buf = [0u8; 16];
    let ext_lower = if ext.len() <= 16 && ext.bytes().any(|b| b.is_ascii_uppercase()) {
        let mut len = 0;
        for (b, dest) in ext.bytes().zip(buf.iter_mut()) {
            *dest = b.to_ascii_lowercase();
            len += 1;
        }
        std::str::from_utf8(&buf[..len]).unwrap_or(ext)
    } else {
        ext
    };

    match ext_lower {
        "rs" => EXT_RUST,
        "toml" => EXT_TOML,
        "git" | "gitignore" => EXT_GIT,
        "js" | "ts" => EXT_JS_TS,
        "json" | "yaml" => EXT_CONFIG,
        "html" | "css" => EXT_WEB,
        "py" => EXT_PYTHON,
        "c" | "cpp" | "h" => EXT_CPP,
        "zip" | "tar" | "gz" => EXT_COMPRESSED,
        "mp3" | "wav" | "flac" => EXT_AUDIO,
        "mp4" | "mkv" | "avi" => EXT_VIDEO,
        crate::arena::NO_EXTENSION => EXT_NONE,
        _ => {
            // Retrieve color from cache, or compute and insert if absent
            COLOR_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if let Some(&color) = cache.get(ext_lower) {
                    color
                } else {
                    // Hash the extension to generate a stable, pseudo-random color
                    let mut hash: u32 = 5381;
                    for b in ext_lower.bytes() {
                        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
                    }
                    // Hue from hash, Saturation ~75%, Lightness ~55%
                    #[allow(clippy::cast_precision_loss)]
                    let hue = (hash % 360) as f32 / 360.0;

                    let color = egui::epaint::Hsva::new(hue, 0.75, 0.55, 1.0);
                    let color32 = egui::Color32::from(color);

                    cache.insert(ext_lower.to_string(), color32);
                    color32
                }
            })
        }
    }
}

// Custom Glassmorphic styling settings for multiple themes
pub fn setup_custom_style(ctx: &egui::Context, theme: AppTheme) {
    set_current_theme(theme);
    let mut visuals = match theme {
        AppTheme::Dark => {
            let mut v = egui::Visuals::dark();
            v.panel_fill = BG_PANEL_SLATE;
            v.window_fill = BG_WINDOW_SLATE;
            v.widgets.noninteractive.bg_fill = BG_WINDOW_SLATE;
            v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0f32, STROKE_BORDER_SLATE);
            // Brighter table text in Dark theme
            v.widgets.noninteractive.fg_stroke.color = Color32::from_gray(220);
            v.widgets.inactive.fg_stroke.color = Color32::from_gray(200);
            v
        }
        AppTheme::HighContrast => {
            let mut v = egui::Visuals::dark();
            v.panel_fill = Color32::from_rgb(0, 0, 0);
            v.window_fill = Color32::from_rgb(0, 0, 0);
            v.extreme_bg_color = Color32::from_rgb(45, 45, 45); // Clear gray background for checkbox boxes/inputs
            v.faint_bg_color = Color32::from_rgb(25, 25, 25); // Prominent alternating row striping

            let soft_white = Color32::from_rgb(235, 235, 235);
            let bright_yellow = Color32::from_rgb(255, 255, 0);

            // High contrast noninteractive elements
            v.widgets.noninteractive.bg_fill = Color32::from_rgb(0, 0, 0);
            // noninteractive.bg_stroke.color is used by egui-table-kit to draw column header backgrounds!
            v.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)); // dark gray header row bg
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, soft_white);

            // High contrast interactive elements
            v.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45); // Visible background for checkboxes and buttons
            v.widgets.inactive.bg_stroke = egui::Stroke::new(0.0, Color32::TRANSPARENT);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, soft_white);

            // Extreme hover colors (Yellow border/text, slightly lighter gray background)
            v.widgets.hovered.bg_fill = Color32::from_rgb(65, 65, 65);
            v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, bright_yellow);
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, bright_yellow);

            // Extreme active colors (White text/border, dark blue background to keep strong_text_color bright)
            v.widgets.active.bg_fill = Color32::from_rgb(0, 0, 180);
            v.widgets.active.bg_stroke = egui::Stroke::new(1.0, soft_white);
            v.widgets.active.fg_stroke = egui::Stroke::new(1.0, soft_white);

            v.widgets.open.bg_fill = Color32::from_rgb(0, 0, 0);
            v.widgets.open.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 255));
            v.widgets.open.fg_stroke = egui::Stroke::new(1.0, soft_white);

            v.selection.bg_fill = Color32::from_rgb(80, 80, 0); // dark yellow row bg
            v.selection.stroke = egui::Stroke::new(1.0, bright_yellow); // bright yellow selected text

            v
        }
        AppTheme::Light => {
            let mut v = egui::Visuals::light();
            v.panel_fill = Color32::from_rgb(245, 245, 247);
            v.window_fill = Color32::from_rgb(255, 255, 255);
            v.extreme_bg_color = Color32::from_rgb(250, 250, 250);
            v.faint_bg_color = Color32::from_rgb(225, 225, 230); // More visible light striping

            v.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
            v.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(220, 220, 224));
            v.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(28, 28, 30));

            v.widgets.inactive.bg_fill = Color32::from_rgb(240, 240, 243);
            v.widgets.inactive.bg_stroke = egui::Stroke::new(0.0, Color32::TRANSPARENT);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(28, 28, 30));

            // Menu hover highlights the text in Royal Blue
            v.widgets.hovered.bg_fill = Color32::from_rgb(230, 230, 235);
            v.widgets.hovered.bg_stroke = egui::Stroke::new(0.0, Color32::TRANSPARENT);
            v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0, 102, 204));

            v.widgets.active.bg_fill = Color32::from_rgb(209, 209, 214);
            v.widgets.active.bg_stroke = egui::Stroke::new(0.0, Color32::TRANSPARENT);
            v.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0, 0, 0));

            v.widgets.open.bg_fill = Color32::from_rgb(245, 245, 247);
            v.widgets.open.bg_stroke = egui::Stroke::new(0.0, Color32::TRANSPARENT);
            v.widgets.open.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(28, 28, 30));

            v.selection.bg_fill = Color32::from_rgb(204, 229, 255);
            v.selection.stroke = egui::Stroke::new(0.0, Color32::from_rgb(0, 102, 204)); // Keep text color with 0 width border

            v
        }
    };

    // Eliminate widget expansion/scaling on hover or click to prevent layout shift and visual jitter
    visuals.widgets.noninteractive.expansion = 0.0;
    visuals.widgets.inactive.expansion = 0.0;
    visuals.widgets.hovered.expansion = 0.0;
    visuals.widgets.active.expansion = 0.0;
    visuals.widgets.open.expansion = 0.0;

    ctx.set_visuals(visuals);
}
