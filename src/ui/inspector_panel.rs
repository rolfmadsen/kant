use crate::design_system::tokens::{ColorTokens, SpacingTokens, TypographyTokens};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use super::app::Message;
use super::theme::{card_container_style, secondary_button_style};

pub const PROPERTIES_TITLE: &str = "Egenskaber";
pub const GUIDANCE_TITLE: &str = "Vejledning";

/// Ensartet header for Egenskaber-panelet med titel, type-badge og luk-knap.
pub fn panel_header<'a>(
    title: &'a str,
    badge_info: Option<(&'a str, Color, Color)>,
    on_close: Option<Message>,
) -> Element<'a, Message> {
    let mut header_row = row![
        text(title)
            .size(TypographyTokens::BODY)
            .color(ColorTokens::PRIMARY),
        Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(SpacingTokens::SM);

    if let Some((badge_text, bg, border_color)) = badge_info {
        let badge = container(
            text(badge_text)
                .size(TypographyTokens::CAPTION)
                .color(ColorTokens::TEXT_PRIMARY),
        )
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding([SpacingTokens::XXS, SpacingTokens::SM]);
        header_row = header_row.push(badge);
    }

    if let Some(msg) = on_close {
        let close_btn = button(text("✕").size(TypographyTokens::CAPTION))
            .style(secondary_button_style)
            .on_press(msg)
            .padding([SpacingTokens::XXS, SpacingTokens::SM]);
        header_row = header_row.push(close_btn);
    }

    header_row.into()
}

/// Sektionstitel med ensartet typografi på tværs af inspectorer.
pub fn section_header<'a>(title: &'a str) -> Element<'a, Message> {
    text(title)
        .size(TypographyTokens::BODY_SM)
        .color(ColorTokens::TEXT_MUTED)
        .into()
}

/// Generisk container til højre sidepanel med fast bredde og COSMIC card styling.
pub fn panel_container<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(scrollable(content))
        .style(card_container_style)
        .padding(SpacingTokens::LG)
        .width(Length::Fixed(290.0))
        .height(Length::Fill)
        .into()
}

/// Ensartet vejledningspanel vist når der ikke er noget valgt element.
pub fn guidance_panel<'a>(
    phase_subtitle: &'a str,
    tips: &[(&'a str, &'a str)],
) -> Element<'a, Message> {
    let header = row![
        text(format!("💡 {}", GUIDANCE_TITLE))
            .size(TypographyTokens::BODY)
            .color(ColorTokens::PRIMARY),
        Space::new().width(Length::Fill),
        container(
            text(phase_subtitle)
                .size(TypographyTokens::CAPTION)
                .color(ColorTokens::TEXT_MUTED),
        )
        .style(move |_| container::Style {
            background: Some(Background::Color(ColorTokens::SURFACE_MUTED.into())),
            border: Border {
                color: ColorTokens::BORDER_SUBTLE.into(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding([SpacingTokens::XXS, SpacingTokens::SM]),
    ]
    .align_y(Alignment::Center);

    let mut content = column![header].spacing(SpacingTokens::MD);

    for (tip_title, tip_body) in tips {
        let tip_card = container(
            column![
                text(*tip_title)
                    .size(TypographyTokens::BODY_SM)
                    .color(ColorTokens::TEXT_PRIMARY),
                text(*tip_body)
                    .size(TypographyTokens::CAPTION)
                    .color(ColorTokens::TEXT_MUTED),
            ]
            .spacing(SpacingTokens::XXS),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(ColorTokens::BACKGROUND.into())),
            border: Border {
                color: ColorTokens::BORDER_SUBTLE.into(),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(SpacingTokens::SM)
        .width(Length::Fill);

        content = content.push(tip_card);
    }

    panel_container(content.into())
}
