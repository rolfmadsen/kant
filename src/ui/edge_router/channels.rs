use crate::features::concept_model::{DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind};
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

        let (from_side, to_side) = select_ports(
            from,
            to,
            edge.kind(),
            edge.source_port(),
            edge.target_port(),
        );

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
