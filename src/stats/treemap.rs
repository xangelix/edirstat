use eframe::egui::{Color32, Rect, pos2};
use smallvec::SmallVec;

use super::{StatComponent, StatContext, StatsChart};
use crate::arena::{FileNode, NO_INDEX, StringPool};

pub struct TreemapBlock {
    pub rect: Rect,
    pub node_idx: u32,
    pub color: Color32,
}

pub struct TreemapChart {
    pub cached_blocks: Vec<TreemapBlock>,
    pub last_snapshot_ptr: usize,
    pub last_rect: Rect,
}

impl TreemapChart {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_blocks: Vec::new(),
            last_snapshot_ptr: 0,
            last_rect: Rect::NOTHING,
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
        let mut curr = nodes[idx as usize].parent;
        let mut ancestor_selected = false;
        while curr != crate::arena::NO_INDEX {
            if selected_nodes.contains(&curr) {
                ancestor_selected = true;
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
            let fill_color = block.color;
            let color_light = fill_color.linear_multiply(1.15);
            let color_dark = fill_color.linear_multiply(0.75);

            let base_vertex_idx = combined_mesh.vertices.len() as u32;

            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: block.rect.left_top(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_light,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: block.rect.right_top(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_light,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: block.rect.right_bottom(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_dark,
            });
            combined_mesh.vertices.push(eframe::egui::epaint::Vertex {
                pos: block.rect.left_bottom(),
                uv: eframe::egui::epaint::WHITE_UV,
                color: color_dark,
            });

            combined_mesh.add_triangle(base_vertex_idx, base_vertex_idx + 1, base_vertex_idx + 2);
            combined_mesh.add_triangle(base_vertex_idx, base_vertex_idx + 2, base_vertex_idx + 3);
        }

        painter.add(combined_mesh);

        // Dynamic overlays for highlights
        if let Some(block) = hovered_block {
            let stroke = eframe::egui::Stroke::new(1.5, crate::colors::COLOR_WHITE);
            painter.rect(
                block.rect,
                0.0,
                crate::colors::COLOR_TRANSPARENT,
                stroke,
                eframe::egui::StrokeKind::Inside,
            );
        }

        if !context.selected_nodes.is_empty() {
            let roots = get_selection_roots(&snapshot.nodes, context.selected_nodes);
            for root_idx in roots {
                // Reconstruct the bounding box union of all blocks belonging to the selection.
                // For a file, this yields its individual rect. For a directory, it yields the
                // exact unified rect of its visible children on-screen.
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
                    let time = ui.input(|i| i.time);

                    // A wave factor oscillating smoothly between 0.0 and 1.0 (approx. 1Hz frequency)
                    let pulse = 0.5f64.mul_add((time * 6.0).sin(), 0.5);

                    // 1. Draw Outer Expanding Glow (grows and fades)
                    let glow_alpha = 0.20f64.mul_add(pulse, 0.1);
                    let glow_color =
                        crate::colors::GLOW_OUTER_BASE.linear_multiply(glow_alpha as f32);
                    let glow_thickness = 6.0f32.mul_add(pulse as f32, 4.0); // Oscillates thickness
                    painter.rect(
                        rect,
                        0.0,
                        crate::colors::COLOR_TRANSPARENT,
                        eframe::egui::Stroke::new(glow_thickness, glow_color),
                        eframe::egui::StrokeKind::Outside,
                    );

                    // 2. Draw Inner Sharp Contrast Core (stays crisp)
                    let core_color = crate::colors::GLOW_INNER_CORE; // Soft pastel purple/violet
                    let core_thickness = 1.0f32.mul_add(pulse as f32, 1.5);
                    painter.rect(
                        rect,
                        0.0,
                        crate::colors::COLOR_TRANSPARENT,
                        eframe::egui::Stroke::new(core_thickness, core_color),
                        eframe::egui::StrokeKind::Inside,
                    );
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
                context.selected_nodes.clear();
                context.selected_nodes.insert(block.node_idx);
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
                ui.label(format!("📁 {path_str}"));
                ui.label(format!("💾 Size: {size_str}"));
            });
        }
    }
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

    let node = &config.nodes[node_idx as usize];
    if node.size == 0 || rect.width() < 2.0 || rect.height() < 2.0 {
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

            for (k, &area) in current_row.iter().enumerate() {
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
