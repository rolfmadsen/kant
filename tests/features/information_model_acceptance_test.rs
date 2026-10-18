//! Acceptance tests for Task 086: Information Model Decomposition.
//! Verifies decomposition of domain and UI monolits into modular feature slices under 800 lines.

use kant::features::concept_model::RelationKind;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::{
    Attribute, ClassRelation, InformationClass, InformationModel, Multiplicity, PrimitiveType,
};
use kant::ui::app::{App, Message, Tab};
use std::fs;
use std::path::{Path, PathBuf};

fn get_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn count_file_lines(path: &Path) -> usize {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Kunne ikke læse fil {:?}: {}", path, e));
    content.lines().count()
}

#[test]
fn test_task_086_ac1_to_ac4_file_decomposition_and_line_limits() {
    let root = get_repo_root();

    // AC1: Domænemodel dekomponering i models.rs (<= 600), validation.rs (<= 500), lineage.rs (<= 400)
    let models_file = root.join("src/features/information_model/models.rs");
    assert!(
        models_file.exists(),
        "AC1: src/features/information_model/models.rs skal eksistere"
    );
    let models_lines = count_file_lines(&models_file);
    assert!(
        models_lines <= 600,
        "AC1: models.rs skal have <= 600 linjer (faktisk: {})",
        models_lines
    );

    let validation_file = root.join("src/features/information_model/validation.rs");
    assert!(
        validation_file.exists(),
        "AC1: src/features/information_model/validation.rs skal eksistere"
    );
    let validation_lines = count_file_lines(&validation_file);
    assert!(
        validation_lines <= 500,
        "AC1: validation.rs skal have <= 500 linjer (faktisk: {})",
        validation_lines
    );

    let lineage_file = root.join("src/features/information_model/lineage.rs");
    assert!(
        lineage_file.exists(),
        "AC1: src/features/information_model/lineage.rs skal eksistere"
    );
    let lineage_lines = count_file_lines(&lineage_file);
    assert!(
        lineage_lines <= 400,
        "AC1: lineage.rs skal have <= 400 linjer (faktisk: {})",
        lineage_lines
    );

    let mod_file = root.join("src/features/information_model/mod.rs");
    let mod_lines = count_file_lines(&mod_file);
    assert!(
        mod_lines <= 800,
        "AC1: src/features/information_model/mod.rs skal have <= 800 linjer (faktisk: {})",
        mod_lines
    );

    // AC2: UI Modul: Klasseliste & Metadata
    let class_list_file = root.join("src/features/information_model/ui/class_list.rs");
    assert!(
        class_list_file.exists(),
        "AC2: src/features/information_model/ui/class_list.rs skal eksistere"
    );
    let class_list_lines = count_file_lines(&class_list_file);
    assert!(
        class_list_lines <= 500,
        "AC2: class_list.rs skal have <= 500 linjer (faktisk: {})",
        class_list_lines
    );

    let class_detail_file = root.join("src/features/information_model/ui/class_detail.rs");
    assert!(
        class_detail_file.exists(),
        "AC2: src/features/information_model/ui/class_detail.rs skal eksistere"
    );
    let class_detail_lines = count_file_lines(&class_detail_file);
    assert!(
        class_detail_lines <= 500,
        "AC2: class_detail.rs skal have <= 500 linjer (faktisk: {})",
        class_detail_lines
    );

    // AC3: UI Modul: Attributter & Associationer
    let attr_editor_file = root.join("src/features/information_model/ui/attribute_editor.rs");
    assert!(
        attr_editor_file.exists(),
        "AC3: src/features/information_model/ui/attribute_editor.rs skal eksistere"
    );
    let attr_editor_lines = count_file_lines(&attr_editor_file);
    assert!(
        attr_editor_lines <= 600,
        "AC3: attribute_editor.rs skal have <= 600 linjer (faktisk: {})",
        attr_editor_lines
    );

    let assoc_editor_file = root.join("src/features/information_model/ui/association_editor.rs");
    assert!(
        assoc_editor_file.exists(),
        "AC3: src/features/information_model/ui/association_editor.rs skal eksistere"
    );
    let assoc_editor_lines = count_file_lines(&assoc_editor_file);
    assert!(
        assoc_editor_lines <= 500,
        "AC3: association_editor.rs skal have <= 500 linjer (faktisk: {})",
        assoc_editor_lines
    );

    // AC4: Fjernelse af undtagelser i gauntlet.toml
    let info_model_view_file = root.join("src/ui/information_model_view.rs");
    let info_model_view_lines = count_file_lines(&info_model_view_file);
    assert!(
        info_model_view_lines <= 800,
        "AC4: src/ui/information_model_view.rs skal have <= 800 linjer (faktisk: {})",
        info_model_view_lines
    );

    let gauntlet_toml =
        fs::read_to_string(root.join("gauntlet.toml")).expect("Kunne ikke læse gauntlet.toml");
    assert!(
        !gauntlet_toml.contains("src/ui/information_model_view.rs"),
        "AC4: gauntlet.toml må IKKE længere have exemption for src/ui/information_model_view.rs"
    );
    assert!(
        !gauntlet_toml.contains("src/features/information_model/mod.rs"),
        "AC4: gauntlet.toml må IKKE længere have exemption for src/features/information_model/mod.rs"
    );
}

#[test]
fn test_task_086_domain_lineage_and_validation() {
    let mut model = InformationModel::new();

    // 1. Opret FDA koncept og tilknyt til klasse via lineage
    let c1 = Concept::new("Køretøj", "Rullende transportmiddel", BelongsToDomain::Yes);
    let c1_id = c1.id();

    let mut cls = InformationClass::new("Koeretoej");
    cls.add_concept_id(c1_id);

    let attr = Attribute::new(
        "registreringsnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    );
    cls.add_attribute(attr);

    let cls_id = model.add_class(cls);
    assert_eq!(model.classes().len(), 1);

    // Verificer lineage queries
    let linked_classes = model.classes_for_concept(c1_id);
    assert_eq!(linked_classes.len(), 1);
    assert_eq!(linked_classes[0].id(), cls_id);

    // Verificer cascading lineage cleanup
    model.remove_concept_references(c1_id);
    let remaining_linked = model.classes_for_concept(c1_id);
    assert!(
        remaining_linked.is_empty(),
        "Lineage referencer skal fjernes efter remove_concept_references"
    );
}

#[test]
fn test_task_086_ui_model_and_relations_interaction() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // Opret to klasser og verificer relation kan tilføjes
    let cls1 = InformationClass::new("Kunde");
    let cls2 = InformationClass::new("Ordre");
    let id1 = app.project_mut().information_model_mut().add_class(cls1);
    let id2 = app.project_mut().information_model_mut().add_class(cls2);

    let rel = ClassRelation::new(id1, id2, RelationKind::Association, None);
    app.project_mut().information_model_mut().add_relation(rel);

    assert_eq!(app.project().information_model().classes().len(), 2);
    assert_eq!(app.project().information_model().relations().len(), 1);
}
