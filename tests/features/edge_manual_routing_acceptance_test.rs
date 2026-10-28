use iced::Point;
use kant::features::concept_model::{BendPoint, DiagramEdge, DiagramNode, RelationKind, GRID_SIZE};
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::ClassDiagramEdge;
use kant::features::model::decomposed::{load_decomposed, save_decomposed};
use kant::features::model::ModelProject;
use kant::ui::canvas::viewport::CanvasViewport;
use kant::ui::canvas::CanvasEdge;
use kant::ui::edge_router::geometry::prune_collinear_bends;
use kant::ui::edge_router::EdgeRouter;
use uuid::Uuid;

#[test]
fn test_ac1_bendpoint_and_edge_bends_accessors() {
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();

    let mut edge = DiagramEdge::new(from_id, to_id, RelationKind::Association);
    assert!(edge.bends().is_none());

    let bends = vec![BendPoint::new(100.0, 200.0), BendPoint::new(100.0, 400.0)];
    edge.set_bends(Some(bends.clone()));
    assert_eq!(edge.bends(), Some(bends.as_slice()));

    // Test for ClassDiagramEdge
    let mut class_edge = ClassDiagramEdge::new(from_id, to_id, RelationKind::Association, None);
    assert!(class_edge.bends().is_none());
    class_edge.set_bends(Some(bends.clone()));
    assert_eq!(class_edge.bends(), Some(bends.as_slice()));

    // Test CanvasEdge trait abstraction
    let canvas_edge: &dyn CanvasEdge = &edge;
    assert_eq!(canvas_edge.bends(), Some(bends.as_slice()));
}

