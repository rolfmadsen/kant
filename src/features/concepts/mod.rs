use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod excel;
pub mod linter;
pub mod ui;
pub use linter::{
    AristotleBreakdown, DefinitionIssueKind, DefinitionLintIssue, DefinitionLinter,
    DefinitionSeverity,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BelongsToDomain {
    Yes,
    No,
    ModelRef(String),
}

impl BelongsToDomain {
    pub fn display_label(&self) -> String {
        match self {
            Self::Yes => "Ja (Lokalt begreb)".to_string(),
            Self::No => "Nej (Indlånt begreb)".to_string(),
            Self::ModelRef(uri) => format!("Model: {}", uri),
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Yes)
    }

    pub fn from_str_loose(s: &str) -> Self {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("ja") || trimmed.eq_ignore_ascii_case("yes") {
            Self::Yes
        } else if trimmed.eq_ignore_ascii_case("nej") || trimmed.eq_ignore_ascii_case("no") {
            Self::No
        } else if !trimmed.is_empty() {
            Self::ModelRef(trimmed.to_string())
        } else {
            Self::Yes
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConceptEnglishFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_term: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_term: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated_term: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_note: Option<String>,
}

impl ConceptEnglishFields {
    pub fn is_empty(&self) -> bool {
        self.preferred_term
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
            && self
                .accepted_term
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            && self
                .deprecated_term
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            && self.definition.as_deref().unwrap_or("").trim().is_empty()
            && self.example.as_deref().unwrap_or("").trim().is_empty()
            && self.comment.as_deref().unwrap_or("").trim().is_empty()
            && self
                .application_note
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    id: Uuid,
    preferred_term: String,
    definition: String,
    belongs_to_domain: BelongsToDomain,
    accepted_term: Option<String>,
    deprecated_term: Option<String>,
    example: Option<String>,
    comment: Option<String>,
    application_note: Option<String>,
    legal_source: Option<String>,
    source: Option<String>,
    identifier: Option<String>,
    derived_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    english: Option<ConceptEnglishFields>,
}

impl Concept {
    pub fn new(
        preferred_term: impl Into<String>,
        definition: impl Into<String>,
        belongs_to_domain: BelongsToDomain,
    ) -> Self {
        Self::new_with_id(
            Uuid::new_v4(),
            preferred_term,
            definition,
            belongs_to_domain,
        )
    }

    pub fn new_with_id(
        id: Uuid,
        preferred_term: impl Into<String>,
        definition: impl Into<String>,
        belongs_to_domain: BelongsToDomain,
    ) -> Self {
        Self {
            id,
            preferred_term: preferred_term.into(),
            definition: definition.into(),
            belongs_to_domain,
            accepted_term: None,
            deprecated_term: None,
            example: None,
            comment: None,
            application_note: None,
            legal_source: None,
            source: None,
            identifier: None,
            derived_from: None,
            english: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn preferred_term(&self) -> &str {
        &self.preferred_term
    }

    pub fn set_preferred_term(&mut self, term: impl Into<String>) {
        self.preferred_term = term.into();
    }

    pub fn definition(&self) -> &str {
        &self.definition
    }

    pub fn set_definition(&mut self, definition: impl Into<String>) {
        self.definition = definition.into();
    }

    pub fn belongs_to_domain(&self) -> &BelongsToDomain {
        &self.belongs_to_domain
    }

    pub fn set_belongs_to_domain(&mut self, belongs: BelongsToDomain) {
        self.belongs_to_domain = belongs;
    }

    pub fn accepted_term(&self) -> Option<&str> {
        self.accepted_term.as_deref()
    }

    pub fn set_accepted_term(&mut self, term: Option<String>) {
        self.accepted_term = term;
    }

    pub fn deprecated_term(&self) -> Option<&str> {
        self.deprecated_term.as_deref()
    }

    pub fn set_deprecated_term(&mut self, term: Option<String>) {
        self.deprecated_term = term;
    }

    pub fn example(&self) -> Option<&str> {
        self.example.as_deref()
    }

    pub fn set_example(&mut self, example: Option<String>) {
        self.example = example;
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }

    pub fn application_note(&self) -> Option<&str> {
        self.application_note.as_deref()
    }

    pub fn set_application_note(&mut self, note: Option<String>) {
        self.application_note = note;
    }

    pub fn legal_source(&self) -> Option<&str> {
        self.legal_source.as_deref()
    }

    pub fn set_legal_source(&mut self, source: Option<String>) {
        self.legal_source = source;
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn set_source(&mut self, source: Option<String>) {
        self.source = source;
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    pub fn set_identifier(&mut self, id: Option<String>) {
        self.identifier = id;
    }

    pub fn derived_from(&self) -> Option<&str> {
        self.derived_from.as_deref()
    }

    pub fn set_derived_from(&mut self, derived: Option<String>) {
        self.derived_from = derived;
    }

    pub fn english(&self) -> Option<&ConceptEnglishFields> {
        self.english.as_ref()
    }

    pub fn english_mut(&mut self) -> Option<&mut ConceptEnglishFields> {
        self.english.as_mut()
    }

    pub fn set_english(&mut self, english: Option<ConceptEnglishFields>) {
        self.english = match english {
            Some(f) if f.is_empty() => None,
            other => other,
        };
    }

    pub fn english_preferred_term(&self) -> Option<&str> {
        self.english
            .as_ref()
            .and_then(|e| e.preferred_term.as_deref())
    }

    pub fn english_definition(&self) -> Option<&str> {
        self.english.as_ref().and_then(|e| e.definition.as_deref())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Obligatorisk felt mangler: {0}")]
    MissingRequiredField(&'static str),

    #[error("Ugyldigt URI format: {0}")]
    InvalidUri(String),
}

pub struct ConceptValidator;

impl ConceptValidator {
    pub fn validate(concept: &Concept) -> Result<(), ValidationError> {
        if concept.preferred_term.trim().is_empty() {
            return Err(ValidationError::MissingRequiredField(
                "Foretrukken dansk term",
            ));
        }
        if concept.definition.trim().is_empty() {
            return Err(ValidationError::MissingRequiredField("Definition"));
        }
        if let Some(uri) = &concept.identifier {
            if !uri.trim().is_empty() && url::Url::parse(uri).is_err() {
                return Err(ValidationError::InvalidUri(uri.clone()));
            }
        }
        if let Some(uri) = &concept.derived_from {
            if !uri.trim().is_empty() && url::Url::parse(uri).is_err() {
                return Err(ValidationError::InvalidUri(uri.clone()));
            }
        }
        Ok(())
    }
}
