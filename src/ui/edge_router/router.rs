use crate::features::concept_model::{DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind};
use crate::ui::edge_router::channels::{compute_port_assignments, port_point, select_ports};
use crate::ui::edge_router::geometry::{
    build_orthogonal_path, compute_arrow_head, compute_diamond, compute_half_arrow,
    compute_label_pos, detect_bridges,
};
use crate::ui::edge_router::types::{
    EdgePortAssignment, RoutedEdge, ARROW_HEAD_LENGTH, DIAMOND_LENGTH,
};
use iced::Point;
use std::collections::HashMap;

pub struct EdgeRouter;

impl EdgeRouter {
    pub fn assign_ports(nodes: &[DiagramNode], edges: &[DiagramEdge]) -> Vec<EdgePortAssignment> {
        let node_map: HashMap<NodeId, &DiagramNode> = nodes.iter().map(|n| (n.id(), n)).collect();
        compute_port_assignments(&node_map, edges)
    }

    pub fn select_ports(
        from: &DiagramNode,
        to: &DiagramNode,
        kind: RelationKind,
        current_source_port: Option<PortSide>,
        current_target_port: Option<PortSide>,
    ) -> (PortSide, PortSide) {
        select_ports(from, to, kind, current_source_port, current_target_port)
    }

    pub fn route_edges(nodes: &[DiagramNode], edges: &[DiagramEdge]) -> Vec<RoutedEdge> {
        let start = std::time::Instant::now();
        let node_map: HashMap<NodeId, &DiagramNode> = nodes.iter().map(|n| (n.id(), n)).collect();

        // 1. Vælg porte og tildel slots til hver edge
        let assignments = compute_port_assignments(&node_map, edges);

        // 2. Generer 90-graders ortogonale stier for hver edge
        let mut routes: Vec<RoutedEdge> = assignments
            .iter()
            .filter_map(|assign| {
                let from_node = node_map.get(&assign.from_id)?;
                let to_node = node_map.get(&assign.to_id)?;
                let edge = &edges[assign.edge_idx];

                Some(Self::generate_route(from_node, to_node, edge, assign))
            })
            .collect();

        // 3. Detekter linjekrydsninger og tilføj broer (bridge hops)
        detect_bridges(&mut routes);

        let elapsed = start.elapsed();
        if elapsed > std::time::Duration::from_millis(16) {
            crate::features::diagnostics::log_warn(
                "canvas_perf",
                &format!(
                    "Edge routing for {} noder og {} kanter tog {:?} (>16ms frame budget)",
                    nodes.len(),
                    edges.len(),
                    elapsed
                ),
            );
        }

        routes
    }

    fn generate_route(
        from_node: &DiagramNode,
        to_node: &DiagramNode,
        edge: &DiagramEdge,
        assign: &EdgePortAssignment,
    ) -> RoutedEdge {
        let start_pt = port_point(from_node, assign.from_side, assign.from_slot_offset);
        let end_pt = port_point(to_node, assign.to_side, assign.to_slot_offset);

        // Beregn pilehoved (Generalisering trekant eller Dependency åben pil)
        let arrow_head = if assign.kind == RelationKind::Generalization
            || assign.kind == RelationKind::Dependency
        {
            Some(compute_arrow_head(end_pt, assign.to_side))
        } else {
            None
        };

        // Rute-endepunkt: hvis der er pilehoved, stopper linjen ved pilens base
        let line_end_pt = if arrow_head.is_some() {
            match assign.to_side {
                PortSide::Bottom => Point::new(end_pt.x, end_pt.y + ARROW_HEAD_LENGTH),
                PortSide::Top => Point::new(end_pt.x, end_pt.y - ARROW_HEAD_LENGTH),
                PortSide::Left => Point::new(end_pt.x - ARROW_HEAD_LENGTH, end_pt.y),
                PortSide::Right => Point::new(end_pt.x + ARROW_HEAD_LENGTH, end_pt.y),
            }
        } else {
            end_pt
        };

        // Beregn kildemarkør (f.eks. sort diamant for komposition)
        let source_diamond = if assign.kind == RelationKind::Composition {
            Some(compute_diamond(start_pt, assign.from_side))
        } else {
            None
        };

        // Rute-startpunkt: hvis der er kildediamant, starter linjen ved diamantens bagkant
        let line_start_pt = if source_diamond.is_some() {
            match assign.from_side {
                PortSide::Bottom => Point::new(start_pt.x, start_pt.y + DIAMOND_LENGTH),
                PortSide::Top => Point::new(start_pt.x, start_pt.y - DIAMOND_LENGTH),
                PortSide::Left => Point::new(start_pt.x - DIAMOND_LENGTH, start_pt.y),
                PortSide::Right => Point::new(start_pt.x + DIAMOND_LENGTH, start_pt.y),
            }
        } else {
            start_pt
        };

        // Generer 90-graders ortogonale punkter
        let points = build_orthogonal_path(
            line_start_pt,
            line_end_pt,
            assign.from_side,
            assign.to_side,
            assign.channel_index,
        );

        // FDA label-semantik: Generalisering har ingen tekst-label
        let label = if assign.kind == RelationKind::Generalization {
            None
        } else {
            edge.label().map(|s| s.to_string())
        };

        // Beregn label position midt på det længste segment
        let label_pos = label.as_ref().and_then(|_| compute_label_pos(&points));

        // Beregn halv pil for rettet association
        let half_arrow = if assign.kind == RelationKind::Association && edge.is_directed() {
            Some(compute_half_arrow(end_pt, assign.to_side))
        } else {
            None
        };

        RoutedEdge {
            from: from_node.id(),
            to: to_node.id(),
            kind: assign.kind,
            label,
            label_pos,
            points,
            arrow_head,
            half_arrow,
            source_diamond,
            bridges: Vec::new(),
            from_side: assign.from_side,
            to_side: assign.to_side,
        }
    }
}
