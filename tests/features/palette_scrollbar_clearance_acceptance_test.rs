//! Acceptance test for Task 091: Palette Scrollbar Action Controls Clearance.
//! Verifies that scrollbar gutter padding and expanded palette width prevent
//! vertical scrollbars from obscuring action buttons (+ and trashcan / delete).

use kant::features::concept_model::ui::palette;
use kant::features::concept_model::ConceptGraph;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::models::{InformationClass, InformationModel};
use kant::features::information_model::ui::class_list;
use kant::features::information_model::ClassGraph;
use kant::ui::app::{App, Message, Tab};

#[test]
fn test_task_091_concept_palette_scrollbar_clearance_and_width() {
    // AC1 & AC4: Begrebspaletten skal have en bredde på 268px og en scrollbar-gutter på mindst 8px (10px)
    assert_eq!(
        palette::PALETTE_WIDTH,
        268.0,
        "AC4: Begrebspalettens bredde skal være 268px for at sikre plads til både tekst og scrollbar"
    );
    assert_eq!(
        palette::SCROLLBAR_GUTTER,
        10.0,
        "AC1: Begrebspalettens scrollbar gutter skal være 10px så rullefeltet ikke dækker over knapper"
    );

    // Verificer rendering med 20 begreber
    let mut concepts = Vec::new();
    for i in 0..20 {
        concepts.push(Concept::new(
            format!("Begreb_{:02}", i),
            "Definition",
            BelongsToDomain::Yes,
        ));
    }
    let graph = ConceptGraph::new();
    let _widget = palette::view_palette(&concepts, &graph, None, "", 0);
}

#[test]
fn test_task_091_information_model_palette_scrollbar_clearance_and_width() {
    // AC2, AC3 & AC4: Klasselisten skal have en bredde på 268px og en scrollbar-gutter på mindst 8px (10px)
    assert_eq!(
        class_list::PALETTE_WIDTH,
        268.0,
        "AC4: Informationsmodellens paletbredde skal være 268px"
    );
    assert_eq!(
        class_list::SCROLLBAR_GUTTER,
        10.0,
        "AC2 & AC3: Klasselistens scrollbar gutter skal være 10px så rullefeltet ikke dækker over knapper"
    );

    // Verificer rendering med 20 klasser
    let mut model = InformationModel::new();
    for i in 0..20 {
        model.add_class(InformationClass::new(format!("Klasse_{:02}", i)));
    }
    let graph = ClassGraph::new();
    let _widget = class_list::view(&model, &graph, Vec::new(), None, None, "", 0);
}

#[test]
fn test_task_091_app_integration_tab_renders_with_overflow() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // Opret 15 begreber for at aktivere overflow og scrollbar
    for i in 0..15 {
        let c = Concept::new(
            format!("LangtBegrebsNavn_{:02}", i),
            "Aristotelisk definition",
            BelongsToDomain::Yes,
        );
        let _ = app.project_mut().add_concept(c);
    }

    assert_eq!(app.project().concepts().len(), 15);
    // Skal kunne rendre view uden panic
    let _view = app.view();
}
