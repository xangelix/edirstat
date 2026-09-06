use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, WidgetText};

// --- Application Shortcuts Catalog ---

/// Trigger new scan modal: `⌘O` on macOS, `Ctrl+O` on Windows/Linux.
pub const SHORTCUT_NEW_SCAN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);

/// Rescan active directory: `⌘R` on macOS, `Ctrl+R` on Windows/Linux.
pub const SHORTCUT_RESCAN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);

/// Secondary shortcut for rescan: `F5` across all platforms.
pub const SHORTCUT_RESCAN_F5: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);

/// Save current snapshot: `⌘S` on macOS, `Ctrl+S` on Windows/Linux.
pub const SHORTCUT_SAVE_SNAPSHOT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

/// Focus filter/search input: `⌘F` on macOS, `Ctrl+F` on Windows/Linux.
pub const SHORTCUT_SEARCH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);

/// Close active modal or scan: `⌘W` on macOS, `Ctrl+W` on Windows/Linux.
pub const SHORTCUT_CLOSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::W);

/// Gracefully quit application: `⌘Q` on macOS, `Ctrl+Q` on Windows/Linux.
pub const SHORTCUT_QUIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);

/// Toggle left tree panel in Classic layout: `F9`.
pub const SHORTCUT_TOGGLE_LEFT_PANEL: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::NONE, Key::F9);

/// Toggle right details/chart panel: `F11`.
pub const SHORTCUT_TOGGLE_RIGHT_PANEL: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::NONE, Key::F11);

/// Collapse all directory rows in tree view: `⇧⌘C` on macOS, `Ctrl+Shift+C` on Windows/Linux.
pub const SHORTCUT_COLLAPSE_ALL: KeyboardShortcut = KeyboardShortcut::new(
    Modifiers {
        alt: false,
        ctrl: false,
        shift: true,
        mac_cmd: true,
        command: true,
    },
    Key::C,
);

/// Move selected items to trash: `Delete`.
pub const SHORTCUT_TRASH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Delete);

/// Permanently delete selected items: `Shift+Delete`.
pub const SHORTCUT_DELETE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::SHIFT, Key::Delete);

/// Navigate up one level in treemap or explorer: `Alt+Up`.
pub const SHORTCUT_UP_ONE_LEVEL: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::ALT, Key::ArrowUp);

/// Focus/zoom treemap to selected directory: `Enter`.
pub const SHORTCUT_ZOOM_SELECTION: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::NONE, Key::Enter);

/// Open About / Help modal: `F1`.
pub const SHORTCUT_ABOUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F1);

/// Formats a keyboard shortcut using egui's platform context (e.g. `⌘O` on macOS vs `Ctrl+O` on Windows/Linux).
#[must_use]
pub fn format_shortcut(ctx: &egui::Context, shortcut: &KeyboardShortcut) -> String {
    ctx.format_shortcut(shortcut)
}

/// Creates an `egui::Button` with a right-aligned shortcut badge derived from the platform context.
pub fn button_with_shortcut<'a>(
    label: impl Into<WidgetText>,
    shortcut: &KeyboardShortcut,
    ctx: &egui::Context,
) -> egui::Button<'a> {
    egui::Button::new(label).shortcut_text(ctx.format_shortcut(shortcut))
}

/// Creates an `egui::Button` with an explicit shortcut string (e.g., `"F9"`, `"F11"`, `"Del"`).
pub fn button_with_shortcut_str(
    label: impl Into<WidgetText>,
    shortcut_str: &str,
) -> egui::Button<'_> {
    egui::Button::new(label).shortcut_text(shortcut_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_definitions() {
        assert_eq!(SHORTCUT_NEW_SCAN.logical_key, Key::O);
        const { assert!(SHORTCUT_NEW_SCAN.modifiers.command) };

        assert_eq!(SHORTCUT_RESCAN.logical_key, Key::R);
        const { assert!(SHORTCUT_RESCAN.modifiers.command) };

        assert_eq!(SHORTCUT_RESCAN_F5.logical_key, Key::F5);
        assert_eq!(SHORTCUT_SAVE_SNAPSHOT.logical_key, Key::S);
        assert_eq!(SHORTCUT_SEARCH.logical_key, Key::F);
        assert_eq!(SHORTCUT_CLOSE.logical_key, Key::W);
        assert_eq!(SHORTCUT_QUIT.logical_key, Key::Q);
        assert_eq!(SHORTCUT_TOGGLE_LEFT_PANEL.logical_key, Key::F9);
        assert_eq!(SHORTCUT_TOGGLE_RIGHT_PANEL.logical_key, Key::F11);
        assert_eq!(SHORTCUT_TRASH.logical_key, Key::Delete);
        assert_eq!(SHORTCUT_DELETE.logical_key, Key::Delete);
        const { assert!(SHORTCUT_DELETE.modifiers.shift) };
        assert_eq!(SHORTCUT_ABOUT.logical_key, Key::F1);
        assert_eq!(SHORTCUT_UP_ONE_LEVEL.logical_key, Key::ArrowUp);
        const { assert!(SHORTCUT_UP_ONE_LEVEL.modifiers.alt) };
        assert_eq!(SHORTCUT_ZOOM_SELECTION.logical_key, Key::Enter);
    }

    fn test_context() -> egui::Context {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        ctx
    }

    #[test]
    fn test_format_shortcut_context() {
        let ctx = test_context();
        let formatted = format_shortcut(&ctx, &SHORTCUT_NEW_SCAN);
        assert!(!formatted.is_empty());
        assert!(formatted.contains('O') || formatted.contains('o'));

        ctx.set_os(egui::os::OperatingSystem::Mac);
        let formatted_mac = format_shortcut(&ctx, &SHORTCUT_NEW_SCAN);
        assert!(!formatted_mac.is_empty());
    }

    #[test]
    fn test_button_builders() {
        let ctx = test_context();
        let _btn = button_with_shortcut("Scan", &SHORTCUT_NEW_SCAN, &ctx);
        let _btn_str = button_with_shortcut_str("Toggle", "F9");

        ctx.set_os(egui::os::OperatingSystem::Mac);
        let _btn_mac = button_with_shortcut("Scan", &SHORTCUT_NEW_SCAN, &ctx);
    }
}
