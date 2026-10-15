use std::fs::File;
use std::path::{Path, PathBuf};

fn get_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_appx_manifest_structure_and_identity() {
    let manifest_path = get_repo_root().join("assets/windows/AppxManifest.xml");
    assert!(
        manifest_path.exists(),
        "assets/windows/AppxManifest.xml skal eksistere"
    );

    let content = std::fs::read_to_string(&manifest_path).expect("Kunne ikke læse AppxManifest.xml");
    assert!(
        content.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#),
        "XML deklaration skal være præcis lowercase version=\"1.0\""
    );
    assert!(
        !content.contains("x-default"),
        "Må IKKE indeholde pseudo-sprogkoden x-default"
    );
    assert!(
        content.contains(r#"<Resource Language="da-DK" />"#),
        "Skal indeholde da-DK som sprogdeklaration"
    );

    // Valider nøgleelementer og Partner Center identitet
    assert!(content.contains(r#"Name="RolfMadsen.Kant""#), "Identity Name skal være RolfMadsen.Kant");
    assert!(
        content.contains(r#"Publisher="CN=6101A799-2390-4F81-88B1-08BC634BEC12""#),
        "Publisher skal matche Partner Center CN"
    );
    assert!(
        content.contains("<PublisherDisplayName>Rolf Madsen</PublisherDisplayName>"),
        "PublisherDisplayName skal være Rolf Madsen"
    );
    assert!(
        content.contains("<DisplayName>Kant</DisplayName>"),
        "DisplayName skal være Kant"
    );
    assert!(
        content.contains(r#"EntryPoint="Windows.FullTrustApplication""#),
        "EntryPoint skal være Windows.FullTrustApplication"
    );
    assert!(
        content.contains(r#"Executable="kant.exe""#),
        "Executable skal være kant.exe"
    );
    assert!(
        content.contains(r#"<rescap:Capability Name="runFullTrust"/>"#)
            || content.contains(r#"<rescap:Capability Name="runFullTrust" />"#),
        "Skal have runFullTrust capability"
    );
    assert!(
        content.contains(r#"Category="windows.fileTypeAssociation""#),
        "Skal registrere filtypeassociation"
    );
    assert!(
        content.contains(r#"<uap:FileType>.kant</uap:FileType>"#),
        "Skal associere .kant filer"
    );
}

fn verify_png_dimensions(path: &Path, expected_width: u32, expected_height: u32) {
    assert!(
        path.exists(),
        "Billedfilen {:?} skal eksistere",
        path.file_name().unwrap()
    );

    let file = File::open(path).expect("Kunne ikke åbne PNG fil");
    let decoder = png::Decoder::new(file);
    let reader = decoder.read_info().expect("Kunne ikke dekode PNG header");
    let info = reader.info();

    assert_eq!(
        info.width, expected_width,
        "{:?} har forkert bredde: forventede {}, fik {}",
        path.file_name().unwrap(),
        expected_width,
        info.width
    );
    assert_eq!(
        info.height, expected_height,
        "{:?} har forkert højde: forventede {}, fik {}",
        path.file_name().unwrap(),
        expected_height,
        info.height
    );
}

#[test]
fn test_windows_store_assets_exist_and_have_exact_dimensions() {
    let images_dir = get_repo_root().join("assets/windows/images");

    verify_png_dimensions(&images_dir.join("Square44x44Logo.png"), 44, 44);
    verify_png_dimensions(&images_dir.join("Square50x50Logo.png"), 50, 50);
    verify_png_dimensions(&images_dir.join("Square150x150Logo.png"), 150, 150);
    verify_png_dimensions(&images_dir.join("Wide310x150Logo.png"), 310, 150);
    verify_png_dimensions(&images_dir.join("SplashScreen.png"), 620, 300);
}
