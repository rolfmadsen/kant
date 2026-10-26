//! Acceptance test for Task 090: Unified Palette Combobox and Selective Canvas Placement.
//! Verifies selective canvas placement, palette combobox keyboard navigation, auto-focus,
//! viewport centering on placed nodes, and viewport-center placement of unplaced nodes.

use iced::Point;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::models::InformationClass;
use kant::ui::app::{App, Message, Tab};
use kant::ui::canvas::CanvasViewport;
use uuid::Uuid;

#[test]
fn test_task_090_ac1_selective_canvas_placement_for_concepts() {
    let mut app = App::new();

    // 1. Initialt er lærredet tomt
    assert_eq!(app.project().concepts().len(), 0);
    assert_eq!(app.project().concept_graph().nodes().len(), 0);

    // 2. Tilføj begreb til modellen (svarende til oprettelse i Fane 1 eller Excel-import)
    let c1 = Concept::new("Vej", "Færdselsareal for køretøjer", BelongsToDomain::Yes);
    let c1_id = c1.id();
    let res = app.project_mut().add_concept(c1);
    assert!(res.is_ok());

    // 3. Begrebet findes i kataloget, men må IKKE automatisk placeres på diagramcanvas
    assert_eq!(app.project().concepts().len(), 1);
    assert_eq!(
        app.project().concept_graph().nodes().len(),
        0,
        "AC1: add_concept må IKKE automatisk tilføje noder til concept_graph"
    );
    assert!(
        !app.project().concept_graph().is_concept_on_diagram(c1_id),
        "AC1: c1 må ikke være markeret som værende på diagrammet"
    );

    // 4. Tilføj et andet begreb via fletning/tilføjelse
    let c2 = Concept::new("Kørebane", "Del af vej bestemt for kørende", BelongsToDomain::Yes);
    let c2_id = c2.id();
    let _ = app.project_mut().add_concept(c2);

    assert_eq!(app.project().concepts().len(), 2);
    assert_eq!(
        app.project().concept_graph().nodes().len(),
        0,
        "AC1: Ingen af de tilføjede begreber må auto-dumpes på canvas"
    );
    assert!(!app.project().concept_graph().is_concept_on_diagram(c2_id));
}

#[test]
fn test_task_090_ac7_canvas_viewport_center_on_rect() {
    let mut vp = CanvasViewport::default();
    vp.set_zoom(1.25);

    // Målrektangel i verdenskoordinater: (x: 400.0, y: 300.0, w: 180.0, h: 80.0)
    // Rektanglets centrum i verden: x = 490.0, y = 340.0
    let screen_center = Point::new(600.0, 400.0);

    vp.center_on_rect(400.0, 300.0, 180.0, 80.0, screen_center);

    // Verificer at zoom er bevaret
    assert_eq!(vp.zoom(), 1.25, "AC7: Zoom skal forblive uændret ved centrering");

    // Beregn skærmposition af rektanglets midtpunkt
    let world_center = Point::new(490.0, 340.0);
    let transformed_screen = vp.to_screen(world_center);

    assert!(
        (transformed_screen.x - screen_center.x).abs() < 1e-3,
        "AC7: X-koordinat skal matche screen_center præcist (faktisk: {}, forventet: {})",
        transformed_screen.x,
        screen_center.x
    );
    assert!(
        (transformed_screen.y - screen_center.y).abs() < 1e-3,
        "AC7: Y-koordinat skal matche screen_center præcist (faktisk: {}, forventet: {})",
        transformed_screen.y,
        screen_center.y
    );
}

