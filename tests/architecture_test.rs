//! Architectural Ratchet & Quality Gate integration test for Task 080.

use std::path::PathBuf;
use std::process::Command;

fn get_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_gauntlet_toml_architecture_configuration() {
    let toml_path = get_repo_root().join("gauntlet.toml");
    assert!(toml_path.exists(), "gauntlet.toml skal eksistere");

    let content = std::fs::read_to_string(&toml_path).expect("Kunne ikke læse gauntlet.toml");

    // 1. [architecture] sektion og grænser
    assert!(
        content.contains("[architecture]"),
        "gauntlet.toml skal indeholde [architecture] sektionen"
    );
    assert!(
        content.contains("max_file_lines = 800"),
        "gauntlet.toml skal deklarere max_file_lines = 800"
    );
    assert!(
        content.contains("max_test_file_lines = 1000"),
        "gauntlet.toml skal deklarere max_test_file_lines = 1000"
    );
    assert!(
        content.contains(r#"forbidden_tokens = ["Color::from_rgb", "Color::from_rgba"]"#),
        "gauntlet.toml skal have forbidden_tokens = [\"Color::from_rgb\", \"Color::from_rgba\"]"
    );

    // 2. Ratchet-undtagelser for monolitterne
    let required_exemptions = [
        ("src/ui/app.rs", 9200),
        ("tests/acceptance.rs", 9500),
        ("src/ui/information_model_view.rs", 1950),
        ("src/features/information_model/mod.rs", 1750),
        ("src/features/model/merge.rs", 1200),
        ("src/features/git/service.rs", 1000),
        ("src/ui/concept_model_view.rs", 900),
        ("src/ui/concept_editor.rs", 900),
        ("src/ui/edge_router.rs", 900),
        ("src/features/concepts/excel.rs", 850),
    ];

    for (path, ceiling) in required_exemptions {
        assert!(
            content.contains(&format!("path = \"{}\"", path)),
            "gauntlet.toml skal indeholde exemption for '{}'",
            path
        );
        assert!(
            content.contains(&format!("max_lines = {}", ceiling)),
            "gauntlet.toml skal have max_lines = {} for '{}'",
            ceiling,
            path
        );
    }
}

#[test]
fn test_gauntlet_toml_architecture_verification_layer() {
    let toml_path = get_repo_root().join("gauntlet.toml");
    let content = std::fs::read_to_string(&toml_path).expect("Kunne ikke læse gauntlet.toml");

    // Valider at architecture layer er til stede som non-optional
    assert!(
        content.contains("name = \"architecture\""),
        "gauntlet.toml skal have et verification layer med name = 'architecture'"
    );
    assert!(
        content.contains(r#"command = ["xgauntlet", "check-architecture"]"#),
        "gauntlet.toml skal kalde ['xgauntlet', 'check-architecture'] i architecture layer"
    );
}

#[test]
fn test_xgauntlet_check_architecture_passes_cleanly() {
    let status = Command::new("xgauntlet")
        .arg("check-architecture")
        .current_dir(get_repo_root())
        .status()
        .expect("Kunne ikke eksekvere xgauntlet check-architecture");

    assert!(
        status.success(),
        "xgauntlet check-architecture skal returnere exit code 0"
    );
}
