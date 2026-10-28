use crate::features::concept_model::{
    BendPoint, DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind,
};
use crate::ui::edge_router::types::{
    EdgePortAssignment, SideAttachmentMap, MIN_ARROW_CLEARANCE, SLOT_SPACING,
};
use iced::Point;
use std::collections::HashMap;

pub fn compute_port_assignments(
    node_map: &HashMap<NodeId, &DiagramNode>,
    edges: &[DiagramEdge],
) -> Vec<EdgePortAssignment> {
    let mut initial_assignments = Vec::new();
    let mut side_attachments: SideAttachmentMap = HashMap::new();

    for (idx, edge) in edges.iter().enumerate() {
        let (Some(from), Some(to)) = (node_map.get(&edge.from()), node_map.get(&edge.to())) else {
            continue;
        };

        let (from_side, to_side) = if let Some(bends) = edge.bends() {
            if !bends.is_empty() {
                (
                    select_port_for_terminal_segment(from, bends, true, to),
                    select_port_for_terminal_segment(to, bends, false, from),
                )
            } else {
                select_ports(
                    from,
                    to,
                    edge.kind(),
                    edge.source_port(),
                    edge.target_port(),
                )
            }
        } else {
            select_ports(
                from,
                to,
                edge.kind(),
                edge.source_port(),
                edge.target_port(),
            )
        };

        side_attachments
            .entry((from.id(), from_side))
            .or_default()
            .push((idx, edge.kind(), true));

        side_attachments
            .entry((to.id(), to_side))
            .or_default()
            .push((idx, edge.kind(), false));

        initial_assignments.push((idx, edge.from(), edge.to(), edge.kind(), from_side, to_side));
    }

    let mut slot_offsets: HashMap<(usize, bool), f32> = HashMap::new();

    for ((_node_id, side), attachments) in side_attachments {
        let mut groups: HashMap<(RelationKind, bool), Vec<usize>> = HashMap::new();
        for &(edge_idx, kind, is_source) in &attachments {
            groups.entry((kind, is_source)).or_default().push(edge_idx);
        }

        if groups.len() <= 1 {
            for &(edge_idx, _, is_source) in &attachments {
                slot_offsets.insert((edge_idx, is_source), 0.0);
            }
        } else {
            let mut sorted_groups: Vec<((RelationKind, bool), Vec<usize>, f32)> = groups
                .into_iter()
                .map(|(key, edge_indices)| {
                    let avg_coord = if edge_indices.is_empty() {
                        0.0
                    } else {
                        let sum: f32 = edge_indices
                            .iter()
                            .map(|&e_idx| {
                                let other_id = if key.1 {
                                    edges[e_idx].to()
                                } else {
                                    edges[e_idx].from()
                                };
                                node_map
                                    .get(&other_id)
                                    .map(|n| {
                                        if side.is_vertical() {
                                            n.center().0
                                        } else {
                                            n.center().1
                                        }
                                    })
                                    .unwrap_or(0.0)
                            })
                            .sum();
                        sum / edge_indices.len() as f32
                    };
                    (key, edge_indices, avg_coord)
                })
                .collect();

            sorted_groups
                .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            let count = sorted_groups.len();
            let mid = (count as f32 - 1.0) / 2.0;
            for (i, ((_kind, is_source), edge_indices, _)) in sorted_groups.into_iter().enumerate()
            {
                let offset = (i as f32 - mid) * SLOT_SPACING;
                for edge_idx in edge_indices {
                    slot_offsets.insert((edge_idx, is_source), offset);
                }
            }
        }
    }

    initial_assignments
        .into_iter()
        .enumerate()
        .map(
            |(channel_idx, (edge_idx, from_id, to_id, kind, from_side, to_side))| {
                let from_slot_offset = slot_offsets.get(&(edge_idx, true)).copied().unwrap_or(0.0);
                let to_slot_offset = slot_offsets.get(&(edge_idx, false)).copied().unwrap_or(0.0);

                EdgePortAssignment {
                    edge_idx,
                    from_id,
                    to_id,
                    kind,
                    from_side,
                    to_side,
                    from_slot_offset,
                    to_slot_offset,
                    channel_index: channel_idx,
                }
            },
        )
        .collect()
}