#[test]
fn test_task_090_ac3_and_ac4_palette_combobox_navigation_and_center_placed_node() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // Opret to begreber i projektet
    let c1 = Concept::new("Bygning", "Fast konstruktion", BelongsToDomain::Yes);
    let c2 = Concept::new("Vejstykke", "Afgrænset del af vej", BelongsToDomain::Yes);
    let c1_id = c1.id();
    let c2_id = c2.id();
    let _ = app.project_mut().add_concept(c1);
    let _ = app.project_mut().add_concept(c2);

    // Placer c1 på canvas ved (600.0, 400.0)
    let _ = app.update(Message::AddConceptToDiagramAt(c1_id, 600.0, 400.0));
    assert!(app.project().concept_graph().is_concept_on_diagram(c1_id));
    assert!(!app.project().concept_graph().is_concept_on_diagram(c2_id));

    // Naviger paletten med piletaster
    // Begreber sorteres alfabetisk: "Bygning", "Vejstykke"
    let _ = app.update(Message::ConceptPaletteNext);
    // Index er nu 1 ("Vejstykke")
    assert_eq!(app.concept_palette_index(), 1);

    let _ = app.update(Message::ConceptPalettePrevious);
    // Index er nu 0 ("Bygning")
    assert_eq!(app.concept_palette_index(), 0);

    // Tryk Enter på index 0 ("Bygning" - som ER på canvas)
    let _ = app.update(Message::ConceptPaletteSubmit);

    // Noden for Bygning skal være valgt
    let node1 = app.project().concept_graph().find_node_by_concept(c1_id).unwrap();
    assert_eq!(app.selected_graph_node_id(), Some(node1.id()));

    // Viewport pan skal være opdateret til at centrere på noden
    let node_center = Point::new(node1.x() + node1.width() / 2.0, node1.y() + node1.height() / 2.0);
    let screen_pos = app.canvas_viewport().to_screen(node_center);
    let expected_center = app.canvas_visible_center();
    assert!(
        (screen_pos.x - expected_center.x).abs() < 1.0,
        "AC4: Canvas skal centrere på Bygning noden (skærm X: {}, forventet X: {})",
        screen_pos.x,
        expected_center.x
    );
}

#[test]
fn test_task_090_ac5_enter_places_unplaced_concept_at_viewport_center() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // Opret u-placeret begreb
    let c = Concept::new("Kommune", "Geografisk og administrativ enhed", BelongsToDomain::Yes);
    let c_id = c.id();
    let _ = app.project_mut().add_concept(c);

    assert!(!app.project().concept_graph().is_concept_on_diagram(c_id));

    // Vælg begrebet i paletten og tryk Enter
    assert_eq!(app.concept_palette_index(), 0);
    let _ = app.update(Message::ConceptPaletteSubmit);

    // Nu SKAL begrebet være tilføjet til canvas i centrum af viewporten
    assert!(
        app.project().concept_graph().is_concept_on_diagram(c_id),
        "AC5: Enter på u-placeret begreb skal placere det på canvas"
    );

    let node = app.project().concept_graph().find_node_by_concept(c_id).unwrap();
    assert_eq!(app.selected_graph_node_id(), Some(node.id()));

    // Verificer 20px grid snapping
    assert_eq!(node.x() % 20.0, 0.0, "AC5: Node X skal være snappet til 20px grid");
    assert_eq!(node.y() % 20.0, 0.0, "AC5: Node Y skal være snappet til 20px grid");
}

#[test]
fn test_task_090_ac6_escape_handling_in_palette_search() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // Sørg for at paletten er åben
    if !app.show_left_sidebar() {
        let _ = app.update(Message::ToggleLeftSidebar);
    }
    assert!(app.show_left_sidebar());

    // Skriv i søgefeltet
    let _ = app.update(Message::ConceptModelSearchChanged("bro".to_string()));
    assert_eq!(app.concept_model_search(), "bro");

    // 1. tryk på Escape rydder søgeteksten, men holder paletten åben
    let _ = app.update(Message::EscapePressed);
    assert_eq!(app.concept_model_search(), "", "AC6: Escape skal rydde søgefelt");
    assert!(app.show_left_sidebar(), "AC6: Paletten forbliver åben ved første Escape");

    // 2. tryk på Escape (med tomt søgefelt) lukker paletten
    let _ = app.update(Message::EscapePressed);
    assert!(!app.show_left_sidebar(), "AC6: Escape med tomt søgefelt skal lukke paletten");
}

#[test]
fn test_task_090_information_model_palette_combobox_and_placement() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // Opret en informationsklasse i modellen
    let cls = InformationClass::new("Ejendom");
    let cls_id = cls.id();
    app.project_mut().information_model_mut().add_class(cls);

    assert!(!app.project().information_graph().is_class_on_diagram(cls_id));

    // Naviger paletten og tryk Enter på u-placeret klasse
    assert_eq!(app.info_palette_index(), 0);
    let _ = app.update(Message::InfoPaletteSubmit);

    // Klassen skal nu være placeret på UML canvas og valgt
    assert!(
        app.project().information_graph().is_class_on_diagram(cls_id),
        "AC5: Enter på u-placeret klasse i Fane 3 skal placere den på canvas"
    );
    assert_eq!(app.selected_info_class_id(), Some(cls_id));
}
