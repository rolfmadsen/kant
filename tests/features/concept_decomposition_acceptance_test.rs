//! Acceptance tests for Task 087: Concept Studio and Model Decomposition.
//! Verifies decomposition of concept editor, excel, concept model view, and edge router into modular slices under 800 lines.

use kant::features::concept_model::{ConceptGraph, RelationKind};
use kant::features::concepts::{BelongsToDomain, Concept, DefinitionLinter};
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
fn test_task_087_ac1_to_ac5_decomposition_and_line_limits() {
    let root = get_repo_root();

    // AC1: Begrebsliste Editor Dekomponering
    let concept_ui_dir = root.join("src/features/concepts/ui");
    assert!(
        concept_ui_dir.exists(),
        "AC1: src/features/concepts/ui/ skal eksistere som modul"
    );

    let concept_editor_file = root.join("src/ui/concept_editor.rs");
    let concept_editor_lines = count_file_lines(&concept_editor_file);
    assert!(
        concept_editor_lines <= 400,
        "AC1: src/ui/concept_editor.rs facade skal have <= 400 linjer (faktisk: {})",
        concept_editor_lines
    );

    // AC2: Excel Import & Eksport Dekomponering
    let excel_dir = root.join("src/features/concepts/excel");
    assert!(
        excel_dir.exists(),
        "AC2: src/features/concepts/excel/ skal eksistere som modulmappe"
    );
    let excel_mod = excel_dir.join("mod.rs");
    assert!(
        excel_mod.exists(),
        "AC2: src/features/concepts/excel/mod.rs skal eksistere"
    );
    let excel_mod_lines = count_file_lines(&excel_mod);
    assert!(
        excel_mod_lines <= 500,
        "AC2: excel/mod.rs skal have <= 500 linjer (faktisk: {})",
        excel_mod_lines
    );

    // AC3: Begrebsmodel Studio Dekomponering
    let concept_model_ui_dir = root.join("src/features/concept_model/ui");
    assert!(
        concept_model_ui_dir.exists(),
        "AC3: src/features/concept_model/ui/ skal eksistere som modul"
    );
    let concept_model_view_file = root.join("src/ui/concept_model_view.rs");
    let concept_model_view_lines = count_file_lines(&concept_model_view_file);
    assert!(
        concept_model_view_lines <= 400,
        "AC3: src/ui/concept_model_view.rs facade skal have <= 400 linjer (faktisk: {})",
        concept_model_view_lines
    );

    // AC4: Edge Router Dekomponering
    let edge_router_file = root.join("src/ui/edge_router.rs");
    let edge_router_lines = count_file_lines(&edge_router_file);
    assert!(
        edge_router_lines <= 500,
        "AC4: src/ui/edge_router.rs skal have <= 500 linjer (faktisk: {})",
        edge_router_lines
    );

    // AC5: Fjernelse af de 4 undtagelser i gauntlet.toml
    let gauntlet_toml =
        fs::read_to_string(root.join("gauntlet.toml")).expect("Kunne ikke læse gauntlet.toml");
    assert!(
        !gauntlet_toml.contains("src/ui/concept_editor.rs"),
        "AC5: gauntlet.toml må IKKE længere have exemption for src/ui/concept_editor.rs"
    );
    assert!(
        !gauntlet_toml.contains("src/features/concepts/excel.rs"),
        "AC5: gauntlet.toml må IKKE længere have exemption for src/features/concepts/excel.rs"
    );
    assert!(
        !gauntlet_toml.contains("src/ui/concept_model_view.rs"),
        "AC5: gauntlet.toml må IKKE længere have exemption for src/ui/concept_model_view.rs"
    );
    assert!(
        !gauntlet_toml.contains("src/ui/edge_router.rs"),
        "AC5: gauntlet.toml må IKKE længere have exemption for src/ui/edge_router.rs"
    );
}

#[test]
fn test_task_087_concept_and_linter_domain_operations() {
    let mut concept = Concept::new(
        "Køretøj",
        "et transportmiddel der bevæger sig på hjul",
        BelongsToDomain::Yes,
    );
    concept.set_source(Some("Færdselsloven § 2".to_string()));

    let issues = DefinitionLinter::lint_text(concept.preferred_term(), None, concept.definition());
    assert!(
        issues.is_empty(),
        "Gyldig definition efter genus proximum bør ikke give linterfejl"
    );

    let mut graph = ConceptGraph::new();
    let n1 = graph.add_node(&concept);
    let c2 = Concept::new(
        "Bil",
        "Et motordrevet køretøj med fire hjul",
        BelongsToDomain::Yes,
    );
    let n2 = graph.add_node(&c2);

    graph.add_relation(n2, n1, RelationKind::Generalization);
    assert_eq!(graph.nodes().len(), 2);
    assert_eq!(graph.edges().len(), 1);
}

#[test]
fn test_task_087_concept_app_tab_navigation() {
    let mut app = App::new();
    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    assert_eq!(app.active_tab(), Tab::ConceptList);

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);
}
