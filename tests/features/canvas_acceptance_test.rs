use iced::mouse::{Button, Cursor};
use iced::widget::canvas::{Event, Program};
use iced::{Point, Rectangle, Size};
use kant::features::concept_model::{DiagramEdge, DiagramNode};
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::diagnostics::get_recent_logs;
use kant::features::information_model::{InformationClass, InformationEnumeration};
use kant::features::model::storage::ProjectStorage;
use kant::ui::app::{App, Message, Tab};
use kant::ui::concept_editor::ConceptFormField;
use kant::ui::diagram_canvas::{CanvasViewport, DiagramCanvas, DiagramCanvasState};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

#[test]
fn test_task_067_canvas_drag_latency_optimization_drop_commit_autosave_and_telemetry() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_task_067_drag_{}.edge.json", Uuid::new_v4()));

    let mut app = App::new_with_path(Some(file_path.clone()));
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // 1. Opret begreb og node i grafen ved (100.0, 100.0)
    let c = Concept::new("Vejbane", "Areal til kørende færdsel", BelongsToDomain::Yes);
    let nid = app.project_mut().concept_graph_mut().add_node(&c);
    app.project_mut()
        .concept_graph_mut()
        .update_node_position(nid, 100.0, 100.0);

    // Initial gem for at etablere fil på disk
    app.trigger_autosave();
    assert!(file_path.exists());
    let disk_baseline = ProjectStorage::load_from_file(&file_path).unwrap();
    let disk_node = disk_baseline.concept_graph().find_node(nid).unwrap();
    assert_eq!(disk_node.x(), 100.0);
    assert_eq!(disk_node.y(), 100.0);

    // 2. Drag undervejs: GraphNodesMoved opdaterer in-memory, men må IKKE udføre autosave
    let _ = app.update(Message::GraphNodesMoved(vec![(nid, 220.0, 340.0)]));

    // Verificer in-memory er opdateret
    let mem_node = app.project().concept_graph().find_node(nid).unwrap();
    assert_eq!(mem_node.x(), 220.0);
    assert_eq!(mem_node.y(), 340.0);

    // Verificer disk STADIG har gammel position (Drop-Commit invariant: ingen autosave under træk)
    let disk_during_drag = ProjectStorage::load_from_file(&file_path).unwrap();
    let disk_node_during = disk_during_drag.concept_graph().find_node(nid).unwrap();
    assert_eq!(
        disk_node_during.x(),
        100.0,
        "Drop-Commit invariant: disk må IKKE opdateres under aktiv drag (CursorMoved/GraphNodesMoved)"
    );

    // 3. Drop afsluttet: CanvasNodesDragFinished trigger én autosave og logger telemetri
    let _ = app.update(Message::CanvasNodesDragFinished);

    // Nu SKAL disken være opdateret til (220.0, 340.0)
    let disk_after_drop = ProjectStorage::load_from_file(&file_path).unwrap();
    let disk_node_after = disk_after_drop.concept_graph().find_node(nid).unwrap();
    assert_eq!(
        disk_node_after.x(),
        220.0,
        "Disken skal være opdateret efter CanvasNodesDragFinished"
    );
    assert_eq!(disk_node_after.y(), 340.0);

    // Verificer telemetri i diagnostics log buffer
    let logs = get_recent_logs();
    assert!(
        logs.iter()
            .any(|l| l.contains("Autosave gennemført") || l.contains("Node drag completed")),
        "Diagnostics skal indeholde autosave eller drag telemetri"
    );

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_task_073_canvas_node_double_click_focuses_name_for_concept_class_and_enum() {
    let mut app = App::new();

    // 1. Informationsmodel: Dobbeltklik på Klasse
    let class = InformationClass::new("Kunde");
    let class_id = class.id();
    app.project_mut().information_model_mut().add_class(class);
    let class_node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_id, 0);

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    // Dobbeltklik på klassenoden
    let _ = app.update(Message::InfoNodeDoubleClicked(class_node_id));
    assert_eq!(
        app.selected_info_graph_node_id(),
        Some(class_node_id),
        "Klassenoden skal vælges ved dobbeltklik"
    );
    assert_eq!(
        app.selected_info_class_id(),
        Some(class_id),
        "Klassens id skal sættes som selected_info_class_id"
    );
    assert_eq!(
        app.selected_info_enum_id(),
        None,
        "Enumeration skal være None når en klasse vælges"
    );
    assert_eq!(
        app.info_inspector_focused_button(),
        None,
        "Fokus skal ligge i tekstfeltet (info_class_name_input, ikke en knap)"
    );

    // 2. Informationsmodel: Dobbeltklik på Enumeration
    let enumeration = InformationEnumeration::new("OrdreStatus", vec!["oprettet".to_string()]);
    let enum_id = enumeration.id();
    app.project_mut()
        .information_model_mut()
        .add_enumeration(enumeration);
    let enum_node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node(enum_id, 0);

    let _ = app.update(Message::InfoNodeDoubleClicked(enum_node_id));
    assert_eq!(
        app.selected_info_graph_node_id(),
        Some(enum_node_id),
        "Enumerationsnoden skal vælges ved dobbeltklik"
    );
    assert_eq!(
        app.selected_info_enum_id(),
        Some(enum_id),
        "Enumerationens id skal sættes som selected_info_enum_id"
    );
    assert_eq!(
        app.selected_info_class_id(),
        None,
        "Klasse skal være None når en enumeration vælges"
    );
    assert_eq!(
        app.info_inspector_focused_button(),
        None,
        "Fokus skal ligge i tekstfeltet (info_enum_name_input, ikke en knap)"
    );

    // 3. Begrebsmodel: Dobbeltklik på Begreb
    let concept = Concept::new("Kunde", "En part der køber", BelongsToDomain::Yes);
    let concept_id = app.project_mut().add_concept(concept).unwrap();
    let _ = app.update(Message::AddConceptToDiagram(concept_id));
    let concept_node_id = app
        .project()
        .concept_graph()
        .find_node_by_concept(concept_id)
        .unwrap()
        .id();

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphNodeDoubleClicked(concept_node_id));
    assert_eq!(
        app.selected_graph_node_id(),
        Some(concept_node_id),
        "Begrebsnoden skal vælges ved dobbeltklik"
    );
    assert!(
        app.is_node_editing(),
        "Dobbeltklik på begrebsnode skal åbne inline editor/egenskaber svarende til Enter"
    );
    assert_eq!(
        app.selected_edge(),
        None,
        "Eventuel relation skal fravælges ved dobbeltklik på node"
    );
}

