#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kant::ui::app::App;
use kant::ui::platform::{default_window_settings, initialize_platform_defaults};

fn main() -> iced::Result {
    initialize_platform_defaults();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--diagnostics" || arg == "-d") {
        let diag = kant::features::diagnostics::get_system_diagnostics();
        println!("{}", diag.formatted_report());
        return Ok(());
    }

    if args
        .iter()
        .any(|arg| arg == "--install-desktop" || arg == "-i")
    {
        match kant::ui::platform::install_user_desktop_entry() {
            Ok(paths) => {
                println!("Kant desktop integration installeret for brugeren:");
                for p in paths {
                    println!("  {}", p.display());
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Fejl under desktop integration: {}", e);
                std::process::exit(1);
            }
        }
    }

    let initial_path = args
        .get(1)
        .filter(|s| !s.starts_with('-'))
        .map(std::path::PathBuf::from);
    let app_creator = move || {
        kant::features::diagnostics::record_startup_milestone("app_creator_invoked");
        App::boot(initial_path.clone())
    };

    kant::features::diagnostics::record_startup_milestone("iced_application_init");
    iced::application(app_creator, App::update, App::view)
        .title("Kant - Begrebs- og Informationsmodellering med FDA")
        .theme(App::theme)
        .style(|_state, _theme| iced::theme::Style {
            background_color: kant::ui::theme::ThemeColors::SURFACE_BG,
            text_color: kant::ui::theme::ThemeColors::SLATE_800,
        })
        .subscription(App::subscription)
        .window(default_window_settings())
        // 4x MSAA antialiasing giver silkebløde vektorlinjer og kurver på Linux og macOS.
        // På Windows holdes det deaktiveret for at undgå GPU DX12 swapchain-stalls under Aero Snap (ADR 010).
        .antialiasing(cfg!(not(target_os = "windows")))
        .run()
}
