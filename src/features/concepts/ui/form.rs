use crate::design_system::tokens::ColorTokens;
use crate::features::concepts::linter::DefinitionLinter;
use crate::features::concepts::ui::state::{
    ConceptEditorState, ConceptFormField, BTN_DOMAIN_BORROWED, BTN_DOMAIN_LOCAL,
    BTN_TOGGLE_SUPPLEMENTARY,
};
use crate::features::concepts::BelongsToDomain;
use crate::ui::app::Message;
use crate::ui::theme::{
    card_container_style, focused_button_style, modern_input_style, primary_button_style,
    secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn render_form_fields(state: &ConceptEditorState) -> Vec<Element<'_, Message>> {
    let mut elements = Vec::new();

    // 1. Primære felter (Foretrukken term, Emneområde)
    let term_input = column![
        text("Foretrukken dansk term *")
            .size(13)
            .color(ThemeColors::SLATE_800),
        text_input(
            "F.eks. Køretøj, Personbil, Myndighed...",
            &state.preferred_term
        )
        .id("preferred_term_input")
        .style(modern_input_style)
        .on_input(|v| Message::UpdateConceptField(ConceptFormField::PreferredTerm, v))
        .padding(8),
        text("Den officielle primære sproglige betegnelse (§18, §19)")
            .size(11)
            .color(ThemeColors::SLATE_500),
    ]
    .spacing(4)
    .width(Length::FillPortion(2));

    let is_local = state.belongs_to_domain == BelongsToDomain::Yes;
    let is_local_focused = state.focused_button == Some(BTN_DOMAIN_LOCAL);
    let is_borrowed_focused = state.focused_button == Some(BTN_DOMAIN_BORROWED);
    let mut domain_col = column![
        text("Domæne / Kontekst (Tilhører emneområde §26) *")
            .size(13)
            .color(ThemeColors::SLATE_800),
        row![
            button(text("Lokalt begreb (Ja)").size(11))
                .style(focused_button_style(
                    if is_local {
                        primary_button_style
                    } else {
                        secondary_button_style
                    },
                    is_local_focused,
                ))
                .on_press(Message::SetConceptDomain(BelongsToDomain::Yes))
                .padding([6, 12]),
            button(text("Indlånt begreb (Nej)").size(11))
                .style(focused_button_style(
                    if !is_local {
                        primary_button_style
                    } else {
                        secondary_button_style
                    },
                    is_borrowed_focused,
                ))
                .on_press(Message::SetConceptDomain(BelongsToDomain::No))
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        text("Lokale begreber defineres her (FDA Sand). Indlånte markeres med FDA Blå (§26)")
            .size(11)
            .color(ThemeColors::SLATE_500),
    ]
    .spacing(4)
    .width(Length::FillPortion(2));

    if !is_local {
        domain_col = domain_col.push(
            column![
                text("Kildemodel URI (valgfri)")
                    .size(12)
                    .color(ThemeColors::SLATE_700),
                text_input(
                    "F.eks. https://data.gov.dk/model/core/cpr",
                    &state.model_uri
                )
                .id("editor_model_uri")
                .style(modern_input_style)
                .on_input(|v| Message::UpdateConceptField(ConceptFormField::ModelUri, v))
                .padding(6),
                text("URI på den eksterne FDA-model, begrebet er defineret i")
                    .size(11)
                    .color(ThemeColors::SLATE_500),
            ]
            .spacing(3),
        );
    }

    let term_domain_row = row![term_input, domain_col].spacing(16);

    // 2. Definition og Aristoteles linter feedback
    let opt_syn = if state.accepted_term.trim().is_empty() {
        None
    } else {
        Some(state.accepted_term.as_str())
    };
    let lint_issues =
        DefinitionLinter::lint_text(&state.preferred_term, opt_syn, &state.definition);
    let aristotle = DefinitionLinter::analyze_aristotle(&state.definition);

    let mut definition_input = column![
        text("Definition * (Aristoteles' formel)")
            .size(13)
            .color(ThemeColors::SLATE_800),
        text_input(
            "Genus proximum + differentia specifica (hvad er det, og hvad adskiller det)...",
            &state.definition,
        )
        .id("editor_definition")
        .style(modern_input_style)
        .on_input(|v| Message::UpdateConceptField(ConceptFormField::Definition, v))
        .padding(10),
    ]
    .spacing(6);

    if !lint_issues.is_empty() {
        let mut issues_col = column![text("⚠️ FDA Vejledning (§20-§22):")
            .size(11)
            .color(ColorTokens::WARNING_TEXT.into_color())]
        .spacing(3);

        for issue in &lint_issues {
            issues_col = issues_col.push(
                text(format!("• {}", issue.message))
                    .size(11)
                    .color(ColorTokens::WARNING_TEXT.into_color()),
            );
        }

        let issues_box = container(issues_col)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(
                    ColorTokens::WARNING_BG.into_color(),
                )),
                border: iced::Border {
                    color: ColorTokens::WARNING_BORDER.into_color(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([6, 10])
            .width(Length::Fill);

        definition_input = definition_input.push(issues_box);
    } else if !state.definition.trim().is_empty() {
        if let (Some(genus), Some(diff)) =
            (&aristotle.genus_proximum, &aristotle.differentia_specifica)
        {
            let ok_box = container(
                row![
                    text("✓").size(12).color(ThemeColors::ACCENT_GREEN),
                    Space::new().width(4),
                    text(format!(
                        "Aristoteles: Overbegreb = '{}' • Adskillende træk = '{}'",
                        genus, diff
                    ))
                    .size(11)
                    .color(ThemeColors::ACCENT_GREEN),
                ]
                .align_y(Alignment::Center),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::ACCENT_GREEN_LIGHT)),
                border: iced::Border {
                    color: ColorTokens::SUCCESS_BORDER.into_color(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding([5, 8])
            .width(Length::Fill);

            definition_input = definition_input.push(ok_box);
        } else {
            definition_input = definition_input.push(
                text("Tip: Angiv nærmeste overbegreb og adskillende træk (f.eks. 'køretøj der drives frem ved pedalkraft').")
                    .size(11)
                    .color(ThemeColors::SLATE_500),
            );
        }
    } else {
        definition_input = definition_input.push(
            text("Formuleret iht. Aristoteles' formel. Undgå cirkulære eller negative definitioner (§20-§22)")
                .size(11)
                .color(ThemeColors::SLATE_500),
        );
    }

    // 3. Kilder
    let legal_source_input = column![
        text("Juridisk kilde (Lovhjemmel)")
            .size(13)
            .color(ThemeColors::SLATE_800),
        text_input("F.eks. LBK nr 1324 af 21/11/2023 § 2", &state.legal_source)
            .id("editor_legal_source")
            .style(modern_input_style)
            .on_input(|v| Message::UpdateConceptField(ConceptFormField::LegalSource, v))
            .padding(8),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let general_source_input = column![
        text("Kilde").size(13).color(ThemeColors::SLATE_800),
        text_input(
            "F.eks. Dansk Standard, ISO 10241, fagordbog...",
            &state.source
        )
        .id("editor_source")
        .style(modern_input_style)
        .on_input(|v| Message::UpdateConceptField(ConceptFormField::Source, v))
        .padding(8),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let sources_row = row![legal_source_input, general_source_input].spacing(16);

    let primary_card =
        container(column![term_domain_row, definition_input, sources_row].spacing(14))
            .style(card_container_style)
            .padding(16)
            .width(Length::Fill);

    elements.push(primary_card.into());

    // Foldbar sektion med supplerende felter
    let is_supp_focused = state.focused_button == Some(BTN_TOGGLE_SUPPLEMENTARY);
    let toggle_supplementary_btn = button(
        text(if state.show_supplementary {
            "▲ Skjul supplerende felter"
        } else {
            "▼ Vis supplerende felter (Accepteret term, Eksempel, Identifikator m.fl.)"
        })
        .size(13),
    )
    .style(focused_button_style(
        secondary_button_style,
        is_supp_focused,
    ))
    .on_press(Message::ToggleShowAllFields)
    .padding([7, 14]);

    elements.push(toggle_supplementary_btn.into());

    if state.show_supplementary {
        let supplementary_content = column![
            row![
                column![
                    text("Accepteret dansk term")
                        .size(13)
                        .color(ThemeColors::SLATE_700),
                    text_input("Synonym eller tilladt betegnelse...", &state.accepted_term)
                        .id("editor_accepted_term")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(ConceptFormField::AcceptedTerm, v))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
                column![
                    text("Frarådet dansk term")
                        .size(13)
                        .color(ThemeColors::SLATE_700),
                    text_input("Betegnelse der ikke bør anvendes...", &state.deprecated_term)
                        .id("editor_deprecated_term")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(
                            ConceptFormField::DeprecatedTerm,
                            v
                        ))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            row![
                column![
                    text("Eksempel").size(13).color(ThemeColors::SLATE_700),
                    text_input("Typisk tilfælde der illustrerer begrebet...", &state.example)
                        .id("editor_example")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(ConceptFormField::Example, v))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
                column![
                    text("Kommentar").size(13).color(ThemeColors::SLATE_700),
                    text_input("Supplerende bemærkning...", &state.comment)
                        .id("editor_comment")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(ConceptFormField::Comment, v))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            column![
                text("Anvendelsesnote")
                    .size(13)
                    .color(ThemeColors::SLATE_700),
                text_input(
                    "Note om specifik anvendelseskontekst...",
                    &state.application_note
                )
                .id("editor_application_note")
                .style(modern_input_style)
                .on_input(|v| Message::UpdateConceptField(ConceptFormField::ApplicationNote, v))
                .padding(8),
            ]
            .spacing(4),
            row![
                column![
                    text("Identifikator (HTTP-URI)")
                        .size(13)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "https://data.gov.dk/model/core/domain/Term",
                        &state.identifier
                    )
                    .id("editor_identifier")
                    .style(modern_input_style)
                    .on_input(|v| Message::UpdateConceptField(ConceptFormField::Identifier, v))
                    .padding(8),
                    text("Begrebets egen persistente URI i begrebskataloget (Bilag D)")
                        .size(11)
                        .color(ThemeColors::SLATE_500),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
                column![
                    text("Afledt af").size(13).color(ThemeColors::SLATE_700),
                    text_input("HTTP-URI på oprindeligt begreb...", &state.derived_from)
                        .id("editor_derived_from")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(ConceptFormField::DerivedFrom, v))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            // Engelsk begrebslag (DA+EN jf. FDA skabelon)
            column![
                text("🇬🇧 Engelsk begrebslag (DA+EN jf. FDA skabelon)")
                    .size(13)
                    .color(ThemeColors::PRIMARY),
                text("Valgfrie felter til international interoperabilitet og genbrug (Begrebsliste DA+EN)")
                    .size(11)
                    .color(ThemeColors::SLATE_500),
            ]
            .spacing(2),
            row![
                column![
                    text("Foretrukken term (en)").size(13).color(ThemeColors::SLATE_700),
                    text_input("prefLabel (en)...", &state.english_preferred_term)
                        .id("editor_en_preferred_term")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(
                            ConceptFormField::EnglishPreferredTerm,
                            v
                        ))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
                column![
                    text("Accepteret term (en)").size(13).color(ThemeColors::SLATE_700),
                    text_input("altLabel (en)...", &state.english_accepted_term)
                        .id("editor_en_accepted_term")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(
                            ConceptFormField::EnglishAcceptedTerm,
                            v
                        ))
                        .padding(8),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            column![
                text("Definition (en)").size(13).color(ThemeColors::SLATE_700),
                text_input("definition (en)...", &state.english_definition)
                    .id("editor_en_definition")
                    .style(modern_input_style)
                    .on_input(|v| Message::UpdateConceptField(
                        ConceptFormField::EnglishDefinition,
                        v
                    ))
                    .padding(8),
            ]
            .spacing(4),
        ]
        .spacing(14);

        let supp_card = container(supplementary_content)
            .style(card_container_style)
            .padding(16)
            .width(Length::Fill);

        elements.push(supp_card.into());
    }

    elements
}
