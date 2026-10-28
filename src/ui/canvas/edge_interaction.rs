use super::edge_renderer::distance_to_segment;
use super::interaction::{ClickRecord, ClickTarget};
use super::{
    CanvasEdge, CanvasNode, DiagramCanvasState, DragBendState, DragSegmentState,
    EdgeBendsChangeHandler, EdgeCreateHandler, EdgeSelectHandler, CLICK_SLOP_PX,
};
use crate::features::concept_model::{BendPoint, DiagramEdge, DiagramNode, NodeId, GRID_SIZE};
use crate::ui::edge_router::geometry::prune_collinear_bends;
use crate::ui::edge_router::{EdgeRouter, RoutedEdge};
use iced::widget::canvas::Action;
use iced::Point;
use std::time::Instant;

pub fn check_bend_handle_click<Message, E: CanvasEdge>(
    selected_edge: Option<(NodeId, NodeId)>,
    edges: &[E],
    state: &mut DiagramCanvasState,
    cursor_pos: Point,
    world_pos: Point,
    on_edge_bends_changed: &Option<EdgeBendsChangeHandler<'_, Message>>,
) -> Option<Action<Message>> {
    let (sel_from, sel_to) = selected_edge?;
    let edge = edges.iter().find(|e| {
        (e.from() == sel_from && e.to() == sel_to) || (e.from() == sel_to && e.to() == sel_from)
    })?;
    let bends = edge.bends()?;

    for (idx, bend) in bends.iter().enumerate() {
        let dist = (world_pos.x - bend.x).hypot(world_pos.y - bend.y);
        if dist <= 8.0 {
            let target = ClickTarget::Bend((sel_from, sel_to), idx);
            let now = Instant::now();
            let is_double = state
                .last_click
                .map(|last| {
                    let d = (cursor_pos.x - last.position.x).hypot(cursor_pos.y - last.position.y);
                    d <= 20.0
                        && now.duration_since(last.time).as_millis() <= 500
                        && last.target == target
                })
                .unwrap_or(false);

            if is_double {
                state.last_click = None;
                state.pending_double = None;
                state.selected_bend = None;
                state.dragging_bend = None;

                let mut remaining = bends.to_vec();
                if idx < remaining.len() {
                    remaining.remove(idx);
                }
                let pruned = prune_collinear_bends(&remaining);
                let final_bends = if pruned.is_empty() {
                    None
                } else {
                    Some(pruned)
                };
                if let Some(ref on_bends) = on_edge_bends_changed {
                    return Some(
                        Action::publish(on_bends((sel_from, sel_to), final_bends)).and_capture(),
                    );
                }
                return Some(Action::request_redraw().and_capture());
            } else {
                state.last_click = Some(ClickRecord {
                    position: cursor_pos,
                    world_position: world_pos,
                    time: now,
                    target,
                });
                state.selected_bend = Some(((sel_from, sel_to), idx));
                state.dragging_bend = Some(DragBendState {
                    edge: (sel_from, sel_to),
                    bend_index: idx,
                    current_bends: bends.to_vec(),
                });
                state.press_screen_pos = Some(cursor_pos);
                return Some(Action::request_redraw().and_capture());
            }
        }
    }
    None
}