#[test]
fn test_ac2_decomposed_persistence_of_bends() {
    let temp_dir = std::env::temp_dir().join(format!("kant_test_bends_{}", Uuid::new_v4()));
    let kant_dir = temp_dir.join(".kant");

    let mut project = ModelProject::default();
    let c1 = Concept::new("Ordre", "En forretningsordre", BelongsToDomain::Yes);
    let c2 = Concept::new("Kunde", "En køber", BelongsToDomain::Yes);
    let n1 = project.concept_graph_mut().add_node(&c1);
    let n2 = project.concept_graph_mut().add_node(&c2);

    let mut edge = DiagramEdge::new(n1, n2, RelationKind::Association);
    let bends = vec![BendPoint::new(120.0, 240.0), BendPoint::new(260.0, 240.0)];
    edge.set_bends(Some(bends.clone()));
    project.concept_graph_mut().edges_mut().push(edge);

    // Save decomposed
    save_decomposed(&project, &kant_dir).expect("save_decomposed should succeed");

    // Load back
    let loaded = load_decomposed(&kant_dir).expect("load_decomposed should succeed");
    let loaded_edge = loaded
        .concept_graph()
        .edges()
        .first()
        .expect("edge should exist");
    assert_eq!(loaded_edge.bends(), Some(bends.as_slice()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ac3_ac4_orthogonal_routing_with_bends_and_port_sliding() {
    // Node A at (100, 100), size 160x80 -> bottom edge is y = 180, x in [100, 260]
    let node_a = DiagramNode::custom(Uuid::new_v4(), "A".to_string(), 100.0, 100.0, 160.0, 80.0);
    // Node B at (100, 400), size 160x80 -> top edge is y = 400, x in [100, 260]
    let node_b = DiagramNode::custom(Uuid::new_v4(), "B".to_string(), 100.0, 400.0, 160.0, 80.0);

    // Bend at (220, 280), shifted toward right flank of node A (220 is within [108, 252])
    let mut edge = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);
    edge.set_bends(Some(vec![BendPoint::new(220.0, 280.0)]));

    let routes = EdgeRouter::route_edges(&[node_a.clone(), node_b.clone()], &[edge]);
    assert_eq!(routes.len(), 1);
    let route = &routes[0];

    // AC4: Port-sliding - start point should have x = 220.0 (sliding to align with P_1)
    let start_pt = route.points[0];
    assert_eq!(
        start_pt.y, 180.0,
        "Source port should be on bottom of Node A"
    );
    assert_eq!(
        start_pt.x, 220.0,
        "Source port should slide along bottom edge to align with P_1.x"
    );

    // AC3: Route should contain the bend (220.0, 280.0)
    assert!(
        route
            .points
            .iter()
            .any(|p| (p.x - 220.0).abs() < 0.1 && (p.y - 280.0).abs() < 0.1),
        "Route must pass through bend (220.0, 280.0)"
    );
}

#[test]
fn test_ac6_ac10_grid_snapping_and_collinear_pruning() {
    // Grid snapping helper
    let p = Point::new(43.2, 78.9);
    let snapped = CanvasViewport::snap_to_grid(p, GRID_SIZE);
    assert_eq!(snapped, Point::new(40.0, 80.0));

    // AC10: Collinear pruning: three points on the same vertical axis
    let pts = vec![
        BendPoint::new(100.0, 100.0),
        BendPoint::new(100.0, 200.0),
        BendPoint::new(100.0, 300.0),
        BendPoint::new(200.0, 300.0),
    ];
    let pruned = prune_collinear_bends(&pts);
    assert_eq!(
        pruned,
        vec![
            BendPoint::new(100.0, 100.0),
            BendPoint::new(100.0, 300.0),
            BendPoint::new(200.0, 300.0),
        ],
        "Collinear middle point (100.0, 200.0) must be pruned"
    );
}

#[test]
fn test_ac9_anchored_bends_when_nodes_moved() {
    let node_a = DiagramNode::custom(Uuid::new_v4(), "A".to_string(), 100.0, 100.0, 160.0, 80.0);
    let node_b = DiagramNode::custom(Uuid::new_v4(), "B".to_string(), 400.0, 100.0, 160.0, 80.0);

    let mut edge = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);
    let original_bends = vec![BendPoint::new(200.0, 260.0), BendPoint::new(350.0, 260.0)];
    edge.set_bends(Some(original_bends.clone()));

    // Move Node A from (100, 100) to (120, 80)
    let moved_node_a = DiagramNode::custom(node_a.id(), "A".to_string(), 120.0, 80.0, 160.0, 80.0);

    // Route edges with moved node
    let routes = EdgeRouter::route_edges(&[moved_node_a, node_b], &[edge.clone()]);
    assert_eq!(routes.len(), 1);

    // Invariant: Bends on the edge remain anchored at original canvas coordinates
    assert_eq!(
        edge.bends(),
        Some(original_bends.as_slice()),
        "Edge bends must remain anchored and untouched when node moves"
    );
}

#[test]
fn test_ac11_reset_edge_routing_and_models_sync() {
    let mut project = ModelProject::default();
    let c1 = Concept::new("Vare", "En fysisk vare", BelongsToDomain::Yes);
    let c2 = Concept::new("Lager", "En lokation", BelongsToDomain::Yes);
    let n1 = project.concept_graph_mut().add_node(&c1);
    let n2 = project.concept_graph_mut().add_node(&c2);

    let mut edge = DiagramEdge::new(n1, n2, RelationKind::Association);
    edge.set_bends(Some(vec![BendPoint::new(200.0, 300.0)]));
    project.concept_graph_mut().edges_mut().push(edge);

    assert!(project.concept_graph().edges()[0].bends().is_some());

    // Nulstil rute via update_edge_bends(None)
    project.concept_graph_mut().update_edge_bends(n1, n2, None);
    assert!(project.concept_graph().edges()[0].bends().is_none());
}

#[test]
fn test_ac7_ac8_orthogonal_segment_dragging_and_split() {
    // Node A at (100, 100), size 160x80 -> right edge is x = 260, center y = 140
    let node_a = DiagramNode::custom(Uuid::new_v4(), "A".to_string(), 100.0, 100.0, 160.0, 80.0);
    // Node B at (440, 100), size 160x80 -> left edge is x = 440, center y = 140
    let node_b = DiagramNode::custom(Uuid::new_v4(), "B".to_string(), 440.0, 100.0, 160.0, 80.0);

    let mut edge = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);

    // AC8: Split straight line segment into 3 segments with 2 bends on grid (GRID_SIZE = 20)
    let b1 = BendPoint::new(320.0, 220.0);
    let b2 = BendPoint::new(380.0, 220.0);
    edge.set_bends(Some(vec![b1, b2]));

    let routes = EdgeRouter::route_edges(&[node_a.clone(), node_b.clone()], &[edge.clone()]);
    assert_eq!(routes.len(), 1);
    let pts = &routes[0].points;

    // Must pass through the two split bends at y = 220.0
    assert!(pts
        .iter()
        .any(|p| (p.x - 320.0).abs() < 0.1 && (p.y - 220.0).abs() < 0.1));
    assert!(pts
        .iter()
        .any(|p| (p.x - 380.0).abs() < 0.1 && (p.y - 220.0).abs() < 0.1));

    // AC7: Dragging the horizontal segment (b1 to b2) in Y from 220 to 260 preserves X
    let dragged_b1 = BendPoint::new(320.0, 260.0);
    let dragged_b2 = BendPoint::new(380.0, 260.0);
    edge.set_bends(Some(vec![dragged_b1, dragged_b2]));

    let routes_dragged = EdgeRouter::route_edges(&[node_a, node_b], &[edge]);
    assert_eq!(routes_dragged.len(), 1);
    let dragged_pts = &routes_dragged[0].points;

    assert!(dragged_pts
        .iter()
        .any(|p| (p.x - 320.0).abs() < 0.1 && (p.y - 260.0).abs() < 0.1));
    assert!(dragged_pts
        .iter()
        .any(|p| (p.x - 380.0).abs() < 0.1 && (p.y - 260.0).abs() < 0.1));
}

