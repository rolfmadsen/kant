//! Acceptance test for Task 095: Palette Scrollbar Edge Alignment and Ergonomics.
//! Verifies that the vertical scrollbar in the left palette of Concept Model and Information Model
//! aligns with the right edge of the card (2px fuge), has an ergonomic 6px width, and reserves
//! a dedicated clearance corridor for action controls (+, ✓, 🗑️).

use iced::widget::scrollable::Status;
use iced::Theme;
use kant::features::concept_model::ui::palette;
use kant::features::concept_model::ConceptGraph;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::models::{InformationClass, InformationModel};
use kant::features::information_model::ui::class_list;
use kant::features::information_model::ClassGraph;
use kant::ui::app::{App, Message, Tab};
use kant::ui::theme::modern_scrollable_style;

#[test]
fn test_task_095_ac1_scrollbar_width_is_6px() {
    // AC1: Scrollbaren skal have en ergonomisk bredde på 6.0px (i stedet for 4.0px)
    assert_eq!(
        palette::SCROLLBAR_WIDTH,
        6.0,
        "AC1: Begrebspalettens scrollbar bredde skal være 6.0px"
    );
    assert_eq!(
        class_list::SCROLLBAR_WIDTH,
        6.0,
        "AC1: Klasselistens scrollbar bredde skal være 6.0px"
    );
}

#[test]
fn test_task_095_ac2_and_ac3_edge_alignment_and_actions_clearance_constants() {
    // AC2: Scrollable skal spænde helt ud til kanten med kun 2px fuge mod hjørneradius
    assert_eq!(
        palette::PALETTE_RIGHT_EDGE_FUGE,
        2.0,
        "AC2: Begrebspalettens ydre højre padding skal være 2.0px for kantflugt"
    );
    assert_eq!(
        class_list::PALETTE_RIGHT_EDGE_FUGE,
        2.0,
        "AC2: Klasselistens ydre højre padding skal være 2.0px for kantflugt"
    );

    // AC3: Handlingsikonerne skal have mindst 10px dedikeret clearance-korridor
    assert_eq!(
        palette::PALETTE_ACTIONS_CLEARANCE_PADDING,
        10.0,
        "AC3: Begrebspalettens listeindhold skal have 10px højre-padding som sikkerhedszone"
    );
    assert_eq!(
        class_list::PALETTE_ACTIONS_CLEARANCE_PADDING,
        10.0,
        "AC3: Klasselistens listeindhold skal have 10px højre-padding som sikkerhedszone"
    );
}

#[test]
fn test_task_095_modern_scrollable_style_scroller_pill_radius() {
    let theme = Theme::Light;
    let hovered_status = Status::Hovered {
        is_horizontal_scrollbar_hovered: false,
        is_vertical_scrollbar_hovered: true,
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let style = modern_scrollable_style(&theme, hovered_status);

    // Pill radius for 6px bredde skal være 3.0px (halvdelen af bredden)
    let radius = style.vertical_rail.scroller.border.radius;
    assert_eq!(
        radius,
        3.0.into(),
        "Scroller pill radius skal være 3.0px for perfekt afrunding af 6px pill"
    );
    assert!(
        style.vertical_rail.background.is_none(),
        "Rail skal forblive 100% transparent (ingen grå baggrundsskinne)"
    );
}

#[test]
fn test_task_095_concept_palette_renders_with_overflow_and_edge_alignment() {
    let mut concepts = Vec::new();
    for i in 0..25 {
        concepts.push(Concept::new(
            format!("Begreb_{:02}", i),
            "Definition af begreb for test",
            BelongsToDomain::Yes,
        ));
    }
    let graph = ConceptGraph::new();
    let _widget = palette::view_palette(&concepts, &graph, None, "", 0);
}

#[test]
fn test_task_095_information_model_palette_renders_with_overflow_and_edge_alignment() {
    let mut model = InformationModel::new();
    for i in 0..25 {
        model.add_class(InformationClass::new(format!("Klasse_{:02}", i)));
    }
    let graph = ClassGraph::new();
    let _widget = class_list::view(&model, &graph, None, None, "", 0);
}

#[test]
fn test_task_095_app_integration_overflow_navigation() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    for i in 0..20 {
        let _ = app.project_mut().add_concept(Concept::new(
            format!("IntegrationsBegreb_{:02}", i),
            "Definition",
            BelongsToDomain::Yes,
        ));
    }
    {
        let _view = app.view();
    }

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    {
        let _view = app.view();
    }
}