pub fn try_start_segment_drag<E: CanvasEdge>(
    routed: &RoutedEdge,
    world_pos: Point,
    cursor_pos: Point,
    _edges: &[E],
    state: &mut DiagramCanvasState,
) -> bool {
    let mut hit_seg = None;
    let mut min_dist = 10.0;
    for (seg_idx, window) in routed.points.windows(2).enumerate() {
        let d = distance_to_segment(world_pos, window[0], window[1]);
        if d <= min_dist {
            min_dist = d;
            hit_seg = Some((seg_idx, window[0], window[1]));
        }
    }

    let Some((seg_idx, p1, p2)) = hit_seg else {
        return false;
    };

    let is_horizontal = (p1.y - p2.y).abs() < 1.0;

    let (current_bends, bend_indices) = if routed.points.len() == 2 {
        let p0 = routed.points[0];
        let p1 = routed.points[1];
        if is_horizontal {
            let min_x = p0.x.min(p1.x);
            let max_x = p0.x.max(p1.x);
            let span = max_x - min_x;
            let b1_x = ((min_x + span * 0.33) / GRID_SIZE).round() * GRID_SIZE;
            let b2_x = ((min_x + span * 0.67) / GRID_SIZE).round() * GRID_SIZE;
            let snap_y = (world_pos.y / GRID_SIZE).round() * GRID_SIZE;
            let b1 = BendPoint::new(b1_x, snap_y);
            let b2 = BendPoint::new(b2_x, snap_y);
            (vec![b1, b2], vec![0, 1])
        } else {
            let min_y = p0.y.min(p1.y);
            let max_y = p0.y.max(p1.y);
            let span = max_y - min_y;
            let b1_y = ((min_y + span * 0.33) / GRID_SIZE).round() * GRID_SIZE;
            let b2_y = ((min_y + span * 0.67) / GRID_SIZE).round() * GRID_SIZE;
            let snap_x = (world_pos.x / GRID_SIZE).round() * GRID_SIZE;
            let b1 = BendPoint::new(snap_x, b1_y);
            let b2 = BendPoint::new(snap_x, b2_y);
            (vec![b1, b2], vec![0, 1])
        }
    } else {
        let corners: Vec<BendPoint> = routed.points[1..routed.points.len() - 1]
            .iter()
            .map(|p| BendPoint::new(p.x, p.y))
            .collect();
        let b_indices = if seg_idx == 0 {
            vec![0]
        } else if seg_idx >= corners.len() {
            vec![corners.len() - 1]
        } else {
            vec![seg_idx - 1, seg_idx]
        };
        (corners, b_indices)
    };

    state.dragging_segment = Some(DragSegmentState {
        edge: (routed.from, routed.to),
        segment_index: seg_idx,
        is_horizontal,
        bend_indices,
        current_bends,
    });
    state.press_screen_pos = Some(cursor_pos);
    true
}

pub fn handle_delete_bend_key<Message, E: CanvasEdge>(
    state: &mut DiagramCanvasState,
    edges: &[E],
    on_edge_bends_changed: &Option<EdgeBendsChangeHandler<'_, Message>>,
) -> Option<Action<Message>> {
    let ((from, to), idx) = state.selected_bend.take()?;
    let edge = edges
        .iter()
        .find(|e| (e.from() == from && e.to() == to) || (e.from() == to && e.to() == from))?;
    let bends = edge.bends()?;
    let mut remaining = bends.to_vec();
    if idx < remaining.len() {
        remaining.remove(idx);
    }
    let pruned = prune_collinear_bends(&remaining);
    let final_bends = if pruned.is_empty() {
        None
    } else {
        Some(pruned)
    };
    if let Some(ref cb) = on_edge_bends_changed {
        Some(Action::publish(cb((from, to), final_bends)).and_capture())
    } else {
        Some(Action::request_redraw().and_capture())
    }
}

pub fn handle_bend_or_segment_cursor_drag<Message>(
    state: &mut DiagramCanvasState,
    world_pos: Point,
) -> Option<Action<Message>> {
    if let Some(ref mut d) = state.dragging_bend {
        let snapped = super::viewport::CanvasViewport::snap_to_grid(world_pos, GRID_SIZE);
        if d.bend_index < d.current_bends.len() {
            d.current_bends[d.bend_index] = BendPoint::new(snapped.x, snapped.y);
        }
        return Some(Action::request_redraw().and_capture());
    }

    if let Some(ref mut d) = state.dragging_segment {
        let snapped = super::viewport::CanvasViewport::snap_to_grid(world_pos, GRID_SIZE);
        for &b_idx in &d.bend_indices {
            if b_idx < d.current_bends.len() {
                if d.is_horizontal {
                    d.current_bends[b_idx].y = snapped.y;
                } else {
                    d.current_bends[b_idx].x = snapped.x;
                }
            }
        }
        return Some(Action::request_redraw().and_capture());
    }

    None
}

