use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    Selected(PathBuf),
    Cancelled,
    Unavailable,
}

/// Åbner native filvælger til at vælge en eksisterende FDA modelprojektfil (*.kant.json / *.edge.json / *.json)
pub fn pick_file_to_open() -> DialogResult {
    let dialog = rfd::FileDialog::new()
        .set_title("Åbn FDA Modelprojekt")
        .add_filter(
            "FDA Modelprojekter (*.kant.json, *.edge.json, *.json)",
            &["kant.json", "edge.json", "json"],
        )
        .add_filter("Alle filer", &["*"]);

    match dialog.pick_file() {
        Some(path) => DialogResult::Selected(path),
        None => DialogResult::Cancelled,
    }
}

/// Åbner native filvælger til at vælge en Excel-fil (*.xlsx, *.xls) til import
pub fn pick_excel_file_to_import() -> DialogResult {
    let dialog = rfd::FileDialog::new()
        .set_title("Vælg Excel-regneark med begreber (*.xlsx, *.xls)")
        .add_filter("Excel-arbejdsbøger (*.xlsx, *.xls)", &["xlsx", "xls"])
        .add_filter("Alle filer", &["*"]);

    match dialog.pick_file() {
        Some(path) => DialogResult::Selected(path),
        None => DialogResult::Cancelled,
    }
}

/// Åbner native filvælger til at gemme modelprojekt som ny fil
pub fn pick_file_to_save(default_name: Option<&str>) -> DialogResult {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Gem FDA Modelprojekt som...")
        .add_filter(
            "FDA Modelprojekter (*.kant.json, *.edge.json, *.json)",
            &["kant.json", "edge.json", "json"],
        )
        .add_filter("Alle filer", &["*"]);

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(name);
    } else {
        dialog = dialog.set_file_name("model.kant.json");
    }

    match dialog.save_file() {
        Some(mut path) => {
            if path.extension().is_none() {
                path.set_extension("kant.json");
            }
            DialogResult::Selected(path)
        }
        None => DialogResult::Cancelled,
    }
}

/// Åbner native filvælger til eksport af SVG, CSV eller modelrapporter
pub fn pick_file_to_export(default_name: &str, filter_name: &str, extension: &str) -> DialogResult {
    let dialog = rfd::FileDialog::new()
        .set_title(format!("Eksporter {}", filter_name))
        .add_filter(filter_name, &[extension])
        .add_filter("Alle filer", &["*"])
        .set_file_name(default_name);

    match dialog.save_file() {
        Some(mut path) => {
            if path.extension().is_none() {
                path.set_extension(extension);
            }
            DialogResult::Selected(path)
        }
        None => DialogResult::Cancelled,
    }
}

/// Åbner native mappevælger til at vælge en destinationsmappe til f.eks. Git-kloning
pub fn pick_folder() -> DialogResult {
    let dialog = rfd::FileDialog::new().set_title("Vælg mappe til model");
    match dialog.pick_folder() {
        Some(path) => DialogResult::Selected(path),
        None => DialogResult::Cancelled,
    }
}

/// Scanner en mappe for eksisterende *.kant.json, *.edge.json eller model*.json filer
pub fn scan_local_project_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if filename.ends_with(".kant.json")
                    || filename.ends_with(".kant")
                    || filename.ends_with(".edge.json")
                    || filename.ends_with(".edge")
                {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

use crate::ui::app::{FileDialogMode, Message};
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view_file_dialog_modal<'a>(
    mode: FileDialogMode,
    input: &'a str,
    modal_focus: &'a FocusCycle,
) -> Element<'a, Message> {
    let (mode_title, confirm_label) = match mode {
        FileDialogMode::Open => ("Åbn FDA Modelprojekt", "Åbn"),
        FileDialogMode::SaveAs => ("Gem Modelprojekt som...", "Gem"),
    };

    let mut dialog_body = column![
        row![
            text(mode_title).size(16).color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(13))
                .style(secondary_button_style)
                .on_press(Message::CloseFileDialog)
                .padding([3, 7]),
        ]
        .align_y(Alignment::Center),
        text("Angiv filsti eller gennemse filer på maskinen:")
            .size(12)
            .color(ThemeColors::TEXT_MUTED),
        row![
            text_input("Filsti (f.eks. model.kant.json)...", input)
                .id("file_dialog_input")
                .style(modern_input_style)
                .on_input(Message::FileDialogInputChanged)
                .on_submit(Message::ConfirmFileDialog)
                .padding(8)
                .width(Length::Fill),
            button(text("🖥️ Gennemse...").size(12))
                .style(secondary_button_style)
                .on_press(match mode {
                    FileDialogMode::Open => Message::OpenProjectDialog,
                    FileDialogMode::SaveAs => Message::SaveProjectAsDialog,
                })
                .padding([8, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(12);

    if mode == FileDialogMode::Open {
        let local_files = scan_local_project_files(&PathBuf::from("."));
        if !local_files.is_empty() {
            let mut chips = column![text("Filer i projektmappen:")
                .size(12)
                .color(ThemeColors::TEXT_MUTED)]
            .spacing(6);

            let mut chip_row = row![].spacing(6);
            for f in local_files {
                let name = f
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("model.kant.json")
                    .to_string();
                chip_row = chip_row.push(
                    button(text(format!("📄 {}", name)).size(11))
                        .style(secondary_button_style)
                        .on_press(Message::OpenProjectFile(f))
                        .padding([3, 8]),
                );
            }
            chips = chips.push(chip_row);
            dialog_body = dialog_body.push(chips);
        }
    }

    let is_cancel_focused = modal_focus.is_focused(0);
    let is_confirm_focused = modal_focus.is_focused(1);

    let actions_row = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused,
            ))
            .on_press(Message::CloseFileDialog)
            .padding([6, 14]),
        button(text(confirm_label).size(12))
            .style(focused_button_style(
                primary_button_style,
                is_confirm_focused,
            ))
            .on_press(Message::ConfirmFileDialog)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    dialog_body = dialog_body.push(actions_row);

    let modal_card = container(dialog_body)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(520.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
