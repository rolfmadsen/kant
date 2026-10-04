use crate::features::concept_model::{NodeId, RelationKind};
use crate::features::concepts::{BelongsToDomain, Concept, ConceptValidator, ValidationError};
use crate::features::export::{
    export_concept_model_svg, export_concepts_to_csv, export_information_model_svg,
    export_model_report_html, export_model_report_markdown, export_to_shacl_turtle,
    export_to_skos_turtle, export_to_xmi_2_1,
};
use crate::features::git::{
    ChangeAction, DomainChangeEvent, DomainEventMapper, GitService, PullResult, RepoSyncStatus,
};
use crate::features::information_model::{
    Attribute, ClassRelation, InformationClass, InformationDataType, InformationEnumeration,
    Multiplicity, PrimitiveType,
};
use crate::features::model::storage::ProjectStorage;
use crate::features::model::{
    ApprovalStatus, ModelMetadata, ModelProject, ModelScope, ModelStatus,
};
use crate::ui::concept_editor::ConceptEditorState;
use crate::ui::concept_model_view;
use crate::ui::concept_table;
use crate::ui::diagram_canvas::{CanvasEdge, CanvasNode};
use crate::ui::information_model_view;
use crate::ui::theme::{
    card_container_style, modal_backdrop_style, modal_card_style, modern_input_style,
    pill_container_style, primary_button_style, secondary_button_style, segmented_tab_button,
    ThemeColors,
};
use iced::event::{self, Event};
use iced::keyboard::{self, key::Named, Key};
use iced::mouse;
use iced::widget::{
    button, checkbox, column, container, mouse_area, operation, pick_list, row, scrollable, stack,
    svg, text, text_input, tooltip, Space,
};
use iced::{Alignment, Element, Length, Point, Subscription, Task};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PublishAsyncResult {
    Success {
        sync_status: RepoSyncStatus,
    },
    Conflict {
        conflicts: Vec<crate::features::model::merge::ModelConflict>,
        sync_status: RepoSyncStatus,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ExcelImportPreviewState {
    pub import_summary: crate::features::concepts::excel::ExcelImportSummary,
    pub new_count: usize,
    pub update_count: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsModalState {
    pub diagnostics: crate::features::diagnostics::SystemDiagnostics,
    pub logs: Vec<String>,
}

impl DiagnosticsModalState {
    pub fn new() -> Self {
        Self {
            diagnostics: crate::features::diagnostics::get_system_diagnostics(),
            logs: crate::features::diagnostics::get_recent_logs(),
        }
    }
}

impl Default for DiagnosticsModalState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PublishModalState {
    pub message: String,
    pub preview_events: Vec<DomainChangeEvent>,
    pub is_publishing: bool,
    pub error: Option<String>,
}

impl PublishModalState {
    pub fn new(message: String, preview_events: Vec<DomainChangeEvent>) -> Self {
        Self {
            message,
            preview_events,
            is_publishing: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Ours,
    Theirs,
}

#[derive(Debug, Clone)]
pub struct ModelHistoryModalState {
    pub target_element_id: Option<Uuid>,
    pub commits: Vec<crate::features::git::GitCommitInfo>,
}

impl ModelHistoryModalState {
    pub fn new(
        target_element_id: Option<Uuid>,
        commits: Vec<crate::features::git::GitCommitInfo>,
    ) -> Self {
        Self {
            target_element_id,
            commits,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConflictResolverModalState {
    pub conflicts: Vec<crate::features::model::merge::ModelConflict>,
    pub resolved_choices: std::collections::HashMap<usize, ConflictChoice>,
}

impl ConflictResolverModalState {
    pub fn new(conflicts: Vec<crate::features::model::merge::ModelConflict>) -> Self {
        let mut resolved_choices = std::collections::HashMap::new();
        for (i, _) in conflicts.iter().enumerate() {
            resolved_choices.insert(i, ConflictChoice::Ours);
        }
        Self {
            conflicts,
            resolved_choices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitConnectionModalState {
    pub remote_url: String,
    pub target_dir: String,
    pub custom_target_dir: bool,
    pub token: String,
    pub author_name: String,
    pub author_email: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCloneModalState {
    pub remote_url: String,
    pub target_dir: String,
    pub custom_target_dir: bool,
    pub token: String,
    pub is_cloning: bool,
    pub error_message: Option<String>,
}

pub type EdgeApp = App;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeOption {
    pub id: NodeId,
    pub label: String,
}

impl std::fmt::Display for NodeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptOption {
    pub id: Uuid,
    pub term: String,
}

impl std::fmt::Display for ConceptOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.term)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeConceptOption {
    pub id: Option<Uuid>,
    pub label: String,
}

impl std::fmt::Display for AttributeConceptOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteDragItem {
    Concept(Uuid),
    Class(Uuid),
    Enumeration(Uuid),
}

#[derive(Debug, Clone)]
pub struct PaletteDragState {
    pub item: PaletteDragItem,
    pub start_pos: Point,
    pub current_pos: Point,
    pub is_dragging: bool,
}

#[derive(Debug, Clone)]
pub struct RelationDialogState {
    pub from_node: Option<NodeOption>,
    pub to_node: Option<NodeOption>,
    pub kind: RelationKind,
    pub label: String,
    pub source_multiplicity: Option<Multiplicity>,
    pub target_multiplicity: Option<Multiplicity>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuickCreateState {
    pub position: (f32, f32),
    pub preferred_term: String,
    pub definition: String,
    pub belongs_to_domain: BelongsToDomain,
    pub validation_error: Option<String>,
}

impl QuickCreateState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            preferred_term: String::new(),
            definition: String::new(),
            belongs_to_domain: BelongsToDomain::Yes,
            validation_error: None,
        }
    }

    pub fn build_concept(&self) -> Result<Concept, ValidationError> {
        let concept = Concept::new(
            self.preferred_term.trim(),
            self.definition.trim(),
            self.belongs_to_domain.clone(),
        );
        ConceptValidator::validate(&concept)?;
        Ok(concept)
    }
}

pub use crate::ui::concept_editor::ConceptFormField;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelayServerPreset {
    #[default]
    Koyeb,
    InternalOrg,
    LocalDocker,
    Custom,
}

impl RelayServerPreset {
    pub const ALL: [RelayServerPreset; 4] = [
        RelayServerPreset::Koyeb,
        RelayServerPreset::InternalOrg,
        RelayServerPreset::LocalDocker,
        RelayServerPreset::Custom,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Koyeb => "Koyeb Cloud (Standard - Frankfurt)",
            Self::InternalOrg => "Intern Organisation",
            Self::LocalDocker => "Lokal Docker (ws://localhost:8080/ws)",
            Self::Custom => "Brugerdefineret URL...",
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            Self::Koyeb => "wss://kant.koyeb.app/ws",
            Self::InternalOrg => "wss://collab.intern.org/ws",
            Self::LocalDocker => "ws://localhost:8080/ws",
            Self::Custom => "",
        }
    }
}

impl std::fmt::Display for RelayServerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub struct StartSessionModalState {
    pub preset: RelayServerPreset,
    pub custom_url: String,
    pub remember_choice: bool,
    pub ticket: crate::features::collab::SessionTicket,
    pub copied: bool,
}

impl StartSessionModalState {
    pub fn new() -> Self {
        let preset = RelayServerPreset::Koyeb;
        let room = crate::features::collab::RoomId::generate();
        let key = crate::features::collab::CollabKey::generate();
        let ticket = crate::features::collab::SessionTicket::new(preset.default_url(), room, key);
        Self {
            preset,
            custom_url: String::new(),
            remember_choice: false,
            ticket,
            copied: false,
        }
    }

    pub fn current_url(&self) -> &str {
        if (self.preset == RelayServerPreset::Custom
            || self.preset == RelayServerPreset::InternalOrg)
            && !self.custom_url.is_empty()
        {
            &self.custom_url
        } else {
            self.preset.default_url()
        }
    }

    pub fn set_preset(&mut self, preset: RelayServerPreset) {
        self.preset = preset;
        let room = self.ticket.room_id.clone();
        let key = self.ticket.key.clone();
        let target_url = if (preset == RelayServerPreset::Custom
            || preset == RelayServerPreset::InternalOrg)
            && !self.custom_url.is_empty()
        {
            self.custom_url.as_str()
        } else {
            preset.default_url()
        };
        self.ticket = crate::features::collab::SessionTicket::new(target_url, room, key);
    }

    pub fn set_custom_url(&mut self, url: String) {
        self.custom_url = url.clone();
        if self.preset == RelayServerPreset::Custom || self.preset == RelayServerPreset::InternalOrg
        {
            let room = self.ticket.room_id.clone();
            let key = self.ticket.key.clone();
            self.ticket = crate::features::collab::SessionTicket::new(url, room, key);
        }
    }
}

impl Default for StartSessionModalState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct JoinSessionModalState {
    pub token_input: String,
    pub parsed_ticket: Option<crate::features::collab::SessionTicket>,
    pub error_message: Option<String>,
}

impl JoinSessionModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_token_input(&mut self, token: String) {
        let trimmed = token.trim().to_string();
        self.token_input = token;
        if trimmed.is_empty() {
            self.parsed_ticket = None;
            self.error_message = None;
        } else {
            match crate::features::collab::SessionTicket::from_token(&trimmed) {
                Ok(ticket) => {
                    self.parsed_ticket = Some(ticket);
                    self.error_message = None;
                }
                Err(err) => {
                    self.parsed_ticket = None;
                    self.error_message = Some(format!("Ugyldig sessionskode: {err}"));
                }
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.parsed_ticket.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct GuestEndedNoticeModalState {
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    ConceptList,
    ConceptModel,
    InformationModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveStatus {
    Saved { path: String, timestamp: String },
    Saving,
    Unsaved,
    Error(String),
}

impl SaveStatus {
    pub fn is_saved(&self) -> bool {
        matches!(self, SaveStatus::Saved { .. })
    }

    pub fn display_text(&self, current_file_path: Option<&PathBuf>) -> String {
        match self {
            SaveStatus::Saved { path, timestamp } => {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(path);
                format!("💾 Sidst gemt kl. {} • {}", timestamp, filename)
            }
            SaveStatus::Saving => "⏳ Gemmer...".to_string(),
            SaveStatus::Unsaved => {
                if current_file_path.is_some() {
                    "⚠️ Ikke gemte ændringer".to_string()
                } else {
                    "⚠️ Nyt projekt (ikke gemt til disk - tryk Gem)".to_string()
                }
            }
            SaveStatus::Error(msg) => format!("❌ Fejl ved gemning: {}", msg),
        }
    }
}

pub fn current_timestamp() -> String {
    unsafe {
        let mut now: libc::time_t = 0;
        libc::time(&mut now);
        let mut tm: libc::tm = std::mem::zeroed();

        #[cfg(windows)]
        let success = libc::localtime_s(&mut tm, &now) == 0;

        #[cfg(not(windows))]
        let success = !libc::localtime_r(&now, &mut tm).is_null();

        if success {
            format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
        } else {
            "00:00:00".to_string()
        }
    }
}

fn is_running_under_test() -> bool {
    if cfg!(test) || std::env::var("KANT_SUPPRESS_BROWSER").is_ok() {
        return true;
    }
    if std::env::var_os("RUST_TEST_THREADS").is_some() {
        return true;
    }
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy();
        if exe_str.contains("deps/acceptance")
            || exe_str.contains("deps/kant-")
            || exe_str.contains("deps/proptests")
            || exe_str.contains("deps/relay_integration_tests")
        {
            return true;
        }
    }
    false
}

pub fn open_browser(url: &str) -> std::io::Result<()> {
    // Undgå at åbne rigtige systembrowsere under automatiserede testkørsler (f.eks. cargo test)
    if is_running_under_test() {
        crate::features::diagnostics::log_info(
            "browser",
            &format!("Browseråbning undertrykt i testmiljø for URL: {}", url),
        );
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported platform",
        ));
    }
    Ok(())
}

/// Returnerer brugerens hjemmemappe robust på tværs af macOS (HOME), Linux (HOME) og Windows (USERPROFILE / HOMEDRIVE+HOMEPATH).
pub fn user_home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .or_else(|_| {
            let drive = std::env::var("HOMEDRIVE").unwrap_or_default();
            let path = std::env::var("HOMEPATH").unwrap_or_default();
            if !drive.is_empty() && !path.is_empty() {
                Ok(format!("{}{}", drive, path))
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogMode {
    Open,
    SaveAs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    DiagramSvg,
    ConceptListCsv,
    ConceptListXlsx,
    FdaTemplateXlsx,
    ModelReportMarkdown,
    ModelReportHtml,
    ModelXmi,
    ModelSkosTurtle,
    ModelShaclTurtle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    File,
    Export,
    Help,
    Collab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CollabState {
    #[default]
    None,
    Host,
    Guest,
}

impl CollabState {
    pub fn is_guest(&self) -> bool {
        matches!(self, Self::Guest)
    }

    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host)
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    NewProject,

    // Live Kollaborering (Task 027 & Task 028)
    CollabNetworkEventReceived(crate::features::collab::CollabNetworkEvent),
    CollabApplyMutation(Box<crate::features::collab::protocol::ModelMutation>),
    CollabApplySnapshot(Box<crate::features::model::ModelProject>),

    OpenStartSessionModal,
    CloseCollabModal,
    CollabPresetSelected(RelayServerPreset),
    CollabCustomUrlChanged(String),
    CollabToggleRememberPreset(bool),
    CollabCopyTicket,
    CollabStartSession,
    OpenJoinSessionModal,
    CollabJoinTokenChanged(String),
    CollabJoinSession,
    CollabDisconnect,
    CollabGuestDismissEndedModal,

    // Desktop Menulinje & Sidebar Toggle (Task 024)
    ToggleMenu(MenuType),
    CloseMenu,
    ToggleLeftSidebar,

    // Eksportmotor (Task 059 & Task 060)
    ExportActiveDiagramSvgDialog,
    ExportConceptListCsvDialog,
    ExportModelReportMarkdownDialog,
    ExportModelReportHtmlDialog,
    ExportModelXmiDialog,
    ExportModelSkosDialog,
    ExportModelShaclDialog,
    ExportDialogCompleted(crate::ui::file_dialog::DialogResult, ExportKind),

    // Excel Import, Eksport & Skemaguide (Task 043)
    ExportConceptListXlsxDialog,
    ExportFdaTemplateXlsxDialog,
    ImportConceptsExcelDialog,
    ImportConceptsExcelCompleted(crate::ui::file_dialog::DialogResult),
    OpenExcelSchemaGuideModal,
    CloseExcelSchemaGuideModal,
    ConfirmExcelImport,
    DismissExcelImportModal,

    // Modelomslag & Metadata modal (Task 017 & Task 055)
    OpenMetadataModal,
    CloseMetadataModal,
    SaveMetadataModal,
    UpdateMetadataField(MetadataField, String),
    UpdateMetadataStatus(ModelStatus),
    UpdateMetadataApprovalStatus(ApprovalStatus),
    UpdateMetadataScope(ModelScope),

    // Diagnostik & Logning (Task 065)
    OpenDiagnosticsModal,
    CloseDiagnosticsModal,
    CopyDiagnosticsToClipboard,

    // Git & Versionsstyring (Task 047)
    RefreshGitStatus,
    OpenPublishModal,
    ClosePublishModal,
    UpdatePublishMessage(String),
    ConfirmPublish,
    PublishCompleted(PublishAsyncResult),
    InitGitRepository,
    PullModel,

    // Modelhistorik, Element-historik & Konfliktløsning (Task 048)
    OpenModelHistoryModal,
    OpenElementHistoryModal(Uuid),
    CloseModelHistoryModal,
    OpenConflictResolverModal(Vec<crate::features::model::merge::ModelConflict>),
    CloseConflictResolverModal,
    ResolveConflict {
        index: usize,
        choice: ConflictChoice,
    },
    ApplyResolvedConflicts,

    // Git Forbindelse & Kloning (Task 049 & 050)
    OpenGitConnectionModal,
    CloseGitConnectionModal,
    UpdateGitRemoteUrl(String),
    UpdateGitTargetDir(String),
    BrowseGitTargetDir,
    GitTargetDirSelected(crate::ui::file_dialog::DialogResult),
    UpdateGitToken(String),
    UpdateGitAuthorName(String),
    UpdateGitAuthorEmail(String),
    SaveGitConnection,
    RemoveGitRemote,

    OpenGitCloneModal,
    CloseGitCloneModal,
    UpdateGitCloneUrl(String),
    UpdateGitCloneTargetDir(String),
    UpdateGitCloneToken(String),
    BrowseGitCloneTargetDir,
    GitCloneTargetDirSelected(crate::ui::file_dialog::DialogResult),
    ExecuteGitClone,

    // Statusbar / Eksterne links (Task 018)
    OpenModelRules,

    // Begrebsliste CRUD-handlinger
    StartNewConcept,
    EditConcept(Uuid),
    DeleteConcept(Uuid),
    SaveConcept,
    CancelConceptEdit,
    UpdateConceptField(ConceptFormField, String),
    SetConceptDomain(crate::features::concepts::BelongsToDomain),
    SearchQueryChanged(String),
    ToggleShowAllFields,

    // Persistens & Filhåndtering
    SaveProject,
    OpenProjectDialog,
    OpenProjectFolderDialog,
    OpenProjectFolderCompleted(crate::ui::file_dialog::DialogResult),
    OpenRecentProject(PathBuf),
    SaveProjectAsDialog,
    OpenDialogCompleted(crate::ui::file_dialog::DialogResult),
    SaveDialogCompleted(crate::ui::file_dialog::DialogResult),
    OpenInlineFileDialog(FileDialogMode),
    CloseFileDialog,
    FileDialogInputChanged(String),
    ConfirmFileDialog,
    OpenProjectFile(PathBuf),
    SaveProjectToFile(PathBuf),

    // Tastaturnavigation & genveje
    FocusNext,
    FocusPrevious,
    EscapePressed,
    EnterPressed,

    // Graf-handlinger (Fase 3)
    GraphNodeSelected(Option<NodeId>),
    GraphNodesSelected(std::collections::HashSet<NodeId>),
    GraphNodeMoved(NodeId, f32, f32),
    GraphNodesMoved(Vec<(NodeId, f32, f32)>),
    CanvasNodesDragFinished,
    GraphOpenRelationDialog,
    GraphCloseRelationDialog,
    GraphRelationFromChanged(NodeOption),
    GraphRelationToChanged(NodeOption),
    GraphRelationKindChanged(RelationKind),
    GraphRelationLabelChanged(String),
    GraphCreateRelation,
    GraphDeleteRelation(NodeId, NodeId),
    GraphSyncNodes,
    AddConceptToDiagram(Uuid),
    AddConceptToDiagramAt(Uuid, f32, f32),
    RemoveConceptFromDiagram(NodeId),
    ConceptModelSearchChanged(String),

    // Lynoprettelse & Node-redigering på Canvas (Task 006)
    CanvasDoubleClicked(f32, f32),
    CreateConceptAtCenter,
    QuickCreateTermChanged(String),
    QuickCreateDefinitionChanged(String),
    QuickCreateDomainChanged(BelongsToDomain),
    QuickCreateSubmit,
    QuickCreateCancel,
    GraphNodeDoubleClicked(NodeId),

    // Begrebsmodel Edge Interaktivitet & Drag-to-Connect (Task 007 & 014)
    GraphEdgeSelected(Option<(NodeId, NodeId)>),
    GraphEdgeCreated(NodeId, NodeId),
    GraphUpdateEdgeKind(NodeId, NodeId, RelationKind),
    GraphUpdateEdgeLabel(NodeId, NodeId, String),
    GraphToggleEdgeDirected(NodeId, NodeId, bool),
    GraphReverseEdge(NodeId, NodeId),
    GraphDeleteSelected,

    // Canvas ergonomi, zoom, pan & grid (Task 008)
    CanvasViewportChanged(crate::ui::graph_canvas::CanvasViewport),
    ToggleSnapToGrid,
    CanvasZoomIn,
    CanvasZoomOut,
    CanvasResetView,
    CanvasSpacePressed(bool),
    CanvasModifiersChanged(iced::keyboard::Modifiers),

    // Informationsmodel (Task 010 & 011)
    SelectInformationClass(Option<Uuid>),
    CreateInformationClass,
    CreateInformationClassAt(f32, f32),
    CreateInformationClassAtCenter,
    CreateInformationClassFromConcept(ConceptOption),
    UpdateInformationClassName(Uuid, String),
    UpdateInformationClassDescription(Uuid, String),
    SetInformationClassAbstract(Uuid, bool),
    SetInformationClassLocal(Uuid, bool),
    SetInformationClassOriginModel(Uuid, String),
    AddConceptToInformationClass(Uuid, ConceptOption),
    RemoveConceptFromInformationClass(Uuid, Uuid),
    DeleteInformationClass(Uuid),
    AddAttributeToClass(Uuid),
    UpdateAttributeName(Uuid, Uuid, String),
    UpdateAttributeType(Uuid, Uuid, PrimitiveType),
    UpdateAttributeDataType(Uuid, Uuid, InformationDataType),
    UpdateAttributeMultiplicity(Uuid, Uuid, Multiplicity),
    AddConceptToAttribute(Uuid, Uuid, ConceptOption),
    RemoveConceptFromAttribute(Uuid, Uuid, Uuid),
    SetAttributeConcept(Uuid, Uuid, Option<Uuid>),
    DeleteAttribute(Uuid, Uuid),
    InformationClassSearchChanged(String),

    // Enumeration styring (Task 063)
    CreateInformationEnumeration,
    CreateInformationEnumerationAtCenter,
    SelectInformationEnumeration(Option<Uuid>),
    UpdateInformationEnumerationName(Uuid, String),
    UpdateInformationEnumerationDefinition(Uuid, String),
    AddValueToEnumeration(Uuid, String),
    RemoveValueFromEnumeration(Uuid, String),
    DeleteInformationEnumeration(Uuid),
    AddEnumerationToDiagram(Uuid),
    AddEnumerationToDiagramAt(Uuid, f32, f32),
    RemoveEnumerationFromDiagram(Uuid),
    NewEnumValueInputChanged(String),

    // Informationsmodel Canvas & Studio (Task 011 & 014)
    AddClassToDiagram(Uuid),
    AddClassToDiagramAt(Uuid, f32, f32),
    RemoveClassFromDiagram(NodeId),
    UpdateClassNodePosition(NodeId, f32, f32),
    UpdateClassNodesPositions(Vec<(NodeId, f32, f32)>),
    SelectInfoGraphNode(Option<NodeId>),
    SelectInfoGraphNodes(std::collections::HashSet<NodeId>),
    AddClassRelation(NodeId, NodeId, RelationKind, Option<String>),
    DeleteClassRelation(NodeId, NodeId),
    InfoCanvasViewportChanged(crate::ui::graph_canvas::CanvasViewport),
    ToggleInfoSnapToGrid,
    InfoCanvasZoomIn,
    InfoCanvasZoomOut,
    InfoCanvasResetView,
    OpenInfoRelationDialog,
    CloseInfoRelationDialog,
    InfoRelationFromChanged(NodeOption),
    InfoRelationToChanged(NodeOption),
    InfoRelationKindChanged(RelationKind),
    InfoRelationLabelChanged(String),
    InfoRelationSourceMultiplicityChanged(Option<Multiplicity>),
    InfoRelationTargetMultiplicityChanged(Option<Multiplicity>),
    InfoCreateRelation,
    InfoEdgeSelected(Option<(NodeId, NodeId)>),
    InfoEdgeCreated(NodeId, NodeId),
    InfoUpdateEdgeKind(NodeId, NodeId, RelationKind),
    InfoUpdateEdgeLabel(NodeId, NodeId, String),
    InfoUpdateEdgeSourceMultiplicity(NodeId, NodeId, Option<Multiplicity>),
    InfoUpdateEdgeTargetMultiplicity(NodeId, NodeId, Option<Multiplicity>),
    InfoToggleEdgeDirected(NodeId, NodeId, bool),
    InfoReverseEdge(NodeId, NodeId),

    // Palette Drag & Drop (Task 064)
    StartPaletteDrag(PaletteDragItem),
    PaletteDragMoved(Point),
    PaletteDragDropped,
    CanvasDropAt {
        x: f32,
        y: f32,
    },
}

pub struct App {
    project: ModelProject,
    active_tab: Tab,
    editor_state: Option<ConceptEditorState>,
    search_query: String,
    current_file_path: Option<PathBuf>,
    save_status: SaveStatus,
    file_dialog_mode: Option<FileDialogMode>,
    file_dialog_input: String,
    selected_graph_node_id: Option<NodeId>,
    selected_graph_node_ids: std::collections::HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    relation_dialog: Option<RelationDialogState>,
    quick_create: Option<QuickCreateState>,
    is_inline_graph_editing: bool,
    canvas_viewport: crate::ui::graph_canvas::CanvasViewport,
    snap_to_grid: bool,
    is_space_pressed: bool,
    palette_drag: Option<PaletteDragState>,
    last_cursor_pos: Point,
    selected_info_class_id: Option<Uuid>,
    selected_info_enum_id: Option<Uuid>,
    new_enum_value_input: String,
    selected_info_graph_node_id: Option<NodeId>,
    selected_info_graph_node_ids: std::collections::HashSet<NodeId>,
    selected_info_edge: Option<(NodeId, NodeId)>,
    info_class_search: String,
    info_canvas_viewport: crate::ui::graph_canvas::CanvasViewport,
    info_relation_dialog: Option<RelationDialogState>,
    info_snap_to_grid: bool,
    concept_model_search: String,
    metadata_modal: Option<ModelMetadataModalState>,
    active_menu: Option<MenuType>,
    show_left_sidebar: bool,
    collab_state: CollabState,
    collab_channel: Option<crate::features::collab::CollabChannel>,
    collab_key: Option<crate::features::collab::CollabKey>,
    last_node_broadcast: Option<std::time::Instant>,
    start_session_modal: Option<StartSessionModalState>,
    join_session_modal: Option<JoinSessionModalState>,
    guest_ended_notice: Option<GuestEndedNoticeModalState>,
    collab_participant_count: usize,
    collab_connection_status: crate::features::collab::ConnectionStatus,
    collab_seq: std::sync::atomic::AtomicU64,
    collab_last_seen_seq: u64,
    git_sync_status: RepoSyncStatus,
    publish_modal: Option<PublishModalState>,
    history_modal: Option<ModelHistoryModalState>,
    conflict_resolver_modal: Option<ConflictResolverModalState>,
    git_connection_modal: Option<GitConnectionModalState>,
    git_clone_modal: Option<GitCloneModalState>,
    diagnostics_modal: Option<DiagnosticsModalState>,
    excel_schema_guide_open: bool,
    excel_import_preview: Option<ExcelImportPreviewState>,
    recent_store: crate::features::model::recent::RecentStore,
}

impl Default for App {
    fn default() -> Self {
        Self::new_auto_open()
    }
}

impl App {
    pub fn new() -> Self {
        Self::new_with_path(None)
    }

    /// Opretter applikationen og åbner automatisk den senest anvendte model, hvis den findes på disken.
    pub fn new_auto_open() -> Self {
        crate::features::diagnostics::record_startup_milestone("recent_store_lookup");
        let store = crate::features::model::recent::RecentStore::load();
        if let Some(candidate) = store.get_auto_open_candidate() {
            crate::features::diagnostics::record_startup_milestone("auto_opening_candidate");
            Self::new_with_path(Some(candidate))
        } else {
            Self::new_with_path(None)
        }
    }

    pub fn new_with_path(path: Option<PathBuf>) -> Self {
        let mut recent_store = crate::features::model::recent::RecentStore::load();
        let effective_path = path.as_deref().map(ProjectStorage::effective_model_path);
        if let Some(ref ep) = effective_path {
            recent_store.record_opened(ep);
        }

        let _repo_dir = match &effective_path {
            Some(p) => {
                let mut candidate = if p.is_dir() {
                    p.clone()
                } else {
                    match p.parent() {
                        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
                        _ => PathBuf::new(),
                    }
                };
                if candidate.file_name().and_then(|n| n.to_str()) == Some(".kant") {
                    if let Some(parent) = candidate.parent() {
                        candidate = parent.to_path_buf();
                    }
                }
                if GitService::is_safe_model_repo_dir(&candidate) {
                    Some(candidate)
                } else {
                    None
                }
            }
            None => None,
        };
        // Start altid med Uninitialized uden synkrone proceskald for omgående 60fps vinduesvisning
        let git_sync_status = RepoSyncStatus::Uninitialized;

        if let Some(p) = &effective_path {
            if p.exists() {
                if let Ok(mut proj) = ProjectStorage::load(p) {
                    proj.sync_concept_graph();
                    proj.sync_information_graph();
                    crate::features::diagnostics::record_startup_milestone(
                        "model_loaded_and_synced",
                    );
                    return Self {
                        project: proj,
                        active_tab: Tab::ConceptList,
                        editor_state: None,
                        search_query: String::new(),
                        current_file_path: Some(p.clone()),
                        save_status: SaveStatus::Saved {
                            path: p.display().to_string(),
                            timestamp: current_timestamp(),
                        },
                        recent_store,
                        file_dialog_mode: None,
                        file_dialog_input: String::new(),
                        selected_graph_node_id: None,
                        selected_graph_node_ids: std::collections::HashSet::new(),
                        selected_edge: None,
                        relation_dialog: None,
                        quick_create: None,
                        is_inline_graph_editing: false,
                        canvas_viewport: crate::ui::graph_canvas::CanvasViewport::default(),
                        snap_to_grid: true,
                        is_space_pressed: false,
                        palette_drag: None,
                        last_cursor_pos: Point::ORIGIN,
                        selected_info_class_id: None,
                        selected_info_enum_id: None,
                        new_enum_value_input: String::new(),
                        selected_info_graph_node_id: None,
                        selected_info_graph_node_ids: std::collections::HashSet::new(),
                        selected_info_edge: None,
                        info_class_search: String::new(),
                        info_canvas_viewport: crate::ui::graph_canvas::CanvasViewport::default(),
                        info_relation_dialog: None,
                        info_snap_to_grid: true,
                        concept_model_search: String::new(),
                        metadata_modal: None,
                        active_menu: None,
                        show_left_sidebar: true,
                        collab_state: CollabState::None,
                        collab_channel: None,
                        collab_key: None,
                        last_node_broadcast: None,
                        start_session_modal: None,
                        join_session_modal: None,
                        guest_ended_notice: None,
                        collab_participant_count: 1,
                        collab_connection_status:
                            crate::features::collab::ConnectionStatus::Disconnected,
                        collab_seq: std::sync::atomic::AtomicU64::new(0),
                        collab_last_seen_seq: 0,
                        git_sync_status: git_sync_status.clone(),
                        publish_modal: None,
                        history_modal: None,
                        conflict_resolver_modal: None,
                        git_connection_modal: None,
                        git_clone_modal: None,
                        diagnostics_modal: None,
                        excel_schema_guide_open: false,
                        excel_import_preview: None,
                    };
                }
            }
        }

        let save_status = match &path {
            Some(p) => SaveStatus::Saved {
                path: p.display().to_string(),
                timestamp: current_timestamp(),
            },
            None => SaveStatus::Unsaved,
        };

        crate::features::diagnostics::record_startup_milestone("empty_project_ready");
        Self {
            project: ModelProject::default(),
            active_tab: Tab::ConceptList,
            editor_state: None,
            search_query: String::new(),
            current_file_path: path,
            save_status,
            file_dialog_mode: None,
            file_dialog_input: String::new(),
            selected_graph_node_id: None,
            selected_graph_node_ids: std::collections::HashSet::new(),
            selected_edge: None,
            relation_dialog: None,
            quick_create: None,
            is_inline_graph_editing: false,
            canvas_viewport: crate::ui::graph_canvas::CanvasViewport::default(),
            snap_to_grid: true,
            is_space_pressed: false,
            palette_drag: None,
            last_cursor_pos: Point::ORIGIN,
            selected_info_class_id: None,
            selected_info_enum_id: None,
            new_enum_value_input: String::new(),
            selected_info_graph_node_id: None,
            selected_info_graph_node_ids: std::collections::HashSet::new(),
            selected_info_edge: None,
            info_class_search: String::new(),
            info_canvas_viewport: crate::ui::graph_canvas::CanvasViewport::default(),
            info_relation_dialog: None,
            info_snap_to_grid: true,
            concept_model_search: String::new(),
            metadata_modal: None,
            active_menu: None,
            show_left_sidebar: true,
            collab_state: CollabState::None,
            collab_channel: None,
            collab_key: None,
            last_node_broadcast: None,
            start_session_modal: None,
            join_session_modal: None,
            guest_ended_notice: None,
            collab_participant_count: 1,
            collab_connection_status: crate::features::collab::ConnectionStatus::Disconnected,
            collab_seq: std::sync::atomic::AtomicU64::new(0),
            collab_last_seen_seq: 0,
            git_sync_status,
            publish_modal: None,
            history_modal: None,
            conflict_resolver_modal: None,
            git_connection_modal: None,
            git_clone_modal: None,
            diagnostics_modal: None,
            excel_schema_guide_open: false,
            excel_import_preview: None,
            recent_store,
        }
    }

    pub fn recent_store(&self) -> &crate::features::model::recent::RecentStore {
        &self.recent_store
    }

    pub fn git_sync_status(&self) -> &RepoSyncStatus {
        &self.git_sync_status
    }

    pub fn diagnostics_modal(&self) -> Option<&DiagnosticsModalState> {
        self.diagnostics_modal.as_ref()
    }

    pub fn publish_modal(&self) -> Option<&PublishModalState> {
        self.publish_modal.as_ref()
    }

    pub fn history_modal(&self) -> Option<&ModelHistoryModalState> {
        self.history_modal.as_ref()
    }

    pub fn conflict_resolver_modal(&self) -> Option<&ConflictResolverModalState> {
        self.conflict_resolver_modal.as_ref()
    }

    pub fn git_connection_modal(&self) -> Option<&GitConnectionModalState> {
        self.git_connection_modal.as_ref()
    }

    pub fn git_clone_modal(&self) -> Option<&GitCloneModalState> {
        self.git_clone_modal.as_ref()
    }

    pub fn is_excel_schema_guide_open(&self) -> bool {
        self.excel_schema_guide_open
    }

    pub fn is_excel_import_preview_open(&self) -> bool {
        self.excel_import_preview.is_some()
    }

    pub fn set_active_file_path(&mut self, path: Option<PathBuf>) {
        self.current_file_path = path;
    }

    pub fn git_status_badge_text(&self) -> String {
        match &self.git_sync_status {
            RepoSyncStatus::Synced => "Synkroniseret".to_string(),
            RepoSyncStatus::PendingChanges => "Lokale ændringer • Commit og send".to_string(),
            RepoSyncStatus::UnpublishedCommits(n) => {
                if *n == 0 {
                    "Synkroniseret".to_string()
                } else if *n == 1 {
                    "1 ændring klar til server (Push)".to_string()
                } else {
                    format!("{} ændringer klar til server (Push)", n)
                }
            }
            RepoSyncStatus::IncomingCommits(n) => {
                if *n == 1 {
                    "1 ny på server • Hent (Pull)".to_string()
                } else {
                    format!("{} nye på server • Hent (Pull)", n)
                }
            }
            RepoSyncStatus::Diverged { local, remote } => {
                format!("Afvigelse ({} lokal, {} server)", local, remote)
            }
            RepoSyncStatus::Uninitialized => "Lokal model • Kobl til Git".to_string(),
        }
    }

    pub fn current_file_path(&self) -> Option<&PathBuf> {
        self.current_file_path.as_ref()
    }

    pub fn repo_dir(&self) -> Option<PathBuf> {
        let candidate = match &self.current_file_path {
            Some(p) => {
                let mut dir = if p.is_dir() {
                    p.clone()
                } else {
                    match p.parent() {
                        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
                        _ => PathBuf::new(),
                    }
                };
                if dir.file_name().and_then(|n| n.to_str()) == Some(".kant") {
                    if let Some(parent) = dir.parent() {
                        dir = parent.to_path_buf();
                    }
                }
                Some(dir)
            }
            None => None,
        };

        candidate.filter(|dir| GitService::is_safe_model_repo_dir(dir))
    }

    pub fn save_status(&self) -> &SaveStatus {
        &self.save_status
    }

    pub fn footer_status_text(&self) -> String {
        if self.collab_state.is_guest() {
            "👥 Live Session (Guest) - Autosave deaktiveret (Brug 'Gem som...')".to_string()
        } else if self.collab_state.is_host() {
            format!(
                "👑 Live Session (Host) • {}",
                self.save_status
                    .display_text(self.current_file_path.as_ref())
            )
        } else {
            self.save_status
                .display_text(self.current_file_path.as_ref())
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }

    pub fn active_tab(&self) -> Tab {
        self.active_tab
    }

    pub fn metadata_modal(&self) -> Option<&ModelMetadataModalState> {
        self.metadata_modal.as_ref()
    }

    pub fn active_menu(&self) -> Option<MenuType> {
        self.active_menu
    }

    pub fn is_left_sidebar_visible(&self) -> bool {
        self.show_left_sidebar
    }

    pub fn project(&self) -> &ModelProject {
        &self.project
    }

    pub fn project_mut(&mut self) -> &mut ModelProject {
        &mut self.project
    }

    pub fn is_snap_to_grid_enabled(&self) -> bool {
        self.snap_to_grid
    }

    pub fn canvas_zoom(&self) -> f32 {
        self.canvas_viewport.zoom()
    }

    pub fn canvas_viewport(&self) -> crate::ui::graph_canvas::CanvasViewport {
        self.canvas_viewport
    }

    pub fn is_editing_concept(&self) -> bool {
        self.editor_state.is_some()
    }

    pub fn is_file_dialog_open(&self) -> bool {
        self.file_dialog_mode.is_some()
    }

    pub fn is_relation_dialog_open(&self) -> bool {
        self.relation_dialog.is_some()
    }

    pub fn is_quick_create_open(&self) -> bool {
        self.quick_create.is_some()
    }

    pub fn is_node_editing(&self) -> bool {
        self.is_inline_graph_editing && self.editor_state.is_some()
    }

    pub fn selected_graph_node_id(&self) -> Option<NodeId> {
        self.selected_graph_node_id
    }

    pub fn selected_graph_node_ids(&self) -> &std::collections::HashSet<NodeId> {
        &self.selected_graph_node_ids
    }

    pub fn selected_edge(&self) -> Option<(NodeId, NodeId)> {
        self.selected_edge
    }

    pub fn selected_info_class_id(&self) -> Option<Uuid> {
        self.selected_info_class_id
    }

    pub fn selected_info_enum_id(&self) -> Option<Uuid> {
        self.selected_info_enum_id
    }

    pub fn selected_info_graph_node_id(&self) -> Option<NodeId> {
        self.selected_info_graph_node_id
    }

    pub fn selected_info_graph_node_ids(&self) -> &std::collections::HashSet<NodeId> {
        &self.selected_info_graph_node_ids
    }

    pub fn selected_info_edge(&self) -> Option<(NodeId, NodeId)> {
        self.selected_info_edge
    }

    pub fn concept_model_search(&self) -> &str {
        &self.concept_model_search
    }

    pub fn information_model_search(&self) -> &str {
        &self.info_class_search
    }

    pub fn concept_editor(&self) -> Option<&ConceptEditorState> {
        self.editor_state.as_ref()
    }

    pub fn collab_state(&self) -> CollabState {
        self.collab_state
    }

    pub fn set_collab_state(&mut self, state: CollabState) {
        self.collab_state = state;
    }

    pub fn collab_channel(&self) -> Option<&crate::features::collab::CollabChannel> {
        self.collab_channel.as_ref()
    }

    pub fn set_collab_session(
        &mut self,
        channel: crate::features::collab::CollabChannel,
        key: crate::features::collab::CollabKey,
        role: CollabState,
    ) {
        self.collab_channel = Some(channel);
        self.collab_key = Some(key);
        self.collab_state = role;
        self.collab_last_seen_seq = 0;
    }

    pub fn disconnect_collab(&mut self) {
        if let Some(channel) = &self.collab_channel {
            if self.collab_state.is_host() {
                let _ = channel.send_host_left();
            }
            channel.disconnect();
        }
        self.collab_channel = None;
        self.collab_key = None;
        self.collab_state = CollabState::None;
        self.collab_connection_status = crate::features::collab::ConnectionStatus::Disconnected;
        self.collab_last_seen_seq = 0;
    }

    pub fn start_session_modal(&self) -> Option<&StartSessionModalState> {
        self.start_session_modal.as_ref()
    }

    pub fn join_session_modal(&self) -> Option<&JoinSessionModalState> {
        self.join_session_modal.as_ref()
    }

    pub fn guest_ended_notice(&self) -> Option<&GuestEndedNoticeModalState> {
        self.guest_ended_notice.as_ref()
    }

    pub fn collab_participant_count(&self) -> usize {
        self.collab_participant_count
    }

    /// Returnerer den aktuelle sekvensnummer-tæller — primært til test-inspektion.
    pub fn collab_seq_value(&self) -> u64 {
        self.collab_seq.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn reroute_info_graph_edges(&mut self) {
        let routes = {
            let ig = self.project.information_graph();
            let nodes: Vec<crate::features::concept_model::DiagramNode> =
                ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
            let edges: Vec<crate::features::concept_model::DiagramEdge> =
                ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
            crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges)
        };
        for r in &routes {
            self.project.information_graph_mut().update_edge_ports(
                r.from,
                r.to,
                Some(r.from_side),
                Some(r.to_side),
            );
        }
        let port_updates: Vec<_> = {
            let ig = self.project.information_graph();
            routes
                .into_iter()
                .map(|r| {
                    let fc = ig.find_node(r.from).map(|n| n.class_id()).unwrap_or(r.from);
                    let tc = ig.find_node(r.to).map(|n| n.class_id()).unwrap_or(r.to);
                    (fc, tc, r.from_side, r.to_side)
                })
                .collect()
        };
        for (fc, tc, from_side, to_side) in port_updates {
            self.project.information_model_mut().update_relation_ports(
                fc,
                tc,
                Some(from_side),
                Some(to_side),
            );
        }
    }

    pub fn set_collab_participant_count(&mut self, count: usize) {
        let previous = self.collab_participant_count;
        self.collab_participant_count = count;
        // Lag 2-forsvar: vært re-broadcaster snapshot når en ny deltager tilslutter sig,
        // som sikkerhedsnet hvis relay's last_snapshot-cache ikke nåede at modtage
        // snapshot inden gæsten forbandt (timing race ved session-opstart).
        if self.collab_state.is_host() && count > previous {
            self.broadcast_snapshot();
        }
    }

    pub fn collab_connection_status(&self) -> crate::features::collab::ConnectionStatus {
        self.collab_connection_status
    }

    pub fn set_collab_connection_status(
        &mut self,
        status: crate::features::collab::ConnectionStatus,
    ) {
        self.collab_connection_status = status;
    }

    pub fn notify_host_ended_session(&mut self) {
        if self.collab_state.is_guest() {
            self.collab_state = CollabState::None;
            self.collab_channel = None;
            self.collab_key = None;
            self.collab_connection_status = crate::features::collab::ConnectionStatus::Disconnected;
            self.guest_ended_notice = Some(GuestEndedNoticeModalState {
                message: "Værten har afsluttet sessionen. Vil du gemme en kopi af modellen lokalt?"
                    .to_string(),
            });
        }
    }

    pub fn collab_status_summary(&self) -> String {
        match self.collab_state {
            CollabState::None => "Offline".to_string(),
            CollabState::Host => {
                let suffix = if self.collab_participant_count == 1 {
                    "1 deltager".to_string()
                } else {
                    format!("{} deltagere", self.collab_participant_count)
                };
                match self.collab_connection_status {
                    crate::features::collab::ConnectionStatus::Connected => {
                        format!("Live: Vært ({suffix})")
                    }
                    crate::features::collab::ConnectionStatus::Reconnecting => {
                        "Genforbinder...".to_string()
                    }
                    _ => format!("Live: Vært ({})", self.collab_connection_status),
                }
            }
            CollabState::Guest => match self.collab_connection_status {
                crate::features::collab::ConnectionStatus::Connected => {
                    "Live: Gæst (Forbundet til Vært)".to_string()
                }
                crate::features::collab::ConnectionStatus::Reconnecting => {
                    "Genforbinder...".to_string()
                }
                _ => format!("Live: Gæst ({})", self.collab_connection_status),
            },
        }
    }

    pub fn broadcast_mutation(&self, mutation: &crate::features::collab::protocol::ModelMutation) {
        if !self.collab_state.is_active() {
            return;
        }
        if let (Some(channel), Some(key)) = (&self.collab_channel, &self.collab_key) {
            let seq = self
                .collab_seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let envelope = crate::features::collab::protocol::CollabEnvelope::new(
                seq,
                now,
                crate::features::collab::protocol::CollabPayload::Mutation(mutation.clone()),
            );
            if let Ok(json_bytes) = serde_json::to_vec(&envelope) {
                if let Ok(encrypted) = crate::features::collab::crypto::encrypt(key, &json_bytes) {
                    let _ = channel.send_mutation(encrypted);
                }
            }
        }
    }

    pub fn broadcast_snapshot(&self) {
        if !self.collab_state.is_active() {
            return;
        }
        if let (Some(channel), Some(key)) = (&self.collab_channel, &self.collab_key) {
            let seq = self
                .collab_seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let envelope = crate::features::collab::protocol::CollabEnvelope::new(
                seq,
                now,
                crate::features::collab::protocol::CollabPayload::Snapshot(self.project.clone()),
            );
            if let Ok(json_bytes) = serde_json::to_vec(&envelope) {
                if let Ok(encrypted) = crate::features::collab::crypto::encrypt(key, &json_bytes) {
                    let _ = channel.send_snapshot(encrypted);
                }
            }
        }
    }

    pub fn broadcast_node_moved_throttled(&mut self, id: Uuid, x: f32, y: f32) {
        if !self.collab_state.is_active() {
            return;
        }
        let now = std::time::Instant::now();
        let should_send = match self.last_node_broadcast {
            Some(last) => now.duration_since(last) >= std::time::Duration::from_millis(66), // ~15 Hz
            None => true,
        };
        if should_send {
            self.last_node_broadcast = Some(now);
            self.broadcast_mutation(
                &crate::features::collab::protocol::ModelMutation::NodeMoved { id, x, y },
            );
        }
    }

    pub fn apply_mutation(&mut self, mutation: crate::features::collab::protocol::ModelMutation) {
        use crate::features::collab::protocol::ModelMutation::*;
        match mutation {
            ConceptAdded(concept) => {
                let c_id = concept.id();
                if !self.project.concepts().iter().any(|c| c.id() == c_id) {
                    self.project.concepts_mut().push(concept.clone());
                    if !self.project.concept_graph().is_concept_on_diagram(c_id) {
                        self.project.concept_graph_mut().add_node(&concept);
                    }
                }
            }
            ConceptUpdated(concept) => {
                let c_id = concept.id();
                if let Some(existing) = self
                    .project
                    .concepts_mut()
                    .iter_mut()
                    .find(|c| c.id() == c_id)
                {
                    *existing = concept.clone();
                }
                if let Some(node) = self
                    .project
                    .concept_graph_mut()
                    .find_node_by_concept_mut(c_id)
                {
                    node.set_label(concept.preferred_term().to_string());
                }
            }
            ConceptDeleted(c_id) => {
                self.project.concepts_mut().retain(|c| c.id() != c_id);
                if let Some(node) = self.project.concept_graph().find_node_by_concept(c_id) {
                    let node_id = node.id();
                    self.project.concept_graph_mut().remove_node(node_id);
                }
                if self
                    .selected_graph_node_id
                    .is_some_and(|nid| self.project.concept_graph().find_node(nid).is_none())
                {
                    self.selected_graph_node_id = None;
                }
            }
            ConceptDiagramNodeAdded(concept_id) => {
                if !self
                    .project
                    .concept_graph()
                    .is_concept_on_diagram(concept_id)
                {
                    if let Some(concept) = self.project.get_concept(concept_id).cloned() {
                        self.project.concept_graph_mut().add_node(&concept);
                    }
                }
            }
            ConceptDiagramNodeRemoved(concept_id) => {
                if let Some(node) = self
                    .project
                    .concept_graph()
                    .find_node_by_concept(concept_id)
                {
                    let node_id = node.id();
                    self.project.concept_graph_mut().remove_node(node_id);
                }
                if self
                    .selected_graph_node_id
                    .is_some_and(|nid| self.project.concept_graph().find_node(nid).is_none())
                {
                    self.selected_graph_node_id = None;
                }
            }
            InformationClassAdded(class) => {
                let class_id = class.id();
                if !self
                    .project
                    .information_model()
                    .classes()
                    .iter()
                    .any(|c| c.id() == class_id)
                {
                    let attr_count = class.attributes().len();
                    self.project
                        .information_model_mut()
                        .classes_mut()
                        .push(class);
                    if !self
                        .project
                        .information_graph()
                        .is_class_on_diagram(class_id)
                    {
                        self.project
                            .information_graph_mut()
                            .add_node(class_id, attr_count);
                    }
                }
            }
            InformationClassUpdated(class) => {
                let class_id = class.id();
                let attr_count = class.attributes().len();
                if let Some(existing) = self
                    .project
                    .information_model_mut()
                    .classes_mut()
                    .iter_mut()
                    .find(|c| c.id() == class_id)
                {
                    *existing = class;
                }
                self.project
                    .information_graph_mut()
                    .update_class_dimensions(class_id, attr_count);
                let ig = self.project.information_graph_mut();
                let nodes: Vec<crate::features::concept_model::DiagramNode> =
                    ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                let edges: Vec<crate::features::concept_model::DiagramEdge> =
                    ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                for r in routes {
                    ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                }
            }
            InformationClassDeleted(class_id) => {
                self.project.remove_information_class(class_id);
                if self.selected_info_class_id == Some(class_id) {
                    self.selected_info_class_id = None;
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                    }
                }
            }
            InformationEnumerationAdded(e) => {
                let enum_id = e.id();
                if !self
                    .project
                    .information_model()
                    .enumerations()
                    .iter()
                    .any(|x| x.id() == enum_id)
                {
                    self.project.information_model_mut().add_enumeration(e);
                }
            }
            InformationEnumerationUpdated(e) => {
                let enum_id = e.id();
                let val_count = e.values().len();
                if let Some(existing) = self
                    .project
                    .information_model_mut()
                    .get_enumeration_mut(enum_id)
                {
                    *existing = e;
                }
                self.project
                    .information_graph_mut()
                    .update_class_dimensions(enum_id, val_count);
            }
            InformationEnumerationDeleted(enum_id) => {
                self.project.remove_information_enumeration(enum_id);
                if self.selected_info_enum_id == Some(enum_id) {
                    self.selected_info_enum_id = None;
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                    }
                }
            }
            ClassDiagramNodeAdded(class_id) => {
                if !self
                    .project
                    .information_graph()
                    .is_class_on_diagram(class_id)
                {
                    let count =
                        if let Some(c) = self.project.information_model().get_class(class_id) {
                            c.attributes().len()
                        } else if let Some(e) =
                            self.project.information_model().get_enumeration(class_id)
                        {
                            e.values().len()
                        } else {
                            0
                        };
                    self.project
                        .information_graph_mut()
                        .add_node(class_id, count);
                    self.project.sync_information_graph();
                    let ig = self.project.information_graph_mut();
                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }
            }
            ClassDiagramNodeRemoved(class_id) => {
                if let Some(node) = self
                    .project
                    .information_graph()
                    .find_node_by_class(class_id)
                {
                    let node_id = node.id();
                    self.project.information_graph_mut().remove_node(node_id);
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                    }
                }
            }
            RelationAdded(rel) => {
                if self
                    .project
                    .concept_graph()
                    .find_edge(rel.from, rel.to)
                    .is_none()
                {
                    self.project
                        .concept_graph_mut()
                        .add_relation_full(rel.id, rel.from, rel.to, rel.kind, rel.label);
                    if let Some(directed) = rel.directed {
                        self.project
                            .concept_graph_mut()
                            .update_edge_directed(rel.from, rel.to, directed);
                    }
                    let cg = self.project.concept_graph_mut();
                    let nodes = cg.nodes().to_vec();
                    let edges = cg.edges().to_vec();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }
            }
            RelationUpdated(rel) => {
                if let Some(edge) = self
                    .project
                    .concept_graph_mut()
                    .find_edge_mut(rel.from, rel.to)
                {
                    edge.set_kind(rel.kind);
                    edge.set_label(rel.label);
                    if let Some(directed) = rel.directed {
                        edge.set_directed(directed);
                    }
                    let cg = self.project.concept_graph_mut();
                    let nodes = cg.nodes().to_vec();
                    let edges = cg.edges().to_vec();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }
            }
            RelationDeleted { from, to } => {
                self.project.concept_graph_mut().remove_relation(from, to);
                if self.selected_edge == Some((from, to)) || self.selected_edge == Some((to, from))
                {
                    self.selected_edge = None;
                }
            }
            ClassRelationAdded {
                from_class,
                to_class,
                kind,
                label,
                source_multiplicity,
                target_multiplicity,
                directed,
            } => {
                let rel = ClassRelation::with_multiplicities(
                    from_class,
                    to_class,
                    kind,
                    label,
                    None,
                    None,
                    directed,
                    source_multiplicity,
                    target_multiplicity,
                );
                self.project.information_model_mut().add_relation(rel);
                self.project.sync_information_graph();
                let routes = {
                    let ig = self.project.information_graph();
                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges)
                };
                for r in &routes {
                    self.project.information_graph_mut().update_edge_ports(
                        r.from,
                        r.to,
                        Some(r.from_side),
                        Some(r.to_side),
                    );
                }
                let port_updates: Vec<_> = {
                    let ig = self.project.information_graph();
                    routes
                        .into_iter()
                        .map(|r| {
                            let fc = ig.find_node(r.from).map(|n| n.class_id()).unwrap_or(r.from);
                            let tc = ig.find_node(r.to).map(|n| n.class_id()).unwrap_or(r.to);
                            (fc, tc, r.from_side, r.to_side)
                        })
                        .collect()
                };
                for (fc, tc, from_side, to_side) in port_updates {
                    self.project.information_model_mut().update_relation_ports(
                        fc,
                        tc,
                        Some(from_side),
                        Some(to_side),
                    );
                }
            }
            ClassRelationDeleted {
                from_class,
                to_class,
            } => {
                self.project
                    .information_model_mut()
                    .remove_relation(from_class, to_class);
                let ig = self.project.information_graph();
                let from_id = ig
                    .find_node_by_class(from_class)
                    .map(|n| n.id())
                    .unwrap_or(from_class);
                let to_id = ig
                    .find_node_by_class(to_class)
                    .map(|n| n.id())
                    .unwrap_or(to_class);
                self.project
                    .information_graph_mut()
                    .remove_relation(from_id, to_id);
                if self.selected_info_edge == Some((from_id, to_id))
                    || self.selected_info_edge == Some((to_id, from_id))
                {
                    self.selected_info_edge = None;
                }
            }
            NodeMoved { id, x, y } => {
                if let Some(node) = self.project.concept_graph_mut().find_node_mut(id) {
                    node.set_position(x, y);
                    let cg = self.project.concept_graph_mut();
                    let nodes = cg.nodes().to_vec();
                    let edges = cg.edges().to_vec();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                } else if let Some(node) = self
                    .project
                    .concept_graph_mut()
                    .find_node_by_concept_mut(id)
                {
                    node.set_position(x, y);
                    let cg = self.project.concept_graph_mut();
                    let nodes = cg.nodes().to_vec();
                    let edges = cg.edges().to_vec();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                } else if let Some(class_node) =
                    self.project.information_graph_mut().find_node_mut(id)
                {
                    class_node.set_position(x, y);
                    let ig = self.project.information_graph_mut();
                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                } else if let Some(class_node) = self
                    .project
                    .information_graph_mut()
                    .find_node_by_class_mut(id)
                {
                    class_node.set_position(x, y);
                    let ig = self.project.information_graph_mut();
                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }
            }
        }
        self.trigger_autosave();
    }

    pub fn apply_snapshot(&mut self, snapshot: crate::features::model::ModelProject) {
        if self.collab_state.is_host() {
            return;
        }
        self.project = snapshot;
        self.selected_graph_node_id = None;
        self.selected_graph_node_ids.clear();
        self.selected_edge = None;
        self.selected_info_class_id = None;
        self.selected_info_graph_node_id = None;
        self.selected_info_graph_node_ids.clear();
        self.selected_info_edge = None;
        self.trigger_autosave();
    }

    pub fn trigger_autosave(&mut self) {
        if self.collab_state.is_guest() {
            return;
        }
        if let Some(path) = &self.current_file_path {
            let save_start = std::time::Instant::now();
            match ProjectStorage::save(&self.project, path) {
                Ok(()) => {
                    let save_duration = save_start.elapsed();
                    self.save_status = SaveStatus::Saved {
                        path: path.display().to_string(),
                        timestamp: current_timestamp(),
                    };
                    let git_start = std::time::Instant::now();
                    let mut git_checked = false;
                    if let Some(dir) = self.repo_dir() {
                        if !matches!(self.git_sync_status, RepoSyncStatus::PendingChanges) {
                            if let Ok(st) = GitService::get_sync_status(&dir) {
                                self.git_sync_status = st;
                                git_checked = true;
                            }
                        }
                    }
                    let git_duration = if git_checked {
                        git_start.elapsed()
                    } else {
                        std::time::Duration::ZERO
                    };
                    crate::features::diagnostics::log_info(
                        "autosave",
                        &format!(
                            "Autosave gennemført (disk: {:?}, git: {:?})",
                            save_duration, git_duration
                        ),
                    );
                }
                Err(err) => {
                    self.save_status = SaveStatus::Error(err.to_string());
                    crate::features::diagnostics::log_error(
                        "autosave",
                        &format!("Autosave fejlede: {}", err),
                    );
                }
            }
        }
    }

    pub fn filtered_concepts(&self) -> Vec<&Concept> {
        let q = self.search_query.trim().to_lowercase();
        let mut list: Vec<&Concept> = if q.is_empty() {
            self.project.concepts().iter().collect()
        } else {
            self.project
                .concepts()
                .iter()
                .filter(|c| {
                    c.preferred_term().to_lowercase().contains(&q)
                        || c.definition().to_lowercase().contains(&q)
                        || c.accepted_term()
                            .is_some_and(|t| t.to_lowercase().contains(&q))
                        || c.source().is_some_and(|s| s.to_lowercase().contains(&q))
                        || c.legal_source()
                            .is_some_and(|l| l.to_lowercase().contains(&q))
                        || c.identifier()
                            .is_some_and(|i| i.to_lowercase().contains(&q))
                })
                .collect()
        };
        list.sort_by(|a, b| {
            a.preferred_term()
                .to_lowercase()
                .cmp(&b.preferred_term().to_lowercase())
        });
        list
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CollabNetworkEventReceived(event) => {
                match event {
                    crate::features::collab::CollabNetworkEvent::StatusChanged(status) => {
                        self.set_collab_connection_status(status);
                    }
                    crate::features::collab::CollabNetworkEvent::PresenceUpdated(count) => {
                        self.set_collab_participant_count(count);
                    }
                    crate::features::collab::CollabNetworkEvent::HostEndedSession => {
                        self.notify_host_ended_session();
                    }
                    crate::features::collab::CollabNetworkEvent::MessageReceived(bytes) => {
                        if let Some(key) = &self.collab_key {
                            if let Ok(decrypted) =
                                crate::features::collab::crypto::decrypt(key, &bytes)
                            {
                                if let Ok(envelope) = serde_json::from_slice::<
                                    crate::features::collab::protocol::CollabEnvelope,
                                >(&decrypted)
                                {
                                    if envelope.is_newer_than(self.collab_last_seen_seq) {
                                        self.collab_last_seen_seq = envelope.seq;
                                        match envelope.payload {
                                            crate::features::collab::protocol::CollabPayload::Snapshot(proj) => {
                                                self.apply_snapshot(proj);
                                            }
                                            crate::features::collab::protocol::CollabPayload::Mutation(mutation) => {
                                                self.apply_mutation(mutation);
                                            }
                                        }
                                    }
                                } else if let Ok(payload) =
                                    serde_json::from_slice::<
                                        crate::features::collab::protocol::CollabPayload,
                                    >(&decrypted)
                                {
                                    match payload {
                                        crate::features::collab::protocol::CollabPayload::Snapshot(proj) => {
                                            self.apply_snapshot(proj);
                                        }
                                        crate::features::collab::protocol::CollabPayload::Mutation(mutation) => {
                                            self.apply_mutation(mutation);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    crate::features::collab::CollabNetworkEvent::Error(err) => {
                        eprintln!("Kollaborations netværksfejl: {err}");
                    }
                }
            }
            Message::CollabApplyMutation(mutation) => {
                self.apply_mutation(*mutation);
            }
            Message::CollabApplySnapshot(snapshot) => {
                self.apply_snapshot(*snapshot);
            }
            Message::OpenStartSessionModal => {
                if !self.collab_state.is_active() {
                    self.active_menu = None;
                    self.start_session_modal = Some(StartSessionModalState::new());
                }
            }
            Message::CloseCollabModal => {
                self.start_session_modal = None;
                self.join_session_modal = None;
                self.guest_ended_notice = None;
            }
            Message::CollabPresetSelected(preset) => {
                if let Some(modal) = &mut self.start_session_modal {
                    modal.set_preset(preset);
                }
            }
            Message::CollabCustomUrlChanged(url) => {
                if let Some(modal) = &mut self.start_session_modal {
                    modal.set_custom_url(url);
                }
            }
            Message::CollabToggleRememberPreset(rem) => {
                if let Some(modal) = &mut self.start_session_modal {
                    modal.remember_choice = rem;
                }
            }
            Message::CollabCopyTicket => {
                if let Some(modal) = &mut self.start_session_modal {
                    modal.copied = true;
                    let token = modal.ticket.to_token();
                    return iced::clipboard::write(token);
                }
            }
            Message::CollabStartSession => {
                if let Some(modal) = self.start_session_modal.take() {
                    self.active_menu = None;
                    let ticket = modal.ticket;
                    let channel = crate::features::collab::CollabChannel::connect_registered(
                        &ticket.relay_url,
                        &ticket.room_id,
                    );
                    self.set_collab_session(channel, ticket.key, CollabState::Host);
                    self.set_collab_connection_status(
                        crate::features::collab::ConnectionStatus::Connecting,
                    );
                    self.set_collab_participant_count(1);
                    self.broadcast_snapshot();
                }
            }
            Message::OpenJoinSessionModal => {
                if !self.collab_state.is_active() {
                    self.active_menu = None;
                    self.join_session_modal = Some(JoinSessionModalState::new());
                }
            }
            Message::CollabJoinTokenChanged(tok) => {
                if let Some(modal) = &mut self.join_session_modal {
                    modal.set_token_input(tok);
                }
            }
            Message::CollabJoinSession => {
                if let Some(modal) = self.join_session_modal.take() {
                    if let Some(ticket) = modal.parsed_ticket {
                        self.active_menu = None;
                        let channel = crate::features::collab::CollabChannel::connect_registered(
                            &ticket.relay_url,
                            &ticket.room_id,
                        );
                        self.set_collab_session(channel, ticket.key, CollabState::Guest);
                        self.set_collab_connection_status(
                            crate::features::collab::ConnectionStatus::Connecting,
                        );
                    }
                }
            }

            Message::CollabDisconnect => {
                self.active_menu = None;
                self.disconnect_collab();
            }
            Message::CollabGuestDismissEndedModal => {
                self.guest_ended_notice = None;
            }
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                if tab == Tab::ConceptModel {
                    self.project.sync_concept_graph();
                } else if tab == Tab::InformationModel {
                    self.project.sync_information_graph();
                }
            }
            Message::ToggleMenu(menu) => {
                if self.active_menu == Some(menu) {
                    self.active_menu = None;
                } else {
                    self.active_menu = Some(menu);
                }
            }
            Message::CloseMenu => {
                self.active_menu = None;
            }
            Message::ToggleLeftSidebar => {
                self.show_left_sidebar = !self.show_left_sidebar;
            }
            Message::OpenMetadataModal => {
                self.active_menu = None;
                self.metadata_modal = Some(ModelMetadataModalState::from_metadata(
                    self.project.metadata(),
                ));
            }
            Message::CloseMetadataModal => {
                self.metadata_modal = None;
            }
            Message::OpenDiagnosticsModal => {
                self.diagnostics_modal = Some(DiagnosticsModalState::new());
                self.active_menu = None;
            }
            Message::CloseDiagnosticsModal => {
                self.diagnostics_modal = None;
            }
            Message::CopyDiagnosticsToClipboard => {
                let report =
                    crate::features::diagnostics::get_system_diagnostics().formatted_report();
                return iced::clipboard::write(report);
            }
            Message::RefreshGitStatus => {
                if let Some(dir) = self.repo_dir() {
                    if let Ok(st) = GitService::get_sync_status(&dir) {
                        self.git_sync_status = st;
                    }
                } else {
                    self.git_sync_status = RepoSyncStatus::Uninitialized;
                }
            }
            Message::OpenPublishModal => {
                let dir = match self.repo_dir() {
                    Some(d) => d,
                    None => {
                        self.save_status = SaveStatus::Error(
                            "Gem venligst modellen i en dedikeret modelmappe før den kan udgives med Git".into(),
                        );
                        return self.update(Message::SaveProjectAsDialog);
                    }
                };
                let _ = ProjectStorage::save_to_directory(&self.project, &dir);

                let mut preview_events = Vec::new();
                if let Ok(status_out) =
                    GitService::run_git(&dir, &["status", "--porcelain", "-uall", ".kant"])
                {
                    for line in status_out.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        let status_code = line.chars().take(2).collect::<String>();
                        let file_path = line[2..].trim();
                        let action = if status_code.contains('D') {
                            ChangeAction::Deleted
                        } else if status_code.contains('A') || status_code.contains('?') {
                            ChangeAction::Added
                        } else {
                            ChangeAction::Modified
                        };
                        let content = match action {
                            ChangeAction::Deleted => {
                                GitService::run_git(&dir, &["show", &format!("HEAD:{}", file_path)])
                                    .ok()
                            }
                            _ => std::fs::read_to_string(dir.join(file_path)).ok(),
                        };
                        if let Some(ev) = DomainEventMapper::map_file_change(
                            action,
                            file_path,
                            content.as_deref(),
                        ) {
                            preview_events.push(ev);
                        }
                    }
                }

                let summary = DomainEventMapper::generate_commit_summary(&preview_events);
                let default_msg = if summary.trim().is_empty() {
                    "Opdatering af model".to_string()
                } else {
                    summary
                };
                self.publish_modal = Some(PublishModalState::new(default_msg, preview_events));
            }
            Message::ClosePublishModal => {
                self.publish_modal = None;
            }
            Message::UpdatePublishMessage(msg) => {
                if let Some(modal) = &mut self.publish_modal {
                    modal.message = msg;
                }
            }
            Message::ConfirmPublish => {
                fn execute_publish_workflow(
                    dir: &std::path::Path,
                    project: &crate::features::model::ModelProject,
                    message: &str,
                ) -> PublishAsyncResult {
                    let has_pending =
                        GitService::run_git(dir, &["status", "--porcelain", "-uall", ".kant"])
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false);

                    let commit_result = if has_pending {
                        GitService::publish_model(dir, project, message)
                    } else {
                        Ok("clean".to_string())
                    };

                    if let Err(e) = commit_result {
                        return PublishAsyncResult::Error(format!(
                            "Udgivelse af model fejlede: {}",
                            e
                        ));
                    }

                    if let Ok(Some(_)) = GitService::get_remote_url(dir) {
                        match GitService::pull_model(dir, "origin", "main") {
                            Ok(PullResult::Merged { conflicts }) if !conflicts.is_empty() => {
                                let sync_status = GitService::get_sync_status(dir)
                                    .unwrap_or(RepoSyncStatus::Uninitialized);
                                return PublishAsyncResult::Conflict {
                                    conflicts,
                                    sync_status,
                                };
                            }
                            Ok(_) => {}
                            Err(_) => {
                                // Hvis remote fx er tom eller branch ikke findes endnu, fortsætter vi til push
                            }
                        }

                        if let Err(e) = GitService::push_model(dir, "origin") {
                            return PublishAsyncResult::Error(format!("Git push fejl: {}", e));
                        }
                    }

                    let sync_status =
                        GitService::get_sync_status(dir).unwrap_or(RepoSyncStatus::Synced);
                    PublishAsyncResult::Success { sync_status }
                }

                let dir_opt = self.repo_dir();
                if let Some(modal) = &mut self.publish_modal {
                    let dir = match dir_opt {
                        Some(d) => d,
                        None => {
                            modal.error =
                                Some("Modellen er ikke gemt i en dedikeret modelmappe".into());
                            return Task::none();
                        }
                    };
                    modal.is_publishing = true;
                    modal.error = None;

                    let dir_buf = dir.clone();
                    let project_clone = self.project.clone();
                    let msg_clone = modal.message.clone();

                    if tokio::runtime::Handle::try_current().is_ok() {
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    execute_publish_workflow(&dir_buf, &project_clone, &msg_clone)
                                })
                                .await
                                .unwrap_or_else(|e| PublishAsyncResult::Error(e.to_string()))
                            },
                            Message::PublishCompleted,
                        );
                    } else {
                        // Kør synkront hvis vi er i en testkontekst uden aktiv tokio runtime
                        let res = execute_publish_workflow(&dir_buf, &project_clone, &msg_clone);
                        return self.update(Message::PublishCompleted(res));
                    }
                }
            }
            Message::PublishCompleted(res) => match res {
                PublishAsyncResult::Success { sync_status } => {
                    self.publish_modal = None;
                    self.git_sync_status = sync_status;
                }
                PublishAsyncResult::Conflict {
                    conflicts,
                    sync_status,
                } => {
                    self.publish_modal = None;
                    if let Some(dir) = self.repo_dir() {
                        if let Ok(p) = ProjectStorage::load(&dir) {
                            self.project = p;
                        }
                    }
                    self.conflict_resolver_modal = Some(ConflictResolverModalState::new(conflicts));
                    self.git_sync_status = sync_status;
                }
                PublishAsyncResult::Error(err) => {
                    if let Some(modal) = &mut self.publish_modal {
                        modal.is_publishing = false;
                        modal.error = Some(err);
                    }
                }
            },
            Message::InitGitRepository => {
                if let Some(dir) = self.repo_dir() {
                    let _ = GitService::init_repository(&dir);
                    if let Ok(st) = GitService::get_sync_status(&dir) {
                        self.git_sync_status = st;
                    }
                } else {
                    self.save_status = SaveStatus::Error(
                        "Gem venligst modellen i en dedikeret modelmappe før versionsstyring aktiveres".into(),
                    );
                    return self.update(Message::SaveProjectAsDialog);
                }
            }
            Message::PullModel => {
                if let Some(dir) = self.repo_dir() {
                    match GitService::pull_model(&dir, "origin", "main") {
                        Ok(PullResult::Merged { conflicts }) if !conflicts.is_empty() => {
                            if let Ok(p) = ProjectStorage::load(&dir) {
                                self.project = p;
                            }
                            self.conflict_resolver_modal =
                                Some(ConflictResolverModalState::new(conflicts));
                        }
                        Ok(_) => {
                            if let Ok(p) = ProjectStorage::load(&dir) {
                                self.project = p;
                            }
                        }
                        Err(e) => {
                            eprintln!("Fejl ved hentning af seneste model: {}", e);
                        }
                    }
                    if let Ok(st) = GitService::get_sync_status(&dir) {
                        self.git_sync_status = st;
                    }
                }
            }
            Message::OpenModelHistoryModal => {
                if let Some(dir) = self.repo_dir() {
                    let commits = GitService::get_commit_history(&dir, 50).unwrap_or_default();
                    self.history_modal = Some(ModelHistoryModalState::new(None, commits));
                }
            }
            Message::OpenElementHistoryModal(id) => {
                if let Some(dir) = self.repo_dir() {
                    let commits = GitService::get_element_history(&dir, id).unwrap_or_default();
                    self.history_modal = Some(ModelHistoryModalState::new(Some(id), commits));
                }
            }
            Message::CloseModelHistoryModal => {
                self.history_modal = None;
            }
            Message::OpenConflictResolverModal(conflicts) => {
                self.conflict_resolver_modal = Some(ConflictResolverModalState::new(conflicts));
            }
            Message::CloseConflictResolverModal => {
                self.conflict_resolver_modal = None;
            }
            Message::ResolveConflict { index, choice } => {
                if let Some(modal) = &mut self.conflict_resolver_modal {
                    modal.resolved_choices.insert(index, choice);
                }
            }
            Message::ApplyResolvedConflicts => {
                if let Some(modal) = self.conflict_resolver_modal.take() {
                    for (idx, conflict) in modal.conflicts.iter().enumerate() {
                        let choice = modal
                            .resolved_choices
                            .get(&idx)
                            .copied()
                            .unwrap_or(ConflictChoice::Ours);
                        let winning_value = match choice {
                            ConflictChoice::Ours => &conflict.our_value,
                            ConflictChoice::Theirs => &conflict.their_value,
                        };
                        match conflict.entity_kind {
                            crate::features::model::merge::ConflictEntityKind::Metadata => {
                                let meta = self.project.metadata_mut();
                                match conflict.field_name.as_str() {
                                    "name" => meta.set_name(winning_value),
                                    "description" => meta.set_description(winning_value),
                                    "uri" => meta.set_uri(winning_value),
                                    "responsible_org" => meta.set_responsible_org(winning_value),
                                    _ => {}
                                }
                            }
                            crate::features::model::merge::ConflictEntityKind::Concept(id) => {
                                if let Some(mut concept) = self.project.get_concept(id).cloned() {
                                    match conflict.field_name.as_str() {
                                        "preferred_term" => {
                                            concept.set_preferred_term(winning_value)
                                        }
                                        "definition" => concept.set_definition(winning_value),
                                        _ => {}
                                    }
                                    let _ = self.project.update_concept(concept);
                                }
                            }
                            crate::features::model::merge::ConflictEntityKind::Class(id) => {
                                if let Some(class) =
                                    self.project.information_model_mut().get_class_mut(id)
                                {
                                    match conflict.field_name.as_str() {
                                        "name" => class.set_name(winning_value),
                                        "description" => {
                                            let desc = if winning_value.is_empty() {
                                                None
                                            } else {
                                                Some(winning_value.clone())
                                            };
                                            class.set_description(desc);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.save_status = SaveStatus::Unsaved;
                    if let Some(dir) = self.repo_dir() {
                        let _ = GitService::publish_model(
                            &dir,
                            &self.project,
                            "Løst modelfletningskonflikter",
                        );
                        let _ = GitService::push_model(&dir, "origin");
                        if let Ok(st) = GitService::get_sync_status(&dir) {
                            self.git_sync_status = st;
                        }
                    }
                }
            }

            Message::OpenGitConnectionModal => {
                self.active_menu = None;
                let dir_opt = self.repo_dir();
                let (raw_url, (name, email), target_dir) = if let Some(dir) = &dir_opt {
                    let u = GitService::get_remote_url(dir)
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                    let id = GitService::get_user_identity(dir)
                        .ok()
                        .unwrap_or((None, None));
                    (u, id, dir.display().to_string())
                } else {
                    (String::new(), (None, None), String::new())
                };
                let (url, token) = GitService::parse_remote_url(&raw_url);
                let has_target = !target_dir.is_empty();

                self.git_connection_modal = Some(GitConnectionModalState {
                    remote_url: url,
                    target_dir,
                    custom_target_dir: has_target,
                    token: token.unwrap_or_default(),
                    author_name: name.unwrap_or_default(),
                    author_email: email.unwrap_or_default(),
                    error_message: None,
                });
            }

            Message::CloseGitConnectionModal => {
                self.git_connection_modal = None;
            }

            Message::UpdateGitRemoteUrl(url) => {
                if let Some(m) = &mut self.git_connection_modal {
                    if !m.custom_target_dir {
                        if let Some(repo_name) = GitService::parse_repo_name(&url) {
                            let base_dir = user_home_dir();
                            let suggested = base_dir.join("Kant Modeller").join(&repo_name);
                            m.target_dir = suggested.display().to_string();
                        }
                    }
                    m.remote_url = url;
                    m.error_message = None;
                }
            }

            Message::UpdateGitTargetDir(dir) => {
                if let Some(m) = &mut self.git_connection_modal {
                    m.custom_target_dir = !dir.trim().is_empty();
                    m.target_dir = dir;
                    m.error_message = None;
                }
            }

            Message::BrowseGitTargetDir => {
                return Task::perform(
                    async move { crate::ui::file_dialog::pick_folder() },
                    Message::GitTargetDirSelected,
                );
            }

            Message::GitTargetDirSelected(result) => {
                if let Some(m) = &mut self.git_connection_modal {
                    if let crate::ui::file_dialog::DialogResult::Selected(path) = result {
                        let final_path =
                            if let Some(repo_name) = GitService::parse_repo_name(&m.remote_url) {
                                let is_non_empty_dir = path.is_dir()
                                    && path.read_dir().is_ok_and(|mut i| i.next().is_some());
                                if is_non_empty_dir
                                    && path.file_name().and_then(|n| n.to_str()) != Some(&repo_name)
                                {
                                    path.join(repo_name)
                                } else {
                                    path
                                }
                            } else {
                                path
                            };
                        m.target_dir = final_path.display().to_string();
                        m.custom_target_dir = true;
                        m.error_message = None;
                    }
                }
            }

            Message::UpdateGitToken(token) => {
                if let Some(m) = &mut self.git_connection_modal {
                    m.token = token;
                    m.error_message = None;
                }
            }

            Message::UpdateGitAuthorName(name) => {
                if let Some(m) = &mut self.git_connection_modal {
                    m.author_name = name;
                    m.error_message = None;
                }
            }

            Message::UpdateGitAuthorEmail(email) => {
                if let Some(m) = &mut self.git_connection_modal {
                    m.author_email = email;
                    m.error_message = None;
                }
            }

            Message::SaveGitConnection => {
                if let Some(mut state) = self.git_connection_modal.take() {
                    let target_str = state.target_dir.trim().to_string();
                    let dir = if !target_str.is_empty() {
                        PathBuf::from(target_str)
                    } else if let Some(d) = self.repo_dir() {
                        d
                    } else {
                        state.error_message = Some(
                            "Vælg eller angiv en lokal målmappe, hvor modellen skal placeres på computeren".into(),
                        );
                        self.git_connection_modal = Some(state);
                        return Task::none();
                    };

                    if !GitService::is_safe_model_repo_dir(&dir) {
                        state.error_message = Some(
                            "Den valgte mappe indeholder softwarekildekode (Cargo.toml). Vælg en dedikeret modelmappe (f.eks. i 'Kant Modeller').".into(),
                        );
                        self.git_connection_modal = Some(state);
                        return Task::none();
                    }

                    let url = state.remote_url.trim().to_string();
                    let token = state.token.trim().to_string();
                    let name = state.author_name.trim().to_string();
                    let email = state.author_email.trim().to_string();

                    // Sørg for at forældremappen eksisterer
                    if let Some(parent) = dir.parent() {
                        if !parent.as_os_str().is_empty() && !parent.exists() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                    }

                    // Tjek om mappen er et eksisterende git-repo
                    let is_inside_repo =
                        GitService::run_git(&dir, &["rev-parse", "--is-inside-work-tree"]).is_ok();

                    if !is_inside_repo {
                        if !url.is_empty() {
                            let auth_url = if !token.is_empty() {
                                GitService::build_authenticated_url(&url, &token)
                            } else {
                                url.clone()
                            };

                            let is_empty_dir = !dir.exists()
                                || dir.read_dir().map_or(true, |mut i| i.next().is_none());
                            if is_empty_dir {
                                match GitService::clone_repository(&auth_url, &dir) {
                                    Ok(_) => {
                                        let mut project = match ProjectStorage::load(&dir) {
                                            Ok(p) => p,
                                            Err(_) => {
                                                let mut fresh = ModelProject::default();
                                                if let Some(repo_name) =
                                                    GitService::parse_repo_name(&url)
                                                {
                                                    fresh.metadata_mut().set_name(repo_name);
                                                }
                                                let _ =
                                                    ProjectStorage::save_to_directory(&fresh, &dir);
                                                fresh
                                            }
                                        };
                                        if let Some(repo_name) = GitService::parse_repo_name(&url) {
                                            if project.metadata().name() == "Nyt FDA Modelprojekt"
                                                || project.metadata().name().trim().is_empty()
                                            {
                                                project.metadata_mut().set_name(repo_name);
                                            }
                                        }
                                        project.sync_concept_graph();
                                        project.sync_information_graph();
                                        self.project = project;
                                    }
                                    Err(e) => {
                                        state.error_message =
                                            Some(format!("Kloning fejlede: {}", e));
                                        self.git_connection_modal = Some(state);
                                        return Task::none();
                                    }
                                }
                            } else {
                                if let Err(e) = GitService::init_repository(&dir) {
                                    state.error_message =
                                        Some(format!("Kunne ikke initialisere Git: {}", e));
                                    self.git_connection_modal = Some(state);
                                    return Task::none();
                                }
                                let _ = ProjectStorage::save_to_directory(&self.project, &dir);
                            }
                        } else {
                            if let Err(e) = GitService::init_repository(&dir) {
                                state.error_message =
                                    Some(format!("Kunne ikke initialisere Git: {}", e));
                                self.git_connection_modal = Some(state);
                                return Task::none();
                            }
                            let _ = ProjectStorage::save_to_directory(&self.project, &dir);
                        }
                    } else {
                        let _ = ProjectStorage::save_to_directory(&self.project, &dir);
                    }

                    if !url.is_empty() {
                        let final_url = GitService::build_authenticated_url(&url, &token);
                        if let Err(e) = GitService::set_remote_url(&dir, &final_url) {
                            state.error_message =
                                Some(format!("Kunne ikke gemme remote URL: {}", e));
                            self.git_connection_modal = Some(state);
                            return Task::none();
                        }
                    }

                    if !name.is_empty() || !email.is_empty() {
                        if let Err(e) = GitService::set_user_identity(&dir, &name, &email) {
                            state.error_message =
                                Some(format!("Kunne ikke gemme forfatteridentitet: {}", e));
                            self.git_connection_modal = Some(state);
                            return Task::none();
                        }
                    }

                    self.current_file_path = Some(dir.clone());
                    self.recent_store.record_opened(&dir);
                    self.save_status = SaveStatus::Saved {
                        path: dir.display().to_string(),
                        timestamp: current_timestamp(),
                    };
                    self.git_connection_modal = None;
                    let _ = self.update(Message::RefreshGitStatus);
                }
            }

            Message::RemoveGitRemote => {
                if let Some(dir) = self.repo_dir() {
                    let _ = GitService::remove_remote(&dir);
                    let _ = self.update(Message::RefreshGitStatus);
                }
                if let Some(m) = &mut self.git_connection_modal {
                    m.remote_url.clear();
                    m.token.clear();
                }
            }

            Message::OpenGitCloneModal => {
                self.active_menu = None;
                self.git_clone_modal = Some(GitCloneModalState {
                    remote_url: String::new(),
                    target_dir: String::new(),
                    custom_target_dir: false,
                    token: String::new(),
                    is_cloning: false,
                    error_message: None,
                });
            }

            Message::CloseGitCloneModal => {
                self.git_clone_modal = None;
            }

            Message::UpdateGitCloneUrl(url) => {
                if let Some(m) = &mut self.git_clone_modal {
                    if !m.custom_target_dir {
                        if let Some(repo_name) = GitService::parse_repo_name(&url) {
                            let base_dir = user_home_dir();
                            let suggested = base_dir.join("Kant Modeller").join(&repo_name);
                            m.target_dir = suggested.display().to_string();
                        }
                    }
                    m.remote_url = url;
                    m.error_message = None;
                }
            }

            Message::UpdateGitCloneTargetDir(dir) => {
                if let Some(m) = &mut self.git_clone_modal {
                    m.custom_target_dir = !dir.trim().is_empty();
                    m.target_dir = dir;
                    m.error_message = None;
                }
            }

            Message::UpdateGitCloneToken(token) => {
                if let Some(m) = &mut self.git_clone_modal {
                    m.token = token;
                    m.error_message = None;
                }
            }

            Message::BrowseGitCloneTargetDir => {
                return Task::perform(
                    async move { crate::ui::file_dialog::pick_folder() },
                    Message::GitCloneTargetDirSelected,
                );
            }

            Message::GitCloneTargetDirSelected(result) => {
                if let Some(m) = &mut self.git_clone_modal {
                    if let crate::ui::file_dialog::DialogResult::Selected(path) = result {
                        let final_path =
                            if let Some(repo_name) = GitService::parse_repo_name(&m.remote_url) {
                                let is_non_empty_dir = path.is_dir()
                                    && path.read_dir().is_ok_and(|mut i| i.next().is_some());
                                if is_non_empty_dir
                                    && path.file_name().and_then(|n| n.to_str()) != Some(&repo_name)
                                {
                                    path.join(repo_name)
                                } else {
                                    path
                                }
                            } else {
                                path
                            };
                        m.target_dir = final_path.display().to_string();
                        m.custom_target_dir = true;
                        m.error_message = None;
                    }
                }
            }

            Message::ExecuteGitClone => {
                if let Some(state) = self.git_clone_modal.as_mut() {
                    let mut url = state.remote_url.trim().to_string();
                    let target = state.target_dir.trim().to_string();
                    let token = state.token.trim().to_string();
                    if url.is_empty() || target.is_empty() {
                        state.error_message =
                            Some("Både Git URL og destinationsmappe skal udfyldes".into());
                    } else {
                        if !token.is_empty() {
                            url = GitService::build_authenticated_url(&url, &token);
                        }
                        let target_path = PathBuf::from(&target);
                        match GitService::clone_repository(&url, &target_path) {
                            Ok(_) => {
                                let mut project = match ProjectStorage::load(&target_path) {
                                    Ok(p) => p,
                                    Err(_) => {
                                        // Tomt repository klonet: Initialiser frisk default FDA model
                                        let mut fresh = ModelProject::default();
                                        if let Some(repo_name) = GitService::parse_repo_name(&url) {
                                            fresh.metadata_mut().set_name(repo_name);
                                        }
                                        let _ =
                                            ProjectStorage::save_to_directory(&fresh, &target_path);
                                        fresh
                                    }
                                };

                                // Sæt projektnavn fra repository-navnet hvis det ikke allerede er unikt defineret
                                if let Some(repo_name) = GitService::parse_repo_name(&url) {
                                    if project.metadata().name() == "Nyt FDA Modelprojekt"
                                        || project.metadata().name().trim().is_empty()
                                    {
                                        project.metadata_mut().set_name(repo_name);
                                    }
                                }

                                project.sync_concept_graph();
                                project.sync_information_graph();

                                // Fuldstændig ren tavle for det klonede projekt (AC4)
                                self.project = project;
                                self.current_file_path = Some(target_path.clone());
                                self.save_status = SaveStatus::Saved {
                                    path: target_path.display().to_string(),
                                    timestamp: current_timestamp(),
                                };
                                self.editor_state = None;
                                self.is_inline_graph_editing = false;
                                self.selected_graph_node_id = None;
                                self.selected_edge = None;
                                self.relation_dialog = None;
                                self.quick_create = None;
                                self.selected_info_class_id = None;
                                self.selected_info_graph_node_id = None;
                                self.selected_info_edge = None;
                                self.search_query.clear();
                                self.info_class_search.clear();
                                self.active_tab = Tab::ConceptList;
                                self.git_clone_modal = None;
                                self.recent_store.record_opened(&target_path);
                                let _ = self.update(Message::RefreshGitStatus);
                            }
                            Err(e) => {
                                if let Some(m) = &mut self.git_clone_modal {
                                    m.error_message = Some(format!("{}", e));
                                }
                            }
                        }
                    }
                }
            }

            Message::SaveMetadataModal => {
                if let Some(state) = self.metadata_modal.take() {
                    let meta = self.project.metadata_mut();
                    let name = if state.name.trim().is_empty() {
                        "Nyt FDA Modelprojekt".to_string()
                    } else {
                        state.name
                    };
                    let version = if state.version.trim().is_empty() {
                        "0.1.0".to_string()
                    } else {
                        state.version
                    };
                    meta.set_name(name);
                    meta.set_description(state.description);
                    meta.set_model_status(state.status);
                    meta.set_approval_status(state.approval_status);
                    meta.set_approved_by(if state.approved_by.trim().is_empty() {
                        None
                    } else {
                        Some(state.approved_by.trim().to_string())
                    });
                    meta.set_model_scope(state.model_scope);
                    meta.set_language(if state.language.trim().is_empty() {
                        "da".to_string()
                    } else {
                        state.language.trim().to_string()
                    });
                    meta.set_domain_area(state.domain_area);
                    meta.set_responsible_org(state.responsible_org);
                    meta.set_uri(state.uri);
                    meta.set_version(version);
                    meta.set_version_notes(if state.version_notes.trim().is_empty() {
                        None
                    } else {
                        Some(state.version_notes.trim().to_string())
                    });
                    meta.set_source(if state.source.trim().is_empty() {
                        None
                    } else {
                        Some(state.source.trim().to_string())
                    });
                    meta.set_was_derived_from(if state.was_derived_from.trim().is_empty() {
                        None
                    } else {
                        Some(state.was_derived_from.trim().to_string())
                    });
                    let legal_sources: Vec<String> = state
                        .legal_sources
                        .split(&[',', '\n'][..])
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    meta.set_legal_sources(legal_sources);
                    if !state.date_modified.trim().is_empty() {
                        meta.set_date_modified(state.date_modified.trim());
                    }
                    self.save_status = SaveStatus::Unsaved;
                }
            }
            Message::UpdateMetadataField(field, val) => {
                if let Some(modal) = &mut self.metadata_modal {
                    match field {
                        MetadataField::Name => modal.name = val,
                        MetadataField::Description => modal.description = val,
                        MetadataField::DomainArea => modal.domain_area = val,
                        MetadataField::ResponsibleOrg => modal.responsible_org = val,
                        MetadataField::Uri => modal.uri = val,
                        MetadataField::Version => modal.version = val,
                        MetadataField::ApprovedBy => modal.approved_by = val,
                        MetadataField::Language => modal.language = val,
                        MetadataField::LegalSources => modal.legal_sources = val,
                        MetadataField::Source => modal.source = val,
                        MetadataField::WasDerivedFrom => modal.was_derived_from = val,
                        MetadataField::VersionNotes => modal.version_notes = val,
                        MetadataField::DateModified => modal.date_modified = val,
                    }
                }
            }
            Message::UpdateMetadataStatus(status) => {
                if let Some(modal) = &mut self.metadata_modal {
                    modal.status = status;
                }
            }
            Message::UpdateMetadataApprovalStatus(status) => {
                if let Some(modal) = &mut self.metadata_modal {
                    modal.approval_status = status;
                }
            }
            Message::UpdateMetadataScope(scope) => {
                if let Some(modal) = &mut self.metadata_modal {
                    modal.model_scope = scope;
                }
            }
            Message::NewProject => {
                self.active_menu = None;
                self.project = ModelProject::default();
                self.active_tab = Tab::ConceptList;
                self.metadata_modal = Some(ModelMetadataModalState::from_metadata(
                    self.project.metadata(),
                ));
                self.editor_state = None;
                self.is_inline_graph_editing = false;
                self.search_query.clear();
                self.current_file_path = None;
                self.save_status = SaveStatus::Unsaved;
                self.git_sync_status = RepoSyncStatus::Uninitialized;
                self.selected_graph_node_id = None;
                self.selected_edge = None;
                self.relation_dialog = None;
                self.quick_create = None;
                self.selected_info_class_id = None;
                self.selected_info_graph_node_id = None;
                self.selected_info_edge = None;
                self.info_class_search.clear();
            }
            Message::StartNewConcept => {
                self.editor_state = Some(ConceptEditorState::new_empty());
                self.is_inline_graph_editing = false;
                return operation::focus("preferred_term_input");
            }
            Message::EditConcept(id) => {
                if let Some(concept) = self.project.get_concept(id) {
                    self.editor_state = Some(ConceptEditorState::from_concept(concept));
                    self.is_inline_graph_editing = false;
                    self.active_tab = Tab::ConceptList;
                    return operation::focus("preferred_term_input");
                }
            }
            Message::DeleteConcept(id) => {
                self.project.remove_concept(id);
                if let Some(editor) = &self.editor_state {
                    if editor.editing_id == Some(id) {
                        self.editor_state = None;
                        self.is_inline_graph_editing = false;
                    }
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ConceptDeleted(id),
                );
                self.trigger_autosave();
            }
            Message::SaveConcept => {
                if let Some(editor) = &mut self.editor_state {
                    match editor.build_concept() {
                        Ok(concept) => {
                            let concept_id = concept.id();
                            let pref_term = concept.preferred_term().to_string();
                            let is_node_edit = self.is_inline_graph_editing;
                            let is_edit = editor.editing_id.is_some();

                            let result = if is_edit {
                                self.project.update_concept(concept.clone())
                            } else {
                                self.project.add_concept(concept.clone()).map(|_| ())
                            };

                            match result {
                                Ok(()) => {
                                    if is_edit {
                                        self.broadcast_mutation(
                                            &crate::features::collab::protocol::ModelMutation::ConceptUpdated(
                                                concept,
                                            ),
                                        );
                                    } else {
                                        self.broadcast_mutation(
                                            &crate::features::collab::protocol::ModelMutation::ConceptAdded(
                                                concept,
                                            ),
                                        );
                                    }
                                    self.editor_state = None;
                                    self.is_inline_graph_editing = false;
                                    if is_node_edit {
                                        if let Some(node) = self
                                            .project
                                            .concept_graph_mut()
                                            .find_node_by_concept_mut(concept_id)
                                        {
                                            node.set_label(pref_term);
                                        }
                                    }
                                    self.trigger_autosave();
                                }
                                Err(err) => {
                                    editor.validation_error = Some(err.to_string());
                                }
                            }
                        }
                        Err(err) => {
                            editor.validation_error = Some(err.to_string());
                        }
                    }
                }
            }
            Message::CancelConceptEdit => {
                self.editor_state = None;
                self.is_inline_graph_editing = false;
            }
            Message::UpdateConceptField(field, value) => {
                if let Some(editor) = &mut self.editor_state {
                    editor.update_field(field, value);
                }
            }
            Message::SetConceptDomain(domain) => {
                if let Some(editor) = &mut self.editor_state {
                    editor.set_domain(domain);
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::ToggleShowAllFields => {
                if let Some(editor) = &mut self.editor_state {
                    editor.show_supplementary = !editor.show_supplementary;
                }
            }

            // Persistens & Filhåndtering
            Message::SaveProject => {
                self.active_menu = None;
                if self.collab_state.is_guest() {
                    return self.update(Message::SaveProjectAsDialog);
                }
                if self.current_file_path.is_some() {
                    self.trigger_autosave();
                } else {
                    return self.update(Message::SaveProjectAsDialog);
                }
            }
            Message::OpenProjectDialog => {
                self.active_menu = None;
                return Task::perform(
                    async { crate::ui::file_dialog::pick_file_to_open() },
                    Message::OpenDialogCompleted,
                );
            }
            Message::OpenProjectFolderDialog => {
                self.active_menu = None;
                return Task::perform(
                    async { crate::ui::file_dialog::pick_folder() },
                    Message::OpenProjectFolderCompleted,
                );
            }
            Message::OpenProjectFolderCompleted(res) => {
                if let crate::ui::file_dialog::DialogResult::Selected(path) = res {
                    return self.update(Message::OpenProjectFile(path));
                }
            }
            Message::OpenRecentProject(path) => {
                self.active_menu = None;
                return self.update(Message::OpenProjectFile(path));
            }
            Message::OpenDialogCompleted(res) => match res {
                crate::ui::file_dialog::DialogResult::Selected(path) => {
                    return self.update(Message::OpenProjectFile(path));
                }
                crate::ui::file_dialog::DialogResult::Cancelled => {}
                crate::ui::file_dialog::DialogResult::Unavailable => {
                    return self.update(Message::OpenInlineFileDialog(FileDialogMode::Open));
                }
            },
            Message::SaveProjectAsDialog => {
                self.active_menu = None;
                let default_name = self
                    .current_file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|f| f.to_str())
                    .unwrap_or("model.kant.json")
                    .to_string();

                return Task::perform(
                    async move { crate::ui::file_dialog::pick_file_to_save(Some(&default_name)) },
                    Message::SaveDialogCompleted,
                );
            }
            Message::SaveDialogCompleted(res) => match res {
                crate::ui::file_dialog::DialogResult::Selected(path) => {
                    return self.update(Message::SaveProjectToFile(path));
                }
                crate::ui::file_dialog::DialogResult::Cancelled => {}
                crate::ui::file_dialog::DialogResult::Unavailable => {
                    return self.update(Message::OpenInlineFileDialog(FileDialogMode::SaveAs));
                }
            },
            Message::OpenInlineFileDialog(mode) => {
                self.file_dialog_mode = Some(mode);
                self.file_dialog_input = self
                    .current_file_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "model.kant.json".to_string());
            }
            Message::CloseFileDialog => {
                self.file_dialog_mode = None;
            }
            Message::FileDialogInputChanged(path_str) => {
                self.file_dialog_input = path_str;
            }
            Message::ConfirmFileDialog => {
                if let Some(mode) = self.file_dialog_mode {
                    let path = PathBuf::from(self.file_dialog_input.trim());
                    if !self.file_dialog_input.trim().is_empty() {
                        self.file_dialog_mode = None;
                        return match mode {
                            FileDialogMode::Open => self.update(Message::OpenProjectFile(path)),
                            FileDialogMode::SaveAs => self.update(Message::SaveProjectToFile(path)),
                        };
                    }
                }
                self.file_dialog_mode = None;
            }
            Message::OpenProjectFile(path) => {
                let effective = ProjectStorage::effective_model_path(&path);
                match ProjectStorage::load(&effective) {
                    Ok(mut proj) => {
                        proj.sync_concept_graph();
                        proj.sync_information_graph();
                        self.project = proj;
                        let display = effective.display().to_string();
                        self.current_file_path = Some(effective.clone());
                        self.recent_store.record_opened(&effective);
                        self.save_status = SaveStatus::Saved {
                            path: display,
                            timestamp: current_timestamp(),
                        };
                        self.file_dialog_mode = None;
                        self.selected_graph_node_id = None;
                        self.relation_dialog = None;
                        self.quick_create = None;
                        self.is_inline_graph_editing = false;
                        if !self.project.concepts().is_empty() {
                            self.active_tab = Tab::ConceptList;
                        }
                        if let Some(dir) = self.repo_dir() {
                            if let Ok(st) = GitService::get_sync_status(&dir) {
                                self.git_sync_status = st;
                            }
                        } else {
                            self.git_sync_status = RepoSyncStatus::Uninitialized;
                        }
                    }
                    Err(err) => {
                        self.save_status = SaveStatus::Error(format!(
                            "Kunne ikke åbne {}: {}",
                            path.display(),
                            err
                        ));
                    }
                }
            }
            Message::SaveProjectToFile(path) => {
                let effective = ProjectStorage::effective_model_path(&path);
                self.current_file_path = Some(effective.clone());
                self.recent_store.record_opened(&effective);
                self.trigger_autosave();
                self.file_dialog_mode = None;
            }
            Message::OpenModelRules => {
                self.active_menu = None;
                let url = "https://arkitektur.digst.dk/node/770";
                if let Err(err) = open_browser(url) {
                    eprintln!("Kunne ikke åbne browser for {}: {}", url, err);
                }
            }

            // Eksportmotor (Task 059)
            Message::ExportActiveDiagramSvgDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = match self.active_tab {
                    Tab::InformationModel => format!("{}_informationsmodel.svg", sanitized_name),
                    _ => format!("{}_begrebsmodel.svg", sanitized_name),
                };
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "SVG Vektordiagram (*.svg)",
                            "svg",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::DiagramSvg),
                );
            }
            Message::ExportConceptListCsvDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_begrebsliste.csv", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "CSV-regneark med UTF-8 BOM (*.csv)",
                            "csv",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ConceptListCsv),
                );
            }
            Message::ExportModelReportMarkdownDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_afleveringsrapport.md", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "Markdown afleveringsrapport (*.md)",
                            "md",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ModelReportMarkdown),
                );
            }
            Message::ExportModelReportHtmlDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_afleveringsrapport.html", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "HTML afleveringsrapport (*.html)",
                            "html",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ModelReportHtml),
                );
            }
            Message::ExportModelXmiDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_uml25.xmi", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "Enterprise Architect / UML 2.5 XMI (*.xmi)",
                            "xmi",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ModelXmi),
                );
            }
            Message::ExportModelSkosDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_begreber.skos.ttl", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "W3C SKOS Begrebsmodel RDF/Turtle (*.ttl, *.skos.ttl)",
                            "ttl",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ModelSkosTurtle),
                );
            }
            Message::ExportModelShaclDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_informationsmodel.shacl.ttl", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "W3C SHACL Shapes & OWL RDF/Turtle (*.ttl, *.shacl.ttl)",
                            "ttl",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ModelShaclTurtle),
                );
            }
            Message::ExportConceptListXlsxDialog => {
                self.active_menu = None;
                let sanitized_name = self.project.metadata().name().replace(' ', "_");
                let default_name = format!("{}_begrebsliste.xlsx", sanitized_name);
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "Excel-arbejdsbog (*.xlsx)",
                            "xlsx",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::ConceptListXlsx),
                );
            }
            Message::ExportFdaTemplateXlsxDialog => {
                self.active_menu = None;
                let default_name = "Begrebsliste_i_tabelformat_skabelon.xlsx".to_string();
                return Task::perform(
                    async move {
                        crate::ui::file_dialog::pick_file_to_export(
                            &default_name,
                            "Excel-arbejdsbog (*.xlsx)",
                            "xlsx",
                        )
                    },
                    |res| Message::ExportDialogCompleted(res, ExportKind::FdaTemplateXlsx),
                );
            }
            Message::ImportConceptsExcelDialog => {
                self.active_menu = None;
                return Task::perform(
                    async move { crate::ui::file_dialog::pick_excel_file_to_import() },
                    Message::ImportConceptsExcelCompleted,
                );
            }
            Message::ImportConceptsExcelCompleted(res) => {
                if let crate::ui::file_dialog::DialogResult::Selected(path) = res {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            match crate::features::concepts::excel::parse_concepts_from_xlsx_bytes(
                                &bytes,
                            ) {
                                Ok(import_summary) => {
                                    let mut new_count = 0;
                                    let mut update_count = 0;
                                    for c in &import_summary.valid_concepts {
                                        let is_match =
                                            self.project.concepts().iter().any(|existing| {
                                                (c.identifier().is_some()
                                                    && c.identifier() == existing.identifier())
                                                    || c.preferred_term()
                                                        .trim()
                                                        .eq_ignore_ascii_case(
                                                            existing.preferred_term().trim(),
                                                        )
                                            });
                                        if is_match {
                                            update_count += 1;
                                        } else {
                                            new_count += 1;
                                        }
                                    }
                                    self.excel_import_preview = Some(ExcelImportPreviewState {
                                        import_summary,
                                        new_count,
                                        update_count,
                                        file_path: path,
                                    });
                                }
                                Err(err) => {
                                    eprintln!("Fejl ved parsing af Excel-regneark: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Fejl ved indlæsning af fil: {}", err);
                        }
                    }
                }
            }
            Message::OpenExcelSchemaGuideModal => {
                self.active_menu = None;
                self.excel_schema_guide_open = true;
            }
            Message::CloseExcelSchemaGuideModal => {
                self.excel_schema_guide_open = false;
            }
            Message::ConfirmExcelImport => {
                if let Some(preview) = self.excel_import_preview.take() {
                    let summary = crate::features::concepts::excel::apply_concept_upsert(
                        &mut self.project,
                        preview.import_summary.valid_concepts,
                    );
                    println!(
                        "Excel import gennemført: {} nye begreber tilføjet, {} eksisterende opdateret",
                        summary.inserted, summary.updated
                    );
                    self.trigger_autosave();
                }
            }
            Message::DismissExcelImportModal => {
                self.excel_import_preview = None;
            }
            Message::ExportDialogCompleted(res, kind) => {
                if let crate::ui::file_dialog::DialogResult::Selected(path) = res {
                    let write_res = match kind {
                        ExportKind::ConceptListXlsx => {
                            match crate::features::concepts::excel::export_project_concepts_to_xlsx(
                                &self.project,
                            ) {
                                Ok(bytes) => std::fs::write(&path, bytes),
                                Err(err) => {
                                    eprintln!("Fejl under generering af Excel-fil: {}", err);
                                    return Task::none();
                                }
                            }
                        }
                        ExportKind::FdaTemplateXlsx => {
                            let bytes =
                                crate::features::concepts::excel::get_fda_template_xlsx_bytes();
                            std::fs::write(&path, bytes)
                        }
                        ExportKind::DiagramSvg => {
                            let content = if self.active_tab == Tab::InformationModel {
                                export_information_model_svg(
                                    self.project.information_graph(),
                                    self.project.information_model(),
                                )
                            } else {
                                export_concept_model_svg(
                                    self.project.concept_graph(),
                                    self.project.concepts(),
                                )
                            };
                            std::fs::write(&path, content)
                        }
                        ExportKind::ConceptListCsv => {
                            std::fs::write(&path, export_concepts_to_csv(self.project.concepts()))
                        }
                        ExportKind::ModelReportMarkdown => {
                            std::fs::write(&path, export_model_report_markdown(&self.project))
                        }
                        ExportKind::ModelReportHtml => {
                            std::fs::write(&path, export_model_report_html(&self.project))
                        }
                        ExportKind::ModelXmi => {
                            std::fs::write(&path, export_to_xmi_2_1(&self.project))
                        }
                        ExportKind::ModelSkosTurtle => {
                            std::fs::write(&path, export_to_skos_turtle(&self.project))
                        }
                        ExportKind::ModelShaclTurtle => {
                            std::fs::write(&path, export_to_shacl_turtle(&self.project))
                        }
                    };

                    if let Err(err) = write_res {
                        eprintln!("Fejl under skrivning af eksportfil {:?}: {}", path, err);
                    }
                }
            }

            // Tastaturnavigation & genveje
            Message::FocusNext => {
                return operation::focus_next();
            }
            Message::FocusPrevious => {
                return operation::focus_previous();
            }
            Message::EscapePressed => {
                if self.active_menu.is_some() {
                    self.active_menu = None;
                } else if self.start_session_modal.is_some() {
                    self.start_session_modal = None;
                } else if self.join_session_modal.is_some() {
                    self.join_session_modal = None;
                } else if self.guest_ended_notice.is_some() {
                    self.guest_ended_notice = None;
                } else if self.metadata_modal.is_some() {
                    self.metadata_modal = None;
                } else if self.quick_create.is_some() {
                    self.quick_create = None;
                } else if self.relation_dialog.is_some() {
                    self.relation_dialog = None;
                } else if self.file_dialog_mode.is_some() {
                    self.file_dialog_mode = None;
                } else if self.is_inline_graph_editing {
                    self.is_inline_graph_editing = false;
                    self.editor_state = None;
                } else if self.editor_state.is_some() {
                    self.editor_state = None;
                } else if self.selected_edge.is_some() {
                    self.selected_edge = None;
                } else if self.selected_info_edge.is_some() {
                    self.selected_info_edge = None;
                } else if self.selected_graph_node_id.is_some() {
                    self.selected_graph_node_id = None;
                } else if self.selected_info_graph_node_id.is_some() {
                    self.selected_info_graph_node_id = None;
                }
            }
            Message::EnterPressed => {
                // Task 069 placeholder for RED test
                return Task::none();
            }

            // Graf-handlinger (Fase 3)
            Message::GraphNodeSelected(node_id) => {
                self.selected_graph_node_id = node_id;
                self.selected_graph_node_ids = node_id.into_iter().collect();
                self.selected_edge = None;
            }
            Message::GraphNodesSelected(ids) => {
                self.selected_graph_node_ids = ids;
                self.selected_graph_node_id = self.selected_graph_node_ids.iter().next().copied();
                self.selected_edge = None;
            }
            Message::GraphEdgeSelected(edge) => {
                self.selected_edge = edge;
                if edge.is_some() {
                    self.selected_graph_node_id = None;
                    self.selected_graph_node_ids.clear();
                }
            }
            Message::GraphEdgeCreated(from, to) => {
                if from == to {
                    return Task::none();
                }
                let graph = self.project.concept_graph();
                if graph.find_node(from).is_some() && graph.find_node(to).is_some() {
                    if graph.find_edge(from, to).is_none() {
                        let rel_id = Uuid::new_v4();
                        self.project.concept_graph_mut().add_relation_full(
                            rel_id,
                            from,
                            to,
                            RelationKind::Association,
                            None,
                        );
                        let cg = self.project.concept_graph_mut();
                        let nodes = cg.nodes().to_vec();
                        let edges = cg.edges().to_vec();
                        let routes =
                            crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                        for r in routes {
                            cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                        }
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::RelationAdded(
                                crate::features::collab::protocol::Relation::with_all(
                                    rel_id,
                                    from,
                                    to,
                                    RelationKind::Association,
                                    None,
                                    Some(true),
                                ),
                            ),
                        );
                        self.trigger_autosave();
                    }
                    self.selected_graph_node_id = None;
                    self.selected_edge = Some((from, to));
                    return operation::focus("edge_label_input");
                }
            }
            Message::GraphUpdateEdgeKind(from, to, kind) => {
                if self
                    .project
                    .concept_graph_mut()
                    .update_edge_kind(from, to, kind)
                {
                    let label = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .and_then(|e| e.label().map(String::from));
                    let directed = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .and_then(|e| e.directed());
                    let rel = crate::features::collab::protocol::Relation::with_all(
                        Uuid::new_v4(),
                        from,
                        to,
                        kind,
                        label,
                        directed,
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::RelationUpdated(rel),
                    );
                    self.trigger_autosave();
                }
            }
            Message::GraphUpdateEdgeLabel(from, to, label) => {
                let lbl = if label.trim().is_empty() {
                    None
                } else {
                    Some(label)
                };
                if self
                    .project
                    .concept_graph_mut()
                    .update_edge_label(from, to, lbl.clone())
                {
                    let kind = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .map(|e| e.kind())
                        .unwrap_or(RelationKind::Association);
                    let directed = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .and_then(|e| e.directed());
                    let rel = crate::features::collab::protocol::Relation::with_all(
                        Uuid::new_v4(),
                        from,
                        to,
                        kind,
                        lbl,
                        directed,
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::RelationUpdated(rel),
                    );
                    self.trigger_autosave();
                }
            }
            Message::GraphToggleEdgeDirected(from, to, directed) => {
                if self
                    .project
                    .concept_graph_mut()
                    .update_edge_directed(from, to, directed)
                {
                    let kind = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .map(|e| e.kind())
                        .unwrap_or(RelationKind::Association);
                    let label = self
                        .project
                        .concept_graph()
                        .find_edge(from, to)
                        .and_then(|e| e.label().map(String::from));
                    let rel = crate::features::collab::protocol::Relation::with_all(
                        Uuid::new_v4(),
                        from,
                        to,
                        kind,
                        label,
                        Some(directed),
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::RelationUpdated(rel),
                    );
                    self.trigger_autosave();
                }
            }
            Message::GraphReverseEdge(from, to) => {
                if self.project.concept_graph_mut().reverse_relation(from, to) {
                    self.selected_edge = Some((to, from));
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::RelationDeleted {
                            from,
                            to,
                        },
                    );
                    if let Some(edge) = self.project.concept_graph().find_edge(to, from) {
                        let rel = crate::features::collab::protocol::Relation::with_all(
                            Uuid::new_v4(),
                            to,
                            from,
                            edge.kind(),
                            edge.label().map(String::from),
                            edge.directed(),
                        );
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::RelationAdded(rel),
                        );
                    }
                    self.trigger_autosave();
                }
            }
            Message::GraphDeleteSelected => match self.active_tab {
                Tab::ConceptModel => {
                    if let Some((from, to)) = self.selected_edge.take() {
                        self.project.concept_graph_mut().remove_relation(from, to);
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::RelationDeleted {
                                from,
                                to,
                            },
                        );
                        self.trigger_autosave();
                    } else if !self.selected_graph_node_ids.is_empty() {
                        let to_remove: Vec<NodeId> = self.selected_graph_node_ids.drain().collect();
                        self.selected_graph_node_id = None;
                        for node_id in to_remove {
                            let concept_id = self
                                .project
                                .concept_graph()
                                .find_node(node_id)
                                .map(|n| n.concept_id());
                            self.project.concept_graph_mut().remove_node(node_id);
                            if let Some(cid) = concept_id {
                                self.broadcast_mutation(
                                    &crate::features::collab::protocol::ModelMutation::ConceptDiagramNodeRemoved(
                                        cid,
                                    ),
                                );
                            }
                        }
                        self.trigger_autosave();
                    } else if let Some(node_id) = self.selected_graph_node_id.take() {
                        let concept_id = self
                            .project
                            .concept_graph()
                            .find_node(node_id)
                            .map(|n| n.concept_id());
                        self.project.concept_graph_mut().remove_node(node_id);
                        if let Some(cid) = concept_id {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ConceptDiagramNodeRemoved(
                                    cid,
                                ),
                            );
                        }
                        self.trigger_autosave();
                    }
                }
                Tab::InformationModel => {
                    if let Some((from, to)) = self.selected_info_edge.take() {
                        let from_class = self
                            .project
                            .information_graph()
                            .find_node(from)
                            .map(|n| n.class_id());
                        let to_class = self
                            .project
                            .information_graph()
                            .find_node(to)
                            .map(|n| n.class_id());
                        self.project
                            .information_graph_mut()
                            .remove_relation(from, to);
                        if let (Some(fc), Some(tc)) = (from_class, to_class) {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassRelationDeleted {
                                    from_class: fc,
                                    to_class: tc,
                                },
                            );
                        }
                        self.trigger_autosave();
                    } else if !self.selected_info_graph_node_ids.is_empty() {
                        let to_remove: Vec<NodeId> =
                            self.selected_info_graph_node_ids.drain().collect();
                        self.selected_info_graph_node_id = None;
                        self.selected_info_class_id = None;
                        for node_id in to_remove {
                            let class_id = self
                                .project
                                .information_graph()
                                .find_node(node_id)
                                .map(|n| n.class_id());
                            self.project.information_graph_mut().remove_node(node_id);
                            if let Some(cid) = class_id {
                                self.broadcast_mutation(
                                    &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeRemoved(
                                        cid,
                                    ),
                                );
                            }
                        }
                        self.trigger_autosave();
                    } else if let Some(node_id) = self.selected_info_graph_node_id.take() {
                        let class_id = self
                            .project
                            .information_graph()
                            .find_node(node_id)
                            .map(|n| n.class_id());
                        self.project.information_graph_mut().remove_node(node_id);
                        if let Some(cid) = class_id {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeRemoved(
                                    cid,
                                ),
                            );
                        }
                        self.trigger_autosave();
                    }
                }
                _ => {}
            },
            Message::GraphNodeMoved(node_id, x, y) => {
                let final_x = (x / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let final_y = (y / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let cg = self.project.concept_graph_mut();
                cg.update_node_position(node_id, final_x, final_y);

                let nodes = cg.nodes().to_vec();
                let edges = cg.edges().to_vec();
                let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                for r in routes {
                    cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                }
                self.broadcast_node_moved_throttled(node_id, final_x, final_y);
            }
            Message::GraphNodesMoved(moves) => {
                let computed_moves: Vec<(NodeId, f32, f32)> = moves
                    .into_iter()
                    .map(|(node_id, x, y)| {
                        (
                            node_id,
                            (x / crate::features::concept_model::GRID_SIZE).round()
                                * crate::features::concept_model::GRID_SIZE,
                            (y / crate::features::concept_model::GRID_SIZE).round()
                                * crate::features::concept_model::GRID_SIZE,
                        )
                    })
                    .collect();

                {
                    let cg = self.project.concept_graph_mut();
                    for &(node_id, final_x, final_y) in &computed_moves {
                        cg.update_node_position(node_id, final_x, final_y);
                    }
                    let nodes = cg.nodes().to_vec();
                    let edges = cg.edges().to_vec();
                    let routes = crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                    for r in routes {
                        cg.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }

                for (node_id, final_x, final_y) in computed_moves {
                    self.broadcast_node_moved_throttled(node_id, final_x, final_y);
                }
            }
            Message::CanvasNodesDragFinished => {
                self.trigger_autosave();
                crate::features::diagnostics::log_info(
                    "canvas",
                    "Node drag completed and state persisted",
                );
            }
            Message::GraphOpenRelationDialog => {
                let node_options: Vec<NodeOption> = self
                    .project
                    .concept_graph()
                    .nodes()
                    .iter()
                    .map(|n| NodeOption {
                        id: n.id(),
                        label: n.label().to_string(),
                    })
                    .collect();

                let from_node = node_options.first().cloned();
                let to_node = node_options
                    .get(1)
                    .or_else(|| node_options.first())
                    .cloned();
                self.relation_dialog = Some(RelationDialogState {
                    from_node,
                    to_node,
                    kind: RelationKind::Generalization,
                    label: String::new(),
                    source_multiplicity: None,
                    target_multiplicity: None,
                    error: None,
                });
            }
            Message::GraphCloseRelationDialog => {
                self.relation_dialog = None;
            }
            Message::GraphRelationFromChanged(opt) => {
                if let Some(dialog) = &mut self.relation_dialog {
                    dialog.from_node = Some(opt);
                    dialog.error = None;
                }
            }
            Message::GraphRelationToChanged(opt) => {
                if let Some(dialog) = &mut self.relation_dialog {
                    dialog.to_node = Some(opt);
                    dialog.error = None;
                }
            }
            Message::GraphRelationKindChanged(kind) => {
                if let Some(dialog) = &mut self.relation_dialog {
                    dialog.kind = kind;
                }
            }
            Message::GraphRelationLabelChanged(label) => {
                if let Some(dialog) = &mut self.relation_dialog {
                    dialog.label = label;
                }
            }
            Message::GraphCreateRelation => {
                if let Some(dialog) = &self.relation_dialog {
                    match (&dialog.from_node, &dialog.to_node) {
                        (Some(from), Some(to)) => {
                            if from.id == to.id {
                                if let Some(d) = &mut self.relation_dialog {
                                    d.error = Some(
                                        "Kilde og mål kan ikke være det samme begreb.".to_string(),
                                    );
                                }
                            } else {
                                let label = if dialog.kind == RelationKind::Association
                                    && !dialog.label.trim().is_empty()
                                {
                                    Some(dialog.label.trim().to_string())
                                } else {
                                    None
                                };
                                let rel = crate::features::collab::protocol::Relation::with_all(
                                    Uuid::new_v4(),
                                    from.id,
                                    to.id,
                                    dialog.kind,
                                    label.clone(),
                                    if dialog.kind == RelationKind::Association {
                                        Some(true)
                                    } else {
                                        None
                                    },
                                );
                                self.broadcast_mutation(
                                    &crate::features::collab::protocol::ModelMutation::RelationAdded(
                                        rel,
                                    ),
                                );
                                self.project.concept_graph_mut().add_relation_with_label(
                                    from.id,
                                    to.id,
                                    dialog.kind,
                                    label,
                                );
                                let cg = self.project.concept_graph_mut();
                                let nodes = cg.nodes().to_vec();
                                let edges = cg.edges().to_vec();
                                let routes =
                                    crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges);
                                for r in routes {
                                    cg.update_edge_ports(
                                        r.from,
                                        r.to,
                                        Some(r.from_side),
                                        Some(r.to_side),
                                    );
                                }
                                self.relation_dialog = None;
                                self.trigger_autosave();
                            }
                        }
                        _ => {
                            if let Some(d) = &mut self.relation_dialog {
                                d.error =
                                    Some("Vælg venligst både kilde og målbegreb.".to_string());
                            }
                        }
                    }
                }
            }
            Message::GraphDeleteRelation(from, to) => {
                self.project.concept_graph_mut().remove_relation(from, to);
                if self.selected_edge == Some((from, to)) || self.selected_edge == Some((to, from))
                {
                    self.selected_edge = None;
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::RelationDeleted { from, to },
                );
                self.trigger_autosave();
            }
            Message::GraphSyncNodes => {
                self.project.sync_concept_graph();
                self.trigger_autosave();
            }
            Message::AddConceptToDiagram(concept_id) => {
                if !self
                    .project
                    .concept_graph()
                    .is_concept_on_diagram(concept_id)
                {
                    if let Some(concept) = self.project.get_concept(concept_id).cloned() {
                        let new_id = self.project.concept_graph_mut().add_node(&concept);
                        self.selected_graph_node_id = Some(new_id);
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ConceptDiagramNodeAdded(
                                concept_id,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::AddConceptToDiagramAt(concept_id, x, y) => {
                if !self
                    .project
                    .concept_graph()
                    .is_concept_on_diagram(concept_id)
                {
                    if let Some(concept) = self.project.get_concept(concept_id).cloned() {
                        let snapped_x = (x / crate::features::concept_model::GRID_SIZE).round()
                            * crate::features::concept_model::GRID_SIZE;
                        let snapped_y = (y / crate::features::concept_model::GRID_SIZE).round()
                            * crate::features::concept_model::GRID_SIZE;
                        let new_id = self
                            .project
                            .concept_graph_mut()
                            .add_node_at(&concept, snapped_x, snapped_y);
                        self.selected_graph_node_id = Some(new_id);
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ConceptDiagramNodeAdded(
                                concept_id,
                            ),
                        );
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::NodeMoved {
                                id: concept_id,
                                x: snapped_x,
                                y: snapped_y,
                            },
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::RemoveConceptFromDiagram(node_id) => {
                let concept_id = self
                    .project
                    .concept_graph()
                    .find_node(node_id)
                    .map(|n| n.concept_id());
                self.project.concept_graph_mut().remove_node(node_id);
                if self.selected_graph_node_id == Some(node_id) {
                    self.selected_graph_node_id = None;
                }
                if let Some(cid) = concept_id {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::ConceptDiagramNodeRemoved(
                            cid,
                        ),
                    );
                }
                self.trigger_autosave();
            }
            Message::ConceptModelSearchChanged(query) => {
                self.concept_model_search = query;
            }

            // Lynoprettelse & Node-redigering på Canvas (Task 006, 030 & 039)
            Message::CanvasDoubleClicked(x, y) => {
                let (nx, ny) = (x, y);
                let concept =
                    Concept::new("Nyt begreb", "Definition mangler", BelongsToDomain::Yes);
                if let Ok(concept_id) = self.project.add_concept(concept.clone()) {
                    if let Some(node) = self
                        .project
                        .concept_graph_mut()
                        .find_node_by_concept_mut(concept_id)
                    {
                        node.set_position(nx, ny);
                    }
                    if let Some(node) = self
                        .project
                        .concept_graph()
                        .find_node_by_concept(concept_id)
                    {
                        self.selected_graph_node_id = Some(node.id());
                    }
                    self.selected_edge = None;
                    let mut editor = ConceptEditorState::from_concept(&concept);
                    editor.preferred_term = String::new();
                    self.editor_state = Some(editor);
                    self.is_inline_graph_editing = true;
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::ConceptAdded(concept),
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::NodeMoved {
                            id: concept_id,
                            x: nx,
                            y: ny,
                        },
                    );
                    self.trigger_autosave();
                    return operation::focus("preferred_term_input");
                }
            }
            Message::CreateConceptAtCenter => {
                let center_world = self.canvas_viewport.to_world(Point::new(500.0, 350.0));
                let cx = center_world.x - crate::features::concept_model::DEFAULT_NODE_WIDTH / 2.0;
                let cy = center_world.y - crate::features::concept_model::DEFAULT_NODE_HEIGHT / 2.0;
                return self.update(Message::CanvasDoubleClicked(cx, cy));
            }
            Message::QuickCreateTermChanged(term) => {
                if let Some(qc) = &mut self.quick_create {
                    qc.preferred_term = term;
                    qc.validation_error = None;
                }
            }
            Message::QuickCreateDefinitionChanged(def) => {
                if let Some(qc) = &mut self.quick_create {
                    qc.definition = def;
                    qc.validation_error = None;
                }
            }
            Message::QuickCreateDomainChanged(domain) => {
                if let Some(qc) = &mut self.quick_create {
                    qc.belongs_to_domain = domain;
                }
            }
            Message::QuickCreateCancel => {
                self.quick_create = None;
            }
            Message::QuickCreateSubmit => {
                if let Some(qc) = &mut self.quick_create {
                    match qc.build_concept() {
                        Ok(concept) => {
                            let (x, y) = qc.position;
                            match self.project.add_concept(concept.clone()) {
                                Ok(concept_id) => {
                                    if let Some(node) = self
                                        .project
                                        .concept_graph_mut()
                                        .find_node_by_concept_mut(concept_id)
                                    {
                                        node.set_position(x, y);
                                    }
                                    if let Some(node) = self
                                        .project
                                        .concept_graph()
                                        .find_node_by_concept(concept_id)
                                    {
                                        self.selected_graph_node_id = Some(node.id());
                                    }
                                    self.broadcast_mutation(
                                        &crate::features::collab::protocol::ModelMutation::ConceptAdded(
                                            concept,
                                        ),
                                    );
                                    self.broadcast_mutation(
                                        &crate::features::collab::protocol::ModelMutation::NodeMoved {
                                            id: concept_id,
                                            x,
                                            y,
                                        },
                                    );
                                    self.quick_create = None;
                                    self.trigger_autosave();
                                }
                                Err(err) => {
                                    qc.validation_error = Some(err.to_string());
                                }
                            }
                        }
                        Err(err) => {
                            qc.validation_error = Some(err.to_string());
                        }
                    }
                }
            }
            Message::GraphNodeDoubleClicked(node_id) => {
                self.selected_graph_node_id = Some(node_id);
                if let Some(node) = self.project.concept_graph().find_node(node_id) {
                    if let Some(concept) = self.project.get_concept(node.concept_id()) {
                        self.editor_state = Some(ConceptEditorState::from_concept(concept));
                        self.is_inline_graph_editing = true;
                        return operation::focus("preferred_term_input");
                    }
                }
            }

            // Canvas ergonomi (Task 008)
            Message::CanvasViewportChanged(vp) => {
                self.canvas_viewport = vp;
            }
            Message::ToggleSnapToGrid => {
                self.snap_to_grid = !self.snap_to_grid;
            }
            Message::CanvasZoomIn => {
                self.canvas_viewport.zoom_at(Point::new(500.0, 400.0), 1.15);
            }
            Message::CanvasZoomOut => {
                self.canvas_viewport
                    .zoom_at(Point::new(500.0, 400.0), 1.0 / 1.15);
            }
            Message::CanvasResetView => {
                self.canvas_viewport.reset();
            }
            Message::CanvasSpacePressed(pressed) => {
                self.is_space_pressed = pressed;
            }
            Message::CanvasModifiersChanged(_mods) => {}

            // Informationsmodel (Task 010 & 011)
            Message::SelectInformationClass(id) => {
                self.selected_info_class_id = id;
                if id.is_some() {
                    self.selected_info_edge = None;
                }
                if let Some(cid) = id {
                    if let Some(node) = self.project.information_graph().find_node_by_class(cid) {
                        self.selected_info_graph_node_id = Some(node.id());
                    } else {
                        self.selected_info_graph_node_id = None;
                    }
                } else {
                    self.selected_info_graph_node_id = None;
                }
            }
            Message::CreateInformationClass => {
                let class = InformationClass::new("");
                let id = self.project.information_model_mut().add_class(class);
                let node_id = self.project.information_graph_mut().add_node(id, 0);
                self.selected_info_class_id = Some(id);
                self.selected_info_graph_node_id = Some(node_id);
                self.selected_info_edge = None;
                if let Some(cls) = self.project.information_model().get_class(id).cloned() {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassAdded(
                            cls,
                        ),
                    );
                }
                self.trigger_autosave();
                return operation::focus("info_class_name_input");
            }
            Message::CreateInformationClassAt(x, y) => {
                let class = InformationClass::new("");
                let id = self.project.information_model_mut().add_class(class);
                let nx = (x / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let ny = (y / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let node_id = self
                    .project
                    .information_graph_mut()
                    .add_node_at(id, nx, ny, 0);
                self.selected_info_class_id = Some(id);
                self.selected_info_graph_node_id = Some(node_id);
                self.selected_info_edge = None;
                if let Some(cls) = self.project.information_model().get_class(id).cloned() {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassAdded(
                            cls,
                        ),
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::NodeMoved {
                            id,
                            x: nx,
                            y: ny,
                        },
                    );
                }
                self.trigger_autosave();
                return operation::focus("info_class_name_input");
            }
            Message::CreateInformationClassAtCenter => {
                let center_world = self.info_canvas_viewport.to_world(Point::new(500.0, 350.0));
                let cx = center_world.x
                    - crate::features::information_model::DEFAULT_CLASS_NODE_WIDTH / 2.0;
                let cy = center_world.y
                    - crate::features::information_model::calculate_class_node_height(0) / 2.0;
                return self.update(Message::CreateInformationClassAt(cx, cy));
            }
            Message::CreateInformationClassFromConcept(opt) => {
                self.selected_info_edge = None;
                if let Some(concept) = self.project.get_concept(opt.id).cloned() {
                    let id = self
                        .project
                        .information_model_mut()
                        .create_class_from_concept(&concept);
                    let attr_count = self
                        .project
                        .information_model()
                        .get_class(id)
                        .map(|c| c.attributes().len())
                        .unwrap_or(0);
                    let node_id = self
                        .project
                        .information_graph_mut()
                        .add_node(id, attr_count);
                    self.selected_info_class_id = Some(id);
                    self.selected_info_graph_node_id = Some(node_id);
                    if let Some(cls) = self.project.information_model().get_class(id).cloned() {
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassAdded(
                                cls,
                            ),
                        );
                    }
                    self.trigger_autosave();
                }
            }
            Message::UpdateInformationClassName(class_id, name) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    let final_name = if class.name() == "NyKlasse" && name != "NyKlasse" {
                        name.replace("NyKlasse", "")
                    } else {
                        name
                    };
                    class.set_name(final_name);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::UpdateInformationClassDescription(class_id, desc) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.set_description(if desc.trim().is_empty() {
                        None
                    } else {
                        Some(desc)
                    });
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::SetInformationClassAbstract(class_id, is_abstract) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.set_abstract(is_abstract);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::SetInformationClassLocal(class_id, is_local) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.set_local(is_local);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::SetInformationClassOriginModel(class_id, origin) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    let opt = if origin.trim().is_empty() {
                        None
                    } else {
                        Some(origin)
                    };
                    class.set_origin_model(opt);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::AddConceptToInformationClass(class_id, opt) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.add_concept_id(opt.id);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::RemoveConceptFromInformationClass(class_id, concept_id) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.remove_concept_id(concept_id);
                    let cls = class.clone();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::DeleteInformationClass(class_id) => {
                self.project.remove_information_class(class_id);
                if self.selected_info_class_id == Some(class_id) {
                    self.selected_info_class_id = None;
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                    }
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::InformationClassDeleted(
                        class_id,
                    ),
                );
                self.trigger_autosave();
            }
            Message::AddAttributeToClass(class_id) => {
                let cls_opt = if let Some(class) =
                    self.project.information_model_mut().get_class_mut(class_id)
                {
                    let attr = Attribute::new(
                        "",
                        PrimitiveType::CharacterString,
                        Multiplicity::exactly_one(),
                    );
                    class.add_attribute(attr);
                    Some(class.clone())
                } else {
                    None
                };
                if let Some(cls) = cls_opt {
                    let attr_count = cls.attributes().len();
                    self.project
                        .information_graph_mut()
                        .update_class_dimensions(class_id, attr_count);
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::UpdateAttributeName(class_id, attr_id, name) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        let final_name = if attr.name() == "nyAttribut" && name != "nyAttribut" {
                            name.replace("nyAttribut", "")
                        } else {
                            name
                        };
                        attr.set_name(final_name);
                        let cls = class.clone();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                                cls,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::UpdateAttributeType(class_id, attr_id, dt) => {
                let cls_opt = if let Some(class) =
                    self.project.information_model_mut().get_class_mut(class_id)
                {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        attr.set_data_type(dt);
                    }
                    Some(class.clone())
                } else {
                    None
                };
                if let Some(cls) = cls_opt {
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::UpdateAttributeDataType(class_id, attr_id, dt) => {
                let cls_opt = if let Some(class) =
                    self.project.information_model_mut().get_class_mut(class_id)
                {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        attr.set_data_type(dt);
                    }
                    Some(class.clone())
                } else {
                    None
                };
                if let Some(cls) = cls_opt {
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::UpdateAttributeMultiplicity(class_id, attr_id, m) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        attr.set_multiplicity(m);
                        let cls = class.clone();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                                cls,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::AddConceptToAttribute(class_id, attr_id, opt) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        attr.add_concept_id(opt.id);
                        let cls = class.clone();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                                cls,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::RemoveConceptFromAttribute(class_id, attr_id, concept_id) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        attr.remove_concept_id(concept_id);
                        let cls = class.clone();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                                cls,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::SetAttributeConcept(class_id, attr_id, maybe_concept_id) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    if let Some(attr) = class
                        .attributes_mut()
                        .iter_mut()
                        .find(|a| a.id() == attr_id)
                    {
                        if let Some(cid) = maybe_concept_id {
                            attr.set_concept_ids(vec![cid]);
                        } else {
                            attr.clear_concept_ids();
                        }
                        let cls = class.clone();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                                cls,
                            ),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::DeleteAttribute(class_id, attr_id) => {
                let cls_opt = if let Some(class) =
                    self.project.information_model_mut().get_class_mut(class_id)
                {
                    class.remove_attribute(attr_id);
                    Some(class.clone())
                } else {
                    None
                };
                if let Some(cls) = cls_opt {
                    let attr_count = cls.attributes().len();
                    self.project
                        .information_graph_mut()
                        .update_class_dimensions(class_id, attr_count);
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationClassUpdated(
                            cls,
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::InformationClassSearchChanged(q) => {
                self.info_class_search = q;
            }

            // Enumeration styring (Task 063)
            Message::CreateInformationEnumeration => {
                let count = self.project.information_model().enumerations().len() + 1;
                let enum_name = format!("Enumeration{}", count);
                let e = InformationEnumeration::new(enum_name, Vec::new());
                let enum_id = self
                    .project
                    .information_model_mut()
                    .add_enumeration(e.clone());
                self.selected_info_enum_id = Some(enum_id);
                self.selected_info_class_id = None;
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::InformationEnumerationAdded(
                        e,
                    ),
                );
                self.trigger_autosave();
            }
            Message::CreateInformationEnumerationAtCenter => {
                let center_world = self.info_canvas_viewport.to_world(Point::new(500.0, 350.0));
                let count = self.project.information_model().enumerations().len() + 1;
                let enum_name = format!("Enumeration{}", count);
                let e = InformationEnumeration::new(enum_name, Vec::new());
                let enum_id = self
                    .project
                    .information_model_mut()
                    .add_enumeration(e.clone());
                let val_count = e.values().len();
                let cx = ((center_world.x
                    - crate::features::information_model::DEFAULT_CLASS_NODE_WIDTH / 2.0)
                    / crate::features::concept_model::GRID_SIZE)
                    .round()
                    * crate::features::concept_model::GRID_SIZE;
                let cy = ((center_world.y
                    - crate::features::information_model::calculate_class_node_height(0) / 2.0)
                    / crate::features::concept_model::GRID_SIZE)
                    .round()
                    * crate::features::concept_model::GRID_SIZE;
                let node_id = self
                    .project
                    .information_graph_mut()
                    .add_node_at(enum_id, cx, cy, val_count);
                self.project.sync_information_graph();
                self.reroute_info_graph_edges();
                self.selected_info_enum_id = Some(enum_id);
                self.selected_info_class_id = None;
                self.selected_info_graph_node_id = Some(node_id);
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::InformationEnumerationAdded(
                        e,
                    ),
                );
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeAdded(
                        enum_id,
                    ),
                );
                self.trigger_autosave();
            }
            Message::SelectInformationEnumeration(id) => {
                self.selected_info_enum_id = id;
                if id.is_some() {
                    self.selected_info_class_id = None;
                }
            }
            Message::UpdateInformationEnumerationName(enum_id, name) => {
                let e_opt = if let Some(e) = self
                    .project
                    .information_model_mut()
                    .get_enumeration_mut(enum_id)
                {
                    e.set_name(name);
                    Some(e.clone())
                } else {
                    None
                };
                if let Some(e) = e_opt {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationEnumerationUpdated(e),
                    );
                    self.trigger_autosave();
                }
            }
            Message::UpdateInformationEnumerationDefinition(enum_id, def) => {
                let e_opt = if let Some(e) = self
                    .project
                    .information_model_mut()
                    .get_enumeration_mut(enum_id)
                {
                    let d = if def.trim().is_empty() {
                        None
                    } else {
                        Some(def)
                    };
                    e.set_definition(d);
                    Some(e.clone())
                } else {
                    None
                };
                if let Some(e) = e_opt {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationEnumerationUpdated(e),
                    );
                    self.trigger_autosave();
                }
            }
            Message::AddValueToEnumeration(enum_id, val) => {
                let val_trimmed = val.trim().to_string();
                if !val_trimmed.is_empty()
                    && crate::features::information_model::NamingLinter::is_lower_camel_case(
                        &val_trimmed,
                    )
                {
                    let (val_count, e_clone) = if let Some(e) = self
                        .project
                        .information_model_mut()
                        .get_enumeration_mut(enum_id)
                    {
                        e.add_value(val_trimmed);
                        (Some(e.values().len()), Some(e.clone()))
                    } else {
                        (None, None)
                    };
                    if let (Some(val_count), Some(e)) = (val_count, e_clone) {
                        self.project
                            .information_graph_mut()
                            .update_class_dimensions(enum_id, val_count);
                        self.new_enum_value_input.clear();
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::InformationEnumerationUpdated(e),
                        );
                        self.trigger_autosave();
                    }
                }
            }
            Message::RemoveValueFromEnumeration(enum_id, val) => {
                let (val_count, e_clone) = if let Some(e) = self
                    .project
                    .information_model_mut()
                    .get_enumeration_mut(enum_id)
                {
                    e.remove_value(&val);
                    (Some(e.values().len()), Some(e.clone()))
                } else {
                    (None, None)
                };
                if let (Some(val_count), Some(e)) = (val_count, e_clone) {
                    self.project
                        .information_graph_mut()
                        .update_class_dimensions(enum_id, val_count);
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::InformationEnumerationUpdated(e),
                    );
                    self.trigger_autosave();
                }
            }
            Message::DeleteInformationEnumeration(enum_id) => {
                self.project.remove_information_enumeration(enum_id);
                if self.selected_info_enum_id == Some(enum_id) {
                    self.selected_info_enum_id = None;
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                    }
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::InformationEnumerationDeleted(enum_id),
                );
                self.trigger_autosave();
            }
            Message::AddEnumerationToDiagram(enum_id) => {
                let val_count = self
                    .project
                    .information_model()
                    .get_enumeration(enum_id)
                    .map(|e| e.values().len())
                    .unwrap_or(0);
                let node_id = self
                    .project
                    .information_graph_mut()
                    .add_node(enum_id, val_count);
                self.project.sync_information_graph();
                self.reroute_info_graph_edges();
                self.selected_info_enum_id = Some(enum_id);
                self.selected_info_graph_node_id = Some(node_id);
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeAdded(
                        enum_id,
                    ),
                );
                self.trigger_autosave();
            }
            Message::AddEnumerationToDiagramAt(enum_id, x, y) => {
                if !self
                    .project
                    .information_graph()
                    .is_class_on_diagram(enum_id)
                {
                    let val_count = self
                        .project
                        .information_model()
                        .get_enumeration(enum_id)
                        .map(|e| e.values().len())
                        .unwrap_or(0);
                    let snapped_x = (x / crate::features::concept_model::GRID_SIZE).round()
                        * crate::features::concept_model::GRID_SIZE;
                    let snapped_y = (y / crate::features::concept_model::GRID_SIZE).round()
                        * crate::features::concept_model::GRID_SIZE;
                    let node_id = self
                        .project
                        .information_graph_mut()
                        .add_node_at(enum_id, snapped_x, snapped_y, val_count);
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
                    self.selected_info_enum_id = Some(enum_id);
                    self.selected_info_graph_node_id = Some(node_id);
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeAdded(
                            enum_id,
                        ),
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::NodeMoved {
                            id: enum_id,
                            x: snapped_x,
                            y: snapped_y,
                        },
                    );
                    self.trigger_autosave();
                }
            }
            Message::RemoveEnumerationFromDiagram(enum_id) => {
                let node_id_opt = self
                    .project
                    .information_graph()
                    .find_node_by_class(enum_id)
                    .map(|n| n.id());
                if let Some(node_id) = node_id_opt {
                    self.project.information_graph_mut().remove_node(node_id);
                    if self.selected_info_graph_node_id == Some(node_id) {
                        self.selected_info_graph_node_id = None;
                    }
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeRemoved(
                        enum_id,
                    ),
                );
                self.trigger_autosave();
            }
            Message::NewEnumValueInputChanged(s) => {
                self.new_enum_value_input = s;
            }

            // Informationsmodel Canvas & Studio (Task 011)
            Message::AddClassToDiagram(class_id) => {
                let attr_count = self
                    .project
                    .information_model()
                    .get_class(class_id)
                    .map(|c| c.attributes().len())
                    .unwrap_or(0);
                let node_id = self
                    .project
                    .information_graph_mut()
                    .add_node(class_id, attr_count);
                self.project.sync_information_graph();
                let routes = {
                    let ig = self.project.information_graph();
                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges)
                };
                for r in &routes {
                    self.project.information_graph_mut().update_edge_ports(
                        r.from,
                        r.to,
                        Some(r.from_side),
                        Some(r.to_side),
                    );
                }
                let port_updates: Vec<_> = {
                    let ig = self.project.information_graph();
                    routes
                        .into_iter()
                        .map(|r| {
                            let fc = ig.find_node(r.from).map(|n| n.class_id()).unwrap_or(r.from);
                            let tc = ig.find_node(r.to).map(|n| n.class_id()).unwrap_or(r.to);
                            (fc, tc, r.from_side, r.to_side)
                        })
                        .collect()
                };
                for (fc, tc, from_side, to_side) in port_updates {
                    self.project.information_model_mut().update_relation_ports(
                        fc,
                        tc,
                        Some(from_side),
                        Some(to_side),
                    );
                }
                self.selected_info_class_id = Some(class_id);
                self.selected_info_graph_node_id = Some(node_id);
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeAdded(
                        class_id,
                    ),
                );
                self.trigger_autosave();
            }
            Message::AddClassToDiagramAt(class_id, x, y) => {
                if !self
                    .project
                    .information_graph()
                    .is_class_on_diagram(class_id)
                {
                    let attr_count = self
                        .project
                        .information_model()
                        .get_class(class_id)
                        .map(|c| c.attributes().len())
                        .unwrap_or(0);
                    let snapped_x = (x / crate::features::concept_model::GRID_SIZE).round()
                        * crate::features::concept_model::GRID_SIZE;
                    let snapped_y = (y / crate::features::concept_model::GRID_SIZE).round()
                        * crate::features::concept_model::GRID_SIZE;
                    let node_id = self
                        .project
                        .information_graph_mut()
                        .add_node_at(class_id, snapped_x, snapped_y, attr_count);
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
                    self.selected_info_class_id = Some(class_id);
                    self.selected_info_graph_node_id = Some(node_id);
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeAdded(
                            class_id,
                        ),
                    );
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::NodeMoved {
                            id: class_id,
                            x: snapped_x,
                            y: snapped_y,
                        },
                    );
                    self.trigger_autosave();
                }
            }
            Message::RemoveClassFromDiagram(node_id) => {
                let class_id = self
                    .project
                    .information_graph()
                    .find_node(node_id)
                    .map(|n| n.class_id());
                self.project.information_graph_mut().remove_node(node_id);
                if self.selected_info_graph_node_id == Some(node_id) {
                    self.selected_info_graph_node_id = None;
                }
                if let Some(cid) = class_id {
                    self.broadcast_mutation(
                        &crate::features::collab::protocol::ModelMutation::ClassDiagramNodeRemoved(
                            cid,
                        ),
                    );
                }
                self.trigger_autosave();
            }
            Message::UpdateClassNodePosition(node_id, x, y) => {
                let final_x = (x / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let final_y = (y / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let ig = self.project.information_graph_mut();
                ig.update_node_position(node_id, final_x, final_y);
                let class_id_opt = ig.find_node(node_id).map(|n| n.class_id());

                let d_nodes: Vec<crate::features::concept_model::DiagramNode> =
                    ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                let d_edges: Vec<crate::features::concept_model::DiagramEdge> =
                    ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                let routes = crate::ui::edge_router::EdgeRouter::route_edges(&d_nodes, &d_edges);
                for r in routes {
                    ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                }
                let broadcast_id = class_id_opt.unwrap_or(node_id);
                self.broadcast_node_moved_throttled(broadcast_id, final_x, final_y);
            }
            Message::UpdateClassNodesPositions(moves) => {
                let computed_moves: Vec<(NodeId, f32, f32)> = moves
                    .iter()
                    .map(|&(node_id, x, y)| {
                        (
                            node_id,
                            (x / crate::features::concept_model::GRID_SIZE).round()
                                * crate::features::concept_model::GRID_SIZE,
                            (y / crate::features::concept_model::GRID_SIZE).round()
                                * crate::features::concept_model::GRID_SIZE,
                        )
                    })
                    .collect();

                let mut broadcasts: Vec<(Uuid, f32, f32)> = Vec::new();
                {
                    let ig = self.project.information_graph_mut();
                    for &(node_id, final_x, final_y) in &computed_moves {
                        ig.update_node_position(node_id, final_x, final_y);
                        let class_id_opt = ig.find_node(node_id).map(|n| n.class_id());
                        let broadcast_id = class_id_opt.unwrap_or(node_id);
                        broadcasts.push((broadcast_id, final_x, final_y));
                    }

                    let d_nodes: Vec<crate::features::concept_model::DiagramNode> =
                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                    let d_edges: Vec<crate::features::concept_model::DiagramEdge> =
                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                    let routes =
                        crate::ui::edge_router::EdgeRouter::route_edges(&d_nodes, &d_edges);
                    for r in routes {
                        ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                    }
                }

                for (broadcast_id, final_x, final_y) in broadcasts {
                    self.broadcast_node_moved_throttled(broadcast_id, final_x, final_y);
                }
            }
            Message::SelectInfoGraphNode(node_id_opt) => {
                self.selected_info_graph_node_id = node_id_opt;
                self.selected_info_graph_node_ids = node_id_opt.into_iter().collect();
                self.selected_info_edge = None;
                if let Some(nid) = node_id_opt {
                    if let Some(node) = self.project.information_graph().find_node(nid) {
                        let cid = node.class_id();
                        if self
                            .project
                            .information_model()
                            .get_enumeration(cid)
                            .is_some()
                        {
                            self.selected_info_enum_id = Some(cid);
                            self.selected_info_class_id = None;
                        } else {
                            self.selected_info_class_id = Some(cid);
                            self.selected_info_enum_id = None;
                        }
                    }
                } else {
                    self.selected_info_class_id = None;
                    self.selected_info_enum_id = None;
                }
            }
            Message::SelectInfoGraphNodes(ids) => {
                self.selected_info_graph_node_ids = ids;
                self.selected_info_graph_node_id =
                    self.selected_info_graph_node_ids.iter().next().copied();
                self.selected_info_edge = None;
                if let Some(nid) = self.selected_info_graph_node_id {
                    if let Some(node) = self.project.information_graph().find_node(nid) {
                        let cid = node.class_id();
                        if self
                            .project
                            .information_model()
                            .get_enumeration(cid)
                            .is_some()
                        {
                            self.selected_info_enum_id = Some(cid);
                            self.selected_info_class_id = None;
                        } else {
                            self.selected_info_class_id = Some(cid);
                            self.selected_info_enum_id = None;
                        }
                    }
                } else {
                    self.selected_info_class_id = None;
                    self.selected_info_enum_id = None;
                }
            }
            Message::InfoEdgeSelected(edge) => {
                self.selected_info_edge = edge;
                if edge.is_some() {
                    self.selected_info_graph_node_id = None;
                    self.selected_info_graph_node_ids.clear();
                    self.selected_info_class_id = None;
                }
            }
            Message::InfoEdgeCreated(from, to) => {
                if from == to {
                    return Task::none();
                }
                let graph = self.project.information_graph();
                if graph.find_node(from).is_some() && graph.find_node(to).is_some() {
                    let from_class = graph.find_node(from).unwrap().class_id();
                    let to_class = graph.find_node(to).unwrap().class_id();
                    if graph.find_edge(from, to).is_none() {
                        let is_enum = self
                            .project
                            .information_model()
                            .get_enumeration(from_class)
                            .is_some()
                            || self
                                .project
                                .information_model()
                                .get_enumeration(to_class)
                                .is_some();
                        let initial_kind = if is_enum {
                            RelationKind::Dependency
                        } else {
                            RelationKind::Association
                        };
                        let initial_label = if is_enum {
                            Some("«use»".to_string())
                        } else {
                            None
                        };
                        let rel = ClassRelation::new(
                            from_class,
                            to_class,
                            initial_kind,
                            initial_label.clone(),
                        );
                        self.project.information_model_mut().add_relation(rel);
                        self.project.information_graph_mut().add_relation(
                            from,
                            to,
                            initial_kind,
                            initial_label.clone(),
                        );
                        let routes = {
                            let ig = self.project.information_graph();
                            let nodes: Vec<crate::features::concept_model::DiagramNode> =
                                ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                            let edges: Vec<crate::features::concept_model::DiagramEdge> =
                                ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                            crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges)
                        };
                        for r in &routes {
                            self.project.information_graph_mut().update_edge_ports(
                                r.from,
                                r.to,
                                Some(r.from_side),
                                Some(r.to_side),
                            );
                        }
                        let port_updates: Vec<_> = {
                            let ig = self.project.information_graph();
                            routes
                                .into_iter()
                                .map(|r| {
                                    let fc = ig
                                        .find_node(r.from)
                                        .map(|n| n.class_id())
                                        .unwrap_or(r.from);
                                    let tc =
                                        ig.find_node(r.to).map(|n| n.class_id()).unwrap_or(r.to);
                                    (fc, tc, r.from_side, r.to_side)
                                })
                                .collect()
                        };
                        for (fc, tc, from_side, to_side) in port_updates {
                            self.project.information_model_mut().update_relation_ports(
                                fc,
                                tc,
                                Some(from_side),
                                Some(to_side),
                            );
                        }
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                from_class,
                                to_class,
                                kind: initial_kind,
                                label: initial_label,
                                source_multiplicity: None,
                                target_multiplicity: None,
                                directed: Some(true),
                            },
                        );
                        self.trigger_autosave();
                    }
                    self.selected_info_graph_node_id = None;
                    self.selected_info_class_id = None;
                    self.selected_info_edge = Some((from, to));
                    return operation::focus("info_edge_label_input");
                }
            }
            Message::InfoUpdateEdgeKind(from, to, kind) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                let is_enum_rel = from_class
                    .map(|c| {
                        self.project
                            .information_model()
                            .get_enumeration(c)
                            .is_some()
                    })
                    .unwrap_or(false)
                    || to_class
                        .map(|c| {
                            self.project
                                .information_model()
                                .get_enumeration(c)
                                .is_some()
                        })
                        .unwrap_or(false);
                if is_enum_rel && kind != RelationKind::Dependency {
                    return Task::none();
                }
                let label = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.label().map(String::from));
                let src_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.source_multiplicity());
                let tgt_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.target_multiplicity());
                let directed = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.directed());
                if self
                    .project
                    .information_graph_mut()
                    .update_edge_kind(from, to, kind)
                {
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project
                            .information_model_mut()
                            .update_relation_kind(fc, tc, kind);
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                from_class: fc,
                                to_class: tc,
                                kind,
                                label,
                                source_multiplicity: src_mult,
                                target_multiplicity: tgt_mult,
                                directed,
                            },
                        );
                    }
                    self.trigger_autosave();
                }
            }
            Message::InfoUpdateEdgeLabel(from, to, label) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                let kind = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .map(|e| e.kind())
                    .unwrap_or(RelationKind::Association);
                let src_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.source_multiplicity());
                let tgt_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.target_multiplicity());
                let directed = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.directed());
                let lbl = if label.trim().is_empty() {
                    None
                } else {
                    Some(label)
                };
                if self
                    .project
                    .information_graph_mut()
                    .update_edge_label(from, to, lbl.clone())
                {
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project.information_model_mut().update_relation_label(
                            fc,
                            tc,
                            lbl.clone(),
                        );
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                from_class: fc,
                                to_class: tc,
                                kind,
                                label: lbl,
                                source_multiplicity: src_mult,
                                target_multiplicity: tgt_mult,
                                directed,
                            },
                        );
                    }
                    self.trigger_autosave();
                }
            }
            Message::InfoUpdateEdgeSourceMultiplicity(from, to, mult) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                if self
                    .project
                    .information_graph_mut()
                    .update_edge_source_multiplicity(from, to, mult)
                {
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project
                            .information_model_mut()
                            .update_relation_source_multiplicity(fc, tc, mult);
                        if let Some(edge) = self.project.information_graph().find_edge(from, to) {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                    from_class: fc,
                                    to_class: tc,
                                    kind: edge.kind(),
                                    label: edge.label().map(String::from),
                                    source_multiplicity: mult,
                                    target_multiplicity: edge.target_multiplicity(),
                                    directed: edge.directed(),
                                },
                            );
                        }
                    }
                    self.trigger_autosave();
                }
            }
            Message::InfoUpdateEdgeTargetMultiplicity(from, to, mult) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                if self
                    .project
                    .information_graph_mut()
                    .update_edge_target_multiplicity(from, to, mult)
                {
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project
                            .information_model_mut()
                            .update_relation_target_multiplicity(fc, tc, mult);
                        if let Some(edge) = self.project.information_graph().find_edge(from, to) {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                    from_class: fc,
                                    to_class: tc,
                                    kind: edge.kind(),
                                    label: edge.label().map(String::from),
                                    source_multiplicity: edge.source_multiplicity(),
                                    target_multiplicity: mult,
                                    directed: edge.directed(),
                                },
                            );
                        }
                    }
                    self.trigger_autosave();
                }
            }
            Message::InfoToggleEdgeDirected(from, to, directed) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                if self
                    .project
                    .information_graph_mut()
                    .update_edge_directed(from, to, directed)
                {
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project
                            .information_model_mut()
                            .update_relation_directed(fc, tc, directed);
                        if let Some(edge) = self.project.information_graph().find_edge(from, to) {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                    from_class: fc,
                                    to_class: tc,
                                    kind: edge.kind(),
                                    label: edge.label().map(String::from),
                                    source_multiplicity: edge.source_multiplicity(),
                                    target_multiplicity: edge.target_multiplicity(),
                                    directed: Some(directed),
                                },
                            );
                        }
                    }
                    self.trigger_autosave();
                }
            }
            Message::InfoReverseEdge(from, to) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id());
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id());
                if self
                    .project
                    .information_graph_mut()
                    .reverse_relation(from, to)
                {
                    self.selected_info_edge = Some((to, from));
                    if let (Some(fc), Some(tc)) = (from_class, to_class) {
                        self.project
                            .information_model_mut()
                            .reverse_relation(fc, tc);
                        self.broadcast_mutation(
                            &crate::features::collab::protocol::ModelMutation::ClassRelationDeleted {
                                from_class: fc,
                                to_class: tc,
                            },
                        );
                        if let Some(edge) = self.project.information_graph().find_edge(to, from) {
                            self.broadcast_mutation(
                                &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                    from_class: tc,
                                    to_class: fc,
                                    kind: edge.kind(),
                                    label: edge.label().map(String::from),
                                    source_multiplicity: edge.source_multiplicity(),
                                    target_multiplicity: edge.target_multiplicity(),
                                    directed: edge.directed(),
                                },
                            );
                        }
                    }
                    self.trigger_autosave();
                }
            }
            Message::AddClassRelation(from, to, kind, label) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id())
                    .unwrap_or(from);
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id())
                    .unwrap_or(to);
                self.project.information_model_mut().add_relation(
                    crate::features::information_model::ClassRelation::new(
                        from_class,
                        to_class,
                        kind,
                        label.clone(),
                    ),
                );
                self.project
                    .information_graph_mut()
                    .add_relation(from, to, kind, label);
                self.trigger_autosave();
            }
            Message::DeleteClassRelation(from, to) => {
                let from_class = self
                    .project
                    .information_graph()
                    .find_node(from)
                    .map(|n| n.class_id())
                    .unwrap_or(from);
                let to_class = self
                    .project
                    .information_graph()
                    .find_node(to)
                    .map(|n| n.class_id())
                    .unwrap_or(to);
                self.project
                    .information_graph_mut()
                    .remove_relation(from, to);
                self.project
                    .information_model_mut()
                    .remove_relation(from_class, to_class);
                if self.selected_info_edge == Some((from, to))
                    || self.selected_info_edge == Some((to, from))
                {
                    self.selected_info_edge = None;
                }
                self.broadcast_mutation(
                    &crate::features::collab::protocol::ModelMutation::ClassRelationDeleted {
                        from_class,
                        to_class,
                    },
                );
                self.trigger_autosave();
            }
            Message::InfoCanvasViewportChanged(vp) => {
                self.info_canvas_viewport = vp;
            }
            Message::ToggleInfoSnapToGrid => {
                self.info_snap_to_grid = !self.info_snap_to_grid;
            }
            Message::InfoCanvasZoomIn => {
                self.info_canvas_viewport
                    .zoom_at(Point::new(500.0, 400.0), 1.15);
            }
            Message::InfoCanvasZoomOut => {
                self.info_canvas_viewport
                    .zoom_at(Point::new(500.0, 400.0), 1.0 / 1.15);
            }
            Message::InfoCanvasResetView => {
                self.info_canvas_viewport.reset();
            }
            Message::OpenInfoRelationDialog => {
                let default_from = self.selected_info_graph_node_id.and_then(|id| {
                    self.project.information_graph().find_node(id).map(|n| {
                        let name = if let Some(c) =
                            self.project.information_model().get_class(n.class_id())
                        {
                            c.name().to_string()
                        } else if let Some(e) = self
                            .project
                            .information_model()
                            .get_enumeration(n.class_id())
                        {
                            format!("{} (enum)", e.name())
                        } else {
                            "Element".to_string()
                        };
                        NodeOption { id, label: name }
                    })
                });

                self.info_relation_dialog = Some(RelationDialogState {
                    from_node: default_from,
                    to_node: None,
                    kind: RelationKind::Generalization,
                    label: String::new(),
                    source_multiplicity: None,
                    target_multiplicity: None,
                    error: None,
                });
            }
            Message::CloseInfoRelationDialog => {
                self.info_relation_dialog = None;
            }
            Message::InfoRelationFromChanged(opt) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.from_node = Some(opt);
                    dlg.error = None;
                }
            }
            Message::InfoRelationToChanged(opt) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.to_node = Some(opt);
                    dlg.error = None;
                }
            }
            Message::InfoRelationKindChanged(kind) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.kind = kind;
                }
            }
            Message::InfoRelationLabelChanged(label) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.label = label;
                }
            }
            Message::InfoRelationSourceMultiplicityChanged(mult) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.source_multiplicity = mult;
                }
            }
            Message::InfoRelationTargetMultiplicityChanged(mult) => {
                if let Some(dlg) = &mut self.info_relation_dialog {
                    dlg.target_multiplicity = mult;
                }
            }
            Message::InfoCreateRelation => {
                if let Some(dlg) = &self.info_relation_dialog {
                    match (&dlg.from_node, &dlg.to_node) {
                        (Some(from), Some(to)) => {
                            if from.id == to.id {
                                if let Some(d) = &mut self.info_relation_dialog {
                                    d.error = Some(
                                        "En klasse kan ikke have en relation til sig selv"
                                            .to_string(),
                                    );
                                }
                            } else {
                                let label = if dlg.label.trim().is_empty() {
                                    None
                                } else {
                                    Some(dlg.label.trim().to_string())
                                };
                                let (src_mult, tgt_mult) =
                                    if dlg.kind == RelationKind::Generalization {
                                        (None, None)
                                    } else {
                                        (dlg.source_multiplicity, dlg.target_multiplicity)
                                    };
                                let graph = self.project.information_graph();
                                let from_class = graph
                                    .find_node(from.id)
                                    .map(|n| n.class_id())
                                    .unwrap_or(from.id);
                                let to_class = graph
                                    .find_node(to.id)
                                    .map(|n| n.class_id())
                                    .unwrap_or(to.id);
                                let is_enum = self
                                    .project
                                    .information_model()
                                    .get_enumeration(from_class)
                                    .is_some()
                                    || self
                                        .project
                                        .information_model()
                                        .get_enumeration(to_class)
                                        .is_some();
                                if is_enum && dlg.kind != RelationKind::Dependency {
                                    if let Some(d) = &mut self.info_relation_dialog {
                                        d.error = Some("Relationer til/fra enumerationer skal være Dependency (jf. FDA §5.5)".to_string());
                                    }
                                    return Task::none();
                                }
                                self.project
                                    .information_graph_mut()
                                    .add_relation_with_multiplicities(
                                        from.id,
                                        to.id,
                                        dlg.kind,
                                        label.clone(),
                                        src_mult,
                                        tgt_mult,
                                    );
                                let routes = {
                                    let ig = self.project.information_graph();
                                    let nodes: Vec<crate::features::concept_model::DiagramNode> =
                                        ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                                    let edges: Vec<crate::features::concept_model::DiagramEdge> =
                                        ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                                    crate::ui::edge_router::EdgeRouter::route_edges(&nodes, &edges)
                                };
                                for r in &routes {
                                    self.project.information_graph_mut().update_edge_ports(
                                        r.from,
                                        r.to,
                                        Some(r.from_side),
                                        Some(r.to_side),
                                    );
                                }
                                let port_updates: Vec<_> = {
                                    let ig = self.project.information_graph();
                                    routes
                                        .into_iter()
                                        .map(|r| {
                                            let fc = ig
                                                .find_node(r.from)
                                                .map(|n| n.class_id())
                                                .unwrap_or(r.from);
                                            let tc = ig
                                                .find_node(r.to)
                                                .map(|n| n.class_id())
                                                .unwrap_or(r.to);
                                            (fc, tc, r.from_side, r.to_side)
                                        })
                                        .collect()
                                };
                                for (fc, tc, from_side, to_side) in port_updates {
                                    self.project.information_model_mut().update_relation_ports(
                                        fc,
                                        tc,
                                        Some(from_side),
                                        Some(to_side),
                                    );
                                }
                                let (source_port, target_port) = self
                                    .project
                                    .information_graph()
                                    .find_edge(from.id, to.id)
                                    .map(|e| (e.source_port(), e.target_port()))
                                    .unwrap_or((None, None));
                                let directed = if dlg.kind == RelationKind::Association {
                                    Some(true)
                                } else {
                                    None
                                };
                                self.project.information_model_mut().add_relation(
                                    crate::features::information_model::ClassRelation::with_multiplicities(
                                        from_class,
                                        to_class,
                                        dlg.kind,
                                        label.clone(),
                                        source_port,
                                        target_port,
                                        directed,
                                        src_mult,
                                        tgt_mult,
                                    ),
                                );
                                self.broadcast_mutation(
                                    &crate::features::collab::protocol::ModelMutation::ClassRelationAdded {
                                        from_class,
                                        to_class,
                                        kind: dlg.kind,
                                        label,
                                        source_multiplicity: src_mult,
                                        target_multiplicity: tgt_mult,
                                        directed,
                                    },
                                );
                                self.info_relation_dialog = None;
                                self.trigger_autosave();
                            }
                        }
                        _ => {
                            if let Some(d) = &mut self.info_relation_dialog {
                                d.error = Some(
                                    "Vælg venligst både en kilde- og destinationsklasse"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }

            Message::StartPaletteDrag(item) => {
                self.palette_drag = Some(PaletteDragState {
                    item,
                    start_pos: self.last_cursor_pos,
                    current_pos: self.last_cursor_pos,
                    is_dragging: false,
                });
            }
            Message::PaletteDragMoved(pos) => {
                self.last_cursor_pos = pos;
                if let Some(ref mut drag) = self.palette_drag {
                    let dist = (pos.x - drag.start_pos.x).hypot(pos.y - drag.start_pos.y);
                    if dist > 4.0 {
                        drag.is_dragging = true;
                        drag.current_pos = pos;
                    }
                }
            }
            Message::CanvasDropAt { x, y } => {
                if let Some(drag) = self.palette_drag.take() {
                    match drag.item {
                        PaletteDragItem::Concept(cid) => {
                            return self.update(Message::AddConceptToDiagramAt(cid, x, y));
                        }
                        PaletteDragItem::Class(cid) => {
                            return self.update(Message::AddClassToDiagramAt(cid, x, y));
                        }
                        PaletteDragItem::Enumeration(eid) => {
                            return self.update(Message::AddEnumerationToDiagramAt(eid, x, y));
                        }
                    }
                }
            }
            Message::PaletteDragDropped => {
                if let Some(drag) = self.palette_drag.take() {
                    if !drag.is_dragging {
                        // Click without drag -> select or add
                        match drag.item {
                            PaletteDragItem::Concept(cid) => {
                                if let Some(node) =
                                    self.project.concept_graph().find_node_by_concept(cid)
                                {
                                    self.selected_graph_node_id = Some(node.id());
                                } else {
                                    return self.update(Message::AddConceptToDiagram(cid));
                                }
                            }
                            PaletteDragItem::Class(cid) => {
                                return self.update(Message::SelectInformationClass(Some(cid)));
                            }
                            PaletteDragItem::Enumeration(eid) => {
                                return self
                                    .update(Message::SelectInformationEnumeration(Some(eid)));
                            }
                        }
                    }
                }
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = event::listen_with(|event, status, _window| {
            if let Event::Window(iced::window::Event::Unfocused) = event {
                return Some(Message::CanvasSpacePressed(false));
            }
            if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                return Some(Message::PaletteDragMoved(position));
            }
            if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
                return Some(Message::PaletteDragDropped);
            }
            if status == event::Status::Captured {
                return None;
            }
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key.as_ref() {
                        Key::Named(Named::Tab) => {
                            if modifiers.shift() {
                                Some(Message::FocusPrevious)
                            } else {
                                Some(Message::FocusNext)
                            }
                        }
                        Key::Named(Named::Escape) => Some(Message::EscapePressed),
                        Key::Named(Named::Delete) | Key::Named(Named::Backspace) => {
                            Some(Message::GraphDeleteSelected)
                        }
                        Key::Named(Named::Space) => Some(Message::CanvasSpacePressed(true)),
                        Key::Character(c)
                            if (c == "s" || c == "S")
                                && (modifiers.control() || modifiers.command()) =>
                        {
                            Some(Message::SaveProject)
                        }
                        Key::Character(c)
                            if (c == "b" || c == "B")
                                && (modifiers.control() || modifiers.command()) =>
                        {
                            Some(Message::ToggleLeftSidebar)
                        }
                        Key::Character(c)
                            if (c == "+" || c == "=")
                                && (modifiers.control() || modifiers.command()) =>
                        {
                            Some(Message::CanvasZoomIn)
                        }
                        Key::Character(c)
                            if c == "-" && (modifiers.control() || modifiers.command()) =>
                        {
                            Some(Message::CanvasZoomOut)
                        }
                        Key::Character(c)
                            if c == "0" && (modifiers.control() || modifiers.command()) =>
                        {
                            Some(Message::CanvasResetView)
                        }
                        _ => None,
                    }
                }
                Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                    if let Key::Named(Named::Space) = key.as_ref() {
                        Some(Message::CanvasSpacePressed(false))
                    } else {
                        None
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::CanvasModifiersChanged(modifiers))
                }
                _ => None,
            }
        });

        if let Some(channel) = &self.collab_channel {
            let sub_id = channel.sub_id();
            let collab_sub = Subscription::run_with(sub_id, |&sub_id| {
                iced::futures::stream::unfold(sub_id, |sub_id| async move {
                    let event =
                        crate::features::collab::network::next_registered_collab_event(sub_id)
                            .await?;
                    Some((Message::CollabNetworkEventReceived(event), sub_id))
                })
            });
            Subscription::batch([keyboard_sub, collab_sub])
        } else {
            keyboard_sub
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        static FIRST_FRAME: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FIRST_FRAME.swap(true, std::sync::atomic::Ordering::Relaxed) {
            crate::features::diagnostics::record_startup_milestone("first_frame_rendered");
        }

        // 1. Desktop Header Bar
        const LOGO_SVG_BYTES: &[u8] = include_bytes!("../../assets/kant_logo.svg");
        let brand_logo = svg(svg::Handle::from_memory(LOGO_SVG_BYTES))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0));

        let menu_button = |label: &'static str, menu_type: MenuType| {
            let is_active = self.active_menu == Some(menu_type);
            button(
                row![
                    text(label).size(13).color(if is_active {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_800
                    }),
                    Space::new().width(3),
                    text("▾").size(10).color(if is_active {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_400
                    }),
                ]
                .align_y(Alignment::Center),
            )
            .style(if is_active {
                primary_button_style
            } else {
                secondary_button_style
            })
            .on_press(Message::ToggleMenu(menu_type))
            .padding([4, 10])
        };

        let brand_section = row![
            brand_logo,
            Space::new().width(8),
            text("Kant").size(18).color(ThemeColors::PRIMARY),
            Space::new().width(16),
            menu_button("Filer", MenuType::File),
            Space::new().width(4),
            menu_button("Eksporter", MenuType::Export),
            Space::new().width(4),
            menu_button("Hjælp", MenuType::Help),
            Space::new().width(4),
            menu_button("Samarbejde", MenuType::Collab),
        ]
        .align_y(Alignment::Center);

        let tab_item = |tab: Tab, label: &'static str| {
            let is_active = self.active_tab == tab;
            button(text(label).size(12))
                .style(segmented_tab_button(is_active))
                .on_press(Message::SelectTab(tab))
                .padding([5, 12])
        };

        let tab_pill_bar = container(
            row![
                tab_item(Tab::ConceptList, "1. Begrebsliste"),
                tab_item(Tab::ConceptModel, "2. Begrebsdiagram"),
                tab_item(Tab::InformationModel, "3. Informationsmodel"),
            ]
            .spacing(2)
            .align_y(Alignment::Center),
        )
        .style(pill_container_style)
        .padding(3);

        let collab_header_badge: Element<Message> = if self.collab_state.is_active() {
            let (bg_color, dot_color) = match self.collab_connection_status {
                crate::features::collab::ConnectionStatus::Connected => (
                    iced::Color::from_rgb(0.92, 0.98, 0.94),
                    iced::Color::from_rgb(0.12, 0.65, 0.35),
                ),
                crate::features::collab::ConnectionStatus::Reconnecting => (
                    iced::Color::from_rgb(1.0, 0.97, 0.88),
                    iced::Color::from_rgb(0.85, 0.55, 0.10),
                ),
                _ => (
                    iced::Color::from_rgb(0.95, 0.95, 0.96),
                    iced::Color::from_rgb(0.55, 0.60, 0.68),
                ),
            };

            container(
                row![
                    text("●").size(10).color(dot_color),
                    Space::new().width(4),
                    text(self.collab_status_summary())
                        .size(11)
                        .color(ThemeColors::SLATE_800),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    color: dot_color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .padding([3, 10])
            .into()
        } else {
            Space::new().width(0).height(0).into()
        };

        let header_right = row![
            collab_header_badge,
            Space::new().width(6),
            self.view_git_status_badge(),
            Space::new().width(8),
            text(self.project.metadata().name())
                .size(12)
                .color(ThemeColors::SLATE_600),
        ]
        .align_y(Alignment::Center);

        let header_bar = container(
            row![
                container(brand_section)
                    .width(Length::FillPortion(1))
                    .align_x(Alignment::Start),
                tab_pill_bar,
                container(header_right)
                    .width(Length::FillPortion(1))
                    .align_x(Alignment::End),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(card_container_style)
        .padding([8, 16])
        .width(Length::Fill);

        // 2. Modale dialoger (Stack Overlay)
        let maybe_diagnostics_modal: Option<Element<Message>> = self
            .diagnostics_modal
            .as_ref()
            .map(|modal| self.view_diagnostics_modal(modal));

        let maybe_publish_modal: Option<Element<Message>> = self
            .publish_modal
            .as_ref()
            .map(|modal| self.view_publish_modal(modal));

        let maybe_history_modal: Option<Element<Message>> = self
            .history_modal
            .as_ref()
            .map(|modal| self.view_history_modal(modal));

        let maybe_conflict_resolver_modal: Option<Element<Message>> = self
            .conflict_resolver_modal
            .as_ref()
            .map(|modal| self.view_conflict_resolver_modal(modal));

        let maybe_git_connection_modal: Option<Element<Message>> = self
            .git_connection_modal
            .as_ref()
            .map(|modal| self.view_git_connection_modal(modal));

        let maybe_git_clone_modal: Option<Element<Message>> = self
            .git_clone_modal
            .as_ref()
            .map(|modal| self.view_git_clone_modal(modal));

        let maybe_excel_schema_guide_modal: Option<Element<Message>> =
            if self.excel_schema_guide_open {
                Some(self.view_excel_schema_guide_modal())
            } else {
                None
            };

        let maybe_excel_import_preview_modal: Option<Element<Message>> = self
            .excel_import_preview
            .as_ref()
            .map(|modal| self.view_excel_import_preview_modal(modal));

        let maybe_metadata_modal: Option<Element<Message>> =
            self.metadata_modal.as_ref().map(|meta_state| {
                let title_row = row![
                    text("📋 Modelomslag & Metadata (FDA Tabel D & E)")
                        .size(17)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::CloseMetadataModal)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center);

                let subtitle =
                    text("Rediger overordnede metadata for modelprojektet jf. FDA Modelreglerne:")
                        .size(12)
                        .color(ThemeColors::TEXT_MUTED);

                // 1. Modelnavn
                let name_field = column![
                    text("Modelnavn *").size(12).color(ThemeColors::SLATE_700),
                    text_input("Nyt FDA Modelprojekt", &meta_state.name)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(MetadataField::Name, val))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4);

                // 2. Beskrivelse
                let desc_field = column![
                    text("Modelbeskrivelse")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "Formål og omfang jf. FDA Modelreglerne...",
                        &meta_state.description
                    )
                    .style(modern_input_style)
                    .on_input(|val| Message::UpdateMetadataField(MetadataField::Description, val))
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4);

                // 3. Model-URI
                let uri_field = column![
                    text("Model-URI *").size(12).color(ThemeColors::SLATE_700),
                    text_input("https://data.gov.dk/model/core/...", &meta_state.uri)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(MetadataField::Uri, val))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4);

                // 4. Livscyklus & Godkendelse (Tabel D & E)
                let status_pick = pick_list(
                    &ModelStatus::ALL[..],
                    Some(meta_state.status),
                    Message::UpdateMetadataStatus,
                )
                .padding(7)
                .width(Length::Fill);

                let status_field = column![
                    text("Modelstatus (Tabel E)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    status_pick,
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let approval_pick = pick_list(
                    &ApprovalStatus::ALL[..],
                    Some(meta_state.approval_status),
                    Message::UpdateMetadataApprovalStatus,
                )
                .padding(7)
                .width(Length::Fill);

                let approval_field = column![
                    text("Godkendelsesstatus")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    approval_pick,
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let status_approval_row = row![status_field, approval_field].spacing(12);

                let scope_pick = pick_list(
                    &ModelScope::ALL[..],
                    Some(meta_state.model_scope),
                    Message::UpdateMetadataScope,
                )
                .padding(7)
                .width(Length::Fill);

                let scope_field = column![
                    text("Modelomfang").size(12).color(ThemeColors::SLATE_700),
                    scope_pick,
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let approved_by_field = column![
                    text("Godkendt af").size(12).color(ThemeColors::SLATE_700),
                    text_input("f.eks. FDA Fagråd eller Styrelse", &meta_state.approved_by)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(
                            MetadataField::ApprovedBy,
                            val
                        ))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let scope_approved_row = row![scope_field, approved_by_field].spacing(12);

                // 5. Ansvar & Versionering
                let domain_field = column![
                    text("Emneområde (§26)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input("f.eks. Vej og Trafik", &meta_state.domain_area)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(
                            MetadataField::DomainArea,
                            val
                        ))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let org_field = column![
                    text("Ansvarlig organisation")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "f.eks. Styrelsen for Dataforsyning eller Vejdirektoratet",
                        &meta_state.responsible_org,
                    )
                    .style(modern_input_style)
                    .on_input(|val| Message::UpdateMetadataField(
                        MetadataField::ResponsibleOrg,
                        val
                    ))
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let domain_org_row = row![domain_field, org_field].spacing(12);

                let version_field = column![
                    text("Version").size(12).color(ThemeColors::SLATE_700),
                    text_input("0.1.0", &meta_state.version)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(MetadataField::Version, val))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let lang_field = column![
                    text("Sprog").size(12).color(ThemeColors::SLATE_700),
                    text_input("da", &meta_state.language)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(MetadataField::Language, val))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let date_field = column![
                    text("Senest ændret (ISO)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input("YYYY-MM-DD", &meta_state.date_modified)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(
                            MetadataField::DateModified,
                            val
                        ))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let version_lang_row = row![version_field, lang_field, date_field].spacing(12);

                let notes_field = column![
                    text("Revisionsnote").size(12).color(ThemeColors::SLATE_700),
                    text_input(
                        "f.eks. Væsentlige ændringer i relationer efter offentlig høring",
                        &meta_state.version_notes
                    )
                    .style(modern_input_style)
                    .on_input(|val| Message::UpdateMetadataField(MetadataField::VersionNotes, val))
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4);

                // 6. Proveniens & Lovgrundlag
                let legal_field = column![
                    text("Lovgrundlag (ELI URI'er, kommasepareret)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "https://www.retsinformation.dk/eli/lta/...",
                        &meta_state.legal_sources
                    )
                    .style(modern_input_style)
                    .on_input(|val| Message::UpdateMetadataField(MetadataField::LegalSources, val))
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4);

                let source_field = column![
                    text("Kilde (ekstern standard)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input("f.eks. ISO 19107 eller INSPIRE", &meta_state.source)
                        .style(modern_input_style)
                        .on_input(|val| Message::UpdateMetadataField(MetadataField::Source, val))
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let derived_field = column![
                    text("Udledt af (URI)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "https://data.gov.dk/model/core/...",
                        &meta_state.was_derived_from
                    )
                    .style(modern_input_style)
                    .on_input(|val| Message::UpdateMetadataField(
                        MetadataField::WasDerivedFrom,
                        val
                    ))
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4)
                .width(Length::FillPortion(1));

                let source_derived_row = row![source_field, derived_field].spacing(12);

                // 7. Action knapper
                let actions = row![
                    Space::new().width(Length::Fill),
                    button(text("Annuller").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::CloseMetadataModal)
                        .padding([6, 14]),
                    button(text("💾 Gem Omslag").size(12))
                        .style(primary_button_style)
                        .on_press(Message::SaveMetadataModal)
                        .padding([6, 16]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                let form_col = column![
                    name_field,
                    desc_field,
                    uri_field,
                    status_approval_row,
                    scope_approved_row,
                    domain_org_row,
                    version_lang_row,
                    notes_field,
                    legal_field,
                    source_derived_row,
                ]
                .spacing(12);

                let scrollable_form = scrollable(form_col).height(Length::Fixed(460.0));

                let dialog_col = column![
                    title_row,
                    subtitle,
                    Space::new().height(4),
                    scrollable_form,
                    Space::new().height(4),
                    actions,
                ]
                .spacing(10);

                let modal_card = container(dialog_col)
                    .style(modal_card_style)
                    .padding(24)
                    .width(Length::Fixed(640.0));

                container(modal_card)
                    .style(modal_backdrop_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            });

        let maybe_start_session_modal: Option<Element<Message>> =
            self.start_session_modal.as_ref().map(|modal_state| {
                let title_row = row![
                    text("🌐 Start Live Session (Vært)")
                        .size(17)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::CloseCollabModal)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center);

                let subtitle = text("Start en end-to-end krypteret (E2EE) samarbejdssession. Vælg relay-server og del sessionskoden med dine deltagere:")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED);

                let preset_pick = pick_list(
                    &RelayServerPreset::ALL[..],
                    Some(modal_state.preset),
                    Message::CollabPresetSelected,
                )
                .padding(7)
                .width(Length::Fill);

                let preset_field = column![
                    text("Relay Server Preset").size(12).color(ThemeColors::SLATE_700),
                    preset_pick,
                ]
                .spacing(4);

                let custom_url_field = if modal_state.preset == RelayServerPreset::Custom
                    || modal_state.preset == RelayServerPreset::InternalOrg
                {
                    column![
                        text("Server WebSocket URL").size(12).color(ThemeColors::SLATE_700),
                        text_input("wss://relay.eksempel.dk/ws", &modal_state.custom_url)
                            .style(modern_input_style)
                            .on_input(Message::CollabCustomUrlChanged)
                            .padding(8)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                } else {
                    column![
                        text("Aktiv Server URL").size(12).color(ThemeColors::SLATE_500),
                        container(
                            text(modal_state.current_url())
                                .size(12)
                                .color(ThemeColors::SLATE_700)
                        )
                        .style(|_| container::Style {
                            background: Some(iced::Background::Color(ThemeColors::SURFACE_BG)),
                            border: iced::Border {
                                color: ThemeColors::SURFACE_BORDER,
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        })
                        .padding([8, 10])
                        .width(Length::Fill),
                    ]
                    .spacing(4)
                };

                let remember_checkbox = row![
                    checkbox(modal_state.remember_choice)
                        .on_toggle(Message::CollabToggleRememberPreset)
                        .size(16),
                    Space::new().width(6),
                    text("Husk servervalg til fremtidige sessioner")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                ]
                .align_y(Alignment::Center);

                let token_str = modal_state.ticket.to_token();
                let token_box = column![
                    text("Sessionskode (Sessionsbillet med E2EE-nøgle)")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    container(
                        text(token_str)
                            .size(11)
                            .color(ThemeColors::PRIMARY)
                    )
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::PRIMARY,
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4);

                let copy_label = if modal_state.copied {
                    "✓ Kopieret!"
                } else {
                    "📋 Kopiér sessionskode"
                };

                let copy_btn = button(text(copy_label).size(12))
                    .style(if modal_state.copied {
                        primary_button_style
                    } else {
                        secondary_button_style
                    })
                    .on_press(Message::CollabCopyTicket)
                    .padding([6, 14]);

                let actions = row![
                    copy_btn,
                    Space::new().width(Length::Fill),
                    button(text("Annuller").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::CloseCollabModal)
                        .padding([6, 14]),
                    button(text("🚀 Start Session").size(12))
                        .style(primary_button_style)
                        .on_press(Message::CollabStartSession)
                        .padding([6, 16]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                let dialog_col = column![
                    title_row,
                    subtitle,
                    preset_field,
                    custom_url_field,
                    remember_checkbox,
                    token_box,
                    actions,
                ]
                .spacing(12);

                let modal_card = container(dialog_col)
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
            });

        let maybe_join_session_modal: Option<Element<Message>> =
            self.join_session_modal.as_ref().map(|modal_state| {
                let title_row = row![
                    text("🌐 Deltag i Live Session (Gæst)")
                        .size(17)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::CloseCollabModal)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center);

                let subtitle = text("Indsæt den sessionskode du har modtaget fra sessionens vært:")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED);

                let validation_feedback: Element<Message> =
                    if let Some(ticket) = &modal_state.parsed_ticket {
                        container(
                            row![
                                text("✓")
                                    .size(14)
                                    .color(iced::Color::from_rgb(0.12, 0.65, 0.35)),
                                Space::new().width(6),
                                text(format!(
                                    "Gyldig kode (Server: {} | Rum: {})",
                                    ticket.relay_url,
                                    ticket.room_id.as_str()
                                ))
                                .size(12)
                                .color(iced::Color::from_rgb(0.12, 0.65, 0.35)),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .padding([4, 8])
                        .into()
                    } else if let Some(err) = &modal_state.error_message {
                        container(
                            row![
                                text("⚠️").size(12),
                                Space::new().width(6),
                                text(err)
                                    .size(12)
                                    .color(iced::Color::from_rgb(0.85, 0.20, 0.20)),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .padding([4, 8])
                        .into()
                    } else {
                        container(
                            text("Format: kant:v1:<base64-payload>")
                                .size(11)
                                .color(ThemeColors::TEXT_MUTED),
                        )
                        .padding([4, 8])
                        .into()
                    };

                let token_field = column![
                    text("Sessionskode *")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "Indsæt sessionskode her (f.eks. kant:v1:...)",
                        &modal_state.token_input
                    )
                    .style(modern_input_style)
                    .on_input(Message::CollabJoinTokenChanged)
                    .padding(8)
                    .width(Length::Fill),
                    validation_feedback,
                ]
                .spacing(4);

                let mut join_btn = button(text("🔗 Forbind").size(12))
                    .style(primary_button_style)
                    .padding([6, 16]);

                if modal_state.is_valid() {
                    join_btn = join_btn.on_press(Message::CollabJoinSession);
                }

                let actions = row![
                    Space::new().width(Length::Fill),
                    button(text("Annuller").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::CloseCollabModal)
                        .padding([6, 14]),
                    join_btn,
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                let dialog_col = column![title_row, subtitle, token_field, actions,].spacing(14);

                let modal_card = container(dialog_col)
                    .style(modal_card_style)
                    .padding(24)
                    .width(Length::Fixed(540.0));

                container(modal_card)
                    .style(modal_backdrop_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            });

        let maybe_guest_ended_modal: Option<Element<Message>> =
            self.guest_ended_notice.as_ref().map(|notice| {
                let title_row = row![
                    text("⚠️ Live Session Afsluttet")
                        .size(17)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::CollabGuestDismissEndedModal)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center);

                let notice_text = text(&notice.message)
                    .size(13)
                    .color(ThemeColors::SLATE_700);

                let explanation = text("Værten har afsluttet sessionen. Du har modellen i hukommelsen (RAM) og kan gemme en kopi som en ny lokal modelprojektfil før sessionen lukkes.")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED);

                let actions = row![
                    Space::new().width(Length::Fill),
                    button(text("Luk uden at gemme").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::CollabGuestDismissEndedModal)
                        .padding([6, 14]),
                    button(text("💾 Gem som kopi...").size(12))
                        .style(primary_button_style)
                        .on_press(Message::SaveProjectAsDialog)
                        .padding([6, 16]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                let dialog_col = column![
                    title_row,
                    notice_text,
                    explanation,
                    actions,
                ]
                .spacing(14);

                let modal_card = container(dialog_col)
                    .style(modal_card_style)
                    .padding(24)
                    .width(Length::Fixed(520.0));

                container(modal_card)
                    .style(modal_backdrop_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            });

        let maybe_file_dialog_modal: Option<Element<Message>> = self.file_dialog_mode.map(|mode| {
            let (mode_title, confirm_label) = match mode {
                FileDialogMode::Open => ("Åbn FDA Modelprojekt", "Åbn"),
                FileDialogMode::SaveAs => ("Gem Modelprojekt som...", "Gem"),
            };

            let mut dialog_body = column![
                row![
                    text(mode_title).size(16).color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::CloseFileDialog)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center),
                text("Angiv filsti eller gennemse filer på maskinen:")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED),
                row![
                    text_input(
                        "Filsti (f.eks. model.kant.json)...",
                        &self.file_dialog_input
                    )
                    .style(modern_input_style)
                    .on_input(Message::FileDialogInputChanged)
                    .on_submit(Message::ConfirmFileDialog)
                    .padding(8)
                    .width(Length::Fill),
                    button(text("🖥️ Gennemse...").size(12))
                        .style(secondary_button_style)
                        .on_press(match mode {
                            FileDialogMode::Open => Message::OpenProjectDialog,
                            FileDialogMode::SaveAs => Message::SaveProjectAsDialog,
                        })
                        .padding([8, 12]),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(12);

            if mode == FileDialogMode::Open {
                let local_files =
                    crate::ui::file_dialog::scan_local_project_files(&PathBuf::from("."));
                if !local_files.is_empty() {
                    let mut chips = column![text("Filer i projektmappen:")
                        .size(12)
                        .color(ThemeColors::TEXT_MUTED)]
                    .spacing(6);

                    let mut chip_row = row![].spacing(6);
                    for f in local_files {
                        let name = f
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("model.kant.json")
                            .to_string();
                        chip_row = chip_row.push(
                            button(text(format!("📄 {}", name)).size(11))
                                .style(secondary_button_style)
                                .on_press(Message::OpenProjectFile(f))
                                .padding([3, 8]),
                        );
                    }
                    chips = chips.push(chip_row);
                    dialog_body = dialog_body.push(chips);
                }
            }

            let actions_row = row![
                Space::new().width(Length::Fill),
                button(text("Annuller").size(12))
                    .style(secondary_button_style)
                    .on_press(Message::CloseFileDialog)
                    .padding([6, 14]),
                button(text(confirm_label).size(12))
                    .style(primary_button_style)
                    .on_press(Message::ConfirmFileDialog)
                    .padding([6, 16]),
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            dialog_body = dialog_body.push(actions_row);

            let modal_card = container(dialog_body)
                .style(modal_card_style)
                .padding(24)
                .width(Length::Fixed(520.0));

            container(modal_card)
                .style(modal_backdrop_style)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        });

        let maybe_relation_modal: Option<Element<Message>> =
            self.relation_dialog.as_ref().map(|d| {
                let node_options: Vec<NodeOption> = self
                    .project
                    .concept_graph()
                    .nodes()
                    .iter()
                    .map(|n| NodeOption {
                        id: n.id(),
                        label: n.label().to_string(),
                    })
                    .collect();

                let kinds = [RelationKind::Generalization, RelationKind::Association];

                let mut dialog_col = column![row![
                    text("Opret Ny Relation i Begrebsmodel")
                        .size(16)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(13))
                        .style(secondary_button_style)
                        .on_press(Message::GraphCloseRelationDialog)
                        .padding([3, 7]),
                ]
                .align_y(Alignment::Center),]
                .spacing(12);

                if let Some(err) = &d.error {
                    dialog_col = dialog_col.push(
                        container(
                            text(format!("⚠️ {}", err))
                                .size(12)
                                .color(ThemeColors::ACCENT_RED),
                        )
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(
                                ThemeColors::ACCENT_RED_LIGHT,
                            )),
                            border: iced::Border {
                                color: ThemeColors::ACCENT_RED,
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        })
                        .padding([6, 10])
                        .width(Length::Fill),
                    );
                }

                let from_pick = pick_list(
                    node_options.clone(),
                    d.from_node.clone(),
                    Message::GraphRelationFromChanged,
                )
                .placeholder("Vælg kilde...")
                .padding(7)
                .width(Length::Fixed(220.0));

                let to_pick = pick_list(
                    node_options,
                    d.to_node.clone(),
                    Message::GraphRelationToChanged,
                )
                .placeholder("Vælg mål...")
                .padding(7)
                .width(Length::Fixed(220.0));

                let kind_pick = pick_list(
                    kinds.to_vec(),
                    Some(d.kind),
                    Message::GraphRelationKindChanged,
                )
                .padding(7)
                .width(Length::Fixed(220.0));

                let fields = column![
                    row![
                        column![
                            text("Kildebegreb (fra):")
                                .size(12)
                                .color(ThemeColors::TEXT_MUTED),
                            from_pick
                        ]
                        .spacing(4),
                        column![
                            text("Relationstype:")
                                .size(12)
                                .color(ThemeColors::TEXT_MUTED),
                            kind_pick
                        ]
                        .spacing(4),
                    ]
                    .spacing(16),
                    row![
                        column![
                            text("Målbegreb (til):")
                                .size(12)
                                .color(ThemeColors::TEXT_MUTED),
                            to_pick
                        ]
                        .spacing(4),
                        if d.kind == RelationKind::Association {
                            column![
                                text("Associationsnavn (naturligt sprog):")
                                    .size(12)
                                    .color(ThemeColors::TEXT_MUTED),
                                text_input("f.eks. ejer, anvender...", &d.label)
                                    .style(modern_input_style)
                                    .on_input(Message::GraphRelationLabelChanged)
                                    .padding(7)
                                    .width(Length::Fixed(220.0)),
                            ]
                            .spacing(4)
                        } else {
                            column![
                                text("Generaliseringsregel:")
                                    .size(12)
                                    .color(ThemeColors::TEXT_MUTED),
                                text("Specialisering ➔ Superklasse (hvid pil)")
                                    .size(11)
                                    .color(ThemeColors::SLATE_600),
                            ]
                            .spacing(4)
                        },
                    ]
                    .spacing(16),
                ]
                .spacing(12);

                dialog_col = dialog_col.push(fields);

                let actions = row![
                    Space::new().width(Length::Fill),
                    button(text("Annuller").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::GraphCloseRelationDialog)
                        .padding([6, 14]),
                    button(text("Opret Relation").size(12))
                        .style(primary_button_style)
                        .on_press(Message::GraphCreateRelation)
                        .padding([6, 16]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                dialog_col = dialog_col.push(actions);

                let modal_card = container(dialog_col)
                    .style(modal_card_style)
                    .padding(24)
                    .width(Length::Fixed(500.0));

                container(modal_card)
                    .style(modal_backdrop_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            });

        let maybe_quick_create_modal: Option<Element<Message>> =
            self.quick_create.as_ref().map(|qc| {
                let mut dialog_col = column![
                    row![
                        text("Nyt Begreb på Lærred")
                            .size(16)
                            .color(ThemeColors::SLATE_900),
                        Space::new().width(Length::Fill),
                        button(text("✕").size(13))
                            .style(secondary_button_style)
                            .on_press(Message::QuickCreateCancel)
                            .padding([3, 7]),
                    ]
                    .align_y(Alignment::Center),
                    text(format!(
                        "Opretter begreb ved ({:.0}, {:.0}) på lærredet jf. FDA Modelreglerne:",
                        qc.position.0, qc.position.1
                    ))
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED),
                ]
                .spacing(12);

                if let Some(err) = &qc.validation_error {
                    dialog_col = dialog_col.push(
                        container(
                            text(format!("⚠️ {}", err))
                                .size(12)
                                .color(ThemeColors::ACCENT_RED),
                        )
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(
                                ThemeColors::ACCENT_RED_LIGHT,
                            )),
                            border: iced::Border {
                                color: ThemeColors::ACCENT_RED,
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        })
                        .padding([6, 10])
                        .width(Length::Fill),
                    );
                }

                let term_field = column![
                    text("Foretrukken term *")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input("Foretrukken term (f.eks. Godsvogn)...", &qc.preferred_term)
                        .id("quick_create_term_input")
                        .style(modern_input_style)
                        .on_input(Message::QuickCreateTermChanged)
                        .on_submit(Message::QuickCreateSubmit)
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(4);

                let def_field = column![
                    text("Definition (Aristoteles' formel) *")
                        .size(12)
                        .color(ThemeColors::SLATE_700),
                    text_input(
                        "Overordnet begreb + adskillende egenskaber...",
                        &qc.definition
                    )
                    .style(modern_input_style)
                    .on_input(Message::QuickCreateDefinitionChanged)
                    .on_submit(Message::QuickCreateSubmit)
                    .padding(8)
                    .width(Length::Fill),
                ]
                .spacing(4);

                let is_local = qc.belongs_to_domain == BelongsToDomain::Yes;
                let domain_row = row![
                    text("Tilknytning:").size(12).color(ThemeColors::SLATE_700),
                    button(text("Lokalt begreb (FDA Sand)").size(11))
                        .style(if is_local {
                            primary_button_style
                        } else {
                            secondary_button_style
                        })
                        .on_press(Message::QuickCreateDomainChanged(BelongsToDomain::Yes))
                        .padding([4, 8]),
                    button(text("Indlånt begreb (FDA Blå)").size(11))
                        .style(if !is_local {
                            primary_button_style
                        } else {
                            secondary_button_style
                        })
                        .on_press(Message::QuickCreateDomainChanged(BelongsToDomain::No))
                        .padding([4, 8]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                dialog_col = dialog_col.push(term_field);
                dialog_col = dialog_col.push(def_field);
                dialog_col = dialog_col.push(domain_row);

                let actions = row![
                    Space::new().width(Length::Fill),
                    button(text("Annuller").size(12))
                        .style(secondary_button_style)
                        .on_press(Message::QuickCreateCancel)
                        .padding([6, 14]),
                    button(text("Opret Begreb").size(12))
                        .style(primary_button_style)
                        .on_press(Message::QuickCreateSubmit)
                        .padding([6, 16]),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                dialog_col = dialog_col.push(actions);

                let modal_card = container(dialog_col)
                    .style(modal_card_style)
                    .padding(24)
                    .width(Length::Fixed(480.0));

                container(modal_card)
                    .style(modal_backdrop_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            });

        let maybe_menu_overlay: Option<Element<Message>> = self.active_menu.map(|menu_type| {
            let menu_item = |icon: &'static str, label: &'static str, msg: Message| {
                button(
                    row![
                        text(icon).size(14),
                        Space::new().width(10),
                        text(label).size(13).color(ThemeColors::SLATE_800),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(|_theme, status| {
                    let background = match status {
                        button::Status::Hovered => {
                            Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT))
                        }
                        button::Status::Pressed => {
                            Some(iced::Background::Color(ThemeColors::SURFACE_BORDER))
                        }
                        _ => None,
                    };
                    button::Style {
                        background,
                        text_color: ThemeColors::SLATE_800,
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    }
                })
                .on_press(msg)
                .padding([7, 10])
                .width(Length::Fill)
            };

            let make_separator = || {
                container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(|_| {
                    container::Style {
                        background: Some(iced::Background::Color(ThemeColors::SURFACE_BORDER)),
                        ..Default::default()
                    }
                })
            };

            let (left_offset, menu_body) = match menu_type {
                MenuType::File => {
                    let recent_item =
                        |icon: &'static str, title: String, path_str: String, msg: Message| {
                            button(
                                row![
                                    text(icon).size(14),
                                    Space::new().width(8),
                                    column![
                                        text(title).size(12).color(ThemeColors::SLATE_800),
                                        text(path_str).size(9).color(ThemeColors::TEXT_MUTED),
                                    ]
                                    .spacing(1),
                                ]
                                .align_y(Alignment::Center),
                            )
                            .style(|_theme, status| {
                                let background = match status {
                                    button::Status::Hovered => {
                                        Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT))
                                    }
                                    button::Status::Pressed => {
                                        Some(iced::Background::Color(ThemeColors::SURFACE_BORDER))
                                    }
                                    _ => None,
                                };
                                button::Style {
                                    background,
                                    text_color: ThemeColors::SLATE_800,
                                    border: iced::Border {
                                        radius: 6.0.into(),
                                        ..Default::default()
                                    },
                                    shadow: iced::Shadow::default(),
                                    ..Default::default()
                                }
                            })
                            .on_press(msg)
                            .width(Length::Fill)
                            .padding([4, 8])
                        };

                    let mut file_col = column![
                        text("PROJEKT & FILER")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item("➕", "Nyt projekt", Message::NewProject),
                        menu_item("📄", "Åbn projektfil...", Message::OpenProjectDialog),
                        menu_item("📁", "Åbn modelmappe...", Message::OpenProjectFolderDialog),
                        menu_item("💾", "Gem", Message::SaveProject),
                        menu_item("💾", "Gem som...", Message::SaveProjectAsDialog),
                        menu_item(
                            "📥",
                            "Importér begreber (Excel)...",
                            Message::ImportConceptsExcelDialog
                        ),
                    ]
                    .spacing(2);

                    let existing_recents: Vec<_> = self
                        .recent_store
                        .recent_paths
                        .iter()
                        .filter(|p| p.exists())
                        .take(5)
                        .cloned()
                        .collect();

                    if !existing_recents.is_empty() {
                        file_col = file_col.push(Space::new().height(4));
                        file_col = file_col.push(make_separator());
                        file_col = file_col.push(Space::new().height(4));
                        file_col = file_col.push(
                            text("SENESTE MODELLER (RECENT)")
                                .size(10)
                                .color(ThemeColors::TEXT_MUTED),
                        );
                        file_col = file_col.push(Space::new().height(2));

                        for path in existing_recents {
                            let is_dir = path.is_dir() || path.join(".kant").is_dir();
                            let icon = if is_dir { "📁" } else { "📄" };
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("model")
                                .to_string();
                            let full_str = path.display().to_string();
                            file_col = file_col.push(recent_item(
                                icon,
                                name,
                                full_str,
                                Message::OpenRecentProject(path),
                            ));
                        }
                    }

                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(make_separator());
                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(
                        text("MODELINDSTILLINGER")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                    );
                    file_col = file_col.push(Space::new().height(2));
                    file_col = file_col.push(menu_item(
                        "📋",
                        "Modelomslag & Metadata...",
                        Message::OpenMetadataModal,
                    ));
                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(make_separator());
                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(
                        text("VERSIONSSTYRING (GIT)")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                    );
                    file_col = file_col.push(Space::new().height(2));
                    file_col =
                        file_col.push(menu_item("📥", "Hent ændringer (Pull)", Message::PullModel));
                    file_col = file_col.push(menu_item(
                        "🚀",
                        "Commit og send ændringer (pull, add & push)...",
                        Message::OpenPublishModal,
                    ));
                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(make_separator());
                    file_col = file_col.push(Space::new().height(4));
                    file_col = file_col.push(menu_item(
                        "⏳",
                        "Modelhistorik & Tidslinje...",
                        Message::OpenModelHistoryModal,
                    ));
                    file_col = file_col.push(menu_item(
                        "⚙️",
                        "Git indstillinger...",
                        Message::OpenGitConnectionModal,
                    ));

                    (136.0, file_col.width(Length::Fixed(360.0)))
                }
                MenuType::Export => {
                    let svg_label = match self.active_tab {
                        Tab::ConceptModel => "Vektor-SVG: Begrebsmodel...",
                        Tab::InformationModel => "Vektor-SVG: Informationsmodel...",
                        _ => "Vektor-SVG: Aktivt diagram...",
                    };

                    let export_col = column![
                        text("VEKTOR-DIAGRAMMER (SVG)")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item("🖼️", svg_label, Message::ExportActiveDiagramSvgDialog),
                        Space::new().height(4),
                        make_separator(),
                        Space::new().height(4),
                        text("BEGREBSLISTE (TABEL)")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item(
                            "📊",
                            "Begrebsliste (Excel .xlsx)...",
                            Message::ExportConceptListXlsxDialog,
                        ),
                        menu_item(
                            "📄",
                            "Tom FDA Begrebsliste-skabelon (.xlsx)...",
                            Message::ExportFdaTemplateXlsxDialog,
                        ),
                        menu_item(
                            "📋",
                            "Begrebsliste (CSV)...",
                            Message::ExportConceptListCsvDialog,
                        ),
                        Space::new().height(4),
                        make_separator(),
                        Space::new().height(4),
                        text("FDA AFLEVERINGSRAPPORT")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item(
                            "📝",
                            "Modelrapport (Markdown)...",
                            Message::ExportModelReportMarkdownDialog,
                        ),
                        menu_item(
                            "🌐",
                            "Modelrapport (HTML)...",
                            Message::ExportModelReportHtmlDialog,
                        ),
                        Space::new().height(4),
                        make_separator(),
                        Space::new().height(4),
                        text("MASKINLÆSBAR MODEL (XMI & RDF)")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item(
                            "🏛️",
                            "Enterprise Architect (XMI 2.1)...",
                            Message::ExportModelXmiDialog,
                        ),
                        menu_item(
                            "🌐",
                            "Begrebsliste (W3C SKOS Turtle)...",
                            Message::ExportModelSkosDialog,
                        ),
                        menu_item(
                            "📐",
                            "Informationsmodel (W3C SHACL/OWL)...",
                            Message::ExportModelShaclDialog,
                        ),
                    ]
                    .spacing(2)
                    .width(Length::Fixed(290.0));

                    (212.0, export_col)
                }
                MenuType::Help => (
                    310.0,
                    column![
                        text("DOKUMENTATION & HJÆLP")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        menu_item("📖", "FDA Modelregler v2.1 ↗", Message::OpenModelRules),
                        menu_item(
                            "ℹ️",
                            "Excel Skemaguide (FDA)...",
                            Message::OpenExcelSchemaGuideModal
                        ),
                        menu_item(
                            "🔍",
                            "System- og grafikdiagnostik...",
                            Message::OpenDiagnosticsModal
                        ),
                    ]
                    .spacing(2)
                    .width(Length::Fixed(240.0)),
                ),
                MenuType::Collab => (
                    384.0,
                    column![
                        text("LIVE SAMARBEJDE (E2EE)")
                            .size(10)
                            .color(ThemeColors::TEXT_MUTED),
                        Space::new().height(2),
                        if self.collab_state.is_active() {
                            column![menu_item("🔴", "Afbryd session", Message::CollabDisconnect)]
                        } else {
                            column![
                                menu_item(
                                    "👑",
                                    "Start session (Vært)...",
                                    Message::OpenStartSessionModal
                                ),
                                menu_item(
                                    "👥",
                                    "Deltag i session (Gæst)...",
                                    Message::OpenJoinSessionModal
                                ),
                            ]
                            .spacing(2)
                        },
                    ]
                    .spacing(2)
                    .width(Length::Fixed(240.0)),
                ),
            };

            let menu_card = container(menu_body)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::SURFACE_CARD)),
                    border: iced::Border {
                        color: ThemeColors::SURFACE_BORDER,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.12),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                })
                .padding(10);

            let backdrop = mouse_area(
                container(Space::new().width(Length::Fill).height(Length::Fill))
                    .style(|_| container::Style::default()),
            )
            .on_press(Message::CloseMenu);

            let dropdown_position = container(row![
                Space::new().width(Length::Fixed(left_offset)),
                column![Space::new().height(Length::Fixed(56.0)), menu_card,],
            ])
            .width(Length::Fill)
            .height(Length::Fill);

            stack![backdrop, dropdown_position].into()
        });

        // 3. Fane Indhold
        let content: Element<Message> = match self.active_tab {
            Tab::ConceptList => {
                if let Some(editor) = &self.editor_state {
                    editor.view()
                } else {
                    let filtered = self.filtered_concepts();
                    concept_table::view(filtered, self.project.concepts().len(), &self.search_query)
                }
            }

            Tab::ConceptModel => concept_model_view::view(
                self.project.concepts(),
                self.project.concept_graph(),
                self.selected_graph_node_id,
                &self.selected_graph_node_ids,
                self.selected_edge,
                &self.concept_model_search,
                self.canvas_viewport,
                self.is_space_pressed,
                self.is_inline_graph_editing,
                self.editor_state.as_ref(),
                self.relation_dialog.as_ref(),
                self.show_left_sidebar,
                self.palette_drag.as_ref().is_some_and(|d| d.is_dragging),
            ),

            Tab::InformationModel => information_model_view::view(
                self.project.information_model(),
                self.project.information_graph(),
                self.project.concepts(),
                self.selected_info_class_id,
                self.selected_info_enum_id,
                self.selected_info_graph_node_id,
                &self.selected_info_graph_node_ids,
                self.selected_info_edge,
                &self.info_class_search,
                &self.new_enum_value_input,
                self.info_canvas_viewport,
                self.is_space_pressed,
                self.info_relation_dialog.as_ref(),
                self.show_left_sidebar,
                self.palette_drag.as_ref().is_some_and(|d| d.is_dragging),
            ),
        };

        // 4. Status Bar
        let save_status_text = self.footer_status_text();

        let rules_button = button(
            row![
                text("FDA Modelregler v2.1")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED),
                text("↗").size(10).color(ThemeColors::SLATE_400),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .on_press(Message::OpenModelRules)
        .style(|_theme, status| {
            let text_color = match status {
                button::Status::Hovered => ThemeColors::PRIMARY,
                _ => ThemeColors::SLATE_500,
            };
            button::Style {
                background: None,
                text_color,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .padding([0, 2]);

        let save_widget: Element<Message> = match &self.save_status {
            SaveStatus::Saved { path, .. } => {
                let full_path = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
                tooltip(
                    text(format!("• {}", save_status_text))
                        .size(12)
                        .color(ThemeColors::TEXT_MUTED),
                    container(
                        text(full_path.display().to_string())
                            .size(11)
                            .color(ThemeColors::TEXT_DARK),
                    )
                    .padding([4, 8])
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::SURFACE_CARD)),
                        border: iced::Border {
                            color: ThemeColors::SURFACE_BORDER,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }),
                    tooltip::Position::Top,
                )
                .into()
            }
            _ => text(format!("• {}", save_status_text))
                .size(12)
                .color(ThemeColors::TEXT_MUTED)
                .into(),
        };

        let collab_status_widget: Element<Message> = if self.collab_state.is_active() {
            let dot_color = match self.collab_connection_status {
                crate::features::collab::ConnectionStatus::Connected => {
                    iced::Color::from_rgb(0.12, 0.65, 0.35)
                }
                crate::features::collab::ConnectionStatus::Reconnecting => {
                    iced::Color::from_rgb(0.85, 0.55, 0.10)
                }
                _ => iced::Color::from_rgb(0.55, 0.60, 0.68),
            };
            row![
                text("•").size(12).color(dot_color),
                Space::new().width(4),
                text(self.collab_status_summary())
                    .size(12)
                    .color(ThemeColors::SLATE_700),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            text("• Offline")
                .size(12)
                .color(ThemeColors::SLATE_500)
                .into()
        };

        let status_bar = row![
            rules_button,
            text("• Klar").size(12).color(ThemeColors::SLATE_500),
            Space::new().width(12),
            text(format!("• {} begreber", self.project.concepts().len()))
                .size(12)
                .color(ThemeColors::SLATE_600),
            Space::new().width(12),
            save_widget,
            Space::new().width(12),
            collab_status_widget,
            Space::new().width(Length::Fill),
            text(format!("Aktiv fane: {:?}", self.active_tab))
                .size(12)
                .color(ThemeColors::SLATE_500),
        ]
        .padding([2, 4])
        .align_y(Alignment::Center);

        // 5. Samlet layout med Stack Overlay
        let main_col = column![
            header_bar,
            container(content)
                .style(card_container_style)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16),
            status_bar,
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        let base_layout: Element<Message> = container(main_col)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::SURFACE_BG)),
                ..Default::default()
            })
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let maybe_drag_ghost: Option<Element<Message>> =
            self.palette_drag.as_ref().and_then(|drag| {
                if !drag.is_dragging {
                    return None;
                }
                let (icon, label, tag_color) = match drag.item {
                    PaletteDragItem::Concept(cid) => {
                        let term = self
                            .project
                            .concepts()
                            .iter()
                            .find(|c| c.id() == cid)
                            .map(|c| c.preferred_term().to_string())
                            .unwrap_or_else(|| "Begreb".to_string());
                        ("💡", term, ThemeColors::PRIMARY)
                    }
                    PaletteDragItem::Class(cid) => {
                        let name = self
                            .project
                            .information_model()
                            .get_class(cid)
                            .map(|c| c.name().to_string())
                            .unwrap_or_else(|| "Klasse".to_string());
                        ("🏛️", name, ThemeColors::PRIMARY)
                    }
                    PaletteDragItem::Enumeration(eid) => {
                        let name = self
                            .project
                            .information_model()
                            .get_enumeration(eid)
                            .map(|e| e.name().to_string())
                            .unwrap_or_else(|| "Enumeration".to_string());
                        ("🔢", name, ThemeColors::ACCENT_GREEN)
                    }
                };

                let ghost_card = container(
                    row![
                        text(icon).size(13),
                        Space::new().width(4),
                        text(label).size(12).color(tag_color),
                        Space::new().width(6),
                        container(
                            text("Slip på lærred")
                                .size(9)
                                .color(ThemeColors::TEXT_MUTED)
                        )
                        .style(pill_container_style)
                        .padding([1, 4]),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        1.0, 1.0, 1.0, 0.94,
                    ))),
                    border: iced::Border {
                        color: tag_color,
                        width: 1.5,
                        radius: 6.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                        offset: iced::Vector::new(2.0, 4.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                })
                .padding([6, 10]);

                let overlay = container(column![
                    Space::new().height(Length::Fixed((drag.current_pos.y + 12.0).max(0.0))),
                    row![
                        Space::new().width(Length::Fixed((drag.current_pos.x + 12.0).max(0.0))),
                        ghost_card,
                    ],
                ])
                .width(Length::Fill)
                .height(Length::Fill);

                Some(overlay.into())
            });

        let layered_layout: Element<Message> = if let Some(ghost) = maybe_drag_ghost {
            stack![base_layout, ghost].into()
        } else {
            base_layout
        };

        if let Some(modal) = maybe_diagnostics_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_conflict_resolver_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_history_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_publish_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_git_connection_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_git_clone_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_excel_schema_guide_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_excel_import_preview_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_metadata_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_start_session_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_join_session_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_guest_ended_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_file_dialog_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_relation_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_quick_create_modal {
            stack![layered_layout, modal].into()
        } else if let Some(menu) = maybe_menu_overlay {
            stack![layered_layout, menu].into()
        } else {
            layered_layout
        }
    }

    fn view_git_status_badge(&self) -> Element<'_, Message> {
        let (dot_color, bg_color) = match &self.git_sync_status {
            RepoSyncStatus::Synced => (
                iced::Color::from_rgb(0.06, 0.72, 0.44), // emerald green
                iced::Color::from_rgb(0.92, 0.98, 0.94),
            ),
            RepoSyncStatus::PendingChanges => (
                iced::Color::from_rgb(0.92, 0.60, 0.05), // amber
                iced::Color::from_rgb(1.0, 0.98, 0.88),
            ),
            RepoSyncStatus::UnpublishedCommits(n) if *n > 0 => (
                iced::Color::from_rgb(0.20, 0.50, 0.95), // blue
                iced::Color::from_rgb(0.92, 0.95, 1.0),
            ),
            RepoSyncStatus::IncomingCommits(_) => (
                iced::Color::from_rgb(0.55, 0.35, 0.95), // purple
                iced::Color::from_rgb(0.95, 0.92, 1.0),
            ),
            RepoSyncStatus::Diverged { .. } => (
                iced::Color::from_rgb(0.92, 0.25, 0.25), // red
                iced::Color::from_rgb(1.0, 0.92, 0.92),
            ),
            _ => (
                iced::Color::from_rgb(0.55, 0.60, 0.68), // slate
                iced::Color::from_rgb(0.95, 0.95, 0.96),
            ),
        };

        let badge_text = self.git_status_badge_text();
        let is_uninit = matches!(self.git_sync_status, RepoSyncStatus::Uninitialized);

        let content = row![
            text("●").size(10).color(dot_color),
            Space::new().width(4),
            text(badge_text).size(11).color(ThemeColors::SLATE_800),
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

    fn view_publish_modal<'a>(&self, modal: &'a PublishModalState) -> Element<'a, Message> {
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

        let unpushed_count = match self.git_sync_status {
            RepoSyncStatus::UnpublishedCommits(n) => n,
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
                    ChangeAction::Added => ("🟢 Oprettet", iced::Color::from_rgb(0.06, 0.72, 0.44)),
                    ChangeAction::Modified => {
                        ("🟡 Opdateret", iced::Color::from_rgb(0.92, 0.60, 0.05))
                    }
                    ChangeAction::Deleted => {
                        ("🔴 Fjernet", iced::Color::from_rgb(0.92, 0.25, 0.25))
                    }
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
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.98, 0.98, 0.99,
                ))),
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
            row![
                Space::new().width(Length::Fill),
                button(text("Luk"))
                    .style(primary_button_style)
                    .on_press(Message::ClosePublishModal),
            ]
            .align_y(Alignment::Center)
        } else {
            row![
                Space::new().width(Length::Fill),
                button(text("Annuller"))
                    .style(secondary_button_style)
                    .on_press(Message::ClosePublishModal),
                Space::new().width(8),
                button(text(primary_button_label))
                    .style(primary_button_style)
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
                        .color(iced::Color::from_rgb(0.85, 0.2, 0.2)),
                    text(err)
                        .size(11)
                        .color(iced::Color::from_rgb(0.7, 0.1, 0.1)),
                ]
                .spacing(2),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.94, 0.94,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.9, 0.4, 0.4),
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

    fn view_history_modal<'a>(&self, modal: &'a ModelHistoryModalState) -> Element<'a, Message> {
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
                        .style(primary_button_style)
                        .on_press(Message::OpenPublishModal)
                        .padding([6, 14]),
                ]
                .spacing(4),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.96, 0.97, 0.99,
                ))),
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
            for commit in &modal.commits {
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
                        ChangeAction::Added => {
                            ("🟢 Oprettet", iced::Color::from_rgb(0.06, 0.72, 0.44))
                        }
                        ChangeAction::Modified => {
                            ("🟡 Opdateret", iced::Color::from_rgb(0.92, 0.60, 0.05))
                        }
                        ChangeAction::Deleted => {
                            ("🔴 Fjernet", iced::Color::from_rgb(0.92, 0.25, 0.25))
                        }
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

                let commit_card = container(
                    column![commit_header, Space::new().height(4), events_col].spacing(4),
                )
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.98, 0.98, 0.99,
                    ))),
                    border: iced::Border {
                        color: ThemeColors::SURFACE_BORDER,
                        width: 1.0,
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

        let footer = row![
            Space::new().width(Length::Fill),
            button(text("Luk"))
                .style(secondary_button_style)
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

    fn view_conflict_resolver_modal<'a>(
        &self,
        modal: &'a ConflictResolverModalState,
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
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.98, 0.98, 0.99,
                ))),
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

        let footer = row![
            Space::new().width(Length::Fill),
            button(text("Annuller"))
                .style(secondary_button_style)
                .on_press(Message::CloseConflictResolverModal),
            Space::new().width(8),
            button(text("Anvend valgte versioner"))
                .style(primary_button_style)
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

    fn view_git_connection_modal<'a>(
        &self,
        modal: &'a GitConnectionModalState,
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
            .style(modern_input_style)
            .on_input(Message::UpdateGitRemoteUrl)
            .padding([8, 12]),
        ]
        .spacing(4);

        let dir_field = column![
            text("Lokal Målmappe (Hvor modellen placeres på din computer)")
                .size(12)
                .color(ThemeColors::SLATE_700),
            row![
                text_input("/sti/til/lokal/modelmappe", &modal.target_dir)
                    .style(modern_input_style)
                    .on_input(Message::UpdateGitTargetDir)
                    .padding([8, 12])
                    .width(Length::Fill),
                button(text("📂 Gennemse...").size(12))
                    .style(secondary_button_style)
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
                    .color(iced::Color::from_rgb(0.85, 0.2, 0.2)),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.94, 0.94,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.9, 0.4, 0.4),
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
            actions = actions.push(
                button(text("Fjern fjernlager").size(12))
                    .style(|_theme, status| button::Style {
                        background: match status {
                            button::Status::Hovered => Some(iced::Background::Color(
                                iced::Color::from_rgb(1.0, 0.9, 0.9),
                            )),
                            _ => None,
                        },
                        text_color: iced::Color::from_rgb(0.85, 0.2, 0.2),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.85, 0.2, 0.2),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    })
                    .on_press(Message::RemoveGitRemote)
                    .padding([6, 12]),
            );
        }

        actions = actions.push(Space::new().width(Length::Fill));
        actions = actions.push(
            button(text("Annuller").size(12))
                .style(secondary_button_style)
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
                .style(primary_button_style)
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

    fn view_git_clone_modal<'a>(&self, modal: &'a GitCloneModalState) -> Element<'a, Message> {
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

        let subtitle = text("Hent en eksisterende fælles FDA-model ned til din computer fra et centralt repository.")
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
            .style(modern_input_style)
            .on_input(Message::UpdateGitCloneUrl)
            .padding([8, 12]),
        ]
        .spacing(4);

        let dir_field = column![
            text("Lokal Målmappe (Hvor modellen skal placeres)")
                .size(12)
                .color(ThemeColors::SLATE_700),
            row![
                text_input("/sti/til/lokal/modelmappe", &modal.target_dir)
                    .style(modern_input_style)
                    .on_input(Message::UpdateGitCloneTargetDir)
                    .padding([8, 12])
                    .width(Length::Fill),
                button(text("📂 Gennemse...").size(12))
                    .style(secondary_button_style)
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
                    .color(iced::Color::from_rgb(0.85, 0.2, 0.2)),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.94, 0.94,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.9, 0.4, 0.4),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding(8)
            .width(Length::Fill);
            body = body.push(err_box);
        }

        let actions = row![
            Space::new().width(Length::Fill),
            button(text("Annuller").size(12))
                .style(secondary_button_style)
                .on_press(Message::CloseGitCloneModal)
                .padding([6, 14]),
            button(text("Klon og åbn model").size(12))
                .style(primary_button_style)
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

    fn view_diagnostics_modal<'a>(&self, modal: &'a DiagnosticsModalState) -> Element<'a, Message> {
        let title_row = row![
            text("🔍 System- og Grafikdiagnostik")
                .size(17)
                .color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(13))
                .style(secondary_button_style)
                .on_press(Message::CloseDiagnosticsModal)
                .padding([3, 7]),
        ]
        .align_y(Alignment::Center);

        let subtitle = text(
            "Platforminformation, aktiv rendering-motor og lokale loghændelser til fejlfinding.",
        )
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

        let info_item = |label: &'static str, val: &str| {
            row![
                text(label)
                    .size(12)
                    .color(ThemeColors::SLATE_600)
                    .width(Length::Fixed(140.0)),
                text(val.to_string()).size(12).color(ThemeColors::SLATE_900),
            ]
            .align_y(Alignment::Center)
        };

        let log_path_str = modal
            .diagnostics
            .log_path
            .as_deref()
            .unwrap_or("(Ingen skriveadgang / in-memory)");

        let startup_time_str = modal
            .diagnostics
            .startup_milestones
            .last()
            .map(|m| format!("{}ms ({})", m.elapsed_ms, m.name))
            .unwrap_or_else(|| "N/A".to_string());

        let system_info_card = container(
            column![
                info_item("Operativsystem:", &modal.diagnostics.os),
                info_item("Arkitektur:", &modal.diagnostics.arch),
                info_item("Kant Version:", &modal.diagnostics.app_version),
                info_item("Iced Backend:", &modal.diagnostics.iced_backend),
                info_item("WGPU Backend:", &modal.diagnostics.wgpu_backend),
                info_item("Opstartstid:", &startup_time_str),
                info_item("Logfil Sti:", log_path_str),
            ]
            .spacing(6),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
            border: iced::Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .padding(12)
        .width(Length::Fill);

        let logs_header = text("Seneste loghændelser (In-memory buffer):")
            .size(12)
            .color(ThemeColors::SLATE_700);

        let mut log_column = column![].spacing(3);
        if modal.logs.is_empty() {
            log_column = log_column.push(
                text("Ingen logbeskeder registreret endnu.")
                    .size(11)
                    .color(ThemeColors::SLATE_500),
            );
        } else {
            for entry in modal.logs.iter().rev().take(30) {
                log_column =
                    log_column.push(text(entry.clone()).size(11).color(ThemeColors::SLATE_800));
            }
        }

        let logs_container = container(scrollable(log_column).height(Length::Fixed(180.0)))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                border: iced::Border {
                    color: ThemeColors::SLATE_200,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .padding(10)
            .width(Length::Fill);

        let actions = row![
            button(text("📋 Kopier rapport til udklipsholder").size(12))
                .style(primary_button_style)
                .on_press(Message::CopyDiagnosticsToClipboard)
                .padding([6, 14]),
            Space::new().width(Length::Fill),
            button(text("Luk").size(12))
                .style(secondary_button_style)
                .on_press(Message::CloseDiagnosticsModal)
                .padding([6, 16]),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let body = column![
            title_row,
            subtitle,
            Space::new().height(4),
            system_info_card,
            Space::new().height(4),
            logs_header,
            logs_container,
            Space::new().height(8),
            actions,
        ]
        .spacing(8);

        let modal_card = container(body)
            .style(modal_card_style)
            .padding(24)
            .width(Length::Fixed(620.0));

        container(modal_card)
            .style(modal_backdrop_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_excel_schema_guide_modal<'a>(&self) -> Element<'a, Message> {
        let title_row = row![
            text("ℹ️ FDA Begrebsliste - Excel Skemaguide")
                .size(17)
                .color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(13))
                .style(secondary_button_style)
                .on_press(Message::CloseExcelSchemaGuideModal)
                .padding([3, 7]),
        ]
        .align_y(Alignment::Center);

        let subtitle = text(
            "Oversigt over felter og kolonner i den officielle FDA Excel-skabelon (SKOS-kompatibel med tosprogethed):",
        )
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

        let entries = crate::features::concepts::excel::get_fda_schema_guide_entries();
        let mut entry_cards = column![].spacing(8);

        for entry in entries {
            let badge = if entry.required {
                container(text("Obligatorisk").size(10).color(ThemeColors::ACCENT_RED))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::ACCENT_RED,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([2, 6])
            } else {
                container(text("Valgfri").size(10).color(ThemeColors::SLATE_600))
                    .style(pill_container_style)
                    .padding([2, 6])
            };

            let header = row![
                text(entry.name).size(13).color(ThemeColors::SLATE_900),
                Space::new().width(Length::Fill),
                text(entry.tag).size(11).color(ThemeColors::PRIMARY),
                Space::new().width(6),
                badge,
            ]
            .align_y(Alignment::Center);

            let desc = text(entry.description)
                .size(12)
                .color(ThemeColors::SLATE_700);
            let example = text(format!("Eksempel / format: {}", entry.example))
                .size(11)
                .color(ThemeColors::TEXT_MUTED);

            let card = container(column![header, desc, example].spacing(4))
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
                    border: iced::Border {
                        color: ThemeColors::SLATE_200,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .padding([8, 12])
                .width(Length::Fill);

            entry_cards = entry_cards.push(card);
        }

        let scrollable_entries = scrollable(entry_cards).height(Length::Fixed(400.0));

        let actions_row = row![
            button(
                row![
                    text("📄").size(13),
                    Space::new().width(6),
                    text("Hent officiel FDA-skabelon (.xlsx)").size(12),
                ]
                .align_y(Alignment::Center),
            )
            .style(secondary_button_style)
            .on_press(Message::ExportFdaTemplateXlsxDialog)
            .padding([6, 14]),
            Space::new().width(Length::Fill),
            button(text("Luk").size(12))
                .style(primary_button_style)
                .on_press(Message::CloseExcelSchemaGuideModal)
                .padding([6, 18]),
        ]
        .align_y(Alignment::Center);

        let modal_card = container(
            column![
                title_row,
                subtitle,
                scrollable_entries,
                Space::new().height(4),
                actions_row,
            ]
            .spacing(12),
        )
        .style(modal_card_style)
        .padding(20)
        .width(Length::Fixed(680.0));

        container(modal_card)
            .style(modal_backdrop_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_excel_import_preview_modal<'a>(
        &self,
        preview: &'a ExcelImportPreviewState,
    ) -> Element<'a, Message> {
        let file_name = preview
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Excel-fil");

        let title_row = row![
            text("📥 Forhåndsvisning af Excel Import")
                .size(17)
                .color(ThemeColors::SLATE_900),
            Space::new().width(Length::Fill),
            button(text("✕").size(13))
                .style(secondary_button_style)
                .on_press(Message::DismissExcelImportModal)
                .padding([3, 7]),
        ]
        .align_y(Alignment::Center);

        let file_info = text(format!(
            "Valgt regneark: {} (Ark: {})",
            file_name, preview.import_summary.sheet_name
        ))
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

        let stats_row = row![
            container(
                row![
                    text("Gyldige i Excel:")
                        .size(12)
                        .color(ThemeColors::SLATE_600),
                    Space::new().width(6),
                    text(format!("{}", preview.import_summary.valid_concepts.len()))
                        .size(13)
                        .color(ThemeColors::SLATE_900),
                ]
                .align_y(Alignment::Center),
            )
            .style(pill_container_style)
            .padding([4, 10]),
            container(
                row![
                    text("+ Nye begreber:")
                        .size(12)
                        .color(ThemeColors::ACCENT_GREEN),
                    Space::new().width(6),
                    text(format!("{}", preview.new_count))
                        .size(13)
                        .color(ThemeColors::ACCENT_GREEN),
                ]
                .align_y(Alignment::Center),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::ACCENT_GREEN_LIGHT)),
                border: iced::Border {
                    color: ThemeColors::ACCENT_GREEN,
                    width: 1.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            })
            .padding([4, 10]),
            container(
                row![
                    text("🔄 Opdateres (upsert):")
                        .size(12)
                        .color(ThemeColors::PRIMARY),
                    Space::new().width(6),
                    text(format!("{}", preview.update_count))
                        .size(13)
                        .color(ThemeColors::PRIMARY),
                ]
                .align_y(Alignment::Center),
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                border: iced::Border {
                    color: ThemeColors::PRIMARY,
                    width: 1.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            })
            .padding([4, 10]),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let notice_card = container(
            text("💡 Intelligent upsert matcher på begrebets Identifikator (URI) eller Foretrukken term (case-insensitiv). Eksisterende begrebers interne Uuid bevares uændret, så eksisterende diagram-placeringer og relationer aldrig brydes.")
                .size(12)
                .color(ThemeColors::SLATE_700),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
            border: iced::Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding([8, 12])
        .width(Length::Fill);

        let mut preview_list = column![].spacing(6);
        for concept in &preview.import_summary.valid_concepts {
            let is_match = self.project.concepts().iter().any(|existing| {
                (concept.identifier().is_some() && concept.identifier() == existing.identifier())
                    || concept
                        .preferred_term()
                        .trim()
                        .eq_ignore_ascii_case(existing.preferred_term().trim())
            });

            let status_badge = if is_match {
                container(text("Opdateres").size(10).color(ThemeColors::PRIMARY))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::PRIMARY_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::PRIMARY,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([1, 6])
            } else {
                container(text("Ny").size(10).color(ThemeColors::ACCENT_GREEN))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::ACCENT_GREEN_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::ACCENT_GREEN,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([1, 6])
            };

            let row_item = row![
                status_badge,
                Space::new().width(6),
                text(concept.preferred_term())
                    .size(12)
                    .color(ThemeColors::SLATE_900)
                    .width(Length::FillPortion(2)),
                text(concept.definition())
                    .size(11)
                    .color(ThemeColors::TEXT_MUTED)
                    .width(Length::FillPortion(3)),
            ]
            .align_y(Alignment::Center);

            preview_list = preview_list.push(row_item);
        }

        let scrollable_list = scrollable(preview_list).height(Length::Fixed(240.0));

        let actions_row = row![
            Space::new().width(Length::Fill),
            button(text("Annuller").size(12))
                .style(secondary_button_style)
                .on_press(Message::DismissExcelImportModal)
                .padding([6, 14]),
            button(row![text("Bekræft og Gennemfør Import").size(12),].align_y(Alignment::Center),)
                .style(primary_button_style)
                .on_press(Message::ConfirmExcelImport)
                .padding([6, 16]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut modal_content = column![title_row, file_info, stats_row, notice_card,].spacing(12);

        if !preview.import_summary.rejected_rows.is_empty() {
            let mut rej_col = column![
                text(format!(
                    "⚠️ {} række(r) blev sprunget over pga. manglende påkrævet Foretrukken term eller Definition:",
                    preview.import_summary.rejected_rows.len()
                ))
                .size(12)
                .color(ThemeColors::ACCENT_RED),
            ]
            .spacing(2);

            for rej in preview.import_summary.rejected_rows.iter().take(5) {
                rej_col = rej_col.push(
                    text(format!(
                        "• Række {}: \"{}\" ({})",
                        rej.row_index, rej.preferred_term, rej.reason
                    ))
                    .size(11)
                    .color(ThemeColors::ACCENT_RED),
                );
            }
            if preview.import_summary.rejected_rows.len() > 5 {
                rej_col = rej_col.push(
                    text(format!(
                        "... og {} mere",
                        preview.import_summary.rejected_rows.len() - 5
                    ))
                    .size(10)
                    .color(ThemeColors::ACCENT_RED),
                );
            }

            modal_content = modal_content.push(
                container(rej_col)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::ACCENT_RED_LIGHT)),
                        border: iced::Border {
                            color: ThemeColors::ACCENT_RED,
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding([8, 12])
                    .width(Length::Fill),
            );
        }

        modal_content = modal_content.push(scrollable_list);
        modal_content = modal_content.push(Space::new().height(4));
        modal_content = modal_content.push(actions_row);

        let modal_card = container(modal_content)
            .style(modal_card_style)
            .padding(20)
            .width(Length::Fixed(660.0));

        container(modal_card)
            .style(modal_backdrop_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

/// Indlæser applikationens 32x32 RGBA vinduesikon til Iced Settings
pub fn load_window_icon() -> Option<iced::window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/kant-32.rgba");
    iced::window::icon::from_rgba(ICON_BYTES.to_vec(), 32, 32).ok()
}