#[test]
fn test_task_074_palette_selection_clears_canvas_node_selection() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // Opret Klasse A og tilføj til diagram
    let class_a = InformationClass::new("KlasseA");
    let class_a_id = class_a.id();
    app.project_mut().information_model_mut().add_class(class_a);
    let node_a = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_a_id, 0);

    // Opret Klasse B og tilføj til diagram
    let class_b = InformationClass::new("KlasseB");
    let class_b_id = class_b.id();
    app.project_mut().information_model_mut().add_class(class_b);
    let node_b = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_b_id, 0);

    // Opret Klasse C som IKKE er på diagrammet
    let class_c = InformationClass::new("KlasseC");
    let class_c_id = class_c.id();
    app.project_mut().information_model_mut().add_class(class_c);

    // Opret Enumeration D og tilføj til diagram
    let enum_d = InformationEnumeration::new("EnumD", vec!["V1".to_string()]);
    let enum_d_id = enum_d.id();
    app.project_mut()
        .information_model_mut()
        .add_enumeration(enum_d);
    let node_d = app
        .project_mut()
        .information_graph_mut()
        .add_node(enum_d_id, 0);

    // 1. Vælg Node A på canvas
    let _ = app.update(Message::SelectInfoGraphNode(Some(node_a)));
    assert_eq!(app.selected_info_graph_node_id(), Some(node_a));
    assert!(app.selected_info_graph_node_ids().contains(&node_a));

    // 2. Vælg Klasse B fra paletten
    let _ = app.update(Message::SelectInformationClass(Some(class_b_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_b_id));
    assert_eq!(app.selected_info_graph_node_id(), Some(node_b));
    assert!(
        app.selected_info_graph_node_ids().contains(&node_b),
        "Node B skal være markeret på canvas"
    );
    assert!(
        !app.selected_info_graph_node_ids().contains(&node_a),
        "Node A må IKKE længere være markeret når Klasse B vælges i paletten!"
    );

    // 3. Vælg Klasse C fra paletten (findes IKKE på canvas)
    let _ = app.update(Message::SelectInformationClass(Some(class_c_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_c_id));
    assert_eq!(app.selected_info_graph_node_id(), None);
    assert!(
        app.selected_info_graph_node_ids().is_empty(),
        "Ingen noder på canvas må være markerede når Klasse C udenfor canvas vælges!"
    );

    // 4. Vælg Node A på canvas igen
    let _ = app.update(Message::SelectInfoGraphNode(Some(node_a)));
    assert!(app.selected_info_graph_node_ids().contains(&node_a));

    // 5. Vælg Enumeration D fra paletten
    let _ = app.update(Message::SelectInformationEnumeration(Some(enum_d_id)));
    assert_eq!(app.selected_info_enum_id(), Some(enum_d_id));
    assert_eq!(app.selected_info_class_id(), None);
    assert_eq!(app.selected_info_graph_node_id(), Some(node_d));
    assert!(
        app.selected_info_graph_node_ids().contains(&node_d),
        "Node D skal være markeret på canvas"
    );
    assert!(
        !app.selected_info_graph_node_ids().contains(&node_a),
        "Node A må IKKE længere være markeret når Enumeration D vælges i paletten!"
    );
}

#[test]
fn test_task_077_click_then_drag_moves_nodes() {
    let node_id = Uuid::new_v4();
    let node = DiagramNode::custom(node_id, "TestNode".to_string(), 100.0, 100.0, 120.0, 60.0);
    let nodes = vec![node];
    let edges: Vec<DiagramEdge> = vec![];

    let node_dc_count = Arc::new(AtomicUsize::new(0));
    let canvas_dc_count = Arc::new(AtomicUsize::new(0));
    let moves_count = Arc::new(AtomicUsize::new(0));
    let drag_release_count = Arc::new(AtomicUsize::new(0));

    let ndc_clone = Arc::clone(&node_dc_count);
    let cdc_clone = Arc::clone(&canvas_dc_count);
    let m_clone = Arc::clone(&moves_count);
    let r_clone = Arc::clone(&drag_release_count);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        move |_, _| {
            cdc_clone.fetch_add(1, Ordering::SeqCst);
        },
        move |_| {
            ndc_clone.fetch_add(1, Ordering::SeqCst);
        },
        |_| (),
    )
    .on_nodes_moved(move |_| {
        m_clone.fetch_add(1, Ordering::SeqCst);
    })
    .on_drag_released(move || {
        r_clone.fetch_add(1, Ordering::SeqCst);
    });

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let click_pos = Point::new(120.0, 120.0);
    let cursor_pos = Cursor::Available(click_pos);

    // 1. Klik for at vælge noden: Press + Release
    let press1 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press1, bounds, cursor_pos);
    let release1 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release1, bounds, cursor_pos);

    // 2. Hurtigt efter (<500ms): tryk ned for at påbegynde træk
    let press2 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press2, bounds, cursor_pos);

    // 3. Bevæg markøren 20 skærm-px
    let moved_pos = Point::new(140.0, 120.0);
    let cursor_moved = Cursor::Available(moved_pos);
    let move_event = Event::Mouse(iced::mouse::Event::CursorMoved {
        position: moved_pos,
    });
    let _ = canvas.update(&mut state, &move_event, bounds, cursor_moved);

    // 4. Slip musen
    let release2 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release2, bounds, cursor_moved);

    // AC1: Dette skal være et træk, IKKE et dobbeltklik!
    assert_eq!(
        node_dc_count.load(Ordering::SeqCst),
        0,
        "Tryk efterfulgt af bevægelse må IKKE udløse on_node_double_clicked"
    );
    assert_eq!(
        canvas_dc_count.load(Ordering::SeqCst),
        0,
        "Tryk efterfulgt af bevægelse må IKKE udløse on_canvas_double_clicked"
    );
    assert!(
        moves_count.load(Ordering::SeqCst) >= 1,
        "on_nodes_moved skal være udsendt ved træk"
    );
    assert_eq!(
        drag_release_count.load(Ordering::SeqCst),
        1,
        "on_drag_released skal være udsendt ved afslutning af træk"
    );
}