pub fn select_ports(
    from: &DiagramNode,
    to: &DiagramNode,
    kind: RelationKind,
    current_source_port: Option<PortSide>,
    current_target_port: Option<PortSide>,
) -> (PortSide, PortSide) {
    let from_side = if let Some(saved_from) = current_source_port {
        evaluate_source_hysteresis(from, to, saved_from)
    } else {
        select_initial_source_port(from, to, kind)
    };

    let to_side = if let Some(saved_to) = current_target_port {
        evaluate_target_hysteresis(to, from, saved_to, from_side)
    } else {
        select_matching_target_port(from, to, kind, from_side)
    };

    (from_side, to_side)
}

fn evaluate_source_hysteresis(
    from: &DiagramNode,
    to: &DiagramNode,
    current_port: PortSide,
) -> PortSide {
    let from_right = from.x() + from.width();
    let from_bottom = from.y() + from.height();
    let from_left = from.x();
    let from_top = from.y();

    match current_port {
        PortSide::Right => {
            if to.x() >= from_right - 5.0 {
                PortSide::Right
            } else if to.center().1 > from.center().1 {
                PortSide::Bottom
            } else if to.center().1 < from.center().1 {
                PortSide::Top
            } else if to.x() + to.width() <= from_left {
                PortSide::Left
            } else {
                PortSide::Right
            }
        }
        PortSide::Bottom => {
            if to.y() >= from_bottom - 5.0 {
                PortSide::Bottom
            } else if to.center().0 > from.center().0 {
                PortSide::Right
            } else if to.center().0 < from.center().0 {
                PortSide::Left
            } else if to.y() + to.height() <= from_top {
                PortSide::Top
            } else {
                PortSide::Bottom
            }
        }
        PortSide::Left => {
            if to.x() + to.width() <= from_left + 5.0 {
                PortSide::Left
            } else if to.center().1 > from.center().1 {
                PortSide::Bottom
            } else if to.center().1 < from.center().1 {
                PortSide::Top
            } else if to.x() >= from_right {
                PortSide::Right
            } else {
                PortSide::Left
            }
        }
        PortSide::Top => {
            if to.y() + to.height() <= from_top + 5.0 {
                PortSide::Top
            } else if to.center().0 > from.center().0 {
                PortSide::Right
            } else if to.center().0 < from.center().0 {
                PortSide::Left
            } else if to.y() >= from_bottom {
                PortSide::Bottom
            } else {
                PortSide::Top
            }
        }
    }
}

fn evaluate_target_hysteresis(
    to: &DiagramNode,
    from: &DiagramNode,
    current_to_port: PortSide,
    from_side: PortSide,
) -> PortSide {
    let to_right = to.x() + to.width();
    let to_bottom = to.y() + to.height();
    let to_left = to.x();
    let to_top = to.y();

    let valid = match current_to_port {
        PortSide::Right => from.x() >= to_right - 5.0,
        PortSide::Bottom => from.y() >= to_bottom - 5.0,
        PortSide::Left => from.x() + from.width() <= to_left + 5.0,
        PortSide::Top => from.y() + from.height() <= to_top + 5.0,
    };

    if valid {
        current_to_port
    } else {
        select_matching_target_port(from, to, RelationKind::Association, from_side)
    }
}

