//! Acceptance test for Task 092: Palette Search Alignment & Header Cleanup.
//! Verifies that the redundant "+ Fra begreb..." pick_list is completely removed from the
//! Information Model palette, that search input padding is balanced for optical centering,
//! and that multi-concept aggregation remains fully supported in the domain and inspector.

use kant::features::concept_model::ui::palette;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::models::{InformationClass, InformationModel};
use kant::features::information_model::ui::class_list;
use kant::features::information_model::ClassGraph;
use kant::ui::app::{App, Message, Tab};
use uuid::Uuid;

#[test]
fn test_task_092_search_input_padding_is_balanced_for_optical_centering() {
    // AC2 & AC3: Både Begrebspaletten og Klasselisten skal anvende [7.0, 9.0] padding for optisk lodret centrering
    assert_eq!(
        palette::SEARCH_INPUT_PADDING,
        [7.0, 9.0],
        "AC2: Begrebspalettens søgefelt skal have [7.0, 9.0] padding for perfekt optisk centrering"
    );
    assert_eq!(
        class_list::SEARCH_INPUT_PADDING,
        [7.0, 9.0],
        "AC3: Klasselistens søgefelt skal have [7.0, 9.0] padding for perfekt optisk centrering"
    );
}

#[test]
fn test_task_092_class_list_view_without_concept_picklist() {
    // AC1: class_list::view skal ikke længere modtage eller fremvise concept_options pick_list
    let model = InformationModel::new();
    let graph = ClassGraph::new();

    // Kaldes direkte uden concept_options parameter
    let _widget = class_list::view(&model, &graph, None, None, "", 0);
}

#[test]
fn test_task_092_multi_concept_aggregation_and_app_rendering() {
    // AC4: Multi-begrebsaggregering på klasse og rendering i Informationsmodel
    let mut class = InformationClass::new("SammensatKlasse");
    let c1 = Uuid::new_v4();
    let c2 = Uuid::new_v4();

    class.add_concept_id(c1);
    class.add_concept_id(c2);

    assert_eq!(class.concept_ids().len(), 2);
    assert!(class.concept_ids().contains(&c1));
    assert!(class.concept_ids().contains(&c2));

    // Integration i App og fane-skift til InformationModel
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // Opret et par begreber og en klasse
    let concept = Concept::new("Bygning", "En bygning", BelongsToDomain::Yes);
    let _ = app.project_mut().add_concept(concept);
    let _ = app.project_mut().information_model_mut().add_class(class);

    // Skal kunne rendere app uden fejl eller pick_list panics
    let _view = app.view();
}
