use std::{
    borrow::Cow,
    sync::{Arc, mpsc::Sender},
};

#[cfg(not(target_family = "wasm"))]
use std::path::Path;

use egui_table_kit::{
    error::TableError,
    operations::{OperationContext, TableOperation, TableOperationEnablement},
};
use fluent_zero::t;

use crate::{arena::FileArenaSnapshot, state::SharedState};

#[derive(Debug, Clone)]
pub enum BackgroundOpResult {
    Deletion {
        successfully_deleted: Vec<u32>,
        failures: Vec<(String, String, bool)>, // (path, error_msg, is_permission_denied)
        to_trash: bool,
        snapshot: Arc<FileArenaSnapshot>,
    },
    Hardlinking {
        successfully_linked: Vec<u32>,
        failures: Vec<(String, String, bool)>, // (path, error_msg, is_permission_denied)
        snapshot: Arc<FileArenaSnapshot>,
    },
    Softlinking {
        successfully_linked: Vec<u32>,
        failures: Vec<(String, String, bool)>, // (path, error_msg, is_permission_denied)
        snapshot: Arc<FileArenaSnapshot>,
    },
}

/// Decoupled actions sent from the `TableOperations` directly to the main `GuiApp` loop.
#[derive(Debug, Clone)]
pub enum AppCommand {
    RefreshSubtrees(Vec<u32>),
    ScrollToSelected,
    ShowTrashModal(Vec<u32>),
    ShowDeleteModal(Vec<u32>),
    BackgroundOpCompleted(BackgroundOpResult),
    /// A snapshot file picked in the browser, delivered as raw bytes
    /// (used by the wasm frontend's async file picker).
    LoadSnapshotBytes {
        name: String,
        bytes: Vec<u8>,
    },
}

// Helper to retrieve the current snapshot safely
fn get_snapshot(shared_state: &Arc<SharedState>) -> Arc<FileArenaSnapshot> {
    shared_state.current_snapshot.load().clone()
}

// --- Up One Level ---
#[derive(Debug)]
pub struct UpOneLevelOp {
    shared_state: Arc<SharedState>,
    command_tx: Sender<AppCommand>,
}

impl UpOneLevelOp {
    pub const fn new(shared_state: Arc<SharedState>, command_tx: Sender<AppCommand>) -> Self {
        Self {
            shared_state,
            command_tx,
        }
    }
}

impl TableOperation for UpOneLevelOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-up-one-level")
    }

    fn icon(&self) -> &'static str {
        "⏶"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::OneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        if let Some(idx) = ctx.data.selected_rows.iter().next()
            && (idx as usize) < snapshot.nodes.len()
        {
            let parent = snapshot.nodes[idx as usize].parent;
            if parent == crate::arena::NO_INDEX {
                crate::gui::toast_warning(t!("toast-already-root"));
            } else {
                ctx.data.selected_rows.clear();
                ctx.data.selected_rows.insert(parent);
                let _ = self.command_tx.send(AppCommand::ScrollToSelected);
                crate::gui::toast_info(t!("toast-navigated-up"));
            }
        }
        Ok(())
    }
}

// --- Refresh Entire Scan (Root) ---
#[derive(Debug)]
pub struct RefreshRootOp {
    shared_state: Arc<SharedState>,
    command_tx: Sender<AppCommand>,
}

impl RefreshRootOp {
    pub const fn new(shared_state: Arc<SharedState>, command_tx: Sender<AppCommand>) -> Self {
        Self {
            shared_state,
            command_tx,
        }
    }
}

impl TableOperation for RefreshRootOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-refresh-entire-scan")
    }

    fn icon(&self) -> &'static str {
        "🔁"
    }

    // Always enabled, regardless of whether a row is selected
    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::Always
    }

    fn exec(&mut self, _ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        // Safety check to ensure we only refresh if a tree is actually loaded
        if !snapshot.nodes.is_empty() {
            // The root node is always strictly at index 0 in the arena
            let _ = self.command_tx.send(AppCommand::RefreshSubtrees(vec![0]));
            crate::gui::toast_info(t!("toast-refreshing-scan"));
        }

        Ok(())
    }
}

// --- Refresh Directory ---
#[derive(Debug)]
pub struct RefreshDirectoryOp {
    shared_state: Arc<SharedState>,
    command_tx: Sender<AppCommand>,
}

impl RefreshDirectoryOp {
    pub const fn new(shared_state: Arc<SharedState>, command_tx: Sender<AppCommand>) -> Self {
        Self {
            shared_state,
            command_tx,
        }
    }
}

impl TableOperation for RefreshDirectoryOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-refresh-directory")
    }

    fn icon(&self) -> &'static str {
        "🔄"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::AtLeastOneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        let dirs: Vec<u32> = ctx
            .data
            .selected_rows
            .iter()
            .filter(|&idx| {
                (idx as usize) < snapshot.nodes.len() && snapshot.nodes[idx as usize].is_directory()
            })
            .collect();

        if !dirs.is_empty() {
            let _ = self.command_tx.send(AppCommand::RefreshSubtrees(dirs));
            crate::gui::toast_info(t!("toast-refreshing-dir"));
        }
        Ok(())
    }
}

// --- Open in File Manager ---
#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct OpenFileManagerOp {
    shared_state: Arc<SharedState>,
}

#[cfg(not(target_family = "wasm"))]
impl OpenFileManagerOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

