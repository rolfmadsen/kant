use crate::features::concepts::{
    BelongsToDomain, Concept, ConceptValidator, DefinitionLinter, ValidationError,
};
use crate::ui::app::Message;
use crate::ui::theme::ThemeColors;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConceptFormField {
    PreferredTerm,
    Definition,
    BelongsToDomain,
    ModelUri,
    AcceptedTerm,
    DeprecatedTerm,
    Example,
    Comment,
    ApplicationNote,
    LegalSource,
    Source,
    Identifier,
    DerivedFrom,
    EnglishPreferredTerm,
    EnglishAcceptedTerm,
    EnglishDeprecatedTerm,
    EnglishDefinition,
    EnglishExample,
    EnglishComment,
    EnglishApplicationNote,
}

pub const BTN_CANCEL: usize = 0;
pub const BTN_SAVE: usize = 1;
pub const BTN_DOMAIN_LOCAL: usize = 2;
pub const BTN_DOMAIN_BORROWED: usize = 3;
pub const BTN_TOGGLE_SUPPLEMENTARY: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConceptEditorFocusTarget {
    Input(&'static str),
    Button(usize),
}

#[derive(Debug, Clone)]
pub struct ConceptEditorState {
    pub editing_id: Option<Uuid>,
    pub preferred_term: String,
    pub definition: String,
    pub belongs_to_domain: BelongsToDomain,
    pub model_uri: String,
    pub accepted_term: String,
    pub deprecated_term: String,
    pub example: String,
    pub comment: String,
    pub application_note: String,
    pub legal_source: String,
    pub source: String,
    pub identifier: String,
    pub derived_from: String,
    pub english_preferred_term: String,
    pub english_accepted_term: String,
    pub english_deprecated_term: String,
    pub english_definition: String,
    pub english_example: String,
    pub english_comment: String,
    pub english_application_note: String,
    pub show_supplementary: bool,
    pub validation_error: Option<String>,
    pub focused_button: Option<usize>,
    pub current_target_index: usize,
}

impl Default for ConceptEditorState {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl ConceptEditorState {
    pub fn new_empty() -> Self {
        Self {
            editing_id: None,
            preferred_term: String::new(),
            definition: String::new(),
            belongs_to_domain: BelongsToDomain::Yes,
            model_uri: String::new(),
            accepted_term: String::new(),
            deprecated_term: String::new(),
            example: String::new(),
            comment: String::new(),
            application_note: String::new(),
            legal_source: String::new(),
            source: String::new(),
            identifier: String::new(),
            derived_from: String::new(),
            english_preferred_term: String::new(),
            english_accepted_term: String::new(),
            english_deprecated_term: String::new(),
            english_definition: String::new(),
            english_example: String::new(),
            english_comment: String::new(),
            english_application_note: String::new(),
            show_supplementary: false,
            validation_error: None,
            focused_button: None,
            current_target_index: 0,
        }
    }

    pub fn from_concept(concept: &Concept) -> Self {
        let (belongs, model_uri) = match concept.belongs_to_domain() {
            BelongsToDomain::Yes => (BelongsToDomain::Yes, String::new()),
            BelongsToDomain::No => (BelongsToDomain::No, String::new()),
            BelongsToDomain::ModelRef(uri) => (BelongsToDomain::No, uri.clone()),
        };

        let (en_pref, en_acc, en_dep, en_def, en_ex, en_comm, en_app) = match concept.english() {
            Some(en) => (
                en.preferred_term.clone().unwrap_or_default(),
                en.accepted_term.clone().unwrap_or_default(),
                en.deprecated_term.clone().unwrap_or_default(),
                en.definition.clone().unwrap_or_default(),
                en.example.clone().unwrap_or_default(),
                en.comment.clone().unwrap_or_default(),
                en.application_note.clone().unwrap_or_default(),
            ),
            None => (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ),
        };

        Self {
            editing_id: Some(concept.id()),
            preferred_term: concept.preferred_term().to_string(),
            definition: concept.definition().to_string(),
            belongs_to_domain: belongs,
            model_uri,
            accepted_term: concept.accepted_term().unwrap_or_default().to_string(),
            deprecated_term: concept.deprecated_term().unwrap_or_default().to_string(),
            example: concept.example().unwrap_or_default().to_string(),
            comment: concept.comment().unwrap_or_default().to_string(),
            application_note: concept.application_note().unwrap_or_default().to_string(),
            legal_source: concept.legal_source().unwrap_or_default().to_string(),
            source: concept.source().unwrap_or_default().to_string(),
            identifier: concept.identifier().unwrap_or_default().to_string(),
            derived_from: concept.derived_from().unwrap_or_default().to_string(),
            english_preferred_term: en_pref,
            english_accepted_term: en_acc,
            english_deprecated_term: en_dep,
            english_definition: en_def,
            english_example: en_ex,
            english_comment: en_comm,
            english_application_note: en_app,
            show_supplementary: concept.accepted_term().is_some()
                || concept.deprecated_term().is_some()
                || concept.example().is_some()
                || concept.comment().is_some()
                || concept.application_note().is_some()
                || concept.identifier().is_some()
                || concept.derived_from().is_some()
                || concept.english().is_some(),
            validation_error: None,
            focused_button: None,
            current_target_index: 0,
        }
    }

    pub fn focused_button(&self) -> Option<usize> {
        self.focused_button
    }

    pub fn set_focused_button(&mut self, btn: Option<usize>) {
        self.focused_button = btn;
        if let Some(b) = btn {
            let targets = self.focus_targets();
            if let Some(pos) = targets.iter().position(|t| match t {
                ConceptEditorFocusTarget::Button(tb) => *tb == b,
                _ => false,
            }) {
                self.current_target_index = pos;
            }
        }
    }

    pub fn focus_targets(&self) -> Vec<ConceptEditorFocusTarget> {
        let mut targets = Vec::new();
        // 1. Foretrukken dansk term
        targets.push(ConceptEditorFocusTarget::Input("preferred_term_input"));
        // 2. Lokalt begreb (Ja)
        targets.push(ConceptEditorFocusTarget::Button(BTN_DOMAIN_LOCAL));
        // 3. Indlånt begreb (Nej)
        targets.push(ConceptEditorFocusTarget::Button(BTN_DOMAIN_BORROWED));

        if self.belongs_to_domain == BelongsToDomain::No {
            targets.push(ConceptEditorFocusTarget::Input("editor_model_uri"));
        }

        targets.push(ConceptEditorFocusTarget::Input("editor_definition"));
        targets.push(ConceptEditorFocusTarget::Input("editor_legal_source"));
        targets.push(ConceptEditorFocusTarget::Input("editor_source"));

        targets.push(ConceptEditorFocusTarget::Button(BTN_TOGGLE_SUPPLEMENTARY));

        if self.show_supplementary {
            targets.push(ConceptEditorFocusTarget::Input("editor_accepted_term"));
            targets.push(ConceptEditorFocusTarget::Input("editor_deprecated_term"));
            targets.push(ConceptEditorFocusTarget::Input("editor_example"));
            targets.push(ConceptEditorFocusTarget::Input("editor_comment"));
            targets.push(ConceptEditorFocusTarget::Input("editor_application_note"));
            targets.push(ConceptEditorFocusTarget::Input("editor_identifier"));
            targets.push(ConceptEditorFocusTarget::Input("editor_derived_from"));
            targets.push(ConceptEditorFocusTarget::Input("editor_en_preferred_term"));
            targets.push(ConceptEditorFocusTarget::Input("editor_en_accepted_term"));
            targets.push(ConceptEditorFocusTarget::Input("editor_en_definition"));
        }

        targets.push(ConceptEditorFocusTarget::Button(BTN_CANCEL));
        targets.push(ConceptEditorFocusTarget::Button(BTN_SAVE));
        targets
    }

    pub fn advance_focus(&mut self, reverse: bool) -> ConceptEditorFocusTarget {
        let targets = self.focus_targets();
        if targets.is_empty() {
            return ConceptEditorFocusTarget::Button(BTN_SAVE);
        }
        if reverse {
            if self.current_target_index == 0 {
                self.current_target_index = targets.len().saturating_sub(1);
            } else {
                self.current_target_index -= 1;
            }
        } else {
            if self.current_target_index + 1 >= targets.len() {
                self.current_target_index = 0;
            } else {
                self.current_target_index += 1;
            }
        }
        let target = targets[self.current_target_index].clone();
        match target {
            ConceptEditorFocusTarget::Button(btn) => {
                self.focused_button = Some(btn);
            }
            ConceptEditorFocusTarget::Input(_) => {
                self.focused_button = None;
            }
        }
        target
    }

    pub fn sync_focus_to_field(&mut self, field: ConceptFormField) {
        let expected_input = match field {
            ConceptFormField::PreferredTerm => "preferred_term_input",
            ConceptFormField::ModelUri => "editor_model_uri",
            ConceptFormField::Definition => "editor_definition",
            ConceptFormField::LegalSource => "editor_legal_source",
            ConceptFormField::Source => "editor_source",
            ConceptFormField::AcceptedTerm => "editor_accepted_term",
            ConceptFormField::DeprecatedTerm => "editor_deprecated_term",
            ConceptFormField::Example => "editor_example",
            ConceptFormField::Comment => "editor_comment",
            ConceptFormField::ApplicationNote => "editor_application_note",
            ConceptFormField::Identifier => "editor_identifier",
            ConceptFormField::DerivedFrom => "editor_derived_from",
            ConceptFormField::EnglishPreferredTerm => "editor_en_preferred_term",
            ConceptFormField::EnglishAcceptedTerm => "editor_en_accepted_term",
            ConceptFormField::EnglishDefinition => "editor_en_definition",
            _ => return,
        };
        let targets = self.focus_targets();
        if let Some(pos) = targets.iter().position(|t| match t {
            ConceptEditorFocusTarget::Input(id) => *id == expected_input,
            _ => false,
        }) {
            self.current_target_index = pos;
            self.focused_button = None;
        }
    }

    pub fn set_domain(&mut self, domain: BelongsToDomain) {
        self.belongs_to_domain = domain;
    }

    pub fn update_field(&mut self, field: ConceptFormField, value: String) {
        self.validation_error = None;
        self.sync_focus_to_field(field);
        match field {
            ConceptFormField::PreferredTerm => self.preferred_term = value,
            ConceptFormField::Definition => self.definition = value,
            ConceptFormField::BelongsToDomain => {
                let parsed = BelongsToDomain::from_str_loose(&value);
                match parsed {
                    BelongsToDomain::ModelRef(uri) => {
                        self.belongs_to_domain = BelongsToDomain::No;
                        self.model_uri = uri;
                    }
                    other => {
                        self.belongs_to_domain = other;
                    }
                }
            }
            ConceptFormField::ModelUri => self.model_uri = value,
            ConceptFormField::AcceptedTerm => self.accepted_term = value,
            ConceptFormField::DeprecatedTerm => self.deprecated_term = value,
            ConceptFormField::Example => self.example = value,
            ConceptFormField::Comment => self.comment = value,
            ConceptFormField::ApplicationNote => self.application_note = value,
            ConceptFormField::LegalSource => self.legal_source = value,
            ConceptFormField::Source => self.source = value,
            ConceptFormField::Identifier => self.identifier = value,
            ConceptFormField::DerivedFrom => self.derived_from = value,
            ConceptFormField::EnglishPreferredTerm => self.english_preferred_term = value,
            ConceptFormField::EnglishAcceptedTerm => self.english_accepted_term = value,
            ConceptFormField::EnglishDeprecatedTerm => self.english_deprecated_term = value,
            ConceptFormField::EnglishDefinition => self.english_definition = value,
            ConceptFormField::EnglishExample => self.english_example = value,
            ConceptFormField::EnglishComment => self.english_comment = value,
            ConceptFormField::EnglishApplicationNote => self.english_application_note = value,
        }
    }

    pub fn build_concept(&self) -> Result<Concept, ValidationError> {
        let belongs = match &self.belongs_to_domain {
            BelongsToDomain::Yes => BelongsToDomain::Yes,
            BelongsToDomain::No | BelongsToDomain::ModelRef(_) => {
                let trimmed = self.model_uri.trim();
                if trimmed.is_empty() {
                    BelongsToDomain::No
                } else {
                    BelongsToDomain::ModelRef(trimmed.to_string())
                }
            }
        };
        let mut concept = Concept::new(&self.preferred_term, &self.definition, belongs.clone());

        if let Some(id) = self.editing_id {
            concept = Concept::new_with_id(id, &self.preferred_term, &self.definition, belongs);
        }

        let opt = |s: &str| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        };

        concept.set_accepted_term(opt(&self.accepted_term));
        concept.set_deprecated_term(opt(&self.deprecated_term));
        concept.set_example(opt(&self.example));
        concept.set_comment(opt(&self.comment));
        concept.set_application_note(opt(&self.application_note));
        concept.set_legal_source(opt(&self.legal_source));
        concept.set_source(opt(&self.source));
        concept.set_identifier(opt(&self.identifier));
        concept.set_derived_from(opt(&self.derived_from));

        let en = crate::features::concepts::ConceptEnglishFields {
            preferred_term: opt(&self.english_preferred_term),
            accepted_term: opt(&self.english_accepted_term),
            deprecated_term: opt(&self.english_deprecated_term),
            definition: opt(&self.english_definition),
            example: opt(&self.english_example),
            comment: opt(&self.english_comment),
            application_note: opt(&self.english_application_note),
        };
        if !en.is_empty() {
            concept.set_english(Some(en));
        }

        ConceptValidator::validate(&concept)?;
        Ok(concept)
    }

    pub fn view(&self) -> Element<'_, Message> {
        use crate::ui::theme::{
            card_container_style, focused_button_style, modern_input_style, primary_button_style,
            secondary_button_style,
        };

        let is_edit = self.editing_id.is_some();
        let title_text = if is_edit {
            format!("Rediger Begreb: {}", self.preferred_term)
        } else {
            "Opret Nyt Begreb".to_string()
        };

        let title_row = row![
            text(title_text).size(20).color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(secondary_button_style)
                .on_press(Message::CancelConceptEdit),
        ]
        .align_y(Alignment::Center);

        let mut form = column![title_row].spacing(16);

        if let Some(err) = &self.validation_error {
            form = form.push(
                container(
                    row![
                        text("⚠️ ").size(16),
                        text(err).color(ThemeColors::ACCENT_RED).size(13),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                    border: iced::Border {
                        color: ThemeColors::ACCENT_RED,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([10, 14]),
            );
        }

        // 1. Primære felter (Foretrukken term, Emneområde)
        let term_input = column![
            text("Foretrukken dansk term *")
                .size(13)
                .color(ThemeColors::SLATE_800),
            text_input(
                "F.eks. Køretøj, Personbil, Myndighed...",
                &self.preferred_term
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

        let is_local = self.belongs_to_domain == BelongsToDomain::Yes;
        let is_local_focused = self.focused_button == Some(BTN_DOMAIN_LOCAL);
        let is_borrowed_focused = self.focused_button == Some(BTN_DOMAIN_BORROWED);
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
                    text_input("F.eks. https://data.gov.dk/model/core/cpr", &self.model_uri)
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
        let opt_syn = if self.accepted_term.trim().is_empty() {
            None
        } else {
            Some(self.accepted_term.as_str())
        };
        let lint_issues =
            DefinitionLinter::lint_text(&self.preferred_term, opt_syn, &self.definition);
        let aristotle = DefinitionLinter::analyze_aristotle(&self.definition);

        let mut definition_input =
            column![
            text("Definition * (Aristoteles' formel)").size(13).color(ThemeColors::SLATE_800),
            text_input(
                "Genus proximum + differentia specifica (hvad er det, og hvad adskiller det)...",
                &self.definition,
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
                .color(Color::from_rgb(0.75, 0.45, 0.05))]
            .spacing(3);

            for issue in &lint_issues {
                issues_col = issues_col.push(
                    text(format!("• {}", issue.message))
                        .size(11)
                        .color(Color::from_rgb(0.70, 0.40, 0.05)),
                );
            }

            let issues_box = container(issues_col)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.99, 0.97, 0.90))),
                    border: iced::Border {
                        color: Color::from_rgb(0.92, 0.80, 0.50),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([6, 10])
                .width(Length::Fill);

            definition_input = definition_input.push(issues_box);
        } else if !self.definition.trim().is_empty() {
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
                        color: Color::from_rgb(0.70, 0.88, 0.75),
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
            text_input("F.eks. LBK nr 1324 af 21/11/2023 § 2", &self.legal_source)
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
                &self.source
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

        form = form.push(primary_card);

        // Foldbar sektion med supplerende felter
        let is_supp_focused = self.focused_button == Some(BTN_TOGGLE_SUPPLEMENTARY);
        let toggle_supplementary_btn = button(
            text(if self.show_supplementary {
                "▲ Skjul supplerende felter"
            } else {
                "▼ Vis supplerende felter (Accepteret term, Eksempel, Identifikator m.fl.)"
            })
            .size(13),
        )
        .style(focused_button_style(secondary_button_style, is_supp_focused))
        .on_press(Message::ToggleShowAllFields)
        .padding([7, 14]);

        form = form.push(toggle_supplementary_btn);

        if self.show_supplementary {
            let supplementary_content = column![
                row![
                    column![
                        text("Accepteret dansk term")
                            .size(13)
                            .color(ThemeColors::SLATE_700),
                        text_input("Synonym eller tilladt betegnelse...", &self.accepted_term)
                            .id("editor_accepted_term")
                            .style(modern_input_style)
                            .on_input(|v| Message::UpdateConceptField(
                                ConceptFormField::AcceptedTerm,
                                v
                            ))
                            .padding(8),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Frarådet dansk term")
                            .size(13)
                            .color(ThemeColors::SLATE_700),
                        text_input("Betegnelse der ikke bør anvendes...", &self.deprecated_term)
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
                        text_input("Typisk tilfælde der illustrerer begrebet...", &self.example)
                            .id("editor_example")
                            .style(modern_input_style)
                            .on_input(|v| Message::UpdateConceptField(ConceptFormField::Example, v))
                            .padding(8),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Kommentar").size(13).color(ThemeColors::SLATE_700),
                        text_input("Supplerende bemærkning...", &self.comment)
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
                        &self.application_note
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
                            &self.identifier
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
                        text_input("HTTP-URI på oprindeligt begreb...", &self.derived_from)
                            .id("editor_derived_from")
                            .style(modern_input_style)
                            .on_input(|v| Message::UpdateConceptField(
                                ConceptFormField::DerivedFrom,
                                v
                            ))
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
                        text_input("prefLabel (en)...", &self.english_preferred_term)
                            .id("editor_en_preferred_term")
                            .style(modern_input_style)
                            .on_input(|v| Message::UpdateConceptField(ConceptFormField::EnglishPreferredTerm, v))
                            .padding(8),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    column![
                        text("Accepteret term (en)").size(13).color(ThemeColors::SLATE_700),
                        text_input("altLabel (en)...", &self.english_accepted_term)
                            .id("editor_en_accepted_term")
                            .style(modern_input_style)
                            .on_input(|v| Message::UpdateConceptField(ConceptFormField::EnglishAcceptedTerm, v))
                            .padding(8),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(16),
                column![
                    text("Definition (en)").size(13).color(ThemeColors::SLATE_700),
                    text_input("definition (en)...", &self.english_definition)
                        .id("editor_en_definition")
                        .style(modern_input_style)
                        .on_input(|v| Message::UpdateConceptField(ConceptFormField::EnglishDefinition, v))
                        .padding(8),
                ]
                .spacing(4),
            ]
            .spacing(14);

            let supp_card = container(supplementary_content)
                .style(card_container_style)
                .padding(16)
                .width(Length::Fill);

            form = form.push(supp_card);
        }

        let is_cancel_focused = self.focused_button == Some(BTN_CANCEL);
        let is_save_focused = self.focused_button == Some(BTN_SAVE);

        let action_bar = row![
            Space::new().width(Length::Fill),
            button(text("Annuller").size(13))
                .style(focused_button_style(secondary_button_style, is_cancel_focused))
                .on_press(Message::CancelConceptEdit)
                .padding([8, 16]),
            button(
                text(if is_edit {
                    "Gem ændringer"
                } else {
                    "Opret begreb"
                })
                .size(13)
            )
            .style(focused_button_style(primary_button_style, is_save_focused))
            .on_press(Message::SaveConcept)
            .padding([8, 20]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        form = form.push(Space::new().height(8));
        form = form.push(action_bar);

        scrollable(form).height(Length::Fill).into()
    }
}