#[test]
fn test_task_077_double_click_requires_same_target() {
    let node = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeA".to_string(),
        100.0,
        100.0,
        120.0,
        60.0,
    );
    let nodes = vec![node];
    let edges: Vec<DiagramEdge> = vec![];

    let canvas_dc_count = Arc::new(AtomicUsize::new(0));
    let cdc_clone = Arc::clone(&canvas_dc_count);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        move |_, _| {
            cdc_clone.fetch_add(1, Ordering::SeqCst);
        },
        |_| (),
        |_| (),
    );

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));

    // 1. Klik på noden ved (110.0, 110.0)
    let node_pos = Point::new(110.0, 110.0);
    let cursor_node = Cursor::Available(node_pos);
    let press1 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press1, bounds, cursor_node);
    let release1 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release1, bounds, cursor_node);

    // 2. Klik straks efter (<500ms) på tomt lærred ved (105.0, 95.0) - kun 15.8 skærm-px væk!
    let empty_pos = Point::new(105.0, 95.0);
    let cursor_empty = Cursor::Available(empty_pos);
    let press2 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press2, bounds, cursor_empty);
    let release2 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release2, bounds, cursor_empty);

    // AC4: Målene er forskellige (Node vs Tomt lærred) -> må IKKE udløse dobbeltklik
    assert_eq!(
        canvas_dc_count.load(Ordering::SeqCst),
        0,
        "Klik på node efterfulgt af klik på tomt lærred må IKKE udløse on_canvas_double_clicked"
    );
}

