use std::{
    borrow::Cow,
    path::Path,
    sync::{Arc, mpsc::Sender},
};

use egui_table_kit::{
    error::TableError,
    operations::{OperationContext, TableOperation, TableOperationEnablement},
};

use crate::{arena::FileArenaSnapshot, coordinator::SharedState};

/// Decoupled actions sent from the `TableOperations` directly to the main `GuiApp` loop.
#[derive(Debug, Clone)]
pub enum AppCommand {
    RefreshSubtrees(Vec<u32>),
    ScrollToSelected,
    ShowTrashModal(Vec<u32>),
    ShowDeleteModal(Vec<u32>),
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
        Cow::Borrowed("Up One Level")
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
            if parent != crate::arena::NO_INDEX {
                ctx.data.selected_rows.clear();
                ctx.data.selected_rows.insert(parent);
                let _ = self.command_tx.send(AppCommand::ScrollToSelected);
            }
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
        Cow::Borrowed("Refresh Directory")
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
        }
        Ok(())
    }
}

// --- Open in File Manager ---
#[derive(Debug)]
pub struct OpenFileManagerOp {
    shared_state: Arc<SharedState>,
}

impl OpenFileManagerOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

impl TableOperation for OpenFileManagerOp {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("Open in File Manager")
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
            let _ = open::that(dir_to_open);
        }
        Ok(())
    }
}

// --- Open Terminal Here ---
#[derive(Debug)]
pub struct OpenTerminalOp {
    shared_state: Arc<SharedState>,
}

impl OpenTerminalOp {
    pub const fn new(shared_state: Arc<SharedState>) -> Self {
        Self { shared_state }
    }
}

impl TableOperation for OpenTerminalOp {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("Open Terminal Here")
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
            let _ = super::open_terminal_at(Path::new(&path_str));
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
        Cow::Borrowed("Copy Path")
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
            paths.push(snapshot.get_full_path(idx));
        }

        ctx.ui.ctx().copy_text(paths.join("\n"));
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
        Cow::Borrowed("Copy Name")
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
                names.push(name.to_string());
            }
        }

        ctx.ui.ctx().copy_text(names.join("\n"));
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
        Cow::Borrowed("Move to Trash")
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
        Cow::Borrowed("Permanently Delete")
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
