use crate::design_system::tokens::ColorTokens;
use crate::features::git::ui::state::{
    GitCloneModalState, GitConnectionModalState, PublishModalState,
};
use crate::features::git::{ChangeAction, RepoSyncStatus};
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub const GIT_CONN_BTN_BROWSE: usize = 0;
pub const GIT_CONN_BTN_REMOVE: usize = 1;
pub const GIT_CONN_BTN_CANCEL: usize = 2;
pub const GIT_CONN_BTN_SAVE: usize = 3;

pub const GIT_CLONE_BTN_BROWSE: usize = 0;
pub const GIT_CLONE_BTN_CANCEL: usize = 1;
pub const GIT_CLONE_BTN_CLONE: usize = 2;

pub fn view_publish_modal<'a>(
    modal: &'a PublishModalState,
    git_sync_status: &RepoSyncStatus,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let close_btn = if modal.is_publishing {
        button(text("✕").size(13))
            .style(secondary_button_style)
            .padding([3, 7])
    } else {
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::ClosePublishModal)
            .padding([3, 7])
    };

    let title_row = row![
        text("🚀 Commit og send ændringer (pull, add & push)")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        close_btn,
    ]
    .align_y(Alignment::Center);

    let unpushed_count = match git_sync_status {
        RepoSyncStatus::UnpublishedCommits(n) => *n,
        _ => 0,
    };

    let is_clean_with_unpushed = modal.preview_events.is_empty() && unpushed_count > 0;
    let is_completely_synced = modal.preview_events.is_empty() && unpushed_count == 0;

    let subtitle_text = if is_clean_with_unpushed {
        format!(
            "Du har {} modelændring{} committet lokalt, som er klar til at blive sendt til serveren (push).",
            unpushed_count,
            if unpushed_count == 1 { "" } else { "er" }
        )
    } else {
        "Gemmer dine modelændringer lokalt (commit) og synkroniserer sikkert med serveren (pull, add & push).".to_string()
    };
    let subtitle = text(subtitle_text).size(12).color(ThemeColors::TEXT_MUTED);

    let mut changes_list = column![].spacing(6);
    if modal.preview_events.is_empty() {
        if is_clean_with_unpushed {
            changes_list = changes_list.push(
                text(format!(
                    "✓ Alle lokale ændringer er allerede committet. Klik 'Send til fjernlager' for at synkronisere {} version{} med serveren.",
                    unpushed_count,
                    if unpushed_count == 1 { "" } else { "er" }
                ))
                .size(12)
                .color(ThemeColors::SLATE_700),
            );
        } else {
            changes_list = changes_list.push(
                text("Der er ingen udestående ændringer i modellen.")
                    .size(12)
                    .color(ThemeColors::SLATE_500),
            );
        }
    } else {
        for event in &modal.preview_events {
            let (badge, color) = match event.action {
                ChangeAction::Added => ("🟢 Oprettet", ColorTokens::SUCCESS_TEXT.into_color()),
                ChangeAction::Modified => ("🟡 Opdateret", ColorTokens::WARNING_AMBER.into_color()),
                ChangeAction::Deleted => ("🔴 Fjernet", ColorTokens::DANGER_TEXT.into_color()),
            };
            let event_row = row![
                text(badge).size(11).color(color),
                Space::new().width(6),
                text(format!("{}: {}", event.entity_type, event.entity_name))
                    .size(12)
                    .color(ThemeColors::SLATE_800),
            ]
            .align_y(Alignment::Center);
            changes_list = changes_list.push(event_row);
        }
    }

    let changes_box = container(changes_list)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ColorTokens::CANVAS_BACKGROUND.into_color())),
            border: iced::Border {
                color: ThemeColors::SURFACE_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill);

    let maybe_message_field = if is_clean_with_unpushed || is_completely_synced {
        None
    } else {
        Some(
            column![
                text("Versionsnote / Commit-besked")
                    .size(12)
                    .color(ThemeColors::SLATE_700),
                text_input(
                    "Beskriv ændringerne i modellen (fx Tilføjet Virksomhed og CVR)...",
                    &modal.message
                )
                .id("publish_message_input")
                .style(modern_input_style)
                .on_input(Message::UpdatePublishMessage)
                .padding(8)
                .width(Length::Fill),
            ]
            .spacing(4),
        )
    };

    let primary_button_label = if is_clean_with_unpushed {
        "Send til server (Push)"
    } else {
        "Commit og send ændringer"
    };

    let footer_buttons = if modal.is_publishing {
        row![
            text("⏳ Synkroniserer med Git...")
                .size(12)
                .color(ThemeColors::PRIMARY),
            Space::new().width(Length::Fill),
            button(text("Udgiver..."))
                .style(primary_button_style)
                .padding([6, 14]),
        ]
        .align_y(Alignment::Center)
    } else if is_completely_synced {
        let is_close_focused = modal_focus.is_focused(0);
        row![
            Space::new().width(Length::Fill),
            button(text("Luk"))
                .style(focused_button_style(primary_button_style, is_close_focused))
                .on_press(Message::ClosePublishModal),
        ]
        .align_y(Alignment::Center)
    } else {
        let is_cancel_focused = modal_focus.is_focused(0);
        let is_confirm_focused = modal_focus.is_focused(1);
        row![
            Space::new().width(Length::Fill),
            button(text("Annuller"))
                .style(focused_button_style(
                    secondary_button_style,
                    is_cancel_focused
                ))
                .on_press(Message::ClosePublishModal),
            Space::new().width(8),
            button(text(primary_button_label))
                .style(focused_button_style(
                    primary_button_style,
                    is_confirm_focused
                ))
                .on_press(Message::ConfirmPublish),
        ]
        .align_y(Alignment::Center)
    };

    let mut dialog_content = column![
        title_row,
        subtitle,
        Space::new().height(8),
        text("Oversigt over ændringer:")
            .size(12)
            .color(ThemeColors::SLATE_700),
        changes_box,
        Space::new().height(8),
    ];

    if let Some(msg_col) = maybe_message_field {
        dialog_content = dialog_content.push(msg_col);
        dialog_content = dialog_content.push(Space::new().height(12));
    }

    if let Some(err) = &modal.error {
        let err_box = container(
            column![
                text("⚠️ Fejl ved overførsel til fjernlager:")
                    .size(12)
                    .color(ColorTokens::DANGER_TEXT.into_color()),
                text(err)
                    .size(11)
                    .color(ColorTokens::DANGER_TEXT.into_color()),
            ]
            .spacing(2),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ColorTokens::DANGER_BG.into_color())),
            border: iced::Border {
                color: ColorTokens::DANGER_BORDER.into_color(),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill);
        dialog_content = dialog_content.push(err_box);
        dialog_content = dialog_content.push(Space::new().height(8));
    }

    dialog_content = dialog_content.push(footer_buttons).spacing(8).width(480);

    let dialog_card = container(dialog_content)
        .style(modal_card_style)
        .padding(20);

    container(dialog_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn view_git_connection_modal<'a>(
    modal: &'a GitConnectionModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("⚙️ Git indstillinger")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseGitConnectionModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text("Konfigurer Git-serverens adresse (URL), din adgangsnøgle (token) samt din forfatterprofil til versionsstyring.")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    let remote_field = column![
        text("Fjernlager URL (Remote Origin)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "fx https://git.organisation.dk/model-arkiv.git (eller GitHub/GitLab)",
            &modal.remote_url
        )
        .id("git_remote_url")
        .style(modern_input_style)
        .on_input(Message::UpdateGitRemoteUrl)
        .padding([8, 12]),
    ]
    .spacing(4);

    let is_browse_focused = modal_focus.is_focused(GIT_CONN_BTN_BROWSE);
    let dir_field = column![
        text("Lokal Målmappe (Hvor modellen placeres på din computer)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        row![
            text_input("/sti/til/lokal/modelmappe", &modal.target_dir)
                .id("git_target_dir")
                .style(modern_input_style)
                .on_input(Message::UpdateGitTargetDir)
                .padding([8, 12])
                .width(Length::Fill),
            button(text("📂 Gennemse...").size(12))
                .style(focused_button_style(
                    secondary_button_style,
                    is_browse_focused
                ))
                .on_press(Message::BrowseGitTargetDir)
                .padding([8, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    let token_field = column![
        text("Personal Access Token (valgfrit ved private HTTPS-arkiver)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "Indtast Personal Access Token (f.eks. glpat_..., ghp_... eller token)",
            &modal.token
        )
        .id("git_token")
        .secure(true)
        .style(modern_input_style)
        .on_input(Message::UpdateGitToken)
        .padding([8, 12]),
    ]
    .spacing(4);

    let author_row = row![
        column![
            text("Git Brugernavn (user.name)")
                .size(12)
                .color(ThemeColors::SLATE_700),
            text_input("f.eks. Anders Hansen", &modal.author_name)
                .id("git_author_name")
                .style(modern_input_style)
                .on_input(Message::UpdateGitAuthorName)
                .padding([8, 12]),
        ]
        .spacing(4)
        .width(Length::FillPortion(1)),
        column![
            text("Git E-mail (user.email)")
                .size(12)
                .color(ThemeColors::SLATE_700),
            text_input("f.eks. anders@organisation.dk", &modal.author_email)
                .id("git_author_email")
                .style(modern_input_style)
                .on_input(Message::UpdateGitAuthorEmail)
                .padding([8, 12]),
        ]
        .spacing(4)
        .width(Length::FillPortion(1)),
    ]
    .spacing(12);

    let author_hint = text("💡 user.name og user.email er forfatter-signaturen i historikken. Tokenet gemmes sikkert til HTTPS-godkendelse (push/pull).")
        .size(11)
        .color(ThemeColors::TEXT_MUTED);

    let mut body = column![
        title_row,
        subtitle,
        Space::new().height(8),
        remote_field,
        dir_field,
        token_field,
        author_row,
        author_hint,
    ]
    .spacing(10);

    if let Some(err) = &modal.error_message {
        let err_box = container(
            text(err)
                .size(12)
                .color(ColorTokens::DANGER_TEXT.into_color()),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ColorTokens::DANGER_BG.into_color())),
            border: iced::Border {
                color: ColorTokens::DANGER_BORDER.into_color(),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(8)
        .width(Length::Fill);
        body = body.push(err_box);
    }

    let mut actions = row![].spacing(8).align_y(Alignment::Center);

    if !modal.remote_url.is_empty() {
        let is_remove_focused = modal_focus.is_focused(GIT_CONN_BTN_REMOVE);
        actions = actions.push(
            button(text("Fjern fjernlager").size(12))
                .style(move |_theme, status| {
                    let base_border_color = if is_remove_focused {
                        ThemeColors::FOCUS_RING
                    } else {
                        ColorTokens::DANGER_TEXT.into_color()
                    };
                    let base_border_width = if is_remove_focused { 1.5 } else { 1.0 };
                    button::Style {
                        background: match status {
                            button::Status::Hovered => Some(iced::Background::Color(
                                ColorTokens::DANGER_BG.into_color(),
                            )),
                            _ => None,
                        },
                        text_color: ColorTokens::DANGER_TEXT.into_color(),
                        border: iced::Border {
                            color: base_border_color,
                            width: base_border_width,
                            radius: 6.0.into(),
                        },
                        shadow: if is_remove_focused {
                            iced::Shadow {
                                color: ColorTokens::FOCUS_RING.with_alpha(0.40).into_color(),
                                offset: iced::Vector::new(0.0, 0.0),
                                blur_radius: 4.0,
                            }
                        } else {
                            iced::Shadow::default()
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::RemoveGitRemote)
                .padding([6, 12]),
        );
    }

    let is_cancel_focused = modal_focus.is_focused(GIT_CONN_BTN_CANCEL);
    let is_save_focused = modal_focus.is_focused(GIT_CONN_BTN_SAVE);

    actions = actions.push(Space::new().width(Length::Fill));
    actions = actions.push(
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused,
            ))
            .on_press(Message::CloseGitConnectionModal)
            .padding([6, 14]),
    );
    let is_empty_target = modal.target_dir.is_empty()
        || !std::path::Path::new(&modal.target_dir).exists()
        || std::path::Path::new(&modal.target_dir)
            .read_dir()
            .map_or(true, |mut i| i.next().is_none());
    let save_label = if is_empty_target && !modal.remote_url.is_empty() {
        "Hent & Forbind model"
    } else {
        "Gem indstillinger"
    };
    actions = actions.push(
        button(text(save_label).size(12))
            .style(focused_button_style(primary_button_style, is_save_focused))
            .on_press(Message::SaveGitConnection)
            .padding([6, 16]),
    );

    body = body.push(Space::new().height(8));
    body = body.push(actions);

    let modal_card = container(body)
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

pub fn view_git_clone_modal<'a>(
    modal: &'a GitCloneModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("📦 Klon Model fra Git Lager")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseGitCloneModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text(
        "Hent en eksisterende fælles FDA-model ned til din computer fra et centralt repository.",
    )
    .size(12)
    .color(ThemeColors::TEXT_MUTED);

    let url_field = column![
        text("Git Lager URL (Clone URL)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "fx https://git.organisation.dk/model-arkiv.git (eller GitHub/GitLab)",
            &modal.remote_url
        )
        .id("git_clone_url")
        .style(modern_input_style)
        .on_input(Message::UpdateGitCloneUrl)
        .padding([8, 12]),
    ]
    .spacing(4);

    let is_browse_focused = modal_focus.is_focused(GIT_CLONE_BTN_BROWSE);
    let dir_field = column![
        text("Lokal Målmappe (Hvor modellen skal placeres)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        row![
            text_input("/sti/til/lokal/modelmappe", &modal.target_dir)
                .id("git_clone_target_dir")
                .style(modern_input_style)
                .on_input(Message::UpdateGitCloneTargetDir)
                .padding([8, 12])
                .width(Length::Fill),
            button(text("📂 Gennemse...").size(12))
                .style(focused_button_style(
                    secondary_button_style,
                    is_browse_focused
                ))
                .on_press(Message::BrowseGitCloneTargetDir)
                .padding([8, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    let token_field = column![
        text("Access Token (valgfrit - nødvendigt for private Git lagre)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input("Personlig Access Token (PAT) eller kodeord", &modal.token)
            .id("git_clone_token")
            .secure(true)
            .style(modern_input_style)
            .on_input(Message::UpdateGitCloneToken)
            .padding([8, 12]),
    ]
    .spacing(4);

    let mut body = column![
        title_row,
        subtitle,
        Space::new().height(8),
        url_field,
        dir_field,
        token_field,
    ]
    .spacing(10);

    if let Some(err) = &modal.error_message {
        let err_box = container(
            text(err)
                .size(12)
                .color(ColorTokens::DANGER_TEXT.into_color()),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ColorTokens::DANGER_BG.into_color())),
            border: iced::Border {
                color: ColorTokens::DANGER_BORDER.into_color(),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(8)
        .width(Length::Fill);
        body = body.push(err_box);
    }

    let is_cancel_focused = modal_focus.is_focused(GIT_CLONE_BTN_CANCEL);
    let is_clone_focused = modal_focus.is_focused(GIT_CLONE_BTN_CLONE);

    let actions = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused
            ))
            .on_press(Message::CloseGitCloneModal)
            .padding([6, 14]),
        button(text("Klon og åbn model").size(12))
            .style(focused_button_style(primary_button_style, is_clone_focused))
            .on_press(Message::ExecuteGitClone)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    body = body.push(Space::new().height(8));
    body = body.push(actions);

    let modal_card = container(body)
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