#[test]
fn test_task_077_double_click_fires_on_release() {
    let nodes: Vec<DiagramNode> = vec![];
    let edges: Vec<DiagramEdge> = vec![];

    let canvas_dc_count = Arc::new(AtomicUsize::new(0));
    let cdc_clone = Arc::clone(&canvas_dc_count);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        move |_, _| {
            cdc_clone.fetch_add(1, Ordering::SeqCst);
        },
        |_| (),
        |_| (),
    );

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let pos = Point::new(300.0, 300.0);
    let cursor = Cursor::Available(pos);

    // 1. Første klik: Press + Release
    let press1 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press1, bounds, cursor);
    let release1 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release1, bounds, cursor);

    // 2. Andet tryk (ButtonPressed)
    let press2 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press2, bounds, cursor);

    // AC2 / RED-hypotese 3: Dobbeltklik må IKKE affyres ved ButtonPressed
    assert_eq!(
        canvas_dc_count.load(Ordering::SeqCst),
        0,
        "Dobbeltklik må IKKE affyres ved ButtonPressed, men skal afvente ButtonReleased"
    );

    // 3. Andet slip (ButtonReleased)
    let release2 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release2, bounds, cursor);

    // Nu skal dobbeltklikket være affyret præcis én gang
    assert_eq!(
        canvas_dc_count.load(Ordering::SeqCst),
        1,
        "Dobbeltklik skal affyres præcis én gang ved ButtonReleased"
    );
}

