use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{NodeId, RelationKind};
use crate::features::concepts::{BelongsToDomain, Concept};
use crate::features::export::{
    export_concept_model_svg, export_concepts_to_csv, export_information_model_svg,
    export_model_report_html, export_model_report_markdown, export_to_shacl_turtle,
    export_to_skos_turtle, export_to_xmi_2_1,
};
use crate::features::information_model::{
    Attribute, ClassRelation, InformationClass, InformationDataType, InformationEnumeration,
    Multiplicity, PrimitiveType,
};
use crate::features::model::storage::ProjectStorage;
use crate::features::model::{ApprovalStatus, ModelProject, ModelScope, ModelStatus};
use crate::ui::concept_editor::ConceptEditorState;
use crate::ui::concept_model_view;
use crate::ui::concept_table;
use crate::ui::diagram_canvas::{CanvasEdge, CanvasNode};
use crate::ui::focus::{FocusCycle, FocusTarget};
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
    button, column, container, mouse_area, operation, pick_list, row, stack, svg, text, text_input,
    tooltip, Space,
};
use iced::{Alignment, Element, Length, Point, Subscription, Task};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub use crate::features::diagnostics::ui::{view_diagnostics_modal, DiagnosticsModalState};
pub use crate::features::export::ui::{
    view_excel_import_preview_modal, view_excel_schema_guide_modal, ExcelImportPreviewState,
};
pub use crate::features::model::ui::{
    view_metadata_modal, MetadataField, ModelMetadataModalState, META_BTN_CANCEL, META_BTN_SAVE,
};

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

pub use crate::features::concept_model::ui::QuickCreateState;

pub use crate::ui::concept_editor::ConceptFormField;

// (MetadataField & ModelMetadataModalState moved to crate::features::model::ui)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    ConceptList,
    ConceptModel,
    InformationModel,
}

pub const TOOLBAR_BTN_NEW_CONCEPT: usize = 0;
pub const TOOLBAR_BTN_IMPORT_EXCEL: usize = 1;
pub const TOOLBAR_BTN_EXPORT_EXCEL: usize = 2;
pub const TOOLBAR_BTN_TEMPLATE: usize = 3;
pub const TOOLBAR_BTN_GUIDE: usize = 4;
pub const TOOLBAR_ROW_BTN_START: usize = 100;

pub const INFO_INSPECTOR_BTN_CANVAS_ACTION: usize = 0;
pub const INFO_INSPECTOR_BTN_DELETE_CLASS: usize = 1;
pub const INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX: usize = 2;
pub const INFO_INSPECTOR_BTN_TYPE_LOCAL: usize = 3;
pub const INFO_INSPECTOR_BTN_TYPE_BORROWED: usize = 4;
pub const INFO_INSPECTOR_BTN_ADD_ATTRIBUTE: usize = 5;
pub const INFO_INSPECTOR_BTN_DELETE_ENUM: usize = 6;
pub const INFO_INSPECTOR_BTN_ADD_ENUM_VAL: usize = 7;
pub const INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER: usize = 8;
pub const INFO_INSPECTOR_CONCEPT_REMOVE_START: usize = 100;
pub const INFO_INSPECTOR_ATTR_DELETE_START: usize = 200;
pub const INFO_INSPECTOR_ENUM_VAL_REMOVE_START: usize = 300;
pub const INFO_INSPECTOR_REL_DELETE_START: usize = 400;

pub const CONCEPT_EDGE_BTN_REVERSE: usize = 0;
pub const CONCEPT_EDGE_BTN_KIND_ASSOC: usize = 1;
pub const CONCEPT_EDGE_BTN_KIND_GEN: usize = 2;
pub const CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX: usize = 3;
pub const CONCEPT_EDGE_BTN_DELETE: usize = 4;

