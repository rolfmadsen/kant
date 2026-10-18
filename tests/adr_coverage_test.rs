//! ADR 015 Coverage & Verification Integration Test for Task 081.

use std::path::PathBuf;
use std::process::Command;

fn get_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_adr_015_file_exists_and_has_accepted_status() {
    let adr_path = get_repo_root().join("docs/adr/015-accessible-design-tokens-and-decomposed-ui-slices.md");
    assert!(
        adr_path.exists(),
        "ADR 015 filen skal eksistere på docs/adr/015-accessible-design-tokens-and-decomposed-ui-slices.md"
    );

    let content = std::fs::read_to_string(&adr_path).expect("Kunne ikke læse ADR 015 filen");

    // Valider frontmatter
    assert!(
        content.contains("type: Architectural Decision Record"),
        "ADR 015 skal have type: Architectural Decision Record"
    );
    assert!(
        content.contains("status: accepted"),
        "ADR 015 skal have status: accepted"
    );
    assert!(
        content.contains("ADR 015"),
        "ADR 015 skal indeholde 'ADR 015' i titel"
    );

    // Valider WCAG AA og Package-by-Feature indhold
    assert!(
        content.contains("4.5:1"),
        "ADR 015 skal specificere minimum 4.5:1 kontrastforhold jf. WCAG AA"
    );
    assert!(
        content.contains("Package-by-Feature") || content.contains("package-by-feature"),
        "ADR 015 skal specificere Package-by-Feature princippet"
    );
    assert!(
        content.contains("src/features/"),
        "ADR 015 skal henvise til src/features/ for dekomponerede UI slices"
    );
}

#[test]
fn test_adr_015_referenced_in_documentation() {
    let repo_root = get_repo_root();

    // 1. docs/adr/README.md
    let adr_readme = std::fs::read_to_string(repo_root.join("docs/adr/README.md"))
        .expect("Kunne ikke læse docs/adr/README.md");
    assert!(
        adr_readme.contains("015-accessible-design-tokens-and-decomposed-ui-slices.md"),
        "docs/adr/README.md skal referere til ADR 015"
    );

    // 2. spec.md
    let spec = std::fs::read_to_string(repo_root.join("spec.md"))
        .expect("Kunne ikke læse spec.md");
    assert!(
        spec.contains("015-accessible-design-tokens-and-decomposed-ui-slices.md"),
        "spec.md skal referere til ADR 015"
    );

    // 3. README.md
    let readme = std::fs::read_to_string(repo_root.join("README.md"))
        .expect("Kunne ikke læse README.md");
    assert!(
        readme.contains("015-accessible-design-tokens-and-decomposed-ui-slices.md"),
        "README.md skal referere til ADR 015"
    );
}

#[test]
fn test_xgauntlet_check_release_passes_with_100_percent_adr_coverage() {
    let status = Command::new("xgauntlet")
        .arg("check-release")
        .current_dir(get_repo_root())
        .status()
        .expect("Kunne ikke eksekvere xgauntlet check-release");

    assert!(
        status.success(),
        "xgauntlet check-release skal returnere exit code 0 med fuld ADR dækning"
    );
}