#[test]
fn test_task_077_ctrl_click_does_not_participate_in_double_click() {
    let node = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeA".to_string(),
        100.0,
        100.0,
        120.0,
        60.0,
    );
    let nodes = vec![node];
    let edges: Vec<DiagramEdge> = vec![];

    let node_dc_count = Arc::new(AtomicUsize::new(0));
    let ndc_clone = Arc::clone(&node_dc_count);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        |_, _| (),
        move |_| {
            ndc_clone.fetch_add(1, Ordering::SeqCst);
        },
        |_| (),
    );

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let pos = Point::new(120.0, 120.0);
    let cursor = Cursor::Available(pos);

    // 1. Ctrl-klik på noden (f.eks. multiselect):
    state.modifiers = iced::keyboard::Modifiers::CTRL;
    let press1 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press1, bounds, cursor);
    let release1 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release1, bounds, cursor);

    // 2. Almindeligt klik uden Ctrl straks efter (<500ms):
    state.modifiers = iced::keyboard::Modifiers::empty();
    let press2 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press2, bounds, cursor);
    let release2 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release2, bounds, cursor);

    // AC5: Ctrl-klik må ikke starte en dobbeltklik-sekvens
    assert_eq!(
        node_dc_count.load(Ordering::SeqCst),
        0,
        "Ctrl-klik efterfulgt af klik må IKKE udløse on_node_double_clicked"
    );
}

#[test]
fn test_task_077_cancel_empty_quick_create_rolls_back() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.project().concepts().len(), 0);

    // 1. Dobbeltklik på tomt lærred lynopretter et begreb med tomt preferred_term
    let _ = app.update(Message::CanvasDoubleClicked(250.0, 150.0));
    assert_eq!(app.project().concepts().len(), 1);

    // 2. Annuller redigering uden at udfylde navn (CancelConceptEdit)
    let _ = app.update(Message::CancelConceptEdit);

    // AC8: Tomt lynoprettet begreb skal rulles tilbage og fjernes helt
    assert_eq!(
        app.project().concepts().len(),
        0,
        "Tomt lynoprettet begreb skal fjernes fra projektet ved CancelConceptEdit"
    );
    assert_eq!(
        app.project().concept_graph().nodes().len(),
        0,
        "Grafnoden for det tomme lynoprettede begreb skal fjernes"
    );

    // 3. Test at begreb MED udfyldt term IKKE rulles tilbage ved annullering:
    let _ = app.update(Message::CanvasDoubleClicked(300.0, 200.0));
    assert_eq!(app.project().concepts().len(), 1);
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Faktura".to_string(),
    ));
    let _ = app.update(Message::CancelConceptEdit);
    assert_eq!(
        app.project().concepts().len(),
        1,
        "Begreb med udfyldt navn må IKKE rulles tilbage ved annullering"
    );
}

#[test]
fn test_task_077_telemetry_sources_and_node_double_click() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // 1. Lynoprettelse via CanvasDoubleClicked -> log "via dobbeltklik"
    let _ = app.update(Message::CanvasDoubleClicked(100.0, 150.0));
    let logs1 = get_recent_logs();
    assert!(
        logs1
            .iter()
            .any(|l| l.contains("Lynoprettede begreb") && l.contains("via dobbeltklik")),
        "CanvasDoubleClicked skal logge telemetri med 'via dobbeltklik'"
    );

    // 2. Lynoprettelse via CreateConceptAtCenter -> log "via værktøjslinje"
    let _ = app.update(Message::CreateConceptAtCenter);
    let logs2 = get_recent_logs();
    assert!(
        logs2
            .iter()
            .any(|l| l.contains("Lynoprettede begreb") && l.contains("via værktøjslinje")),
        "CreateConceptAtCenter skal logge telemetri med 'via værktøjslinje'"
    );

    // 3. Dobbeltklik på node i grafen -> log "Dobbeltklik på node ... – åbner egenskaber"
    let concept = Concept::new("Vej", "Færdselsareal", BelongsToDomain::Yes);
    let cid = app.project_mut().add_concept(concept).unwrap();
    let nid = app
        .project()
        .concept_graph()
        .find_node_by_concept(cid)
        .unwrap()
        .id();
    let _ = app.update(Message::GraphNodeDoubleClicked(nid));
    let logs3 = get_recent_logs();
    assert!(
        logs3
            .iter()
            .any(|l| l.contains("Dobbeltklik på node") && l.contains("åbner egenskaber")),
        "GraphNodeDoubleClicked skal logge telemetri"
    );
}

