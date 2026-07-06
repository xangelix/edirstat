use eframe::egui::{Color32, Rect, pos2};
use smallvec::SmallVec;

use super::{StatComponent, StatContext, StatsChart};
use crate::arena::{FileNode, NO_INDEX, StringPool};

const TRUNCATE_DEPTH: usize = 30000;
const PIXEL_PRECISION_LIMIT: f32 = 0.2;

pub struct TreemapBlock {
    pub rect: Rect,
    pub node_idx: u32,
    pub color: Color32,
}

pub struct TreemapChart {
    pub cached_blocks: Vec<TreemapBlock>,
    pub last_snapshot_ptr: usize,
    pub last_rect: Rect,
    pub draw_borders: bool,
}

impl TreemapChart {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_blocks: Vec::new(),
            last_snapshot_ptr: 0,
            last_rect: Rect::NOTHING,
            draw_borders: false,
        }
    }
}

impl Default for TreemapChart {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsChart for TreemapChart {
    type Output = Vec<TreemapBlock>;

    fn compute(&mut self, snapshot: &crate::arena::FileArenaSnapshot) -> Self::Output {
        let mut blocks = Vec::new();
        if snapshot.nodes.is_empty() {
            return blocks;
        }

        let config = TreemapConfig {
            nodes: &snapshot.nodes,
            string_pool: &snapshot.string_pool,
            max_depth: 20,
        };

        build_treemap(&config, 0, self.last_rect, 0, &mut blocks);

        // Enforce safe memory boundary to protect the GPU staging buffer limits
        if blocks.len() > TRUNCATE_DEPTH {
            blocks.truncate(TRUNCATE_DEPTH);
        }
        blocks
    }
}

#[must_use]
pub fn get_selection_roots<S: ::std::hash::BuildHasher + Default>(
    nodes: &[FileNode],
    selected_nodes: &std::collections::HashSet<u32, S>,
) -> std::collections::HashSet<u32> {
    let mut roots = std::collections::HashSet::new();
    for &idx in selected_nodes {
        if idx as usize >= nodes.len() {
            continue;
        }
        let mut curr = nodes[idx as usize].parent;
        let mut ancestor_selected = false;
        while curr != crate::arena::NO_INDEX {
            if selected_nodes.contains(&curr) {
                ancestor_selected = true;
                break;
            }
            if curr as usize >= nodes.len() {
                break;
            }
            curr = nodes[curr as usize].parent;
        }
        if !ancestor_selected {
            roots.insert(idx);
        }
    }
    roots
}

impl StatComponent for TreemapChart {
    fn render(
        &mut self,
        ui: &mut eframe::egui::Ui,
        snapshot: &crate::arena::FileArenaSnapshot,
        context: &mut StatContext,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let (rect, response) = ui.allocate_exact_size(
            eframe::egui::vec2(available_rect.width() - 4.0, available_rect.height()),
            eframe::egui::Sense::click_and_drag(),
        );

        // --- Layout Cache Check ---
        // Race-free (the pointer and the snapshot it identifies are the same value).
        // The rescan path always publishes a fresh `Arc`, so every content change
        // invalidates this cache.
        let snapshot_ptr = std::sync::Arc::as_ptr(&snapshot.nodes) as usize;
        let needs_rebuild = self.cached_blocks.is_empty()
            || snapshot_ptr != self.last_snapshot_ptr
            || rect != self.last_rect;

        if needs_rebuild {
            let mut blocks = Vec::new();
            if !snapshot.nodes.is_empty() {
                let config = TreemapConfig {
                    nodes: &snapshot.nodes,
                    string_pool: &snapshot.string_pool,
                    max_depth: 20,
                };
                build_treemap(&config, 0, rect, 0, &mut blocks);
            }

            // Safety cap to prevent GPU staging buffer overflows on massive directories
            if blocks.len() > TRUNCATE_DEPTH {
                blocks.truncate(TRUNCATE_DEPTH);
            }

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
        let mut combined_mesh = eframe::egui::Mesh::default();
        for block in &self.cached_blocks {
            let mut render_rect = block.rect;
            let mut draw_border = false;

            if self.draw_borders && block.rect.width() > 2.0 && block.rect.height() > 2.0 {
                draw_border = true;
                render_rect = block.rect.shrink(1.0);
            }

            if draw_border {
                let border_color = crate::colors::TREEMAP_BORDER_COLOR;
                let base_vertex_idx = combined_mesh.vertices.len() as u32;

                combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                    pos: block.rect.left_top(),
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: border_color,
                });
                combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                    pos: block.rect.right_top(),
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: border_color,
                });
                combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                    pos: block.rect.right_bottom(),
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: border_color,
                });
                combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                    pos: block.rect.left_bottom(),
                    uv: eframe::egui::epaint::WHITE_UV,
                    color: border_color,
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

            let fill_color = block.color;
            let color_light = fill_color.gamma_multiply(1.15);
            let color_dark = fill_color.gamma_multiply(0.75);

            let base_vertex_idx = combined_mesh.vertices.len() as u32;

            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: render_rect.left_top(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_light,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: render_rect.right_top(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_light,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: render_rect.right_bottom(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_dark,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: render_rect.left_bottom(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_dark,
            });

            combined_mesh.add_triangle(base_vertex_idx, base_vertex_idx + 1, base_vertex_idx + 2);
            combined_mesh.add_triangle(base_vertex_idx, base_vertex_idx + 2, base_vertex_idx + 3);
        }

        painter.add(combined_mesh);

        // Dynamic overlays for highlights
        if let Some(block) = hovered_block {
            let stroke = eframe::egui::Stroke::new(1.5f32, crate::colors::COLOR_WHITE);
            painter.rect(
                block.rect,
                0.0,
                crate::colors::COLOR_TRANSPARENT,
                stroke,
                eframe::egui::StrokeKind::Inside,
            );
        }

        if !context.selected_nodes.is_empty() {
            if context.selected_nodes.len() == 1 {
                // Fast-path: Skip get_selection_roots entirely.
                // A single selected node is guaranteed to be its own selection root.
                if let Some(&root_idx) = context.selected_nodes.iter().next() {
                    let mut target_rect: Option<eframe::egui::Rect> = None;

                    for block in &self.cached_blocks {
                        if is_descendant(&snapshot.nodes, block.node_idx, root_idx) {
                            match target_rect {
                                None => target_rect = Some(block.rect),
                                Some(ref mut r) => *r = r.union(block.rect),
                            }
                        }
                    }

                    if let Some(rect) = target_rect {
                        draw_glow(ui, &painter, rect);
                    }
                }
            } else {
                // Slow-path: Multiple selections.
                let roots = get_selection_roots(&snapshot.nodes, context.selected_nodes);
                let mut root_rects: std::collections::HashMap<
                    u32,
                    eframe::egui::Rect,
                    ahash::RandomState,
                > = std::collections::HashMap::with_hasher(ahash::RandomState::new());

                for block in &self.cached_blocks {
                    let mut curr = Some(block.node_idx);
                    while let Some(idx) = curr {
                        if roots.contains(&idx) {
                            root_rects
                                .entry(idx)
                                .and_modify(|r| *r = r.union(block.rect))
                                .or_insert(block.rect);
                            break;
                        }
                        curr = snapshot
                            .nodes
                            .get(idx as usize)
                            .and_then(crate::model::arena::FileNode::parent_opt);
                    }
                }

                for (&_root_idx, &rect) in &root_rects {
                    draw_glow(ui, &painter, rect);
                }
            }
        }

        // Click event to select node
        if response.clicked()
            && let Some(block) = hovered_block
        {
            let modifiers = ui.input(|i| i.modifiers);
            if modifiers.command || modifiers.ctrl {
                if context.selected_nodes.contains(&block.node_idx) {
                    context.selected_nodes.remove(&block.node_idx);
                } else {
                    context.selected_nodes.insert(block.node_idx);
                }
            } else {
                // Normal click: toggle selection if already selected singly
                if context.selected_nodes.len() == 1
                    && context.selected_nodes.contains(&block.node_idx)
                {
                    context.selected_nodes.clear();
                } else {
                    context.selected_nodes.clear();
                    context.selected_nodes.insert(block.node_idx);
                }
            }
            *context.scroll_to_selected = true; // Raise scroll trigger

            // Auto expand parents so it shows up in tree view
            let mut curr = Some(block.node_idx);
            while let Some(idx) = curr {
                if let Some(node) = snapshot.nodes.get(idx as usize) {
                    if node.is_directory() {
                        context.expanded_nodes.insert(idx);
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
            eframe::egui::Tooltip::always_open(
                ui.ctx().clone(),
                ui.layer_id(),
                eframe::egui::Id::new("treemap_tooltip"),
                eframe::egui::PopupAnchor::Pointer,
            )
            .show(|ui| {
                ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                let cleaned_path = crate::model::arena::clean_unc_path(&path_str);
                ui.label(format!("📁 {cleaned_path}"));
                ui.label(format!("💾 Size: {size_str}"));
            });
        }
    }
}

fn draw_glow(ui: &eframe::egui::Ui, painter: &eframe::egui::Painter, rect: eframe::egui::Rect) {
    let time = ui.input(|i| i.time);
    let pulse = 0.5f64.mul_add((time * 6.0).sin(), 0.5);

    // 1. Draw Outer Expanding Glow (grows and fades)
    let glow_alpha = 0.20f64.mul_add(pulse, 0.1);
    let glow_color = crate::colors::GLOW_OUTER_BASE.gamma_multiply(glow_alpha as f32);
    let glow_thickness = 6.0f32.mul_add(pulse as f32, 4.0); // Oscillates thickness
    painter.rect(
        rect,
        0.0,
        crate::colors::COLOR_TRANSPARENT,
        eframe::egui::Stroke::new(glow_thickness, glow_color),
        eframe::egui::StrokeKind::Outside,
    );

    // 2. Draw Inner Sharp Contrast Core (stays crisp)
    let core_color = crate::colors::GLOW_INNER_CORE;
    let core_thickness = 1.0f32.mul_add(pulse as f32, 1.5);
    painter.rect(
        rect,
        0.0,
        crate::colors::COLOR_TRANSPARENT,
        eframe::egui::Stroke::new(core_thickness, core_color),
        eframe::egui::StrokeKind::Inside,
    );
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
    child_rect: Rect,
    depth: usize,
    blocks: &mut Vec<TreemapBlock>,
) {
    const MIN_PIXEL_DIM: f32 = 12.0;

    if child_rect.width() < PIXEL_PRECISION_LIMIT || child_rect.height() < PIXEL_PRECISION_LIMIT {
        return; // Discard sub-pixel visual artifacts early
    }

    let child = &config.nodes[child_idx as usize];

    let is_leaf_or_too_small = !child.is_directory()
        || depth >= config.max_depth
        || child_rect.width() < MIN_PIXEL_DIM
        || child_rect.height() < MIN_PIXEL_DIM;

    if is_leaf_or_too_small {
        let name = config.string_pool.get(child.name_id).unwrap_or("");
        let ext = crate::arena::get_ext_slice(name);
        let color = crate::colors::get_color_for_extension(ext);
        blocks.push(TreemapBlock {
            rect: child_rect,
            node_idx: child_idx,
            color,
        });
        return;
    }

    build_treemap(config, child_idx, child_rect, depth + 1, blocks);
}

/// Walks up the parent chain of a node to determine if it is a descendant of a target ancestor.
#[must_use]
pub fn is_descendant(nodes: &[FileNode], child_idx: u32, ancestor_idx: u32) -> bool {
    let mut curr = Some(child_idx);
    while let Some(idx) = curr {
        if idx == ancestor_idx {
            return true;
        }
        if let Some(node) = nodes.get(idx as usize) {
            curr = node.parent_opt();
        } else {
            break;
        }
    }
    false
}

fn build_treemap(
    config: &TreemapConfig,
    node_idx: u32,
    rect: Rect,
    depth: usize,
    blocks: &mut Vec<TreemapBlock>,
) {
    const MIN_AVG_CHILD_AREA: f64 = 16.0;

    // A stale or corrupt index (e.g. a cached layout referencing indices
    // from a snapshot that shrank after a full re-scan) must never panic
    // the render loop — bail out of this branch instead.
    if node_idx as usize >= config.nodes.len() {
        return;
    }

    let node = &config.nodes[node_idx as usize];
    if node.size == 0
        || rect.width() < PIXEL_PRECISION_LIMIT
        || rect.height() < PIXEL_PRECISION_LIMIT
    {
        return;
    }

    if !node.is_directory() || depth >= config.max_depth {
        let name = config.string_pool.get(node.name_id).unwrap_or("");
        let ext = crate::arena::get_ext_slice(name);
        let color = crate::colors::get_color_for_extension(ext);

        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    let mut children = SmallVec::<[u32; 16]>::new();
    let mut curr = node.first_child;
    while curr != NO_INDEX {
        // Stop walking a sibling chain that references an index outside
        // the arena (corrupt/stale linkage) instead of panicking.
        if curr as usize >= config.nodes.len() {
            break;
        }
        children.push(curr);
        curr = config.nodes[curr as usize].next_sibling;
    }

    if children.is_empty() {
        let color = crate::colors::TREEMAP_DIR_FALLBACK;
        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    let area = (rect.width() * rect.height()) as f64;

    #[allow(clippy::cast_precision_loss)]
    let avg_area_per_child = area / children.len() as f64;

    if avg_area_per_child < MIN_AVG_CHILD_AREA {
        let name = config.string_pool.get(node.name_id).unwrap_or("");
        let ext = crate::arena::get_ext_slice(name);
        let color = crate::colors::get_color_for_extension(ext);
        blocks.push(TreemapBlock {
            rect,
            node_idx,
            color,
        });
        return;
    }

    children.sort_by(|&a, &b| {
        config.nodes[b as usize]
            .size
            .cmp(&config.nodes[a as usize].size)
    });

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

        // zero-allocation index bounds tracking
        let mut j = i + 1;
        let mut worst_before = worst_aspect_ratio(&child_areas[i..j], w);

        while j < active_children.len() {
            // Evaluate aspect ratio with next sibling using contiguous slice indexing
            let worst_after = worst_aspect_ratio(&child_areas[i..=j], w);

            if worst_after <= worst_before {
                worst_before = worst_after;
                j += 1;
            } else {
                break;
            }
        }

        // Sum and iterate over child_areas[i..j] slice boundaries
        let row_sum: f64 = child_areas[i..j].iter().sum();
        let vertical_layout = remaining_rect.width() >= remaining_rect.height();

        if vertical_layout {
            let h = remaining_rect.height() as f64;
            let thickness = if h > 0.0 { row_sum / h } else { 0.0 };
            let mut current_y = remaining_rect.min.y;

            for (k, &area) in child_areas[i..j].iter().enumerate() {
                let child_idx = active_children[i + k];
                let item_height = if row_sum > 0.0 {
                    h * (area / row_sum)
                } else {
                    0.0
                };

                let child_rect = Rect::from_min_max(
                    pos2(remaining_rect.min.x, current_y),
                    pos2(
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

            for (k, &area) in child_areas[i..j].iter().enumerate() {
                let child_idx = active_children[i + k];
                let item_width = if row_sum > 0.0 {
                    width * (area / row_sum)
                } else {
                    0.0
                };

                let child_rect = Rect::from_min_max(
                    pos2(current_x, remaining_rect.min.y),
                    pos2(
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use eframe::egui::Rect;

    use super::*;
    use crate::{
        arena::{FileArenaSnapshot, FileNode, NodeStorage, StringPool, precompute_dir_counts},
        stats::StatsChart,
    };

    #[test]
    fn test_is_descendant() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let d_id = pool.get_or_insert(b"dir");
        let f_id = pool.get_or_insert(b"file");
        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(d_id, Some(0), true, false, 0, 0),
            FileNode::new(f_id, Some(1), false, false, 0, 0),
        ];

        assert!(is_descendant(&nodes, 2, 1));
        assert!(is_descendant(&nodes, 2, 0));
        assert!(is_descendant(&nodes, 1, 0));
        assert!(is_descendant(&nodes, 2, 2));
        assert!(!is_descendant(&nodes, 1, 2));
    }

    #[test]
    fn test_get_selection_roots() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let d_id = pool.get_or_insert(b"dir");
        let f_id = pool.get_or_insert(b"file");
        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(d_id, Some(0), true, false, 0, 0),
            FileNode::new(f_id, Some(1), false, false, 0, 0),
        ];

        let mut selected = std::collections::HashSet::new();
        selected.insert(1);
        selected.insert(2);

        let roots = get_selection_roots(&nodes, &selected);
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&1));
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn test_worst_aspect_ratio() {
        assert_eq!(worst_aspect_ratio(&[], 10.0), f64::INFINITY);
        assert_eq!(worst_aspect_ratio(&[10.0], 0.0), f64::INFINITY);

        let ratio = worst_aspect_ratio(&[10.0], 5.0);
        assert_eq!(ratio, 2.5);
    }

    #[test]
    fn test_treemap_compute_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let mut chart = TreemapChart::new();
        let blocks = chart.compute(&snapshot);
        assert!(blocks.is_empty());
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn test_treemap_compute_standard() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"root");
        let f1_id = pool.get_or_insert(b"f1.png");

        let mut nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 0, 0),
        ];
        nodes[0].first_child = 1;
        nodes[0].size = 1000;
        nodes[1].size = 1000;

        let dir_counts = precompute_dir_counts(&nodes);
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(dir_counts),
        };

        let mut chart = TreemapChart::new();
        chart.last_rect =
            Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::vec2(100.0, 100.0));
        let blocks = chart.compute(&snapshot);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].node_idx, 1);
        assert_eq!(blocks[0].rect.width(), 100.0);
        assert_eq!(blocks[0].rect.height(), 100.0);
    }

    #[test]
    fn test_build_treemap_out_of_bounds_root_no_panic() {
        let pool = StringPool::new();
        let nodes: Vec<FileNode> = vec![];
        let config = TreemapConfig {
            nodes: &nodes,
            string_pool: &pool,
            max_depth: 20,
        };
        let mut blocks = Vec::new();
        // A stale node_idx into an empty arena must bail out, not panic.
        build_treemap(&config, 5, Rect::NOTHING, 0, &mut blocks);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_build_treemap_corrupt_child_index_no_panic() {
        let mut pool = StringPool::new();
        let r = pool.get_or_insert(b"r");
        let mut nodes = vec![FileNode::new(r, None, true, false, 0, 0)];
        nodes[0].size = 1000;
        nodes[0].first_child = 999_999; // out-of-bounds sibling — must not panic
        let config = TreemapConfig {
            nodes: &nodes,
            string_pool: &pool,
            max_depth: 20,
        };
        let mut blocks = Vec::new();
        let rect = Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::vec2(100.0, 100.0));
        build_treemap(&config, 0, rect, 0, &mut blocks);
        // No valid children reachable → root rendered as a single fallback block.
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].node_idx, 0);
    }

    #[test]
    fn test_build_treemap_happy_path_unchanged() {
        let mut pool = StringPool::new();
        let r = pool.get_or_insert(b"r");
        let f = pool.get_or_insert(b"f.png");
        let mut nodes = vec![
            FileNode::new(r, None, true, false, 0, 0),
            FileNode::new(f, Some(0), false, false, 0, 0),
        ];
        nodes[0].first_child = 1;
        nodes[0].size = 1000;
        nodes[1].size = 1000;
        let config = TreemapConfig {
            nodes: &nodes,
            string_pool: &pool,
            max_depth: 20,
        };
        let mut blocks = Vec::new();
        let rect = Rect::from_min_size(eframe::egui::Pos2::ZERO, eframe::egui::vec2(100.0, 100.0));
        build_treemap(&config, 0, rect, 0, &mut blocks);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].node_idx, 1);
    }
}
