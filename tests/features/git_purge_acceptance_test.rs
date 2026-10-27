//! Acceptance test for Task 094: Remove Git Integration.
//! Verifies that the internal Git integration and 3-way merge engine
//! are completely decommissioned and purged from Kant, while decomposed
//! project persistence (.kant/) remains fully operational.

use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::model::decomposed;
use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
use kant::ui::app::App;
use uuid::Uuid;

#[test]
fn test_task_094_git_integration_purged() {
    // 1. App-niveau: Git feature skal være deaktiveret/fjernet
    assert_eq!(
        App::has_git_feature(),
        false,
        "Git feature must be completely decommissioned and return false"
    );

    // 2. Decomposed persistence (.kant/) skal fortsat fungere 100% uafhængigt af Git
    let temp_dir =
        std::env::temp_dir().join(format!("kant_test_decomposed_nogit_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let meta = ModelMetadata::new(
        "Køretøjsmodellen NoGit",
        "Test for decomposed persistence without Git",
        "https://data.gov.dk/model/vehicle",
        "Motorstyrelsen",
        "Transport",
        "1.0.0",
        ModelStatus::Draft,
    );
    let mut project = ModelProject::new(meta);
    let c1 = Concept::new(
        "Køretøj",
        "Transportmiddel til personer eller gods",
        BelongsToDomain::Yes,
    );
    project.add_concept(c1).unwrap();

    let kant_dir = temp_dir.join(".kant");
    decomposed::save_decomposed(&project, &kant_dir)
        .expect("Decomposed project save must succeed without Git dependencies");

    assert!(kant_dir.join("metadata.json").is_file());
    assert!(kant_dir.join("concepts").is_dir());

    let loaded_project = decomposed::load_decomposed(&kant_dir)
        .expect("Decomposed project load must succeed without Git dependencies");

    assert_eq!(loaded_project.metadata().name(), "Køretøjsmodellen NoGit");
    assert_eq!(loaded_project.concepts().len(), 1);

    // Ryd op
    let _ = std::fs::remove_dir_all(&temp_dir);
}
