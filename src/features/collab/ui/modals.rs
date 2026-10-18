use crate::features::collab::ui::state::{
    GuestEndedNoticeModalState, JoinSessionModalState, RelayServerPreset, StartSessionModalState,
    START_SESSION_BTN_CANCEL, START_SESSION_BTN_COPY, START_SESSION_BTN_PRESET,
    START_SESSION_BTN_REMEMBER, START_SESSION_BTN_START,
};
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view_start_session_modal<'a>(
    modal_state: &'a StartSessionModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("🌐 Start Live Session (Vært)")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseCollabModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text("Start en end-to-end krypteret (E2EE) samarbejdssession. Vælg relay-server og del sessionskoden med dine deltagere:")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    let is_preset_focused = modal_focus.is_button_focused(START_SESSION_BTN_PRESET);
    let preset_pick = pick_list(
        &RelayServerPreset::ALL[..],
        Some(modal_state.preset),
        Message::CollabPresetSelected,
    )
    .padding(7)
    .width(Length::Fill);

    let preset_container = container(preset_pick).style(move |_| container::Style {
        border: iced::Border {
            color: if is_preset_focused {
                ThemeColors::FOCUS_RING
            } else {
                iced::Color::TRANSPARENT
            },
            width: if is_preset_focused { 1.5 } else { 0.0 },
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let preset_field = column![
        text("Relay Server Preset")
            .size(12)
            .color(ThemeColors::SLATE_700),
        preset_container,
    ]
    .spacing(4);

    let custom_url_field = if modal_state.preset == RelayServerPreset::Custom
        || modal_state.preset == RelayServerPreset::InternalOrg
    {
        column![
            text("Server WebSocket URL")
                .size(12)
                .color(ThemeColors::SLATE_700),
            text_input("wss://relay.eksempel.dk/ws", &modal_state.custom_url)
                .id("start_session_custom_url")
                .style(modern_input_style)
                .on_input(Message::CollabCustomUrlChanged)
                .padding(8)
                .width(Length::Fill),
        ]
        .spacing(4)
    } else {
        column![
            text("Aktiv Server URL")
                .size(12)
                .color(ThemeColors::SLATE_500),
            container(
                text(modal_state.current_url())
                    .size(12)
                    .color(ThemeColors::SLATE_700),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::SURFACE_BG)),
                border: iced::Border {
                    color: ThemeColors::SURFACE_BORDER,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([8, 10])
            .width(Length::Fill),
        ]
        .spacing(4)
    };

    let is_remember_focused = modal_focus.is_button_focused(START_SESSION_BTN_REMEMBER);
    let remember_inner = row![
        checkbox(modal_state.remember_choice)
            .on_toggle(Message::CollabToggleRememberPreset)
            .size(16),
        Space::new().width(6),
        text("Husk servervalg til fremtidige sessioner")
            .size(12)
            .color(ThemeColors::SLATE_700),
    ]
    .align_y(Alignment::Center);

    let remember_checkbox = container(remember_inner)
        .style(move |_| container::Style {
            border: iced::Border {
                color: if is_remember_focused {
                    ThemeColors::FOCUS_RING
                } else {
                    iced::Color::TRANSPARENT
                },
                width: if is_remember_focused { 1.5 } else { 0.0 },
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding([2, 4]);

    let token_str = modal_state.ticket.to_token();
    let token_box = column![
        text("Sessionskode (Sessionsbillet med E2EE-nøgle)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        container(text(token_str).size(11).color(ThemeColors::PRIMARY))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                border: iced::Border {
                    color: ThemeColors::PRIMARY,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4);

    let copy_label = if modal_state.copied {
        "✓ Kopieret!"
    } else {
        "📋 Kopiér sessionskode"
    };

    let is_copy_focused = modal_focus.is_button_focused(START_SESSION_BTN_COPY);
    let is_cancel_focused = modal_focus.is_button_focused(START_SESSION_BTN_CANCEL);
    let is_start_focused = modal_focus.is_button_focused(START_SESSION_BTN_START);

    let copy_base_style = if modal_state.copied {
        primary_button_style
    } else {
        secondary_button_style
    };

    let copy_btn = button(text(copy_label).size(12))
        .style(focused_button_style(copy_base_style, is_copy_focused))
        .on_press(Message::CollabCopyTicket)
        .padding([6, 14]);

    let actions = row![
        copy_btn,
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused
            ))
            .on_press(Message::CloseCollabModal)
            .padding([6, 14]),
        button(text("🚀 Start Session").size(12))
            .style(focused_button_style(primary_button_style, is_start_focused))
            .on_press(Message::CollabStartSession)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let dialog_col = column![
        title_row,
        subtitle,
        preset_field,
        custom_url_field,
        remember_checkbox,
        token_box,
        actions,
    ]
    .spacing(12);

    let modal_card = container(dialog_col)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(560.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn view_join_session_modal<'a>(
    modal_state: &'a JoinSessionModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("🌐 Deltag i Live Session (Gæst)")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseCollabModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text("Indsæt den sessionskode du har modtaget fra sessionens vært:")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    let validation_feedback: Element<Message> = if let Some(ticket) = &modal_state.parsed_ticket {
        container(
            row![
                text("✓")
                    .size(14)
                    .color(iced::Color::from_rgb(0.12, 0.65, 0.35)),
                Space::new().width(6),
                text(format!(
                    "Gyldig kode (Server: {} | Rum: {})",
                    ticket.relay_url,
                    ticket.room_id.as_str()
                ))
                .size(12)
                .color(iced::Color::from_rgb(0.12, 0.65, 0.35)),
            ]
            .align_y(Alignment::Center),
        )
        .padding([4, 8])
        .into()
    } else if let Some(err) = &modal_state.error_message {
        container(
            row![
                text("⚠️").size(12),
                Space::new().width(6),
                text(err)
                    .size(12)
                    .color(iced::Color::from_rgb(0.85, 0.20, 0.20)),
            ]
            .align_y(Alignment::Center),
        )
        .padding([4, 8])
        .into()
    } else {
        container(
            text("Format: kant:v1:<base64-payload>")
                .size(11)
                .color(ThemeColors::TEXT_MUTED),
        )
        .padding([4, 8])
        .into()
    };

    let token_field = column![
        text("Sessionskode *")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "Indsæt sessionskode her (f.eks. kant:v1:...)",
            &modal_state.token_input
        )
        .id("collab_ticket_input")
        .style(modern_input_style)
        .on_input(Message::CollabJoinTokenChanged)
        .padding(8)
        .width(Length::Fill),
        validation_feedback,
    ]
    .spacing(4);

    let is_cancel_focused = modal_focus.is_focused(0);
    let is_join_focused = modal_focus.is_focused(1);

    let mut join_btn = button(text("🔗 Forbind").size(12))
        .style(focused_button_style(primary_button_style, is_join_focused))
        .padding([6, 16]);

    if modal_state.is_valid() {
        join_btn = join_btn.on_press(Message::CollabJoinSession);
    }

    let actions = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused
            ))
            .on_press(Message::CloseCollabModal)
            .padding([6, 14]),
        join_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let dialog_col = column![title_row, subtitle, token_field, actions,].spacing(14);

    let modal_card = container(dialog_col)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(540.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn view_guest_ended_modal<'a>(
    notice: &'a GuestEndedNoticeModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("⚠️ Live Session Afsluttet")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CollabGuestDismissEndedModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let notice_text = text(&notice.message).size(13).color(ThemeColors::SLATE_700);

    let explanation = text("Værten har afsluttet sessionen. Du har modellen i hukommelsen (RAM) og kan gemme en kopi som en ny lokal modelprojektfil før sessionen lukkes.")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    let is_close_focused = modal_focus.is_focused(0);
    let is_save_focused = modal_focus.is_focused(1);

    let actions = row![
        Space::new().width(Length::Fill),
        button(text("Luk uden at gemme").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_close_focused
            ))
            .on_press(Message::CollabGuestDismissEndedModal)
            .padding([6, 14]),
        button(text("💾 Gem som kopi...").size(12))
            .style(focused_button_style(primary_button_style, is_save_focused))
            .on_press(Message::SaveProjectAsDialog)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let dialog_col = column![title_row, notice_text, explanation, actions,].spacing(14);

    let modal_card = container(dialog_col)
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
