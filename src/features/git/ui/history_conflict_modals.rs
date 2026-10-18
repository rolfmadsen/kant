use crate::design_system::tokens::ColorTokens;
use crate::features::git::ui::state::{
    ConflictChoice, ConflictResolverModalState, ModelHistoryModalState,
};
use crate::features::git::ChangeAction;
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, primary_button_style,
    secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

pub const HISTORY_BTN_COMMIT_NOW: usize = 11;
pub const HISTORY_BTN_CLOSE: usize = 0;
pub const HISTORY_COMMIT_CARD_START: usize = 20;

pub fn view_history_modal<'a>(
    modal: &'a ModelHistoryModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_text = if modal.target_element_id.is_some() {
        "🕒 Revisionshistorik for element"
    } else {
        "📜 Modelhistorik (Tidslinje)"
    };

    let title_row = row![
        text(title_text).size(17).color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseModelHistoryModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text("Gennemse historiske versioner og domænehændelser over tid.")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    let content_element: Element<'a, Message> = if modal.commits.is_empty() {
        let is_commit_now_focused = modal_focus.is_button_focused(HISTORY_BTN_COMMIT_NOW);
        let empty_card = container(
            column![
                text(if modal.target_element_id.is_some() {
                    "ℹ️ Ingen versioner fundet for dette element endnu"
                } else {
                    "ℹ️ Ingen udgivne versioner i historikken endnu"
                })
                .size(13)
                .color(ThemeColors::SLATE_900),
                Space::new().height(6),
                text(if modal.target_element_id.is_some() {
                    "Dette element er endnu ikke blevet udgivet som en del af en officiel modelversion."
                } else {
                    "Modellens tidslinje sporer udgivne versioner af begreber, informationsklasser og relationer.\n\nDine nuværende ændringer er gemt lokalt som kladde (se status-knappen 'Lokale ændringer' øverst). For at gemme din første version i historikken, skal du udgive modellen."
                })
                .size(12)
                .color(ThemeColors::SLATE_600),
                Space::new().height(12),
                button(text("🚀 Commit og send ændringer nu...").size(12))
                    .style(focused_button_style(primary_button_style, is_commit_now_focused))
                    .on_press(Message::OpenPublishModal)
                    .padding([6, 14]),
            ]
            .spacing(4),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ColorTokens::SURFACE_BG.into_color())),
            border: iced::Border {
                color: ThemeColors::SURFACE_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .padding(14)
        .width(Length::Fill);

        empty_card.into()
    } else {
        let mut commits_list = column![].spacing(10);
        for (idx, commit) in modal.commits.iter().enumerate() {
            let is_card_focused = modal_focus.is_button_focused(HISTORY_COMMIT_CARD_START + idx);
            let commit_header = row![
                text(&commit.message).size(13).color(ThemeColors::SLATE_900),
                Space::new().width(Length::Fill),
                text(&commit.author_name)
                    .size(11)
                    .color(ThemeColors::SLATE_500),
                Space::new().width(6),
                text(&commit.short_oid)
                    .size(10)
                    .color(ThemeColors::SLATE_400),
            ]
            .align_y(Alignment::Center);

            let mut events_col = column![].spacing(3);
            for ev in &commit.changes {
                let (badge, color) = match ev.action {
                    ChangeAction::Added => ("🟢 Oprettet", ColorTokens::SUCCESS_TEXT.into_color()),
                    ChangeAction::Modified => {
                        ("🟡 Opdateret", ColorTokens::WARNING_AMBER.into_color())
                    }
                    ChangeAction::Deleted => ("🔴 Fjernet", ColorTokens::DANGER_TEXT.into_color()),
                };
                events_col = events_col.push(
                    row![
                        text(badge).size(10).color(color),
                        Space::new().width(4),
                        text(format!("{}: {}", ev.entity_type, ev.entity_name))
                            .size(11)
                            .color(ThemeColors::SLATE_700),
                    ]
                    .align_y(Alignment::Center),
                );
            }

            let commit_card =
                container(column![commit_header, Space::new().height(4), events_col].spacing(4))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(
                            ColorTokens::CANVAS_BACKGROUND.into_color(),
                        )),
                        border: iced::Border {
                            color: if is_card_focused {
                                ThemeColors::FOCUS_RING
                            } else {
                                ThemeColors::SURFACE_BORDER
                            },
                            width: if is_card_focused { 1.5 } else { 1.0 },
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding(10)
                    .width(Length::Fill);

            commits_list = commits_list.push(commit_card);
        }

        scrollable(commits_list).height(Length::Fixed(360.0)).into()
    };

    let is_close_focused = modal_focus.is_button_focused(HISTORY_BTN_CLOSE);
    let footer = row![
        Space::new().width(Length::Fill),
        button(text("Luk"))
            .style(focused_button_style(
                secondary_button_style,
                is_close_focused
            ))
            .on_press(Message::CloseModelHistoryModal),
    ]
    .align_y(Alignment::Center);

    let dialog_content = column![
        title_row,
        subtitle,
        Space::new().height(8),
        content_element,
        Space::new().height(12),
        footer,
    ]
    .spacing(8)
    .width(520);

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

