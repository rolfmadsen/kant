use crate::features::concept_model::ConceptGraph;
use crate::features::concepts::{Concept, ConceptValidator, ValidationError};
use crate::features::information_model::{
    ClassGraph, InformationClass, InformationDataType, InformationEnumeration, InformationModel,
    PrimitiveType,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod decomposed;
pub mod recent;
pub mod storage;
pub mod ui;

pub use recent::RecentStore;
pub use ui::*;

/// Hjælpefunktion til at returnere aktuel dato i ISO-8601 YYYY-MM-DD format uden eksterne afhængigheder.
pub fn current_date_iso() -> String {
    unsafe {
        let mut now: libc::time_t = 0;
        libc::time(&mut now);
        let mut tm: libc::tm = std::mem::zeroed();

        #[cfg(windows)]
        let success = libc::localtime_s(&mut tm, &now) == 0;

        #[cfg(not(windows))]
        let success = !libc::localtime_r(&now, &mut tm).is_null();

        if success {
            format!(
                "{:04}-{:02}-{:02}",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday
            )
        } else {
            "2026-09-30".to_string()
        }
    }
}

/// Modellens livscyklus-status jf. FDA Modelregel 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    #[serde(alias = "Draft", alias = "Candidate")]
    Development,
    #[serde(alias = "Approved")]
    Completed,
    Deprecated,
    Withdrawn,
}

impl ModelStatus {
    pub const ALL: [ModelStatus; 4] = [
        ModelStatus::Development,
        ModelStatus::Completed,
        ModelStatus::Deprecated,
        ModelStatus::Withdrawn,
    ];

    #[allow(non_upper_case_globals)]
    pub const Draft: ModelStatus = ModelStatus::Development;
    #[allow(non_upper_case_globals)]
    pub const Candidate: ModelStatus = ModelStatus::Development;
    #[allow(non_upper_case_globals)]
    pub const Approved: ModelStatus = ModelStatus::Completed;
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelStatus::Development => write!(f, "Under udvikling (Development)"),
            ModelStatus::Completed => write!(f, "Endelig (Completed)"),
            ModelStatus::Deprecated => write!(f, "Forældet (Deprecated)"),
            ModelStatus::Withdrawn => write!(f, "Trukket tilbage (Withdrawn)"),
        }
    }
}

/// Forretningsgodkendelsens status jf. FDA Modelregel 11 og Kapitel 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    AwaitingApproval,
    Approved,
    ApprovedWithRemarks,
    NotRelevant,
}

impl ApprovalStatus {
    pub const ALL: [ApprovalStatus; 4] = [
        ApprovalStatus::AwaitingApproval,
        ApprovalStatus::Approved,
        ApprovalStatus::ApprovedWithRemarks,
        ApprovalStatus::NotRelevant,
    ];
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::AwaitingApproval => write!(f, "Afventer godkendelse"),
            ApprovalStatus::Approved => write!(f, "Godkendt"),
            ApprovalStatus::ApprovedWithRemarks => write!(f, "Godkendt med bemærkninger"),
            ApprovalStatus::NotRelevant => write!(f, "Ikke relevant"),
        }
    }
}

/// Modelomfang jf. FDA Modelregel 25 og Kapitel 6 (kernemodel vs. anvendelsesmodel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelScope {
    Core,
    ApplicationProfile,
}

impl ModelScope {
    pub const ALL: [ModelScope; 2] = [ModelScope::Core, ModelScope::ApplicationProfile];
}

impl std::fmt::Display for ModelScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelScope::Core => write!(f, "Kernemodel (Core)"),
            ModelScope::ApplicationProfile => write!(f, "Anvendelsesmodel (Profile)"),
        }
    }
}

/// Modelmetadata jf. FDA Modelreglerne v2.1.0 Kapitel 6, Tabel D og E.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelMetadata {
    name: String,
    description: String,
    uri: String,
    responsible_org: String,
    domain_area: String,
    version: String,
    model_status: ModelStatus,
    approval_status: ApprovalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    model_scope: ModelScope,
    language: String,
    date_modified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    was_derived_from: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    legal_sources: Vec<String>,
}

