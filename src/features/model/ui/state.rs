use crate::features::model::{ApprovalStatus, ModelMetadata, ModelScope, ModelStatus};

pub const META_BTN_CANCEL: usize = 0;
pub const META_BTN_SAVE: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataField {
    Name,
    Description,
    DomainArea,
    ResponsibleOrg,
    Uri,
    Version,
    ApprovedBy,
    Language,
    LegalSources,
    Source,
    WasDerivedFrom,
    VersionNotes,
    DateModified,
}

#[derive(Debug, Clone)]
pub struct ModelMetadataModalState {
    pub name: String,
    pub description: String,
    pub status: ModelStatus,
    pub approval_status: ApprovalStatus,
    pub approved_by: String,
    pub model_scope: ModelScope,
    pub language: String,
    pub domain_area: String,
    pub responsible_org: String,
    pub uri: String,
    pub version: String,
    pub version_notes: String,
    pub source: String,
    pub was_derived_from: String,
    pub legal_sources: String,
    pub date_modified: String,
}

impl ModelMetadataModalState {
    pub fn from_metadata(meta: &ModelMetadata) -> Self {
        Self {
            name: if meta.name() == "Nyt FDA Modelprojekt" {
                String::new()
            } else {
                meta.name().to_string()
            },
            description: if meta.description() == "Beskrivelse af modelprojektet" {
                String::new()
            } else {
                meta.description().to_string()
            },
            status: meta.model_status(),
            approval_status: meta.approval_status(),
            approved_by: meta.approved_by().unwrap_or("").to_string(),
            model_scope: meta.model_scope(),
            language: meta.language().to_string(),
            domain_area: if meta.domain_area() == "Emneområde" {
                String::new()
            } else {
                meta.domain_area().to_string()
            },
            responsible_org: if meta.responsible_org() == "Ansvarlig Myndighed" {
                String::new()
            } else {
                meta.responsible_org().to_string()
            },
            uri: if meta.uri() == "https://data.gov.dk/model/core/new-model" {
                String::new()
            } else {
                meta.uri().to_string()
            },
            version: if meta.version() == "0.1.0" {
                String::new()
            } else {
                meta.version().to_string()
            },
            version_notes: meta.version_notes().unwrap_or("").to_string(),
            source: meta.source().unwrap_or("").to_string(),
            was_derived_from: meta.was_derived_from().unwrap_or("").to_string(),
            legal_sources: meta.legal_sources().join(", "),
            date_modified: meta.date_modified().to_string(),
        }
    }
}