pub const INFO_EDGE_BTN_REVERSE: usize = 0;
pub const INFO_EDGE_BTN_KIND_ASSOC: usize = 1;
pub const INFO_EDGE_BTN_KIND_GEN: usize = 2;
pub const INFO_EDGE_BTN_KIND_COMP: usize = 3;
pub const INFO_EDGE_BTN_KIND_DEP: usize = 4;
pub const INFO_EDGE_BTN_DIRECTED_CHECKBOX: usize = 5;
pub const INFO_EDGE_BTN_SRC_MULT: usize = 6;
pub const INFO_EDGE_BTN_TGT_MULT: usize = 7;
pub const INFO_EDGE_BTN_DELETE: usize = 8;

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
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    NewProject,

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
    ConfirmExcelImportAsNewModel,
    DismissExcelImportModal,
    SelectExcelImportSheet(String),
    ToggleExcelImportMetadata(bool),

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
    FocusEditorButton(Option<usize>),
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
    ConceptPaletteNext,
    ConceptPalettePrevious,
    ConceptPaletteSubmit,

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
    InfoPaletteNext,
    InfoPalettePrevious,
    InfoPaletteSubmit,

    // Concept Picker Combobox (Task 072)
    OpenConceptPicker,
    CloseConceptPicker,
    ConceptPickerSearchChanged(String),
    ConceptPickerNext,
    ConceptPickerPrevious,
    ConceptPickerSubmit,

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
    InfoNodeDoubleClicked(NodeId),
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
    CanvasDropAt { x: f32, y: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickCreateSource {
    DoubleClick,
    Toolbar,
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
    pending_quick_create: Option<Uuid>,
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
    diagnostics_modal: Option<DiagnosticsModalState>,
    excel_schema_guide_open: bool,
    excel_import_preview: Option<ExcelImportPreviewState>,
    recent_store: crate::features::model::recent::RecentStore,
    pub modal_focus: FocusCycle,
    pub toolbar_focus: FocusCycle,
    pub info_inspector_focus: FocusCycle,
    pub concept_inspector_focus: FocusCycle,
    pub is_concept_picker_open: bool,
    pub concept_picker_search: String,
    pub concept_picker_index: usize,
    pub concept_palette_index: usize,
    pub info_palette_index: usize,
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
                        diagnostics_modal: None,
                        excel_schema_guide_open: false,
                        excel_import_preview: None,
                        modal_focus: FocusCycle::default(),
                        toolbar_focus: FocusCycle::with_targets(
                            vec![
                                FocusTarget::Input("concept_search_input"),
                                FocusTarget::Button(TOOLBAR_BTN_NEW_CONCEPT),
                                FocusTarget::Button(TOOLBAR_BTN_IMPORT_EXCEL),
                                FocusTarget::Button(TOOLBAR_BTN_EXPORT_EXCEL),
                                FocusTarget::Button(TOOLBAR_BTN_TEMPLATE),
                                FocusTarget::Button(TOOLBAR_BTN_GUIDE),
                            ],
                            Some(0),
                        ),
                        info_inspector_focus: FocusCycle::default(),
                        concept_inspector_focus: FocusCycle::default(),
                        is_concept_picker_open: false,
                        concept_picker_search: String::new(),
                        concept_picker_index: 0,
                        concept_palette_index: 0,
                        info_palette_index: 0,
                        pending_quick_create: None,
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
            diagnostics_modal: None,
            excel_schema_guide_open: false,
            excel_import_preview: None,
            recent_store,
            modal_focus: FocusCycle::default(),
            toolbar_focus: FocusCycle::with_targets(
                vec![
                    FocusTarget::Input("concept_search_input"),
                    FocusTarget::Button(TOOLBAR_BTN_NEW_CONCEPT),
                    FocusTarget::Button(TOOLBAR_BTN_IMPORT_EXCEL),
                    FocusTarget::Button(TOOLBAR_BTN_EXPORT_EXCEL),
                    FocusTarget::Button(TOOLBAR_BTN_TEMPLATE),
                    FocusTarget::Button(TOOLBAR_BTN_GUIDE),
                ],
                Some(0),
            ),
            info_inspector_focus: FocusCycle::default(),
            concept_inspector_focus: FocusCycle::default(),
            is_concept_picker_open: false,
            concept_picker_search: String::new(),
            concept_picker_index: 0,
            concept_palette_index: 0,
            info_palette_index: 0,
            pending_quick_create: None,
        }
    }

    pub fn recent_store(&self) -> &crate::features::model::recent::RecentStore {
        &self.recent_store
    }

    pub fn diagnostics_modal(&self) -> Option<&DiagnosticsModalState> {
        self.diagnostics_modal.as_ref()
    }

    pub fn is_excel_schema_guide_open(&self) -> bool {
        self.excel_schema_guide_open
    }

    pub fn is_excel_import_preview_open(&self) -> bool {
        self.excel_import_preview.is_some()
    }

    pub fn set_excel_import_preview_for_test(&mut self, preview: ExcelImportPreviewState) {
        self.modal_focus = FocusCycle::with_initial(3, Some(2));
        self.excel_import_preview = Some(preview);
    }

    pub fn is_concept_picker_open(&self) -> bool {
        self.is_concept_picker_open
    }

    pub fn concept_picker_search(&self) -> &str {
        &self.concept_picker_search
    }

    pub fn concept_picker_index(&self) -> usize {
        self.concept_picker_index
    }

    pub fn concept_palette_index(&self) -> usize {
        self.concept_palette_index
    }

    pub fn info_palette_index(&self) -> usize {
        self.info_palette_index
    }

    pub fn show_left_sidebar(&self) -> bool {
        self.show_left_sidebar
    }

    pub fn info_class_search(&self) -> &str {
        &self.info_class_search
    }

    pub fn canvas_visible_center(&self) -> Point {
        Point::new(500.0, 350.0)
    }

    pub fn filtered_palette_concepts(&self) -> Vec<&crate::features::concepts::Concept> {
        let search_filter = self.concept_model_search.trim().to_lowercase();
        let mut list: Vec<&crate::features::concepts::Concept> = self
            .project
            .concepts()
            .iter()
            .filter(|c| {
                if search_filter.is_empty() {
                    true
                } else {
                    c.preferred_term().to_lowercase().contains(&search_filter)
                        || c.definition().to_lowercase().contains(&search_filter)
                }
            })
            .collect();
        list.sort_by(|a, b| {
            a.preferred_term()
                .to_lowercase()
                .cmp(&b.preferred_term().to_lowercase())
        });
        list
    }

    pub fn filtered_palette_classes(
        &self,
    ) -> Vec<&crate::features::information_model::models::InformationClass> {
        let search_filter = self.info_class_search.trim().to_lowercase();
        let mut list: Vec<&crate::features::information_model::models::InformationClass> = self
            .project
            .information_model()
            .classes()
            .iter()
            .filter(|c| {
                if search_filter.is_empty() {
                    true
                } else {
                    c.name().to_lowercase().contains(&search_filter)
                        || c.description()
                            .map(|d| d.to_lowercase().contains(&search_filter))
                            .unwrap_or(false)
                }
            })
            .collect();
        list.sort_by_key(|a| a.name().to_lowercase());
        list
    }

    pub fn available_concepts_for_class(&self, class_id: Uuid) -> Vec<ConceptOption> {
        if let Some(cls) = self.project.information_model().get_class(class_id) {
            self.project
                .concepts()
                .iter()
                .filter(|c| !cls.concept_ids().contains(&c.id()))
                .map(|c| ConceptOption {
                    id: c.id(),
                    term: c.preferred_term().to_string(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn filtered_available_concepts_for_class(&self, class_id: Uuid) -> Vec<ConceptOption> {
        let available = self.available_concepts_for_class(class_id);
        if self.concept_picker_search.trim().is_empty() {
            available
        } else {
            let q = self.concept_picker_search.to_lowercase();
            available
                .into_iter()
                .filter(|opt| opt.term.to_lowercase().contains(&q))
                .collect()
        }
    }

    pub fn boot(path: Option<PathBuf>) -> (Self, Task<Message>) {
        let app = match path {
            Some(p) => Self::new_with_path(Some(p)),
            None => Self::new_auto_open(),
        };
        (app, Task::none())
    }

    pub fn set_active_file_path(&mut self, path: Option<PathBuf>) {
        self.current_file_path = path;
    }

    pub fn save_status(&self) -> &SaveStatus {
        &self.save_status
    }

    pub fn footer_status_text(&self) -> String {
        self.save_status
            .display_text(self.current_file_path.as_ref())
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

    pub fn set_selected_info_graph_node(&mut self, node_id: Option<NodeId>) {
        self.selected_info_graph_node_id = node_id;
        self.selected_info_graph_node_ids = node_id.into_iter().collect();
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

    pub fn map_key_press(key: &Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        match key.as_ref() {
            Key::Named(Named::Tab) => {
                if modifiers.shift() {
                    Some(Message::FocusPrevious)
                } else {
                    Some(Message::FocusNext)
                }
            }
            Key::Named(Named::Escape) => Some(Message::EscapePressed),
            Key::Named(Named::Enter) => Some(Message::EnterPressed),
            Key::Named(Named::Delete) | Key::Named(Named::Backspace) => {
                Some(Message::GraphDeleteSelected)
            }
            Key::Named(Named::Space) => Some(Message::CanvasSpacePressed(true)),
            Key::Named(Named::ArrowDown) => Some(Message::ConceptPickerNext),
            Key::Named(Named::ArrowUp) => Some(Message::ConceptPickerPrevious),
            Key::Character(c)
                if (c == "s" || c == "S") && (modifiers.control() || modifiers.command()) =>
            {
                Some(Message::SaveProject)
            }
            Key::Character(c)
                if (c == "b" || c == "B") && (modifiers.control() || modifiers.command()) =>
            {
                Some(Message::ToggleLeftSidebar)
            }
            Key::Character("1") if modifiers.control() || modifiers.command() => {
                Some(Message::SelectTab(Tab::ConceptList))
            }
            Key::Character("2") if modifiers.control() || modifiers.command() => {
                Some(Message::SelectTab(Tab::ConceptModel))
            }
            Key::Character("3") if modifiers.control() || modifiers.command() => {
                Some(Message::SelectTab(Tab::InformationModel))
            }
            Key::Character(c)
                if (c == "+" || c == "=") && (modifiers.control() || modifiers.command()) =>
            {
                Some(Message::CanvasZoomIn)
            }
            Key::Character(c) if c == "-" && (modifiers.control() || modifiers.command()) => {
                Some(Message::CanvasZoomOut)
            }
            Key::Character(c) if c == "0" && (modifiers.control() || modifiers.command()) => {
                Some(Message::CanvasResetView)
            }
            _ => None,
        }
    }

    /// Returnerer true hvis der er en aktiv modal eller overlay-dialog åben.
    pub fn has_active_modal(&self) -> bool {
        self.diagnostics_modal.is_some()
            || self.metadata_modal.is_some()
            || self.excel_schema_guide_open
            || self.excel_import_preview.is_some()
            || self.file_dialog_mode.is_some()
            || self.relation_dialog.is_some()
            || self.info_relation_dialog.is_some()
            || self.quick_create.is_some()
    }

    /// Lukker den øverste åbne modal eller dialog. Returnerer true hvis en modal blev lukket.
    pub fn close_active_modal(&mut self) -> bool {
        self.modal_focus.reset();
        if self.diagnostics_modal.is_some() {
            self.diagnostics_modal = None;
            true
        } else if self.metadata_modal.is_some() {
            self.metadata_modal = None;
            true
        } else if self.excel_schema_guide_open {
            self.excel_schema_guide_open = false;
            true
        } else if self.excel_import_preview.is_some() {
            self.excel_import_preview = None;
            true
        } else if self.file_dialog_mode.is_some() {
            self.file_dialog_mode = None;
            true
        } else if self.relation_dialog.is_some() {
            self.relation_dialog = None;
            true
        } else if self.info_relation_dialog.is_some() {
            self.info_relation_dialog = None;
            true
        } else if self.quick_create.is_some() {
            self.quick_create = None;
            true
        } else {
            false
        }
    }

    pub fn modal_focused_button_index(&self) -> Option<usize> {
        if self.has_active_modal() {
            self.modal_focus.focused_button()
        } else {
            None
        }
    }

    pub fn editor_focused_button(&self) -> Option<usize> {
        self.editor_state.as_ref().and_then(|e| e.focused_button())
    }

    pub fn toolbar_focused_button(&self) -> Option<usize> {
        if self.active_tab == Tab::ConceptList
            && !self.has_active_modal()
            && self.editor_state.is_none()
        {
            self.toolbar_focus.focused_button()
        } else {
            None
        }
    }

    pub fn info_inspector_focused_button(&self) -> Option<usize> {
        if self.active_tab == Tab::InformationModel
            && (self.selected_info_class_id.is_some()
                || self.selected_info_enum_id.is_some()
                || self.selected_info_edge.is_some())
            && !self.has_active_modal()
            && self.editor_state.is_none()
        {
            self.info_inspector_focus.focused_button()
        } else {
            None
        }
    }

    pub fn concept_inspector_focused_button(&self) -> Option<usize> {
        if self.active_tab == Tab::ConceptModel
            && self.selected_edge.is_some()
            && !self.has_active_modal()
            && self.editor_state.is_none()
        {
            self.concept_inspector_focus.focused_button()
        } else {
            None
        }
    }

    fn build_concept_inspector_targets(&self) -> Vec<FocusTarget> {
        if let Some((from_id, to_id)) = self.selected_edge {
            if let Some(edge) = self.project.concept_graph().find_edge(from_id, to_id) {
                let mut targets = vec![
                    FocusTarget::Input("edge_label_input"),
                    FocusTarget::Button(CONCEPT_EDGE_BTN_REVERSE),
                    FocusTarget::Button(CONCEPT_EDGE_BTN_KIND_ASSOC),
                    FocusTarget::Button(CONCEPT_EDGE_BTN_KIND_GEN),
                ];
                if edge.kind() == RelationKind::Association {
                    targets.push(FocusTarget::Button(CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX));
                }
                targets.push(FocusTarget::Button(CONCEPT_EDGE_BTN_DELETE));
                targets
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    fn build_concept_toolbar_targets(&self) -> Vec<FocusTarget> {
        let mut targets = vec![
            FocusTarget::Input("concept_search_input"),
            FocusTarget::Button(TOOLBAR_BTN_NEW_CONCEPT),
            FocusTarget::Button(TOOLBAR_BTN_IMPORT_EXCEL),
            FocusTarget::Button(TOOLBAR_BTN_EXPORT_EXCEL),
            FocusTarget::Button(TOOLBAR_BTN_TEMPLATE),
            FocusTarget::Button(TOOLBAR_BTN_GUIDE),
        ];
        for (i, _) in self.filtered_concepts().iter().enumerate() {
            targets.push(FocusTarget::Button(TOOLBAR_ROW_BTN_START + i * 2));
            targets.push(FocusTarget::Button(TOOLBAR_ROW_BTN_START + i * 2 + 1));
        }
        targets
    }

    fn build_info_inspector_targets(&self) -> Vec<FocusTarget> {
        if let Some(class_id) = self.selected_info_class_id {
            let mut targets = vec![
                FocusTarget::Input("info_class_name_input"),
                FocusTarget::Input("info_class_desc_input"),
                FocusTarget::Button(INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX),
                FocusTarget::Button(INFO_INSPECTOR_BTN_TYPE_LOCAL),
                FocusTarget::Button(INFO_INSPECTOR_BTN_TYPE_BORROWED),
            ];
            if let Some(class) = self.project.information_model().get_class(class_id) {
                if !class.is_local() {
                    targets.push(FocusTarget::Input("info_class_origin_model_input"));
                }
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_CANVAS_ACTION));
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_DELETE_CLASS));
                for (k, _) in class.concept_ids().iter().enumerate() {
                    targets.push(FocusTarget::Button(INFO_INSPECTOR_CONCEPT_REMOVE_START + k));
                }
                let has_available = self
                    .project
                    .concepts()
                    .iter()
                    .any(|c| !class.concept_ids().contains(&c.id()));
                if has_available {
                    targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER));
                }
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_ADD_ATTRIBUTE));
                for (j, _) in class.attributes().iter().enumerate() {
                    targets.push(FocusTarget::Button(INFO_INSPECTOR_ATTR_DELETE_START + j));
                }
                let class_relations = self
                    .project
                    .information_model()
                    .relations_for_class(class_id);
                for (r, _) in class_relations.iter().enumerate() {
                    targets.push(FocusTarget::Button(INFO_INSPECTOR_REL_DELETE_START + r));
                }
            } else {
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_CANVAS_ACTION));
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_DELETE_CLASS));
                targets.push(FocusTarget::Button(INFO_INSPECTOR_BTN_ADD_ATTRIBUTE));
            }
            targets
        } else if let Some(enum_id) = self.selected_info_enum_id {
            let mut targets = vec![
                FocusTarget::Input("info_enum_name_input"),
                FocusTarget::Input("info_enum_def_input"),
                FocusTarget::Button(INFO_INSPECTOR_BTN_CANVAS_ACTION),
                FocusTarget::Button(INFO_INSPECTOR_BTN_DELETE_ENUM),
                FocusTarget::Input("new_enum_val_input"),
                FocusTarget::Button(INFO_INSPECTOR_BTN_ADD_ENUM_VAL),
            ];
            if let Some(e) = self.project.information_model().get_enumeration(enum_id) {
                for (v, _) in e.values().iter().enumerate() {
                    targets.push(FocusTarget::Button(
                        INFO_INSPECTOR_ENUM_VAL_REMOVE_START + v,
                    ));
                }
            }
            targets
        } else if let Some((from_id, to_id)) = self.selected_info_edge {
            if let Some(edge) = self.project.information_graph().find_edge(from_id, to_id) {
                let mut targets = vec![
                    FocusTarget::Input("info_edge_label_input"),
                    FocusTarget::Button(INFO_EDGE_BTN_REVERSE),
                    FocusTarget::Button(INFO_EDGE_BTN_KIND_ASSOC),
                    FocusTarget::Button(INFO_EDGE_BTN_KIND_GEN),
                    FocusTarget::Button(INFO_EDGE_BTN_KIND_COMP),
                    FocusTarget::Button(INFO_EDGE_BTN_KIND_DEP),
                ];
                if edge.kind() == RelationKind::Association {
                    targets.push(FocusTarget::Button(INFO_EDGE_BTN_DIRECTED_CHECKBOX));
                }
                if edge.kind() != RelationKind::Generalization {
                    targets.push(FocusTarget::Button(INFO_EDGE_BTN_SRC_MULT));
                    targets.push(FocusTarget::Button(INFO_EDGE_BTN_TGT_MULT));
                }
                targets.push(FocusTarget::Button(INFO_EDGE_BTN_DELETE));
                targets
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn advance_tab_focus(&mut self, reverse: bool) -> Option<Task<Message>> {
        if self.has_active_modal() || self.editor_state.is_some() {
            return None;
        }

        if self.active_tab == Tab::ConceptList {
            let targets = self.build_concept_toolbar_targets();
            if self.toolbar_focus.count() == 0 {
                self.toolbar_focus = FocusCycle::with_targets(targets, Some(0));
            } else {
                self.toolbar_focus.update_targets(targets);
            }

            if reverse {
                self.toolbar_focus.previous();
            } else {
                self.toolbar_focus.next();
            }

            match self.toolbar_focus.current_target() {
                Some(FocusTarget::Input(id)) => {
                    return Some(operation::focus(*id));
                }
                Some(FocusTarget::Button(_)) => {
                    return Some(operation::focus(iced::widget::Id::unique()));
                }
                None => return Some(Task::none()),
            }
        }

        None
    }

    pub fn advance_info_inspector_focus(&mut self, reverse: bool) -> Option<Task<Message>> {
        if self.has_active_modal() || self.editor_state.is_some() {
            return None;
        }

        if self.is_concept_picker_open {
            self.is_concept_picker_open = false;
            self.concept_picker_search.clear();
            self.concept_picker_index = 0;
        }

        if self.active_tab == Tab::InformationModel
            && (self.selected_info_class_id.is_some()
                || self.selected_info_enum_id.is_some()
                || self.selected_info_edge.is_some())
        {
            let targets = self.build_info_inspector_targets();
            if targets.is_empty() {
                return None;
            }
            if self.info_inspector_focus.count() == 0 {
                self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
            } else {
                self.info_inspector_focus.update_targets(targets);
            }

            if reverse {
                self.info_inspector_focus.previous();
            } else {
                self.info_inspector_focus.next();
            }

            match self.info_inspector_focus.current_target() {
                Some(FocusTarget::Input(id)) => {
                    return Some(operation::focus(*id));
                }
                Some(FocusTarget::Button(_)) => {
                    return Some(operation::focus(iced::widget::Id::unique()));
                }
                None => return Some(Task::none()),
            }
        }

        None
    }

    pub fn advance_concept_inspector_focus(&mut self, reverse: bool) -> Option<Task<Message>> {
        if self.has_active_modal() || self.editor_state.is_some() {
            return None;
        }

        if self.active_tab == Tab::ConceptModel && self.selected_edge.is_some() {
            let targets = self.build_concept_inspector_targets();
            if targets.is_empty() {
                return None;
            }
            if self.concept_inspector_focus.count() == 0 {
                self.concept_inspector_focus = FocusCycle::with_targets(targets, Some(0));
            } else {
                self.concept_inspector_focus.update_targets(targets);
            }

            if reverse {
                self.concept_inspector_focus.previous();
            } else {
                self.concept_inspector_focus.next();
            }

            match self.concept_inspector_focus.current_target() {
                Some(FocusTarget::Input(id)) => {
                    return Some(operation::focus(*id));
                }
                Some(FocusTarget::Button(_)) => {
                    return Some(operation::focus(iced::widget::Id::unique()));
                }
                None => return Some(Task::none()),
            }
        }

        None
    }

    pub fn advance_modal_focus(&mut self, reverse: bool) -> Option<Task<Message>> {
        if !self.has_active_modal() {
            return None;
        }

        if self.modal_focus.count() > 0 {
            if reverse {
                self.modal_focus.previous();
            } else {
                self.modal_focus.next();
            }

            match self.modal_focus.current_target() {
                Some(FocusTarget::Input(id)) => {
                    return Some(operation::focus(*id));
                }
                Some(FocusTarget::Button(_)) => {
                    return Some(operation::focus(iced::widget::Id::unique()));
                }
                None => return Some(Task::none()),
            }
        }

        // VIGTIGT: Når en modal er åben, må fokus ALDRIG lække ud til baggrundsfanerne!
        Some(Task::none())
    }

    pub fn advance_editor_focus(&mut self, reverse: bool) -> Option<Task<Message>> {
        if let Some(editor) = &mut self.editor_state {
            let target = editor.advance_focus(reverse);
            match target {
                FocusTarget::Input(id) => {
                    return Some(operation::focus(id));
                }
                FocusTarget::Button(_) => {
                    return Some(operation::focus(iced::widget::Id::unique()));
                }
            }
        }
        None
    }

    pub fn has_collab_feature() -> bool {
        false
    }

    pub fn has_git_feature() -> bool {
        false
    }

    pub fn current_file_path(&self) -> Option<&std::path::PathBuf> {
        self.current_file_path.as_ref()
    }

    pub fn available_menu_types() -> &'static [MenuType] {
        &[MenuType::File, MenuType::Export, MenuType::Help]
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

    pub fn close_concept_editor(&mut self) {
        if let Some(pending_id) = self.pending_quick_create.take() {
            let is_empty = self
                .editor_state
                .as_ref()
                .map(|e| e.preferred_term.trim().is_empty())
                .unwrap_or(true);
            if is_empty {
                let node_id = self
                    .project
                    .concept_graph()
                    .find_node_by_concept(pending_id)
                    .map(|n| n.id());
                self.project.remove_concept(pending_id);
                if let Some(nid) = node_id {
                    if self.selected_graph_node_id == Some(nid) {
                        self.selected_graph_node_id = None;
                    }
                    self.selected_graph_node_ids.remove(&nid);
                }
                self.trigger_autosave();
                crate::features::diagnostics::log_info(
                    "canvas",
                    &format!("Rullede tomt lynoprettet begreb {} tilbage", pending_id),
                );
            }
        }
        self.editor_state = None;
        self.is_inline_graph_editing = false;
    }

    pub fn quick_create_concept_at(
        &mut self,
        x: f32,
        y: f32,
        source: QuickCreateSource,
    ) -> Task<Message> {
        let (nx, ny) = (x, y);
        let concept = Concept::new("Nyt begreb", "Definition mangler", BelongsToDomain::Yes);
        if let Ok(concept_id) = self.project.add_concept(concept.clone()) {
            let node_id = self
                .project
                .concept_graph_mut()
                .add_node_exact(&concept, nx, ny);
            self.selected_graph_node_id = Some(node_id);
            self.selected_graph_node_ids = [node_id].into_iter().collect();
            self.selected_edge = None;
            let mut editor = ConceptEditorState::from_concept(&concept);
            editor.preferred_term = String::new();
            editor.set_inline(true);
            self.editor_state = Some(editor);
            self.is_inline_graph_editing = true;
            self.pending_quick_create = Some(concept_id);

            let source_str = match source {
                QuickCreateSource::DoubleClick => "dobbeltklik",
                QuickCreateSource::Toolbar => "værktøjslinje",
            };
            crate::features::diagnostics::log_info(
                "canvas",
                &format!(
                    "Lynoprettede begreb {} via {} på ({:.1}, {:.1})",
                    concept_id, source_str, nx, ny
                ),
            );

            self.trigger_autosave();
            return operation::focus("preferred_term_input");
        }
        Task::none()
    }

    pub fn trigger_autosave(&mut self) {
        if let Some(path) = &self.current_file_path {
            let save_start = std::time::Instant::now();
            match ProjectStorage::save(&self.project, path) {
                Ok(()) => {
                    let save_duration = save_start.elapsed();
                    self.save_status = SaveStatus::Saved {
                        path: path.display().to_string(),
                        timestamp: current_timestamp(),
                    };
                    crate::features::diagnostics::log_info(
                        "autosave",
                        &format!("Autosave gennemført (disk: {:?})", save_duration),
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
        self.update_inner(message)
    }

    fn update_inner(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectTab(tab) => {
                if self.is_inline_graph_editing {
                    self.close_concept_editor();
                }
                self.active_tab = tab;
                self.toolbar_focus.reset();
                self.info_inspector_focus.reset();
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
                if self.show_left_sidebar {
                    match self.active_tab {
                        Tab::ConceptModel => {
                            self.concept_palette_index = 0;
                            return operation::focus("concept_model_search_input");
                        }
                        Tab::InformationModel => {
                            self.info_palette_index = 0;
                            return operation::focus("info_class_search_input");
                        }
                        _ => {}
                    }
                }
            }
            Message::OpenMetadataModal => {
                self.active_menu = None;
                let targets = vec![
                    FocusTarget::Input("meta_name"),
                    FocusTarget::Input("meta_desc"),
                    FocusTarget::Input("meta_uri"),
                    FocusTarget::Input("meta_approved_by"),
                    FocusTarget::Input("meta_domain_area"),
                    FocusTarget::Input("meta_responsible_org"),
                    FocusTarget::Input("meta_version"),
                    FocusTarget::Input("meta_language"),
                    FocusTarget::Input("meta_date_modified"),
                    FocusTarget::Input("meta_version_notes"),
                    FocusTarget::Input("meta_legal_sources"),
                    FocusTarget::Input("meta_source"),
                    FocusTarget::Input("meta_derived_from"),
                    FocusTarget::Button(META_BTN_CANCEL),
                    FocusTarget::Button(META_BTN_SAVE),
                ];
                self.modal_focus = FocusCycle::with_targets(targets, Some(0));
                self.metadata_modal = Some(ModelMetadataModalState::from_metadata(
                    self.project.metadata(),
                ));
                return operation::focus("meta_name");
            }
            Message::CloseMetadataModal => {
                self.metadata_modal = None;
                self.modal_focus.reset();
            }
            Message::OpenDiagnosticsModal => {
                self.diagnostics_modal = Some(DiagnosticsModalState::new());
                self.modal_focus = FocusCycle::with_initial(2, Some(0));
                self.active_menu = None;
            }
            Message::CloseDiagnosticsModal => {
                self.diagnostics_modal = None;
                self.modal_focus.reset();
            }
            Message::CopyDiagnosticsToClipboard => {
                let report =
                    crate::features::diagnostics::get_system_diagnostics().formatted_report();
                return iced::clipboard::write(report);
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
                let input_id = match field {
                    MetadataField::Name => "meta_name",
                    MetadataField::Description => "meta_desc",
                    MetadataField::DomainArea => "meta_domain_area",
                    MetadataField::ResponsibleOrg => "meta_responsible_org",
                    MetadataField::Uri => "meta_uri",
                    MetadataField::Version => "meta_version",
                    MetadataField::ApprovedBy => "meta_approved_by",
                    MetadataField::Language => "meta_language",
                    MetadataField::LegalSources => "meta_legal_sources",
                    MetadataField::Source => "meta_source",
                    MetadataField::WasDerivedFrom => "meta_derived_from",
                    MetadataField::VersionNotes => "meta_version_notes",
                    MetadataField::DateModified => "meta_date_modified",
                };
                self.modal_focus.sync_input(input_id);
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
                let targets = vec![
                    FocusTarget::Input("meta_name"),
                    FocusTarget::Input("meta_desc"),
                    FocusTarget::Input("meta_uri"),
                    FocusTarget::Input("meta_approved_by"),
                    FocusTarget::Input("meta_domain_area"),
                    FocusTarget::Input("meta_responsible_org"),
                    FocusTarget::Input("meta_version"),
                    FocusTarget::Input("meta_language"),
                    FocusTarget::Input("meta_date_modified"),
                    FocusTarget::Input("meta_version_notes"),
                    FocusTarget::Input("meta_legal_sources"),
                    FocusTarget::Input("meta_source"),
                    FocusTarget::Input("meta_derived_from"),
                    FocusTarget::Button(META_BTN_CANCEL),
                    FocusTarget::Button(META_BTN_SAVE),
                ];
                self.modal_focus = FocusCycle::with_targets(targets, Some(0));
                self.metadata_modal = Some(ModelMetadataModalState::from_metadata(
                    self.project.metadata(),
                ));
                self.editor_state = None;
                self.is_inline_graph_editing = false;
                self.search_query.clear();
                self.current_file_path = None;
                self.save_status = SaveStatus::Unsaved;
                self.selected_graph_node_id = None;
                self.selected_edge = None;
                self.relation_dialog = None;
                self.quick_create = None;
                self.selected_info_class_id = None;
                self.selected_info_enum_id = None;
                self.selected_info_graph_node_id = None;
                self.selected_info_graph_node_ids.clear();
                self.selected_info_edge = None;
                self.info_class_search.clear();
                return operation::focus("meta_name");
            }
            Message::StartNewConcept => {
                self.editor_state = Some(ConceptEditorState::new_empty());
                self.is_inline_graph_editing = false;
                self.toolbar_focus.reset();
                return operation::focus("preferred_term_input");
            }
            Message::EditConcept(id) => {
                if let Some(concept) = self.project.get_concept(id) {
                    self.editor_state = Some(ConceptEditorState::from_concept(concept));
                    self.is_inline_graph_editing = false;
                    self.active_tab = Tab::ConceptList;
                    self.toolbar_focus.reset();
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
                                    self.pending_quick_create = None;
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
                self.close_concept_editor();
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
                self.toolbar_focus.sync_input("concept_search_input");
            }
            Message::ToggleShowAllFields => {
                if let Some(editor) = &mut self.editor_state {
                    editor.show_supplementary = !editor.show_supplementary;
                }
            }

            // Persistens & Filhåndtering
            Message::SaveProject => {
                self.active_menu = None;
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
                let targets = vec![
                    FocusTarget::Input("file_dialog_input"),
                    FocusTarget::Button(0),
                    FocusTarget::Button(1),
                ];
                self.modal_focus = FocusCycle::with_targets(targets, Some(0));
                return operation::focus("file_dialog_input");
            }
            Message::CloseFileDialog => {
                self.modal_focus.reset();
                self.file_dialog_mode = None;
            }
            Message::FileDialogInputChanged(path_str) => {
                self.modal_focus.sync_input("file_dialog_input");
                self.file_dialog_input = path_str;
            }
            Message::ConfirmFileDialog => {
                self.modal_focus.reset();
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
                        return Task::none();
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
                                    let update_metadata = import_summary.metadata.is_some();
                                    self.modal_focus = FocusCycle::with_initial(3, Some(2));
                                    self.excel_import_preview = Some(ExcelImportPreviewState {
                                        import_summary,
                                        update_metadata,
                                        new_count,
                                        update_count,
                                        file_path: path,
                                    });
                                }
                                Err(err) => {
                                    crate::features::diagnostics::log_error(
                                        "excel",
                                        &format!("Fejl ved parsing af Excel-regneark: {}", err),
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            crate::features::diagnostics::log_error(
                                "excel",
                                &format!("Fejl ved indlæsning af fil {:?}: {}", path, err),
                            );
                        }
                    }
                }
            }
            Message::OpenExcelSchemaGuideModal => {
                self.active_menu = None;
                self.modal_focus = FocusCycle::with_initial(2, Some(1));
                self.excel_schema_guide_open = true;
            }
            Message::CloseExcelSchemaGuideModal => {
                self.modal_focus.reset();
                self.excel_schema_guide_open = false;
            }
            Message::ConfirmExcelImport => {
                self.modal_focus.reset();
                if let Some(preview) = self.excel_import_preview.take() {
                    if preview.update_metadata {
                        if let Some(meta) = preview.import_summary.metadata {
                            crate::features::diagnostics::log_info(
                                "excel",
                                &format!(
                                    "Opdaterer modelmetadata fra Forretningsmetadata: \"{}\" ({})",
                                    meta.name(),
                                    meta.uri()
                                ),
                            );
                            *self.project.metadata_mut() = meta;
                        }
                    }
                    let summary = crate::features::concepts::excel::apply_concept_upsert(
                        &mut self.project,
                        preview.import_summary.valid_concepts,
                    );
                    crate::features::diagnostics::log_info(
                        "excel",
                        &format!(
                            "Excel import bekræftet og gennemført: {} nye begreber tilføjet, {} eksisterende opdateret",
                            summary.inserted, summary.updated
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::ConfirmExcelImportAsNewModel => {
                self.modal_focus.reset();
                if let Some(preview) = self.excel_import_preview.take() {
                    self.active_menu = None;
                    self.project = ModelProject::default();
                    self.active_tab = Tab::ConceptList;
                    self.editor_state = None;
                    self.is_inline_graph_editing = false;
                    self.search_query.clear();
                    self.current_file_path = None;
                    self.save_status = SaveStatus::Unsaved;
                    self.selected_graph_node_id = None;
                    self.selected_edge = None;
                    self.relation_dialog = None;
                    self.quick_create = None;
                    self.selected_info_class_id = None;
                    self.selected_info_enum_id = None;
                    self.selected_info_graph_node_id = None;
                    self.selected_info_graph_node_ids.clear();
                    self.selected_info_edge = None;
                    self.info_class_search.clear();

                    if let Some(meta) = preview.import_summary.metadata {
                        crate::features::diagnostics::log_info(
                            "excel",
                            &format!(
                                "Initialiserer ny model med Forretningsmetadata: \"{}\" ({})",
                                meta.name(),
                                meta.uri()
                            ),
                        );
                        *self.project.metadata_mut() = meta;
                    } else {
                        let fallback_name = preview
                            .file_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Ny model");
                        self.project.metadata_mut().set_name(fallback_name);
                    }

                    let summary = crate::features::concepts::excel::apply_concept_upsert(
                        &mut self.project,
                        preview.import_summary.valid_concepts,
                    );
                    crate::features::diagnostics::log_info(
                        "excel",
                        &format!(
                            "Oprettet ny model fra Excel import med {} begreber",
                            summary.inserted
                        ),
                    );
                    self.trigger_autosave();
                }
            }
            Message::DismissExcelImportModal => {
                self.modal_focus.reset();
                self.excel_import_preview = None;
            }
            Message::SelectExcelImportSheet(sheet_name) => {
                if let Some(preview) = &mut self.excel_import_preview {
                    if let Some(candidate) = preview
                        .import_summary
                        .available_sheets
                        .iter()
                        .find(|s| s.sheet_name == sheet_name)
                    {
                        preview.import_summary.sheet_name = candidate.sheet_name.clone();
                        preview.import_summary.valid_concepts = candidate.valid_concepts.clone();
                        preview.import_summary.rejected_rows = candidate.rejected_rows.clone();

                        let mut new_count = 0;
                        let mut update_count = 0;
                        for c in &preview.import_summary.valid_concepts {
                            let is_match = self.project.concepts().iter().any(|existing| {
                                (c.identifier().is_some()
                                    && c.identifier() == existing.identifier())
                                    || c.preferred_term()
                                        .trim()
                                        .eq_ignore_ascii_case(existing.preferred_term().trim())
                            });
                            if is_match {
                                update_count += 1;
                            } else {
                                new_count += 1;
                            }
                        }
                        preview.new_count = new_count;
                        preview.update_count = update_count;
                    }
                }
            }
            Message::ToggleExcelImportMetadata(val) => {
                if let Some(preview) = &mut self.excel_import_preview {
                    preview.update_metadata = val;
                }
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
                if let Some(task) = self.advance_modal_focus(false) {
                    return task;
                }
                if let Some(task) = self.advance_editor_focus(false) {
                    return task;
                }
                if let Some(task) = self.advance_info_inspector_focus(false) {
                    return task;
                }
                if let Some(task) = self.advance_concept_inspector_focus(false) {
                    return task;
                }
                if let Some(task) = self.advance_tab_focus(false) {
                    return task;
                }
                return operation::focus_next();
            }
            Message::FocusPrevious => {
                if let Some(task) = self.advance_modal_focus(true) {
                    return task;
                }
                if let Some(task) = self.advance_editor_focus(true) {
                    return task;
                }
                if let Some(task) = self.advance_info_inspector_focus(true) {
                    return task;
                }
                if let Some(task) = self.advance_concept_inspector_focus(true) {
                    return task;
                }
                if let Some(task) = self.advance_tab_focus(true) {
                    return task;
                }
                return operation::focus_previous();
            }
            Message::FocusEditorButton(btn) => {
                if let Some(editor) = &mut self.editor_state {
                    editor.set_focused_button(btn);
                }
            }
            Message::EscapePressed => {
                // 1. Concept Picker combobox lukning
                if self.is_concept_picker_open {
                    self.is_concept_picker_open = false;
                    self.concept_picker_search.clear();
                    self.concept_picker_index = 0;
                    return Task::none();
                }

                // 1b. Palet søgefelt Escape jf. Task 090 AC6
                if self.active_tab == Tab::ConceptModel
                    && self.show_left_sidebar
                    && !self.concept_model_search.is_empty()
                {
                    self.concept_model_search.clear();
                    self.concept_palette_index = 0;
                    return Task::none();
                } else if self.active_tab == Tab::InformationModel
                    && self.show_left_sidebar
                    && !self.info_class_search.is_empty()
                {
                    self.info_class_search.clear();
                    self.info_palette_index = 0;
                    return Task::none();
                }

                // 2. Aktiv menu overlay
                if self.active_menu.is_some() {
                    self.active_menu = None;
                    return Task::none();
                }

                // 3. Modale dialoger (alle 9 modaler inkl. diagnostik samt overlay-dialoger)
                if self.close_active_modal() {
                    return Task::none();
                }

                // 3. Inline redigering på canvas eller i begrebslisten
                if self.is_inline_graph_editing || self.editor_state.is_some() {
                    self.close_concept_editor();
                    return Task::none();
                }

                // 4. Canvas afmarkering (noder, multiselection og kanter)
                let had_selection = self.selected_edge.is_some()
                    || self.selected_graph_node_id.is_some()
                    || !self.selected_graph_node_ids.is_empty()
                    || self.selected_info_edge.is_some()
                    || self.selected_info_graph_node_id.is_some()
                    || !self.selected_info_graph_node_ids.is_empty()
                    || self.selected_info_class_id.is_some()
                    || self.selected_info_enum_id.is_some();

                if had_selection {
                    self.selected_edge = None;
                    self.concept_inspector_focus = FocusCycle::default();
                    self.selected_graph_node_id = None;
                    self.selected_graph_node_ids.clear();
                    self.selected_info_edge = None;
                    self.info_inspector_focus = FocusCycle::default();
                    self.selected_info_graph_node_id = None;
                    self.selected_info_graph_node_ids.clear();
                    self.selected_info_class_id = None;
                    self.selected_info_enum_id = None;
                    return Task::none();
                }

                // 4b. Luk åben palet jf. Task 090 AC6 (hvis søgefeltet allerede er tomt)
                if self.show_left_sidebar
                    && (self.active_tab == Tab::ConceptModel
                        || self.active_tab == Tab::InformationModel)
                {
                    self.show_left_sidebar = false;
                    return operation::focus(iced::widget::Id::unique());
                }

                // 5. Landing på de 3 primære faneblade
                let tab_id = match self.active_tab {
                    Tab::ConceptList => "tab_concept_list",
                    Tab::ConceptModel => "tab_concept_model",
                    Tab::InformationModel => "tab_information_model",
                };
                return operation::focus(tab_id);
            }
            Message::EnterPressed => {
                if self.is_concept_picker_open {
                    return self.update(Message::ConceptPickerSubmit);
                }

                // 1. Åbne modaler
                if self.has_active_modal() {
                    let focused = self.modal_focus.focused_button();
                    if self.metadata_modal.is_some() {
                        match focused {
                            Some(META_BTN_CANCEL) => {
                                return self.update(Message::CloseMetadataModal)
                            }
                            Some(META_BTN_SAVE) | None => {
                                return self.update(Message::SaveMetadataModal)
                            }
                            _ => {}
                        }
                    } else if self.excel_import_preview.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::DismissExcelImportModal),
                            Some(1) => return self.update(Message::ConfirmExcelImportAsNewModel),
                            Some(2) | None => return self.update(Message::ConfirmExcelImport),
                            _ => {}
                        }
                    } else if self.diagnostics_modal.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::CopyDiagnosticsToClipboard),
                            Some(1) | None => return self.update(Message::CloseDiagnosticsModal),
                            _ => {}
                        }
                    } else if self.excel_schema_guide_open {
                        return self.update(Message::CloseExcelSchemaGuideModal);
                    } else if self.file_dialog_mode.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::CloseFileDialog),
                            Some(1) | None => return self.update(Message::ConfirmFileDialog),
                            _ => {}
                        }
                    } else if self.relation_dialog.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::GraphCloseRelationDialog),
                            Some(1) | None => return self.update(Message::GraphCreateRelation),
                            _ => {}
                        }
                    } else if self.info_relation_dialog.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::CloseInfoRelationDialog),
                            Some(1) | None => return self.update(Message::InfoCreateRelation),
                            _ => {}
                        }
                    } else if self.quick_create.is_some() {
                        match focused {
                            Some(0) => return self.update(Message::QuickCreateCancel),
                            Some(1) | None => return self.update(Message::QuickCreateSubmit),
                            _ => {}
                        }
                    }
                    return Task::none();
                }

                // 2. Hvis ConceptEditor har en fokuseret handlingsknap
                if let Some(editor) = &self.editor_state {
                    match editor.focused_button() {
                        Some(crate::ui::concept_editor::BTN_CANCEL) => {
                            return self.update(Message::CancelConceptEdit);
                        }
                        Some(crate::ui::concept_editor::BTN_SAVE) => {
                            return self.update(Message::SaveConcept);
                        }
                        Some(crate::ui::concept_editor::BTN_DOMAIN_LOCAL) => {
                            return self.update(Message::SetConceptDomain(
                                crate::features::concepts::BelongsToDomain::Yes,
                            ));
                        }
                        Some(crate::ui::concept_editor::BTN_DOMAIN_BORROWED) => {
                            return self.update(Message::SetConceptDomain(
                                crate::features::concepts::BelongsToDomain::No,
                            ));
                        }
                        Some(crate::ui::concept_editor::BTN_TOGGLE_SUPPLEMENTARY) => {
                            return self.update(Message::ToggleShowAllFields);
                        }
                        _ => {}
                    }
                }

                // 3. Hvis Informationsmodellens inspektor har en fokuseret knap
                if let Some(btn) = self.info_inspector_focused_button() {
                    if let Some(cid) = self.selected_info_class_id {
                        match btn {
                            INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX => {
                                if let Some(cls) = self.project.information_model().get_class(cid) {
                                    return self.update(Message::SetInformationClassAbstract(
                                        cid,
                                        !cls.is_abstract(),
                                    ));
                                }
                            }
                            INFO_INSPECTOR_BTN_TYPE_LOCAL => {
                                return self.update(Message::SetInformationClassLocal(cid, true));
                            }
                            INFO_INSPECTOR_BTN_TYPE_BORROWED => {
                                return self.update(Message::SetInformationClassLocal(cid, false));
                            }
                            INFO_INSPECTOR_BTN_ADD_ATTRIBUTE => {
                                return self.update(Message::AddAttributeToClass(cid));
                            }
                            INFO_INSPECTOR_BTN_CANVAS_ACTION => {
                                if let Some(node) =
                                    self.project.information_graph().find_node_by_class(cid)
                                {
                                    let node_id = node.id();
                                    return self.update(Message::RemoveClassFromDiagram(node_id));
                                } else {
                                    return self.update(Message::AddClassToDiagram(cid));
                                }
                            }
                            INFO_INSPECTOR_BTN_DELETE_CLASS => {
                                return self.update(Message::DeleteInformationClass(cid));
                            }
                            INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER => {
                                return self.update(Message::OpenConceptPicker);
                            }
                            _ if btn >= INFO_INSPECTOR_REL_DELETE_START => {
                                let rel_idx = btn - INFO_INSPECTOR_REL_DELETE_START;
                                let class_relations =
                                    self.project.information_model().relations_for_class(cid);
                                if let Some(rel) = class_relations.get(rel_idx) {
                                    let from = rel.from_class();
                                    let to = rel.to_class();
                                    return self.update(Message::DeleteClassRelation(from, to));
                                }
                            }
                            _ if btn >= INFO_INSPECTOR_ATTR_DELETE_START => {
                                let attr_idx = btn - INFO_INSPECTOR_ATTR_DELETE_START;
                                if let Some(cls) = self.project.information_model().get_class(cid) {
                                    if let Some(attr) = cls.attributes().get(attr_idx) {
                                        let attr_id = attr.id();
                                        return self.update(Message::DeleteAttribute(cid, attr_id));
                                    }
                                }
                            }
                            _ if btn >= INFO_INSPECTOR_CONCEPT_REMOVE_START => {
                                let concept_idx = btn - INFO_INSPECTOR_CONCEPT_REMOVE_START;
                                if let Some(cls) = self.project.information_model().get_class(cid) {
                                    if let Some(concept_id) =
                                        cls.concept_ids().get(concept_idx).copied()
                                    {
                                        return self.update(
                                            Message::RemoveConceptFromInformationClass(
                                                cid, concept_id,
                                            ),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else if let Some(eid) = self.selected_info_enum_id {
                        match btn {
                            INFO_INSPECTOR_BTN_CANVAS_ACTION => {
                                if self.project.information_graph().is_class_on_diagram(eid) {
                                    return self.update(Message::RemoveEnumerationFromDiagram(eid));
                                } else {
                                    return self.update(Message::AddEnumerationToDiagram(eid));
                                }
                            }
                            INFO_INSPECTOR_BTN_DELETE_ENUM => {
                                return self.update(Message::DeleteInformationEnumeration(eid));
                            }
                            INFO_INSPECTOR_BTN_ADD_ENUM_VAL => {
                                let val = self.new_enum_value_input.trim().to_string();
                                if !val.is_empty() {
                                    return self.update(Message::AddValueToEnumeration(eid, val));
                                }
                            }
                            _ if btn >= INFO_INSPECTOR_ENUM_VAL_REMOVE_START => {
                                let val_idx = btn - INFO_INSPECTOR_ENUM_VAL_REMOVE_START;
                                if let Some(e) =
                                    self.project.information_model().get_enumeration(eid)
                                {
                                    if let Some(val) = e.values().get(val_idx).cloned() {
                                        return self
                                            .update(Message::RemoveValueFromEnumeration(eid, val));
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else if let Some((from_id, to_id)) = self.selected_info_edge {
                        match btn {
                            INFO_EDGE_BTN_REVERSE => {
                                return self.update(Message::InfoReverseEdge(from_id, to_id));
                            }
                            INFO_EDGE_BTN_KIND_ASSOC => {
                                return self.update(Message::InfoUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Association,
                                ));
                            }
                            INFO_EDGE_BTN_KIND_GEN => {
                                return self.update(Message::InfoUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Generalization,
                                ));
                            }
                            INFO_EDGE_BTN_KIND_COMP => {
                                return self.update(Message::InfoUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Composition,
                                ));
                            }
                            INFO_EDGE_BTN_KIND_DEP => {
                                return self.update(Message::InfoUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Dependency,
                                ));
                            }
                            INFO_EDGE_BTN_DIRECTED_CHECKBOX => {
                                if let Some(edge) =
                                    self.project.information_graph().find_edge(from_id, to_id)
                                {
                                    return self.update(Message::InfoToggleEdgeDirected(
                                        from_id,
                                        to_id,
                                        !edge.is_directed(),
                                    ));
                                }
                            }
                            INFO_EDGE_BTN_SRC_MULT => {
                                if let Some(edge) =
                                    self.project.information_graph().find_edge(from_id, to_id)
                                {
                                    let current = edge.source_multiplicity();
                                    let presets =
                                        crate::features::information_model::Multiplicity::PRESETS;
                                    let next_idx = match current {
                                        Some(m) => {
                                            (presets.iter().position(|&p| p == m).unwrap_or(0) + 1)
                                                % presets.len()
                                        }
                                        None => 0,
                                    };
                                    return self.update(Message::InfoUpdateEdgeSourceMultiplicity(
                                        from_id,
                                        to_id,
                                        Some(presets[next_idx]),
                                    ));
                                }
                            }
                            INFO_EDGE_BTN_TGT_MULT => {
                                if let Some(edge) =
                                    self.project.information_graph().find_edge(from_id, to_id)
                                {
                                    let current = edge.target_multiplicity();
                                    let presets =
                                        crate::features::information_model::Multiplicity::PRESETS;
                                    let next_idx = match current {
                                        Some(m) => {
                                            (presets.iter().position(|&p| p == m).unwrap_or(0) + 1)
                                                % presets.len()
                                        }
                                        None => 0,
                                    };
                                    return self.update(Message::InfoUpdateEdgeTargetMultiplicity(
                                        from_id,
                                        to_id,
                                        Some(presets[next_idx]),
                                    ));
                                }
                            }
                            INFO_EDGE_BTN_DELETE => {
                                return self.update(Message::DeleteClassRelation(from_id, to_id));
                            }
                            _ => {}
                        }
                    }
                }

                // 3b. Hvis Begrebsdiagrammets inspektor har en fokuseret knap
                if let Some(btn) = self.concept_inspector_focused_button() {
                    if let Some((from_id, to_id)) = self.selected_edge {
                        match btn {
                            CONCEPT_EDGE_BTN_REVERSE => {
                                return self.update(Message::GraphReverseEdge(from_id, to_id));
                            }
                            CONCEPT_EDGE_BTN_KIND_ASSOC => {
                                return self.update(Message::GraphUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Association,
                                ));
                            }
                            CONCEPT_EDGE_BTN_KIND_GEN => {
                                return self.update(Message::GraphUpdateEdgeKind(
                                    from_id,
                                    to_id,
                                    RelationKind::Generalization,
                                ));
                            }
                            CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX => {
                                if let Some(edge) =
                                    self.project.concept_graph().find_edge(from_id, to_id)
                                {
                                    return self.update(Message::GraphToggleEdgeDirected(
                                        from_id,
                                        to_id,
                                        !edge.is_directed(),
                                    ));
                                }
                            }
                            CONCEPT_EDGE_BTN_DELETE => {
                                return self.update(Message::GraphDeleteRelation(from_id, to_id));
                            }
                            _ => {}
                        }
                    }
                }

                // 4. Hvis Værktøjslinjen (Tab 1) har en fokuseret knap
                if let Some(btn) = self.toolbar_focused_button() {
                    if btn >= TOOLBAR_ROW_BTN_START {
                        let rel = btn - TOOLBAR_ROW_BTN_START;
                        let row_idx = rel / 2;
                        let is_delete = rel % 2 == 1;
                        let filtered = self.filtered_concepts();
                        if let Some(concept) = filtered.get(row_idx) {
                            let id = concept.id();
                            if is_delete {
                                return self.update(Message::DeleteConcept(id));
                            } else {
                                return self.update(Message::EditConcept(id));
                            }
                        }
                    } else {
                        match btn {
                            TOOLBAR_BTN_NEW_CONCEPT => {
                                return self.update(Message::StartNewConcept)
                            }
                            TOOLBAR_BTN_IMPORT_EXCEL => {
                                return self.update(Message::ImportConceptsExcelDialog)
                            }
                            TOOLBAR_BTN_EXPORT_EXCEL => {
                                return self.update(Message::ExportConceptListXlsxDialog)
                            }
                            TOOLBAR_BTN_TEMPLATE => {
                                return self.update(Message::ExportFdaTemplateXlsxDialog)
                            }
                            TOOLBAR_BTN_GUIDE => {
                                return self.update(Message::OpenExcelSchemaGuideModal)
                            }
                            _ => {}
                        }
                    }
                }

                match self.active_tab {
                    Tab::ConceptModel => {
                        if let Some(node_id) = self.selected_graph_node_id {
                            if let Some(node) = self.project.concept_graph().find_node(node_id) {
                                if let Some(concept) = self.project.get_concept(node.concept_id()) {
                                    let mut editor = ConceptEditorState::from_concept(concept);
                                    editor.set_inline(true);
                                    self.editor_state = Some(editor);
                                    self.is_inline_graph_editing = true;
                                    return operation::focus("preferred_term_input");
                                }
                            }
                        } else if self.selected_edge.is_some() {
                            return operation::focus("edge_label_input");
                        } else if self.show_left_sidebar
                            && !self.filtered_palette_concepts().is_empty()
                        {
                            return self.update(Message::ConceptPaletteSubmit);
                        }
                    }
                    Tab::InformationModel => {
                        if let Some(node_id) = self.selected_info_graph_node_id {
                            if let Some(node) = self.project.information_graph().find_node(node_id)
                            {
                                let cid = node.class_id();
                                if self
                                    .project
                                    .information_model()
                                    .get_enumeration(cid)
                                    .is_some()
                                {
                                    self.selected_info_enum_id = Some(cid);
                                    self.selected_info_class_id = None;
                                    return operation::focus("info_enum_name_input");
                                } else {
                                    self.selected_info_class_id = Some(cid);
                                    self.selected_info_enum_id = None;
                                    return operation::focus("info_class_name_input");
                                }
                            }
                        } else if self.selected_info_enum_id.is_some() {
                            return operation::focus("info_enum_name_input");
                        } else if self.selected_info_class_id.is_some() {
                            return operation::focus("info_class_name_input");
                        } else if self.selected_info_edge.is_some() {
                            return operation::focus("info_edge_label_input");
                        } else if self.show_left_sidebar
                            && !self.filtered_palette_classes().is_empty()
                        {
                            return self.update(Message::InfoPaletteSubmit);
                        }
                    }
                    Tab::ConceptList => {
                        if self.editor_state.is_some() {
                            return operation::focus("preferred_term_input");
                        }
                    }
                }
                return Task::none();
            }

            // Graf-handlinger (Fase 3)
            Message::GraphNodeSelected(node_id) => {
                if self.is_inline_graph_editing {
                    let cur_node_id = self
                        .editor_state
                        .as_ref()
                        .and_then(|e| e.editing_id)
                        .and_then(|cid| self.project.concept_graph().find_node_by_concept(cid))
                        .map(|n| n.id());
                    if cur_node_id != node_id {
                        self.close_concept_editor();
                    }
                }
                self.selected_graph_node_id = node_id;
                self.selected_graph_node_ids = node_id.into_iter().collect();
                self.selected_edge = None;
            }
            Message::GraphNodesSelected(ids) => {
                if self.is_inline_graph_editing {
                    let cur_node_id = self
                        .editor_state
                        .as_ref()
                        .and_then(|e| e.editing_id)
                        .and_then(|cid| self.project.concept_graph().find_node_by_concept(cid))
                        .map(|n| n.id());
                    if !cur_node_id.map(|id| ids.contains(&id)).unwrap_or(false) {
                        self.close_concept_editor();
                    }
                }
                self.selected_graph_node_ids = ids;
                self.selected_graph_node_id = self.selected_graph_node_ids.iter().next().copied();
                self.selected_edge = None;
            }
            Message::GraphEdgeSelected(edge) => {
                if edge.is_some() && self.is_inline_graph_editing {
                    self.close_concept_editor();
                }
                self.selected_edge = edge;
                if edge.is_some() {
                    self.selected_graph_node_id = None;
                    self.selected_graph_node_ids.clear();
                    let targets = self.build_concept_inspector_targets();
                    self.concept_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                } else {
                    self.concept_inspector_focus.reset();
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
                    self.trigger_autosave();
                }
            }
            Message::GraphToggleEdgeDirected(from, to, directed) => {
                if self
                    .project
                    .concept_graph_mut()
                    .update_edge_directed(from, to, directed)
                {
                    self.trigger_autosave();
                }
            }
            Message::GraphReverseEdge(from, to) => {
                if self.project.concept_graph_mut().reverse_relation(from, to) {
                    self.selected_edge = Some((to, from));
                    self.trigger_autosave();
                }
            }
            Message::GraphDeleteSelected => match self.active_tab {
                Tab::ConceptModel => {
                    if let Some((from, to)) = self.selected_edge.take() {
                        self.project.concept_graph_mut().remove_relation(from, to);
                        self.trigger_autosave();
                    } else if !self.selected_graph_node_ids.is_empty() {
                        let to_remove: Vec<NodeId> = self.selected_graph_node_ids.drain().collect();
                        self.selected_graph_node_id = None;
                        for node_id in to_remove {
                            self.project.concept_graph_mut().remove_node(node_id);
                        }
                        self.trigger_autosave();
                    } else if let Some(node_id) = self.selected_graph_node_id.take() {
                        self.project.concept_graph_mut().remove_node(node_id);
                        self.trigger_autosave();
                    }
                }
                Tab::InformationModel => {
                    if let Some((from, to)) = self.selected_info_edge.take() {
                        self.project
                            .information_graph_mut()
                            .remove_relation(from, to);
                        self.trigger_autosave();
                    } else if !self.selected_info_graph_node_ids.is_empty() {
                        let to_remove: Vec<NodeId> =
                            self.selected_info_graph_node_ids.drain().collect();
                        self.selected_info_graph_node_id = None;
                        self.selected_info_class_id = None;
                        for node_id in to_remove {
                            self.project.information_graph_mut().remove_node(node_id);
                        }
                        self.trigger_autosave();
                    } else if let Some(node_id) = self.selected_info_graph_node_id.take() {
                        self.project.information_graph_mut().remove_node(node_id);
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
                        self.trigger_autosave();
                    }
                }
            }
            Message::RemoveConceptFromDiagram(node_id) => {
                self.project.concept_graph_mut().remove_node(node_id);
                if self.selected_graph_node_id == Some(node_id) {
                    self.selected_graph_node_id = None;
                }
                self.trigger_autosave();
            }
            Message::ConceptModelSearchChanged(query) => {
                self.concept_model_search = query;
                self.concept_palette_index = 0;
            }
            Message::ConceptPaletteNext => {
                let count = self.filtered_palette_concepts().len();
                if count > 0 {
                    self.concept_palette_index = (self.concept_palette_index + 1) % count;
                }
            }
            Message::ConceptPalettePrevious => {
                let count = self.filtered_palette_concepts().len();
                if count > 0 {
                    self.concept_palette_index = if self.concept_palette_index == 0 {
                        count - 1
                    } else {
                        self.concept_palette_index - 1
                    };
                }
            }
            Message::ConceptPaletteSubmit => {
                let filtered = self.filtered_palette_concepts();
                if let Some(concept) = filtered.get(self.concept_palette_index).copied() {
                    let concept_id = concept.id();
                    if let Some(node) = self
                        .project
                        .concept_graph()
                        .find_node_by_concept(concept_id)
                    {
                        let nid = node.id();
                        let nx = node.x();
                        let ny = node.y();
                        let nw = node.width();
                        let nh = node.height();
                        self.selected_graph_node_id = Some(nid);
                        self.selected_graph_node_ids.clear();
                        self.selected_graph_node_ids.insert(nid);
                        let center = self.canvas_visible_center();
                        self.canvas_viewport.center_on_rect(nx, ny, nw, nh, center);
                        return operation::focus(iced::widget::Id::unique());
                    } else {
                        let center_world =
                            self.canvas_viewport.to_world(self.canvas_visible_center());
                        let raw_x = center_world.x
                            - crate::features::concept_model::DEFAULT_NODE_WIDTH / 2.0;
                        let raw_y = center_world.y
                            - crate::features::concept_model::DEFAULT_NODE_HEIGHT / 2.0;
                        let _ =
                            self.update(Message::AddConceptToDiagramAt(concept_id, raw_x, raw_y));
                        return operation::focus(iced::widget::Id::unique());
                    }
                }
            }

            // Lynoprettelse & Node-redigering på Canvas (Task 006, 030, 039 & 077)
            Message::CanvasDoubleClicked(x, y) => {
                return self.quick_create_concept_at(x, y, QuickCreateSource::DoubleClick);
            }
            Message::CreateConceptAtCenter => {
                let center_world = self.canvas_viewport.to_world(Point::new(500.0, 350.0));
                let cx = center_world.x - crate::features::concept_model::DEFAULT_NODE_WIDTH / 2.0;
                let cy = center_world.y - crate::features::concept_model::DEFAULT_NODE_HEIGHT / 2.0;
                return self.quick_create_concept_at(cx, cy, QuickCreateSource::Toolbar);
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
                crate::features::diagnostics::log_info(
                    "canvas",
                    &format!("Dobbeltklik på node {} – åbner egenskaber", node_id),
                );
                self.selected_graph_node_id = Some(node_id);
                self.selected_graph_node_ids = Some(node_id).into_iter().collect();
                self.selected_edge = None;
                if let Some(node) = self.project.concept_graph().find_node(node_id) {
                    if let Some(concept) = self.project.get_concept(node.concept_id()) {
                        let mut editor = ConceptEditorState::from_concept(concept);
                        editor.set_inline(true);
                        self.editor_state = Some(editor);
                        self.is_inline_graph_editing = true;
                        return operation::focus("preferred_term_input");
                    }
                }
                return Task::none();
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
                if pressed {
                    if self.has_active_modal() && self.modal_focus.focused_button().is_some() {
                        return self.update(Message::EnterPressed);
                    }
                    if let Some(editor) = &self.editor_state {
                        if editor.focused_button().is_some() {
                            return self.update(Message::EnterPressed);
                        }
                    }
                    if self.info_inspector_focused_button().is_some() {
                        return self.update(Message::EnterPressed);
                    }
                    if self.toolbar_focused_button().is_some() {
                        return self.update(Message::EnterPressed);
                    }
                }
            }
            Message::CanvasModifiersChanged(_mods) => {}

            // Informationsmodel (Task 010 & 011)
            Message::SelectInformationClass(id) => {
                self.is_concept_picker_open = false;
                self.concept_picker_search.clear();
                self.concept_picker_index = 0;
                self.selected_info_class_id = id;
                if id.is_some() {
                    self.selected_info_enum_id = None;
                    self.selected_info_edge = None;
                }
                if let Some(cid) = id {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                    let node_id = self
                        .project
                        .information_graph()
                        .find_node_by_class(cid)
                        .map(|n| n.id());
                    self.set_selected_info_graph_node(node_id);
                } else {
                    self.info_inspector_focus.reset();
                    self.set_selected_info_graph_node(None);
                }
            }
            Message::CreateInformationClass => {
                let class = InformationClass::new("");
                let id = self.project.information_model_mut().add_class(class);
                let node_id = self.project.information_graph_mut().add_node(id, 0);
                self.selected_info_class_id = Some(id);
                self.selected_info_enum_id = None;
                self.set_selected_info_graph_node(Some(node_id));
                self.selected_info_edge = None;
                let targets = self.build_info_inspector_targets();
                self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
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
                self.selected_info_enum_id = None;
                self.set_selected_info_graph_node(Some(node_id));
                self.selected_info_edge = None;
                let targets = self.build_info_inspector_targets();
                self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
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
                    self.selected_info_enum_id = None;
                    self.set_selected_info_graph_node(Some(node_id));
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
                    let _cls = class.clone();
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
                    let _cls = class.clone();
                    self.trigger_autosave();
                }
            }
            Message::SetInformationClassAbstract(class_id, is_abstract) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.set_abstract(is_abstract);
                    let _cls = class.clone();
                    self.trigger_autosave();
                }
            }
            Message::SetInformationClassLocal(class_id, is_local) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.set_local(is_local);
                    let _cls = class.clone();
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
                    let _cls = class.clone();
                    self.trigger_autosave();
                }
            }
            Message::OpenConceptPicker => {
                if self.selected_info_class_id.is_some() {
                    self.is_concept_picker_open = true;
                    self.concept_picker_search.clear();
                    self.concept_picker_index = 0;
                    return operation::focus("info_concept_picker_input");
                }
            }
            Message::CloseConceptPicker => {
                self.is_concept_picker_open = false;
                self.concept_picker_search.clear();
                self.concept_picker_index = 0;
            }
            Message::ConceptPickerSearchChanged(q) => {
                self.concept_picker_search = q;
                self.concept_picker_index = 0;
            }
            Message::ConceptPickerNext => {
                if self.is_concept_picker_open {
                    if let Some(cid) = self.selected_info_class_id {
                        let count = self.filtered_available_concepts_for_class(cid).len();
                        if count > 0 {
                            self.concept_picker_index = (self.concept_picker_index + 1) % count;
                        }
                    }
                } else if self.active_tab == Tab::ConceptModel && self.show_left_sidebar {
                    return self.update(Message::ConceptPaletteNext);
                } else if self.active_tab == Tab::InformationModel && self.show_left_sidebar {
                    return self.update(Message::InfoPaletteNext);
                }
            }
            Message::ConceptPickerPrevious => {
                if self.is_concept_picker_open {
                    if let Some(cid) = self.selected_info_class_id {
                        let count = self.filtered_available_concepts_for_class(cid).len();
                        if count > 0 {
                            self.concept_picker_index = if self.concept_picker_index == 0 {
                                count - 1
                            } else {
                                self.concept_picker_index - 1
                            };
                        }
                    }
                } else if self.active_tab == Tab::ConceptModel && self.show_left_sidebar {
                    return self.update(Message::ConceptPalettePrevious);
                } else if self.active_tab == Tab::InformationModel && self.show_left_sidebar {
                    return self.update(Message::InfoPalettePrevious);
                }
            }
            Message::ConceptPickerSubmit => {
                if let Some(cid) = self.selected_info_class_id {
                    let filtered = self.filtered_available_concepts_for_class(cid);
                    if let Some(opt) = filtered.get(self.concept_picker_index).cloned() {
                        self.is_concept_picker_open = false;
                        self.concept_picker_search.clear();
                        self.concept_picker_index = 0;
                        return self.update(Message::AddConceptToInformationClass(cid, opt));
                    }
                }
            }
            Message::AddConceptToInformationClass(class_id, opt) => {
                self.is_concept_picker_open = false;
                self.concept_picker_search.clear();
                self.concept_picker_index = 0;
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.add_concept_id(opt.id);
                    let _cls = class.clone();
                    self.trigger_autosave();
                }
                if self.selected_info_class_id == Some(class_id) {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus.update_targets(targets);
                }
            }
            Message::RemoveConceptFromInformationClass(class_id, concept_id) => {
                if let Some(class) = self.project.information_model_mut().get_class_mut(class_id) {
                    class.remove_concept_id(concept_id);
                    let _cls = class.clone();
                    self.trigger_autosave();
                }
                if self.selected_info_class_id == Some(class_id) {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus.update_targets(targets);
                }
            }
            Message::DeleteInformationClass(class_id) => {
                self.project.remove_information_class(class_id);
                if self.selected_info_class_id == Some(class_id) {
                    self.selected_info_class_id = None;
                    self.info_inspector_focus.reset();
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                        self.selected_info_graph_node_ids.remove(&nid);
                    }
                }
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
                    self.trigger_autosave();
                }
                if self.selected_info_class_id == Some(class_id) {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus.update_targets(targets);
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
                        let _cls = class.clone();
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
                if let Some(_cls) = cls_opt {
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
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
                if let Some(_cls) = cls_opt {
                    self.project.sync_information_graph();
                    self.reroute_info_graph_edges();
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
                        let _cls = class.clone();
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
                        let _cls = class.clone();
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
                        let _cls = class.clone();
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
                        let _cls = class.clone();
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
                    self.trigger_autosave();
                }
                if self.selected_info_class_id == Some(class_id) {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus.update_targets(targets);
                }
            }
            Message::InformationClassSearchChanged(q) => {
                self.info_class_search = q;
                self.info_palette_index = 0;
            }
            Message::InfoPaletteNext => {
                let count = self.filtered_palette_classes().len();
                if count > 0 {
                    self.info_palette_index = (self.info_palette_index + 1) % count;
                }
            }
            Message::InfoPalettePrevious => {
                let count = self.filtered_palette_classes().len();
                if count > 0 {
                    self.info_palette_index = if self.info_palette_index == 0 {
                        count - 1
                    } else {
                        self.info_palette_index - 1
                    };
                }
            }
            Message::InfoPaletteSubmit => {
                let filtered = self.filtered_palette_classes();
                if let Some(cls) = filtered.get(self.info_palette_index).copied() {
                    let class_id = cls.id();
                    if let Some(node) = self
                        .project
                        .information_graph()
                        .find_node_by_class(class_id)
                    {
                        let nid = node.id();
                        let nx = node.x();
                        let ny = node.y();
                        let nw = node.width();
                        let nh = node.height();
                        self.set_selected_info_graph_node(Some(nid));
                        self.selected_info_class_id = Some(class_id);
                        self.selected_info_enum_id = None;
                        let center = self.canvas_visible_center();
                        self.info_canvas_viewport
                            .center_on_rect(nx, ny, nw, nh, center);
                        return operation::focus(iced::widget::Id::unique());
                    } else {
                        let center_world = self
                            .info_canvas_viewport
                            .to_world(self.canvas_visible_center());
                        let raw_x = center_world.x
                            - crate::features::concept_model::DEFAULT_NODE_WIDTH / 2.0;
                        let raw_y = center_world.y
                            - crate::features::concept_model::DEFAULT_NODE_HEIGHT / 2.0;
                        let _ = self.update(Message::AddClassToDiagramAt(class_id, raw_x, raw_y));
                        return operation::focus(iced::widget::Id::unique());
                    }
                }
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
                self.set_selected_info_graph_node(None);
                self.selected_info_edge = None;
                let targets = self.build_info_inspector_targets();
                self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                self.trigger_autosave();
                return operation::focus("info_enum_name_input");
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
                self.set_selected_info_graph_node(Some(node_id));
                self.selected_info_edge = None;
                let targets = self.build_info_inspector_targets();
                self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                self.trigger_autosave();
                return operation::focus("info_enum_name_input");
            }
            Message::SelectInformationEnumeration(id) => {
                self.selected_info_enum_id = id;
                if id.is_some() {
                    self.selected_info_class_id = None;
                    self.selected_info_edge = None;
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                    let node_id = id.and_then(|eid| {
                        self.project
                            .information_graph()
                            .find_node_by_class(eid)
                            .map(|n| n.id())
                    });
                    self.set_selected_info_graph_node(node_id);
                } else {
                    self.info_inspector_focus.reset();
                    self.set_selected_info_graph_node(None);
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
                if let Some(_e) = e_opt {
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
                if let Some(_e) = e_opt {
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
                    if let (Some(val_count), Some(_e)) = (val_count, e_clone) {
                        self.project
                            .information_graph_mut()
                            .update_class_dimensions(enum_id, val_count);
                        self.new_enum_value_input.clear();
                        self.trigger_autosave();
                        if self.selected_info_enum_id == Some(enum_id) {
                            let targets = self.build_info_inspector_targets();
                            self.info_inspector_focus.update_targets(targets);
                        }
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
                if let (Some(val_count), Some(_e)) = (val_count, e_clone) {
                    self.project
                        .information_graph_mut()
                        .update_class_dimensions(enum_id, val_count);
                    self.trigger_autosave();
                    if self.selected_info_enum_id == Some(enum_id) {
                        let targets = self.build_info_inspector_targets();
                        self.info_inspector_focus.update_targets(targets);
                    }
                }
            }
            Message::DeleteInformationEnumeration(enum_id) => {
                self.project.remove_information_enumeration(enum_id);
                if self.selected_info_enum_id == Some(enum_id) {
                    self.selected_info_enum_id = None;
                    self.info_inspector_focus.reset();
                }
                if let Some(nid) = self.selected_info_graph_node_id {
                    if self.project.information_graph().find_node(nid).is_none() {
                        self.selected_info_graph_node_id = None;
                        self.selected_info_graph_node_ids.remove(&nid);
                    }
                }
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
                self.selected_info_class_id = None;
                self.set_selected_info_graph_node(Some(node_id));
                self.selected_info_edge = None;
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
                    self.selected_info_class_id = None;
                    self.set_selected_info_graph_node(Some(node_id));
                    self.selected_info_edge = None;
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
                    self.selected_info_graph_node_ids.remove(&node_id);
                }
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
                self.selected_info_enum_id = None;
                self.set_selected_info_graph_node(Some(node_id));
                self.selected_info_edge = None;
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
                    self.selected_info_enum_id = None;
                    self.set_selected_info_graph_node(Some(node_id));
                    self.selected_info_edge = None;
                    self.trigger_autosave();
                }
            }
            Message::RemoveClassFromDiagram(node_id) => {
                self.project.information_graph_mut().remove_node(node_id);
                if self.selected_info_graph_node_id == Some(node_id) {
                    self.selected_info_graph_node_id = None;
                }
                self.selected_info_graph_node_ids.remove(&node_id);
                self.trigger_autosave();
            }
            Message::UpdateClassNodePosition(node_id, x, y) => {
                let final_x = (x / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let final_y = (y / crate::features::concept_model::GRID_SIZE).round()
                    * crate::features::concept_model::GRID_SIZE;
                let ig = self.project.information_graph_mut();
                ig.update_node_position(node_id, final_x, final_y);

                let d_nodes: Vec<crate::features::concept_model::DiagramNode> =
                    ig.nodes().iter().map(|n| n.to_diagram_node()).collect();
                let d_edges: Vec<crate::features::concept_model::DiagramEdge> =
                    ig.edges().iter().map(|e| e.to_diagram_edge()).collect();
                let routes = crate::ui::edge_router::EdgeRouter::route_edges(&d_nodes, &d_edges);
                for r in routes {
                    ig.update_edge_ports(r.from, r.to, Some(r.from_side), Some(r.to_side));
                }
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
                if self.selected_info_class_id.is_some() || self.selected_info_enum_id.is_some() {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                } else {
                    self.info_inspector_focus.reset();
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
                if self.selected_info_class_id.is_some() || self.selected_info_enum_id.is_some() {
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                } else {
                    self.info_inspector_focus.reset();
                }
            }
            Message::InfoNodeDoubleClicked(node_id) => {
                crate::features::diagnostics::log_info(
                    "canvas",
                    &format!("Dobbeltklik på node {} – åbner egenskaber", node_id),
                );
                self.selected_info_graph_node_id = Some(node_id);
                self.selected_info_graph_node_ids = Some(node_id).into_iter().collect();
                self.selected_info_edge = None;
                if let Some(node) = self.project.information_graph().find_node(node_id) {
                    let cid = node.class_id();
                    if self
                        .project
                        .information_model()
                        .get_enumeration(cid)
                        .is_some()
                    {
                        self.selected_info_enum_id = Some(cid);
                        self.selected_info_class_id = None;
                        let targets = self.build_info_inspector_targets();
                        self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                        return operation::focus("info_enum_name_input");
                    } else {
                        self.selected_info_class_id = Some(cid);
                        self.selected_info_enum_id = None;
                        let targets = self.build_info_inspector_targets();
                        self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                        return operation::focus("info_class_name_input");
                    }
                }
                return Task::none();
            }
            Message::InfoEdgeSelected(edge) => {
                self.selected_info_edge = edge;
                if edge.is_some() {
                    self.selected_info_graph_node_id = None;
                    self.selected_info_graph_node_ids.clear();
                    self.selected_info_class_id = None;
                    self.selected_info_enum_id = None;
                    let targets = self.build_info_inspector_targets();
                    self.info_inspector_focus = FocusCycle::with_targets(targets, Some(0));
                } else {
                    self.info_inspector_focus.reset();
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
                let _label = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.label().map(String::from));
                let _src_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.source_multiplicity());
                let _tgt_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.target_multiplicity());
                let _directed = self
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
                let _kind = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .map(|e| e.kind())
                    .unwrap_or(RelationKind::Association);
                let _src_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.source_multiplicity());
                let _tgt_mult = self
                    .project
                    .information_graph()
                    .find_edge(from, to)
                    .and_then(|e| e.target_multiplicity());
                let _directed = self
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

    pub fn handle_event_for_subscription(event: Event, status: event::Status) -> Option<Message> {
        if let Event::Window(iced::window::Event::Unfocused) = event {
            return Some(Message::CanvasSpacePressed(false));
        }
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            return Some(Message::PaletteDragMoved(position));
        }
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
            return Some(Message::PaletteDragDropped);
        }

        // VIGTIGT: Tab, Shift+Tab, Escape, Enter, ArrowDown, ArrowUp og Ctrl+B skal ALTID opfanges, selv hvis et
        // tekstfelt har sat status == event::Status::Captured.
        // Dette sikrer at tastaturnavigation (Tab/Shift+Tab) og lukning af dialoger (Esc)
        // fungerer pålideligt uanset om et inputfelt aktuelt har fokus.
        // Ligeledes opfanges piletasterne ArrowDown og ArrowUp til comboboksen.
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = &event {
            match key.as_ref() {
                Key::Named(Named::Tab) => {
                    if modifiers.shift() {
                        return Some(Message::FocusPrevious);
                    } else {
                        return Some(Message::FocusNext);
                    }
                }
                Key::Named(Named::Escape) => {
                    return Some(Message::EscapePressed);
                }
                Key::Named(Named::ArrowDown) => {
                    return Some(Message::ConceptPickerNext);
                }
                Key::Named(Named::ArrowUp) => {
                    return Some(Message::ConceptPickerPrevious);
                }
                Key::Character(c)
                    if (c == "b" || c == "B") && (modifiers.control() || modifiers.command()) =>
                {
                    return Some(Message::ToggleLeftSidebar);
                }
                _ => {}
            }
        }

        if status == event::Status::Captured {
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                App::map_key_press(&key, modifiers)
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
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status, _window| {
            App::handle_event_for_subscription(event, status)
        })
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
        ]
        .align_y(Alignment::Center);

        let tab_item = |tab: Tab, label: &'static str| {
            let is_active = self.active_tab == tab;
            let tab_id = match tab {
                Tab::ConceptList => "tab_concept_list",
                Tab::ConceptModel => "tab_concept_model",
                Tab::InformationModel => "tab_information_model",
            };
            container(
                button(text(label).size(12))
                    .style(segmented_tab_button(is_active))
                    .on_press(Message::SelectTab(tab))
                    .padding([5, 12]),
            )
            .id(tab_id)
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

        let header_right = row![text(self.project.metadata().name())
            .size(12)
            .color(ThemeColors::SLATE_600),]
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
        let maybe_diagnostics_modal: Option<Element<Message>> =
            self.diagnostics_modal.as_ref().map(|modal| {
                crate::features::diagnostics::ui::view_diagnostics_modal(modal, &self.modal_focus)
            });

        let maybe_excel_schema_guide_modal: Option<Element<Message>> =
            if self.excel_schema_guide_open {
                Some(crate::features::export::ui::view_excel_schema_guide_modal(
                    &self.modal_focus,
                ))
            } else {
                None
            };

        let maybe_excel_import_preview_modal: Option<Element<Message>> =
            self.excel_import_preview.as_ref().map(|modal| {
                crate::features::export::ui::view_excel_import_preview_modal(
                    modal,
                    &self.project,
                    &self.modal_focus,
                )
            });

        let maybe_metadata_modal: Option<Element<Message>> =
            self.metadata_modal.as_ref().map(|meta_state| {
                crate::features::model::ui::view_metadata_modal(meta_state, &self.modal_focus)
            });

        let maybe_file_dialog_modal: Option<Element<Message>> = self.file_dialog_mode.map(|mode| {
            crate::ui::file_dialog::view_file_dialog_modal(
                mode,
                &self.file_dialog_input,
                &self.modal_focus,
            )
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
                crate::features::concept_model::ui::view_quick_create_modal(qc, &self.modal_focus)
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
                        color: ColorTokens::SLATE_900.with_alpha(0.12).into_color(),
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
                    concept_table::view(
                        filtered,
                        self.project.concepts().len(),
                        &self.search_query,
                        self.toolbar_focused_button(),
                    )
                }
            }

            Tab::ConceptModel => concept_model_view::view(
                self.project.concepts(),
                self.project.concept_graph(),
                self.selected_graph_node_id,
                &self.selected_graph_node_ids,
                self.selected_edge,
                &self.concept_model_search,
                self.concept_palette_index,
                self.canvas_viewport,
                self.is_space_pressed,
                self.is_inline_graph_editing,
                self.editor_state.as_ref(),
                self.relation_dialog.as_ref(),
                self.show_left_sidebar,
                self.palette_drag.as_ref().is_some_and(|d| d.is_dragging),
                self.concept_inspector_focused_button(),
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
                self.info_palette_index,
                &self.new_enum_value_input,
                self.info_canvas_viewport,
                self.is_space_pressed,
                self.info_relation_dialog.as_ref(),
                self.show_left_sidebar,
                self.palette_drag.as_ref().is_some_and(|d| d.is_dragging),
                self.info_inspector_focused_button(),
                self.is_concept_picker_open,
                &self.concept_picker_search,
                self.concept_picker_index,
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

        let status_bar = row![
            rules_button,
            text("• Klar").size(12).color(ThemeColors::SLATE_500),
            Space::new().width(12),
            text(format!("• {} begreber", self.project.concepts().len()))
                .size(12)
                .color(ThemeColors::SLATE_600),
            Space::new().width(12),
            save_widget,
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
                    background: Some(iced::Background::Color(
                        ColorTokens::SURFACE.with_alpha(0.94).into_color(),
                    )),
                    border: iced::Border {
                        color: tag_color,
                        width: 1.5,
                        radius: 6.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: ColorTokens::SLATE_900.with_alpha(0.15).into_color(),
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
        } else if let Some(modal) = maybe_excel_schema_guide_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_excel_import_preview_modal {
            stack![layered_layout, modal].into()
        } else if let Some(modal) = maybe_metadata_modal {
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
}

/// Indlæser applikationens 32x32 RGBA vinduesikon til Iced Settings
pub fn load_window_icon() -> Option<iced::window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/kant-32.rgba");
    iced::window::icon::from_rgba(ICON_BYTES.to_vec(), 32, 32).ok()
}
