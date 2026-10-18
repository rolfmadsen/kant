use crate::features::diagnostics::SystemDiagnostics;
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, primary_button_style,
    secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct DiagnosticsModalState {
    pub diagnostics: SystemDiagnostics,
    pub logs: Vec<String>,
}

impl DiagnosticsModalState {
    pub fn new() -> Self {
        Self {
            diagnostics: crate::features::diagnostics::get_system_diagnostics(),
            logs: crate::features::diagnostics::get_recent_logs(),
        }
    }
}

impl Default for DiagnosticsModalState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view_diagnostics_modal<'a>(
    modal: &'a DiagnosticsModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("🔍 System- og Grafikdiagnostik")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseDiagnosticsModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle =
        text("Platforminformation, aktiv rendering-motor og lokale loghændelser til fejlfinding.")
            .size(12)
            .color(ThemeColors::TEXT_MUTED);

    let info_item = |label: &'static str, val: &str| {
        row![
            text(label)
                .size(12)
                .color(ThemeColors::SLATE_600)
                .width(Length::Fixed(140.0)),
            text(val.to_string()).size(12).color(ThemeColors::SLATE_900),
        ]
        .align_y(Alignment::Center)
    };

    let log_path_str = modal
        .diagnostics
        .log_path
        .as_deref()
        .unwrap_or("(Ingen skriveadgang / in-memory)");

    let startup_time_str = modal
        .diagnostics
        .startup_milestones
        .last()
        .map(|m| format!("{}ms ({})", m.elapsed_ms, m.name))
        .unwrap_or_else(|| "N/A".to_string());

    let system_info_card = container(
        column![
            info_item("Operativsystem:", &modal.diagnostics.os),
            info_item("Arkitektur:", &modal.diagnostics.arch),
            info_item("Kant Version:", &modal.diagnostics.app_version),
            info_item("Iced Backend:", &modal.diagnostics.iced_backend),
            info_item("WGPU Backend:", &modal.diagnostics.wgpu_backend),
            info_item("Opstartstid:", &startup_time_str),
            info_item("Logfil Sti:", log_path_str),
        ]
        .spacing(6),
    )
    .style(|_| container::Style {
        background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
        border: iced::Border {
            color: ThemeColors::SLATE_200,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .padding(12)
    .width(Length::Fill);

    let logs_header = text("Seneste loghændelser (In-memory buffer):")
        .size(12)
        .color(ThemeColors::SLATE_700);

    let mut log_column = column![].spacing(3);
    if modal.logs.is_empty() {
        log_column = log_column.push(
            text("Ingen logbeskeder registreret endnu.")
                .size(11)
                .color(ThemeColors::SLATE_500),
        );
    } else {
        for entry in modal.logs.iter().rev().take(30) {
            log_column =
                log_column.push(text(entry.clone()).size(11).color(ThemeColors::SLATE_800));
        }
    }

    let logs_container = container(scrollable(log_column).height(Length::Fixed(180.0)))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::WHITE)),
            border: iced::Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill);

    let is_copy_focused = modal_focus.is_focused(0);
    let is_close_focused = modal_focus.is_focused(1);

    let actions = row![
        button(text("📋 Kopier rapport til udklipsholder").size(12))
            .style(focused_button_style(primary_button_style, is_copy_focused))
            .on_press(Message::CopyDiagnosticsToClipboard)
            .padding([6, 14]),
        Space::new().width(Length::Fill),
        button(text("Luk").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_close_focused,
            ))
            .on_press(Message::CloseDiagnosticsModal)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let body = column![
        title_row,
        subtitle,
        Space::new().height(4),
        system_info_card,
        Space::new().height(4),
        logs_header,
        logs_container,
        Space::new().height(8),
        actions,
    ]
    .spacing(8);

    let modal_card = container(body)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(620.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
