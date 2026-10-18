use std::path::PathBuf;

fn get_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_default_window_settings_has_linux_application_id() {
    let _settings = kant::ui::platform::default_window_settings();
    #[cfg(target_os = "linux")]
    assert_eq!(
        _settings.platform_specific.application_id, "kant",
        "settings.platform_specific.application_id skal være sat til 'kant' så Wayland og X11 matcher kant.desktop"
    );
}

#[test]
fn test_multi_resolution_png_icons_exist_and_valid() {
    let repo_root = get_repo_root();
    let icons_dir = repo_root.join("assets/icons");

    let required_icons = [
        ("kant-48.png", 48, 48),
        ("kant-128.png", 128, 128),
        ("kant-256.png", 256, 256),
        ("kant.png", 256, 256),
    ];

    for (filename, expected_width, expected_height) in required_icons {
        let path = icons_dir.join(filename);
        assert!(path.exists(), "Ikonfilen {} skal eksistere", path.display());

        let bytes = std::fs::read(&path).expect("Kunne ikke læse ikonfil");
        assert!(!bytes.is_empty(), "Ikonfilen {} må ikke være tom", filename);

        // Valider PNG signature: 137 80 78 71 13 10 26 10
        assert!(
            bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            "Filen {} skal have en gyldig PNG header",
            filename
        );

        // Læs bredde og højde fra IHDR chunk (starter ved byte 16)
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

        assert_eq!(
            width, expected_width,
            "Filen {} skal have bredde {}",
            filename, expected_width
        );
        assert_eq!(
            height, expected_height,
            "Filen {} skal have højde {}",
            filename, expected_height
        );
    }
}

#[test]
fn test_cargo_deb_package_assets_include_pixmaps_and_icons() {
    let cargo_toml_path = get_repo_root().join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path).expect("Kunne ikke læse Cargo.toml");

    // Valider deb assets
    assert!(
        content.contains(r#"["assets/icons/kant-256.png", "usr/share/pixmaps/kant.png", "644"]"#)
            || content
                .contains(r#"["assets/icons/kant.png", "usr/share/pixmaps/kant.png", "644"]"#),
        "Cargo.toml deb packaging skal indeholde usr/share/pixmaps/kant.png som Pop!_OS fallback"
    );
    assert!(
        content.contains(
            r#"["assets/icons/kant-48.png", "usr/share/icons/hicolor/48x48/apps/kant.png", "644"]"#
        ),
        "Cargo.toml deb packaging skal indeholde 48x48 hicolor ikon"
    );
    assert!(
        content.contains(r#"["assets/icons/kant-128.png", "usr/share/icons/hicolor/128x128/apps/kant.png", "644"]"#),
        "Cargo.toml deb packaging skal indeholde 128x128 hicolor ikon"
    );
    assert!(
        content.contains(r#"["assets/icons/kant-256.png", "usr/share/icons/hicolor/256x256/apps/kant.png", "644"]"#),
        "Cargo.toml deb packaging skal indeholde 256x256 hicolor ikon"
    );
    assert!(
        content.contains(
            r#"["assets/icons/kant.svg", "usr/share/icons/hicolor/scalable/apps/kant.svg", "644"]"#
        ),
        "Cargo.toml deb packaging skal indeholde scalable SVG ikon"
    );
}

#[test]
fn test_desktop_file_standards_compliance() {
    let desktop_path = get_repo_root().join("assets/kant.desktop");
    assert!(desktop_path.exists(), "assets/kant.desktop skal eksistere");

    let content = std::fs::read_to_string(&desktop_path).expect("Kunne ikke læse kant.desktop");
    assert!(
        content.contains("StartupWMClass=kant"),
        "kant.desktop skal have StartupWMClass=kant for at matche Iced window class"
    );
    assert!(
        content.contains("Icon=kant"),
        "kant.desktop skal have Icon=kant"
    );
    assert!(
        content.contains("Exec=kant"),
        "kant.desktop skal have Exec=kant"
    );
}