#[derive(Deserialize)]
struct RawModelMetadata {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    uri: String,
    #[serde(default)]
    responsible_org: String,
    #[serde(default)]
    domain_area: String,
    #[serde(default = "default_version_str")]
    version: String,
    #[serde(default)]
    model_status: Option<ModelStatus>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    approval_status: Option<ApprovalStatus>,
    #[serde(default)]
    approved_by: Option<String>,
    #[serde(default)]
    is_approved_by: Option<String>,
    #[serde(default)]
    model_scope: Option<ModelScope>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    date_modified: Option<String>,
    #[serde(default)]
    version_notes: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    was_derived_from: Option<String>,
    #[serde(default)]
    legal_sources: Option<Vec<String>>,
    #[serde(default)]
    legal_source: Option<String>,
}

fn default_version_str() -> String {
    "0.1.0".to_string()
}

impl<'de> Deserialize<'de> for ModelMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawModelMetadata::deserialize(deserializer)?;

        let model_status = if let Some(ms) = raw.model_status {
            ms
        } else if let Some(ref st) = raw.status {
            match st.trim().to_lowercase().as_str() {
                "approved" | "godkendt" | "completed" | "endelig" => ModelStatus::Completed,
                "deprecated" | "forældet" => ModelStatus::Deprecated,
                "withdrawn" | "trukket tilbage" => ModelStatus::Withdrawn,
                _ => ModelStatus::Development,
            }
        } else {
            ModelStatus::Development
        };

        let approval_status = if let Some(as_status) = raw.approval_status {
            as_status
        } else if let Some(ref st) = raw.status {
            match st.trim().to_lowercase().as_str() {
                "approved" | "godkendt" => ApprovalStatus::Approved,
                _ => ApprovalStatus::AwaitingApproval,
            }
        } else {
            ApprovalStatus::AwaitingApproval
        };

        let approved_by = raw.approved_by.or(raw.is_approved_by);
        let model_scope = raw.model_scope.unwrap_or(ModelScope::Core);
        let language = raw.language.unwrap_or_else(|| "da".to_string());
        let date_modified = raw.date_modified.unwrap_or_else(current_date_iso);
        let version_notes = raw.version_notes;
        let source = raw.source;
        let was_derived_from = raw.was_derived_from;

        let mut legal_sources = raw.legal_sources.unwrap_or_default();
        if legal_sources.is_empty() {
            if let Some(ls) = raw.legal_source {
                if !ls.trim().is_empty() {
                    legal_sources.push(ls);
                }
            }
        }

        Ok(ModelMetadata {
            name: raw.name,
            description: raw.description,
            uri: raw.uri,
            responsible_org: raw.responsible_org,
            domain_area: raw.domain_area,
            version: raw.version,
            model_status,
            approval_status,
            approved_by,
            model_scope,
            language,
            date_modified,
            version_notes,
            source,
            was_derived_from,
            legal_sources,
        })
    }
}

impl ModelMetadata {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        uri: impl Into<String>,
        responsible_org: impl Into<String>,
        domain_area: impl Into<String>,
        version: impl Into<String>,
        model_status: ModelStatus,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            uri: uri.into(),
            responsible_org: responsible_org.into(),
            domain_area: domain_area.into(),
            version: version.into(),
            model_status,
            approval_status: ApprovalStatus::AwaitingApproval,
            approved_by: None,
            model_scope: ModelScope::Core,
            language: "da".to_string(),
            date_modified: current_date_iso(),
            version_notes: None,
            source: None,
            was_derived_from: None,
            legal_sources: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn set_uri(&mut self, uri: impl Into<String>) {
        self.uri = uri.into();
    }

    pub fn responsible_org(&self) -> &str {
        &self.responsible_org
    }

    pub fn set_responsible_org(&mut self, responsible_org: impl Into<String>) {
        self.responsible_org = responsible_org.into();
    }

    pub fn domain_area(&self) -> &str {
        &self.domain_area
    }