#[test]
fn test_ac10_microsegment_absorption() {
    // Two bends closer than 4px should be absorbed into empty
    let micro_pts = vec![BendPoint::new(100.0, 100.0), BendPoint::new(102.0, 101.0)];
    let pruned = prune_collinear_bends(&micro_pts);
    assert!(pruned.is_empty(), "Microsegment under 4px must be absorbed");
}

#[test]
fn test_terminal_segment_dragging_does_not_move_distant_bend() {
    let node_a = DiagramNode::custom(
        Uuid::new_v4(),
        "Datamodel".to_string(),
        600.0,
        400.0,
        160.0,
        80.0,
    );
    let node_b = DiagramNode::custom(
        Uuid::new_v4(),
        "Model".to_string(),
        180.0,
        200.0,
        160.0,
        80.0,
    );

    let mut edge = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);
    let b0 = BendPoint::new(700.0, 340.0);
    let b1 = BendPoint::new(80.0, 340.0);
    edge.set_bends(Some(vec![b0, b1]));

    let routes = EdgeRouter::route_edges(&[node_a, node_b], &[edge.clone()]);
    assert_eq!(routes.len(), 1);
    let routed = &routes[0];

    // The route has points: P0 (Datamodel top), P1 (b0), P2 (b1), P3 (corner), P4 (Model left port)
    // Click on the terminal segment entering Model's left port (between P3 and P4) at x = 120.0, y = 240.0
    let mut state = kant::ui::canvas::DiagramCanvasState::default();
    let hit = kant::ui::canvas::edge_interaction::try_start_segment_drag(
        routed,
        Point::new(120.0, 272.0),
        Point::new(120.0, 272.0),
        &[edge],
        &mut state,
    );
    assert!(hit, "Must hit terminal segment entering Model");

    let drag = state
        .dragging_segment
        .as_ref()
        .expect("Drag segment must be started");
    assert!(
        drag.is_horizontal,
        "Segment entering left port is horizontal"
    );

    // Simulate cursor drag up in Y to 220.0
    let world_pos = Point::new(120.0, 220.0);
    kant::ui::canvas::edge_interaction::handle_bend_or_segment_cursor_drag::<()>(
        &mut state, world_pos,
    );

    let updated_drag = state.dragging_segment.as_ref().unwrap();
    // CRITICAL: Bend b1 at (80.0, 340.0) MUST NOT be moved to Y=220.0! Its Y must remain 340.0!
    let bend_1 = updated_drag
        .current_bends
        .iter()
        .find(|b| (b.x - 80.0).abs() < 1.0 && (b.y - 340.0).abs() < 1.0);
    assert!(
        bend_1.is_some(),
        "Distant bend at (80, 340) must remain intact and NOT moved to Y=220! Found bends: {:?}",
        updated_drag.current_bends
    );
}

#[test]
fn test_unselected_edge_drag_starts_on_first_press() {
    let node_a = DiagramNode::custom(Uuid::new_v4(), "A".to_string(), 100.0, 100.0, 160.0, 80.0);
    let node_b = DiagramNode::custom(Uuid::new_v4(), "B".to_string(), 440.0, 100.0, 160.0, 80.0);

    let edge = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);
    let mut state = kant::ui::canvas::DiagramCanvasState::default();

    // Edge is NOT currently selected: selected_edge is None
    let on_selected: Option<kant::ui::canvas::EdgeSelectHandler<'_, ()>> = Some(Box::new(|_| ()));

    let action = kant::ui::canvas::edge_interaction::check_edge_click(
        &[node_a, node_b],
        &[edge],
        None, // unselected!
        &mut state,
        Point::new(300.0, 140.0),
        Point::new(300.0, 140.0),
        &on_selected,
    );

    assert!(action.is_some(), "Must handle edge click");
    assert!(
        state.dragging_segment.is_some(),
        "Must immediately initialize dragging_segment on first press of an unselected edge so drag is not dropped"
    );
}
