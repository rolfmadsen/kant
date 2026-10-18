//! Enumeration inspector for FDA Information Model.

use crate::design_system::tokens::ColorTokens;
use crate::features::information_model::diagram::ClassGraph;
use crate::features::information_model::linter::NamingLinter;
use crate::features::information_model::models::InformationEnumeration;
use crate::ui::app::Message;
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view<'a>(
    e: &'a InformationEnumeration,
    class_graph: &'a ClassGraph,
    new_enum_value_input: &'a str,
    inspector_focused_button: Option<usize>,
) -> Element<'a, Message> {
    let enum_id = e.id();
    let is_on_canvas = class_graph.is_class_on_diagram(enum_id);

    let header = crate::ui::inspector_panel::panel_header(
        crate::ui::inspector_panel::PROPERTIES_TITLE,
        Some((
            "Enumeration",
            ThemeColors::FDA_ENUM_GREEN,
            ColorTokens::FDA_ENUM_ACCENT.into_color(),
        )),
        Some(Message::SelectInformationEnumeration(None)),
    );

    let mut name_col = column![text_input("Enumerationsnavn...", e.name())
        .id("info_enum_name_input")
        .style(modern_input_style)
        .size(13.0)
        .on_input(move |s| Message::UpdateInformationEnumerationName(enum_id, s))
        .padding([4, 6])]
    .spacing(4);

    if let Some(issue) = NamingLinter::check_enumeration_name(e.name()) {
        if !e.name().trim().is_empty() {
            let mut msg = format!("⚠️ {}: {}", issue.rule, issue.message);
            if let Some(fix) = &issue.suggested_fix {
                msg.push_str(&format!(" (Forslag: {})", fix));
            }
            name_col = name_col.push(
                text(msg)
                    .size(10.5)
                    .color(ColorTokens::WARNING_AMBER.into_color()),
            );
        }
    }

    let def_input = text_input("Valgfri definition...", e.definition().unwrap_or(""))
        .id("info_enum_def_input")
        .style(modern_input_style)
        .size(12.0)
        .on_input(move |s| Message::UpdateInformationEnumerationDefinition(enum_id, s))
        .padding([4, 6]);

    let is_canvas_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_CANVAS_ACTION);
    let canvas_btn = if is_on_canvas {
        button(text("Fjern fra diagram").size(11))
            .style(focused_button_style(
                secondary_button_style,
                is_canvas_focused,
            ))
            .on_press(Message::RemoveEnumerationFromDiagram(enum_id))
            .padding([4, 8])
    } else {
        button(text("+ Tilføj til diagram").size(11))
            .style(focused_button_style(
                primary_button_style,
                is_canvas_focused,
            ))
            .on_press(Message::AddEnumerationToDiagram(enum_id))
            .padding([4, 8])
    };

    let is_del_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_DELETE_ENUM);
    let actions_row = row![
        canvas_btn,
        Space::new().width(Length::Fill),
        button(text("🗑️ Slet enumeration").size(11))
            .style(focused_button_style(danger_button_style, is_del_focused,))
            .on_press(Message::DeleteInformationEnumeration(enum_id))
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let mut values_list = column![].spacing(4);
    for (v_idx, val) in e.values().iter().enumerate() {
        let is_rem_focused = inspector_focused_button
            == Some(crate::ui::app::INFO_INSPECTOR_ENUM_VAL_REMOVE_START + v_idx);
        let val_str = val.clone();
        values_list = values_list.push(
            container(
                row![
                    text(val)
                        .size(12)
                        .color(ThemeColors::SLATE_800)
                        .width(Length::Fill),
                    button(text("✕").size(9))
                        .style(focused_button_style(secondary_button_style, is_rem_focused,))
                        .on_press(Message::RemoveValueFromEnumeration(enum_id, val_str))
                        .padding([1, 4]),
                ]
                .align_y(Alignment::Center),
            )
            .style(card_container_style)
            .padding(4)
            .width(Length::Fill),
        );
    }

    let is_add_val_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_ADD_ENUM_VAL);
    let mut val_section = column![
        crate::ui::inspector_panel::section_header("Værdier (FDA Tabel B)"),
        row![
            text_input("f.eks. aktiv, inaktiv...", new_enum_value_input)
                .id("info_new_enum_val_input")
                .style(modern_input_style)
                .size(12.0)
                .on_input(Message::NewEnumValueInputChanged)
                .on_submit(Message::AddValueToEnumeration(
                    enum_id,
                    new_enum_value_input.to_string()
                ))
                .padding([3, 6])
                .width(Length::Fill),
            button(text("+").size(12))
                .style(focused_button_style(
                    secondary_button_style,
                    is_add_val_focused
                ))
                .on_press(Message::AddValueToEnumeration(
                    enum_id,
                    new_enum_value_input.to_string()
                ))
                .padding([3, 8]),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    ]
    .spacing(6);

    if !new_enum_value_input.trim().is_empty() {
        if let Some(issue) = NamingLinter::check_enumeration_value(new_enum_value_input) {
            let mut msg = format!("⚠️ {}: {}", issue.rule, issue.message);
            if let Some(fix) = &issue.suggested_fix {
                msg.push_str(&format!(" (Forslag: {})", fix));
            }
            val_section = val_section.push(
                text(msg)
                    .size(10.0)
                    .color(ColorTokens::WARNING_AMBER.into_color()),
            );
        }
    }
    val_section = val_section.push(values_list);

    let content = column![
        header,
        column![
            crate::ui::inspector_panel::section_header("Identifikation"),
            name_col,
            def_input
        ]
        .spacing(6),
        val_section,
        actions_row,
    ]
    .spacing(10);

    crate::ui::inspector_panel::panel_container(content.into())
}
