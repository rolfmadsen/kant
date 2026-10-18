use crate::features::git::RepoSyncStatus;
use crate::ui::app::Message;
use crate::ui::theme::ThemeColors;
use iced::widget::{button, row, text, Space};
use iced::{Alignment, Element};

pub fn view_git_status_badge<'a>(
    git_sync_status: &RepoSyncStatus,
    badge_text: &str,
) -> Element<'a, Message> {
    let (dot_color, bg_color) = match git_sync_status {
        RepoSyncStatus::Synced => (
            iced::Color::from_rgb(0.06, 0.72, 0.44), // emerald green
            iced::Color::from_rgb(0.92, 0.98, 0.94),
        ),
        RepoSyncStatus::PendingChanges => (
            iced::Color::from_rgb(0.92, 0.60, 0.05), // amber
            iced::Color::from_rgb(1.0, 0.98, 0.88),
        ),
        RepoSyncStatus::UnpublishedCommits(n) if *n > 0 => (
            iced::Color::from_rgb(0.20, 0.50, 0.95), // blue
            iced::Color::from_rgb(0.92, 0.95, 1.0),
        ),
        RepoSyncStatus::IncomingCommits(_) => (
            iced::Color::from_rgb(0.55, 0.35, 0.95), // purple
            iced::Color::from_rgb(0.95, 0.92, 1.0),
        ),
        RepoSyncStatus::Diverged { .. } => (
            iced::Color::from_rgb(0.92, 0.25, 0.25), // red
            iced::Color::from_rgb(1.0, 0.92, 0.92),
        ),
        _ => (
            iced::Color::from_rgb(0.55, 0.60, 0.68), // slate
            iced::Color::from_rgb(0.95, 0.95, 0.96),
        ),
    };

    let is_uninit = matches!(git_sync_status, RepoSyncStatus::Uninitialized);

    let content = row![
        text("●").size(10).color(dot_color),
        Space::new().width(4),
        text(badge_text.to_string())
            .size(11)
            .color(ThemeColors::SLATE_800),
    ]
    .align_y(Alignment::Center);

    let click_msg = if is_uninit {
        Message::OpenGitConnectionModal
    } else {
        Message::OpenPublishModal
    };

    button(content)
        .on_press(click_msg)
        .style(move |_theme, status| {
            let hover_bg = match status {
                button::Status::Hovered => iced::Color { a: 0.8, ..bg_color },
                _ => bg_color,
            };
            button::Style {
                background: Some(iced::Background::Color(hover_bg)),
                text_color: ThemeColors::SLATE_800,
                border: iced::Border {
                    color: dot_color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .padding([3, 10])
        .into()
}