fn select_initial_source_port(
    from: &DiagramNode,
    to: &DiagramNode,
    kind: RelationKind,
) -> PortSide {
    let from_right = from.x() + from.width();
    let from_bottom = from.y() + from.height();
    let from_left = from.x();
    let from_top = from.y();

    let to_right = to.x() + to.width();
    let to_bottom = to.y() + to.height();
    let to_left = to.x();
    let to_top = to.y();

    if kind == RelationKind::Generalization {
        let super_bottom = to.y() + to.height();
        let sub_top = from.y();
        let vert_clearance = sub_top - super_bottom;

        if vert_clearance >= MIN_ARROW_CLEARANCE {
            PortSide::Top
        } else if from.y() + from.height() <= to.y() - MIN_ARROW_CLEARANCE {
            PortSide::Bottom
        } else if from.x() >= to_right {
            PortSide::Left
        } else {
            // from_right <= to_left eller kritisk nærhed (< 36px) / overlap: brug side-port så pilen ikke mastes
            PortSide::Right
        }
    } else {
        let horiz_clearance = if to_left > from_right {
            to_left - from_right
        } else if from_left > to_right {
            from_left - to_right
        } else {
            0.0
        };

        let vert_clearance = if to_top > from_bottom {
            to_top - from_bottom
        } else if from_top > to_bottom {
            from_top - to_bottom
        } else {
            0.0
        };

        let dx = to.center().0 - from.center().0;
        let dy = to.center().1 - from.center().1;

        if to_left >= from_right && horiz_clearance >= vert_clearance {
            PortSide::Right
        } else if to_right <= from_left && horiz_clearance >= vert_clearance {
            PortSide::Left
        } else if to_top >= from_bottom {
            PortSide::Bottom
        } else if to_bottom <= from_top {
            PortSide::Top
        } else if dx.abs() > dy.abs() {
            if dx > 0.0 {
                PortSide::Right
            } else {
                PortSide::Left
            }
        } else if dy > 0.0 {
            PortSide::Bottom
        } else {
            PortSide::Top
        }
    }
}

fn select_matching_target_port(
    from: &DiagramNode,
    to: &DiagramNode,
    kind: RelationKind,
    from_side: PortSide,
) -> PortSide {
    if kind == RelationKind::Generalization {
        let super_bottom = to.y() + to.height();
        let sub_top = from.y();
        let vert_clearance = sub_top - super_bottom;

        if from_side == PortSide::Right
            && from.x() < to.x() + to.width()
            && vert_clearance < MIN_ARROW_CLEARANCE
        {
            PortSide::Right
        } else {
            match from_side {
                PortSide::Left => PortSide::Right,
                PortSide::Right => PortSide::Left,
                PortSide::Top => PortSide::Bottom,
                PortSide::Bottom => PortSide::Top,
            }
        }
    } else {
        from_side.opposite()
    }
}

pub fn port_point(node: &DiagramNode, side: PortSide, slot_offset: f32) -> Point {
    let (cx, cy) = node.center();
    match side {
        PortSide::Top => Point::new(cx + slot_offset, node.y()),
        PortSide::Bottom => Point::new(cx + slot_offset, node.y() + node.height()),
        PortSide::Left => Point::new(node.x(), cy + slot_offset),
        PortSide::Right => Point::new(node.x() + node.width(), cy + slot_offset),
    }
}

pub fn select_port_towards_point(node: &DiagramNode, pt: Point) -> PortSide {
    let node_right = node.x() + node.width();
    let node_bottom = node.y() + node.height();
    let node_left = node.x();
    let node_top = node.y();

    let (cx, cy) = node.center();
    let dx = pt.x - cx;
    let dy = pt.y - cy;

    if pt.x >= node_right && (pt.y >= node_top && pt.y <= node_bottom) {
        PortSide::Right
    } else if pt.x <= node_left && (pt.y >= node_top && pt.y <= node_bottom) {
        PortSide::Left
    } else if pt.y >= node_bottom && (pt.x >= node_left && pt.x <= node_right) {
        PortSide::Bottom
    } else if pt.y <= node_top && (pt.x >= node_left && pt.x <= node_right) {
        PortSide::Top
    } else if dx.abs() > dy.abs() {
        if dx > 0.0 {
            PortSide::Right
        } else {
            PortSide::Left
        }
    } else if dy > 0.0 {
        PortSide::Bottom
    } else {
        PortSide::Top
    }
}