#[test]
fn test_task_077_zoom_independent_threshold() {
    let node = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeA".to_string(),
        100.0,
        100.0,
        120.0,
        60.0,
    );
    let nodes = vec![node];
    let edges: Vec<DiagramEdge> = vec![];

    let drag_release_count = Arc::new(AtomicUsize::new(0));
    let r_clone = Arc::clone(&drag_release_count);

    let mut vp_zoomed_out = CanvasViewport::default();
    vp_zoomed_out.set_zoom(0.25);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        vp_zoomed_out,
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        |_, _| (),
        |_| (),
        |_| (),
    )
    .on_drag_released(move || {
        r_clone.fetch_add(1, Ordering::SeqCst);
    });

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let screen_node_pos = Point::new(25.0, 25.0);
    let cursor = Cursor::Available(screen_node_pos);

    // 1. Press på node
    let press = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press, bounds, cursor);

    // 2. Flyt 2 skærm-px (svarende til 8 verdens-px ved zoom 0.25)
    let moved_2px = Point::new(27.0, 25.0);
    let cursor_moved = Cursor::Available(moved_2px);
    let move_event = Event::Mouse(iced::mouse::Event::CursorMoved {
        position: moved_2px,
    });
    let _ = canvas.update(&mut state, &move_event, bounds, cursor_moved);

    // 3. Slip musen
    let release = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release, bounds, cursor_moved);

    // AC6: 2 skærm-px er < 4 skærm-px -> må IKKE tælle som drag release!
    assert_eq!(
        drag_release_count.load(Ordering::SeqCst),
        0,
        "Bevægelse på 2 skærm-px ved zoom 0.25 må IKKE udløse drag_released (skal bruge skærm-slop)"
    );
}

#[test]
fn test_task_077_stationary_double_click_on_node_emits_double_clicked() {
    let node_id = Uuid::new_v4();
    let node = DiagramNode::custom(
        node_id,
        "StationaryNode".to_string(),
        100.0,
        100.0,
        120.0,
        60.0,
    );
    let nodes = vec![node];
    let edges: Vec<DiagramEdge> = vec![];

    let node_dc_count = Arc::new(AtomicUsize::new(0));
    let moves_count = Arc::new(AtomicUsize::new(0));
    let drag_release_count = Arc::new(AtomicUsize::new(0));

    let ndc_clone = Arc::clone(&node_dc_count);
    let m_clone = Arc::clone(&moves_count);
    let r_clone = Arc::clone(&drag_release_count);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        |_, _, _, _| (),
        |_| (),
        |_, _, _| (),
        |_, _| (),
        move |id| {
            if id == node_id {
                ndc_clone.fetch_add(1, Ordering::SeqCst);
            }
        },
        |_| (),
    )
    .on_nodes_moved(move |_| {
        m_clone.fetch_add(1, Ordering::SeqCst);
    })
    .on_drag_released(move || {
        r_clone.fetch_add(1, Ordering::SeqCst);
    });

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let pos = Point::new(130.0, 130.0);
    let cursor = Cursor::Available(pos);

    // 1. Første klik (stillestående): Press + Release
    let press1 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press1, bounds, cursor);
    let release1 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release1, bounds, cursor);

    // 2. Andet klik (stillestående inden for 500ms): Press + Release
    let press2 = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press2, bounds, cursor);
    assert_eq!(
        node_dc_count.load(Ordering::SeqCst),
        0,
        "Dobbeltklik må ikke affyres før ButtonReleased"
    );

    let release2 = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(&mut state, &release2, bounds, cursor);

    // AC3: on_node_double_clicked affyres præcis én gang ved andet slip
    assert_eq!(
        node_dc_count.load(Ordering::SeqCst),
        1,
        "on_node_double_clicked skal affyres præcis én gang"
    );
    // Ingen on_drag_released og ingen on_nodes_moved
    assert_eq!(
        drag_release_count.load(Ordering::SeqCst),
        0,
        "on_drag_released må IKKE affyres ved stillestående dobbeltklik"
    );
    assert_eq!(
        moves_count.load(Ordering::SeqCst),
        0,
        "Nodepositioner må IKKE ændres ved stillestående dobbeltklik"
    );
}
