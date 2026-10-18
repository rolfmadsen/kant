use crate::design_system::tokens::ColorTokens;
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
            ColorTokens::SUCCESS_TEXT.into_color(), // emerald green
            ColorTokens::SUCCESS_BG.into_color(),
        ),
        RepoSyncStatus::PendingChanges => (
            ColorTokens::WARNING_AMBER.into_color(), // amber
            ColorTokens::WARNING_AMBER_LIGHT.into_color(),
        ),
        RepoSyncStatus::UnpublishedCommits(n) if *n > 0 => (
            ColorTokens::INFO_BLUE.into_color(), // blue
            ColorTokens::INFO_BG.into_color(),
        ),
        RepoSyncStatus::IncomingCommits(_) => (
            ColorTokens::PURPLE_TEXT.into_color(), // purple
            ColorTokens::PURPLE_BG.into_color(),
        ),
        RepoSyncStatus::Diverged { .. } => (
            ColorTokens::DANGER_TEXT.into_color(), // red
            ColorTokens::DANGER_BG.into_color(),
        ),
        _ => (
            ColorTokens::MUTED_BADGE_DOT.into_color(), // slate
            ColorTokens::MUTED_BADGE_BG.into_color(),
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
