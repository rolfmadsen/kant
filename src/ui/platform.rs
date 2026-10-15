//! Platform-specifik initialisering og runtime-konfiguration.

/// Bestemmer den anbefalede standard Iced rendering backend baseret på operativsystem og eksisterende miljøvariabel.
///
/// Ifølge ADR 013 benyttes standard WGPU hardware-acceleration på alle platforme (inkl. Windows).
/// Funktionen returnerer `None`, så Iced frit kan vælge systemets hardware-renderer.
/// Eventuel eksplicit `ICED_BACKEND` (fx `tiny-skia` sat af brugeren for fejlsøgning) respekteres og overskrives ikke.
pub fn resolve_default_iced_backend(
    _is_windows: bool,
    _existing_backend: Option<&str>,
) -> Option<&'static str> {
    None
}

/// Bestemmer den anbefalede standard WGPU backend (fx `dx12` på Windows) hvis ikke sat.
///
/// På Windows tvinges `dx12` som standard for at forhindre WGPU i at enumerere
/// eventuelt defekte eller langsomme Vulkan ICD-drivere på Intel Iris Xe grafik (Task 066).
pub fn resolve_default_wgpu_backend(
    is_windows: bool,
    existing: Option<&str>,
) -> Option<&'static str> {
    if is_windows && existing.is_none() {
        Some("dx12")
    } else {
        None
    }
}

/// Initialiserer miljøvariabler og platformspecifikke standarder før Iced GUI-løkken starter.
pub fn initialize_platform_defaults() {
    crate::features::diagnostics::init_process_timer();
    crate::features::diagnostics::record_startup_milestone("platform_init");

    let existing_iced = std::env::var("ICED_BACKEND").ok();
    let existing_wgpu = std::env::var("WGPU_BACKEND").ok();

    crate::features::diagnostics::log_info(
        "platform",
        &format!(
            "Initialiserer platform. OS: {}, Arch: {}, ICED_BACKEND: {:?}, WGPU_BACKEND: {:?}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            existing_iced,
            existing_wgpu
        ),
    );

    #[cfg(target_os = "windows")]
    {
        let is_windows = true;
        if let Some(backend) = resolve_default_iced_backend(is_windows, existing_iced.as_deref()) {
            std::env::set_var("ICED_BACKEND", backend);
            crate::features::diagnostics::log_info(
                "platform",
                &format!("Overstyrede ICED_BACKEND til {}", backend),
            );
        }
        if let Some(backend) = resolve_default_wgpu_backend(is_windows, existing_wgpu.as_deref()) {
            std::env::set_var("WGPU_BACKEND", backend);
            crate::features::diagnostics::log_info(
                "platform",
                &format!("Aktiverede WGPU DX12 fast-path: WGPU_BACKEND={}", backend),
            );
        }
    }
}

/// Returnerer standard window settings for Kant med korrekt platform-identitet.
///
/// På Linux (Wayland og X11) er det afgørende, at `settings.platform_specific.application_id`
/// er sat til `"kant"` (matchende `kant.desktop` og `StartupWMClass=kant`).
/// Uden dette modtager Wayland-compositors (Mutter/GNOME, COSMIC `cosmic-comp`, KWin)
/// en tom `app_id` og falder tilbage til standardikonet (det grå tandhjul).
pub fn default_window_settings() -> iced::window::Settings {
    let mut settings = iced::window::Settings {
        size: iced::Size::new(1280.0, 800.0),
        min_size: Some(iced::Size::new(800.0, 600.0)),
        icon: crate::ui::app::load_window_icon(),
        ..Default::default()
    };
    settings.platform_specific.application_id = "kant".to_string();
    settings
}

/// Zero-privilege installation eller opdatering af Kants desktop-integration
/// i brugerens personlige XDG-mapper (`~/.local/share/`).
///
/// Installerer:
/// - `~/.local/share/applications/kant.desktop`
/// - `~/.local/share/icons/hicolor/scalable/apps/kant.svg`
/// - `~/.local/share/icons/hicolor/48x48/apps/kant.png`
/// - `~/.local/share/icons/hicolor/128x128/apps/kant.png`
/// - `~/.local/share/icons/hicolor/256x256/apps/kant.png`
/// - `~/.local/share/pixmaps/kant.png`
///
/// Trigger derefter `update-desktop-database` og `gtk-update-icon-cache` hvis tilgængelige.
pub fn install_user_desktop_entry() -> Result<Vec<std::path::PathBuf>, String> {
    let home_dir = std::env::var("HOME").map_err(|_| "HOME miljøvariabel ikke fundet".to_string())?;
    let home = std::path::Path::new(&home_dir);

    let desktop_content = include_str!("../../assets/kant.desktop");
    let svg_bytes = include_bytes!("../../assets/icons/kant.svg");
    let png_48_bytes = include_bytes!("../../assets/icons/kant-48.png");
    let png_128_bytes = include_bytes!("../../assets/icons/kant-128.png");
    let png_256_bytes = include_bytes!("../../assets/icons/kant-256.png");

    let mut written_files = Vec::new();

    // 1. Applications directory
    let apps_dir = home.join(".local/share/applications");
    std::fs::create_dir_all(&apps_dir).map_err(|e| format!("Kunne ikke oprette {:?}: {}", apps_dir, e))?;
    let desktop_target = apps_dir.join("kant.desktop");
    std::fs::write(&desktop_target, desktop_content)
        .map_err(|e| format!("Kunne ikke skrive {:?}: {}", desktop_target, e))?;
    written_files.push(desktop_target);

    // 2. Scalable icon
    let scalable_dir = home.join(".local/share/icons/hicolor/scalable/apps");
    std::fs::create_dir_all(&scalable_dir).map_err(|e| format!("Kunne ikke oprette {:?}: {}", scalable_dir, e))?;
    let svg_target = scalable_dir.join("kant.svg");
    std::fs::write(&svg_target, svg_bytes)
        .map_err(|e| format!("Kunne ikke skrive {:?}: {}", svg_target, e))?;
    written_files.push(svg_target);

    // 3. Fixed size icons
    let fixed_sizes: [(&str, &[u8]); 3] = [
        ("48x48", png_48_bytes),
        ("128x128", png_128_bytes),
        ("256x256", png_256_bytes),
    ];
    for (size, bytes) in fixed_sizes {
        let size_dir = home.join(format!(".local/share/icons/hicolor/{}/apps", size));
        std::fs::create_dir_all(&size_dir).map_err(|e| format!("Kunne ikke oprette {:?}: {}", size_dir, e))?;
        let png_target = size_dir.join("kant.png");
        std::fs::write(&png_target, bytes)
            .map_err(|e| format!("Kunne ikke skrive {:?}: {}", png_target, e))?;
        written_files.push(png_target);
    }

    // 4. Pixmaps fallback (for Pop!_OS / COSMIC)
    let pixmaps_dir = home.join(".local/share/pixmaps");
    std::fs::create_dir_all(&pixmaps_dir).map_err(|e| format!("Kunne ikke oprette {:?}: {}", pixmaps_dir, e))?;
    let pixmap_target = pixmaps_dir.join("kant.png");
    std::fs::write(&pixmap_target, png_256_bytes)
        .map_err(|e| format!("Kunne ikke skrive {:?}: {}", pixmap_target, e))?;
    written_files.push(pixmap_target);

    // 5. Opdater desktop database & icon cache hvis værktøjer er installeret
    let _ = std::process::Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output();
    let _ = std::process::Command::new("gtk-update-icon-cache")
        .args(["-f", "-t"])
        .arg(home.join(".local/share/icons/hicolor"))
        .output();

    Ok(written_files)
}
