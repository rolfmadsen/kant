use crate::features::concepts::ui::form::render_form_fields;
use crate::features::concepts::ui::state::{ConceptEditorState, BTN_CANCEL, BTN_SAVE};
use crate::ui::app::Message;
use crate::ui::theme::{
    focused_button_style, primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub fn view<'a>(state: &'a ConceptEditorState) -> Element<'a, Message> {
    let is_edit = state.editing_id.is_some();
    let title_text = if is_edit {
        format!("Rediger Begreb: {}", state.preferred_term)
    } else {
        "Opret Nyt Begreb".to_string()
    };

    let title_row = row![
        text(title_text).size(20).color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(14))
            .style(secondary_button_style)
            .on_press(Message::CancelConceptEdit),
    ]
    .align_y(Alignment::Center);

    let mut form = column![title_row].spacing(16);

    if let Some(err) = &state.validation_error {
        form = form.push(
            container(
                row![
                    text("⚠️ ").size(16),
                    text(err).color(ThemeColors::ACCENT_RED).size(13),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                border: iced::Border {
                    color: ThemeColors::ACCENT_RED,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([10, 14]),
        );
    }

    for el in render_form_fields(state) {
        form = form.push(el);
    }

    let is_cancel_focused = state.focused_button == Some(BTN_CANCEL);
    let is_save_focused = state.focused_button == Some(BTN_SAVE);

    let action_bar = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(13))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused,
            ))
            .on_press(Message::CancelConceptEdit)
            .padding([8, 16]),
        button(
            text(if is_edit {
                "Gem ændringer"
            } else {
                "Opret begreb"
            })
            .size(13)
        )
        .style(focused_button_style(primary_button_style, is_save_focused))
        .on_press(Message::SaveConcept)
        .padding([8, 20]),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    form = form.push(Space::new().height(8));
    form = form.push(action_bar);

    scrollable(form).height(Length::Fill).into()
}