#[cfg(not(target_family = "wasm"))]
impl TableOperation for OpenFileManagerOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-open-file-manager")
    }

    fn icon(&self) -> &'static str {
        "🗁"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::OneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        if let Some(idx) = ctx.data.selected_rows.iter().next() {
            let path_str = snapshot.get_full_path(idx);
            let path = Path::new(&path_str);
            let dir_to_open = if path.is_dir() {
                path
            } else {
                path.parent().map_or(path, |p| p)
            };
            match open::that(dir_to_open) {
                Ok(()) => {
                    let path_lossy = dir_to_open.to_string_lossy();
                    let cleaned_path = crate::arena::clean_unc_path(&path_lossy);
                    crate::gui::toast_info(
                        t!("toast-opened-manager", { "path" => cleaned_path.as_ref() }),
                    );
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    crate::gui::toast_error(
                        t!("toast-failed-open-manager", { "error" => err_msg.as_str() }),
                    );
                }
            }
        }
        Ok(())
    }
}

// --- Open Terminal Here ---
#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct OpenTerminalOp {
    shared_state: Arc<SharedState>,
}

#[cfg(not(target_family = "wasm"))]
impl OpenTerminalOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

#[cfg(not(target_family = "wasm"))]
impl TableOperation for OpenTerminalOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-open-terminal")
    }

    fn icon(&self) -> &'static str {
        "💻"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::OneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        if let Some(idx) = ctx.data.selected_rows.iter().next()
            && (idx as usize) < snapshot.nodes.len()
            && snapshot.nodes[idx as usize].is_directory()
        {
            let path_str = snapshot.get_full_path(idx);
            match super::open_terminal_at(Path::new(&path_str)) {
                Ok(()) => crate::gui::toast_info(
                    t!("toast-opened-terminal", { "path" => path_str.as_str() }),
                ),
                Err(e) => {
                    let err_msg = e.to_string();
                    crate::gui::toast_error(
                        t!("toast-failed-open-terminal", { "error" => err_msg.as_str() }),
                    );
                }
            }
        }
        Ok(())
    }
}

// --- Copy Full Path ---
#[derive(Debug)]
pub struct CopyPathOp {
    shared_state: Arc<SharedState>,
}

impl CopyPathOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

impl TableOperation for CopyPathOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-copy-path")
    }

    fn icon(&self) -> &'static str {
        "📎"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::AtLeastOneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        let mut paths = Vec::new();
        let mut selected: Vec<u32> = ctx.data.selected_rows.iter().collect();
        selected.sort_unstable();
        for idx in selected {
            let path_str = snapshot.get_full_path(idx);
            paths.push(crate::arena::clean_unc_path(&path_str).into_owned());
        }

        let num_paths = paths.len();
        ctx.ui.ctx().copy_text(paths.join("\n"));
        crate::gui::toast_success(t!("toast-copied-paths", { "count" => num_paths }));
        Ok(())
    }
}

// --- Copy Name Only ---
#[derive(Debug)]
pub struct CopyNameOp {
    shared_state: Arc<SharedState>,
}

impl CopyNameOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

impl TableOperation for CopyNameOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-copy-name")
    }

    fn icon(&self) -> &'static str {
        "📋"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::AtLeastOneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let snapshot = get_snapshot(&self.shared_state);

        let mut names = Vec::new();
        let mut selected: Vec<u32> = ctx.data.selected_rows.iter().collect();
        selected.sort_unstable();
        for idx in selected {
            if (idx as usize) < snapshot.nodes.len() {
                let node = &snapshot.nodes[idx as usize];
                let name = snapshot.string_pool.get(node.name_id).unwrap_or("unknown");
                let cleaned_name = if node.parent_opt().is_none() {
                    crate::arena::clean_unc_path(name).into_owned()
                } else {
                    name.to_string()
                };
                names.push(cleaned_name);
            }
        }

        let num_names = names.len();
        ctx.ui.ctx().copy_text(names.join("\n"));
        crate::gui::toast_success(t!("toast-copied-names", { "count" => num_names }));
        Ok(())
    }
}

// --- Trash Selected ---
#[derive(Debug)]
pub struct TrashSelectedOp {
    command_tx: Sender<AppCommand>,
}

impl TrashSelectedOp {
    #[must_use]
    pub const fn new(command_tx: Sender<AppCommand>) -> Self {
        Self { command_tx }
    }
}

impl TableOperation for TrashSelectedOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-move-trash")
    }

    fn icon(&self) -> &'static str {
        "♻"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::AtLeastOneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let targets: Vec<u32> = ctx.data.selected_rows.iter().collect();
        let _ = self.command_tx.send(AppCommand::ShowTrashModal(targets));
        Ok(())
    }
}

// --- Permanently Delete Selected ---
#[derive(Debug)]
pub struct DeleteSelectedOp {
    command_tx: Sender<AppCommand>,
}

impl DeleteSelectedOp {
    #[must_use]
    pub const fn new(command_tx: Sender<AppCommand>) -> Self {
        Self { command_tx }
    }
}

impl TableOperation for DeleteSelectedOp {
    fn name(&self) -> Cow<'_, str> {
        t!("op-permanently-delete")
    }

    fn icon(&self) -> &'static str {
        "🗑"
    }

    fn enabled(&self) -> TableOperationEnablement {
        TableOperationEnablement::AtLeastOneSelected
    }

    fn exec(&mut self, ctx: &mut OperationContext<'_, '_>) -> Result<(), TableError> {
        let targets: Vec<u32> = ctx.data.selected_rows.iter().collect();
        let _ = self.command_tx.send(AppCommand::ShowDeleteModal(targets));
        Ok(())
    }
}
