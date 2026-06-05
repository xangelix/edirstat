pub mod deduplicator;
pub mod dir_composition;
pub mod duplicate_waste;
pub mod extension_boxplot;
pub mod scatter_plot;
pub mod size_distribution;
pub mod temporal_timeline;
pub mod treemap;

pub trait StatsChart {
    type Output;

    /// Iteratively computes or updates the chart's visual/plot data
    /// using the latest thread-safe snapshot frame.
    fn compute(&mut self, snapshot: &crate::arena::FileArenaSnapshot) -> Self::Output;
}

pub struct StatContext<'a> {
    pub selected_node_idx: &'a mut Option<u32>,
    pub expanded_nodes: &'a mut std::collections::HashSet<u32>,
    pub scroll_to_selected: &'a mut bool,
    pub deduplicator_results: Option<
        &'a std::sync::Arc<parking_lot::RwLock<crate::stats::deduplicator::DeduplicationResults>>,
    >,
}

pub trait StatComponent {
    /// Renders the statistics/visualization component inside the given UI.
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        context: &mut StatContext,
    );
}