pub fn handle_bend_or_segment_release<Message, N: CanvasNode, E: CanvasEdge>(
    state: &mut DiagramCanvasState,
    cursor_pos: Point,
    nodes: &[N],
    _edges: &[E],
    on_edge_bends_changed: &Option<EdgeBendsChangeHandler<'_, Message>>,
) -> Option<Action<Message>> {
    let move_dist = state
        .press_screen_pos
        .map(|p| (cursor_pos.x - p.x).hypot(cursor_pos.y - p.y))
        .unwrap_or(0.0);

    if move_dist < CLICK_SLOP_PX {
        state.dragging_bend = None;
        state.dragging_segment = None;
        return None;
    }

    let prune_bends_for_edge = |edge_id: (NodeId, NodeId),
                                bends: &[BendPoint]|
     -> Option<Vec<BendPoint>> {
        let from_node = nodes
            .iter()
            .find(|n| n.id() == edge_id.0)
            .map(|n| n.to_diagram_node());
        let to_node = nodes
            .iter()
            .find(|n| n.id() == edge_id.1)
            .map(|n| n.to_diagram_node());

        let pruned = if let (Some(ref f), Some(ref t)) = (from_node, to_node) {
            let from_side = crate::ui::edge_router::channels::select_port_for_terminal_segment(
                f, bends, true, t,
            );
            let to_side = crate::ui::edge_router::channels::select_port_for_terminal_segment(
                t, bends, false, f,
            );
            let p1 = Point::new(bends[0].x, bends[0].y);
            let pk = Point::new(bends.last().unwrap().x, bends.last().unwrap().y);
            let start =
                crate::ui::edge_router::channels::compute_sliding_port_point(f, from_side, p1);
            let end = crate::ui::edge_router::channels::compute_sliding_port_point(t, to_side, pk);
            crate::ui::edge_router::geometry::prune_collinear_bends_with_endpoints(
                bends, start, end,
            )
        } else {
            prune_collinear_bends(bends)
        };

        if pruned.is_empty() {
            None
        } else {
            Some(pruned)
        }
    };

    if let Some(d) = state.dragging_bend.take() {
        let final_bends = prune_bends_for_edge(d.edge, &d.current_bends);
        if let Some(ref cb) = on_edge_bends_changed {
            return Some(Action::publish(cb(d.edge, final_bends)).and_capture());
        }
        return Some(Action::request_redraw().and_capture());
    }

    if let Some(d) = state.dragging_segment.take() {
        let final_bends = prune_bends_for_edge(d.edge, &d.current_bends);
        if let Some(ref cb) = on_edge_bends_changed {
            return Some(Action::publish(cb(d.edge, final_bends)).and_capture());
        }
        return Some(Action::request_redraw().and_capture());
    }

    None
}

pub fn handle_bend_double_click<Message, E: CanvasEdge>(
    from: NodeId,
    to: NodeId,
    idx: usize,
    edges: &[E],
    state: &mut DiagramCanvasState,
    on_edge_bends_changed: &Option<EdgeBendsChangeHandler<'_, Message>>,
) -> Option<Action<Message>> {
    if let Some(edge) = edges
        .iter()
        .find(|e| (e.from() == from && e.to() == to) || (e.from() == to && e.to() == from))
    {
        if let Some(bends) = edge.bends() {
            let mut remaining = bends.to_vec();
            if idx < remaining.len() {
                remaining.remove(idx);
            }
            let pruned = prune_collinear_bends(&remaining);
            let final_bends = if pruned.is_empty() {
                None
            } else {
                Some(pruned)
            };
            state.selected_bend = None;
            if let Some(ref cb) = on_edge_bends_changed {
                return Some(Action::publish(cb((from, to), final_bends)).and_capture());
            }
        }
    }
    Some(Action::request_redraw().and_capture())
}

