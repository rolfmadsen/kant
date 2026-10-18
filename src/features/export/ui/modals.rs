use crate::features::concepts::excel::get_fda_schema_guide_entries;
use crate::features::export::ui::state::ExcelImportPreviewState;
use crate::features::model::ModelProject;
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, pill_container_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub fn view_excel_schema_guide_modal<'a>(modal_focus: &FocusCycle) -> Element<'a, Message> {
    let title_row = row![
        text("ℹ️ FDA Begrebsliste - Excel Skemaguide")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseExcelSchemaGuideModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text(
        "Oversigt over felter og kolonner i den officielle FDA Excel-skabelon (SKOS-kompatibel med tosprogethed):",
    )
    .size(12)
    .color(ThemeColors::TEXT_MUTED);

    let entries = get_fda_schema_guide_entries();
    let mut entry_cards = column![].spacing(8);

    for entry in entries {
        let badge = if entry.required {
            container(text("Obligatorisk").size(10).color(ThemeColors::ACCENT_RED))
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                    border: iced::Border {
                        color: ThemeColors::ACCENT_RED,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding([2, 6])
        } else {
            container(text("Valgfri").size(10).color(ThemeColors::SLATE_600))
                .style(pill_container_style)
                .padding([2, 6])
        };

        let header = row![
            text(entry.name).size(13).color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            text(entry.tag).size(11).color(ThemeColors::PRIMARY),
            Space::new().width(6),
            badge,
        ]
        .align_y(Alignment::Center);

        let desc = text(entry.description)
            .size(12)
            .color(ThemeColors::SLATE_700);
        let example = text(format!("Eksempel / format: {}", entry.example))
            .size(11)
            .color(ThemeColors::TEXT_MUTED);

        let card = container(column![header, desc, example].spacing(4))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
                border: iced::Border {
                    color: ThemeColors::SLATE_200,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([8, 12])
            .width(Length::Fill);

        entry_cards = entry_cards.push(card);
    }

    let scrollable_entries = scrollable(entry_cards).height(Length::Fixed(400.0));

    let is_export_focused = modal_focus.is_focused(0);
    let is_close_focused = modal_focus.is_focused(1);

    let actions_row = row![
        button(
            row![
                text("📄").size(13),
                Space::new().width(6),
                text("Hent officiel FDA-skabelon (.xlsx)").size(12),
            ]
            .align_y(Alignment::Center),
        )
        .style(focused_button_style(
            secondary_button_style,
            is_export_focused,
        ))
        .on_press(Message::ExportFdaTemplateXlsxDialog)
        .padding([6, 14]),
        Space::new().width(Length::Fill),
        button(text("Luk").size(12))
            .style(focused_button_style(primary_button_style, is_close_focused))
            .on_press(Message::CloseExcelSchemaGuideModal)
            .padding([6, 18]),
    ]
    .align_y(Alignment::Center);

    let modal_card = container(
        column![
            title_row,
            subtitle,
            scrollable_entries,
            Space::new().height(4),
            actions_row,
        ]
        .spacing(12),
    )
    .style(modal_card_style)
    .padding(20)
    .width(Length::Fixed(680.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn view_excel_import_preview_modal<'a>(
    preview: &'a ExcelImportPreviewState,
    project: &ModelProject,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let file_name = preview
        .file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Excel-fil");

    let title_row = row![
        text("📥 Forhåndsvisning af Excel Import")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::DismissExcelImportModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let file_info = text(format!(
        "Valgt regneark: {} (Ark: {})",
        file_name, preview.import_summary.sheet_name
    ))
    .size(12)
    .color(ThemeColors::TEXT_MUTED);

    let stats_row = row![
        container(
            row![
                text("Gyldige i Excel:")
                    .size(12)
                    .color(ThemeColors::SLATE_600),
                Space::new().width(6),
                text(format!("{}", preview.import_summary.valid_concepts.len()))
                    .size(13)
                    .color(ThemeColors::SLATE_900),
            ]
            .align_y(Alignment::Center),
        )
        .style(pill_container_style)
        .padding([4, 10]),
        container(
            row![
                text("+ Nye begreber:")
                    .size(12)
                    .color(ThemeColors::ACCENT_GREEN),
                Space::new().width(6),
                text(format!("{}", preview.new_count))
                    .size(13)
                    .color(ThemeColors::ACCENT_GREEN),
            ]
            .align_y(Alignment::Center),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::ACCENT_GREEN_LIGHT)),
            border: iced::Border {
                color: ThemeColors::ACCENT_GREEN,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..Default::default()
        })
        .padding([4, 10]),
        container(
            row![
                text("🔄 Opdateres (upsert):")
                    .size(12)
                    .color(ThemeColors::PRIMARY),
                Space::new().width(6),
                text(format!("{}", preview.update_count))
                    .size(13)
                    .color(ThemeColors::PRIMARY),
            ]
            .align_y(Alignment::Center),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
            border: iced::Border {
                color: ThemeColors::PRIMARY,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..Default::default()
        })
        .padding([4, 10]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let notice_card = container(
        text("💡 Intelligent upsert matcher på begrebets Identifikator (URI) eller Foretrukken term (case-insensitiv). Eksisterende begrebers interne Uuid bevares uændret, så eksisterende diagram-placeringer og relationer aldrig brydes.")
            .size(12)
            .color(ThemeColors::SLATE_700),
    )
    .style(|_| container::Style {
        background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
        border: iced::Border {
            color: ThemeColors::SLATE_200,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .padding([8, 12])
    .width(Length::Fill);

    let mut preview_list = column![].spacing(6);
    for concept in &preview.import_summary.valid_concepts {
        let is_match = project.concepts().iter().any(|existing| {
            (concept.identifier().is_some() && concept.identifier() == existing.identifier())
                || concept
                    .preferred_term()
                    .trim()
                    .eq_ignore_ascii_case(existing.preferred_term().trim())
        });

        let status_badge = if is_match {
            container(text("Opdateres").size(10).color(ThemeColors::PRIMARY))
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                    border: iced::Border {
                        color: ThemeColors::PRIMARY,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding([1, 6])
        } else {
            container(text("Ny").size(10).color(ThemeColors::ACCENT_GREEN))
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::ACCENT_GREEN_LIGHT)),
                    border: iced::Border {
                        color: ThemeColors::ACCENT_GREEN,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding([1, 6])
        };

        let row_item = row![
            status_badge,
            Space::new().width(6),
            text(concept.preferred_term())
                .size(12)
                .color(ThemeColors::SLATE_900)
                .width(Length::FillPortion(2)),
            text(concept.definition())
                .size(11)
                .color(ThemeColors::TEXT_MUTED)
                .width(Length::FillPortion(3)),
        ]
        .align_y(Alignment::Center);

        preview_list = preview_list.push(row_item);
    }

    let scrollable_list = scrollable(preview_list).height(Length::Fixed(240.0));

    let is_cancel_focused = modal_focus.is_focused(0);
    let is_confirm_focused = modal_focus.is_focused(1);

    let actions_row = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused,
            ))
            .on_press(Message::DismissExcelImportModal)
            .padding([6, 14]),
        button(row![text("Bekræft og Gennemfør Import").size(12),].align_y(Alignment::Center),)
            .style(focused_button_style(
                primary_button_style,
                is_confirm_focused,
            ))
            .on_press(Message::ConfirmExcelImport)
            .padding([6, 16]),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let mut modal_content = column![title_row, file_info, stats_row, notice_card,].spacing(12);

    if !preview.import_summary.rejected_rows.is_empty() {
        let mut rej_col = column![
            text(format!(
                "⚠️ {} række(r) blev sprunget over pga. manglende påkrævet Foretrukken term eller Definition:",
                preview.import_summary.rejected_rows.len()
            ))
            .size(12)
            .color(ThemeColors::ACCENT_RED),
        ]
        .spacing(2);

        for rej in preview.import_summary.rejected_rows.iter().take(5) {
            rej_col = rej_col.push(
                text(format!(
                    "• Række {}: \"{}\" ({})",
                    rej.row_index, rej.preferred_term, rej.reason
                ))
                .size(11)
                .color(ThemeColors::ACCENT_RED),
            );
        }
        if preview.import_summary.rejected_rows.len() > 5 {
            rej_col = rej_col.push(
                text(format!(
                    "... og {} mere",
                    preview.import_summary.rejected_rows.len() - 5
                ))
                .size(10)
                .color(ThemeColors::ACCENT_RED),
            );
        }

        modal_content = modal_content.push(
            container(rej_col)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                    border: iced::Border {
                        color: ThemeColors::ACCENT_RED,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([8, 12])
                .width(Length::Fill),
        );
    }

    modal_content = modal_content.push(scrollable_list);
    modal_content = modal_content.push(Space::new().height(4));
    modal_content = modal_content.push(actions_row);

    let modal_card = container(modal_content)
        .style(modal_card_style)
        .padding(20)
        .width(Length::Fixed(660.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
