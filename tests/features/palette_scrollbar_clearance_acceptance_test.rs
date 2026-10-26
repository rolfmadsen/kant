//! Acceptance test for Task 091: Palette Scrollbar Action Controls Clearance.
//! Verifies that modern auto-hide scrollbar styling, transparent rail, slim 4px width,
//! and balanced card margins provide an elegant, clutter-free palette aesthetic.

use iced::widget::scrollable::Status;
use iced::{Background, Color, Theme};
use kant::features::concept_model::ui::palette;
use kant::features::concept_model::ConceptGraph;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::models::{InformationClass, InformationModel};
use kant::features::information_model::ui::class_list;
use kant::features::information_model::ClassGraph;
use kant::ui::app::{App, Message, Tab};
use kant::ui::theme::modern_scrollable_style;

#[test]
fn test_task_091_modern_scrollable_auto_hide_styling() {
    let theme = Theme::Light;

    // AC1: I aktiv/idle tilstand skal scrollbaren være 100% usynlig (transparent scroller og transparent rail)
    let active_status = Status::Active {
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let active_style = modern_scrollable_style(&theme, active_status);
    assert!(
        active_style.vertical_rail.background.is_none(),
        "AC1: Rail må ikke have nogen hård baggrundsskinne i idle tilstand"
    );
    assert_eq!(
        active_style.vertical_rail.scroller.background,
        Background::Color(Color::TRANSPARENT),
        "AC1: Scroller skal være usynlig (transparent) ved inaktivitet"
    );

    // AC1: Ved hover over scrollable skal en diskret, afrundet pill tone frem uden baggrundsskinne
    let hovered_status = Status::Hovered {
        is_horizontal_scrollbar_hovered: false,
        is_vertical_scrollbar_hovered: false,
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let hovered_style = modern_scrollable_style(&theme, hovered_status);
    assert!(
        hovered_style.vertical_rail.background.is_none(),
        "AC2: Rail skal forblive transparent uden uigennemsigtig grå skinne ved hover"
    );
    assert_ne!(
        hovered_style.vertical_rail.scroller.background,
        Background::Color(Color::TRANSPARENT),
        "AC1: Scroller skal have synlig farve ved hover"
    );

    // AC1: Ved drag skal scrolleren have forhøjet kontrast
    let dragged_status = Status::Dragged {
        is_horizontal_scrollbar_dragged: false,
        is_vertical_scrollbar_dragged: true,
        is_horizontal_scrollbar_disabled: true,
        is_vertical_scrollbar_disabled: false,
    };
    let dragged_style = modern_scrollable_style(&theme, dragged_status);
    assert!(dragged_style.vertical_rail.background.is_none());
    assert_ne!(
        dragged_style.vertical_rail.scroller.background,
        Background::Color(Color::TRANSPARENT)
    );
}

#[test]
fn test_task_091_concept_palette_scrollbar_dimensions_and_width() {
    // AC2 & AC4: Begrebspaletten skal have en bredde på 268px og slank scrollbar på 4px
    assert_eq!(
        palette::PALETTE_WIDTH,
        268.0,
        "AC4: Begrebspalettens bredde skal være 268px for at sikre plads til både tekst og scrollbar"
    );
    assert_eq!(
        palette::SCROLLBAR_WIDTH,
        4.0,
        "AC2: Begrebspalettens scrollbar bredde skal være 4px (slank, moderne pill)"
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
fn test_task_091_information_model_palette_scrollbar_dimensions_and_width() {
    // AC2, AC3 & AC4: Klasselisten skal have en bredde på 268px og en slank scrollbar på 4px
    assert_eq!(
        class_list::PALETTE_WIDTH,
        268.0,
        "AC4: Informationsmodellens paletbredde skal være 268px"
    );
    assert_eq!(
        class_list::SCROLLBAR_WIDTH,
        4.0,
        "AC2: Klasselistens scrollbar bredde skal være 4px"
    );

    // Verificer rendering med 20 klasser
    let mut model = InformationModel::new();
    for i in 0..20 {
        model.add_class(InformationClass::new(format!("Klasse_{:02}", i)));
    }
    let graph = ClassGraph::new();
    let _widget = class_list::view(&model, &graph, None, None, "", 0);
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
