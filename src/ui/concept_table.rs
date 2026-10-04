use crate::features::concepts::{BelongsToDomain, Concept, DefinitionLinter};
use crate::ui::app::{
    Message, TOOLBAR_BTN_EXPORT_EXCEL, TOOLBAR_BTN_GUIDE, TOOLBAR_BTN_IMPORT_EXCEL,
    TOOLBAR_BTN_NEW_CONCEPT, TOOLBAR_BTN_TEMPLATE, TOOLBAR_ROW_BTN_START,
};
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, modern_input_style,
    pill_container_style, primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

pub fn view<'a>(
    concepts: Vec<&'a Concept>,
    total_count: usize,
    search_query: &'a str,
    focused_toolbar_button: Option<usize>,
) -> Element<'a, Message> {
    // 1. Top bar: Søgning, statistik og "Opret"-knap
    let search_bar = text_input(
        "🔍 Søg i foretrukken term, definition eller kilder...",
        search_query,
    )
    .id("concept_search_input")
    .style(modern_input_style)
    .on_input(Message::SearchQueryChanged)
    .padding(9)
    .width(Length::FillPortion(3));

    let count_badge = container(
        row![
            text("Begreber:").size(12).color(ThemeColors::SLATE_500),
            Space::new().width(4),
            text(format!("{}/{}", concepts.len(), total_count))
                .size(12)
                .color(ThemeColors::SLATE_800),
        ]
        .align_y(Alignment::Center),
    )
    .style(pill_container_style)
    .padding([6, 12])
    .align_y(Alignment::Center);

    let is_create_focused = focused_toolbar_button == Some(TOOLBAR_BTN_NEW_CONCEPT);
    let is_import_focused = focused_toolbar_button == Some(TOOLBAR_BTN_IMPORT_EXCEL);
    let is_export_focused = focused_toolbar_button == Some(TOOLBAR_BTN_EXPORT_EXCEL);
    let is_template_focused = focused_toolbar_button == Some(TOOLBAR_BTN_TEMPLATE);
    let is_guide_focused = focused_toolbar_button == Some(TOOLBAR_BTN_GUIDE);

    let create_btn = button(
        row![
            text("+").size(15),
            Space::new().width(6),
            text("Nyt Begreb").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .style(focused_button_style(
        primary_button_style,
        is_create_focused,
    ))
    .on_press(Message::StartNewConcept)
    .padding([8, 16]);

    let import_btn = button(
        row![
            text("📥").size(13),
            Space::new().width(6),
            text("Importér Excel").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .style(focused_button_style(
        secondary_button_style,
        is_import_focused,
    ))
    .on_press(Message::ImportConceptsExcelDialog)
    .padding([8, 14]);

    let export_btn = button(
        row![
            text("📊").size(13),
            Space::new().width(6),
            text("Eksportér Excel").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .style(focused_button_style(
        secondary_button_style,
        is_export_focused,
    ))
    .on_press(Message::ExportConceptListXlsxDialog)
    .padding([8, 14]);

    let template_btn = button(
        row![
            text("📄").size(13),
            Space::new().width(6),
            text("Hent skabelon").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .style(focused_button_style(
        secondary_button_style,
        is_template_focused,
    ))
    .on_press(Message::ExportFdaTemplateXlsxDialog)
    .padding([8, 14]);

    let guide_btn = button(
        row![
            text("ℹ️").size(13),
            Space::new().width(6),
            text("Skemaguide").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .style(focused_button_style(
        secondary_button_style,
        is_guide_focused,
    ))
    .on_press(Message::OpenExcelSchemaGuideModal)
    .padding([8, 14]);

    let top_bar = row![
        search_bar,
        count_badge,
        Space::new().width(4),
        create_btn,
        import_btn,
        export_btn,
        template_btn,
        guide_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // 2. Tabelhoved
    let header_row = row![
        container(
            text("Foretrukken term")
                .size(12)
                .color(ThemeColors::SLATE_600)
        )
        .width(Length::FillPortion(2)),
        container(text("Definition").size(12).color(ThemeColors::SLATE_600))
            .width(Length::FillPortion(4)),
        container(
            text("Kilder (Juridisk / Almen)")
                .size(12)
                .color(ThemeColors::SLATE_600)
        )
        .width(Length::FillPortion(2)),
        container(
            text("Emneområde (§26)")
                .size(12)
                .color(ThemeColors::SLATE_600)
        )
        .width(Length::FillPortion(2)),
        container(text("Handlinger").size(12).color(ThemeColors::SLATE_600))
            .width(Length::FillPortion(2)),
    ]
    .spacing(12)
    .padding([10, 14]);

    let header_container = container(header_row)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::SLATE_100)),
            border: iced::Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill);

    // 3. Rækker eller tom tilstand
    let content: Element<Message> = if concepts.is_empty() {
        let empty_msg = if !search_query.trim().is_empty() {
            "Ingen begreber matcher din søgning."
        } else {
            "Der er endnu ingen begreber i projektets begrebsliste."
        };

        container(
            column![
                text("📋").size(44),
                Space::new().height(8),
                text(empty_msg).size(16).color(ThemeColors::SLATE_800),
                Space::new().height(4),
                text("Opret dit første begreb med foretrukken term, struktureret definition, kilder og emneområde jf. FDA Modelreglerne.")
                    .size(13)
                    .color(ThemeColors::TEXT_MUTED),
                Space::new().height(16),
                row![
                    button(text("+ Opret Første Begreb").size(13))
                        .style(primary_button_style)
                        .on_press(Message::StartNewConcept)
                        .padding([8, 18]),
                    button(
                        row![
                            text("📥").size(13),
                            Space::new().width(6),
                            text("Importér fra Excel").size(13),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .style(secondary_button_style)
                    .on_press(Message::ImportConceptsExcelDialog)
                    .padding([8, 16]),
                    button(
                        row![
                            text("📄").size(13),
                            Space::new().width(6),
                            text("Hent officiel skabelon").size(13),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .style(secondary_button_style)
                    .on_press(Message::ExportFdaTemplateXlsxDialog)
                    .padding([8, 16]),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .style(card_container_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(32)
        .into()
    } else {
        let mut rows = column![].spacing(8);

        for (i, concept) in concepts.iter().enumerate() {
            let id = concept.id();
            let edit_btn_idx = TOOLBAR_ROW_BTN_START + i * 2;
            let delete_btn_idx = TOOLBAR_ROW_BTN_START + i * 2 + 1;
            let is_edit_focused = focused_toolbar_button == Some(edit_btn_idx);
            let is_delete_focused = focused_toolbar_button == Some(delete_btn_idx);

            // Foretrukken term kolonne
            let mut term_content = column![text(concept.preferred_term())
                .size(14)
                .color(ThemeColors::SLATE_900)]
            .spacing(2);
            if let Some(en_term) = concept.english_preferred_term() {
                term_content = term_content.push(
                    row![
                        text("🇬🇧").size(10),
                        Space::new().width(4),
                        text(en_term).size(12).color(ThemeColors::SLATE_600),
                    ]
                    .align_y(Alignment::Center),
                );
            }
            if let Some(uri) = concept.identifier() {
                term_content = term_content.push(text(uri).size(11).color(ThemeColors::TEXT_MUTED));
            }

            // Definition kolonne med eventuelle FDA linter badges
            let lint_issues = DefinitionLinter::lint(concept);
            let mut def_col = column![text(concept.definition())
                .size(13)
                .color(ThemeColors::SLATE_800),]
            .spacing(4);

            if !lint_issues.is_empty() {
                def_col = def_col.push(
                    container(
                        text(format!("⚠️ {} FDA-bemærkning(er)", lint_issues.len()))
                            .size(10)
                            .color(Color::from_rgb(0.75, 0.45, 0.05)),
                    )
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(
                            0.99, 0.97, 0.90,
                        ))),
                        border: iced::Border {
                            color: Color::from_rgb(0.92, 0.80, 0.50),
                            width: 0.5,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([2, 6]),
                );
            }

            let def_content = def_col;

            // Kilde kolonne
            let mut sources_content = column![].spacing(3);
            if let Some(legal) = concept.legal_source() {
                sources_content = sources_content.push(
                    container(
                        text(format!("§ {}", legal))
                            .size(11)
                            .color(ThemeColors::PRIMARY),
                    )
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::PRIMARY,
                            width: 0.5,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([2, 6]),
                );
            }
            if let Some(src) = concept.source() {
                sources_content =
                    sources_content.push(text(src).size(11).color(ThemeColors::SLATE_600));
            } else if concept.legal_source().is_none() {
                sources_content = sources_content.push(
                    text("Ingen kilde angivet")
                        .size(11)
                        .color(ThemeColors::TEXT_MUTED),
                );
            }

            // Emneområde kolonne - FDA Anbefalede Farver jf. Kapitel 7.3
            let domain_badge = match concept.belongs_to_domain() {
                BelongsToDomain::Yes => container(
                    text("🏷️ Lokalt begreb")
                        .size(11)
                        .color(ThemeColors::SLATE_800),
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::FDA_SAND)),
                    border: iced::Border {
                        color: ThemeColors::FDA_SAND_BORDER,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([4, 8]),
                BelongsToDomain::No => container(
                    text("🌐 Indlånt begreb")
                        .size(11)
                        .color(ThemeColors::PRIMARY),
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::FDA_BORROWED_BLUE_BG)),
                    border: iced::Border {
                        color: ThemeColors::FDA_BORROWED_BLUE,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([4, 8]),
                BelongsToDomain::ModelRef(model_uri) => container(
                    text(format!("🌐 {}", model_uri))
                        .size(11)
                        .color(ThemeColors::PRIMARY),
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::FDA_BORROWED_BLUE_BG)),
                    border: iced::Border {
                        color: ThemeColors::FDA_BORROWED_BLUE,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([4, 8]),
            };

            // Handlinger kolonne
            let actions = row![
                button(text("Rediger").size(12))
                    .style(focused_button_style(
                        secondary_button_style,
                        is_edit_focused,
                    ))
                    .on_press(Message::EditConcept(id))
                    .padding([5, 12]),
                button(text("Slet").size(12))
                    .style(focused_button_style(danger_button_style, is_delete_focused,))
                    .on_press(Message::DeleteConcept(id))
                    .padding([5, 10]),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            let row_card = container(
                row![
                    container(term_content).width(Length::FillPortion(2)),
                    container(def_content).width(Length::FillPortion(4)),
                    container(sources_content).width(Length::FillPortion(2)),
                    container(domain_badge).width(Length::FillPortion(2)),
                    container(actions).width(Length::FillPortion(2)),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .style(card_container_style)
            .padding([12, 16])
            .width(Length::Fill);

            rows = rows.push(row_card);
        }

        scrollable(rows).height(Length::Fill).into()
    };

    column![top_bar, Space::new().height(6), header_container, content]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