pub fn view_conflict_resolver_modal<'a>(
    modal: &'a ConflictResolverModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("⚠️ Modstridende Ændringer (Konflikthåndtering)")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseConflictResolverModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle =
        text("Der er opstået modstridende feltændringer. Vælg hvilken version der skal gælde:")
            .size(12)
            .color(ThemeColors::TEXT_MUTED);

    let mut conflicts_list = column![].spacing(12);
    for (idx, conflict) in modal.conflicts.iter().enumerate() {
        let choice = modal
            .resolved_choices
            .get(&idx)
            .copied()
            .unwrap_or(ConflictChoice::Ours);

        let header = text(format!(
            "Felt: {} i element '{}'",
            conflict.field_name, conflict.entity_name
        ))
        .size(13)
        .color(ThemeColors::SLATE_900);

        let base_info = text(format!("Oprindelig værdi: \"{}\"", conflict.base_value))
            .size(11)
            .color(ThemeColors::SLATE_500);

        let ours_selected = choice == ConflictChoice::Ours;
        let theirs_selected = choice == ConflictChoice::Theirs;

        let ours_btn = button(
            row![
                text(if ours_selected { "● " } else { "○ " })
                    .size(11)
                    .color(if ours_selected {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_400
                    }),
                text(format!("Lokal version: \"{}\"", conflict.our_value)).size(11),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::ResolveConflict {
            index: idx,
            choice: ConflictChoice::Ours,
        })
        .style(if ours_selected {
            primary_button_style
        } else {
            secondary_button_style
        })
        .padding([6, 12])
        .width(Length::Fill);

        let theirs_btn = button(
            row![
                text(if theirs_selected { "● " } else { "○ " })
                    .size(11)
                    .color(if theirs_selected {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_400
                    }),
                text(format!("Server version: \"{}\"", conflict.their_value)).size(11),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::ResolveConflict {
            index: idx,
            choice: ConflictChoice::Theirs,
        })
        .style(if theirs_selected {
            primary_button_style
        } else {
            secondary_button_style
        })
        .padding([6, 12])
        .width(Length::Fill);

        let card = container(
            column![
                header,
                base_info,
                Space::new().height(4),
                ours_btn,
                theirs_btn
            ]
            .spacing(4),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(
                ColorTokens::CANVAS_BACKGROUND.into_color(),
            )),
            border: iced::Border {
                color: ThemeColors::SURFACE_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .padding(10)
        .width(Length::Fill);

        conflicts_list = conflicts_list.push(card);
    }

    let is_cancel_focused = modal_focus.is_focused(0);
    let is_apply_focused = modal_focus.is_focused(1);

    let footer = row![
        Space::new().width(Length::Fill),
        button(text("Annuller"))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused
            ))
            .on_press(Message::CloseConflictResolverModal),
        Space::new().width(8),
        button(text("Anvend valgte versioner"))
            .style(focused_button_style(primary_button_style, is_apply_focused))
            .on_press(Message::ApplyResolvedConflicts),
    ]
    .align_y(Alignment::Center);

    let dialog_content = column![
        title_row,
        subtitle,
        Space::new().height(8),
        container(conflicts_list).max_height(400),
        Space::new().height(12),
        footer,
    ]
    .spacing(8)
    .width(540);

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