    pub fn set_domain_area(&mut self, domain_area: impl Into<String>) {
        self.domain_area = domain_area.into();
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_version(&mut self, version: impl Into<String>) {
        self.version = version.into();
    }

    pub fn model_status(&self) -> ModelStatus {
        self.model_status
    }

    pub fn set_model_status(&mut self, status: ModelStatus) {
        self.model_status = status;
    }

    /// Bagudkompatibelt alias for `model_status()`
    pub fn status(&self) -> ModelStatus {
        self.model_status
    }

    /// Bagudkompatibelt alias for `set_model_status()`
    pub fn set_status(&mut self, status: ModelStatus) {
        self.model_status = status;
    }

    pub fn approval_status(&self) -> ApprovalStatus {
        self.approval_status
    }

    pub fn set_approval_status(&mut self, status: ApprovalStatus) {
        self.approval_status = status;
    }

    pub fn approved_by(&self) -> Option<&str> {
        self.approved_by.as_deref()
    }

    pub fn set_approved_by(&mut self, approved_by: Option<String>) {
        self.approved_by = approved_by;
    }

    pub fn model_scope(&self) -> ModelScope {
        self.model_scope
    }

    pub fn set_model_scope(&mut self, scope: ModelScope) {
        self.model_scope = scope;
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }

    pub fn date_modified(&self) -> &str {
        &self.date_modified
    }

    pub fn set_date_modified(&mut self, date: impl Into<String>) {
        self.date_modified = date.into();
    }

    pub fn version_notes(&self) -> Option<&str> {
        self.version_notes.as_deref()
    }

    pub fn set_version_notes(&mut self, notes: Option<String>) {
        self.version_notes = notes;
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn set_source(&mut self, source: Option<String>) {
        self.source = source;
    }

    pub fn was_derived_from(&self) -> Option<&str> {
        self.was_derived_from.as_deref()
    }

    pub fn set_was_derived_from(&mut self, uri: Option<String>) {
        self.was_derived_from = uri;
    }

    pub fn legal_sources(&self) -> &[String] {
        &self.legal_sources
    }

    pub fn set_legal_sources(&mut self, sources: Vec<String>) {
        self.legal_sources = sources;
    }

    pub fn add_legal_source(&mut self, source: impl Into<String>) {
        self.legal_sources.push(source.into());
    }

    /// Bagudkompatibel singular accessor for første lovkilde
    pub fn legal_source(&self) -> Option<&str> {
        self.legal_sources.first().map(|s| s.as_str())
    }

    /// Bagudkompatibel singular mutator for lovkilde
    pub fn set_legal_source(&mut self, source: Option<String>) {
        if let Some(s) = source {
            self.legal_sources = vec![s];
        } else {
            self.legal_sources.clear();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelProject {
    metadata: ModelMetadata,
    concepts: Vec<Concept>,
    #[serde(default)]
    concept_graph: ConceptGraph,
    #[serde(default)]
    information_model: InformationModel,
    #[serde(default)]
    information_graph: ClassGraph,
}

impl ModelProject {
    pub fn new(metadata: ModelMetadata) -> Self {
        Self {
            metadata,
            concepts: Vec::new(),
            concept_graph: ConceptGraph::new(),
            information_model: InformationModel::new(),
            information_graph: ClassGraph::new(),
        }
    }

    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut ModelMetadata {
        &mut self.metadata
    }

    pub fn concepts(&self) -> &[Concept] {
        &self.concepts
    }

    pub fn concepts_mut(&mut self) -> &mut Vec<Concept> {
        &mut self.concepts
    }

    pub fn concept_graph(&self) -> &ConceptGraph {
        &self.concept_graph
    }

    pub fn concept_graph_mut(&mut self) -> &mut ConceptGraph {
        &mut self.concept_graph
    }

    pub fn information_model(&self) -> &InformationModel {
        &self.information_model
    }

    pub fn information_model_mut(&mut self) -> &mut InformationModel {
        &mut self.information_model
    }

    pub fn information_graph(&self) -> &ClassGraph {
        &self.information_graph
    }

    pub fn information_graph_mut(&mut self) -> &mut ClassGraph {
        &mut self.information_graph
    }

    pub fn remove_information_class(&mut self, id: Uuid) -> Option<InformationClass> {
        let removed = self.information_model.remove_class(id);
        if removed.is_some() {
            self.information_graph.remove_class_node(id);
        }
        removed
    }

    pub fn remove_information_enumeration(&mut self, id: Uuid) -> Option<InformationEnumeration> {
        let removed = self.information_model.remove_enumeration(id);
        if removed.is_some() {
            self.information_graph.remove_class_node(id);
            for class in self.information_model.classes_mut() {
                for attr in class.attributes_mut() {
                    if let InformationDataType::Enumeration { enumeration_id } = attr.data_type() {
                        if *enumeration_id == id {
                            attr.set_data_type(PrimitiveType::CharacterString);
                        }
                    }
                }
            }
            self.sync_information_graph();
        }
        removed
    }

    pub fn sync_concept_graph(&mut self) {
        self.concept_graph.sync_with_concepts(&self.concepts);
    }

    pub fn populate_missing_concepts_to_graph(&mut self) {
        self.concept_graph.populate_missing_concepts(&self.concepts);
    }

    pub fn sync_information_graph(&mut self) {
        self.information_model.sync_attribute_dependencies();
        self.information_graph
            .sync_with_information_model(&self.information_model);
    }

    pub fn migrate_information_graph_edges_to_model(&mut self) {
        self.information_graph
            .migrate_edges_to_model(&mut self.information_model);
    }

    pub fn get_concept(&self, id: Uuid) -> Option<&Concept> {
        self.concepts.iter().find(|c| c.id() == id)
    }

    pub fn get_concept_mut(&mut self, id: Uuid) -> Option<&mut Concept> {
        self.concepts.iter_mut().find(|c| c.id() == id)
    }

    pub fn sort_concepts_alphabetically(&mut self) {
        self.concepts.sort_by(|a, b| {
            a.preferred_term()
                .to_lowercase()
                .cmp(&b.preferred_term().to_lowercase())
        });
    }

    pub fn add_concept(&mut self, concept: Concept) -> Result<Uuid, ValidationError> {
        ConceptValidator::validate(&concept)?;
        let id = concept.id();
        self.concepts.push(concept);
        self.sort_concepts_alphabetically();
        self.sync_concept_graph();
        Ok(id)
    }

    pub fn update_concept(&mut self, concept: Concept) -> Result<(), ValidationError> {
        ConceptValidator::validate(&concept)?;
        if let Some(existing) = self.concepts.iter_mut().find(|c| c.id() == concept.id()) {
            *existing = concept;
            self.sort_concepts_alphabetically();
            self.sync_concept_graph();
            Ok(())
        } else {
            Err(ValidationError::MissingRequiredField(
                "Begreb ikke fundet i projekt",
            ))
        }
    }

    pub fn remove_concept(&mut self, id: Uuid) -> Option<Concept> {
        if let Some(idx) = self.concepts.iter().position(|c| c.id() == id) {
            let removed = self.concepts.remove(idx);
            self.concept_graph.remove_node_by_concept(id);
            self.information_model.remove_concept_references(id);
            Some(removed)
        } else {
            None
        }
    }
}

impl Default for ModelProject {
    fn default() -> Self {
        Self::new(ModelMetadata::new(
            "Nyt FDA Modelprojekt",
            "",
            "",
            "",
            "",
            "0.1.0",
            ModelStatus::Draft,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_metadata_setters_and_status() {
        let mut meta = ModelMetadata::new(
            "Orig",
            "Desc",
            "https://orig",
            "Org",
            "Domain",
            "1.0.0",
            ModelStatus::Draft,
        );

        meta.set_name("Updated Name");
        meta.set_description("Updated Desc");
        meta.set_uri("https://updated");
        meta.set_responsible_org("Updated Org");
        meta.set_domain_area("Updated Domain");
        meta.set_version("2.0.0");
        meta.set_status(ModelStatus::Approved);

        assert_eq!(meta.name(), "Updated Name");
        assert_eq!(meta.description(), "Updated Desc");
        assert_eq!(meta.uri(), "https://updated");
        assert_eq!(meta.responsible_org(), "Updated Org");
        assert_eq!(meta.domain_area(), "Updated Domain");
        assert_eq!(meta.version(), "2.0.0");
        assert_eq!(meta.status(), ModelStatus::Approved);

        assert_eq!(ModelStatus::ALL.len(), 4);
        assert_eq!(
            format!("{}", ModelStatus::Development),
            "Under udvikling (Development)"
        );
        assert_eq!(format!("{}", ModelStatus::Completed), "Endelig (Completed)");
        assert_eq!(
            format!("{}", ModelStatus::Deprecated),
            "Forældet (Deprecated)"
        );
        assert_eq!(
            format!("{}", ModelStatus::Withdrawn),
            "Trukket tilbage (Withdrawn)"
        );
    }
}