pub fn check_edge_click<Message, N: CanvasNode, E: CanvasEdge>(
    nodes: &[N],
    edges: &[E],
    selected_edge: Option<(NodeId, NodeId)>,
    state: &mut DiagramCanvasState,
    cursor_pos: Point,
    world_pos: Point,
    on_edge_selected: &Option<EdgeSelectHandler<'_, Message>>,
) -> Option<Action<Message>> {
    let diagram_nodes: Vec<DiagramNode> = nodes.iter().map(|n| n.to_diagram_node()).collect();
    let diagram_edges: Vec<DiagramEdge> = edges.iter().map(|e| e.to_diagram_edge()).collect();
    let routed_edges = EdgeRouter::route_edges(&diagram_nodes, &diagram_edges);

    for routed in routed_edges.iter().rev() {
        let hit_segment = routed
            .points
            .windows(2)
            .any(|w| distance_to_segment(world_pos, w[0], w[1]) <= 10.0);
        let hit_label = if let (Some(label), Some(pos)) = (&routed.label, routed.label_pos) {
            let approx_w = label.trim().len() as f32 * 6.8 + 16.0;
            world_pos.x >= pos.x - approx_w / 2.0
                && world_pos.x <= pos.x + approx_w / 2.0
                && world_pos.y >= pos.y - 10.0
                && world_pos.y <= pos.y + 10.0
        } else {
            false
        };

        if hit_segment || hit_label {
            let is_already_selected_edge = selected_edge == Some((routed.from, routed.to))
                || selected_edge == Some((routed.to, routed.from));

            let started_drag =
                hit_segment && try_start_segment_drag(routed, world_pos, cursor_pos, edges, state);

            if !is_already_selected_edge {
                if let Some(ref cb) = on_edge_selected {
                    return Some(Action::publish(cb(Some((routed.from, routed.to)))).and_capture());
                }
            }

            if started_drag {
                return Some(Action::request_redraw().and_capture());
            }
        }
    }
    None
}

pub fn check_connect_handle_click<Message, N: CanvasNode>(
    selected_node_id: Option<NodeId>,
    nodes: &[N],
    state: &mut DiagramCanvasState,
    world_pos: Point,
) -> Option<Action<Message>> {
    let sel_id = selected_node_id?;
    let node = nodes.iter().find(|n| n.id() == sel_id)?;
    let (nx, ny) = node.position();
    let (nw, nh) = node.size();
    let handle_center = Point::new(nx + nw, ny + nh / 2.0);
    let dist = (world_pos.x - handle_center.x).hypot(world_pos.y - handle_center.y);
    if dist <= 14.0 {
        state.connecting_from = Some(sel_id);
        state.connecting_cursor = Some(world_pos);
        state.hovered_target_node = None;
        Some(Action::request_redraw().and_capture())
    } else {
        None
    }
}

pub fn handle_connecting_cursor_move<Message, N: CanvasNode>(
    state: &mut DiagramCanvasState,
    nodes: &[N],
    world_pos: Point,
) -> Option<Action<Message>> {
    let source_id = state.connecting_from?;
    state.connecting_cursor = Some(world_pos);
    state.hovered_target_node = nodes
        .iter()
        .rev()
        .find(|n| n.id() != source_id && n.contains(world_pos.x, world_pos.y))
        .map(|n| n.id());
    Some(Action::request_redraw().and_capture())
}

pub fn handle_connecting_release<Message>(
    state: &mut DiagramCanvasState,
    on_edge_created: &Option<EdgeCreateHandler<'_, Message>>,
) -> Option<Action<Message>> {
    let source_id = state.connecting_from.take()?;
    state.connecting_cursor = None;
    let target = state.hovered_target_node.take();
    if let Some(target_id) = target {
        if target_id != source_id {
            if let Some(ref cb) = on_edge_created {
                return Some(Action::publish(cb(source_id, target_id)).and_capture());
            }
        }
    }
    Some(Action::request_redraw().and_capture())
}
