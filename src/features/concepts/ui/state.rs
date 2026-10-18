use crate::features::concepts::{BelongsToDomain, Concept, ConceptValidator, ValidationError};
use crate::ui::app::Message;
use crate::ui::focus::{FocusCycle, FocusTarget};
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
    pub is_inline: bool,
    pub focus: FocusCycle,
}

impl Default for ConceptEditorState {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl ConceptEditorState {
    pub fn new_empty() -> Self {
        let mut editor = Self {
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
            is_inline: false,
            focus: FocusCycle::default(),
        };
        let targets = editor.focus_targets();
        editor.focus = FocusCycle::with_targets(targets, Some(0));
        editor
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

        let mut editor = Self {
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
            is_inline: false,
            focus: FocusCycle::default(),
        };
        let targets = editor.focus_targets();
        editor.focus = FocusCycle::with_targets(targets, Some(0));
        editor
    }

    pub fn set_inline(&mut self, inline: bool) {
        self.is_inline = inline;
        let targets = self.focus_targets();
        self.focus = FocusCycle::with_targets(targets, Some(0));
        self.current_target_index = 0;
        self.focused_button = None;
    }

    pub fn focused_button(&self) -> Option<usize> {
        self.focus.focused_button().or(self.focused_button)
    }

    pub fn set_focused_button(&mut self, btn: Option<usize>) {
        let targets = self.focus_targets();
        self.focus = FocusCycle::with_targets(targets, None);
        if let Some(b) = btn {
            if let Some(pos) = self
                .focus_targets()
                .iter()
                .position(|t| matches!(t, FocusTarget::Button(tb) if *tb == b))
            {
                self.focus.set_current(Some(pos));
            }
        }
        self.current_target_index = self.focus.current().unwrap_or(0);
        self.focused_button = self.focus.focused_button();
    }

    pub fn focus_targets(&self) -> Vec<FocusTarget> {
        if self.is_inline {
            return vec![
                FocusTarget::Input("preferred_term_input"),
                FocusTarget::Input("inline_concept_definition"),
                FocusTarget::Button(BTN_CANCEL),
                FocusTarget::Button(BTN_SAVE),
            ];
        }

        let mut targets = Vec::new();
        // 1. Foretrukken dansk term
        targets.push(FocusTarget::Input("preferred_term_input"));
        // 2. Lokalt begreb (Ja)
        targets.push(FocusTarget::Button(BTN_DOMAIN_LOCAL));
        // 3. Indlånt begreb (Nej)
        targets.push(FocusTarget::Button(BTN_DOMAIN_BORROWED));

        if self.belongs_to_domain == BelongsToDomain::No {
            targets.push(FocusTarget::Input("editor_model_uri"));
        }

        targets.push(FocusTarget::Input("editor_definition"));
        targets.push(FocusTarget::Input("editor_legal_source"));
        targets.push(FocusTarget::Input("editor_source"));

        targets.push(FocusTarget::Button(BTN_TOGGLE_SUPPLEMENTARY));

        if self.show_supplementary {
            targets.push(FocusTarget::Input("editor_accepted_term"));
            targets.push(FocusTarget::Input("editor_deprecated_term"));
            targets.push(FocusTarget::Input("editor_example"));
            targets.push(FocusTarget::Input("editor_comment"));
            targets.push(FocusTarget::Input("editor_application_note"));
            targets.push(FocusTarget::Input("editor_identifier"));
            targets.push(FocusTarget::Input("editor_derived_from"));
            targets.push(FocusTarget::Input("editor_en_preferred_term"));
            targets.push(FocusTarget::Input("editor_en_accepted_term"));
            targets.push(FocusTarget::Input("editor_en_definition"));
        }

        targets.push(FocusTarget::Button(BTN_CANCEL));
        targets.push(FocusTarget::Button(BTN_SAVE));
        targets
    }

    pub fn advance_focus(&mut self, reverse: bool) -> FocusTarget {
        let targets = self.focus_targets();
        if self.focus.count() != targets.len() {
            self.focus = FocusCycle::with_targets(targets, self.focus.current());
        }
        if reverse {
            self.focus.previous();
        } else {
            self.focus.next();
        }
        self.current_target_index = self.focus.current().unwrap_or(0);
        self.focused_button = self.focus.focused_button();
        self.focus
            .current_target()
            .cloned()
            .unwrap_or(FocusTarget::Button(BTN_SAVE))
    }

    pub fn sync_focus_to_field(&mut self, field: ConceptFormField) {
        let expected_input = match field {
            ConceptFormField::PreferredTerm => "preferred_term_input",
            ConceptFormField::ModelUri => "editor_model_uri",
            ConceptFormField::Definition => {
                if self.is_inline {
                    "inline_concept_definition"
                } else {
                    "editor_definition"
                }
            }
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
        self.focus = FocusCycle::with_targets(targets, self.focus.current());
        self.focus.sync_input(expected_input);
        self.current_target_index = self.focus.current().unwrap_or(0);
        self.focused_button = self.focus.focused_button();
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

    pub fn view(&self) -> iced::Element<'_, Message> {
        crate::features::concepts::ui::modal::view(self)
    }
}
