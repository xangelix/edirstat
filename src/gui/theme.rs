use eframe::egui::{self, Color32};

// --- Custom Theme Palette Constants ---

// UI Status Indicators
pub const COLOR_SCANNING: Color32 = Color32::from_rgb(139, 92, 246); // Purple/violet
pub const COLOR_SCAN_COMPLETE: Color32 = Color32::from_rgb(34, 197, 94); // Green

// Treemap Highlight Overlays
pub const GLOW_OUTER_BASE: Color32 = Color32::from_rgb(246, 92, 92); // Gentle red
pub const GLOW_INNER_CORE: Color32 = Color32::from_rgb(253, 181, 181); // Even lighter red
pub const TREEMAP_DIR_FALLBACK: Color32 = Color32::from_gray(100);

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
            // Hash the extension to generate a stable, beautiful pseudo-random color
            let mut hash: u32 = 5381;
            for b in ext_lower.bytes() {
                hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
            }
            // Hue from hash, Saturation ~75%, Lightness ~55%
            #[allow(clippy::cast_precision_loss)]
            let hue = (hash % 360) as f32 / 360.0;

            let color = egui::epaint::Hsva::new(hue, 0.75, 0.55, 1.0);
            egui::Color32::from(color)
        }
    }
}

// Custom Glassmorphic Dark styling settings
pub fn setup_custom_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Background Slate Color
    visuals.panel_fill = BG_PANEL_SLATE;
    visuals.window_fill = BG_WINDOW_SLATE;

    // Borders
    visuals.widgets.noninteractive.bg_fill = BG_WINDOW_SLATE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, STROKE_BORDER_SLATE);

    ctx.set_visuals(visuals);
}
