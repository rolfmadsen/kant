use crate::features::concepts::{BelongsToDomain, Concept, ConceptValidator, ValidationError};
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct QuickCreateState {
    pub position: (f32, f32),
    pub preferred_term: String,
    pub definition: String,
    pub belongs_to_domain: BelongsToDomain,
    pub validation_error: Option<String>,
}

impl QuickCreateState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            preferred_term: String::new(),
            definition: String::new(),
            belongs_to_domain: BelongsToDomain::Yes,
            validation_error: None,
        }
    }

    pub fn build_concept(&self) -> Result<Concept, ValidationError> {
        let concept = Concept::new(
            self.preferred_term.trim(),
            self.definition.trim(),
            self.belongs_to_domain.clone(),
        );
        ConceptValidator::validate(&concept)?;
        Ok(concept)
    }
}

pub fn view_quick_create_modal<'a>(
    qc: &'a QuickCreateState,
    modal_focus: &'a FocusCycle,
) -> Element<'a, Message> {
    let mut dialog_col = column![
        row![
            text("Nyt Begreb på Lærred")
                .size(16)
                .color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(13))
                .style(secondary_button_style)
                .on_press(Message::QuickCreateCancel)
                .padding([3, 7]),
        ]
        .align_y(Alignment::Center),
        text(format!(
            "Opretter begreb ved ({:.0}, {:.0}) på lærredet jf. FDA Modelreglerne:",
            qc.position.0, qc.position.1
        ))
        .size(12)
        .color(ThemeColors::TEXT_MUTED),
    ]
    .spacing(12);

    if let Some(err) = &qc.validation_error {
        dialog_col = dialog_col.push(
            container(
                text(format!("⚠️ {}", err))
                    .size(12)
                    .color(ThemeColors::ACCENT_RED),
            )
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                border: iced::Border {
                    color: ThemeColors::ACCENT_RED,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([6, 10])
            .width(Length::Fill),
        );
    }

    let term_field = column![
        text("Foretrukken term *")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input("Foretrukken term (f.eks. Godsvogn)...", &qc.preferred_term)
            .id("quick_create_term_input")
            .style(modern_input_style)
            .on_input(Message::QuickCreateTermChanged)
            .on_submit(Message::QuickCreateSubmit)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4);

    let def_field = column![
        text("Definition (Aristoteles' formel) *")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "Overordnet begreb + adskillende egenskaber...",
            &qc.definition
        )
        .style(modern_input_style)
        .on_input(Message::QuickCreateDefinitionChanged)
        .on_submit(Message::QuickCreateSubmit)
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4);

    let is_local = qc.belongs_to_domain == BelongsToDomain::Yes;
    let domain_row = row![
        text("Tilknytning:").size(12).color(ThemeColors::SLATE_700),
        button(text("Lokalt begreb (FDA Sand)").size(11))
            .style(if is_local {
                primary_button_style
            } else {
                secondary_button_style
            })
            .on_press(Message::QuickCreateDomainChanged(BelongsToDomain::Yes))
            .padding([4, 8]),
        button(text("Indlånt begreb (FDA Blå)").size(11))
            .style(if !is_local {
                primary_button_style
            } else {
                secondary_button_style
            })
            .on_press(Message::QuickCreateDomainChanged(BelongsToDomain::No))
            .padding([4, 8]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    dialog_col = dialog_col.push(term_field);
    dialog_col = dialog_col.push(def_field);
    dialog_col = dialog_col.push(domain_row);

    let is_cancel_focused = modal_focus.is_focused(0);
    let is_submit_focused = modal_focus.is_focused(1);

    let actions = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused
            ))
            .on_press(Message::QuickCreateCancel)
            .padding([6, 14]),
        button(text("Opret Begreb").size(12))
            .style(focused_button_style(
                primary_button_style,
                is_submit_focused
            ))
            .on_press(Message::QuickCreateSubmit)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    dialog_col = dialog_col.push(actions);

    let modal_card = container(dialog_col)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(480.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