pub fn compute_sliding_port_point(node: &DiagramNode, side: PortSide, target_pt: Point) -> Point {
    const MARGIN: f32 = 8.0;
    let min_x = node.x() + MARGIN;
    let max_x = (node.x() + node.width() - MARGIN).max(min_x);
    let min_y = node.y() + MARGIN;
    let max_y = (node.y() + node.height() - MARGIN).max(min_y);

    match side {
        PortSide::Top => Point::new(target_pt.x.clamp(min_x, max_x), node.y()),
        PortSide::Bottom => Point::new(target_pt.x.clamp(min_x, max_x), node.y() + node.height()),
        PortSide::Left => Point::new(node.x(), target_pt.y.clamp(min_y, max_y)),
        PortSide::Right => Point::new(node.x() + node.width(), target_pt.y.clamp(min_y, max_y)),
    }
}

pub fn select_port_for_terminal_segment(
    node: &DiagramNode,
    bends: &[BendPoint],
    is_source: bool,
    other_node: &DiagramNode,
) -> PortSide {
    if bends.is_empty() {
        return PortSide::Bottom;
    }

    let node_left = node.x();
    let node_right = node.x() + node.width();
    let node_top = node.y();
    let node_bottom = node.y() + node.height();

    if is_source {
        let p_start = Point::new(bends[0].x, bends[0].y);
        let p_next = if bends.len() >= 2 {
            Point::new(bends[1].x, bends[1].y)
        } else {
            Point::new(other_node.center().0, other_node.center().1)
        };

        if (p_start.y - p_next.y).abs() < 1.0 {
            let y = p_start.y;
            if p_start.x < node_left {
                PortSide::Left
            } else if p_start.x > node_right {
                PortSide::Right
            } else if y < node_top {
                PortSide::Top
            } else if y <= node_bottom {
                if p_next.x < node.center().0 {
                    PortSide::Left
                } else {
                    PortSide::Right
                }
            } else {
                PortSide::Bottom
            }
        } else if (p_start.x - p_next.x).abs() < 1.0 {
            let x = p_start.x;
            if p_start.y < node_top {
                PortSide::Top
            } else if p_start.y > node_bottom {
                PortSide::Bottom
            } else if x < node_left {
                PortSide::Left
            } else if x <= node_right {
                if p_next.y < node.center().1 {
                    PortSide::Top
                } else {
                    PortSide::Bottom
                }
            } else {
                PortSide::Right
            }
        } else {
            select_port_towards_point(node, p_start)
        }
    } else {
        let k = bends.len() - 1;
        let pk = Point::new(bends[k].x, bends[k].y);
        let prev = if k >= 1 {
            Point::new(bends[k - 1].x, bends[k - 1].y)
        } else {
            Point::new(other_node.center().0, other_node.center().1)
        };

        if (prev.y - pk.y).abs() < 1.0 {
            let y = pk.y;
            if pk.x < node_left {
                PortSide::Left
            } else if pk.x > node_right {
                PortSide::Right
            } else if y < node_top {
                PortSide::Top
            } else if y <= node_bottom {
                if prev.x < node.center().0 {
                    PortSide::Left
                } else {
                    PortSide::Right
                }
            } else {
                PortSide::Bottom
            }
        } else if (prev.x - pk.x).abs() < 1.0 {
            let x = pk.x;
            if pk.y < node_top {
                PortSide::Top
            } else if pk.y > node_bottom {
                PortSide::Bottom
            } else if x < node_left {
                PortSide::Left
            } else if x <= node_right {
                if prev.y < node.center().1 {
                    PortSide::Top
                } else {
                    PortSide::Bottom
                }
            } else {
                PortSide::Right
            }
        } else {
            select_port_towards_point(node, pk)
        }
    }
}
