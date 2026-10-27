use kant::features::concept_model::{ConceptGraph, RelationKind};
use kant::features::concepts::{BelongsToDomain, Concept, ConceptValidator};
use kant::features::information_model::{
    Attribute, InformationClass, InformationModel, Multiplicity, PrimitiveType,
};
use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
use kant::ui::app::{App, ConceptOption, Message, Tab};

#[test]
fn test_fda_project_initialization_and_metadata() {
    let metadata = ModelMetadata::new(
        "Køretøjsmodellen",
        "Kernemodel for køretøjer og registrering i Danmark",
        "https://data.gov.dk/model/core/vehicle",
        "Motorstyrelsen",
        "Transport og Trafik",
        "1.0.0",
        ModelStatus::Draft,
    );

    let project = ModelProject::new(metadata);
    assert_eq!(project.metadata().name(), "Køretøjsmodellen");
    assert_eq!(project.metadata().version(), "1.0.0");
    assert_eq!(project.metadata().status(), ModelStatus::Draft);
}

#[test]
fn test_fda_concept_validation_rules() {
    // Gyldigt begreb efter Bilag D og E
    let mut valid_concept = Concept::new(
        "Køretøj",
        "Et mobilt teknisk anlæg, der anvendes til transport af personer eller gods.",
        BelongsToDomain::Yes,
    );
    valid_concept.set_accepted_term(Some("Transportmiddel".to_string()));
    valid_concept.set_source(Some("Færdselsloven § 2, stk. 1".to_string()));
    valid_concept.set_identifier(Some(
        "https://data.gov.dk/model/core/vehicle/Koeretoej".to_string(),
    ));

    let validation = ConceptValidator::validate(&valid_concept);
    assert!(
        validation.is_ok(),
        "Gyldigt begreb skal bestå FDA validering"
    );

    // Ugyldigt begreb (mangler påkrævet definition)
    let invalid_concept = Concept::new("Ugyldigt", "", BelongsToDomain::Yes);
    let invalid_res = ConceptValidator::validate(&invalid_concept);
    assert!(
        invalid_res.is_err(),
        "Begreb uden definition skal fejle validering"
    );
}

#[test]
fn test_progression_from_concept_to_graph_and_information_model() {
    // 1. Opret begreber
    let c1 = Concept::new("Køretøj", "Transportmiddel...", BelongsToDomain::Yes);
    let c2 = Concept::new(
        "Personbil",
        "Køretøj indrettet til befordring af højst 9 personer...",
        BelongsToDomain::Yes,
    );

    // 2. Begrebsmodel (Graf med generalisering)
    let mut graph = ConceptGraph::new();
    let n1 = graph.add_node(&c1);
    let n2 = graph.add_node(&c2);
    graph.add_relation(n2, n1, RelationKind::Generalization);

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    // 3. Informationsmodel (Klasse med attributter og multipliciteter)
    let mut info_class = InformationClass::from_concept(&c2);
    info_class.add_attribute(Attribute::new(
        "registreringsnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    ));

    assert_eq!(info_class.name(), "Personbil");
    assert_eq!(info_class.attributes().len(), 1);
    assert_eq!(
        info_class.attributes()[0].multiplicity(),
        Multiplicity::new(1, Some(1))
    );
}

#[test]
fn test_ui_app_state_and_tab_switching() {
    let mut app = App::new_with_path(None);
    assert_eq!(app.active_tab(), Tab::ConceptList);

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    assert_eq!(app.active_tab(), Tab::ConceptList);
}

#[test]
fn test_fda_project_concept_crud() {
    let mut project = ModelProject::default();
    assert!(project.concepts().is_empty());

    let mut c1 = Concept::new(
        "Køretøj",
        "Et mobilt teknisk anlæg til transport af personer eller gods.",
        BelongsToDomain::Yes,
    );
    c1.set_source(Some("Færdselsloven § 2, stk. 1".to_string()));
    c1.set_legal_source(Some("LBK nr 1324 af 21/11/2023".to_string()));

    // 1. Create (Add)
    let id1 = project
        .add_concept(c1.clone())
        .expect("Gyldigt begreb skal tilføjes");
    assert_eq!(project.concepts().len(), 1);
    assert_eq!(
        project.get_concept(id1).unwrap().preferred_term(),
        "Køretøj"
    );

    // 2. Reject Invalid Concept
    let invalid = Concept::new("", "Ugyldig uden term", BelongsToDomain::Yes);
    assert!(project.add_concept(invalid).is_err());

    // 3. Update
    let mut updated = project.get_concept(id1).unwrap().clone();
    updated.set_definition("Opdateret præcis definition af køretøj.");
    project
        .update_concept(updated)
        .expect("Opdatering skal lykkes");
    assert_eq!(
        project.get_concept(id1).unwrap().definition(),
        "Opdateret præcis definition af køretøj."
    );

    // 4. Delete
    let removed = project.remove_concept(id1);
    assert!(removed.is_some());
    assert!(project.concepts().is_empty());
}

#[test]
fn test_concept_list_ui_crud_cycle() {
    use kant::ui::app::ConceptFormField;

    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::ConceptList));

    // 1. Start nyt begreb
    let _ = app.update(Message::StartNewConcept);
    assert!(app.is_editing_concept());

    // 2. Udfyld felter
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Personbil".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "Køretøj indrettet til befordring af højst 9 personer.".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::BelongsToDomain,
        "Ja".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Source,
        "Færdselsloven".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::LegalSource,
        "LBK nr 1324".to_string(),
    ));

    // 3. Gem begreb
    let _ = app.update(Message::SaveConcept);
    assert!(!app.is_editing_concept());
    assert_eq!(app.project().concepts().len(), 1);

    // Verificer at view() renderer uden fejl for tabel med begreb
    let _ = app.view();

    let id = {
        let saved = &app.project().concepts()[0];
        assert_eq!(saved.preferred_term(), "Personbil");
        assert_eq!(
            saved.definition(),
            "Køretøj indrettet til befordring af højst 9 personer."
        );
        assert_eq!(saved.belongs_to_domain(), &BelongsToDomain::Yes);
        assert_eq!(saved.source(), Some("Færdselsloven"));
        assert_eq!(saved.legal_source(), Some("LBK nr 1324"));
        saved.id()
    };

    // 4. Søgning / filtrering
    let _ = app.update(Message::SearchQueryChanged("Person".to_string()));
    assert_eq!(app.filtered_concepts().len(), 1);
    let _ = app.view();

    let _ = app.update(Message::SearchQueryChanged("Ukendt".to_string()));
    assert_eq!(app.filtered_concepts().len(), 0);
    let _ = app.view();

    let _ = app.update(Message::SearchQueryChanged("".to_string()));
    assert_eq!(app.filtered_concepts().len(), 1);

    // 5. Rediger begreb
    let _ = app.update(Message::EditConcept(id));
    assert!(app.is_editing_concept());
    let _ = app.view();
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Personbil (M1)".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);
    assert_eq!(
        app.project().concepts()[0].preferred_term(),
        "Personbil (M1)"
    );

    // 6. Slet begreb
    let _ = app.update(Message::DeleteConcept(id));
    assert!(app.project().concepts().is_empty());
}

#[test]
fn test_keyboard_navigation_and_shortcuts() {
    let mut app = App::new_with_path(None);

    // FocusNext og FocusPrevious returnerer gyldige tasks
    let _ = app.update(Message::FocusNext);
    let _ = app.update(Message::FocusPrevious);

    // Escape lukker editor
    let _ = app.update(Message::StartNewConcept);
    assert!(app.is_editing_concept());
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_editing_concept(), "Escape skal annullere editor");

    // Escape lukker også fildialog
    let _ = app.update(Message::OpenInlineFileDialog(
        kant::ui::app::FileDialogMode::Open,
    ));
    assert!(app.is_file_dialog_open());
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_file_dialog_open(), "Escape skal lukke fildialog");
}

#[test]
fn test_new_project_does_not_overwrite_disk_file() {
    use kant::features::model::storage::ProjectStorage;
    use kant::ui::app::ConceptFormField;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_edge_no_overwrite_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut app = App::new_with_path(Some(file_path.clone()));

    // Opret et begreb så filen findes på disk med data
    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    let _ = app.update(Message::StartNewConcept);
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "TestTerm".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "TestDefinition".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);

    let on_disk_before = ProjectStorage::load_from_file(&file_path).expect("Skal kunne læses");
    assert_eq!(on_disk_before.concepts().len(), 1);

    // Tryk "Nyt Projekt"
    let _ = app.update(Message::NewProject);
    assert!(app.project().concepts().is_empty());
    assert_eq!(
        app.current_file_path(),
        None,
        "Nyt projekt skal nulstille aktiv filsti"
    );

    // Verificer at filen på disken STADIG indeholder det oprindelige begreb!
    let on_disk_after =
        ProjectStorage::load_from_file(&file_path).expect("Skal stadig kunne læses");
    assert_eq!(
        on_disk_after.concepts().len(),
        1,
        "Nyt projekt må IKKE slette eksisterende fil på disk"
    );

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_project_storage_roundtrip_and_atomic_save() {
    use kant::features::model::storage::ProjectStorage;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_edge_project_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut project = ModelProject::default();
    let mut c1 = Concept::new(
        "Vej",
        "Færdselsareal for køretøjer og fodgængere.",
        BelongsToDomain::Yes,
    );
    c1.set_legal_source(Some("Vejloven § 3".to_string()));
    project.add_concept(c1).unwrap();

    // 1. Gem til fil
    let save_res = ProjectStorage::save_to_file(&project, &file_path);
    assert!(save_res.is_ok(), "Skal kunne gemme projektfil atomisk");
    assert!(file_path.exists(), "Projektfil skal eksistere på disken");

    // 2. Indlæs fra fil
    let loaded =
        ProjectStorage::load_from_file(&file_path).expect("Skal kunne indlæse gemt projektfil");
    assert_eq!(loaded.metadata().name(), project.metadata().name());
    assert_eq!(loaded.concepts().len(), 1);
    assert_eq!(loaded.concepts()[0].preferred_term(), "Vej");
    assert_eq!(loaded.concepts()[0].legal_source(), Some("Vejloven § 3"));

    // Oprydning
    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_app_autosave_lifecycle() {
    use kant::features::model::storage::ProjectStorage;
    use kant::ui::app::ConceptFormField;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_edge_autosave_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut app = App::new_with_path(Some(file_path.clone()));
    assert_eq!(app.current_file_path(), Some(&file_path));

    // 1. Opret begreb -> autosave skal opdatere filen på disken
    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    let _ = app.update(Message::StartNewConcept);
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Cykelsti".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "Færdselsareal forbeholdt cykler.".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);

    assert!(
        file_path.exists(),
        "Autosave skal have oprettet filen på disken"
    );
    let on_disk =
        ProjectStorage::load_from_file(&file_path).expect("Skal kunne læse autosaved fil");
    assert_eq!(on_disk.concepts().len(), 1);
    assert_eq!(on_disk.concepts()[0].preferred_term(), "Cykelsti");

    // 2. Slet begreb -> autosave skal genskrive filen på disken
    let id = on_disk.concepts()[0].id();
    let _ = app.update(Message::DeleteConcept(id));

    let on_disk_after_del =
        ProjectStorage::load_from_file(&file_path).expect("Skal kunne læse efter sletning");
    assert!(
        on_disk_after_del.concepts().is_empty(),
        "Autosaved fil skal have 0 begreber efter sletning"
    );

    // Oprydning
    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_concept_graph_lifecycle_and_persistence() {
    use kant::features::concept_model::RelationKind;
    use kant::features::model::storage::ProjectStorage;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_edge_graph_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut project = ModelProject::default();

    let mut c1 = Concept::new("Køretøj", "Mobilt teknisk anlæg...", BelongsToDomain::Yes);
    c1.set_legal_source(Some("Færdselsloven § 2".to_string()));
    let id1 = project.add_concept(c1).unwrap();

    let c2 = Concept::new(
        "Personbil",
        "Køretøj til højst 9 personer...",
        BelongsToDomain::Yes,
    );
    let id2 = project.add_concept(c2).unwrap();

    let c3 = Concept::new("Person", "CPR-registreret person...", BelongsToDomain::No);
    let id3 = project.add_concept(c3).unwrap();

    // 1. Synkroniser projektbegreber med concept_graph
    project.populate_missing_concepts_to_graph();
    let graph = project.concept_graph();
    assert_eq!(graph.node_count(), 3, "Skal have 3 noder i grafen");

    let node1 = graph
        .find_node_by_concept(id1)
        .expect("Skal finde node for Køretøj");
    let node2 = graph
        .find_node_by_concept(id2)
        .expect("Skal finde node for Personbil");
    let node3 = graph
        .find_node_by_concept(id3)
        .expect("Skal finde node for Person");

    assert_eq!(node1.label(), "Køretøj");
    assert!(node1.is_local(), "Køretøj er lokalt begreb (FDA sand)");
    assert!(!node3.is_local(), "Person er fremmed begreb (FDA blå)");

    let n1_id = node1.id();
    let n2_id = node2.id();
    let n3_id = node3.id();

    // 2. Opret UML relationer: Generalisering (Personbil -> Køretøj) og Association (Person -> Personbil)
    project
        .concept_graph_mut()
        .add_relation(n2_id, n1_id, RelationKind::Generalization);
    project.concept_graph_mut().add_relation_with_label(
        n3_id,
        n2_id,
        RelationKind::Association,
        Some("ejer".to_string()),
    );
    assert_eq!(project.concept_graph().edge_count(), 2);

    // 3. Flyt node position (bruger trækker node på lærredet)
    project
        .concept_graph_mut()
        .update_node_position(n1_id, 320.0, 140.0);
    let moved_node = project.concept_graph().find_node(n1_id).unwrap();
    assert_eq!(moved_node.x(), 320.0);
    assert_eq!(moved_node.y(), 140.0);

    // 4. Persistens roundtrip: Gem til .edge.json og indlæs igen
    ProjectStorage::save_to_file(&project, &file_path).expect("Skal kunne gemme projekt med graf");
    let loaded =
        ProjectStorage::load_from_file(&file_path).expect("Skal kunne indlæse projekt med graf");

    assert_eq!(loaded.concepts().len(), 3);
    assert_eq!(loaded.concept_graph().node_count(), 3);
    assert_eq!(loaded.concept_graph().edge_count(), 2);

    let loaded_n1 = loaded
        .concept_graph()
        .find_node(n1_id)
        .expect("Node1 skal findes efter indlæsning");
    assert_eq!(loaded_n1.x(), 320.0);
    assert_eq!(loaded_n1.y(), 140.0);

    // 5. Fail-Closed kaskadesletning: Sletning af Køretøj skal fjerne noden OG generaliserings-relationen
    let mut project_mut = loaded;
    project_mut.remove_concept(id1);
    assert_eq!(project_mut.concept_graph().node_count(), 2);
    assert_eq!(
        project_mut.concept_graph().edge_count(),
        1,
        "Generaliseringen til Køretøj skal automatisk være slettet (ingen dangling edges)"
    );

    // 6. Test App TEA graf-beskeder
    let mut app = App::new_with_path(Some(file_path.clone()));
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphNodeSelected(Some(n2_id)));
    assert_eq!(app.selected_graph_node_id(), Some(n2_id));

    let _ = app.update(Message::GraphNodeMoved(n2_id, 500.0, 300.0));
    let app_n2 = app.project().concept_graph().find_node(n2_id).unwrap();
    assert_eq!(app_n2.x(), 500.0);
    assert_eq!(app_n2.y(), 300.0);

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_ui_theme_tokens_and_widget_styles() {
    use iced::Theme;
    use kant::ui::theme::{
        card_container_style, modal_backdrop_style, modal_card_style, modern_input_style,
        pill_container_style, primary_button_style, secondary_button_style, ThemeColors,
    };

    let theme = Theme::Light;

    // 1. Verificer at de nye tokens er tilgængelige
    assert_ne!(ThemeColors::SURFACE_CARD, ThemeColors::SURFACE_BG);
    assert_ne!(ThemeColors::SLATE_50, ThemeColors::SLATE_900);
    assert_ne!(ThemeColors::PRIMARY_HOVER, ThemeColors::PRIMARY_ACTIVE);

    // 2. Verificer container styles
    let card = card_container_style(&theme);
    assert!(card.background.is_some());
    assert_eq!(card.border.radius, 12.0.into());

    let pill = pill_container_style(&theme);
    assert!(pill.background.is_some());
    assert_eq!(pill.border.radius, 18.0.into());

    let modal_bd = modal_backdrop_style(&theme);
    assert!(modal_bd.background.is_some());

    let modal_card = modal_card_style(&theme);
    assert!(modal_card.background.is_some());
    assert_eq!(modal_card.border.radius, 16.0.into());

    // 3. Verificer knap styles (Active & Hovered)
    let btn_prim_active = primary_button_style(&theme, iced::widget::button::Status::Active);
    assert!(btn_prim_active.background.is_some());
    let btn_prim_hover = primary_button_style(&theme, iced::widget::button::Status::Hovered);
    assert_ne!(btn_prim_active.background, btn_prim_hover.background);

    let btn_sec_active = secondary_button_style(&theme, iced::widget::button::Status::Active);
    assert!(btn_sec_active.background.is_some());

    // 4. Verificer input style
    let input_active = modern_input_style(&theme, iced::widget::text_input::Status::Active);
    let input_focused = modern_input_style(
        &theme,
        iced::widget::text_input::Status::Focused { is_hovered: false },
    );
    assert_ne!(input_active.border.color, input_focused.border.color);
}

#[test]
fn test_app_modal_overlay_rendering() {
    let mut app = App::new_with_path(None);

    // 1. Åbn fildialog og verificer at modal view renderes uden panic
    let _ = app.update(Message::OpenInlineFileDialog(
        kant::ui::app::FileDialogMode::SaveAs,
    ));
    assert!(app.is_file_dialog_open());
    let _ = app.view();

    // 2. Escape lukker modal fildialog
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_file_dialog_open());
    let _ = app.view();

    // 3. Åbn graf relationsdialog og verificer at modal view renderes uden panic
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphOpenRelationDialog);
    assert!(app.is_relation_dialog_open());
    let _ = app.view();

    // 4. Escape lukker modal relationsdialog
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_relation_dialog_open());
    let _ = app.view();
}

#[test]
fn test_canvas_direct_concept_creation_and_node_editing() {
    use kant::features::concept_model::NodeId;
    use kant::features::model::storage::ProjectStorage;
    use kant::ui::app::ConceptFormField;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_edge_canvas_crud_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut app = App::new_with_path(Some(file_path.clone()));

    // 1. Skift til Begrebsmodel (Graf) fanen
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);

    // 2. Dobbeltklik på tomt canvas (x: 450.0, y: 250.0) opretter begreb og åbner Egenskaber
    let _ = app.update(Message::CanvasDoubleClicked(450.0, 250.0));
    assert!(
        app.selected_graph_node_id().is_some(),
        "Dobbeltklik på tomt canvas skal oprette og vælge begrebsnode"
    );
    assert!(
        app.is_node_editing(),
        "Egenskabs-panelet skal åbnes i hurtigredigering"
    );

    // Verificer at view() renderer uden fejl
    let _ = app.view();

    // 3. Validering via ConceptValidator: Ugyldigt begreb (tom term) afvises ved SaveConcept
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);
    assert!(
        app.is_node_editing(),
        "Editor forbliver åben ved valideringsfejl"
    );

    // Udfyld gyldig term og FDA definition
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Godsvogn".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "Jernbanekøretøj indrettet til transport af gods.".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);

    // 4. Oprettelse lykkes: begreb opdateres og editor lukkes
    assert!(
        !app.is_node_editing(),
        "Editor skal lukke efter succesfuld gemning"
    );
    assert_eq!(app.project().concepts().len(), 1);

    let concept = &app.project().concepts()[0];
    assert_eq!(concept.preferred_term(), "Godsvogn");
    assert_eq!(
        concept.definition(),
        "Jernbanekøretøj indrettet til transport af gods."
    );

    let node = app
        .project()
        .concept_graph()
        .find_node_by_concept(concept.id())
        .expect("Grafen skal indeholde en node for det nye begreb");
    assert_eq!(node.label(), "Godsvogn");
    assert_eq!(
        node.x(),
        450.0,
        "Noden skal placeres præcist på klikkets x-koordinat"
    );
    assert_eq!(
        node.y(),
        250.0,
        "Noden skal placeres præcist på klikkets y-koordinat"
    );
    let node_id: NodeId = node.id();
    assert_eq!(
        app.selected_graph_node_id(),
        Some(node_id),
        "Den nyoprettede grafnode skal automatisk markeres"
    );

    // Verificer autosave: projektfilen på disken indeholder det nye begreb og noden
    let on_disk = ProjectStorage::load_from_file(&file_path)
        .expect("Projektfil skal være autosaved efter lynoprettelse på canvas");
    assert_eq!(on_disk.concepts().len(), 1);
    assert_eq!(on_disk.concept_graph().node_count(), 1);

    // 5. Dobbeltklik på eksisterende grafnode åbner hurtigredigering uden at forlade canvas-fanen
    let _ = app.update(Message::GraphNodeDoubleClicked(node_id));
    assert_eq!(
        app.active_tab(),
        Tab::ConceptModel,
        "Dobbeltklik på node må IKKE navigere væk fra ConceptModel-fanen"
    );
    assert!(
        app.is_node_editing(),
        "Hurtigredigering af nodens begreb skal være aktiv"
    );

    let _ = app.view();

    // Rediger nodens begreb direkte og gem
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Godsvogn (Jernbane)".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);

    // 6. Verificer at fanen forbliver ConceptModel, og data er opdateret i både begreb og grafnode
    assert_eq!(
        app.active_tab(),
        Tab::ConceptModel,
        "Skal forblive på ConceptModel fanen efter gem af hurtigredigering"
    );
    assert!(
        !app.is_node_editing(),
        "Hurtigredigering skal være lukket efter gem"
    );
    assert_eq!(
        app.project().concepts()[0].preferred_term(),
        "Godsvogn (Jernbane)"
    );

    let updated_node = app
        .project()
        .concept_graph()
        .find_node(node_id)
        .expect("Noden skal findes");
    assert_eq!(
        updated_node.label(),
        "Godsvogn (Jernbane)",
        "Grafnodens label skal være opdateret til den nye term"
    );

    // Verificer autosave efter hurtigredigering
    let on_disk_after_edit = ProjectStorage::load_from_file(&file_path)
        .expect("Projektfil skal være autosaved efter hurtigredigering");
    assert_eq!(
        on_disk_after_edit.concepts()[0].preferred_term(),
        "Godsvogn (Jernbane)"
    );

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_canvas_ergonomics_zoom_pan_grid() {
    use iced::Point;
    use kant::features::concept_model::{DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH, GRID_SIZE};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::graph_canvas::CanvasViewport;

    // 1. Verificer nodedimensioner og gitter-alignment
    assert_eq!(GRID_SIZE, 20.0, "Gitteret skal være 20px raster");
    assert_eq!(DEFAULT_NODE_WIDTH, 180.0, "Bredde skal være 180px (9x20)");
    assert_eq!(DEFAULT_NODE_HEIGHT, 80.0, "Højde skal være 80px (4x20)");
    assert_eq!(
        DEFAULT_NODE_WIDTH % GRID_SIZE,
        0.0,
        "Bredde skal være multiplum af gitter"
    );
    assert_eq!(
        DEFAULT_NODE_HEIGHT % GRID_SIZE,
        0.0,
        "Højde skal være multiplum af gitter"
    );

    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // Opret et begreb i projektet og tilføj til diagram
    let c = Concept::new("Vogn", "Rullende materiel", BelongsToDomain::Yes);
    let cid = app.project_mut().add_concept(c).unwrap();
    let _ = app.update(Message::AddConceptToDiagram(cid));

    let node = &app.project().concept_graph().nodes()[0];
    let node_id = node.id();
    assert_eq!(node.width(), 180.0, "Nodebredde skal være 180px");
    assert_eq!(node.height(), 80.0, "Nodehøjde skal være 80px");

    // 2. Test CanvasViewport transformation og zoom-grænser
    let mut viewport = CanvasViewport::default();
    assert_eq!(viewport.zoom(), 1.0, "Default zoom skal være 1.0 (100%)");
    assert_eq!(viewport.pan().x, 0.0);
    assert_eq!(viewport.pan().y, 0.0);

    // Test world-to-screen og screen-to-world
    let pt = Point::new(100.0, 100.0);
    assert_eq!(viewport.to_world(pt), pt);
    assert_eq!(viewport.to_screen(pt), pt);

    // Test zoom clamping (min 0.25, max 1.50)
    viewport.set_zoom(0.10);
    assert_eq!(viewport.zoom(), 0.25, "Zoom skal klemmes til min 0.25");
    viewport.set_zoom(3.0);
    assert_eq!(viewport.zoom(), 1.50, "Zoom skal klemmes til max 1.50");

    // Test zoom_at forankring (punkt under cursor skal forblive uændret i verdenskoordinater)
    viewport.set_zoom(1.0);
    let cursor = Point::new(200.0, 200.0);
    let world_before = viewport.to_world(cursor);
    viewport.zoom_at(cursor, 1.2);
    let world_after = viewport.to_world(cursor);
    assert!((world_before.x - world_after.x).abs() < 0.001);
    assert!((world_before.y - world_after.y).abs() < 0.001);

    // 3. Test Magnetisk Snap-to-Grid i App (Always-On)
    assert!(
        app.is_snap_to_grid_enabled(),
        "Snap to grid skal være slået til som default (Always-On)"
    );

    // Flyt node til arbitrære koordinater (137.4, 91.2) - skal snappe til (140.0, 100.0)
    let _ = app.update(Message::GraphNodeMoved(node_id, 137.4, 91.2));
    let moved_node = app.project().concept_graph().find_node(node_id).unwrap();
    assert_eq!(
        moved_node.x(),
        140.0,
        "Node x skal snappe til nærmeste multiplum af 20"
    );
    assert_eq!(
        moved_node.y(),
        100.0,
        "Node y skal snappe til nærmeste multiplum af 20"
    );

    // Verificer at alle 4 hjørner rammer gitterpunkter
    assert_eq!(
        (moved_node.x() + moved_node.width()) % GRID_SIZE,
        0.0,
        "Top-højre hjørne"
    );
    assert_eq!(
        (moved_node.y() + moved_node.height()) % GRID_SIZE,
        0.0,
        "Bund-venstre hjørne"
    );
    assert_eq!(
        (moved_node.x() + moved_node.width()) % GRID_SIZE,
        0.0,
        "Bund-højre x"
    );
    assert_eq!(
        (moved_node.y() + moved_node.height()) % GRID_SIZE,
        0.0,
        "Bund-højre y"
    );

    // 4. Test Zoom-kontroller i App
    assert_eq!(app.canvas_zoom(), 1.0);
    let _ = app.update(Message::CanvasZoomIn);
    assert!(app.canvas_zoom() > 1.0);
    let _ = app.update(Message::CanvasResetView);
    assert_eq!(app.canvas_zoom(), 1.0);
}

#[test]
fn test_orthogonal_edge_routing_and_ports() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::edge_router::{EdgeRouter, PortSide};

    // Opret test-begreber og noder
    let c_super = Concept::new("Superklasse", "Overordnet begreb", BelongsToDomain::Yes);
    let c_sub1 = Concept::new("Subklasse1", "Underordnet begreb 1", BelongsToDomain::Yes);
    let c_sub2 = Concept::new("Subklasse2", "Underordnet begreb 2", BelongsToDomain::Yes);
    let c_assoc = Concept::new("Associeret", "Tilknyttet begreb", BelongsToDomain::Yes);

    // Node dimensioner er standard 180x80
    // Superklasse placeret øverst: x=200, y=40 (bottom er y=120)
    let node_super = DiagramNode::new(&c_super, 200.0, 40.0);
    // Subklasse 1 placeret nederst til venstre: x=100, y=240 (top er y=240)
    let node_sub1 = DiagramNode::new(&c_sub1, 100.0, 240.0);
    // Subklasse 2 placeret nederst til højre: x=300, y=240 (top er y=240)
    let node_sub2 = DiagramNode::new(&c_sub2, 300.0, 240.0);
    // Associeret placeret til højre for superklasse: x=500, y=40
    let node_assoc = DiagramNode::new(&c_assoc, 500.0, 40.0);

    let nodes = vec![
        node_super.clone(),
        node_sub1.clone(),
        node_sub2.clone(),
        node_assoc.clone(),
    ];

    // Edge 1: Generalisering sub1 -> super
    let mut edge_gen1 = DiagramEdge::new(
        node_sub1.id(),
        node_super.id(),
        RelationKind::Generalization,
    );
    edge_gen1.set_label(Some("er en".to_string())); // skal undertrykkes jf FDA

    // Edge 2: Generalisering sub2 -> super
    let edge_gen2 = DiagramEdge::new(
        node_sub2.id(),
        node_super.id(),
        RelationKind::Generalization,
    );

    // Edge 3: Association super -> assoc
    let mut edge_asc =
        DiagramEdge::new(node_super.id(), node_assoc.id(), RelationKind::Association);
    edge_asc.set_label(Some("relaterer".to_string()));

    let edges = vec![edge_gen1.clone(), edge_gen2.clone(), edge_asc.clone()];

    let routes = EdgeRouter::route_edges(&nodes, &edges);
    assert_eq!(routes.len(), 3, "Skal route alle 3 edges");

    // 1. Verificér ortogonalitet (kun 90 graders vinkler: alle segmenter er enten rent horisontale eller vertikale)
    for route in &routes {
        assert!(
            route.points.len() >= 2,
            "En rute skal have mindst 2 punkter"
        );
        for window in route.points.windows(2) {
            let p1 = window[0];
            let p2 = window[1];
            let is_horizontal = (p1.y - p2.y).abs() < 0.001;
            let is_vertical = (p1.x - p2.x).abs() < 0.001;
            assert!(
                is_horizontal || is_vertical,
                "Alle linjesegmenter skal være strengt ortogonale (90°). Segment fra {:?} til {:?}",
                p1,
                p2
            );
        }
    }

    // 2. Verificér FDA label-semantik: ingen label på generalisering, label bevares på association
    let r_gen1 = routes
        .iter()
        .find(|r| r.from == edge_gen1.from() && r.to == edge_gen1.to())
        .unwrap();
    assert_eq!(
        r_gen1.label, None,
        "Generalisering må IKKE vise label jf FDA vejledning linje 1474 & 1526"
    );

    let r_asc = routes
        .iter()
        .find(|r| r.from == edge_asc.from() && r.to == edge_asc.to())
        .unwrap();
    assert_eq!(
        r_asc.label.as_deref(),
        Some("relaterer"),
        "Association skal bevare sin label"
    );

    // 3. Verificér pilehoved: forankret præcist på målnodens kant
    let arrow1 = r_gen1
        .arrow_head
        .as_ref()
        .expect("Generalisering skal have pilehoved");
    assert_eq!(
        arrow1.direction,
        PortSide::Bottom,
        "Pil til superklasse oppefra skal ramme bundporten"
    );
    assert_eq!(
        arrow1.tip.y,
        node_super.y() + node_super.height(),
        "Pilehovedets spids skal røre målnodens bundkant præcist (y = 120.0)"
    );
    assert!(
        arrow1.left.y > arrow1.tip.y && arrow1.right.y > arrow1.tip.y,
        "Pilehovedets trekantsbase skal ligge udenfor noden (større y), ikke skjules inde i noden"
    );

    // 4. Verificér multi-relation af samme type deler anker på target
    let r_gen2 = routes
        .iter()
        .find(|r| r.from == edge_gen2.from() && r.to == edge_gen2.to())
        .unwrap();
    let arrow2 = r_gen2
        .arrow_head
        .as_ref()
        .expect("Generalisering 2 skal have pilehoved");
    assert_eq!(
        arrow1.tip, arrow2.tip,
        "To generaliseringer til samme superklasse på samme side skal dele ankerpunkt (FDA Fig 7.1)"
    );

    // 5. Test Nærhedshåndtering (Proximity Port Shift):
    // Når to noder er så tæt på hinanden at afstanden er mindre end D_min (36px),
    // må pilen IKKE routes direkte mellem modstående flader så pilen klemmes.
    let close_sub = DiagramNode::new(&c_sub1, 200.0, 130.0); // y=130, super bottom=120 -> afstand kun 10px!
    let close_nodes = vec![node_super.clone(), close_sub.clone()];
    let close_edge = DiagramEdge::new(
        close_sub.id(),
        node_super.id(),
        RelationKind::Generalization,
    );
    let close_routes = EdgeRouter::route_edges(&close_nodes, &[close_edge]);
    let close_route = &close_routes[0];
    let close_arrow = close_route
        .arrow_head
        .as_ref()
        .expect("Skal have pilehoved");
    // Da afstanden vertikalt kun er 10px, skal porten skifte til side-port for at undgå flad/inverteret pil
    assert_ne!(
        close_arrow.direction,
        PortSide::Bottom,
        "Ved kritisk nærhed (afstand < 36px) skal porten skifte til side-porte for at bevare pilerum"
    );

    // 6. Test Krydsningsbroer (Bridge hops):
    // Opret to edges der krydser hinanden ortogonalt i et X-kryds
    let n_horiz_left = DiagramNode::new(&c_sub1, 0.0, 300.0);
    let n_horiz_right = DiagramNode::new(&c_sub2, 400.0, 300.0);
    let n_vert_top = DiagramNode::new(&c_super, 200.0, 100.0);
    let n_vert_bottom = DiagramNode::new(&c_assoc, 200.0, 500.0);

    let cross_nodes = vec![
        n_horiz_left.clone(),
        n_horiz_right.clone(),
        n_vert_top.clone(),
        n_vert_bottom.clone(),
    ];
    let edge_h = DiagramEdge::new(
        n_horiz_left.id(),
        n_horiz_right.id(),
        RelationKind::Association,
    );
    let edge_v = DiagramEdge::new(
        n_vert_top.id(),
        n_vert_bottom.id(),
        RelationKind::Association,
    );

    let cross_routes = EdgeRouter::route_edges(&cross_nodes, &[edge_h, edge_v]);
    let has_bridge = cross_routes.iter().any(|r| !r.bridges.is_empty());
    assert!(
        has_bridge,
        "Når to ortogonale linjer krydser, skal der detekteres mindst én krydsningsbro"
    );
}

#[test]
fn test_task_010_information_model_classes_attributes_and_concept_traceability() {
    // 1. Opret kildebegreber i begrebsmodellen
    let c_person = Concept::new(
        "Person",
        "En fysisk person i det danske samfund.",
        BelongsToDomain::Yes,
    );
    let c_cpr = Concept::new(
        "CprNummer",
        "Et 10-cifret unikt personnummer udstedt af CPR-registret.",
        BelongsToDomain::Yes,
    );

    // 2. Opret en Klasse knyttet til begrebet Person
    let mut person_class = InformationClass::from_concept(&c_person);
    assert_eq!(person_class.name(), "Person");
    assert_eq!(
        person_class.description(),
        Some("En fysisk person i det danske samfund.")
    );
    assert!(
        person_class.concept_ids().contains(&c_person.id()),
        "Klassen skal spore tilbage til person-begrebet"
    );

    // 3. Tilføj attribut med direkte begrebssporing (cprNummer -> c_cpr)
    let attr_cpr = Attribute::new(
        "cprNummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    )
    .with_concepts(vec![c_cpr.id()]);

    assert_eq!(attr_cpr.name(), "cprNummer");
    assert_eq!(attr_cpr.data_type(), PrimitiveType::CharacterString);
    assert_eq!(attr_cpr.multiplicity(), Multiplicity::exactly_one());
    assert!(
        attr_cpr.concept_ids().contains(&c_cpr.id()),
        "Attributten skal spore til CprNummer-begrebet"
    );
    person_class.add_attribute(attr_cpr);

    // 4. Tilføj en selvstændig attribut uden begreb (f.eks. registreringsTidspunkt)
    let attr_tid = Attribute::new(
        "registreringsTidspunkt",
        PrimitiveType::DateTime,
        Multiplicity::zero_or_one(),
    );
    assert!(
        attr_tid.concept_ids().is_empty(),
        "Selvstændig attribut har ingen begrebsrelation"
    );
    person_class.add_attribute(attr_tid);

    // 5. Opret en selvstændig teknisk klasse uden begrebsrelation
    let mut audit_class = InformationClass::new("AuditLog");
    audit_class.set_description(Some("Teknisk hændelseslog".to_string()));
    assert!(
        audit_class.concept_ids().is_empty(),
        "Selvstændig klasse skal kunne oprettes uden begrebsrelation"
    );

    // 6. Test InformationModel container
    let mut info_model = InformationModel::new();
    let person_class_id = info_model.add_class(person_class);
    let audit_class_id = info_model.add_class(audit_class);

    assert_eq!(
        info_model.classes().len(),
        2,
        "Informationsmodellen skal indeholde to klasser"
    );

    let retrieved = info_model
        .get_class(person_class_id)
        .expect("Klassen skal kunne hentes via id");
    assert_eq!(retrieved.name(), "Person");
    assert_eq!(retrieved.attributes().len(), 2);

    // Test opslag via begreb
    let classes_for_p = info_model.classes_for_concept(c_person.id());
    assert_eq!(classes_for_p.len(), 1);
    assert_eq!(classes_for_p[0].id(), person_class_id);

    let attrs_for_cpr = info_model.attributes_for_concept(c_cpr.id());
    assert_eq!(attrs_for_cpr.len(), 1);
    assert_eq!(attrs_for_cpr[0].1.name(), "cprNummer");

    // 7. Test integration i ModelProject og Disk-Persistens (Round-trip serialisering)
    let mut project = ModelProject::default();
    project.information_model_mut().add_class(
        info_model
            .get_class(person_class_id)
            .cloned()
            .expect("Skal have klasse"),
    );
    project.information_model_mut().add_class(
        info_model
            .get_class(audit_class_id)
            .cloned()
            .expect("Skal have klasse"),
    );

    let json = serde_json::to_string_pretty(&project).expect("Serialisering skal lykkes");
    let loaded: ModelProject =
        serde_json::from_str(&json).expect("Deserialisering skal genskabe ModelProject");

    assert_eq!(
        loaded.information_model().classes().len(),
        2,
        "De serialiserede klasser skal bevares intakt"
    );

    // 8. Test Bagudkompatibilitet: Deserialisering af projekt JSON uden 'information_model' felt
    let legacy_json = r#"{
        "metadata": {
            "name": "Legacy Model",
            "description": "Uden informationsmodel",
            "uri": "https://data.gov.dk/model/core/legacy",
            "responsible_org": "Myndighed",
            "domain_area": "Test",
            "version": "1.0.0",
            "status": "Draft",
            "legal_source": null
        },
        "concepts": [],
        "concept_graph": {
            "nodes": [],
            "edges": []
        }
    }"#;

    let legacy_project: ModelProject = serde_json::from_str(legacy_json).expect(
        "Bagudkompatibilitet skal sikre at legacy JSON uden information_model kan indlæses",
    );
    assert_eq!(
        legacy_project.information_model().classes().len(),
        0,
        "Legacy projekt skal initialisere en tom InformationModel via #[serde(default)]"
    );
}

#[test]
fn test_information_model_ui_crud_and_concept_linking() {
    let mut app = App::new_with_path(None);

    // 1. Skift til Informationsmodel-fanen
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    // 2. Opret et begreb i projektet til sporing
    let c = Concept::new("Borger", "Fysisk person.", BelongsToDomain::Yes);
    let c_id = app.project_mut().add_concept(c).unwrap();

    // 3. Opret en ny klasse via UI
    let _ = app.update(Message::CreateInformationClass);
    assert_eq!(
        app.project().information_model().classes().len(),
        1,
        "Skal have oprettet 1 klasse"
    );

    let class_id = app.project().information_model().classes()[0].id();

    // 4. Opdater klassens navn og beskrivelse
    let _ = app.update(Message::UpdateInformationClassName(
        class_id,
        "BorgerKlasse".to_string(),
    ));
    let _ = app.update(Message::UpdateInformationClassDescription(
        class_id,
        "En klasse for borgere".to_string(),
    ));

    // 5. Knyt begreb til klassen
    let _ = app.update(Message::AddConceptToInformationClass(
        class_id,
        ConceptOption {
            id: c_id,
            term: "Borger".to_string(),
        },
    ));

    let class = app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap();
    assert_eq!(class.name(), "BorgerKlasse");
    assert_eq!(class.description(), Some("En klasse for borgere"));
    assert!(class.concept_ids().contains(&c_id));

    // 6. Tilføj attribut
    let _ = app.update(Message::AddAttributeToClass(class_id));
    let class = app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap();
    assert_eq!(class.attributes().len(), 1);
    let attr_id = class.attributes()[0].id();

    // 7. Opdater attribut oplysninger
    let _ = app.update(Message::UpdateAttributeName(
        class_id,
        attr_id,
        "cprNummer".to_string(),
    ));
    let _ = app.update(Message::UpdateAttributeType(
        class_id,
        attr_id,
        PrimitiveType::CharacterString,
    ));
    let _ = app.update(Message::UpdateAttributeMultiplicity(
        class_id,
        attr_id,
        Multiplicity::exactly_one(),
    ));

    // 8. Knyt begreb til attribut
    let _ = app.update(Message::AddConceptToAttribute(
        class_id,
        attr_id,
        ConceptOption {
            id: c_id,
            term: "Borger".to_string(),
        },
    ));

    let class = app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap();
    let attr = &class.attributes()[0];
    assert_eq!(attr.name(), "cprNummer");
    assert_eq!(attr.data_type(), PrimitiveType::CharacterString);
    assert_eq!(attr.multiplicity(), Multiplicity::exactly_one());
    assert!(attr.concept_ids().contains(&c_id));

    // 9. Verificer at view renderer uden panik
    {
        let _view = app.view();
    }

    // 10. Slet attribut
    let _ = app.update(Message::DeleteAttribute(class_id, attr_id));
    let class = app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap();
    assert_eq!(class.attributes().len(), 0);

    // 11. Slet klasse
    let _ = app.update(Message::DeleteInformationClass(class_id));
    assert_eq!(app.project().information_model().classes().len(), 0);
}

#[test]
fn test_information_model_uml_canvas_and_studio_layout() {
    let mut app = App::new_with_path(None);

    // 1. Skift til Informationsmodel fanen
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    // 2. Opret to klasser i projektet
    let _ = app.update(Message::CreateInformationClass);
    let class_a_id = app.project().information_model().classes()[0].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_a_id,
        "Køretøj".to_string(),
    ));

    let _ = app.update(Message::CreateInformationClass);
    let class_b_id = app.project().information_model().classes()[1].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_b_id,
        "Personbil".to_string(),
    ));

    // 3. Tilføj klasser til diagrammet (ClassGraph)
    let _ = app.update(Message::AddClassToDiagram(class_a_id));
    let _ = app.update(Message::AddClassToDiagram(class_b_id));

    let (node_a_id, node_b_id, initial_height_a) = {
        let graph = app.project().information_graph();
        assert_eq!(graph.node_count(), 2, "Begge klasser skal være på canvas");
        let node_a = graph
            .find_node_by_class(class_a_id)
            .expect("Node A skal findes");
        let node_b = graph
            .find_node_by_class(class_b_id)
            .expect("Node B skal findes");
        let height = node_a.height();
        assert!(height >= 80.0, "UML node skal have en minimumshøjde");
        (node_a.id(), node_b.id(), height)
    };

    // 4. Tilføj attributter til Klasse A og verificer at nodens højde vokser dynamisk
    let _ = app.update(Message::AddAttributeToClass(class_a_id));
    let attr_id = app
        .project()
        .information_model()
        .get_class(class_a_id)
        .unwrap()
        .attributes()[0]
        .id();
    let _ = app.update(Message::UpdateAttributeName(
        class_a_id,
        attr_id,
        "registreringsNummer".to_string(),
    ));

    let updated_height_a = app
        .project()
        .information_graph()
        .find_node_by_class(class_a_id)
        .unwrap()
        .height();
    assert!(
        updated_height_a > initial_height_a,
        "UML node højde skal vokse dynamisk når attributter tilføjes"
    );

    // 5. Opret en generaliseringsrelation mellem Personbil -> Køretøj
    let _ = app.update(Message::AddClassRelation(
        node_b_id,
        node_a_id,
        RelationKind::Generalization,
        None,
    ));

    assert_eq!(
        app.project().information_graph().edge_count(),
        1,
        "Skal have oprettet 1 relation i informationsgrafen"
    );

    // 6. Flyt node på canvas og verificer position
    let _ = app.update(Message::UpdateClassNodePosition(node_b_id, 320.0, 240.0));
    let node_b_moved = app
        .project()
        .information_graph()
        .find_node(node_b_id)
        .unwrap();
    assert_eq!(node_b_moved.x(), 320.0);
    assert_eq!(node_b_moved.y(), 240.0);

    // 7. Verificer at Canvas Studio viewet renderer fejlfrit (Venstre palet, Canvas, Højre inspector)
    {
        let _view = app.view();
    }

    // 8. Slet Klasse A og verificer kaskadesletning i graf og relationer (Fail-Closed)
    let _ = app.update(Message::DeleteInformationClass(class_a_id));
    let graph = app.project().information_graph();
    assert_eq!(graph.node_count(), 1, "Node A skal være kaskadeslettet");
    assert_eq!(
        graph.edge_count(),
        0,
        "Relationer til Node A skal være kaskadeslettet uden hængende kanter"
    );
}

#[test]
fn test_task_012_unified_diagram_canvas_and_concept_studio_layout() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::{ClassDiagramEdge, ClassDiagramNode};
    use kant::ui::diagram_canvas::{CanvasEdge, CanvasNode};
    use uuid::Uuid;

    // 1. Verificer abstraktionerne for CanvasNode og CanvasEdge
    let c = Concept::new("Kunde", "En aftalepart", BelongsToDomain::Yes);
    let concept_node = DiagramNode::new(&c, 100.0, 150.0);
    assert_eq!(CanvasNode::id(&concept_node), concept_node.id());
    assert_eq!(CanvasNode::position(&concept_node), (100.0, 150.0));
    assert_eq!(CanvasNode::size(&concept_node), (180.0, 80.0));
    assert!(CanvasNode::contains(&concept_node, 110.0, 160.0));
    assert!(!CanvasNode::contains(&concept_node, 10.0, 10.0));

    let edge = DiagramEdge::new(
        concept_node.id(),
        Uuid::new_v4(),
        RelationKind::Generalization,
    );
    assert_eq!(CanvasEdge::kind(&edge), RelationKind::Generalization);

    let class_node = ClassDiagramNode::new(Uuid::new_v4(), 200.0, 250.0, 2);
    assert_eq!(CanvasNode::position(&class_node), (200.0, 250.0));
    assert!(CanvasNode::contains(&class_node, 220.0, 270.0));

    let class_edge = ClassDiagramEdge::new(
        class_node.id(),
        Uuid::new_v4(),
        RelationKind::Association,
        Some("kunde".to_string()),
    );
    assert_eq!(CanvasEdge::label(&class_edge), Some("kunde"));

    // 2. Initialiser App og test Canvas Studio workflow for Begrebsmodellen (Fane 3)
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    let c1 = Concept::new("Kunde", "En aftalepart", BelongsToDomain::Yes);
    let c2 = Concept::new("Faktura", "Et betalingskrav", BelongsToDomain::Yes);
    let c1_id = app.project_mut().add_concept(c1).unwrap();
    let c2_id = app.project_mut().add_concept(c2).unwrap();
    assert!(app.project().get_concept(c2_id).is_some());

    // Verificer is_concept_on_diagram metoden
    let node_c1_id = app
        .project()
        .concept_graph()
        .find_node_by_concept(c1_id)
        .map(|n| n.id());

    // Hvis noder er på diagrammet, test fjernelse af node uden at slette begrebet
    if let Some(n1) = node_c1_id {
        let _ = app.update(Message::RemoveConceptFromDiagram(n1));
        assert!(
            !app.project().concept_graph().is_concept_on_diagram(c1_id),
            "Noden skal være fjernet fra diagrammet"
        );
        assert!(
            app.project().get_concept(c1_id).is_some(),
            "Kernebegrebet må IKKE slettes fra projektets repository når det fjernes fra diagram"
        );
    }

    // Test tilføjelse til diagram via Message::AddConceptToDiagram
    let _ = app.update(Message::AddConceptToDiagram(c1_id));
    assert!(
        app.project().concept_graph().is_concept_on_diagram(c1_id),
        "Begrebet skal nu optræde på diagrammet"
    );

    // 3. Test søgning i Begrebsmodel Studio paletten
    let _ = app.update(Message::ConceptModelSearchChanged("fak".to_string()));

    // 4. Verificer at Canvas Studio viewet for Begrebsmodellen renderer fejlfrit (3-delt opbygning)
    {
        let _view = app.view();
    }

    // 5. Verificer at Informationsmodellen også renderer fejlfrit med DiagramCanvas
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    {
        let _view = app.view();
    }
}

#[test]
fn test_task_012_grid_resize_and_information_model_relations_inspector() {
    use iced::{Point, Rectangle, Size};
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::InformationClass;
    use kant::ui::diagram_canvas::CanvasViewport;

    // 1. Verificer at grid beregning dækker vilkårlige vinduesstørrelser uden fordoblet transformation
    let vp = CanvasViewport::new(iced::Vector::new(15.0, 25.0), 1.0);
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(2560.0, 1440.0));
    let step = 20.0 * vp.zoom();
    let ox = vp.pan().x.rem_euclid(step);
    let oy = vp.pan().y.rem_euclid(step);
    let mut last_x = ox;
    while last_x + step <= bounds.width {
        last_x += step;
    }
    let mut last_y = oy;
    while last_y + step <= bounds.height {
        last_y += step;
    }
    assert!(
        last_x >= 2540.0,
        "Grid dots skal dække hele bredden ved resize"
    );
    assert!(
        last_y >= 1420.0,
        "Grid dots skal dække hele højden ved resize"
    );

    // 2. Verificer at Informationsmodellen viser tilknyttede relationer for en valgt klasse
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    let class_a = InformationClass::new("Person");
    let class_b = InformationClass::new("OrgPerson");
    let class_a_id = class_a.id();
    let class_b_id = class_b.id();

    let _ = app.project_mut().information_model_mut().add_class(class_a);
    let _ = app.project_mut().information_model_mut().add_class(class_b);

    let node_a = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_a_id, 0);
    let node_b = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_b_id, 0);

    // Opret en generaliseringsrelation mellem OrgPerson -> Person
    let _ = app.update(Message::AddClassRelation(
        node_b,
        node_a,
        RelationKind::Generalization,
        None,
    ));

    // Vælg OrgPerson
    let _ = app.update(Message::SelectInformationClass(Some(class_b_id)));

    // Verificer at app.view() renderer fejlfrit med tilknyttede relationer i inspectoren
    {
        let _view = app.view();
    }

    // Slet relationen via DeleteClassRelation og verificer at den fjernes
    let _ = app.update(Message::DeleteClassRelation(node_b, node_a));
    assert_eq!(app.project().information_graph().edge_count(), 0);
}

#[test]
fn test_class_node_height_grows_in_grid_size_increments_and_aligns_with_grid() {
    use kant::features::concept_model::GRID_SIZE;
    use kant::features::information_model::InformationClass;

    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    let class = InformationClass::new("Person");
    let class_id = class.id();
    let _ = app.project_mut().information_model_mut().add_class(class);
    let node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_id, 0);

    // Initial node højde (0 attributter)
    let node = app
        .project()
        .information_graph()
        .find_node(node_id)
        .unwrap();
    let initial_height = node.height();
    assert_eq!(
        (initial_height % GRID_SIZE).abs(),
        0.0,
        "Initial højde skal være et multiplum af GRID_SIZE ({})",
        initial_height
    );

    // Node placeres på en grid-række (f.eks. y = 180.0)
    app.project_mut()
        .information_graph_mut()
        .update_node_position(node_id, 40.0, 180.0);
    let node = app
        .project()
        .information_graph()
        .find_node(node_id)
        .unwrap();
    let bottom_y = node.y() + node.height();
    assert_eq!(
        (bottom_y % GRID_SIZE).abs(),
        0.0,
        "Underkant skal flugte med en grid-række ({})",
        bottom_y
    );

    // Tilføj 1. attribut og verificer at højden vokser med grid-størrelsen
    let _ = app.update(Message::AddAttributeToClass(class_id));
    let node = app
        .project()
        .information_graph()
        .find_node(node_id)
        .unwrap();
    assert_eq!(
        (node.height() % GRID_SIZE).abs(),
        0.0,
        "Højde efter 1 attribut skal være et multiplum af GRID_SIZE ({})",
        node.height()
    );
    assert_eq!(
        ((node.y() + node.height()) % GRID_SIZE).abs(),
        0.0,
        "Underkant skal fortsat flugte med grid-rækker efter tilføjelse af attribut ({})",
        node.y() + node.height()
    );

    // Tilføj 2. attribut og verificer tilsvarende (Person-eksemplet fra brugeren)
    let _ = app.update(Message::AddAttributeToClass(class_id));
    let node = app
        .project()
        .information_graph()
        .find_node(node_id)
        .unwrap();
    assert_eq!(
        (node.height() % GRID_SIZE).abs(),
        0.0,
        "Højde efter 2 attributter skal være et multiplum af GRID_SIZE ({})",
        node.height()
    );
    assert_eq!(
        node.height(),
        120.0,
        "Højde for Person med 2 attributter skal være præcis 120.0 (6 * 20)"
    );
    assert_eq!(
        node.y() + node.height(),
        300.0,
        "Underkant af Person noden skal lande præcis på y=300 (15 * 20) i stedet for mellem to grid-linjer"
    );
}

#[test]
fn test_task_007_interactive_edges_drag_to_connect_and_inspector_crud() {
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));

    // 1. Opret to begreber i projektet
    let c1 = Concept::new(
        "Køretøj",
        "Transportmiddel for personer eller gods",
        BelongsToDomain::Yes,
    );
    let c2 = Concept::new(
        "Motor",
        "Drivkraftkilde for et køretøj",
        BelongsToDomain::Yes,
    );
    let id1 = c1.id();
    let id2 = c2.id();

    let _ = app.project_mut().add_concept(c1);
    let _ = app.project_mut().add_concept(c2);

    // 2. Tilføj begge begreber til begrebsdiagrammet
    let _ = app.update(Message::AddConceptToDiagram(id1));
    let _ = app.update(Message::AddConceptToDiagram(id2));

    let node1 = app
        .project()
        .concept_graph()
        .find_node_by_concept(id1)
        .unwrap()
        .id();
    let node2 = app
        .project()
        .concept_graph()
        .find_node_by_concept(id2)
        .unwrap()
        .id();

    // 3. Markér Node 1 (Køretøj)
    let _ = app.update(Message::GraphNodeSelected(Some(node1)));
    assert_eq!(app.selected_graph_node_id(), Some(node1));

    // 4. Test Invariant: Self-loop afvises (Must NOT: self-loop)
    let _ = app.update(Message::GraphEdgeCreated(node1, node1));
    assert_eq!(
        app.project().concept_graph().edge_count(),
        0,
        "Self-loop må ikke oprettes"
    );

    // 5. Drag-to-Connect: Forbind Node 1 til Node 2 -> skal automatisk oprette en Association og markere relationen
    let _ = app.update(Message::GraphEdgeCreated(node1, node2));
    assert_eq!(
        app.project().concept_graph().edge_count(),
        1,
        "En relation skal være oprettet"
    );
    let edge = &app.project().concept_graph().edges()[0];
    assert_eq!(edge.from(), node1);
    assert_eq!(edge.to(), node2);
    assert_eq!(
        edge.kind(),
        RelationKind::Association,
        "Default relationstype ved drop skal være Association"
    );

    // Relationen skal straks være markeret som et selvstændigt objekt, og nodemarkering ryddet
    assert_eq!(
        app.selected_graph_node_id(),
        None,
        "Nodemarkering skal ryddes ved kantvalg"
    );
    assert_eq!(
        app.selected_edge(),
        Some((node1, node2)),
        "Den nye relation skal være valgt"
    );

    // 6. Inspektørpanel for valgt relation: Ændre label og type
    let _ = app.update(Message::GraphUpdateEdgeLabel(
        node1,
        node2,
        "omfatter".to_string(),
    ));
    assert_eq!(
        app.project().concept_graph().edges()[0].label(),
        Some("omfatter"),
        "Associationsnavn skal opdateres i grafen"
    );

    let _ = app.update(Message::GraphUpdateEdgeKind(
        node1,
        node2,
        RelationKind::Generalization,
    ));
    assert_eq!(
        app.project().concept_graph().edges()[0].kind(),
        RelationKind::Generalization,
        "Relationstype skal kunne ændres fra Association til Generalisering"
    );

    // 7. Verify view() renderer inspektøren uden modal
    {
        let _view = app.view();
        assert!(
            !app.is_relation_dialog_open(),
            "Flimsy modal må ikke være åben"
        );
    }

    // 8. Deselect og Select på canvas
    let _ = app.update(Message::GraphEdgeSelected(None));
    assert_eq!(app.selected_edge(), None);

    let _ = app.update(Message::GraphEdgeSelected(Some((node1, node2))));
    assert_eq!(app.selected_edge(), Some((node1, node2)));

    // 9. Tastatursletning via Delete-tast
    let _ = app.update(Message::GraphDeleteSelected);
    assert_eq!(
        app.project().concept_graph().edge_count(),
        0,
        "Valgt relation skal slettes ved GraphDeleteSelected"
    );
    assert_eq!(
        app.selected_edge(),
        None,
        "Markering skal ryddes efter sletning"
    );
}

#[test]
fn test_information_model_interactive_edges_drag_to_connect_and_inspector_crud() {
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    // 1. Opret to klasser på diagrammet
    let _ = app.update(Message::CreateInformationClass);
    let class1_id = app
        .selected_info_class_id()
        .expect("Klasse 1 skal oprettes");
    let node1_id = app
        .selected_info_graph_node_id()
        .expect("Node 1 skal oprettes");

    let _ = app.update(Message::CreateInformationClass);
    let class2_id = app
        .selected_info_class_id()
        .expect("Klasse 2 skal oprettes");
    let node2_id = app
        .selected_info_graph_node_id()
        .expect("Node 2 skal oprettes");

    // Navngiv klasserne
    let _ = app.update(Message::UpdateInformationClassName(
        class1_id,
        "Kunde".to_string(),
    ));
    let _ = app.update(Message::UpdateInformationClassName(
        class2_id,
        "Ordre".to_string(),
    ));

    // Ingen edge er valgt endnu
    assert_eq!(app.selected_info_edge(), None);

    // 2. Simuler Drag-to-Connect: Forbind node1 -> node2 via InfoEdgeCreated
    let _ = app.update(Message::InfoEdgeCreated(node1_id, node2_id));

    // Skal have oprettet relationen i information_graph
    assert_eq!(
        app.project().information_graph().edge_count(),
        1,
        "Relation skal oprettes i informationsgrafen"
    );
    assert_eq!(
        app.selected_info_edge(),
        Some((node1_id, node2_id)),
        "Nyoprettet relation skal være valgt i informationsmodellen"
    );
    assert_eq!(
        app.selected_info_graph_node_id(),
        None,
        "Node skal fravælges når relation oprettes"
    );

    // 3. Opdater label og type via inspektøren
    let _ = app.update(Message::InfoUpdateEdgeLabel(
        node1_id,
        node2_id,
        "afgiver".to_string(),
    ));
    assert_eq!(
        app.project().information_graph().edges()[0].label(),
        Some("afgiver")
    );

    let _ = app.update(Message::InfoUpdateEdgeKind(
        node1_id,
        node2_id,
        RelationKind::Composition,
    ));
    assert_eq!(
        app.project().information_graph().edges()[0].kind(),
        RelationKind::Composition
    );

    // 4. Test kantudvælgelse (deselect / select)
    let _ = app.update(Message::InfoEdgeSelected(None));
    assert_eq!(app.selected_info_edge(), None);

    let _ = app.update(Message::InfoEdgeSelected(Some((node1_id, node2_id))));
    assert_eq!(app.selected_info_edge(), Some((node1_id, node2_id)));

    // 5. Test sletning via Delete-tast (GraphDeleteSelected)
    let _ = app.update(Message::GraphDeleteSelected);
    assert_eq!(
        app.project().information_graph().edge_count(),
        0,
        "Relation skal slettes fra informationsgrafen ved GraphDeleteSelected"
    );
    assert_eq!(app.selected_info_edge(), None);
}

#[test]
fn test_composition_edge_has_diamond_at_source_node() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::edge_router::{EdgeRouter, PortSide};

    let c_whole = Concept::new("Bil", "Et motorkøretøj", BelongsToDomain::Yes);
    let c_part = Concept::new("Motor", "En fremdriftsmaskine", BelongsToDomain::Yes);

    let node_whole = DiagramNode::new(&c_whole, 100.0, 100.0);
    let node_part = DiagramNode::new(&c_part, 400.0, 100.0);

    let edge_comp = DiagramEdge::new(node_whole.id(), node_part.id(), RelationKind::Composition);

    let routes = EdgeRouter::route_edges(&[node_whole.clone(), node_part.clone()], &[edge_comp]);
    assert_eq!(routes.len(), 1);
    let route = &routes[0];

    // Skal have et source_diamond forankret på source-noden (node_whole)
    let diamond = route
        .source_diamond
        .as_ref()
        .expect("Komposition skal have en diamant ved source noden");
    assert_eq!(
        diamond.direction,
        PortSide::Right,
        "Source noden (venstre) skal have diamanten pegende ud fra højre port mod part-noden"
    );
    assert_eq!(
        diamond.tip.x,
        node_whole.x() + node_whole.width(),
        "Diamantens spids skal røre source nodens højre kant præcist"
    );
    assert!(
        diamond.back.x > diamond.tip.x,
        "Diamantens bagerste spids skal pege ud i lærredet mod part-noden"
    );
}

#[test]
fn test_edges_do_not_cross_unnecessarily_when_sorted_vertically() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::edge_router::EdgeRouter;

    let c_person = Concept::new("Person", "En person", BelongsToDomain::Yes);
    let c_cpr = Concept::new("CprPerson", "CPR person", BelongsToDomain::Yes);
    let c_bil = Concept::new("Personbil", "En bil", BelongsToDomain::No);
    let c_org = Concept::new("OrgPerson", "Organisation person", BelongsToDomain::Yes);

    let node_person = DiagramNode::new(&c_person, 100.0, 200.0);
    let node_cpr = DiagramNode::new(&c_cpr, 500.0, 80.0);
    let node_bil = DiagramNode::new(&c_bil, 500.0, 190.0);
    let node_org = DiagramNode::new(&c_org, 500.0, 300.0);

    let nodes = vec![
        node_person.clone(),
        node_cpr.clone(),
        node_bil.clone(),
        node_org.clone(),
    ];

    // Opret edges i "omvendt" rækkefølge (nederste node først, øverste node sidst)
    let edge_org = DiagramEdge::new(
        node_org.id(),
        node_person.id(),
        RelationKind::Generalization,
    );
    let edge_bil = DiagramEdge::new(node_bil.id(), node_person.id(), RelationKind::Association);
    let edge_cpr = DiagramEdge::new(node_cpr.id(), node_person.id(), RelationKind::Composition);

    let edges = vec![edge_org, edge_bil, edge_cpr];
    let routes = EdgeRouter::route_edges(&nodes, &edges);
    assert_eq!(routes.len(), 3);

    // Ingen af disse tre relationer fra parallelle noder til venstre-noden må krydse hinanden!
    let total_bridges: usize = routes.iter().map(|r| r.bridges.len()).sum();
    assert_eq!(
        total_bridges, 0,
        "Relationer må ikke krydse hinanden unødigt (forventede 0 krydsningsbroer, fik {})",
        total_bridges
    );
}

#[test]
fn test_relation_bundling_by_type_and_direction() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, PortSide, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::edge_router::EdgeRouter;

    // 1. Scenarie: Målnode modtager to generaliseringer fra bunden, og har en udgående komposition fra bunden
    let c_person = Concept::new("Person", "Superklasse", BelongsToDomain::Yes);
    let c_org = Concept::new("Organisation", "Subklasse A", BelongsToDomain::Yes);
    let c_cpr = Concept::new("CprPerson", "Subklasse B", BelongsToDomain::Yes);
    let c_comp = Concept::new("OrgPerson", "Komponent", BelongsToDomain::Yes);

    let node_person = DiagramNode::new(&c_person, 300.0, 100.0);
    let node_org = DiagramNode::new(&c_org, 100.0, 300.0);
    let node_cpr = DiagramNode::new(&c_cpr, 300.0, 300.0);
    let node_comp = DiagramNode::new(&c_comp, 500.0, 300.0);

    let nodes = vec![
        node_person.clone(),
        node_org.clone(),
        node_cpr.clone(),
        node_comp.clone(),
    ];

    let edge_org_gen = DiagramEdge::new(
        node_org.id(),
        node_person.id(),
        RelationKind::Generalization,
    );
    let edge_cpr_gen = DiagramEdge::new(
        node_cpr.id(),
        node_person.id(),
        RelationKind::Generalization,
    );
    let edge_person_comp =
        DiagramEdge::new(node_person.id(), node_comp.id(), RelationKind::Composition);

    let edges = vec![edge_org_gen, edge_cpr_gen, edge_person_comp];
    let assignments = EdgeRouter::assign_ports(&nodes, &edges);

    let assign_org = assignments
        .iter()
        .find(|a| a.from_id == node_org.id())
        .unwrap();
    let assign_cpr = assignments
        .iter()
        .find(|a| a.from_id == node_cpr.id())
        .unwrap();
    let assign_comp = assignments
        .iter()
        .find(|a| a.from_id == node_person.id())
        .unwrap();

    // Begge generaliseringer skal ramme Person i bunden (to_side == Bottom)
    assert_eq!(assign_org.to_side, PortSide::Bottom);
    assert_eq!(assign_cpr.to_side, PortSide::Bottom);
    assert_eq!(assign_comp.from_side, PortSide::Bottom);

    // BUNDLING: De to indgående generaliseringer skal dele præcist samme to_slot_offset (bundlet port)
    assert_eq!(
        assign_org.to_slot_offset, assign_cpr.to_slot_offset,
        "Indgående relationer af samme type (Generalisering) skal bundles i samme port-slot"
    );

    // Den udgående komposition skal have sit eget adskilte slot
    assert_ne!(
        assign_org.to_slot_offset, assign_comp.from_slot_offset,
        "Forskellige relationstyper eller retninger må ikke dele slot"
    );

    // Spatiel rækkefølge: Generaliseringerne (noder ved x=100 og x=300) skal ligge til venstre for kompositionen (node ved x=500)
    assert!(
        assign_org.to_slot_offset < assign_comp.from_slot_offset,
        "Spatiel sortering skal placere generaliseringsbundtet til venstre for kompositionen"
    );

    // Routes: Generaliseringerne skal ramme samme endepunkt
    let routes = EdgeRouter::route_edges(&nodes, &edges);
    let route_org = routes.iter().find(|r| r.from == node_org.id()).unwrap();
    let route_cpr = routes.iter().find(|r| r.from == node_cpr.id()).unwrap();
    assert_eq!(
        route_org.points.last(),
        route_cpr.points.last(),
        "De to generaliseringer skal ramme samme pilehoved-forankringspunkt"
    );

    // 2. Scenarie: Udgående bundling - to udgående associationer fra samme side af en kildenode
    let c_root = Concept::new("Root", "Kilde", BelongsToDomain::Yes);
    let c_left = Concept::new("LeftTarget", "Mål 1", BelongsToDomain::Yes);
    let c_right = Concept::new("RightTarget", "Mål 2", BelongsToDomain::Yes);

    let node_root = DiagramNode::new(&c_root, 300.0, 100.0);
    let node_left = DiagramNode::new(&c_left, 150.0, 300.0);
    let node_right = DiagramNode::new(&c_right, 450.0, 300.0);

    let nodes2 = vec![node_root.clone(), node_left.clone(), node_right.clone()];
    let edge_a = DiagramEdge::new(node_root.id(), node_left.id(), RelationKind::Association);
    let edge_b = DiagramEdge::new(node_root.id(), node_right.id(), RelationKind::Association);

    let assignments2 = EdgeRouter::assign_ports(&nodes2, &[edge_a, edge_b]);
    let assign_a = assignments2
        .iter()
        .find(|a| a.to_id == node_left.id())
        .unwrap();
    let assign_b = assignments2
        .iter()
        .find(|a| a.to_id == node_right.id())
        .unwrap();

    assert_eq!(assign_a.from_side, PortSide::Bottom);
    assert_eq!(assign_b.from_side, PortSide::Bottom);
    assert_eq!(
        assign_a.from_slot_offset, assign_b.from_slot_offset,
        "Udgående relationer af samme type på samme side skal bundles i samme kildeslot"
    );
}

#[test]
fn test_stateful_edge_port_hysteresis_and_persistence() {
    use kant::features::concept_model::{DiagramEdge, DiagramNode, PortSide, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::edge_router::EdgeRouter;

    let c_person = Concept::new("Person", "En person", BelongsToDomain::Yes);
    let c_test = Concept::new("TestKlasse", "En testklasse", BelongsToDomain::Yes);
    let c_org = Concept::new("OrgPerson", "Organisation person", BelongsToDomain::Yes);

    // Person i (100, 200), width=180, height=80 -> right=280, bottom=280
    let node_person = DiagramNode::new(&c_person, 100.0, 200.0);
    // TestKlasse i (400, 50) -> Nordøst for Person (x > 280, y < 200)
    let node_test = DiagramNode::new(&c_test, 400.0, 50.0);
    // OrgPerson i (400, 380) -> Sydøst for Person (x > 280, y > 280)
    let node_org = DiagramNode::new(&c_org, 400.0, 380.0);

    let nodes = vec![node_person.clone(), node_test.clone(), node_org.clone()];

    // 1. Initial oprettelse med låst/husket Right port
    let mut edge_test =
        DiagramEdge::new(node_person.id(), node_test.id(), RelationKind::Association);
    edge_test.set_ports(Some(PortSide::Right), Some(PortSide::Left));

    let mut edge_org = DiagramEdge::new(
        node_org.id(),
        node_person.id(),
        RelationKind::Generalization,
    );
    edge_org.set_ports(Some(PortSide::Left), Some(PortSide::Right));

    let routes = EdgeRouter::route_edges(&nodes, &[edge_test.clone(), edge_org.clone()]);
    assert_eq!(routes.len(), 2);

    // edge_test skal udgå fra Person's højre side og ramme TestKlasse's venstre side
    assert_eq!(
        routes[0].from_side,
        PortSide::Right,
        "Person skal bevare højre port i Nordøst-kvadranten"
    );
    assert_eq!(
        routes[0].to_side,
        PortSide::Left,
        "TestKlasse skal rammes på venstre side"
    );

    // edge_org (generalisering) skal ramme Person på højre side og udgå fra OrgPerson's venstre side
    assert_eq!(
        routes[1].from_side,
        PortSide::Left,
        "OrgPerson skal udgå fra venstre side mod Person"
    );
    assert_eq!(routes[1].to_side, PortSide::Right, "Person skal modtage generalisering på højre side fremfor at lave baglæns u-vending under bunden");

    // 2. Hysterese-udløser: Flyt TestKlasse ind over den vertikale linje (x < person.right)
    // Person right er 100 + 180 = 280. Sæt TestKlasse x = 150 (direkte over Person)
    let mut node_test_above = node_test.clone();
    node_test_above.set_position(150.0, 50.0);
    let routes_above = EdgeRouter::route_edges(
        &[node_person.clone(), node_test_above, node_org.clone()],
        &[edge_test.clone(), edge_org.clone()],
    );
    assert_eq!(
        routes_above[0].from_side,
        PortSide::Top,
        "Når noden trækkes ind over den vertikale grænse over Person, skal porten skifte til Top"
    );

    // 3. Persistens: Serialisering og deserialisering med serde
    let serialized =
        serde_json::to_string(&edge_test).expect("DiagramEdge skal kunne serialiseres");
    assert!(
        serialized.contains("Right"),
        "JSON skal indeholde 'Right' portside"
    );
    let deserialized: DiagramEdge =
        serde_json::from_str(&serialized).expect("DiagramEdge skal deserialiseres");
    assert_eq!(deserialized.source_port(), Some(PortSide::Right));
    assert_eq!(deserialized.target_port(), Some(PortSide::Left));

    // Bagudkompatibilitet: JSON uden porte skal deserialisere med None
    let legacy_json = format!(
        r#"{{"from":"{}","to":"{}","kind":"Association","label":null}}"#,
        node_person.id(),
        node_test.id()
    );
    let legacy_edge: DiagramEdge =
        serde_json::from_str(&legacy_json).expect("Legacy JSON skal deserialiseres uden fejl");
    assert_eq!(legacy_edge.source_port(), None);
    assert_eq!(legacy_edge.target_port(), None);
}

#[test]
fn test_directed_association_half_arrow_and_reversal() {
    use iced::Point;
    use kant::features::concept_model::{DiagramEdge, DiagramNode, PortSide, RelationKind};
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::ClassGraph;
    use kant::ui::edge_router::EdgeRouter;

    let c_a = Concept::new("KlasseA", "A", BelongsToDomain::Yes);
    let c_b = Concept::new("KlasseB", "B", BelongsToDomain::Yes);

    let node_a = DiagramNode::new(&c_a, 100.0, 100.0);
    let node_b = DiagramNode::new(&c_b, 400.0, 100.0);

    // 1. Association er rettet som standard (directed == true)
    let edge_assoc = DiagramEdge::new(node_a.id(), node_b.id(), RelationKind::Association);
    assert!(
        edge_assoc.is_directed(),
        "Association skal være rettet som standard"
    );

    let routes = EdgeRouter::route_edges(
        &[node_a.clone(), node_b.clone()],
        std::slice::from_ref(&edge_assoc),
    );
    assert_eq!(routes.len(), 1);
    let route = &routes[0];

    // Skal have et half_arrow mod målnoden node_b (venstre port på node_b)
    let half_arrow = route
        .half_arrow
        .as_ref()
        .expect("Rettet association skal have en halv pil");
    assert_eq!(half_arrow.tip, Point::new(node_b.x(), node_b.center().1));
    assert_eq!(half_arrow.direction, PortSide::Left);

    // 2. Kan slå pilen fra (undirected association)
    let mut edge_undirected = edge_assoc.clone();
    edge_undirected.set_directed(false);
    assert!(!edge_undirected.is_directed());
    let routes_undirected =
        EdgeRouter::route_edges(&[node_a.clone(), node_b.clone()], &[edge_undirected]);
    assert!(
        routes_undirected[0].half_arrow.is_none(),
        "Uorienteret association må ikke have en halv pil"
    );

    // 3. Retningsvending (reverse_relation) i ClassGraph / ConceptGraph
    let mut graph = ClassGraph::new();
    let n1 = graph.add_node(uuid::Uuid::new_v4(), 0);
    let n2 = graph.add_node(uuid::Uuid::new_v4(), 0);
    graph.add_relation(
        n1,
        n2,
        RelationKind::Association,
        Some("forbinder".to_string()),
    );
    graph.update_edge_ports(n1, n2, Some(PortSide::Right), Some(PortSide::Left));

    assert!(graph.find_edge(n1, n2).is_some());
    assert!(graph.find_edge(n2, n1).is_none());

    let reversed = graph.reverse_relation(n1, n2);
    assert!(reversed, "Skal kunne vende relation");
    assert!(
        graph.find_edge(n1, n2).is_none(),
        "Gammel retning skal være fjernet"
    );
    let rev_edge = graph
        .find_edge(n2, n1)
        .expect("Ny vendt relation skal findes");
    assert_eq!(rev_edge.from(), n2);
    assert_eq!(rev_edge.to(), n1);
    assert_eq!(
        rev_edge.source_port(),
        Some(PortSide::Left),
        "Porte skal være spejlvendt"
    );
    assert_eq!(
        rev_edge.target_port(),
        Some(PortSide::Right),
        "Porte skal være spejlvendt"
    );
    assert_eq!(rev_edge.label(), Some("forbinder"));

    // 4. ConceptGraph reversal og retning
    use kant::features::concept_model::ConceptGraph;
    let mut cgraph = ConceptGraph::new();
    let cn1 = cgraph.add_node(&c_a);
    let cn2 = cgraph.add_node(&c_b);
    cgraph.add_relation(cn1, cn2, RelationKind::Association);
    assert!(cgraph.find_edge(cn1, cn2).unwrap().is_directed());

    cgraph.update_edge_directed(cn1, cn2, false);
    assert!(!cgraph.find_edge(cn1, cn2).unwrap().is_directed());

    let rev_c = cgraph.reverse_relation(cn1, cn2);
    assert!(rev_c);
    assert!(cgraph.find_edge(cn1, cn2).is_none());
    assert!(cgraph.find_edge(cn2, cn1).is_some());

    // 5. App Message håndtering
    use kant::ui::app::{App, Message};
    let mut app = App::new_with_path(None);
    let app_n1 = app.project_mut().concept_graph_mut().add_node(&c_a);
    let app_n2 = app.project_mut().concept_graph_mut().add_node(&c_b);
    app.project_mut()
        .concept_graph_mut()
        .add_relation(app_n1, app_n2, RelationKind::Association);
    let _ = app.update(Message::GraphEdgeSelected(Some((app_n1, app_n2))));

    let _ = app.update(Message::GraphToggleEdgeDirected(app_n1, app_n2, false));
    assert!(!app
        .project()
        .concept_graph()
        .find_edge(app_n1, app_n2)
        .unwrap()
        .is_directed());

    let _ = app.update(Message::GraphReverseEdge(app_n1, app_n2));
    assert!(app
        .project()
        .concept_graph()
        .find_edge(app_n2, app_n1)
        .is_some());

    // 6. Informationsmodel Message håndtering
    use kant::features::information_model::InformationClass;
    let cl_a = app
        .project_mut()
        .information_model_mut()
        .add_class(InformationClass::new("KlasseA"));
    let cl_b = app
        .project_mut()
        .information_model_mut()
        .add_class(InformationClass::new("KlasseB"));
    let info_n1 = app.project_mut().information_graph_mut().add_node(cl_a, 0);
    let info_n2 = app.project_mut().information_graph_mut().add_node(cl_b, 0);
    app.project_mut().information_graph_mut().add_relation(
        info_n1,
        info_n2,
        RelationKind::Association,
        None,
    );
    let _ = app.update(Message::InfoEdgeSelected(Some((info_n1, info_n2))));

    let _ = app.update(Message::InfoToggleEdgeDirected(info_n1, info_n2, false));
    assert!(!app
        .project()
        .information_graph()
        .find_edge(info_n1, info_n2)
        .unwrap()
        .is_directed());

    let _ = app.update(Message::InfoReverseEdge(info_n1, info_n2));
    assert!(app
        .project()
        .information_graph()
        .find_edge(info_n2, info_n1)
        .is_some());

    // 7. Filtrering af "+ Fra begreb..." når klasse med samme navn allerede findes
    let c_c = Concept::new("KlasseC", "C", BelongsToDomain::Yes);
    let concepts_list = [c_a.clone(), c_c.clone()];
    let existing_class_names: std::collections::HashSet<String> = app
        .project()
        .information_model()
        .classes()
        .iter()
        .map(|c| c.name().trim().to_lowercase())
        .collect();

    let filtered_concept_options: Vec<_> = concepts_list
        .iter()
        .filter(|c| !existing_class_names.contains(&c.preferred_term().trim().to_lowercase()))
        .collect();

    assert_eq!(filtered_concept_options.len(), 1);
    assert_eq!(filtered_concept_options[0].preferred_term(), "KlasseC");
}

#[test]
fn test_task_017_model_metadata_modal_and_3phase_tabs() {
    let mut app = App::new_with_path(None);

    // 1. Standard fanen ved opstart skal være Tab::ConceptList (3-faset navigation)
    assert_eq!(app.active_tab(), Tab::ConceptList);

    // Skift imellem faserne
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    assert_eq!(app.active_tab(), Tab::ConceptList);

    // 2. Åbn ModelMetadataModal
    assert!(app.metadata_modal().is_none());
    let _ = app.update(Message::OpenMetadataModal);
    assert!(app.metadata_modal().is_some());

    // 3. Opdater metadatafelter i modalen
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Name,
        "Danmarks Grunddatamodel".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Description,
        "Fællesoffentlig referencemodel".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataStatus(ModelStatus::Approved));
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::DomainArea,
        "Tværoffentlig Grunddata".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::ResponsibleOrg,
        "Digitaliseringsstyrelsen".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Uri,
        "https://data.gov.dk/model/core/grunddata".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Version,
        "2.1.0".to_string(),
    ));

    // Før gem er projektets metadata uændret
    assert_eq!(app.project().metadata().name(), "Nyt FDA Modelprojekt");

    // 4. Gem ændringer
    let _ = app.update(Message::SaveMetadataModal);
    assert!(app.metadata_modal().is_none());

    let meta = app.project().metadata();
    assert_eq!(meta.name(), "Danmarks Grunddatamodel");
    assert_eq!(meta.description(), "Fællesoffentlig referencemodel");
    assert_eq!(meta.status(), ModelStatus::Approved);
    assert_eq!(meta.domain_area(), "Tværoffentlig Grunddata");
    assert_eq!(meta.responsible_org(), "Digitaliseringsstyrelsen");
    assert_eq!(meta.uri(), "https://data.gov.dk/model/core/grunddata");
    assert_eq!(meta.version(), "2.1.0");
    assert_eq!(app.save_status(), &kant::ui::app::SaveStatus::Unsaved);

    // 5. Test annullering via Escape / Close
    let _ = app.update(Message::OpenMetadataModal);
    assert!(app.metadata_modal().is_some());
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Name,
        "Uønsket ændring".to_string(),
    ));
    let _ = app.update(Message::EscapePressed);
    assert!(app.metadata_modal().is_none());
    assert_eq!(app.project().metadata().name(), "Danmarks Grunddatamodel");

    let _ = app.update(Message::OpenMetadataModal);
    let _ = app.update(Message::CloseMetadataModal);
    assert!(app.metadata_modal().is_none());
}

#[test]
fn test_metadata_modal_default_placeholders_allow_direct_typing() {
    let mut app = App::new_with_path(None);

    // Åbn modal på et nyt projekt
    let _ = app.update(Message::OpenMetadataModal);
    let modal = app.metadata_modal().expect("Modal bør være åben");

    // Felter med standardværdier starter som tomme strenge, så placeholder vises og brugeren kan skrive direkte
    assert_eq!(modal.name, "");
    assert_eq!(modal.description, "");
    assert_eq!(modal.domain_area, "");
    assert_eq!(modal.responsible_org, "");
    assert_eq!(modal.uri, "");
    assert_eq!(modal.version, "");

    // Hvis brugeren gemmer uden at skrive noget, fastholdes kanoniske standardværdier
    let _ = app.update(Message::SaveMetadataModal);
    assert_eq!(app.project().metadata().name(), "Nyt FDA Modelprojekt");
    assert_eq!(app.project().metadata().version(), "0.1.0");
}

#[test]
fn test_metadata_modal_fda_tabel_d_fields_roundtrip() {
    use kant::features::model::{ApprovalStatus, ModelScope, ModelStatus};
    use kant::ui::app::MetadataField;

    let mut app = App::new_with_path(None);

    let _ = app.update(Message::OpenMetadataModal);
    assert!(app.metadata_modal().is_some());

    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Name,
        "Bygninger og Adresser".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Description,
        "FDA kernemodel for geografiske enheder".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Uri,
        "https://data.gov.dk/model/core/building".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataStatus(ModelStatus::Completed));
    let _ = app.update(Message::UpdateMetadataApprovalStatus(
        ApprovalStatus::ApprovedWithRemarks,
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::ApprovedBy,
        "FDA Modeludvalg".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataScope(ModelScope::ApplicationProfile));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::DomainArea,
        "BBR & Adresser".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::ResponsibleOrg,
        "Klimadatastyrelsen".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Version,
        "1.2.0".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Language,
        "da".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::DateModified,
        "2026-09-30".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::VersionNotes,
        "Tilføjet etageadskillelse attribut".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::LegalSources,
        "https://www.retsinformation.dk/eli/lta/2020/123, https://www.retsinformation.dk/eli/lta/2021/456".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::Source,
        "INSPIRE Building 2D".to_string(),
    ));
    let _ = app.update(Message::UpdateMetadataField(
        MetadataField::WasDerivedFrom,
        "https://data.gov.dk/model/core/geodata".to_string(),
    ));

    let _ = app.update(Message::SaveMetadataModal);
    assert!(app.metadata_modal().is_none());

    let meta = app.project().metadata();
    assert_eq!(meta.name(), "Bygninger og Adresser");
    assert_eq!(meta.description(), "FDA kernemodel for geografiske enheder");
    assert_eq!(meta.uri(), "https://data.gov.dk/model/core/building");
    assert_eq!(meta.model_status(), ModelStatus::Completed);
    assert_eq!(meta.approval_status(), ApprovalStatus::ApprovedWithRemarks);
    assert_eq!(meta.approved_by(), Some("FDA Modeludvalg"));
    assert_eq!(meta.model_scope(), ModelScope::ApplicationProfile);
    assert_eq!(meta.domain_area(), "BBR & Adresser");
    assert_eq!(meta.responsible_org(), "Klimadatastyrelsen");
    assert_eq!(meta.version(), "1.2.0");
    assert_eq!(meta.language(), "da");
    assert_eq!(meta.date_modified(), "2026-09-30");
    assert_eq!(
        meta.version_notes(),
        Some("Tilføjet etageadskillelse attribut")
    );
    assert_eq!(
        meta.legal_sources(),
        &[
            "https://www.retsinformation.dk/eli/lta/2020/123".to_string(),
            "https://www.retsinformation.dk/eli/lta/2021/456".to_string(),
        ]
    );
    assert_eq!(meta.source(), Some("INSPIRE Building 2D"));
    assert_eq!(
        meta.was_derived_from(),
        Some("https://data.gov.dk/model/core/geodata")
    );
}

#[test]
fn test_task018_footer_timestamp_and_model_rules_link() {
    // 1. Nyt projekt starter som Unsaved med "Nyt projekt" tekst
    let mut app = App::new_with_path(None);
    assert_eq!(app.save_status(), &kant::ui::app::SaveStatus::Unsaved);
    assert!(app.footer_status_text().contains("⚠️ Nyt projekt"));

    // 2. Åbning af modelregler via Message::OpenModelRules (skal undertrykkes i testmiljø uden at åbne ekstern browser)
    let _ = app.update(Message::OpenModelRules);
    assert!(
        kant::features::diagnostics::get_recent_logs()
            .iter()
            .any(|l| l.contains("Browseråbning undertrykt i testmiljø")),
        "Browseråbning skal undertrykkes i testmiljøer for at forhindre popup-faner"
    );

    // 3. Gem til fil genererer tidsstempel med format HH:MM:SS og filnavn
    let file_path =
        std::env::temp_dir().join(format!("danmark_model_{}.edge.json", uuid::Uuid::new_v4()));
    let filename = file_path.file_name().unwrap().to_str().unwrap().to_string();
    let _ = app.update(Message::SaveProjectToFile(file_path.clone()));

    match app.save_status() {
        kant::ui::app::SaveStatus::Saved { path, timestamp } => {
            assert_eq!(path, &file_path.display().to_string());
            assert_eq!(timestamp.len(), 8); // "HH:MM:SS"
            let parts: Vec<&str> = timestamp.split(':').collect();
            assert_eq!(parts.len(), 3);
            let hours: u32 = parts[0].parse().expect("hours should be numeric");
            let mins: u32 = parts[1].parse().expect("mins should be numeric");
            let secs: u32 = parts[2].parse().expect("secs should be numeric");
            assert!(hours < 24);
            assert!(mins < 60);
            assert!(secs < 60);
        }
        other => panic!("Expected SaveStatus::Saved, got {:?}", other),
    }

    // 4. Footer-teksten formateres korrekt med tidsstempel og filnavn
    let footer_text = app.footer_status_text();
    assert!(footer_text.starts_with("💾 Sidst gemt kl. "));
    assert!(footer_text.contains(&format!("• {}", filename)));

    // 5. Ændringer efter gemning viser '⚠️ Ikke gemte ændringer'
    let _ = app.update(Message::OpenMetadataModal);
    let _ = app.update(Message::UpdateMetadataField(
        kant::ui::app::MetadataField::Name,
        "Opdateret navn".to_string(),
    ));
    let _ = app.update(Message::SaveMetadataModal);

    assert_eq!(app.save_status(), &kant::ui::app::SaveStatus::Unsaved);
    assert_eq!(app.footer_status_text(), "⚠️ Ikke gemte ændringer");

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_task019_palette_header_actions_and_search_affinity() {
    let mut app = App::new_with_path(None);

    // 1. Begrebsmodel fane: Valider søge-nærhed og header-handlinger
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);

    // Initial søgetilstand er tom
    assert_eq!(app.concept_model_search(), "");

    // Søgning opdaterer søgestrengen
    let _ = app.update(Message::ConceptModelSearchChanged("køre".to_string()));
    assert_eq!(app.concept_model_search(), "køre");

    // Start nyt begreb via header-handling
    let _ = app.update(Message::StartNewConcept);
    assert!(
        app.concept_editor().is_some(),
        "Inline editor skal åbnes ved StartNewConcept"
    );

    // Rendering af view i søgetilstand med aktiv editor
    {
        let _view = app.view();
    }

    // 2. Informationsmodel fane: Valider søge-nærhed og header-handlinger
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    // Initial søgetilstand er tom
    assert_eq!(app.information_model_search(), "");

    // Søgning opdaterer søgestrengen
    let _ = app.update(Message::InformationClassSearchChanged("bil".to_string()));
    assert_eq!(app.information_model_search(), "bil");

    // Opret ny klasse direkte via header handling
    let initial_class_count = app.project().information_model().classes().len();
    let _ = app.update(Message::CreateInformationClass);
    assert_eq!(
        app.project().information_model().classes().len(),
        initial_class_count + 1,
        "Ny klasse skal oprettes"
    );

    // Opret klasse fra begreb via header pick_list
    // Først tilføjer vi et begreb i projektet
    let c = Concept::new("Vejkøretøj", "Køretøj på vej", BelongsToDomain::Yes);
    let c_id = c.id();
    app.project_mut().concepts_mut().push(c);
    let _ = app.update(Message::CreateInformationClassFromConcept(ConceptOption {
        id: c_id,
        term: "Vejkøretøj".to_string(),
    }));
    assert_eq!(
        app.project().information_model().classes().len(),
        initial_class_count + 2,
        "Klasse fra begreb skal oprettes"
    );

    // Rendering af informationsmodel view med søgning
    {
        let _view = app.view();
    }
}

#[test]
fn test_task020_harmonized_inspector_and_guidance_panels() {
    use kant::ui::inspector_panel::{GUIDANCE_TITLE, PROPERTIES_TITLE};

    assert_eq!(PROPERTIES_TITLE, "Egenskaber");
    assert_eq!(GUIDANCE_TITLE, "Vejledning");

    let mut app = App::new_with_path(None);

    // 1. Begrebsmodel: Tom tilstand viser Vejledning
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.selected_graph_node_id(), None);
    assert_eq!(app.selected_edge(), None);
    {
        let _ = app.view();
    }

    // 2. Begrebsmodel: Node valgt viser Egenskaber
    let c = Concept::new("Bygning", "Fast konstruktion", BelongsToDomain::Yes);
    let node_id = app.project_mut().concept_graph_mut().add_node(&c);
    app.project_mut().concepts_mut().push(c);
    let _ = app.update(Message::GraphNodeSelected(Some(node_id)));
    assert_eq!(app.selected_graph_node_id(), Some(node_id));
    {
        let _ = app.view();
    }

    // Luk Egenskaber via deselect
    let _ = app.update(Message::GraphNodeSelected(None));
    assert_eq!(app.selected_graph_node_id(), None);
    {
        let _ = app.view();
    }

    // 3. Begrebsmodel: Relation valgt viser Egenskaber
    let c2 = Concept::new("Etage", "Vandret del af bygning", BelongsToDomain::Yes);
    let node2_id = app.project_mut().concept_graph_mut().add_node(&c2);
    app.project_mut().concepts_mut().push(c2);
    app.project_mut().concept_graph_mut().add_relation(
        node_id,
        node2_id,
        RelationKind::Composition,
    );
    let _ = app.update(Message::GraphEdgeSelected(Some((node_id, node2_id))));
    assert_eq!(app.selected_edge(), Some((node_id, node2_id)));
    {
        let _ = app.view();
    }

    // Luk relation via deselect
    let _ = app.update(Message::GraphEdgeSelected(None));
    assert_eq!(app.selected_edge(), None);
    {
        let _ = app.view();
    }

    // 4. Informationsmodel: Tom tilstand viser Vejledning
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.selected_info_class_id(), None);
    assert_eq!(app.selected_info_edge(), None);
    {
        let _ = app.view();
    }

    // 5. Informationsmodel: Klasse valgt viser Egenskaber
    let class = InformationClass::new("Bygningsdel");
    let class_id = class.id();
    let _ = app.project_mut().information_model_mut().add_class(class);
    let _ = app.update(Message::SelectInformationClass(Some(class_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_id));
    {
        let _ = app.view();
    }

    // Luk klasse via deselect
    let _ = app.update(Message::SelectInformationClass(None));
    assert_eq!(app.selected_info_class_id(), None);
    {
        let _ = app.view();
    }

    // 6. Informationsmodel: Relation valgt viser Egenskaber
    let class2 = InformationClass::new("Rum");
    let class2_id = class2.id();
    let _ = app.project_mut().information_model_mut().add_class(class2);
    let info_node1 = app
        .project_mut()
        .information_graph_mut()
        .add_node(class_id, 0);
    let info_node2 = app
        .project_mut()
        .information_graph_mut()
        .add_node(class2_id, 0);
    let _ = app.update(Message::AddClassRelation(
        info_node1,
        info_node2,
        RelationKind::Association,
        None,
    ));
    let _ = app.update(Message::InfoEdgeSelected(Some((info_node1, info_node2))));
    assert_eq!(app.selected_info_edge(), Some((info_node1, info_node2)));
    {
        let _ = app.view();
    }

    // Luk info relation via deselect
    let _ = app.update(Message::InfoEdgeSelected(None));
    assert_eq!(app.selected_info_edge(), None);
    {
        let _ = app.view();
    }
}

#[test]
fn test_task021_attribute_to_concept_lineage_and_traceability() {
    let mut app = App::new_with_path(None);

    // 1. Skift til Informationsmodel fanen
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // 2. Opret forretningsbegreb
    let concept = Concept::new(
        "Matrikelnummer",
        "En entydig identifikation af en samlet fast ejendom.",
        BelongsToDomain::Yes,
    );
    let concept_id = concept.id();
    let _ = app.project_mut().add_concept(concept);

    // 3. Opret Klasse og tilføj en attribut
    let mut class = InformationClass::new("FastEjendom");
    let class_id = class.id();
    let attr = Attribute::new(
        "matrikelnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    );
    let attr_id = attr.id();
    class.add_attribute(attr);
    let _ = app.project_mut().information_model_mut().add_class(class);

    // Vælg klassen i inspectoren
    let _ = app.update(Message::SelectInformationClass(Some(class_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_id));

    // Før tildeling er concept_ids tom
    {
        let attr = app
            .project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .attributes()
            .iter()
            .find(|a| a.id() == attr_id)
            .unwrap();
        assert!(attr.concept_ids().is_empty());
    }

    // 4. Sæt attributtens begrebs-lineage via Message::SetAttributeConcept
    let _ = app.update(Message::SetAttributeConcept(
        class_id,
        attr_id,
        Some(concept_id),
    ));

    // Verificer at concept_id er tilknyttet attributten
    {
        let attr = app
            .project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .attributes()
            .iter()
            .find(|a| a.id() == attr_id)
            .unwrap();
        assert_eq!(attr.concept_ids(), &[concept_id]);
    }

    // 5. Serialisering / Deserialisering bevarer concept_ids
    let json = serde_json::to_string(app.project()).expect("Serialisering skal lykkes");
    let loaded_project: ModelProject =
        serde_json::from_str(&json).expect("Deserialisering skal lykkes");
    let loaded_attr = loaded_project
        .information_model()
        .get_class(class_id)
        .unwrap()
        .attributes()
        .iter()
        .find(|a| a.id() == attr_id)
        .unwrap();
    assert_eq!(loaded_attr.concept_ids(), &[concept_id]);

    // 6. Inspectoren renderer view med lineage indikation uden fejl
    {
        let _view = app.view();
    }

    // 7. Fjern begrebstilknytning ved at sætte None (teknisk felt)
    let _ = app.update(Message::SetAttributeConcept(class_id, attr_id, None));
    {
        let attr = app
            .project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .attributes()
            .iter()
            .find(|a| a.id() == attr_id)
            .unwrap();
        assert!(
            attr.concept_ids().is_empty(),
            "Attributten skal nu være uden begreb"
        );
    }

    // 8. Knyt begrebet igen og test sletning af begreb (Robusthed jf. Must NOT)
    let _ = app.update(Message::SetAttributeConcept(
        class_id,
        attr_id,
        Some(concept_id),
    ));
    let _ = app.update(Message::DeleteConcept(concept_id));

    // Must NOT: Må IKKE slette attributten når begrebet slettes
    {
        let class = app
            .project()
            .information_model()
            .get_class(class_id)
            .expect("Klassen skal stadig eksistere");
        let attr = class
            .attributes()
            .iter()
            .find(|a| a.id() == attr_id)
            .expect("Attributten må IKKE være slettet");
        assert!(
            attr.concept_ids().is_empty(),
            "Slettet begreb skal være fjernet fra attributtens lineage"
        );
    }

    // Inspectoren renderer view efter sletning uden fejl
    {
        let _view = app.view();
    }
}

#[test]
fn test_attribute_lineage() {
    test_task021_attribute_to_concept_lineage_and_traceability();
}

#[test]
fn test_task022_information_model_association_multiplicities() {
    use kant::features::information_model::ClassDiagramEdge;
    use kant::ui::diagram_canvas::CanvasEdge;

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_task022_multiplicities_{}.edge.json",
        uuid::Uuid::new_v4()
    ));

    let mut app = App::new_with_path(Some(file_path.clone()));
    let _ = app.update(Message::SelectTab(Tab::InformationModel));

    // 1. Opret to nye klasser i informationsmodellen
    let initial_count = app.project().information_model().classes().len();

    let _ = app.update(Message::CreateInformationClass);
    let class_kunde_id = app.project().information_model().classes()[initial_count].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_kunde_id,
        "Kunde".to_string(),
    ));

    let _ = app.update(Message::CreateInformationClass);
    let class_ordre_id = app.project().information_model().classes()[initial_count + 1].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_ordre_id,
        "Ordre".to_string(),
    ));

    let _ = app.update(Message::AddClassToDiagram(class_kunde_id));
    let _ = app.update(Message::AddClassToDiagram(class_ordre_id));

    let node_kunde_id = app
        .project()
        .information_graph()
        .find_node_by_class(class_kunde_id)
        .unwrap()
        .id();
    let node_ordre_id = app
        .project()
        .information_graph()
        .find_node_by_class(class_ordre_id)
        .unwrap()
        .id();

    // 2. Test ClassDiagramEdge domænefelter og serde bagudkompatibilitet
    let mut direct_edge = ClassDiagramEdge::new(
        node_kunde_id,
        node_ordre_id,
        RelationKind::Association,
        Some("har".to_string()),
    );
    assert_eq!(direct_edge.source_multiplicity(), None);
    assert_eq!(direct_edge.target_multiplicity(), None);

    direct_edge.set_source_multiplicity(Some(Multiplicity::exactly_one()));
    direct_edge.set_target_multiplicity(Some(Multiplicity::zero_or_more()));
    assert_eq!(
        direct_edge.source_multiplicity(),
        Some(Multiplicity::exactly_one())
    );
    assert_eq!(
        direct_edge.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // Serde roundtrip på ClassDiagramEdge
    let edge_json = serde_json::to_string(&direct_edge).expect("Skal kunne serialiseres");
    let deser_edge: ClassDiagramEdge =
        serde_json::from_str(&edge_json).expect("Skal kunne deserialiseres");
    assert_eq!(
        deser_edge.source_multiplicity(),
        Some(Multiplicity::exactly_one())
    );
    assert_eq!(
        deser_edge.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // Bagudkompatibilitet: legacy edge uden multiplicitetsfelter
    let legacy_json = format!(
        r#"{{"from":"{}","to":"{}","kind":"Association","label":"legacy"}}"#,
        node_kunde_id, node_ordre_id
    );
    let legacy_edge: ClassDiagramEdge =
        serde_json::from_str(&legacy_json).expect("Skal kunne deserialisere legacy json");
    assert_eq!(legacy_edge.source_multiplicity(), None);
    assert_eq!(legacy_edge.target_multiplicity(), None);

    // CanvasEdge trait abstraction
    assert_eq!(
        <ClassDiagramEdge as CanvasEdge>::source_multiplicity(&direct_edge),
        Some("1".to_string())
    );
    assert_eq!(
        <ClassDiagramEdge as CanvasEdge>::target_multiplicity(&direct_edge),
        Some("0..*".to_string())
    );

    // 3. Test oprettelsesdialogen for relationer med multipliciteter
    let _ = app.update(Message::OpenInfoRelationDialog);
    let _ = app.update(Message::InfoRelationFromChanged(
        kant::ui::app::NodeOption {
            id: node_kunde_id,
            label: "Kunde".to_string(),
        },
    ));
    let _ = app.update(Message::InfoRelationToChanged(kant::ui::app::NodeOption {
        id: node_ordre_id,
        label: "Ordre".to_string(),
    }));
    let _ = app.update(Message::InfoRelationKindChanged(RelationKind::Association));
    let _ = app.update(Message::InfoRelationLabelChanged("afgiver".to_string()));
    let _ = app.update(Message::InfoRelationSourceMultiplicityChanged(Some(
        Multiplicity::one_or_more(),
    )));
    let _ = app.update(Message::InfoRelationTargetMultiplicityChanged(Some(
        Multiplicity::zero_or_more(),
    )));

    // View under åben dialog med multiplicitetsvælgere
    {
        let _dialog_view = app.view();
    }

    let _ = app.update(Message::InfoCreateRelation);

    let created_edge = app
        .project()
        .information_graph()
        .find_edge(node_kunde_id, node_ordre_id)
        .expect("Relation skal være oprettet i grafen");
    assert_eq!(
        created_edge.source_multiplicity(),
        Some(Multiplicity::one_or_more())
    );
    assert_eq!(
        created_edge.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // 4. Test Egenskaber-panelet (Inspector) for valgt relation
    let _ = app.update(Message::InfoEdgeSelected(Some((
        node_kunde_id,
        node_ordre_id,
    ))));

    // Rediger kildemultiplicitet til 0..1 via inspector
    let _ = app.update(Message::InfoUpdateEdgeSourceMultiplicity(
        node_kunde_id,
        node_ordre_id,
        Some(Multiplicity::zero_or_one()),
    ));
    // Rediger målmultiplicitet til 1 via inspector
    let _ = app.update(Message::InfoUpdateEdgeTargetMultiplicity(
        node_kunde_id,
        node_ordre_id,
        Some(Multiplicity::exactly_one()),
    ));

    {
        let edge = app
            .project()
            .information_graph()
            .find_edge(node_kunde_id, node_ordre_id)
            .unwrap();
        assert_eq!(
            edge.source_multiplicity(),
            Some(Multiplicity::zero_or_one())
        );
        assert_eq!(
            edge.target_multiplicity(),
            Some(Multiplicity::exactly_one())
        );
    }

    // 5. Test reversering af relation (vender også multipliciteter jf. symmetri)
    let _ = app.update(Message::InfoReverseEdge(node_kunde_id, node_ordre_id));
    {
        let reversed = app
            .project()
            .information_graph()
            .find_edge(node_ordre_id, node_kunde_id)
            .expect("Reverseret relation skal eksistere");
        assert_eq!(
            reversed.source_multiplicity(),
            Some(Multiplicity::exactly_one())
        );
        assert_eq!(
            reversed.target_multiplicity(),
            Some(Multiplicity::zero_or_one())
        );
    }

    // 6. Must NOT: Generalisering må IKKE have obligatorisk multiplicitet
    let _ = app.update(Message::InfoUpdateEdgeKind(
        node_ordre_id,
        node_kunde_id,
        RelationKind::Generalization,
    ));
    let gen_edge = app
        .project()
        .information_graph()
        .find_edge(node_ordre_id, node_kunde_id)
        .unwrap();
    assert_eq!(gen_edge.kind(), RelationKind::Generalization);
    // Generalisering på canvas må ikke vise association-multipliciteter
    assert_eq!(
        <ClassDiagramEdge as CanvasEdge>::source_multiplicity(gen_edge),
        None
    );
    assert_eq!(
        <ClassDiagramEdge as CanvasEdge>::target_multiplicity(gen_edge),
        None
    );

    // 7. Skift tilbage til association og verificer lærredsrendering
    let _ = app.update(Message::InfoUpdateEdgeKind(
        node_ordre_id,
        node_kunde_id,
        RelationKind::Association,
    ));
    let _ = app.update(Message::InfoUpdateEdgeSourceMultiplicity(
        node_ordre_id,
        node_kunde_id,
        Some(Multiplicity::exactly_one()),
    ));
    let _ = app.update(Message::InfoUpdateEdgeTargetMultiplicity(
        node_ordre_id,
        node_kunde_id,
        Some(Multiplicity::zero_or_more()),
    ));

    // Verificer at canvas og inspector renderer uden panics og med multipliciteter
    {
        let _view = app.view();
    }

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_class_diagram_edge() {
    test_task022_information_model_association_multiplicities();
}

#[test]
fn test_task023_canvas_floating_controls_and_minimap() {
    use iced::mouse::{self, Cursor};
    use iced::widget::canvas::{Event, Program};
    use iced::{Point, Rectangle, Size};
    use kant::features::concept_model::{DiagramEdge, DiagramNode};
    use kant::ui::diagram_canvas::{
        render_concept_node, CanvasViewport, DiagramCanvas, DiagramCanvasState,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    // 1. Opret diagram med noder - herunder en node placeret nede i højre hjørne
    // hvor det svævende kontrolpanel vil ligge på en 1000x800 skærm
    let n1 = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeA".to_string(),
        100.0,
        100.0,
        160.0,
        80.0,
    );
    let n2 = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeB".to_string(),
        400.0,
        300.0,
        160.0,
        80.0,
    );
    // Node placeret i nederste højre hjørne under det svævende panel (800..980, 630..750)
    let n_under_panel = DiagramNode::custom(
        Uuid::new_v4(),
        "NodeUnderPanel".to_string(),
        800.0,
        630.0,
        180.0,
        120.0,
    );
    let nodes = vec![n1.clone(), n2.clone(), n_under_panel.clone()];
    let edges: Vec<DiagramEdge> = vec![];

    let selected_node = Arc::new(std::sync::Mutex::new(None));
    let last_viewport = Arc::new(std::sync::Mutex::new(CanvasViewport::default()));

    let sel_clone = Arc::clone(&selected_node);
    let vp_clone = Arc::clone(&last_viewport);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        render_concept_node,
        move |id| {
            *sel_clone.lock().unwrap() = id;
        },
        |_, _, _| (),
        |_, _| (),
        |_| (),
        move |vp| {
            *vp_clone.lock().unwrap() = vp;
        },
    );

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 800.0));

    // A. Geometri: Hent rektangler for det svævende panel og kontrollerne
    let panel_rect = DiagramCanvas::<
        (),
        DiagramNode,
        DiagramEdge,
        fn(&mut iced::widget::canvas::Frame, &DiagramNode, bool, CanvasViewport),
    >::floating_panel_rect(bounds);
    assert_eq!(panel_rect.width, 180.0);
    assert_eq!(panel_rect.height, 148.0);
    assert_eq!(panel_rect.x, 1000.0 - 180.0 - 16.0); // 804.0
    assert_eq!(panel_rect.y, 800.0 - 148.0 - 16.0); // 636.0

    let minimap_rect = DiagramCanvas::<
        (),
        DiagramNode,
        DiagramEdge,
        fn(&mut iced::widget::canvas::Frame, &DiagramNode, bool, CanvasViewport),
    >::minimap_rect(panel_rect);
    let zoom_in_rect = DiagramCanvas::<
        (),
        DiagramNode,
        DiagramEdge,
        fn(&mut iced::widget::canvas::Frame, &DiagramNode, bool, CanvasViewport),
    >::zoom_in_button_rect(panel_rect);
    let zoom_out_rect = DiagramCanvas::<
        (),
        DiagramNode,
        DiagramEdge,
        fn(&mut iced::widget::canvas::Frame, &DiagramNode, bool, CanvasViewport),
    >::zoom_out_button_rect(panel_rect);
    let fit_rect = DiagramCanvas::<
        (),
        DiagramNode,
        DiagramEdge,
        fn(&mut iced::widget::canvas::Frame, &DiagramNode, bool, CanvasViewport),
    >::fit_view_button_rect(panel_rect);

    // B. Event isolation: Klik på kontrolpanelets baggrund (eller minimap) må IKKE vælge n_under_panel
    let click_panel_bg = Point::new(panel_rect.x + 10.0, panel_rect.y + 10.0);
    assert!(n_under_panel.contains(click_panel_bg.x, click_panel_bg.y));
    let press_event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let action = canvas.update(
        &mut state,
        &press_event,
        bounds,
        Cursor::Available(click_panel_bg),
    );
    assert!(action.is_some(), "Klik på kontrolpanelet skal captures");
    assert_eq!(
        *selected_node.lock().unwrap(),
        None,
        "Node under kontrolpanelet må IKKE blive valgt ved klik på panelet!"
    );
    let release_event = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
    let _ = canvas.update(
        &mut state,
        &release_event,
        bounds,
        Cursor::Available(click_panel_bg),
    );

    // C. Zoom In (+) kontrol
    let zoom_in_click = Point::new(
        zoom_in_rect.x + zoom_in_rect.width / 2.0,
        zoom_in_rect.y + zoom_in_rect.height / 2.0,
    );
    let action = canvas.update(
        &mut state,
        &press_event,
        bounds,
        Cursor::Available(zoom_in_click),
    );
    assert!(action.is_some());
    assert!(
        last_viewport.lock().unwrap().zoom() > 1.05,
        "Klik på zoom-in knap skal forøge viewport zoom!"
    );

    // D. Zoom Out (-) kontrol på næste frame med opdateret viewport
    let sel_c2 = Arc::clone(&selected_node);
    let vp_c2 = Arc::clone(&last_viewport);
    let canvas_zoomed = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        *last_viewport.lock().unwrap(),
        false,
        render_concept_node,
        move |id| {
            *sel_c2.lock().unwrap() = id;
        },
        |_, _, _| (),
        |_, _| (),
        |_| (),
        move |vp| {
            *vp_c2.lock().unwrap() = vp;
        },
    );
    let zoom_out_click = Point::new(
        zoom_out_rect.x + zoom_out_rect.width / 2.0,
        zoom_out_rect.y + zoom_out_rect.height / 2.0,
    );
    let action = canvas_zoomed.update(
        &mut state,
        &press_event,
        bounds,
        Cursor::Available(zoom_out_click),
    );
    assert!(action.is_some());
    assert!(
        (last_viewport.lock().unwrap().zoom() - 1.0).abs() < 0.05,
        "Klik på zoom-out knap skal reducere viewport zoom tilbage!"
    );

    // E. Fit to View (⊡) kontrol
    let fit_click = Point::new(
        fit_rect.x + fit_rect.width / 2.0,
        fit_rect.y + fit_rect.height / 2.0,
    );
    let action = canvas.update(
        &mut state,
        &press_event,
        bounds,
        Cursor::Available(fit_click),
    );
    assert!(
        action.is_some(),
        "Klik på fit-view knap skal udløse viewport opdatering"
    );

    // F. Minimap interaktion: Klik i minimappet skal panorere viewporten
    let minimap_click = Point::new(
        minimap_rect.x + minimap_rect.width * 0.25,
        minimap_rect.y + minimap_rect.height * 0.25,
    );
    let action = canvas.update(
        &mut state,
        &press_event,
        bounds,
        Cursor::Available(minimap_click),
    );
    assert!(
        action.is_some(),
        "Klik i minimap skal captures og udløse pan"
    );
    assert!(
        state.is_panning_minimap,
        "Minimap panning skal være aktiv under træk"
    );

    // Cursor move i minimap
    let minimap_drag = Point::new(
        minimap_rect.x + minimap_rect.width * 0.75,
        minimap_rect.y + minimap_rect.height * 0.75,
    );
    let move_event = Event::Mouse(mouse::Event::CursorMoved {
        position: minimap_drag,
    });
    let action = canvas.update(
        &mut state,
        &move_event,
        bounds,
        Cursor::Available(minimap_drag),
    );
    assert!(
        action.is_some(),
        "Træk i minimap skal opdatere viewport kontinuerligt"
    );

    // Slip musen
    let _ = canvas.update(
        &mut state,
        &release_event,
        bounds,
        Cursor::Available(minimap_drag),
    );
    assert!(
        !state.is_panning_minimap,
        "Minimap panning skal deaktiveres ved slip"
    );

    // G. Test i fuld App-kontekst for både Begrebsmodel og Informationsmodel
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_task023_{}.fda", Uuid::new_v4()));
    let mut app = App::new_with_path(Some(file_path.clone()));
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.view();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.view();

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_task024_desktop_menu_bar_and_sidebar_toggle() {
    use kant::ui::app::MenuType;

    let mut app = App::new_with_path(None);

    // 1. Initial tilstand
    assert_eq!(
        app.active_menu(),
        None,
        "Ingen menu skal være åben ved start"
    );
    assert!(
        app.is_left_sidebar_visible(),
        "Venstre sidebar skal være synlig som standard"
    );

    // 2. Test Sidebar Toggle
    let _ = app.update(Message::ToggleLeftSidebar);
    assert!(
        !app.is_left_sidebar_visible(),
        "Sidebar skal være skjult efter ToggleLeftSidebar"
    );
    let _ = app.update(Message::ToggleLeftSidebar);
    assert!(
        app.is_left_sidebar_visible(),
        "Sidebar skal være synlig igen efter ToggleLeftSidebar"
    );

    // 3. Test Filer og Hjælp menuer
    let _ = app.update(Message::ToggleMenu(MenuType::File));
    assert_eq!(app.active_menu(), Some(MenuType::File));

    let _ = app.update(Message::ToggleMenu(MenuType::Help));
    assert_eq!(app.active_menu(), Some(MenuType::Help));

    let _ = app.update(Message::CloseMenu);
    assert_eq!(app.active_menu(), None);

    // 4. Test Nyt Projekt onboarding
    let _ = app.update(Message::ToggleMenu(MenuType::File));
    assert_eq!(app.active_menu(), Some(MenuType::File));

    let _ = app.update(Message::NewProject);
    assert_eq!(
        app.active_menu(),
        None,
        "NewProject skal automatisk lukke menuen"
    );
    assert!(
        app.metadata_modal().is_some(),
        "NewProject skal automatisk åbne Modelomslag & Metadata"
    );

    // 5. Test Escape lukker aktiv menu
    let _ = app.update(Message::CloseMetadataModal);
    let _ = app.update(Message::ToggleMenu(MenuType::File));
    assert_eq!(app.active_menu(), Some(MenuType::File));
    let _ = app.update(Message::EscapePressed);
    assert_eq!(app.active_menu(), None);

    // 6. Test rendering af UI med menu åben og sidebar sammenklappet
    let _ = app.update(Message::ToggleLeftSidebar);
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.view();
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.view();

    let _ = app.update(Message::ToggleMenu(MenuType::File));
    let _ = app.view();
    let _ = app.update(Message::ToggleMenu(MenuType::Help));
    let _ = app.view();
}

#[test]
fn test_task_030_canvas_ergonomics_and_edge_geometry() {
    let mut app = App::new_with_path(None);

    // 1. Opret kilde- og målbegreb samt relation
    let c1 = Concept::new("Person", "En person", BelongsToDomain::Yes);
    let c2 = Concept::new("Kunde", "En kunde", BelongsToDomain::Yes);
    let id1 = app.project_mut().add_concept(c1.clone()).unwrap();
    let id2 = app.project_mut().add_concept(c2).unwrap();
    app.project_mut().populate_missing_concepts_to_graph();
    let node1 = app
        .project()
        .concept_graph()
        .find_node_by_concept(id1)
        .unwrap()
        .id();
    let node2 = app
        .project()
        .concept_graph()
        .find_node_by_concept(id2)
        .unwrap()
        .id();
    app.project_mut()
        .concept_graph_mut()
        .add_relation(node2, node1, RelationKind::Generalization);

    // Vælg relationen
    let _ = app.update(Message::GraphEdgeSelected(Some((node2, node1))));
    assert_eq!(app.selected_edge(), Some((node2, node1)));

    // AC5: Enkeltklik på canvas (GraphNodeSelected(None)) deaktiverer valgt relation
    let _ = app.update(Message::GraphNodeSelected(None));
    assert_eq!(
        app.selected_edge(),
        None,
        "Valgt relation skal fravælges ved klik på tomt canvas"
    );

    // Samme for Informationsmodellen
    let _ = app.update(Message::CreateInformationClass);
    let cls_id = app.project().information_model().classes()[0].id();
    let info_node = app
        .project()
        .information_graph()
        .find_node_by_class(cls_id)
        .unwrap()
        .id();
    let _ = app.update(Message::InfoEdgeSelected(Some((info_node, info_node))));
    assert!(app.selected_info_edge().is_some());
    let _ = app.update(Message::SelectInfoGraphNode(None));
    assert_eq!(
        app.selected_info_edge(),
        None,
        "Valgt info relation skal fravælges ved klik på tomt canvas"
    );

    // AC2 & AC3: Opret klasse på koordinater (CreateInformationClassAt)
    let _ = app.update(Message::CreateInformationClassAt(460.0, 320.0));
    let class_to_delete_id = {
        let classes = app.project().information_model().classes();
        let new_cls = classes.last().unwrap();
        let new_node = app
            .project()
            .information_graph()
            .find_node_by_class(new_cls.id())
            .unwrap();
        // Skal være placeret præcist på (460, 320) og ikke i fast modulo-gitter
        assert_eq!(new_node.x(), 460.0);
        assert_eq!(new_node.y(), 320.0);
        new_cls.id()
    };

    // AC2: CreateConceptAtCenter & CreateInformationClassAtCenter
    let _ = app.update(Message::CreateConceptAtCenter);
    assert!(
        app.selected_graph_node_id().is_some() && app.is_node_editing(),
        "CreateConceptAtCenter skal oprette begreb på canvas og åbne Egenskaber"
    );
    let _ = app.update(Message::CancelConceptEdit);

    // AC6: Permanent sletning af begreb og klasse fra paletten
    // Fjern node fra diagrammet (men bevar i model repository)
    app.project_mut().concept_graph_mut().remove_node(node2);
    assert!(!app.project().concept_graph().is_concept_on_diagram(id2));
    assert!(app.project().get_concept(id2).is_some());
    // Slet permanent via DeleteConcept (svarende til klik på skraldespand i paletten)
    let _ = app.update(Message::DeleteConcept(id2));
    assert!(
        app.project().get_concept(id2).is_none(),
        "Begrebet skal slettes permanent fra projektet"
    );

    // Samme for Informationsmodellen
    app.project_mut()
        .information_graph_mut()
        .remove_class_node(class_to_delete_id);
    assert!(!app
        .project()
        .information_graph()
        .is_class_on_diagram(class_to_delete_id));
    assert!(app
        .project()
        .information_model()
        .get_class(class_to_delete_id)
        .is_some());
    // Slet permanent via DeleteInformationClass
    let _ = app.update(Message::DeleteInformationClass(class_to_delete_id));
    assert!(
        app.project()
            .information_model()
            .get_class(class_to_delete_id)
            .is_none(),
        "Klassen skal slettes permanent"
    );

    // AC4: Rute knæk-symmetri:
    // To noder placeret under en forældre-node skal have identisk mid_y knækhøjde uanset om relationen er Generalisering eller Komposition
    use kant::features::concept_model::{DiagramEdge, DiagramNode};
    let parent = DiagramNode::new(&c1, 300.0, 50.0);
    let child_gen = DiagramNode::new(
        &Concept::new("SubGen", "def", BelongsToDomain::Yes),
        100.0,
        250.0,
    );
    let child_comp = DiagramNode::new(
        &Concept::new("SubComp", "def", BelongsToDomain::Yes),
        500.0,
        250.0,
    );
    let edge_g = DiagramEdge::new(child_gen.id(), parent.id(), RelationKind::Generalization);
    let edge_c = DiagramEdge::new(parent.id(), child_comp.id(), RelationKind::Composition);

    let test_nodes = vec![parent.clone(), child_gen.clone(), child_comp.clone()];
    let test_edges = vec![edge_g, edge_c];
    let routed = kant::ui::edge_router::EdgeRouter::route_edges(&test_nodes, &test_edges);
    assert_eq!(routed.len(), 2);
    let route_gen = routed
        .iter()
        .find(|r| r.kind == RelationKind::Generalization)
        .unwrap();
    let route_comp = routed
        .iter()
        .find(|r| r.kind == RelationKind::Composition)
        .unwrap();
    let gen_mid_y = route_gen.points[1].y;
    let comp_mid_y = route_comp.points[1].y;
    assert_eq!(
        gen_mid_y, comp_mid_y,
        "Knækhøjden for generalisering og komposition skal flugte snorlige: gen={}, comp={}",
        gen_mid_y, comp_mid_y
    );
}

#[test]
fn test_task_031_fda_information_class_properties_and_rendering() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::InformationClass;
    use kant::ui::app::{App, Message};

    // 1. AC1: Standard initialisering af InformationClass
    let mut class = InformationClass::new("Køretøj");
    assert_eq!(class.name(), "Køretøj");
    assert!(!class.is_abstract(), "Standard skal være ikke-abstrakt");
    assert!(class.is_local(), "Standard skal være lokal klasse");
    assert_eq!(
        class.origin_model(),
        None,
        "Standard har ingen oprindelsesmodel"
    );

    class.set_abstract(true);
    assert!(class.is_abstract());
    class.set_local(false);
    assert!(!class.is_local());
    class.set_origin_model(Some("https://data.gov.dk/model/cpr".to_string()));
    assert_eq!(class.origin_model(), Some("https://data.gov.dk/model/cpr"));

    // 2. AC2: Arv fra Concept::from_concept med forskellige BelongsToDomain
    let local_concept = Concept::new("Cykel", "def", BelongsToDomain::Yes);
    let class_from_local = InformationClass::from_concept(&local_concept);
    assert!(class_from_local.is_local());
    assert_eq!(class_from_local.origin_model(), None);

    let foreign_concept = Concept::new("Kunde", "def", BelongsToDomain::No);
    let class_from_foreign = InformationClass::from_concept(&foreign_concept);
    assert!(!class_from_foreign.is_local());
    assert_eq!(class_from_foreign.origin_model(), None);

    let ref_concept = Concept::new(
        "Person",
        "def",
        BelongsToDomain::ModelRef("https://data.gov.dk/model/cpr".to_string()),
    );
    let class_from_ref = InformationClass::from_concept(&ref_concept);
    assert!(!class_from_ref.is_local());
    assert_eq!(
        class_from_ref.origin_model(),
        Some("https://data.gov.dk/model/cpr")
    );

    // 3. AC3: App Message håndtering for abstrakte og lokale/fremmede klasser
    let mut app = App::new_with_path(None);
    let cid = class_from_local.id();
    app.project_mut()
        .information_model_mut()
        .classes_mut()
        .push(class_from_local);

    let _ = app.update(Message::SetInformationClassAbstract(cid, true));
    let cls = app.project().information_model().get_class(cid).unwrap();
    assert!(cls.is_abstract(), "Klassen skal nu være abstrakt");

    let _ = app.update(Message::SetInformationClassLocal(cid, false));
    let cls = app.project().information_model().get_class(cid).unwrap();
    assert!(!cls.is_local(), "Klassen skal nu være fremmed/lånt");

    let _ = app.update(Message::SetInformationClassOriginModel(
        cid,
        "https://data.gov.dk/model/vej".to_string(),
    ));
    let cls = app.project().information_model().get_class(cid).unwrap();
    assert_eq!(
        cls.origin_model(),
        Some("https://data.gov.dk/model/vej"),
        "Kildemodel skal være opdateret"
    );

    // 4. AC1 Serde Bagudkompatibilitet: Ældre JSON uden is_abstract/is_local felter
    let legacy_json = r#"{
        "id": "a1b2c3d4-e5f6-4a5b-8c9d-0e1f2a3b4c5d",
        "name": "HistoriskKlasse",
        "concept_ids": [],
        "attributes": []
    }"#;
    let deserialized: InformationClass =
        serde_json::from_str(legacy_json).expect("Legacy JSON skal deserialisere fejlfrit");
    assert_eq!(deserialized.name(), "HistoriskKlasse");
    assert!(
        !deserialized.is_abstract(),
        "Standard is_abstract skal være false"
    );
    assert!(deserialized.is_local(), "Standard is_local skal være true");
    assert_eq!(deserialized.origin_model(), None);
}

#[test]
fn test_task033_rebranding_application_to_kant_defaults_and_compatibility() {
    use kant::features::model::storage::ProjectStorage;
    use kant::ui::file_dialog::scan_local_project_files;
    use std::path::PathBuf;

    // 1. Standard filsti skal være model.kant.json
    assert_eq!(
        ProjectStorage::default_project_path(),
        PathBuf::from("model.kant.json"),
        "Standard filnavn for persistens skal være model.kant.json"
    );

    // 2. Scan skal finde både .kant.json og .edge.json for bagudkompatibilitet
    let temp_dir = std::env::temp_dir().join(format!("kant_compat_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let kant_file = temp_dir.join("projekt.kant.json");
    let edge_file = temp_dir.join("gammelt_projekt.edge.json");
    let other_file = temp_dir.join("notat.txt");

    std::fs::write(&kant_file, "{}").unwrap();
    std::fs::write(&edge_file, "{}").unwrap();
    std::fs::write(&other_file, "text").unwrap();

    let scanned = scan_local_project_files(&temp_dir);
    assert!(scanned.contains(&kant_file), "Skal finde .kant.json");
    assert!(
        scanned.contains(&edge_file),
        "Skal finde ældre .edge.json for bagudkompatibilitet"
    );
    assert!(
        !scanned.contains(&other_file),
        "Skal ignorere irrelevante filer"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task036_concept_editor_domain_toggle_and_model_ref() {
    use kant::features::concepts::BelongsToDomain;
    use kant::ui::concept_editor::ConceptFormField;

    let mut app = App::new_with_path(None);

    // 1. Åbn editor via StartNewConcept
    let _ = app.update(Message::StartNewConcept);
    assert!(app.concept_editor().is_some());

    // 2. Standard tilstand er lokalt begreb (BelongsToDomain::Yes)
    let editor = app.concept_editor().unwrap();
    assert_eq!(editor.belongs_to_domain, BelongsToDomain::Yes);
    assert!(editor.model_uri.is_empty());

    // 3. Skift til Indlånt begreb (Nej) via SetConceptDomain
    let _ = app.update(Message::SetConceptDomain(BelongsToDomain::No));
    let editor = app.concept_editor().unwrap();
    assert_eq!(editor.belongs_to_domain, BelongsToDomain::No);

    // 4. Udfyld felter inkl. kildemodel URI
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "Person".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "Et menneske med borgerrettigheder.".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::ModelUri,
        "https://data.gov.dk/model/core/cpr".to_string(),
    ));

    // 5. Gem begreb og verificer at det gemmes som BelongsToDomain::ModelRef
    let _ = app.update(Message::SaveConcept);
    assert!(!app.is_editing_concept());
    let saved_concept = app
        .project()
        .concepts()
        .iter()
        .find(|c| c.preferred_term() == "Person")
        .unwrap();
    assert_eq!(
        saved_concept.belongs_to_domain(),
        &BelongsToDomain::ModelRef("https://data.gov.dk/model/core/cpr".to_string()),
        "Indlånt begreb med model URI skal gemmes som ModelRef"
    );

    // 6. Genåbn begrebet i editoren og verificer at kildemodel URI bevares
    let _ = app.update(Message::EditConcept(saved_concept.id()));
    let editor = app.concept_editor().unwrap();
    assert_eq!(editor.belongs_to_domain, BelongsToDomain::No);
    assert_eq!(editor.model_uri, "https://data.gov.dk/model/core/cpr");

    // 7. Skift tilbage til lokalt begreb og gem -> skal blive BelongsToDomain::Yes
    let _ = app.update(Message::SetConceptDomain(BelongsToDomain::Yes));
    let _ = app.update(Message::SaveConcept);
    let updated_concept = app
        .project()
        .concepts()
        .iter()
        .find(|c| c.preferred_term() == "Person")
        .unwrap();
    assert_eq!(
        updated_concept.belongs_to_domain(),
        &BelongsToDomain::Yes,
        "Når skiftet til lokalt begreb, skal det gemmes som BelongsToDomain::Yes"
    );
}

#[test]
fn test_task037_windows_native_integration_and_rendering() {
    use kant::ui::app::load_window_icon;
    use std::path::Path;

    // 1. Verificer at kant.ico findes og er en gyldig Windows .ico fil med de korrekte magic bytes
    let ico_path = Path::new("assets/icons/kant.ico");
    assert!(
        ico_path.exists(),
        "assets/icons/kant.ico skal eksistere for Windows PE indlejring"
    );
    let ico_bytes = std::fs::read(ico_path).expect("skal kunne læse kant.ico");
    assert!(
        ico_bytes.len() >= 6,
        "ICO fil skal have mindst 6 bytes header"
    );
    assert_eq!(
        &ico_bytes[0..4],
        &[0x00, 0x00, 0x01, 0x00],
        "ICO magic bytes skal matche [0, 0, 1, 0]"
    );

    // 2. Verificer at kant-32.rgba findes og er præcis 32*32*4 = 4096 bytes
    let rgba_path = Path::new("assets/icons/kant-32.rgba");
    assert!(
        rgba_path.exists(),
        "assets/icons/kant-32.rgba skal eksistere for runtime vinduesikon"
    );
    let rgba_bytes = std::fs::read(rgba_path).expect("skal kunne læse kant-32.rgba");
    assert_eq!(
        rgba_bytes.len(),
        32 * 32 * 4,
        "kant-32.rgba skal have præcis 4096 bytes"
    );

    // 3. Verificer at load_window_icon() returnerer et gyldigt iced::window::Icon
    let icon = load_window_icon();
    assert!(
        icon.is_some(),
        "load_window_icon() skal returnere Some(Icon)"
    );

    // 4. Verificer at build.rs findes og indeholder winres konfiguration for Windows
    let build_rs = Path::new("build.rs");
    assert!(build_rs.exists(), "build.rs skal eksistere");
    let build_rs_content = std::fs::read_to_string(build_rs).expect("skal kunne læse build.rs");
    assert!(
        build_rs_content.contains("winres"),
        "build.rs skal referere winres"
    );
    assert!(
        build_rs_content.contains("kant.ico"),
        "build.rs skal sætte kant.ico som ressource"
    );
}

#[test]
fn test_task_038_concept_list_alphabetical_sorting() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::App;

    // 1. Initialiser App og tilføj begreber i ikke-alfabetisk rækkefølge
    let mut app = App::new_with_path(None);
    // Ryd eventuelle standardbegreber
    while let Some(c) = app.project().concepts().first().cloned() {
        app.project_mut().remove_concept(c.id());
    }

    let c_zebra = Concept::new("Zebra", "Et stribet dyr", BelongsToDomain::Yes);
    let c_abe = Concept::new("Abe", "En primat", BelongsToDomain::Yes);
    let c_kamel = Concept::new("Kamel", "Et ørkendyr med pukler", BelongsToDomain::Yes);
    let c_baad = Concept::new("Båd", "Et fartøj på vandet", BelongsToDomain::Yes);

    app.project_mut()
        .add_concept(c_zebra)
        .expect("skal tilføje Zebra");
    app.project_mut()
        .add_concept(c_abe)
        .expect("skal tilføje Abe");
    app.project_mut()
        .add_concept(c_kamel)
        .expect("skal tilføje Kamel");
    app.project_mut()
        .add_concept(c_baad)
        .expect("skal tilføje Båd");

    // 2. Verificer at App::filtered_concepts() returnerer begreberne alfabetisk sorteret
    let terms: Vec<&str> = app
        .filtered_concepts()
        .iter()
        .map(|c| c.preferred_term())
        .collect();

    assert_eq!(
        terms,
        vec!["Abe", "Båd", "Kamel", "Zebra"],
        "filtered_concepts() skal altid returnere begreber alfabetisk sorteret"
    );

    // 3. Tilføj et nyt begreb 'Delfin' og verificer at det indsættes korrekt midt i listen
    let c_delfin = Concept::new("Delfin", "Et havpattedyr", BelongsToDomain::Yes);
    app.project_mut()
        .add_concept(c_delfin)
        .expect("skal tilføje Delfin");

    let updated_terms: Vec<&str> = app
        .filtered_concepts()
        .iter()
        .map(|c| c.preferred_term())
        .collect();

    assert_eq!(
        updated_terms,
        vec!["Abe", "Båd", "Delfin", "Kamel", "Zebra"],
        "Nyt begreb skal automatisk fremgå på sin alfabetiske plads i Begrebslisten"
    );
}

#[test]
fn test_task_039_canvas_class_text_cutoff_and_inspector_focus() {
    use kant::features::information_model::DEFAULT_CLASS_NODE_WIDTH;
    use kant::ui::app::{App, Message};
    use kant::ui::diagram_canvas::truncate_with_ellipsis;

    // 1. Verificer at klassens faste bredde er bevaret på 220.0 px
    assert_eq!(
        DEFAULT_CLASS_NODE_WIDTH, 220.0,
        "Klassens bredde skal forblive fast på 220.0 px"
    );

    // 2. Verificer tekstafkortning med ellipsis
    let long_attr =
        "+ overensstemmelseserklaeringType : OverensstemmelseserklaeringTypeValue [0..1] 🔗";
    let truncated = truncate_with_ellipsis(long_attr, 26);
    assert!(
        truncated.chars().count() <= 26,
        "Afkortet streng må ikke overstige max_chars (26)"
    );
    assert!(
        truncated.ends_with("..."),
        "Afkortet streng skal afsluttes med '...'"
    );

    let short_attr = "+ id : Integer [1]";
    assert_eq!(
        truncate_with_ellipsis(short_attr, 26),
        short_attr,
        "Korte strenge må ikke afkortes"
    );

    // 3. Verificer at oprettelse af informationsklasse vælges og gør klar til fokus
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::CreateInformationClassAt(200.0, 300.0));
    assert!(
        app.selected_info_class_id().is_some(),
        "Nyoprettet informationsklasse skal være valgt i Egenskabs-panelet"
    );

    // 4. Verificer at oprettelse af begreb på Begrebsmodel opretter direkte på canvas og åbner Egenskaber
    let _ = app.update(Message::CreateConceptAtCenter);
    assert!(
        app.selected_graph_node_id().is_some(),
        "Nyt begreb skal oprettes på canvas og være valgt"
    );
    assert!(
        app.is_node_editing(),
        "Egenskabs-panelet skal åbnes i hurtigredigering med fokus på foretrukken term"
    );
}

#[test]
fn test_task_044_decomposed_storage_and_deterministic_serialization() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::{
        Attribute, InformationClass, Multiplicity, PrimitiveType,
    };
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_044_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Byg et modelprojekt med metadata, begreber, klasser og attributter
    let metadata = ModelMetadata::new(
        "Køretøjsmodellen Decomposed",
        "Test for dekomponeret format",
        "https://data.gov.dk/model/vehicle",
        "Motorstyrelsen",
        "Transport",
        "1.0.0",
        ModelStatus::Draft,
    );
    let mut project = ModelProject::new(metadata);

    let mut c1 = Concept::new(
        "Køretøj",
        "Et motordrevet transportmiddel",
        BelongsToDomain::Yes,
    );
    c1.set_source(Some("Færdselsloven".to_string()));
    let c1_id = project.add_concept(c1).unwrap();

    let c2 = Concept::new(
        "Vejafgift",
        "Gebyr for passage ad vej",
        BelongsToDomain::Yes,
    );
    let c2_id = project.add_concept(c2).unwrap();

    let mut cls1 = InformationClass::new("Køretøj");
    cls1.add_concept_id(c1_id);
    let attr1 = Attribute::new(
        "stelnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    );
    cls1.add_attribute(attr1);
    let cls1_id = project.information_model_mut().add_class(cls1);

    // 2. Gem til dekomponeret mappe (.kant/)
    ProjectStorage::save_to_directory(&project, &temp_dir).expect("save_to_directory skal lykkes");

    let kant_dir = temp_dir.join(".kant");
    assert!(kant_dir.is_dir(), ".kant/ mappen skal eksistere");
    assert!(
        kant_dir.join("metadata.json").is_file(),
        "metadata.json skal eksistere"
    );
    assert!(
        kant_dir
            .join("concepts")
            .join(format!("{}.json", c1_id))
            .is_file(),
        "c1 fil skal eksistere"
    );
    assert!(
        kant_dir
            .join("concepts")
            .join(format!("{}.json", c2_id))
            .is_file(),
        "c2 fil skal eksistere"
    );
    assert!(
        kant_dir
            .join("classes")
            .join(format!("{}.json", cls1_id))
            .is_file(),
        "cls1 fil skal eksistere"
    );
    assert!(
        kant_dir.join("diagrams").is_dir(),
        "diagrams/ mappen skal eksistere"
    );

    // 3. Deterministisk formatering (idempotens)
    let meta_content_first = std::fs::read_to_string(kant_dir.join("metadata.json")).unwrap();
    let c1_content_first =
        std::fs::read_to_string(kant_dir.join("concepts").join(format!("{}.json", c1_id))).unwrap();

    // Gem igen uden ændringer
    ProjectStorage::save_to_directory(&project, &temp_dir).expect("Anden gemning skal lykkes");
    let meta_content_second = std::fs::read_to_string(kant_dir.join("metadata.json")).unwrap();
    let c1_content_second =
        std::fs::read_to_string(kant_dir.join("concepts").join(format!("{}.json", c1_id))).unwrap();

    assert_eq!(
        meta_content_first, meta_content_second,
        "metadata.json skal være bit-for-bit idempotent"
    );
    assert_eq!(
        c1_content_first, c1_content_second,
        "begrebsfil skal være bit-for-bit idempotent"
    );

    // 4. Indlæs igen og bekræft roundtrip
    let loaded =
        ProjectStorage::load_from_directory(&temp_dir).expect("load_from_directory skal lykkes");
    assert_eq!(loaded.metadata().name(), "Køretøjsmodellen Decomposed");
    assert_eq!(loaded.concepts().len(), 2);
    assert_eq!(
        loaded.get_concept(c1_id).unwrap().preferred_term(),
        "Køretøj"
    );
    assert_eq!(
        loaded.get_concept(c2_id).unwrap().preferred_term(),
        "Vejafgift"
    );
    assert_eq!(loaded.information_model().classes().len(), 1);
    assert_eq!(
        loaded
            .information_model()
            .get_class(cls1_id)
            .unwrap()
            .name(),
        "Køretøj"
    );

    // 5. Test automatisk formatdetektion via ProjectStorage::load
    let loaded_from_root =
        ProjectStorage::load(&temp_dir).expect("load på root dir skal detektere .kant/");
    assert_eq!(
        loaded_from_root.metadata().name(),
        "Køretøjsmodellen Decomposed"
    );
    let loaded_from_dot_kant =
        ProjectStorage::load(&kant_dir).expect("load direkte på .kant/ skal lykkes");
    assert_eq!(
        loaded_from_dot_kant.metadata().name(),
        "Køretøjsmodellen Decomposed"
    );

    // 6. Test synkroniseret oprydning (sletning af c2)
    project.remove_concept(c2_id);
    ProjectStorage::save_to_directory(&project, &temp_dir)
        .expect("Gemning efter sletning skal lykkes");
    assert!(
        !kant_dir
            .join("concepts")
            .join(format!("{}.json", c2_id))
            .exists(),
        "Slettet begreb må ikke efterlade zombie-fil i concepts/"
    );
    let loaded_after_delete = ProjectStorage::load_from_directory(&temp_dir).unwrap();
    assert_eq!(loaded_after_delete.concepts().len(), 1);

    // Oprydning
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_045_semantic_three_way_model_merge() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::{
        Attribute, InformationClass, Multiplicity, PrimitiveType,
    };
    use kant::features::model::merge::merge_models;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};

    // 1. Opret Base model (fælles forfader)
    let base_meta = ModelMetadata::new(
        "Trafikmodel",
        "Base beskrivelse",
        "https://data.gov.dk/model/traffic",
        "Transportministeriet",
        "Trafik",
        "1.0.0",
        ModelStatus::Draft,
    );
    let mut base = ModelProject::new(base_meta);

    let c1 = Concept::new("Køretøj", "Et transportmiddel", BelongsToDomain::Yes);
    let c1_id = base.add_concept(c1.clone()).unwrap();

    let c2 = Concept::new("Vejafgift", "Gebyr for vej", BelongsToDomain::Yes);
    let c2_id = base.add_concept(c2.clone()).unwrap();

    let mut cls1 = InformationClass::new("Køretøj");
    cls1.add_concept_id(c1_id);
    let attr1 = Attribute::new("id", PrimitiveType::Integer, Multiplicity::exactly_one());
    let attr1_id = attr1.id();
    cls1.add_attribute(attr1);
    let cls1_id = base.information_model_mut().add_class(cls1);

    // 2. Opret Ours (lokale ændringer)
    let mut ours = base.clone();
    // Ændrer definition på c1
    let mut c1_ours = ours.get_concept(c1_id).unwrap().clone();
    c1_ours.set_definition("Et motordrevet transportmiddel");
    ours.update_concept(c1_ours).unwrap();

    // Ændrer definition på c2 til en lokal værdi
    let mut c2_ours = ours.get_concept(c2_id).unwrap().clone();
    c2_ours.set_definition("Lokal definition af afgift");
    ours.update_concept(c2_ours).unwrap();

    // Tilføjer c3 lokalt
    let c3 = Concept::new(
        "Færdselsregel",
        "Regel for adfærd i trafikken",
        BelongsToDomain::Yes,
    );
    let c3_id = ours.add_concept(c3).unwrap();

    // Tilføjer attribute 2 til cls1 lokalt
    let attr2 = Attribute::new(
        "stelnummer",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    );
    let attr2_id = attr2.id();
    ours.information_model_mut()
        .get_class_mut(cls1_id)
        .unwrap()
        .add_attribute(attr2);

    // 3. Opret Theirs (server ændringer)
    let mut theirs = base.clone();
    // Tilføjer source på c1 (bevarer base definition)
    let mut c1_theirs = theirs.get_concept(c1_id).unwrap().clone();
    c1_theirs.set_source(Some("Færdselsloven § 2".to_string()));
    theirs.update_concept(c1_theirs).unwrap();

    // Ændrer definition på c2 til en server værdi (MODSTRIDENDE)
    let mut c2_theirs = theirs.get_concept(c2_id).unwrap().clone();
    c2_theirs.set_definition("Server definition af afgift");
    theirs.update_concept(c2_theirs).unwrap();

    // Tilføjer c4 på serveren
    let c4 = Concept::new("Parkering", "Henstilling af køretøj", BelongsToDomain::Yes);
    let c4_id = theirs.add_concept(c4).unwrap();

    // Tilføjer attribute 3 til cls1 på serveren
    let attr3 = Attribute::new(
        "registreringsnummer",
        PrimitiveType::CharacterString,
        Multiplicity::zero_or_one(),
    );
    let attr3_id = attr3.id();
    theirs
        .information_model_mut()
        .get_class_mut(cls1_id)
        .unwrap()
        .add_attribute(attr3);

    // 4. Kør merge_models
    let outcome = merge_models(&base, &ours, &theirs);

    // 5. Valider konflikter
    assert_eq!(
        outcome.conflicts.len(),
        1,
        "Der skal være præcis 1 feltkonflikt på Concept 2 (definition)"
    );
    let conflict = &outcome.conflicts[0];
    assert_eq!(conflict.field_name, "definition");
    assert_eq!(conflict.base_value, "Gebyr for vej");
    assert_eq!(conflict.our_value, "Lokal definition af afgift");
    assert_eq!(conflict.their_value, "Server definition af afgift");

    // 6. Valider sammensmeltning af uafhængige felter (Concept 1)
    let merged_c1 = outcome.merged_project.get_concept(c1_id).unwrap();
    assert_eq!(
        merged_c1.definition(),
        "Et motordrevet transportmiddel",
        "Ours definition skal være bevaret i Concept 1"
    );
    assert_eq!(
        merged_c1.source(),
        Some("Færdselsloven § 2"),
        "Theirs kilde skal være flettet ind i Concept 1"
    );

    // 7. Valider uafhængige entiteter tilføjet på hver sin gren
    assert!(
        outcome.merged_project.get_concept(c3_id).is_some(),
        "Concept 3 oprettet i Ours skal findes i flettet model"
    );
    assert!(
        outcome.merged_project.get_concept(c4_id).is_some(),
        "Concept 4 oprettet i Theirs skal findes i flettet model"
    );
    assert_eq!(outcome.merged_project.concepts().len(), 4);

    // 8. Valider sammensmeltning af klasseattributter
    let merged_cls = outcome
        .merged_project
        .information_model()
        .get_class(cls1_id)
        .unwrap();
    assert_eq!(
        merged_cls.attributes().len(),
        3,
        "Klassen skal have alle 3 attributter: base (id), ours (stelnummer) og theirs (registreringsnummer)"
    );
    let attr_ids: Vec<_> = merged_cls.attributes().iter().map(|a| a.id()).collect();
    assert!(attr_ids.contains(&attr1_id));
    assert!(attr_ids.contains(&attr2_id));
    assert!(attr_ids.contains(&attr3_id));

    // 9. Valider idempotens: merge(base, ours, ours) skal have 0 konflikter
    let self_merge = merge_models(&base, &ours, &ours);
    assert!(
        self_merge.conflicts.is_empty(),
        "Merge af identiske grene skal have 0 konflikter"
    );
}

#[test]
fn test_fda_tabel_d_metadata_and_backward_compatibility() {
    use kant::features::model::{ApprovalStatus, ModelMetadata, ModelScope, ModelStatus};

    // 1. Opret ModelMetadata med samtlige FDA Tabel D felter (Kapitel 6)
    let mut meta = ModelMetadata::new(
        "Transportmidler",
        "Begrebsmodel over transportmidler jf. FDA vejledning",
        "https://data.gov.dk/concept/core/transportationMeans",
        "Trafik-, Bygge- og Boligstyrelsen",
        "59.02.05 Overordnede opgaver vedrørende trafik og transport",
        "1.0.0",
        ModelStatus::Completed,
    );

    // Sæt supplerende Tabel D felter
    meta.set_approval_status(ApprovalStatus::Approved);
    meta.set_approved_by(Some("Rådet for Sikker Trafik".to_string()));
    meta.set_model_scope(ModelScope::Core);
    meta.set_language("da");
    meta.set_date_modified("2026-09-30");
    meta.set_version_notes(Some("Første officielle udgave af modellen".to_string()));
    meta.set_source(Some("ISO 4210 Cycles".to_string()));
    meta.set_was_derived_from(Some("https://data.gov.dk/concept/core/vehicle".to_string()));
    meta.set_legal_sources(vec![
        "https://www.retsinformation.dk/eli/lta/2016/976".to_string(),
        "http://data.europa.eu/eli/reg/2013/168/oj".to_string(),
    ]);

    // Verificer getters
    assert_eq!(meta.model_status(), ModelStatus::Completed);
    assert_eq!(meta.approval_status(), ApprovalStatus::Approved);
    assert_eq!(meta.approved_by(), Some("Rådet for Sikker Trafik"));
    assert_eq!(meta.model_scope(), ModelScope::Core);
    assert_eq!(meta.language(), "da");
    assert_eq!(meta.date_modified(), "2026-09-30");
    assert_eq!(
        meta.version_notes(),
        Some("Første officielle udgave af modellen")
    );
    assert_eq!(meta.source(), Some("ISO 4210 Cycles"));
    assert_eq!(
        meta.was_derived_from(),
        Some("https://data.gov.dk/concept/core/vehicle")
    );
    assert_eq!(meta.legal_sources().len(), 2);
    assert_eq!(
        meta.legal_sources()[0],
        "https://www.retsinformation.dk/eli/lta/2016/976"
    );

    // 2. Roundtrip serialisering
    let json = serde_json::to_string_pretty(&meta).expect("Serialization skal lykkes");
    let deserialized: ModelMetadata =
        serde_json::from_str(&json).expect("Deserialization skal lykkes");
    assert_eq!(meta, deserialized);

    // 3. Test bagudkompatibel migration fra ældre model.kant.json format
    let legacy_json = r#"{
        "name": "Ældre Projekt",
        "description": "Model oprettet i ældre Kant version",
        "uri": "https://data.gov.dk/model/core/legacy",
        "responsible_org": "Myndighed",
        "domain_area": "Sundhed",
        "version": "0.2.0",
        "status": "Draft",
        "legal_source": "LBK nr 999 af 01/01/2020"
    }"#;

    let migrated: ModelMetadata =
        serde_json::from_str(legacy_json).expect("Legacy format skal migreres automatisk");
    assert_eq!(migrated.name(), "Ældre Projekt");
    assert_eq!(
        migrated.model_status(),
        ModelStatus::Development,
        "Gammelt Draft status skal migreres til ModelStatus::Development"
    );
    assert_eq!(
        migrated.approval_status(),
        ApprovalStatus::AwaitingApproval,
        "Gammelt Draft status skal migreres til ApprovalStatus::AwaitingApproval"
    );
    assert_eq!(
        migrated.model_scope(),
        ModelScope::Core,
        "Standard model_scope skal være Core"
    );
    assert_eq!(migrated.language(), "da", "Standard sprog skal være 'da'");
    assert_eq!(
        migrated.legal_sources(),
        &["LBK nr 999 af 01/01/2020".to_string()],
        "Gammelt enkeltstående legal_source felt skal migreres til legal_sources listen"
    );
}

#[test]
fn test_fda_uml_stereotypes_and_concept_canvas_relations() {
    use kant::features::concept_model::RelationKind;
    use kant::ui::diagram_canvas::{concept_stereotype_text, uml_class_stereotype_text};

    // 1. Stereotype på begrebslærred og UML-lærred skal være «Concept» jf. FDA Regel 03
    assert_eq!(concept_stereotype_text(), "«Concept»");
    assert_eq!(uml_class_stereotype_text(false), "«Concept»");
    assert_eq!(uml_class_stereotype_text(true), "«Concept» {abstract}");

    // 2. Begrebsrelationer (Tabel A) må kun indeholde Generalisering og Association
    let concept_rels = RelationKind::CONCEPT_RELATIONS;
    assert_eq!(concept_rels.len(), 2);
    assert!(concept_rels.contains(&RelationKind::Generalization));
    assert!(concept_rels.contains(&RelationKind::Association));
    assert!(!concept_rels.contains(&RelationKind::Composition));

    // 3. Informationsmodeller (Tabel B) skal fortsat understøtte Komposition og Dependency
    let all_rels = RelationKind::ALL;
    assert_eq!(all_rels.len(), 4);
    assert!(all_rels.contains(&RelationKind::Composition));
    assert!(all_rels.contains(&RelationKind::Dependency));
}

#[test]
fn test_fda_regler_20_21_22_aristotle_definition_linter() {
    use kant::features::concepts::{
        BelongsToDomain, Concept, ConceptValidator, DefinitionIssueKind, DefinitionLinter,
    };

    // 1. Aristoteles analyse (definitio per genus et differentiam)
    let analysis1 = DefinitionLinter::analyze_aristotle("køretøj der drives frem ved pedalkraft");
    assert_eq!(analysis1.genus_proximum.as_deref(), Some("køretøj"));
    assert_eq!(
        analysis1.differentia_specifica.as_deref(),
        Some("drives frem ved pedalkraft")
    );

    let analysis2 =
        DefinitionLinter::analyze_aristotle("person som er passager på et transportmiddel");
    assert_eq!(analysis2.genus_proximum.as_deref(), Some("person"));
    assert_eq!(
        analysis2.differentia_specifica.as_deref(),
        Some("er passager på et transportmiddel")
    );

    // 2. Formateringsfejl (Regel 20: lille begyndelsesbogstav og intet afsluttende punktum)
    let c_capital = Concept::new(
        "cykel",
        "Køretøj der drives frem ved pedalkraft",
        BelongsToDomain::Yes,
    );
    let issues_cap = DefinitionLinter::lint(&c_capital);
    assert!(
        issues_cap
            .iter()
            .any(|i| i.kind == DefinitionIssueKind::CapitalizedFirstLetter),
        "Definition med stort begyndelsesbogstav skal give advarsel"
    );

    let c_period = Concept::new(
        "cykel",
        "køretøj der drives frem ved pedalkraft.",
        BelongsToDomain::Yes,
    );
    let issues_per = DefinitionLinter::lint(&c_period);
    assert!(
        issues_per
            .iter()
            .any(|i| i.kind == DefinitionIssueKind::TrailingPeriod),
        "Definition med afsluttende punktum skal give advarsel"
    );

    // 3. Forbudte fyldfraser / indledninger (Regel 20)
    for phrase_def in &[
        "er et køretøj der drives frem ved pedalkraft",
        "defineres som et køretøj der drives frem ved pedalkraft",
        "betyder transportmiddel med pedaler",
        "henvisning til køretøj der drives frem ved pedalkraft",
        "angivelse af køretøjets art",
    ] {
        let c_phrase = Concept::new("cykel", *phrase_def, BelongsToDomain::Yes);
        let issues = DefinitionLinter::lint(&c_phrase);
        assert!(
            issues
                .iter()
                .any(|i| i.kind == DefinitionIssueKind::ForbiddenPrefixPhrase),
            "Definition startende med fyldfrase '{}' skal give advarsel",
            phrase_def
        );
    }

    // 4. Cirkularitetsdetektering (Regel 20)
    let mut c_circ = Concept::new(
        "cykel",
        "to-hjulet cykel der drives frem ved pedalkraft",
        BelongsToDomain::Yes,
    );
    let issues_circ = DefinitionLinter::lint(&c_circ);
    assert!(
        issues_circ
            .iter()
            .any(|i| i.kind == DefinitionIssueKind::CircularTermReference),
        "Definition der indeholder foretrukken term skal give cirkularitetsadvarsel"
    );

    // Cirkulær i forhold til accepteret synonym
    c_circ.set_definition("to-hjulet velocipede for personer");
    c_circ.set_accepted_term(Some("velocipede".to_string()));
    let issues_syn = DefinitionLinter::lint(&c_circ);
    assert!(
        issues_syn
            .iter()
            .any(|i| i.kind == DefinitionIssueKind::CircularAcceptedTermReference),
        "Definition der indeholder accepteret term (synonym) skal give advarsel"
    );

    // 5. Vage forbeholdsord (Regler 20 & 21)
    for vague_def in &[
        "køretøj der typisk har to hjul",
        "køretøj der normalt anvendes til personbefordring",
        "køretøj der ofte er udstyret med pedaler",
        "køretøj der som regel har kædetræk",
    ] {
        let c_vague = Concept::new("cykel", *vague_def, BelongsToDomain::Yes);
        let issues = DefinitionLinter::lint(&c_vague);
        assert!(
            issues
                .iter()
                .any(|i| i.kind == DefinitionIssueKind::VagueWord),
            "Definition med forbeholdsord '{}' skal give advarsel",
            vague_def
        );
    }

    // 6. Negative definitioner (Regel 20)
    for neg_def in &[
        "ikke-motoriseret køretøj med to hjul",
        "cykel der ikke er beregnet til voksne",
    ] {
        let c_neg = Concept::new("børnecykel", *neg_def, BelongsToDomain::Yes);
        let issues = DefinitionLinter::lint(&c_neg);
        assert!(
            issues
                .iter()
                .any(|i| i.kind == DefinitionIssueKind::NegativeDefinition),
            "Negativ definition '{}' skal give advarsel",
            neg_def
        );
    }

    // 7. En fuldstændig FDA-kompatibel definition giver 0 advarsler
    let valid_c = Concept::new(
        "cykel",
        "køretøj der drives frem ved pedalkraft",
        BelongsToDomain::Yes,
    );
    let clean_issues = DefinitionLinter::lint(&valid_c);
    assert!(
        clean_issues.is_empty(),
        "FDA-korrekt definition skal ikke give nogen advarsler, fandt: {:?}",
        clean_issues
    );

    // 8. Advarsler er vejledende og blokerer ikke for ConceptValidator (AC5)
    let dirty_c = Concept::new(
        "cykel",
        "Cykel er et køretøj der typisk drives frem ved pedalkraft.",
        BelongsToDomain::Yes,
    );
    assert!(
        ConceptValidator::validate(&dirty_c).is_ok(),
        "Linter-advarsler må IKKE blokere for validering og lagring af eksisterende begreber"
    );
}

#[test]
fn test_fda_controlled_vocabularies_enumerations_and_datatypes() {
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::{
        Attribute, InformationClass, InformationDataType, InformationEnumeration, InformationModel,
        Multiplicity, PrimitiveType, StructuredDataType,
    };
    use kant::ui::diagram_canvas::{datatype_keyword_text, enumeration_keyword_text};
    use kant::ui::theme::ThemeColors;

    let mut model = InformationModel::new();

    // 1. Opret enumeration for DrivkraftType med lowerCamelCase værdier jf. FDA Tabel B
    let enum_values = vec![
        "pedalkraft".to_string(),
        "elmotor".to_string(),
        "forbraendingsmotor".to_string(),
    ];
    let mut drivkraft_enum = InformationEnumeration::new("DrivkraftType", enum_values.clone());
    drivkraft_enum.set_definition(Some(
        "kontrolleret udfaldsrum over mulige fremdriftsformer for køretøjer".to_string(),
    ));
    assert_eq!(drivkraft_enum.name(), "DrivkraftType");
    assert_eq!(drivkraft_enum.values(), &enum_values);
    let drivkraft_id = model.add_enumeration(drivkraft_enum);
    assert_eq!(model.enumerations().len(), 1);
    assert_eq!(
        model.get_enumeration(drivkraft_id).map(|e| e.name()),
        Some("DrivkraftType")
    );

    // 2. Opret struktureret datatype for Stelnummer (Figur 5.18 i FDA vejledning)
    let mut stelnummer_dt = StructuredDataType::new("Stelnummer");
    stelnummer_dt.set_definition(Some(
        "sammensat identifikator for et køretøjs bærende stel".to_string(),
    ));
    stelnummer_dt.add_attribute(Attribute::new(
        "fabrikantmaerke",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    ));
    stelnummer_dt.add_attribute(Attribute::new(
        "serienummer",
        PrimitiveType::Integer,
        Multiplicity::exactly_one(),
    ));
    stelnummer_dt.add_attribute(Attribute::new(
        "aarstalsmaerke",
        PrimitiveType::CharacterString,
        Multiplicity::exactly_one(),
    ));
    assert_eq!(stelnummer_dt.name(), "Stelnummer");
    assert_eq!(stelnummer_dt.attributes().len(), 3);
    let stelnummer_id = model.add_structured_type(stelnummer_dt);
    assert_eq!(model.structured_types().len(), 1);
    assert_eq!(
        model.get_structured_type(stelnummer_id).map(|dt| dt.name()),
        Some("Stelnummer")
    );

    // 3. Opret klasse Cykel med attributter der refererer hhv. primitive, enum og struktureret type
    let mut cykel_class = InformationClass::new("Cykel");
    // Primitiv type via backward-compatible constructor
    cykel_class.add_attribute(Attribute::new(
        "antalHjul",
        PrimitiveType::Integer,
        Multiplicity::exactly_one(),
    ));
    // Enumeration reference
    cykel_class.add_attribute(Attribute::new(
        "drivkraft",
        InformationDataType::Enumeration {
            enumeration_id: drivkraft_id,
        },
        Multiplicity::exactly_one(),
    ));
    // Struktureret datatype reference
    cykel_class.add_attribute(Attribute::new(
        "rammenummer",
        InformationDataType::Structured {
            structured_id: stelnummer_id,
        },
        Multiplicity::exactly_one(),
    ));

    assert_eq!(cykel_class.attributes().len(), 3);
    assert_eq!(
        cykel_class.attributes()[0].data_type(),
        &InformationDataType::Primitive(PrimitiveType::Integer)
    );
    assert_eq!(
        cykel_class.attributes()[1].data_type(),
        &InformationDataType::Enumeration {
            enumeration_id: drivkraft_id
        }
    );
    assert_eq!(
        cykel_class.attributes()[2].data_type(),
        &InformationDataType::Structured {
            structured_id: stelnummer_id
        }
    );
    model.add_class(cykel_class);

    // 4. Dependency-relation (RelationKind::Dependency) for UML diagram lærred
    assert!(RelationKind::ALL.contains(&RelationKind::Dependency));
    assert!(RelationKind::Dependency.is_dependency());
    assert!(!RelationKind::Association.is_dependency());
    // Begrebsmodellen (Tabel A) må STADIG KUN have Generalization og Association
    assert!(!RelationKind::CONCEPT_RELATIONS.contains(&RelationKind::Dependency));

    // 5. UML Keywords & FDA-farver jf. Kapitel 5.2 & 5.5
    assert_eq!(enumeration_keyword_text(), "«enumeration»");
    assert_eq!(datatype_keyword_text(), "«dataType»");
    assert_eq!(
        ThemeColors::FDA_ENUM_GREEN,
        iced::Color::from_rgb(0.910, 0.992, 0.890)
    );
    assert_eq!(
        ThemeColors::FDA_DATA_TYPE_YELLOW,
        iced::Color::from_rgb(0.984, 0.976, 0.776)
    );

    // 6. Roundtrip serialisering & backward compatibility
    let json =
        serde_json::to_string_pretty(&model).expect("InformationModel skal kunne serialiseres");
    let deserialized: InformationModel =
        serde_json::from_str(&json).expect("InformationModel skal kunne deserialiseres");
    assert_eq!(model, deserialized);

    // Ældre JSON format med kun "classes" skal kunne indlæses med tomme enumerations og datatypes
    let legacy_json = r#"{
        "classes": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "name": "Køretøj",
                "attributes": [
                    {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "name": "registreringsnummer",
                        "data_type": "CharacterString",
                        "multiplicity": { "lower": 1, "upper": 1 }
                    }
                ]
            }
        ]
    }"#;
    let legacy_model: InformationModel = serde_json::from_str(legacy_json)
        .expect("Legacy InformationModel JSON skal deserialiseres");
    assert_eq!(legacy_model.classes().len(), 1);
    assert!(legacy_model.enumerations().is_empty());
    assert!(legacy_model.structured_types().is_empty());
    assert_eq!(
        legacy_model.classes()[0].attributes()[0].data_type(),
        &InformationDataType::Primitive(PrimitiveType::CharacterString)
    );
}

#[test]
fn test_task_062_windows_software_rendering_and_backend_resolution() {
    use kant::ui::platform::resolve_default_iced_backend;

    // AC1: På Windows uden sat ICED_BACKEND gennemtvinges ikke længere tiny-skia (Superceded by Task 065 / ADR 013)
    assert_eq!(
        resolve_default_iced_backend(true, None),
        None,
        "På Windows skal tiny-skia IKKE længere vælges som tvungen standard (ADR 013 / Task 065)"
    );

    // AC1: På Windows med eksplicit ICED_BACKEND skal den respekteres (ingen default overskrivning)
    assert_eq!(
        resolve_default_iced_backend(true, Some("wgpu")),
        None,
        "Hvis brugeren har sat ICED_BACKEND=wgpu, må default-funktionen ikke overskrive det"
    );

    // AC1: På ikke-Windows (Linux, macOS) skal der ikke gennemtvinges tiny-skia som default
    assert_eq!(
        resolve_default_iced_backend(false, None),
        None,
        "På Unix/macOS skal Iced have lov til at benytte systemets standard renderer"
    );
}

#[test]
fn test_task_061_naming_convention_linter_and_ui_feedback() {
    use kant::features::information_model::linter::{
        NamingConvention, NamingIssue, NamingIssueKind, NamingLinter, NamingTarget,
    };

    // AC1: UpperCamelCase Linter for Klasser
    assert!(NamingLinter::check_class_name("EthjuletCykel").is_none());
    assert!(NamingLinter::check_class_name("Køretøj").is_none());
    assert!(NamingLinter::check_class_name("Person").is_none());
    assert!(NamingLinter::check_class_name("CPR").is_none());

    let class_issue_snake =
        NamingLinter::check_class_name("ethjulet_cykel").expect("Skal fange snake_case på klasse");
    assert_eq!(class_issue_snake.target, NamingTarget::Class);
    assert_eq!(
        class_issue_snake.convention,
        NamingConvention::UpperCamelCase
    );
    assert_eq!(class_issue_snake.kind, NamingIssueKind::ContainsUnderscore);
    assert!(class_issue_snake.rule.contains("19"));
    assert_eq!(
        class_issue_snake.suggested_fix.as_deref(),
        Some("EthjuletCykel")
    );

    assert!(NamingLinter::check_enumeration_name("KøretøjType").is_none());
    assert!(NamingLinter::check_enumeration_name("køretøj_type").is_some());

    assert!(NamingLinter::check_structured_type_name("AdresseStruktur").is_none());
    assert!(NamingLinter::check_structured_type_name("adresse_struktur").is_some());

    let class_issue_space = NamingLinter::check_class_name("Ethjulet Cykel")
        .expect("Skal fange mellemrum i klassenavn");
    assert_eq!(
        class_issue_space.suggested_fix.as_deref(),
        Some("EthjuletCykel")
    );

    let class_issue_lower =
        NamingLinter::check_class_name("cykel").expect("Skal fange lille forbogstav i klasse");
    assert_eq!(class_issue_lower.suggested_fix.as_deref(), Some("Cykel"));

    // AC2: lowerCamelCase Linter for Attributter
    assert!(NamingLinter::check_attribute_name("stelnummer").is_none());
    assert!(NamingLinter::check_attribute_name("maxPassagerer").is_none());
    assert!(NamingLinter::check_attribute_name("førsteRegistrering").is_none());
    assert!(NamingLinter::check_attribute_name("cvrNummer").is_none());

    let attr_issue_upper: NamingIssue = NamingLinter::check_attribute_name("Stelnummer")
        .expect("Skal fange stort forbogstav på attribut");
    assert_eq!(attr_issue_upper.target, NamingTarget::Attribute);
    assert_eq!(
        attr_issue_upper.convention,
        NamingConvention::LowerCamelCase
    );
    assert_eq!(attr_issue_upper.kind, NamingIssueKind::NotLowerCamelCase);
    assert!(attr_issue_upper.rule.contains("19"));
    assert_eq!(
        attr_issue_upper.suggested_fix.as_deref(),
        Some("stelnummer")
    );

    let attr_issue_snake = NamingLinter::check_attribute_name("stel_nummer")
        .expect("Skal fange snake_case på attribut");
    assert_eq!(
        attr_issue_snake.suggested_fix.as_deref(),
        Some("stelNummer")
    );

    let attr_issue_space = NamingLinter::check_attribute_name("max passagerer")
        .expect("Skal fange mellemrum på attribut");
    assert_eq!(
        attr_issue_space.suggested_fix.as_deref(),
        Some("maxPassagerer")
    );

    // AC3: lowerCamelCase Linter for Associations-Labels
    assert!(NamingLinter::check_association_label("").is_none());
    assert!(NamingLinter::check_association_label("   ").is_none());
    assert!(NamingLinter::check_association_label("omfatter").is_none());
    assert!(NamingLinter::check_association_label("tilhørerKommune").is_none());

    let assoc_issue_upper = NamingLinter::check_association_label("Omfatter")
        .expect("Skal fange stort forbogstav på associationslabel");
    assert_eq!(assoc_issue_upper.target, NamingTarget::AssociationEnd);
    assert_eq!(
        assoc_issue_upper.convention,
        NamingConvention::LowerCamelCase
    );
    assert_eq!(assoc_issue_upper.suggested_fix.as_deref(), Some("omfatter"));

    let assoc_issue_snake = NamingLinter::check_association_label("omfatter_del")
        .expect("Skal fange snake_case på association");
    assert_eq!(
        assoc_issue_snake.suggested_fix.as_deref(),
        Some("omfatterDel")
    );

    // AC4: Ikke-blokerende for App / Model
    use kant::ui::app::{App, Message};
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::CreateInformationClass);
    let class_id = app.project().information_model().classes()[0].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_id,
        "ugyldig_klasse".into(),
    ));
    let classes = app.project().information_model().classes();
    let created_class = classes.iter().find(|c| c.name() == "ugyldig_klasse");
    assert!(
        created_class.is_some(),
        "Klassen skal fortsat kunne gemmes selvom navnet overtræder konventionen (ikke-blokerende linter)"
    );
}

#[test]
fn test_task_040_canvas_multi_node_selection_and_bulk_move() {
    use iced::mouse::{Button, Cursor};
    use iced::widget::canvas::{Event, Program};
    use iced::{Point, Rectangle, Size};
    use kant::features::concept_model::{DiagramEdge, DiagramNode, NodeId, GRID_SIZE};
    use kant::ui::app::{App, Message};
    use kant::ui::diagram_canvas::{
        render_concept_node, CanvasViewport, DiagramCanvas, DiagramCanvasState,
    };
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // 1. Setup 3 noder på et diagram
    let n1 = DiagramNode::custom(
        Uuid::new_v4(),
        "Begreb 1".to_string(),
        100.0,
        100.0,
        120.0,
        60.0,
    );
    let n2 = DiagramNode::custom(
        Uuid::new_v4(),
        "Begreb 2".to_string(),
        300.0,
        100.0,
        120.0,
        60.0,
    );
    let n3 = DiagramNode::custom(
        Uuid::new_v4(),
        "Begreb 3".to_string(),
        500.0,
        100.0,
        120.0,
        60.0,
    );
    let nodes = vec![n1.clone(), n2.clone(), n3.clone()];
    let edges: Vec<DiagramEdge> = vec![];

    let current_selection = Arc::new(Mutex::new(HashSet::new()));
    let moved_nodes = Arc::new(Mutex::new(Vec::new()));

    let sel_cb = Arc::clone(&current_selection);
    let mov_cb = Arc::clone(&moved_nodes);

    let canvas = DiagramCanvas::new(
        &nodes,
        &edges,
        None,
        CanvasViewport::default(),
        false,
        render_concept_node,
        |_| (),
        |_, _, _| (),
        |_, _| (),
        |_| (),
        |_| (),
    )
    .on_selection_changed(move |set| {
        *sel_cb.lock().unwrap() = set;
    })
    .on_nodes_moved(move |updates| {
        *mov_cb.lock().unwrap() = updates;
    });

    let mut state = DiagramCanvasState::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 800.0));

    // AC1: Ctrl+Klik Multi-select toggling
    state.modifiers = iced::keyboard::Modifiers::CTRL;
    let click_n1 = Point::new(120.0, 120.0);
    let press_left = Event::Mouse(iced::mouse::Event::ButtonPressed(Button::Left));
    let _ = canvas.update(&mut state, &press_left, bounds, Cursor::Available(click_n1));
    assert!(
        current_selection.lock().unwrap().contains(&n1.id()),
        "AC1: Ctrl+klik på n1 skal tilføje n1 til udvalget"
    );

    // Ctrl+klik på n2 skal tilføje n2 uden at fjerne n1
    let click_n2 = Point::new(320.0, 120.0);
    let _ = canvas.update(&mut state, &press_left, bounds, Cursor::Available(click_n2));
    assert!(
        current_selection.lock().unwrap().contains(&n1.id())
            && current_selection.lock().unwrap().contains(&n2.id()),
        "AC1: Ctrl+klik på n2 skal udvide udvalget så både n1 og n2 er valgt"
    );

    // AC2: Marquee / Rektangulær Drag-Select
    state.modifiers = iced::keyboard::Modifiers::empty();
    let empty_drag_start = Point::new(50.0, 50.0);
    let _ = canvas.update(
        &mut state,
        &press_left,
        bounds,
        Cursor::Available(empty_drag_start),
    );
    assert!(
        state.marquee.is_some(),
        "AC2: Drag på tomt lærred skal initialisere marquee-tilstand"
    );

    // Flyt mus til (450.0, 200.0) så både n1 og n2 omsluttes, men n3 (ved 500) er udenfor
    let drag_pos = Point::new(450.0, 200.0);
    let move_event = Event::Mouse(iced::mouse::Event::CursorMoved { position: drag_pos });
    let _ = canvas.update(&mut state, &move_event, bounds, Cursor::Available(drag_pos));
    assert_eq!(state.marquee.unwrap().current, drag_pos);

    // Slip musen for at afslutte marquee select
    let release_left = Event::Mouse(iced::mouse::Event::ButtonReleased(Button::Left));
    let _ = canvas.update(
        &mut state,
        &release_left,
        bounds,
        Cursor::Available(drag_pos),
    );
    assert!(
        state.marquee.is_none(),
        "AC2: Marquee skal afsluttes ved ButtonReleased"
    );
    {
        let sel = current_selection.lock().unwrap();
        assert!(
            sel.contains(&n1.id()),
            "AC2: n1 skal være valgt via marquee"
        );
        assert!(
            sel.contains(&n2.id()),
            "AC2: n2 skal være valgt via marquee"
        );
        assert!(
            !sel.contains(&n3.id()),
            "AC2: n3 udenfor marquee må IKKE være valgt"
        );
    }

    // AC2b: Delvist overlappede noder må IKKE vælges (kun fuldt omsluttede noder vælges)
    let partial_drag_start = Point::new(50.0, 50.0);
    let _ = canvas.update(
        &mut state,
        &press_left,
        bounds,
        Cursor::Available(partial_drag_start),
    );
    // n1 er [100..220, 100..160] (fuldt omsluttet af 50..350 x 50..200)
    // n2 er [300..420, 100..160] (delvist skåret ved x=350, så ikke fuldt omsluttet)
    let partial_drag_end = Point::new(350.0, 200.0);
    let move_partial = Event::Mouse(iced::mouse::Event::CursorMoved {
        position: partial_drag_end,
    });
    let _ = canvas.update(
        &mut state,
        &move_partial,
        bounds,
        Cursor::Available(partial_drag_end),
    );
    let _ = canvas.update(
        &mut state,
        &release_left,
        bounds,
        Cursor::Available(partial_drag_end),
    );
    {
        let sel = current_selection.lock().unwrap();
        assert!(
            sel.contains(&n1.id()),
            "AC2b: n1 (fuldt omsluttet) skal vælges"
        );
        assert!(
            !sel.contains(&n2.id()),
            "AC2b: n2 (kun delvist indenfor) må IKKE vælges"
        );
    }

    // AC3 & AC4: Synkron Flytning og Grid Snapping
    // Konfigurer canvas med aktuel udvælgelse {n1, n2}
    let sel_set: HashSet<NodeId> = [n1.id(), n2.id()].into_iter().collect();
    let mov_cb2 = Arc::clone(&moved_nodes);
    let canvas_selected = DiagramCanvas::new(
        &nodes,
        &edges,
        Some(n1.id()),
        CanvasViewport::default(),
        false,
        render_concept_node,
        |_| (),
        |_, _, _| (),
        |_, _| (),
        |_| (),
        |_| (),
    )
    .selected_node_ids(sel_set.clone())
    .on_nodes_moved(move |updates| {
        *mov_cb2.lock().unwrap() = updates;
    });

    let mut drag_state = DiagramCanvasState::default();
    // Start træk på n1 ved (120, 120)
    let _ = canvas_selected.update(
        &mut drag_state,
        &press_left,
        bounds,
        Cursor::Available(click_n1),
    );
    // Træk 40 px til højre og 60 px ned
    let drag_target = Point::new(160.0, 180.0);
    let drag_move = Event::Mouse(iced::mouse::Event::CursorMoved {
        position: drag_target,
    });
    let _ = canvas_selected.update(
        &mut drag_state,
        &drag_move,
        bounds,
        Cursor::Available(drag_target),
    );

    {
        let updates = moved_nodes.lock().unwrap();
        assert_eq!(
            updates.len(),
            2,
            "AC3: Begge markerede noder skal modtage flytte-opdatering"
        );
        let n1_up = updates
            .iter()
            .find(|(id, _, _)| *id == n1.id())
            .expect("n1 skal opdateres");
        let n2_up = updates
            .iter()
            .find(|(id, _, _)| *id == n2.id())
            .expect("n2 skal opdateres");

        let delta_x1 = n1_up.1 - 100.0;
        let delta_y1 = n1_up.2 - 100.0;
        let delta_x2 = n2_up.1 - 300.0;
        let delta_y2 = n2_up.2 - 100.0;

        assert_eq!(
            delta_x1, delta_x2,
            "AC3: Noder skal parallelforskydes med identisk delta X"
        );
        assert_eq!(
            delta_y1, delta_y2,
            "AC3: Noder skal parallelforskydes med identisk delta Y"
        );

        // AC4: Grid snapping
        assert_eq!(
            n1_up.1 % GRID_SIZE,
            0.0,
            "AC4: n1 X skal snappe til GRID_SIZE"
        );
        assert_eq!(
            n1_up.2 % GRID_SIZE,
            0.0,
            "AC4: n1 Y skal snappe til GRID_SIZE"
        );
        assert_eq!(
            n2_up.1 % GRID_SIZE,
            0.0,
            "AC4: n2 X skal snappe til GRID_SIZE"
        );
        assert_eq!(
            n2_up.2 % GRID_SIZE,
            0.0,
            "AC4: n2 Y skal snappe til GRID_SIZE"
        );
    }

    // AC5: Afmarkering ved klik på tomt lærred uden Ctrl
    let empty_click = Point::new(50.0, 50.0);
    let _ = canvas.update(
        &mut state,
        &press_left,
        bounds,
        Cursor::Available(empty_click),
    );
    let _ = canvas.update(
        &mut state,
        &release_left,
        bounds,
        Cursor::Available(empty_click),
    );
    assert!(
        current_selection.lock().unwrap().is_empty(),
        "AC5: Klik på tomt lærred skal nulstille markering"
    );

    // AC7: App integrationstest for Begrebsmodel og Informationsmodel
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(kant::ui::app::Tab::ConceptModel));
    let _ = app.update(Message::CreateConceptAtCenter);
    let cid1 = app.selected_graph_node_id().unwrap();
    let _ = app.update(Message::CreateConceptAtCenter);
    let cid2 = app.selected_graph_node_id().unwrap();

    let mut multi_concept_ids = HashSet::new();
    multi_concept_ids.insert(cid1);
    multi_concept_ids.insert(cid2);

    let _ = app.update(Message::GraphNodesSelected(multi_concept_ids.clone()));
    assert_eq!(
        app.selected_graph_node_ids(),
        &multi_concept_ids,
        "AC7: App skal registrere samtlige valgte begrebsnoder"
    );

    // Bulk flytning i App
    let moves = vec![(cid1, 200.0, 200.0), (cid2, 400.0, 200.0)];
    let _ = app.update(Message::GraphNodesMoved(moves));
    let cg = app.project().concept_graph();
    let node1 = cg.find_node(cid1).unwrap();
    let node2 = cg.find_node(cid2).unwrap();
    assert_eq!((node1.x(), node1.y()), (200.0, 200.0));
    assert_eq!((node2.x(), node2.y()), (400.0, 200.0));

    // Sletning af multimarkerede noder
    let _ = app.update(Message::GraphDeleteSelected);
    assert!(app.project().concept_graph().find_node(cid1).is_none());
    assert!(app.project().concept_graph().find_node(cid2).is_none());
}

#[test]
fn test_task_041_persistent_class_relations_across_canvas_removal() {
    let mut app = App::new_with_path(None);
    let _ = app.update(Message::SelectTab(kant::ui::app::Tab::InformationModel));

    // 1. Opret to klasser i informationsmodellen
    let class_a = InformationClass::new("Køretøj");
    let class_b = InformationClass::new("Motor");
    let class_a_id = class_a.id();
    let class_b_id = class_b.id();

    app.project_mut().information_model_mut().add_class(class_a);
    app.project_mut().information_model_mut().add_class(class_b);

    // 2. Tilføj begge klasser til canvas
    let _ = app.update(Message::AddClassToDiagram(class_a_id));
    let _ = app.update(Message::AddClassToDiagram(class_b_id));

    let node_a_id = app
        .project()
        .information_graph()
        .find_node_by_class(class_a_id)
        .expect("Klasse A skal have en node på lærredet")
        .id();
    let node_b_id = app
        .project()
        .information_graph()
        .find_node_by_class(class_b_id)
        .expect("Klasse B skal have en node på lærredet")
        .id();

    // 3. Opret en relation mellem Klasse A og Klasse B
    let _ = app.update(Message::AddClassRelation(
        node_a_id,
        node_b_id,
        RelationKind::Association,
        Some("har".to_string()),
    ));
    // Sæt multipliciteter på relationen
    let _ = app.update(Message::InfoUpdateEdgeSourceMultiplicity(
        node_a_id,
        node_b_id,
        Some(Multiplicity::exactly_one()),
    ));
    let _ = app.update(Message::InfoUpdateEdgeTargetMultiplicity(
        node_a_id,
        node_b_id,
        Some(Multiplicity::zero_or_more()),
    ));

    // Verificer initial relationstilstand
    assert_eq!(
        app.project().information_graph().edges().len(),
        1,
        "Der skal være 1 kant på lærredet"
    );
    assert_eq!(
        app.project().information_model().relations().len(),
        1,
        "Der skal være 1 persistent relation i modellen"
    );

    let initial_rel = &app.project().information_model().relations()[0];
    assert_eq!(initial_rel.kind(), RelationKind::Association);
    assert_eq!(initial_rel.label(), Some("har"));
    assert_eq!(
        initial_rel.source_multiplicity(),
        Some(Multiplicity::exactly_one())
    );
    assert_eq!(
        initial_rel.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // 4. AC1: Fjern Klasse A fra diagrammet (RemoveClassFromDiagram)
    let _ = app.update(Message::RemoveClassFromDiagram(node_a_id));

    // Verificer at lærredet har 0 kanter og kun 1 node (Klasse B)
    assert_eq!(
        app.project().information_graph().nodes().len(),
        1,
        "Kun Klasse B skal forblive på lærredet"
    );
    assert_eq!(
        app.project().information_graph().edges().len(),
        0,
        "Ingen kanter skal tegnes på lærredet når Klasse A er fjernet"
    );

    // AC1 Assertion: Relationen skal forblive fuldt bevaret i informationsmodellen
    assert_eq!(
        app.project().information_model().relations().len(),
        1,
        "AC1: Relationen skal fortsat være bevaret semantisk i informationsmodellen"
    );
    let preserved_rel = &app.project().information_model().relations()[0];
    assert_eq!(preserved_rel.kind(), RelationKind::Association);
    assert_eq!(preserved_rel.label(), Some("har"));
    assert_eq!(
        preserved_rel.source_multiplicity(),
        Some(Multiplicity::exactly_one())
    );
    assert_eq!(
        preserved_rel.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // Inspector check: relations_for_class for den fjernede klasse skal returnere relationen
    assert_eq!(
        app.project()
            .information_model()
            .relations_for_class(class_a_id)
            .len(),
        1,
        "AC1: relations_for_class skal returnere relationen for den skjulte klasse"
    );

    // 5. AC2: Genindsæt Klasse A på lærredet via AddClassToDiagram
    let _ = app.update(Message::AddClassToDiagram(class_a_id));

    assert_eq!(
        app.project().information_graph().nodes().len(),
        2,
        "Begge klasser skal nu være på lærredet"
    );

    // AC2 Assertion: Relationen skal automatisk genopstå på lærredet
    assert_eq!(
        app.project().information_graph().edges().len(),
        1,
        "AC2: Kanten skal automatisk genopstå på lærredet med alle sine egenskaber"
    );
    let restored_edge = &app.project().information_graph().edges()[0];
    assert_eq!(restored_edge.kind(), RelationKind::Association);
    assert_eq!(restored_edge.label(), Some("har"));
    assert_eq!(
        restored_edge.source_multiplicity(),
        Some(Multiplicity::exactly_one())
    );
    assert_eq!(
        restored_edge.target_multiplicity(),
        Some(Multiplicity::zero_or_more())
    );

    // 6. AC3: Eksplicit sletning af relation
    let restored_from = restored_edge.from();
    let restored_to = restored_edge.to();
    let _ = app.update(Message::DeleteClassRelation(restored_from, restored_to));

    assert_eq!(
        app.project().information_graph().edges().len(),
        0,
        "AC3: Kanten skal være fjernet fra lærredet efter sletning"
    );
    assert_eq!(
        app.project().information_model().relations().len(),
        0,
        "AC3: Relationen skal være fjernet permanent fra modellen efter sletning"
    );

    // 7. AC4: Sletning af klasse kaskaderer
    // Opret en ny relation
    let _ = app.update(Message::AddClassRelation(
        restored_from,
        restored_to,
        RelationKind::Generalization,
        None,
    ));
    assert_eq!(app.project().information_model().relations().len(), 1);
    assert_eq!(app.project().information_graph().edges().len(), 1);

    // Slet Klasse A permanent
    let _ = app.update(Message::DeleteInformationClass(class_a_id));
    assert_eq!(
        app.project().information_model().classes().len(),
        1,
        "Klasse A skal være slettet"
    );
    assert_eq!(
        app.project().information_model().relations().len(),
        0,
        "AC4: Relationen skal kaskadeslettes permanent når klassen slettes"
    );
    assert_eq!(
        app.project().information_graph().edges().len(),
        0,
        "AC4: Kanten på lærredet skal ligeledes være fjernet"
    );

    // 8. AC5: Bagudkompatibilitet - serialisering og migrering
    let project_json = serde_json::to_string(app.project()).expect("Serialisering skal lykkes");
    let deserialized: kant::features::model::ModelProject =
        serde_json::from_str(&project_json).expect("Deserialisering skal lykkes");
    assert_eq!(deserialized.information_model().classes().len(), 1);
}

#[test]
fn test_task_063_controlled_vocabulary_enumeration_ui_and_canvas() {
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::InformationDataType;
    use kant::ui::app::{App, Message};

    let mut app = App::new_with_path(None);

    // 1. AC1: Opret enumeration via UI Message
    let _ = app.update(Message::CreateInformationEnumeration);
    assert_eq!(
        app.project().information_model().enumerations().len(),
        1,
        "AC1: Der skal være oprettet 1 enumeration i information_model"
    );
    let enum_id = app.project().information_model().enumerations()[0].id();

    // Opdater navn og tilføj værdier i lowerCamelCase jf. FDA Tabel B
    let _ = app.update(Message::UpdateInformationEnumerationName(
        enum_id,
        "DrivkraftType".to_string(),
    ));
    let _ = app.update(Message::AddValueToEnumeration(
        enum_id,
        "elmotor".to_string(),
    ));
    let _ = app.update(Message::AddValueToEnumeration(
        enum_id,
        "benzin".to_string(),
    ));

    let updated_enum = app
        .project()
        .information_model()
        .get_enumeration(enum_id)
        .expect("Enumeration skal findes")
        .clone();
    assert_eq!(updated_enum.name(), "DrivkraftType");
    assert_eq!(updated_enum.values(), &["elmotor", "benzin"]);

    // Must NOT: Ugyldige enumerationsværdier (med mellemrum eller forkert case) afvises
    let _ = app.update(Message::AddValueToEnumeration(
        enum_id,
        "ugyldig værdi med mellemrum".to_string(),
    ));
    let _ = app.update(Message::AddValueToEnumeration(
        enum_id,
        "ForkertCase".to_string(),
    ));
    let enum_check = app
        .project()
        .information_model()
        .get_enumeration(enum_id)
        .unwrap();
    assert_eq!(
        enum_check.values(),
        &["elmotor", "benzin"],
        "Must NOT: Ugyldige enumerationsværdier må ikke tilføjes til enumerationen"
    );

    // 2. AC2: Tilføj enumeration til lærredet (grøn boks)
    let _ = app.update(Message::AddEnumerationToDiagram(enum_id));
    assert!(
        app.project()
            .information_graph()
            .nodes()
            .iter()
            .any(|n| n.class_id() == enum_id),
        "AC2: Enumeration skal være repræsenteret på lærredet som diagramnode"
    );

    // 3. Opret klasse "Køretøj" og tilføj til diagram
    let _ = app.update(Message::CreateInformationClass);
    let class_id = app.project().information_model().classes()[0].id();
    let _ = app.update(Message::UpdateInformationClassName(
        class_id,
        "Koeretoej".to_string(),
    ));
    let _ = app.update(Message::AddClassToDiagram(class_id));

    // Tilføj attribut "drivkraft"
    let _ = app.update(Message::AddAttributeToClass(class_id));
    let attr_id = app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap()
        .attributes()[0]
        .id();

    // 4. AC3 & AC4: Vælg enumerationen som datatype for attributten
    let _ = app.update(Message::UpdateAttributeDataType(
        class_id,
        attr_id,
        InformationDataType::Enumeration {
            enumeration_id: enum_id,
        },
    ));

    let attr = &app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap()
        .attributes()[0];
    assert_eq!(
        attr.data_type(),
        &InformationDataType::Enumeration {
            enumeration_id: enum_id
        },
        "AC3: Attributtens datatype skal være sat til den valgte enumeration"
    );

    // Verificer at der automatisk er etableret en Dependency-relation mellem klassen og enumerationen
    let has_dep = app.project().information_graph().edges().iter().any(|e| {
        (e.from() == class_id && e.to() == enum_id) && e.kind() == RelationKind::Dependency
    });
    assert!(
        has_dep,
        "AC4: Der skal automatisk synkroniseres en Dependency-kant på lærredet fra klasse til enumeration"
    );

    // 5. Fjern enumeration fra lærredet
    let _ = app.update(Message::RemoveEnumerationFromDiagram(enum_id));
    assert!(
        !app.project()
            .information_graph()
            .nodes()
            .iter()
            .any(|n| n.class_id() == enum_id),
        "Noden skal være fjernet fra diagrammet"
    );
    assert_eq!(
        app.project().information_model().enumerations().len(),
        1,
        "Enumerationen skal stadig findes i information_model"
    );

    // 6. Genindsæt enumeration på lærredet -> Dependency-kanten skal genopstå
    let _ = app.update(Message::AddEnumerationToDiagram(enum_id));
    let has_restored_dep = app.project().information_graph().edges().iter().any(|e| {
        (e.from() == class_id && e.to() == enum_id) && e.kind() == RelationKind::Dependency
    });
    assert!(
        has_restored_dep,
        "AC4: Dependency-kanten skal genopstå på lærredet når enumerationen genindsættes"
    );

    // AC5: Persistens i dekomponeret format (.kant)
    let temp_dir = std::env::temp_dir().join(format!("kant_test_063_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    kant::features::model::decomposed::save_decomposed(app.project(), &temp_dir)
        .expect("save_decomposed skal lykkes");
    let loaded_project = kant::features::model::decomposed::load_decomposed(&temp_dir)
        .expect("load_decomposed skal lykkes");
    let _ = std::fs::remove_dir_all(&temp_dir);
    assert_eq!(
        loaded_project.information_model().enumerations().len(),
        1,
        "AC5: Enumeration skal persisteres og genindlæses fra dekomponeret lager"
    );
    assert_eq!(
        loaded_project.information_model().enumerations()[0].name(),
        "DrivkraftType"
    );
    assert_eq!(
        loaded_project.information_model().enumerations()[0].values(),
        &["elmotor", "benzin"]
    );
    assert!(
        loaded_project
            .information_graph()
            .nodes()
            .iter()
            .any(|n| n.class_id() == enum_id),
        "AC5: Enumeration-diagramnode skal persisteres og genindlæses fra diagrammer"
    );
    assert!(
        loaded_project.information_graph().edges().iter().any(|e| {
            (e.from() == class_id && e.to() == enum_id) && e.kind() == RelationKind::Dependency
        }),
        "AC5: Dependency-kant skal persisteres og genindlæses"
    );

    // 7. Slet enumeration permanent
    let _ = app.update(Message::DeleteInformationEnumeration(enum_id));
    assert_eq!(
        app.project().information_model().enumerations().len(),
        0,
        "Enumerationen skal være slettet fra modellen"
    );
    assert!(
        !app.project()
            .information_graph()
            .nodes()
            .iter()
            .any(|n| n.class_id() == enum_id),
        "Enumeration-noden skal være fjernet fra diagrammet"
    );
}

#[test]
fn test_task_064_drag_and_drop_from_palette_to_canvas_and_toolbar_refinements() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::InformationClass;
    use kant::ui::app::{App, Message};

    let mut app = App::new_with_path(None);

    // 1. AC1: Værktøjslinje Harmonering - Opret enumeration ved center på informationsmodel canvas
    let _ = app.update(Message::CreateInformationEnumerationAtCenter);
    assert_eq!(
        app.project().information_model().enumerations().len(),
        1,
        "AC1: Der skal være oprettet 1 enumeration i information_model"
    );
    let enum_id = app.project().information_model().enumerations()[0].id();
    assert!(
        app.project()
            .information_graph()
            .is_class_on_diagram(enum_id),
        "AC1: Opret enumeration fra canvas-værktøjslinjen skal placere noden direkte på canvas"
    );

    // 2. AC2: Drag-and-Drop af begreber på Begrebsdiagram med Always-On gitter-snapping
    let concept = Concept::new("Station", "Togstation", BelongsToDomain::Yes);
    let concept_id = app
        .project_mut()
        .add_concept(concept)
        .expect("Tilføj begreb");
    if let Some(node) = app
        .project()
        .concept_graph()
        .find_node_by_concept(concept_id)
    {
        let nid = node.id();
        app.project_mut().concept_graph_mut().remove_node(nid);
    }
    assert!(
        !app.project()
            .concept_graph()
            .is_concept_on_diagram(concept_id),
        "Begrebet må ikke være på diagrammet før det trækkes/tilføjes"
    );

    // Drop begreb på canvas ved (253.7, 184.2) - snapper til (260.0, 180.0)
    let _ = app.update(Message::AddConceptToDiagramAt(concept_id, 253.7, 184.2));
    assert!(
        app.project()
            .concept_graph()
            .is_concept_on_diagram(concept_id),
        "AC2: Begrebet skal være tilføjet til begrebsdiagrammet efter drop"
    );
    let (concept_node_id, node_x, node_y) = {
        let cn = app
            .project()
            .concept_graph()
            .find_node_by_concept(concept_id)
            .expect("Concept node skal findes");
        (cn.id(), cn.x(), cn.y())
    };
    assert_eq!(
        node_x, 260.0,
        "AC2: Noden skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        node_y, 180.0,
        "AC2: Noden skal snappe til nærmeste 20px gitterpunkt y"
    );

    // Must NOT: Duplikerede noder må IKKE oprettes hvis der droppes igen
    let initial_node_count = app.project().concept_graph().node_count();
    let _ = app.update(Message::AddConceptToDiagramAt(concept_id, 300.0, 300.0));
    assert_eq!(
        app.project().concept_graph().node_count(),
        initial_node_count,
        "Must NOT: Duplikerede noder må ikke oprettes hvis et allerede tilføjet begreb droppes igen"
    );

    // 3. AC3: Drag-and-Drop af Klasser og Enumerationer på Informationsmodel med gitter-snapping
    let cls = InformationClass::new("Spor");
    let class_id = app.project_mut().information_model_mut().add_class(cls);
    assert!(
        !app.project()
            .information_graph()
            .is_class_on_diagram(class_id),
        "Klassen må ikke være på informationsdiagrammet endnu"
    );

    // Drop klasse på informationslærredet ved (312.4, 215.8) - snapper til (320.0, 220.0)
    let _ = app.update(Message::AddClassToDiagramAt(class_id, 312.4, 215.8));
    assert!(
        app.project()
            .information_graph()
            .is_class_on_diagram(class_id),
        "AC3: Klassen skal findes på informationsdiagrammet efter drop"
    );
    let (class_x, class_y) = {
        let class_node = app
            .project()
            .information_graph()
            .find_node_by_class(class_id)
            .expect("Klassenode skal findes");
        (class_node.x(), class_node.y())
    };
    assert_eq!(
        class_x, 320.0,
        "AC3: Klassenoden skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        class_y, 220.0,
        "AC3: Klassenoden skal snappe til nærmeste 20px gitterpunkt y"
    );

    // Drop anden enumeration på informationslærredet ved (411.3, 155.6) - snapper til (420.0, 160.0)
    let enum2 = kant::features::information_model::InformationEnumeration::new(
        "SignalStatus",
        vec!["roed".to_string(), "groen".to_string()],
    );
    let enum2_id = app
        .project_mut()
        .information_model_mut()
        .add_enumeration(enum2);
    let _ = app.update(Message::AddEnumerationToDiagramAt(enum2_id, 411.3, 155.6));
    assert!(
        app.project()
            .information_graph()
            .is_class_on_diagram(enum2_id),
        "AC3: Enumerationen skal findes på informationsdiagrammet efter drop"
    );
    let (enum_x, enum_y) = {
        let enum_node = app
            .project()
            .information_graph()
            .find_node_by_class(enum2_id)
            .expect("Enumeration-node skal findes");
        (enum_node.x(), enum_node.y())
    };
    assert_eq!(
        enum_x, 420.0,
        "AC3: Enumeration-noden skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        enum_y, 160.0,
        "AC3: Enumeration-noden skal snappe til nærmeste 20px gitterpunkt y"
    );

    // Must NOT: Gentaget drop duplikerer ikke klasse/enumeration noder
    let initial_info_nodes = app.project().information_graph().node_count();
    let _ = app.update(Message::AddClassToDiagramAt(class_id, 500.0, 500.0));
    let _ = app.update(Message::AddEnumerationToDiagramAt(enum2_id, 500.0, 500.0));
    assert_eq!(
        app.project().information_graph().node_count(),
        initial_info_nodes,
        "Must NOT: Duplikerede noder må ikke oprettes på informationsdiagrammet"
    );

    // 4. AC4: Bevarelse af 1-klik '+' tilføjelse
    let concept_quick = Concept::new("Perron", "Perron til passagerer", BelongsToDomain::Yes);
    let c_quick_id = app.project_mut().add_concept(concept_quick).unwrap();
    if let Some(node) = app
        .project()
        .concept_graph()
        .find_node_by_concept(c_quick_id)
    {
        let nid = node.id();
        app.project_mut().concept_graph_mut().remove_node(nid);
    }
    assert!(
        !app.project()
            .concept_graph()
            .is_concept_on_diagram(c_quick_id),
        "Begrebet må ikke være på diagrammet før '+' klik"
    );
    let _ = app.update(Message::AddConceptToDiagram(c_quick_id));
    assert!(
        app.project()
            .concept_graph()
            .is_concept_on_diagram(c_quick_id),
        "AC4: 1-klik '+' tilføjelse af begreb skal fortsat fungere"
    );

    // 5. AC5: Always-On gitter-snapping ved flytning
    let _ = app.update(Message::GraphNodeMoved(concept_node_id, 137.4, 91.2));
    let moved_node = app
        .project()
        .concept_graph()
        .find_node(concept_node_id)
        .unwrap();
    assert_eq!(
        moved_node.x(),
        140.0,
        "AC5: Node x skal altid snappe magnetisk til nærmeste multiplum af 20"
    );
    assert_eq!(
        moved_node.y(),
        100.0,
        "AC5: Node y skal altid snappe magnetisk til nærmeste multiplum af 20"
    );

    // 6. Fuld Drag-and-Drop livscyklus fra Palet til Canvas via CanvasDropAt
    use iced::Point;
    use kant::ui::app::PaletteDragItem;

    // A: Træk begreb fra palet og slip på canvas
    let concept_drag = Concept::new("Ventesal", "Lokale for passagerer", BelongsToDomain::Yes);
    let c_drag_id = app.project_mut().add_concept(concept_drag).unwrap();
    if let Some(node) = app
        .project()
        .concept_graph()
        .find_node_by_concept(c_drag_id)
    {
        let nid = node.id();
        app.project_mut().concept_graph_mut().remove_node(nid);
    }
    assert!(!app
        .project()
        .concept_graph()
        .is_concept_on_diagram(c_drag_id));

    // Tryk ned på palet
    let _ = app.update(Message::StartPaletteDrag(PaletteDragItem::Concept(
        c_drag_id,
    )));
    // Træk musen over på canvas
    let _ = app.update(Message::PaletteDragMoved(Point::new(150.0, 150.0)));
    // Slip over canvas ved (348.6, 211.2) - snapper til (360.0, 220.0)
    let _ = app.update(Message::CanvasDropAt { x: 348.6, y: 211.2 });
    assert!(
        app.project()
            .concept_graph()
            .is_concept_on_diagram(c_drag_id),
        "AC2: Begreb skal placeres på canvas efter fuld drag-and-drop livscyklus"
    );
    let c_node = app
        .project()
        .concept_graph()
        .find_node_by_concept(c_drag_id)
        .unwrap();
    assert_eq!(
        c_node.x(),
        340.0,
        "Begrebsnode skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        c_node.y(),
        220.0,
        "Begrebsnode skal snappe til nærmeste 20px gitterpunkt y"
    );

    // B: Træk klasse fra palet og slip på informationscanvas
    let cls_drag = InformationClass::new("Billetautomat");
    let cls_drag_id = app
        .project_mut()
        .information_model_mut()
        .add_class(cls_drag);
    assert!(!app
        .project()
        .information_graph()
        .is_class_on_diagram(cls_drag_id));

    let _ = app.update(Message::StartPaletteDrag(PaletteDragItem::Class(
        cls_drag_id,
    )));
    let _ = app.update(Message::PaletteDragMoved(Point::new(180.0, 200.0)));
    let _ = app.update(Message::CanvasDropAt { x: 273.8, y: 189.5 });
    assert!(
        app.project()
            .information_graph()
            .is_class_on_diagram(cls_drag_id),
        "AC3: Klasse skal placeres på canvas efter fuld drag-and-drop livscyklus"
    );
    let cls_node = app
        .project()
        .information_graph()
        .find_node_by_class(cls_drag_id)
        .unwrap();
    assert_eq!(
        cls_node.x(),
        280.0,
        "Klassenode skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        cls_node.y(),
        180.0,
        "Klassenode skal snappe til nærmeste 20px gitterpunkt y"
    );

    // C: Træk enumeration fra palet og slip på informationscanvas
    let enum_drag = kant::features::information_model::InformationEnumeration::new(
        "KøreplanType",
        vec!["Hverdag".to_string(), "Weekend".to_string()],
    );
    let enum_drag_id = app
        .project_mut()
        .information_model_mut()
        .add_enumeration(enum_drag);
    assert!(!app
        .project()
        .information_graph()
        .is_class_on_diagram(enum_drag_id));

    let _ = app.update(Message::StartPaletteDrag(PaletteDragItem::Enumeration(
        enum_drag_id,
    )));
    let _ = app.update(Message::PaletteDragMoved(Point::new(190.0, 210.0)));
    let _ = app.update(Message::CanvasDropAt { x: 418.2, y: 263.9 });
    assert!(
        app.project()
            .information_graph()
            .is_class_on_diagram(enum_drag_id),
        "AC3: Enumeration skal placeres på canvas efter fuld drag-and-drop livscyklus"
    );
    let enum_node = app
        .project()
        .information_graph()
        .find_node_by_class(enum_drag_id)
        .unwrap();
    assert_eq!(
        enum_node.x(),
        420.0,
        "Enumeration-node skal snappe til nærmeste 20px gitterpunkt x"
    );
    assert_eq!(
        enum_node.y(),
        260.0,
        "Enumeration-node skal snappe til nærmeste 20px gitterpunkt y"
    );

    // D: Klik uden træk (afstand <= 4.0px) vælger elementet
    let cls_click = InformationClass::new("KlikKlasse");
    let cls_click_id = app
        .project_mut()
        .information_model_mut()
        .add_class(cls_click);
    let _ = app.update(Message::StartPaletteDrag(PaletteDragItem::Class(
        cls_click_id,
    )));
    let _ = app.update(Message::PaletteDragMoved(Point::new(191.0, 211.0)));
    let _ = app.update(Message::PaletteDragDropped);
    assert_eq!(
        app.selected_info_class_id(),
        Some(cls_click_id),
        "Enkeltklik uden træk skal vælge klassen"
    );
}

#[test]
fn test_enumeration_relations_persisted_and_not_dropped_on_sync_or_restart() {
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::{
        ClassRelation, InformationClass, InformationEnumeration,
    };
    use kant::ui::app::App;

    let mut app = App::new_with_path(None);

    // 1. Opret klasse "Repo" uden attributter
    let repo_class = InformationClass::new("Repo");
    let repo_id = app
        .project_mut()
        .information_model_mut()
        .add_class(repo_class);

    // 2. Opret enumeration "Enumeration2"
    let enumeration = InformationEnumeration::new("Enumeration2", vec!["aktiv".to_string()]);
    let enum_id = app
        .project_mut()
        .information_model_mut()
        .add_enumeration(enumeration);

    // 3. Tilføj begge til informationsdiagrammet
    let repo_node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node_at(repo_id, 100.0, 100.0, 0);
    let enum_node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node_at(enum_id, 300.0, 100.0, 1);

    // 4. Etabler relation fra Repo til Enumeration2 (ligesom ved drag-to-connect på canvas)
    let rel = ClassRelation::new(
        repo_id,
        enum_id,
        RelationKind::Dependency,
        Some("«use»".to_string()),
    );
    app.project_mut().information_model_mut().add_relation(rel);
    app.project_mut().information_graph_mut().add_relation(
        repo_node_id,
        enum_node_id,
        RelationKind::Dependency,
        Some("«use»".to_string()),
    );

    assert_eq!(
        app.project().information_model().relations().len(),
        1,
        "Der skal være 1 relation i information_model"
    );
    assert_eq!(
        app.project().information_graph().edges().len(),
        1,
        "Der skal være 1 kant på diagrammet"
    );

    // 5. Kør sync_information_graph (dette sker ved enhver UI-ændring, tab-skift og ved opstart)
    app.project_mut().sync_information_graph();

    assert_eq!(
        app.project().information_model().relations().len(),
        1,
        "Relationen må IKKE slettes af sync_information_graph selvom klassen ingen attributter har"
    );
    assert_eq!(
        app.project().information_graph().edges().len(),
        1,
        "Kanten må IKKE forsvinde fra diagrammet ved sync_information_graph"
    );

    // 6. Gem og genindlæs fra dekomponeret format (.kant)
    let temp_dir =
        std::env::temp_dir().join(format!("kant_enum_rel_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    kant::features::model::decomposed::save_decomposed(app.project(), &temp_dir)
        .expect("save_decomposed skal lykkes");

    let mut loaded_project = kant::features::model::decomposed::load_decomposed(&temp_dir)
        .expect("load_decomposed skal lykkes");

    let _ = std::fs::remove_dir_all(&temp_dir);

    // 7. Verificer at relation og edge er bevaret efter genindlæsning og synkronisering
    assert_eq!(
        loaded_project.information_model().relations().len(),
        1,
        "Relationen skal overleve lagring og genindlæsning"
    );
    let loaded_rel = &loaded_project.information_model().relations()[0];
    assert_eq!(loaded_rel.from_class(), repo_id);
    assert_eq!(loaded_rel.to_class(), enum_id);
    assert_eq!(loaded_rel.kind(), RelationKind::Dependency);
    assert_eq!(loaded_rel.label(), Some("«use»"));

    assert_eq!(
        loaded_project.information_graph().edges().len(),
        1,
        "Diagramkanten skal overleve lagring og genindlæsning"
    );

    // 8. Kør endnu en sync på det indlæste projekt (simulerer app startup)
    loaded_project.sync_information_graph();
    assert_eq!(
        loaded_project.information_model().relations().len(),
        1,
        "Relationen skal fortsat bestå efter sync på det indlæste projekt"
    );
    assert_eq!(
        loaded_project.information_graph().edges().len(),
        1,
        "Diagramkanten skal fortsat bestå efter sync på det indlæste projekt"
    );
}

#[test]
fn test_task_065_windows_rendering_diagnostics_and_wgpu_restoration() {
    use iced::{Background, Theme};
    use kant::features::diagnostics::{get_recent_logs, get_system_diagnostics, log_info};
    use kant::ui::app::{App, Message};
    use kant::ui::platform::resolve_default_iced_backend;
    use kant::ui::theme::{card_container_style, floating_panel_style};

    // AC1: På Windows skal vi IKKE længere tvinge tiny-skia som default når ingen backend er sat
    assert_eq!(
        resolve_default_iced_backend(true, None),
        None,
        "På Windows skal WGPU være standard (None lader Iced vælge standard WGPU hardware-acceleration)"
    );
    assert_eq!(
        resolve_default_iced_backend(true, Some("tiny-skia")),
        None,
        "Hvis brugeren manuelt sætter ICED_BACKEND=tiny-skia, skal dette bevares uden overstyring"
    );

    // AC2: Diagnostikmodul skal kunne tilgås og levere formateret rapport
    let diag = get_system_diagnostics();
    assert!(!diag.os.is_empty(), "OS skal være udfyldt");
    assert!(!diag.arch.is_empty(), "Arch skal være udfyldt");
    let report = diag.formatted_report();
    assert!(
        report.contains("Kant System- og Grafikdiagnostik"),
        "Rapport skal have overskrift"
    );
    assert!(
        report.contains(&diag.os),
        "Rapport skal indeholde operativsystem"
    );

    // AC2: In-memory log buffer
    log_info("diagnostics_test", "Acceptance test for Task 065 logging");
    let logs = get_recent_logs();
    assert!(
        logs.iter()
            .any(|entry| entry.contains("Acceptance test for Task 065 logging")),
        "Log-besked skal kunne findes i den seneste log-buffer"
    );

    // AC4: In-app modal integration
    let mut app = App::new_with_path(None);
    assert!(
        app.diagnostics_modal().is_none(),
        "Modal skal være lukket som standard"
    );
    let _ = app.update(Message::OpenDiagnosticsModal);
    assert!(
        app.diagnostics_modal().is_some(),
        "Modal skal være åben efter OpenDiagnosticsModal"
    );
    let _ = app.update(Message::CloseDiagnosticsModal);
    assert!(
        app.diagnostics_modal().is_none(),
        "Modal skal være lukket efter CloseDiagnosticsModal"
    );

    // AC5: Hærdet opak styling for card_container_style og floating_panel_style
    let theme = Theme::Light;
    let card_style = card_container_style(&theme);
    if let Some(Background::Color(c)) = card_style.background {
        assert_eq!(
            c.a, 1.0,
            "Card container skal have 100% opak baggrund for at forhindre sorte artefakter"
        );
    } else {
        panic!("Card container skal have en defineret baggrundsfarve");
    }

    let floating_style = floating_panel_style(&theme);
    if let Some(Background::Color(c)) = floating_style.background {
        assert_eq!(c.a, 1.0, "Floating panel skal have 100% opak baggrund");
    } else {
        panic!("Floating panel skal have en defineret baggrundsfarve");
    }
}

#[test]
fn test_task_066_startup_latency_profiling_and_windows_dx12_fastpath() {
    use kant::features::diagnostics::{
        get_recent_logs, get_system_diagnostics, log_info, record_startup_milestone,
    };
    use kant::ui::platform::resolve_default_wgpu_backend;

    // AC1: Relativ tidsstempling (+ms) i logbeskeder
    log_info(
        "startup_profile_test",
        "Testing relative elapsed milliseconds",
    );
    let logs = get_recent_logs();
    let profile_entry = logs
        .iter()
        .find(|l| l.contains("Testing relative elapsed milliseconds"))
        .expect("Log entry should be present in memory");
    assert!(
        profile_entry.contains("[+") && profile_entry.contains("ms]"),
        "Log entry skal indeholde relativ tid i millisekunder: {}",
        profile_entry
    );

    // AC2: WGPU DX12 fast-path på Windows
    assert_eq!(
        resolve_default_wgpu_backend(true, None),
        Some("dx12"),
        "På Windows skal WGPU_BACKEND automatisk sættes til dx12 hvis ikke sat"
    );
    assert_eq!(
        resolve_default_wgpu_backend(true, Some("vulkan")),
        None,
        "Hvis brugeren manuelt har sat WGPU_BACKEND=vulkan, må det ikke overskrives"
    );
    assert_eq!(
        resolve_default_wgpu_backend(false, None),
        None,
        "På Unix/macOS skal WGPU_BACKEND ikke gennemtvinge dx12"
    );

    // AC3: Milepælsregistrering og diagnostikrapport
    record_startup_milestone("model_loaded");
    record_startup_milestone("first_frame");
    let diag = get_system_diagnostics();
    let report = diag.formatted_report();
    assert!(
        report.contains("Opstartstid:") || report.contains("Opstartsmilepæle"),
        "Diagnostikrapport skal inkludere opstartsmetrikker"
    );
}

#[test]
fn test_task_042_concept_english_fields_support() {
    use kant::features::concepts::{BelongsToDomain, Concept, ConceptEnglishFields};
    use kant::features::model::merge::merge_models;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use kant::ui::concept_editor::{ConceptEditorState, ConceptFormField};
    use uuid::Uuid;

    // AC1: Domænemodel & Serde for ConceptEnglishFields
    let en_fields = ConceptEnglishFields {
        preferred_term: Some("Vehicle".to_string()),
        accepted_term: Some("Automobile".to_string()),
        deprecated_term: Some("Cart".to_string()),
        definition: Some("A means of transportation on wheels.".to_string()),
        example: Some("A private passenger car.".to_string()),
        comment: Some("Translates directly to Danish 'Køretøj'.".to_string()),
        application_note: Some("Use within core transportation domain.".to_string()),
    };

    let mut concept = Concept::new(
        "Køretøj",
        "Et transportmiddel på hjul til person- eller godstransport.",
        BelongsToDomain::Yes,
    );
    concept.set_english(Some(en_fields.clone()));

    assert_eq!(concept.english_preferred_term(), Some("Vehicle"));
    assert_eq!(
        concept.english_definition(),
        Some("A means of transportation on wheels.")
    );
    assert_eq!(concept.english(), Some(&en_fields));

    // Bagudkompatibel Serde: Gammel JSON uden 'english' felt skal deserialisere fejlfrit
    let legacy_json = serde_json::json!({
        "id": Uuid::new_v4(),
        "preferred_term": "Bygning",
        "definition": "En fast konstruktion til ophold.",
        "belongs_to_domain": "Yes"
    });
    let legacy_concept: Concept = serde_json::from_value(legacy_json)
        .expect("Skal deserialisere ældre model-filer uden fejl");
    assert!(
        legacy_concept.english().is_none(),
        "Legacy JSON skal give None for english"
    );

    // Fuld serialisering og deserialisering med engelske felter
    let serialized = serde_json::to_string(&concept).expect("Skal serialisere concept med english");
    let deserialized: Concept =
        serde_json::from_str(&serialized).expect("Skal deserialisere concept med english");
    assert_eq!(deserialized.english_preferred_term(), Some("Vehicle"));
    assert_eq!(deserialized.english(), Some(&en_fields));

    // AC2: 3-vejs merge engine integration med engelske felter
    let base_meta = ModelMetadata::new(
        "TestModel",
        "Desc",
        "https://example.com/model",
        "Org",
        "Domain",
        "1.0.0",
        ModelStatus::Draft,
    );
    let mut base_proj = ModelProject::new(base_meta.clone());
    let mut our_proj = ModelProject::new(base_meta.clone());
    let mut their_proj = ModelProject::new(base_meta);

    let c_id = Uuid::new_v4();
    let base_c = Concept::new_with_id(c_id, "Køretøj", "Def", BelongsToDomain::Yes);
    base_proj.add_concept(base_c).unwrap();

    let mut our_c = Concept::new_with_id(c_id, "Køretøj", "Def", BelongsToDomain::Yes);
    let our_en = ConceptEnglishFields {
        preferred_term: Some("Vehicle".to_string()),
        ..Default::default()
    };
    our_c.set_english(Some(our_en));
    our_proj.add_concept(our_c).unwrap();

    let their_c = Concept::new_with_id(
        c_id,
        "Køretøj",
        "Opdateret dansk definition",
        BelongsToDomain::Yes,
    );
    their_proj.add_concept(their_c).unwrap();

    let merge_result = merge_models(&base_proj, &our_proj, &their_proj);
    assert_eq!(
        merge_result.conflicts.len(),
        0,
        "Merge bør ikke have konflikter på separate felter"
    );
    let merged_c = merge_result
        .merged_project
        .get_concept(c_id)
        .expect("Concept findes");
    assert_eq!(merged_c.definition(), "Opdateret dansk definition");
    assert_eq!(merged_c.english_preferred_term(), Some("Vehicle"));

    // AC3: ConceptEditorState understøttelse
    let mut editor = ConceptEditorState::from_concept(&concept);
    assert_eq!(editor.english_preferred_term, "Vehicle");
    assert_eq!(
        editor.english_definition,
        "A means of transportation on wheels."
    );

    editor.update_field(
        ConceptFormField::EnglishPreferredTerm,
        "Automobile".to_string(),
    );
    let updated_concept = editor.build_concept().expect("Skal kunne bygge concept");
    assert_eq!(updated_concept.english_preferred_term(), Some("Automobile"));
}

#[test]
fn test_task_043_concept_list_excel_import_upsert_and_export() {
    use kant::features::concepts::excel::{
        apply_concept_upsert, export_project_concepts_to_xlsx, get_fda_schema_guide_entries,
        get_fda_template_xlsx_bytes, parse_concepts_from_xlsx_bytes,
    };
    use kant::features::concepts::{BelongsToDomain, Concept, ConceptEnglishFields};
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use uuid::Uuid;

    // AC4: Eksport af Tom FDA-skabelon (original officielle .xlsx)
    let template_bytes = get_fda_template_xlsx_bytes();
    assert!(
        template_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "Skabelonen skal være en gyldig ZIP/XLSX fil"
    );

    // Skabelonen kan parses og har de korrekte FDA-ark
    let template_summary = parse_concepts_from_xlsx_bytes(template_bytes)
        .expect("Den officielle skabelon skal kunne indlæses uden fejl");
    // Den tomme skabelon har ingen databegreber, kun overskrifter
    assert_eq!(template_summary.valid_concepts.len(), 0);

    // AC5: De Facto Dataeksport til Excel (.xlsx)
    let metadata = ModelMetadata::new(
        "Transportmodellen",
        "Officiel kernemodel for transport og køretøjer i Danmark",
        "https://data.gov.dk/model/core/transport",
        "Transportministeriet",
        "Mobilitet",
        "1.0.0",
        ModelStatus::Completed,
    );
    let mut project = ModelProject::new(metadata);

    let c1_id = Uuid::new_v4();
    let mut c1 = Concept::new_with_id(
        c1_id,
        "Køretøj",
        "Et transportmiddel på hjul.",
        BelongsToDomain::Yes,
    );
    c1.set_identifier(Some(
        "https://data.gov.dk/model/core/transport/Vehicle".to_string(),
    ));
    c1.set_english(Some(ConceptEnglishFields {
        preferred_term: Some("Vehicle".to_string()),
        definition: Some("A means of transport on wheels.".to_string()),
        ..Default::default()
    }));
    c1.set_legal_source(Some("Færdselsloven § 2".to_string()));
    project.add_concept(c1).unwrap();

    let c2_id = Uuid::new_v4();
    let c2 = Concept::new_with_id(
        c2_id,
        "Cykel",
        "Et tohjulet pedaldrevet køretøj.",
        BelongsToDomain::Yes,
    );
    project.add_concept(c2).unwrap();

    let exported_bytes =
        export_project_concepts_to_xlsx(&project).expect("Eksport til Excel skal lykkes");
    assert!(
        exported_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "Eksporten skal være en gyldig XLSX-fil"
    );

    // AC1, AC2: Parse den eksporterede fil
    let import_summary = parse_concepts_from_xlsx_bytes(&exported_bytes)
        .expect("Skal kunne parse den genererede XLSX fil");
    assert_eq!(import_summary.valid_concepts.len(), 2);
    assert_eq!(import_summary.rejected_rows.len(), 0);

    // AC3: Test Intelligent Upsert
    let mut imported_concepts = import_summary.valid_concepts;
    // Opdater definitionen på Køretøj
    let koeretoej = imported_concepts
        .iter_mut()
        .find(|c| c.preferred_term() == "Køretøj")
        .expect("Køretøj skal findes i importerede begreber");
    koeretoej.set_definition("Opdateret definition af køretøj via Excel import.");

    // Nyt begreb der skal indsættes
    let new_c = Concept::new(
        "El-løbehjul",
        "Et lille elkøretøj med to hjul og et styr.",
        BelongsToDomain::Yes,
    );
    imported_concepts.push(new_c);

    let upsert_report = apply_concept_upsert(&mut project, imported_concepts);
    assert_eq!(
        upsert_report.updated, 2,
        "Begge eksisterende begreber skal opdateres"
    );
    assert_eq!(upsert_report.inserted, 1, "Et nyt begreb skal indsættes");

    // Bevaring af UUID invariant (Must NOT)
    let updated_c1 = project
        .get_concept(c1_id)
        .expect("Køretøj skal stadig findes med samme UUID");
    assert_eq!(
        updated_c1.definition(),
        "Opdateret definition af køretøj via Excel import."
    );
    assert_eq!(updated_c1.english_preferred_term(), Some("Vehicle"));

    // AC6: Visuel Skemaguide
    let schema_guide = get_fda_schema_guide_entries();
    assert!(!schema_guide.is_empty());
    assert!(schema_guide
        .iter()
        .any(|e| e.name == "Foretrukken term" && e.required));
    assert!(schema_guide
        .iter()
        .any(|e| e.name == "Definition" && e.required));
}

#[test]
fn test_task_068_in_app_header_logo_and_os_icons() {
    use kant::ui::app::load_window_icon;
    use std::path::Path;

    // 1. Verificer at det nye logo assets/kant_logo.svg eksisterer og har forventet indhold
    let logo_path = Path::new("assets/kant_logo.svg");
    assert!(logo_path.exists(), "assets/kant_logo.svg skal eksistere");
    let logo_svg = std::fs::read_to_string(logo_path).expect("skal kunne læse kant_logo.svg");
    assert!(
        logo_svg.contains("<svg") && logo_svg.contains("rx=\"6\""),
        "kant_logo.svg skal være et gyldigt SVG badge"
    );

    // 2. Verificer at assets/icons/kant.svg er synkroniseret med assets/kant_logo.svg
    let icon_svg_path = Path::new("assets/icons/kant.svg");
    assert!(
        icon_svg_path.exists(),
        "assets/icons/kant.svg skal eksistere"
    );
    let icon_svg =
        std::fs::read_to_string(icon_svg_path).expect("skal kunne læse assets/icons/kant.svg");
    assert_eq!(
        icon_svg.trim(),
        logo_svg.trim(),
        "assets/icons/kant.svg skal matche assets/kant_logo.svg"
    );

    // 3. Verificer kant-32.rgba: skal være 32x32x4 = 4096 bytes og genereret fra nyt logo
    let rgba_path = Path::new("assets/icons/kant-32.rgba");
    assert!(
        rgba_path.exists(),
        "assets/icons/kant-32.rgba skal eksistere"
    );
    let rgba_bytes = std::fs::read(rgba_path).expect("skal kunne læse kant-32.rgba");
    assert_eq!(
        rgba_bytes.len(),
        32 * 32 * 4,
        "kant-32.rgba skal have præcis 4096 bytes"
    );

    // 4. Verificer runtime vinduesikon
    let window_icon = load_window_icon();
    assert!(
        window_icon.is_some(),
        "load_window_icon() skal returnere et gyldigt vinduesikon"
    );

    // 5. Verificer kant.ico: skal være gyldig ICO header og indeholde flere billeder
    let ico_path = Path::new("assets/icons/kant.ico");
    assert!(ico_path.exists(), "assets/icons/kant.ico skal eksistere");
    let ico_bytes = std::fs::read(ico_path).expect("skal kunne læse kant.ico");
    assert!(ico_bytes.len() >= 6, "kant.ico skal have gyldig header");
    assert_eq!(&ico_bytes[0..4], &[0x00, 0x00, 0x01, 0x00]);
    let image_count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]);
    assert!(image_count >= 1, "kant.ico skal indeholde mindst 1 billede");
}

#[test]
fn test_task_068_palette_ergonomics_and_header_rendering() {
    use kant::ui::app::{App, Message, Tab};

    let mut app = App::new();
    assert!(
        app.is_left_sidebar_visible(),
        "Venstrepalet skal som udgangspunkt være synlig"
    );

    // 1. Rendering med palet synlig
    {
        let _ = app.view();
    }

    // 2. Toggle skjul palet via Message::ToggleLeftSidebar (f.eks. ved klik på kollaps eller Ctrl+B)
    let _ = app.update(Message::ToggleLeftSidebar);
    assert!(
        !app.is_left_sidebar_visible(),
        "Venstrepalet skal nu være skjult"
    );

    // 3. Rendering med palet skjult på tværs af faner
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    {
        let _ = app.view();
    }

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    {
        let _ = app.view();
    }

    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    {
        let _ = app.view();
    }

    // 4. Toggle tilbage til synlig palet
    let _ = app.update(Message::ToggleLeftSidebar);
    assert!(
        app.is_left_sidebar_visible(),
        "Venstrepalet skal være synlig igen"
    );
    {
        let _ = app.view();
    }
}

#[test]
fn test_task_069_enter_quick_edit_on_canvas_node_and_edge() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{App, Message, Tab};

    let mut app = App::new();
    let cid1 = app
        .project_mut()
        .add_concept(Concept::new("A", "Beskrivelse A", BelongsToDomain::Yes))
        .unwrap();
    let _ = app.update(Message::AddConceptToDiagram(cid1));
    let nid1 = app
        .project()
        .concept_graph()
        .find_node_by_concept(cid1)
        .unwrap()
        .id();

    let cid2 = app
        .project_mut()
        .add_concept(Concept::new("B", "Beskrivelse B", BelongsToDomain::Yes))
        .unwrap();
    let _ = app.update(Message::AddConceptToDiagram(cid2));
    let nid2 = app
        .project()
        .concept_graph()
        .find_node_by_concept(cid2)
        .unwrap()
        .id();

    let _ = app.update(Message::GraphEdgeCreated(nid1, nid2));
    let _ = app.update(Message::GraphUpdateEdgeLabel(
        nid1,
        nid2,
        "relaterer til".to_string(),
    ));

    // 1. Enter på valgt node åbner inline editor for preferred_term
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphNodeSelected(Some(nid1)));
    assert_eq!(app.selected_graph_node_id(), Some(nid1));
    assert!(
        !app.is_node_editing(),
        "Skal ikke være i inline redigering før Enter"
    );

    let _ = app.update(Message::EnterPressed);
    assert!(
        app.is_node_editing(),
        "Enter på valgt node skal åbne inline editor og sætte is_node_editing til true"
    );

    // Luk editoren igen med Esc
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_node_editing(), "Esc skal lukke inline editor");
    assert_eq!(
        app.selected_graph_node_id(),
        Some(nid1),
        "Node forbliver valgt efter inline editor lukkes"
    );

    // 2. Enter på valgt relation
    let _ = app.update(Message::GraphEdgeSelected(Some((nid1, nid2))));
    assert_eq!(app.selected_edge(), Some((nid1, nid2)));
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_edge(),
        Some((nid1, nid2)),
        "Relationen forbliver valgt og klar til input"
    );

    // 3. Enter i Informationsmodel (Klasse)
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::CreateInformationClass);
    let info_node = app.selected_info_graph_node_id();
    assert!(
        info_node.is_some(),
        "Informationsklasse node skal være oprettet og valgt"
    );
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_info_graph_node_id(),
        info_node,
        "Info node forbliver valgt ved Enter"
    );

    // 4. Enter på enumeration node i Informationsmodel
    let enum_id = app.project_mut().information_model_mut().add_enumeration(
        kant::features::information_model::InformationEnumeration::new("Status", Vec::new()),
    );
    let enum_node_id = app
        .project_mut()
        .information_graph_mut()
        .add_node(enum_id, 0);
    let _ = app.update(Message::SelectInfoGraphNode(Some(enum_node_id)));
    assert_eq!(app.selected_info_enum_id(), Some(enum_id));
    assert_eq!(app.selected_info_class_id(), None);
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_info_enum_id(),
        Some(enum_id),
        "Enumeration skal forblive valgt"
    );
    assert_eq!(
        app.selected_info_class_id(),
        None,
        "Må ikke overskrive selected_info_class_id med enumeration id"
    );
}

#[test]
fn test_task_069_escape_hierarchy_all_nine_modals_closing() {
    use kant::features::git::GitService;
    use kant::ui::app::{App, Message};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_069_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");

    let mut app = App::new_with_path(Some(temp_dir.clone()));

    // 1. Diagnostics Modal
    let _ = app.update(Message::OpenDiagnosticsModal);
    assert!(
        app.diagnostics_modal().is_some(),
        "Diagnostics modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.diagnostics_modal().is_none(),
        "Diagnostics modal skal lukkes på Esc"
    );

    // 2. Publish Modal
    let _ = app.update(Message::OpenPublishModal);
    assert!(
        app.publish_modal().is_some(),
        "Publish modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.publish_modal().is_none(),
        "Publish modal skal lukkes på Esc"
    );

    // 3. Model History Modal
    let _ = app.update(Message::OpenModelHistoryModal);
    assert!(
        app.history_modal().is_some(),
        "History modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.history_modal().is_none(),
        "History modal skal lukkes på Esc"
    );

    // 4. Conflict Resolver Modal
    let _ = app.update(Message::OpenConflictResolverModal(vec![]));
    assert!(
        app.conflict_resolver_modal().is_some(),
        "Conflict resolver modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.conflict_resolver_modal().is_none(),
        "Conflict resolver modal skal lukkes på Esc"
    );

    // 5. Git Connection Modal
    let _ = app.update(Message::OpenGitConnectionModal);
    assert!(
        app.git_connection_modal().is_some(),
        "Git connection modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.git_connection_modal().is_none(),
        "Git connection modal skal lukkes på Esc"
    );

    // 6. Git Clone Modal
    let _ = app.update(Message::OpenGitCloneModal);
    assert!(
        app.git_clone_modal().is_some(),
        "Git clone modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.git_clone_modal().is_none(),
        "Git clone modal skal lukkes på Esc"
    );

    // 7. Metadata Modal
    let _ = app.update(Message::OpenMetadataModal);
    assert!(
        app.metadata_modal().is_some(),
        "Metadata modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.metadata_modal().is_none(),
        "Metadata modal skal lukkes på Esc"
    );

    // Bonus: Excel Schema Guide Modal
    let _ = app.update(Message::OpenExcelSchemaGuideModal);
    assert!(
        app.is_excel_schema_guide_open(),
        "Excel schema guide modal skal være åben"
    );
    let _ = app.update(Message::EscapePressed);
    assert!(
        !app.is_excel_schema_guide_open(),
        "Excel schema guide modal skal lukkes på Esc"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_069_escape_hierarchy_layer_by_layer() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{App, Message, Tab};

    let mut app = App::new();
    let cid = app
        .project_mut()
        .add_concept(Concept::new("Patient", "Definition", BelongsToDomain::Yes))
        .unwrap();
    let _ = app.update(Message::AddConceptToDiagram(cid));
    let nid = app
        .project()
        .concept_graph()
        .find_node_by_concept(cid)
        .unwrap()
        .id();

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphNodeSelected(Some(nid)));
    // Simuler inline redigering
    let _ = app.update(Message::GraphNodeDoubleClicked(nid));
    assert!(app.is_node_editing(), "Inline editor skal være aktiv");
    assert_eq!(app.selected_graph_node_id(), Some(nid));

    // Lag 2: 1. Esc lukker inline editor, men beholder nodemarkering
    let _ = app.update(Message::EscapePressed);
    assert!(!app.is_node_editing(), "Inline editor skal være lukket");
    assert_eq!(
        app.selected_graph_node_id(),
        Some(nid),
        "Node forbliver markeret"
    );

    // Lag 4: 2. Esc fjerner nodemarkering på canvas
    let _ = app.update(Message::EscapePressed);
    assert_eq!(
        app.selected_graph_node_id(),
        None,
        "Canvas markering skal være ryddet"
    );
    assert!(app.selected_graph_node_ids().is_empty());

    // Lag 5: 3. Esc lander på fanebladene (skal køre uden panik og bevare en konsistent tilstand)
    let _ = app.update(Message::EscapePressed);
    assert_eq!(app.active_tab(), Tab::ConceptModel);
}

#[test]
fn test_task_069_tab_direct_shortcuts_ctrl_1_2_3() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    use kant::ui::app::{App, Message, Tab};

    let mut app = App::new();
    assert_eq!(app.active_tab(), Tab::ConceptList);

    // Ctrl+2: Begrebsdiagram
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(app.active_tab(), Tab::ConceptModel);

    // Ctrl+3: Informationsmodel
    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    assert_eq!(app.active_tab(), Tab::InformationModel);

    // Ctrl+1: Begrebsliste
    let _ = app.update(Message::SelectTab(Tab::ConceptList));
    assert_eq!(app.active_tab(), Tab::ConceptList);

    // Test map_key_press genveje
    assert!(matches!(
        App::map_key_press(&Key::Named(Named::Enter), Modifiers::empty()),
        Some(Message::EnterPressed)
    ));
    assert!(matches!(
        App::map_key_press(&Key::Named(Named::Escape), Modifiers::empty()),
        Some(Message::EscapePressed)
    ));
    assert!(matches!(
        App::map_key_press(&Key::Character("1".into()), Modifiers::CTRL),
        Some(Message::SelectTab(Tab::ConceptList))
    ));
    assert!(matches!(
        App::map_key_press(&Key::Character("2".into()), Modifiers::CTRL),
        Some(Message::SelectTab(Tab::ConceptModel))
    ));
    assert!(matches!(
        App::map_key_press(&Key::Character("3".into()), Modifiers::CTRL),
        Some(Message::SelectTab(Tab::InformationModel))
    ));
}

#[test]
fn test_task_070_concept_editor_tab_cycle_to_buttons() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{App, Message};
    use kant::ui::concept_editor::{BTN_CANCEL, BTN_DOMAIN_BORROWED, BTN_DOMAIN_LOCAL, BTN_SAVE};

    let mut app = App::new();
    let cid = app
        .project_mut()
        .add_concept(Concept::new(
            "Bil",
            "Et firehjulet motorkøretøj",
            BelongsToDomain::Yes,
        ))
        .unwrap();

    let _ = app.update(Message::EditConcept(cid));
    assert!(
        app.concept_editor().is_some(),
        "ConceptEditor skal være åben"
    );

    // 1. Verificer at startfokus er i første tekstfelt (preferred_term_input)
    assert_eq!(
        app.editor_focused_button(),
        None,
        "Startfokus er i tekstfelt"
    );

    // 2. Første Tab rykker til 'Lokalt begreb (Ja)' knappen
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.editor_focused_button(),
        Some(BTN_DOMAIN_LOCAL),
        "Tab skal rykke fokus til 'Lokalt begreb (Ja)' knappen"
    );

    // 3. Næste Tab rykker til 'Indlånt begreb (Nej)' knappen
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.editor_focused_button(),
        Some(BTN_DOMAIN_BORROWED),
        "Tab skal rykke fokus til 'Indlånt begreb (Nej)' knappen"
    );

    // 4. Enter på 'Indlånt begreb (Nej)' aktiverer knappen og sætter domæne til No
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.concept_editor().unwrap().belongs_to_domain,
        BelongsToDomain::No,
        "Enter på fokuseret 'Indlånt begreb' skal ændre domænet til No"
    );

    // 5. Shift+Tab går baglæns til 'Lokalt begreb (Ja)' knappen
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.editor_focused_button(),
        Some(BTN_DOMAIN_LOCAL),
        "Shift+Tab skal gå baglæns til Lokalt begreb"
    );

    // 6. Enter på Lokalt begreb sætter domænet tilbage til Yes
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.concept_editor().unwrap().belongs_to_domain,
        BelongsToDomain::Yes,
        "Enter på fokuseret 'Lokalt begreb' skal sætte domænet tilbage til Yes"
    );

    // 7. Tab fremad igennem felterne indtil Annuller (BTN_CANCEL)
    let mut reached_cancel = false;
    for _ in 0..20 {
        let _ = app.update(Message::FocusNext);
        if app.editor_focused_button() == Some(BTN_CANCEL) {
            reached_cancel = true;
            break;
        }
    }
    assert!(reached_cancel, "Tab skal nå Annuller-knappen");

    // 8. Næste Tab skal nå Gem ændringer (BTN_SAVE)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.editor_focused_button(),
        Some(BTN_SAVE),
        "Tab skal nå Gem-knappen"
    );

    // 9. Næste Tab skal loope tilbage til tekstfelterne (fokus_button == None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.editor_focused_button(),
        None,
        "Tab fra Gem skal loope tilbage til tekstfelterne"
    );

    // 10. Enter på fokuseret Annuller lukker editoren
    let _ = app.update(Message::FocusEditorButton(Some(BTN_CANCEL)));
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.concept_editor().is_none(),
        "Enter på fokuseret Annuller skal lukke editoren"
    );
}

#[test]
fn test_task_070_subscription_handles_captured_tab_and_escape() {
    use iced::keyboard::{key::Named, key::NativeCode, Key, Location, Modifiers};
    use iced::{event::Status, Event};
    use kant::ui::app::{App, Message};

    // 1. Tab med Status::Captured skal opfanges som FocusNext
    let tab_event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Tab),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
        text: None,
        modified_key: Key::Named(Named::Tab),
        physical_key: iced::keyboard::key::Physical::Unidentified(NativeCode::Unidentified),
    });
    assert!(
        matches!(
            App::handle_event_for_subscription(tab_event, Status::Captured),
            Some(Message::FocusNext)
        ),
        "Tab under Status::Captured (i et tekstfelt) skal give Message::FocusNext"
    );

    // 2. Shift+Tab med Status::Captured skal opfanges som FocusPrevious
    let shift_tab_event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Tab),
        location: Location::Standard,
        modifiers: Modifiers::SHIFT,
        repeat: false,
        text: None,
        modified_key: Key::Named(Named::Tab),
        physical_key: iced::keyboard::key::Physical::Unidentified(NativeCode::Unidentified),
    });
    assert!(
        matches!(
            App::handle_event_for_subscription(shift_tab_event, Status::Captured),
            Some(Message::FocusPrevious)
        ),
        "Shift+Tab under Status::Captured skal give Message::FocusPrevious"
    );

    // 3. Escape med Status::Captured skal opfanges som EscapePressed
    let esc_event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Escape),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
        text: None,
        modified_key: Key::Named(Named::Escape),
        physical_key: iced::keyboard::key::Physical::Unidentified(NativeCode::Unidentified),
    });
    assert!(
        matches!(
            App::handle_event_for_subscription(esc_event, Status::Captured),
            Some(Message::EscapePressed)
        ),
        "Escape under Status::Captured skal give Message::EscapePressed"
    );

    // 4. Almindelige bogstaver må IKKE opfanges når Status::Captured (tekstindtastning bevares)
    let char_event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Character("a".into()),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
        repeat: false,
        text: Some("a".into()),
        modified_key: Key::Character("a".into()),
        physical_key: iced::keyboard::key::Physical::Unidentified(NativeCode::Unidentified),
    });
    assert!(
        App::handle_event_for_subscription(char_event, Status::Captured).is_none(),
        "Almindelige tastetryk under Status::Captured må ikke afbryde tekstinput"
    );
}

#[test]
fn test_task_070_git_connection_modal_tab_cycle_and_escape() {
    use kant::ui::app::{
        App, Message, GIT_CONN_BTN_BROWSE, GIT_CONN_BTN_CANCEL, GIT_CONN_BTN_SAVE,
    };

    let mut app = App::new();
    let _ = app.update(Message::OpenGitConnectionModal);
    assert!(
        app.git_connection_modal().is_some(),
        "Git indstillinger modal skal være åben"
    );

    // Start-tilstand: første input er i fokus, ingen knap i fokus
    assert_eq!(
        app.modal_focused_button_index(),
        None,
        "Ved modal åbning skal tekstfelt have fokus, ikke knap"
    );

    // Tab 1: git_target_dir
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 2: GIT_CONN_BTN_BROWSE knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(GIT_CONN_BTN_BROWSE),
        "Tab skal nå Gennemse-knappen"
    );

    // Tab 3: git_token
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 4: git_author_name
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 5: git_author_email
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 6: GIT_CONN_BTN_CANCEL
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(GIT_CONN_BTN_CANCEL),
        "Tab skal nå Annuller-knappen"
    );

    // Tab 7: GIT_CONN_BTN_SAVE
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(GIT_CONN_BTN_SAVE),
        "Tab skal nå Gem-knappen"
    );

    // Tab 8: Loops tilbage til første input (git_remote_url)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.modal_focused_button_index(),
        None,
        "Tab fra Gem-knap skal loope tilbage til første tekstfelt"
    );

    // Shift+Tab fra første input looper baglæns til GIT_CONN_BTN_SAVE
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(GIT_CONN_BTN_SAVE),
        "Shift+Tab fra første input skal loope til Gem-knap"
    );

    // Shift+Tab til GIT_CONN_BTN_CANCEL
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(app.modal_focused_button_index(), Some(GIT_CONN_BTN_CANCEL));

    // Enter på Annuller lukker modalen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.git_connection_modal().is_none(),
        "Enter på fokuseret Annuller-knap skal lukke Git modalen"
    );
}

#[test]
fn test_task_070_git_clone_modal_tab_cycle_and_cancel() {
    use kant::ui::app::{
        App, Message, GIT_CLONE_BTN_BROWSE, GIT_CLONE_BTN_CANCEL, GIT_CLONE_BTN_CLONE,
    };

    let mut app = App::new();
    let _ = app.update(Message::OpenGitCloneModal);
    assert!(
        app.git_clone_modal().is_some(),
        "Git clone modal skal være åben"
    );

    // Start: inputfelt
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 1: git_clone_target_dir
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 2: GIT_CLONE_BTN_BROWSE
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(GIT_CLONE_BTN_BROWSE));

    // Tab 3: git_clone_token
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 4: GIT_CLONE_BTN_CANCEL
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(GIT_CLONE_BTN_CANCEL));

    // Tab 5: GIT_CLONE_BTN_CLONE
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(GIT_CLONE_BTN_CLONE));

    // Tab 6: Loop tilbage til URL
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Enter på Annuller lukker modalen
    let _ = app.update(Message::FocusPrevious); // CLONE
    let _ = app.update(Message::FocusPrevious); // CANCEL
    assert_eq!(app.modal_focused_button_index(), Some(GIT_CLONE_BTN_CANCEL));
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.git_clone_modal().is_none(),
        "Enter på Annuller skal lukke Clone modalen"
    );
}

#[test]
fn test_task_070_metadata_modal_tab_cycle_and_cancel() {
    use kant::ui::app::{App, Message, META_BTN_CANCEL, META_BTN_SAVE};

    let mut app = App::new();
    let _ = app.update(Message::OpenMetadataModal);
    assert!(
        app.metadata_modal().is_some(),
        "Metadata modal skal være åben"
    );

    // Start: inputfelt meta_name
    assert_eq!(app.modal_focused_button_index(), None);

    // Shift+Tab fra start looper direkte til META_BTN_SAVE
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(META_BTN_SAVE),
        "Shift+Tab fra start looper til Gem-knap"
    );

    // Shift+Tab til META_BTN_CANCEL
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(META_BTN_CANCEL),
        "Shift+Tab fra Gem går til Annuller-knap"
    );

    // Enter på Annuller lukker metadata modalen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.metadata_modal().is_none(),
        "Enter på fokuseret Annuller skal lukke Metadata modalen"
    );
}

#[test]
fn test_task_070_file_dialog_modal_tab_cycle_and_cancel() {
    use kant::ui::app::{App, FileDialogMode, Message};

    let mut app = App::new();
    let _ = app.update(Message::OpenInlineFileDialog(FileDialogMode::Open));
    assert!(
        app.is_file_dialog_open(),
        "File dialog modal skal være åben"
    );

    // Start på inputfeltet
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab 1: Annuller (indeks 0)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(0));

    // Tab 2: Bekræft (indeks 1)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(1));

    // Tab 3: Tilbage til inputfeltet
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), None);

    // Tab til Annuller og tryk Enter
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.modal_focused_button_index(), Some(0));
    let _ = app.update(Message::EnterPressed);
    assert!(
        !app.is_file_dialog_open(),
        "Enter på Annuller skal lukke fil-dialogen"
    );
}

#[test]
fn test_task_071_toolbar_tab_cycle_and_activation() {
    use kant::ui::app::{
        App, Message, Tab, TOOLBAR_BTN_EXPORT_EXCEL, TOOLBAR_BTN_GUIDE, TOOLBAR_BTN_IMPORT_EXCEL,
        TOOLBAR_BTN_NEW_CONCEPT, TOOLBAR_BTN_TEMPLATE,
    };

    let mut app = App::new();
    assert_eq!(app.active_tab(), Tab::ConceptList);

    // Initialt er ingen værktøjsknap fokuseret (tekstfelt har fokus eller ingen knap)
    assert_eq!(app.toolbar_focused_button(), None);

    // Tab 1 -> + Nyt Begreb
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_NEW_CONCEPT));

    // Tab 2 -> Importér Excel
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_IMPORT_EXCEL));

    // Tab 3 -> Eksportér Excel
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_EXPORT_EXCEL));

    // Tab 4 -> Hent skabelon
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_TEMPLATE));

    // Tab 5 -> Skemaguide
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_GUIDE));

    // Tab 6 -> Looper tilbage til søgefeltet (None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), None);

    // Shift+Tab fra søgefeltet looper baglæns til Skemaguide
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_GUIDE));

    // Enter på Skemaguide åbner modalen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.is_excel_schema_guide_open(),
        "Enter på fokuseret Skemaguide-knap skal åbne guiden"
    );
    let _ = app.update(Message::CloseExcelSchemaGuideModal);
    assert!(!app.is_excel_schema_guide_open());

    // Tab til + Nyt Begreb og tryk Enter -> åbner concept_editor
    let _ = app.update(Message::FocusNext); // søgefelt (None)
    let _ = app.update(Message::FocusNext); // + Nyt Begreb
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_NEW_CONCEPT));
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.concept_editor().is_some(),
        "Enter på + Nyt Begreb skal åbne ConceptEditor"
    );
}

#[test]
fn test_task_071_new_project_initializes_modal_focus_on_meta_name() {
    use kant::ui::app::{App, Message, META_BTN_CANCEL, META_BTN_SAVE};

    let mut app = App::new();
    let _ = app.update(Message::NewProject);
    assert!(
        app.metadata_modal().is_some(),
        "Nyt projekt skal åbne metadata modalen"
    );

    // Start-tilstand: meta_name har fokus, ingen knap fokuseret
    assert_eq!(
        app.modal_focused_button_index(),
        None,
        "Startfokus skal være i meta_name tekstfelt"
    );

    // Shift+Tab looper direkte baglæns til META_BTN_SAVE
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(META_BTN_SAVE),
        "Shift+Tab fra meta_name skal loope til Gem-knap"
    );

    // Shift+Tab til META_BTN_CANCEL
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.modal_focused_button_index(),
        Some(META_BTN_CANCEL),
        "Shift+Tab fra Gem skal gå til Annuller-knap"
    );

    // Enter på Annuller lukker modalen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.metadata_modal().is_none(),
        "Enter på Annuller skal lukke modalen"
    );
}

#[test]
fn test_task_071_history_modal_focuses_close_button_and_enter_closes() {
    use kant::features::git::GitService;
    use kant::ui::app::{App, Message};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_071_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let mut app = App::new_with_path(Some(temp_dir.clone()));

    let _ = app.update(Message::OpenModelHistoryModal);
    assert!(
        app.history_modal().is_some(),
        "History modal skal være åben"
    );
    assert_eq!(
        app.modal_focused_button_index(),
        Some(0),
        "Luk-knappen i history_modal skal have startfokus"
    );

    let _ = app.update(Message::EnterPressed);
    assert!(
        app.history_modal().is_none(),
        "Enter på Luk-knappen skal lukke history modal"
    );
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_071_inline_graph_editing_inspector_tab_cycle_and_cancel() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{App, Message, Tab};
    use kant::ui::concept_editor::{BTN_CANCEL, BTN_SAVE};

    let mut app = App::new();
    let cid = app
        .project_mut()
        .add_concept(Concept::new("Bil", "Et køretøj", BelongsToDomain::Yes))
        .unwrap();
    let _ = app.update(Message::AddConceptToDiagram(cid));
    let nid = app
        .project()
        .concept_graph()
        .find_node_by_concept(cid)
        .unwrap()
        .id();

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphNodeSelected(Some(nid)));
    let _ = app.update(Message::EnterPressed);
    assert!(app.is_node_editing(), "Skal være i inline redigering");

    // 1. Startfokus er i preferred_term_input (None fokuseret knap)
    assert_eq!(app.editor_focused_button(), None);

    // 2. Tab 1 -> inline_concept_definition (tekstfelt, stadig None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.editor_focused_button(), None);

    // 3. Tab 2 -> Annuller knappen (BTN_CANCEL)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.editor_focused_button(), Some(BTN_CANCEL));

    // 4. Tab 3 -> Gem Begreb knappen (BTN_SAVE)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.editor_focused_button(), Some(BTN_SAVE));

    // 5. Tab 4 -> Loops tilbage til preferred_term_input (None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.editor_focused_button(), None);

    // 6. Shift+Tab -> Gem Begreb knappen
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(app.editor_focused_button(), Some(BTN_SAVE));

    // 7. Shift+Tab -> Annuller knappen
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(app.editor_focused_button(), Some(BTN_CANCEL));

    // 8. Enter på Annuller lukker inline redigeringen
    let _ = app.update(Message::EnterPressed);
    assert!(
        !app.is_node_editing(),
        "Enter på Annuller skal lukke inline redigering"
    );
}

#[test]
fn test_task_071_information_model_inspector_tab_cycle_and_actions() {
    use kant::features::information_model::InformationClass;
    use kant::ui::app::{
        App, Message, Tab, INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX, INFO_INSPECTOR_BTN_CANVAS_ACTION,
        INFO_INSPECTOR_BTN_DELETE_CLASS, INFO_INSPECTOR_BTN_TYPE_BORROWED,
        INFO_INSPECTOR_BTN_TYPE_LOCAL,
    };

    let mut app = App::new();
    let class = InformationClass::new("Kunde");
    let class_id = class.id();
    app.project_mut().information_model_mut().add_class(class);

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::SelectInformationClass(Some(class_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_id));

    // Startfokus er på info_class_name_input (tekstfelt, knap == None)
    assert_eq!(app.info_inspector_focused_button(), None);

    // Tab 1 -> info_class_desc_input (tekstfelt, knap == None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.info_inspector_focused_button(), None);

    // Tab 2 -> Abstrakt klasse checkbox
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX)
    );

    // Tab 3 -> Lokal type knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_TYPE_LOCAL)
    );

    // Tab 4 -> Indlånt type knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_TYPE_BORROWED)
    );

    // Tab 5 -> INFO_INSPECTOR_BTN_CANVAS_ACTION (Tilføj til diagram)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_CANVAS_ACTION)
    );

    // Tab 6 -> INFO_INSPECTOR_BTN_DELETE_CLASS (Slet klasse)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_DELETE_CLASS)
    );

    // Tab 7 -> INFO_INSPECTOR_BTN_ADD_ATTRIBUTE (+ Tilføj attribut)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(kant::ui::app::INFO_INSPECTOR_BTN_ADD_ATTRIBUTE)
    );

    // Tab 8 -> Loops tilbage til info_class_name_input (None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.info_inspector_focused_button(), None);

    // Shift+Tab -> INFO_INSPECTOR_BTN_ADD_ATTRIBUTE
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(kant::ui::app::INFO_INSPECTOR_BTN_ADD_ATTRIBUTE)
    );

    // Shift+Tab -> INFO_INSPECTOR_BTN_DELETE_CLASS
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_DELETE_CLASS)
    );

    // Shift+Tab -> INFO_INSPECTOR_BTN_CANVAS_ACTION
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_CANVAS_ACTION)
    );

    // Enter på Tilføj til diagram tilføjer klassen til diagrammet
    assert!(!app
        .project()
        .information_graph()
        .is_class_on_diagram(class_id));
    let _ = app.update(Message::EnterPressed);
    assert!(app
        .project()
        .information_graph()
        .is_class_on_diagram(class_id));
}

#[test]
fn test_task_072_concept_table_rows_tab_cycle_and_activation() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{App, Message, Tab, TOOLBAR_BTN_GUIDE, TOOLBAR_ROW_BTN_START};

    let mut app = App::new();
    let c1 = Concept::new(
        "Aftale",
        "En juridisk bindende aftale",
        BelongsToDomain::Yes,
    );
    let c2 = Concept::new("Kunde", "En modtager af ydelser", BelongsToDomain::Yes);
    let id1 = c1.id();
    let id2 = c2.id();
    app.project_mut().add_concept(c1).unwrap();
    app.project_mut().add_concept(c2).unwrap();

    assert_eq!(app.active_tab(), Tab::ConceptList);

    // Initialt er ingen værktøjsknap fokuseret (tekstfelt har fokus)
    assert_eq!(app.toolbar_focused_button(), None);

    // Tab 1..5 -> Værktøjsknapper (+ Nyt Begreb, Importér, Eksportér, Skabelon, Guide)
    for _ in 0..5 {
        let _ = app.update(Message::FocusNext);
    }
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_BTN_GUIDE));

    // Tab 6 -> Række 0 Rediger-knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_ROW_BTN_START));

    // Tab 7 -> Række 0 Slet-knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.toolbar_focused_button(),
        Some(TOOLBAR_ROW_BTN_START + 1)
    );

    // Tab 8 -> Række 1 Rediger-knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.toolbar_focused_button(),
        Some(TOOLBAR_ROW_BTN_START + 2)
    );

    // Tab 9 -> Række 1 Slet-knap
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.toolbar_focused_button(),
        Some(TOOLBAR_ROW_BTN_START + 3)
    );

    // Tab 10 -> Looper tilbage til søgefeltet
    let _ = app.update(Message::FocusNext);
    assert_eq!(app.toolbar_focused_button(), None);

    // Shift+Tab looper baglæns til Række 1 Slet-knap
    let _ = app.update(Message::FocusPrevious);
    assert_eq!(
        app.toolbar_focused_button(),
        Some(TOOLBAR_ROW_BTN_START + 3)
    );

    // Enter på Række 1 Slet sletter begreb 2
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.project().concepts().len(),
        1,
        "Række 1 skal være slettet"
    );
    assert!(app.project().get_concept(id2).is_none());
    assert!(app.project().get_concept(id1).is_some());

    // Tab til Række 0 Rediger og tryk Enter -> åbner ConceptEditor med id1
    let _ = app.update(Message::FocusNext); // søgefelt
    for _ in 0..5 {
        let _ = app.update(Message::FocusNext); // værktøjsknapper
    }
    let _ = app.update(Message::FocusNext); // Række 0 Rediger
    assert_eq!(app.toolbar_focused_button(), Some(TOOLBAR_ROW_BTN_START));
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.concept_editor().is_some(),
        "Enter på Rediger skal åbne ConceptEditor"
    );
    assert_eq!(app.concept_editor().unwrap().preferred_term, "Aftale");
}

#[test]
fn test_task_072_history_modal_esc_and_close_button() {
    use kant::features::git::GitService;
    use kant::ui::app::{App, Message, HISTORY_BTN_CLOSE};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_072_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let mut app = App::new_with_path(Some(temp_dir.clone()));

    let _ = app.update(Message::OpenModelHistoryModal);
    assert!(app.history_modal().is_some());

    // Startfokus er på Luk-knappen i bunden
    assert_eq!(
        app.modal_focused_button_index(),
        Some(HISTORY_BTN_CLOSE),
        "Startfokus skal være på Luk-knappen i bunden"
    );

    // Esc lukker modalen
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.history_modal().is_none(),
        "Escape skal lukke historik-modalen"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_072_information_model_inspector_full_properties_cycle() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::InformationClass;
    use kant::ui::app::{
        App, Message, Tab, INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX, INFO_INSPECTOR_BTN_ADD_ATTRIBUTE,
        INFO_INSPECTOR_BTN_TYPE_BORROWED, INFO_INSPECTOR_BTN_TYPE_LOCAL,
    };

    let mut app = App::new();
    let mut class = InformationClass::new("Kunde");
    let cid = app
        .project_mut()
        .add_concept(Concept::new("Kunde", "Person", BelongsToDomain::Yes))
        .unwrap();
    class.add_concept_id(cid);
    let class_id = class.id();
    app.project_mut().information_model_mut().add_class(class);

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::SelectInformationClass(Some(class_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_id));

    // 1. info_class_name_input (tekstfelt)
    // 2. info_class_desc_input (tekstfelt)
    // 3. INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX (checkbox)
    let _ = app.update(Message::FocusNext); // desc_input
    let _ = app.update(Message::FocusNext); // abstract_checkbox
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX),
        "Abstrakt checkbox skal være fokuserbar"
    );

    // Enter toggler abstrakt klasse
    assert!(!app
        .project()
        .information_model()
        .get_class(class_id)
        .unwrap()
        .is_abstract());
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .is_abstract(),
        "Enter på abstrakt checkbox skal gøre klassen abstrakt"
    );

    // 4. INFO_INSPECTOR_BTN_TYPE_LOCAL
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_TYPE_LOCAL)
    );

    // 5. INFO_INSPECTOR_BTN_TYPE_BORROWED
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_TYPE_BORROWED)
    );
    // Enter gør klassen indlånt
    let _ = app.update(Message::EnterPressed);
    assert!(
        !app.project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .is_local(),
        "Enter på Indlånt skal gøre klassen ikke-lokal"
    );

    // 6. INFO_INSPECTOR_BTN_ADD_ATTRIBUTE
    // Find add attribute knappen ved at tabbe fremad
    let mut found_add_attr = false;
    for _ in 0..10 {
        let _ = app.update(Message::FocusNext);
        if app.info_inspector_focused_button() == Some(INFO_INSPECTOR_BTN_ADD_ATTRIBUTE) {
            found_add_attr = true;
            break;
        }
    }
    assert!(
        found_add_attr,
        "Skal finde + Tilføj attribut knap i cyklussen"
    );

    // Enter tilføjer en attribut
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .attributes()
            .len(),
        1,
        "Enter på + Tilføj attribut skal oprette en ny attribut"
    );
}

#[test]
fn test_task_072_information_model_concept_picker_and_enum_cycle() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::information_model::InformationClass;
    use kant::ui::app::{
        App, Message, Tab, INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER, INFO_INSPECTOR_BTN_ADD_ENUM_VAL,
        INFO_INSPECTOR_BTN_CANVAS_ACTION, INFO_INSPECTOR_BTN_DELETE_ENUM,
        INFO_INSPECTOR_ENUM_VAL_REMOVE_START,
    };

    let mut app = App::new();

    // 1. Test "+ Knyt begreb" på en Klasse
    let class = InformationClass::new("Kunde");
    let class_id = class.id();
    app.project_mut().information_model_mut().add_class(class);

    // Opret to begreber i projektet som kan tilknyttes
    let concept_id1 = app
        .project_mut()
        .add_concept(Concept::new(
            "Kunde",
            "Person eller organisation",
            BelongsToDomain::Yes,
        ))
        .unwrap();

    let concept_id2 = app
        .project_mut()
        .add_concept(Concept::new(
            "Leverandør",
            "Virksomhed der leverer varer",
            BelongsToDomain::Yes,
        ))
        .unwrap();

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::SelectInformationClass(Some(class_id)));
    assert_eq!(app.selected_info_class_id(), Some(class_id));

    // Find INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER via Tab
    let mut found_concept_picker = false;
    for _ in 0..15 {
        let _ = app.update(Message::FocusNext);
        if app.info_inspector_focused_button() == Some(INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER) {
            found_concept_picker = true;
            break;
        }
    }
    assert!(
        found_concept_picker,
        "Skal finde + Knyt begreb picker i fokuscyklus"
    );

    // Enter på + Knyt begreb skal åbne comboboksen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.is_concept_picker_open(),
        "Enter på + Knyt begreb skal åbne comboboksen"
    );
    assert_eq!(app.concept_picker_index(), 0);

    // Test piletast ned navigerer til næste begreb i listen ("Leverandør")
    let _ = app.update(Message::ConceptPickerNext);
    assert_eq!(app.concept_picker_index(), 1);

    // Enter vælger og tilknytter "Leverandør" og lukker comboboksen
    let _ = app.update(Message::EnterPressed);
    assert!(
        !app.is_concept_picker_open(),
        "Enter skal lukke comboboksen efter valg"
    );
    assert!(
        app.project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .concept_ids()
            .contains(&concept_id2),
        "Valgt begreb (Leverandør) skal være tilknyttet klassen"
    );

    // Åbn comboboks igen og test søgefiltrering
    let _ = app.update(Message::OpenConceptPicker);
    assert!(app.is_concept_picker_open());
    let _ = app.update(Message::ConceptPickerSearchChanged("Kun".to_string()));
    assert_eq!(app.concept_picker_search(), "Kun");
    // Enter tilknytter det filtrerede begreb ("Kunde")
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.project()
            .information_model()
            .get_class(class_id)
            .unwrap()
            .concept_ids()
            .contains(&concept_id1),
        "Filtreret begreb (Kunde) skal være tilknyttet klassen"
    );

    // 2. Test Enumeration tastaturnavigation
    let _ = app.update(Message::CreateInformationEnumeration);
    let enum_id = app.selected_info_enum_id().expect("enum oprettet");

    // Fokus starter på tekstfeltet (info_enum_name_input)
    // Tab 1 -> info_enum_def_input (tekstfelt, button is None)
    let _ = app.update(Message::FocusNext);
    // Tab 2 -> INFO_INSPECTOR_BTN_CANVAS_ACTION
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_CANVAS_ACTION)
    );

    // Tab 3 -> INFO_INSPECTOR_BTN_DELETE_ENUM
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_DELETE_ENUM)
    );

    // Tab 4 -> new_enum_val_input (tekstfelt, button is None)
    let _ = app.update(Message::FocusNext);

    // Tab 5 -> INFO_INSPECTOR_BTN_ADD_ENUM_VAL
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_BTN_ADD_ENUM_VAL)
    );

    // Indtast en gyldig lowerCamelCase værdi og tilføj via Enter på knappen
    let _ = app.update(Message::NewEnumValueInputChanged("aktivStatus".to_string()));
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.project()
            .information_model()
            .get_enumeration(enum_id)
            .unwrap()
            .values(),
        &["aktivStatus"]
    );

    // Tab 1 mere -> slet-knap for den nye værdi
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_INSPECTOR_ENUM_VAL_REMOVE_START)
    );

    // Enter på slet-knap sletter værdien
    let _ = app.update(Message::EnterPressed);
    assert!(app
        .project()
        .information_model()
        .get_enumeration(enum_id)
        .unwrap()
        .values()
        .is_empty());
}

#[test]
fn test_task_075_concept_diagram_edge_tab_cycle_and_activation() {
    use kant::features::concept_model::RelationKind;
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::ui::app::{
        App, Message, Tab, CONCEPT_EDGE_BTN_DELETE, CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX,
        CONCEPT_EDGE_BTN_KIND_ASSOC, CONCEPT_EDGE_BTN_KIND_GEN, CONCEPT_EDGE_BTN_REVERSE,
    };

    let mut app = App::new();
    let c1 = Concept::new("Kunde", "Beskrivelse 1", BelongsToDomain::Yes);
    let c2 = Concept::new("Ordre", "Beskrivelse 2", BelongsToDomain::Yes);
    let c1_id = app.project_mut().add_concept(c1).unwrap();
    let c2_id = app.project_mut().add_concept(c2).unwrap();
    let n1 = {
        let concept = app.project().get_concept(c1_id).unwrap().clone();
        app.project_mut().concept_graph_mut().add_node(&concept)
    };
    let n2 = {
        let concept = app.project().get_concept(c2_id).unwrap().clone();
        app.project_mut().concept_graph_mut().add_node(&concept)
    };
    app.project_mut()
        .concept_graph_mut()
        .add_relation(n1, n2, RelationKind::Association);

    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    let _ = app.update(Message::GraphEdgeSelected(Some((n1, n2))));
    assert_eq!(app.selected_edge(), Some((n1, n2)));

    // Første tab fra input skal ramme Vend-knappen
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_REVERSE),
        "Første tab fra input skal ramme Vend-knappen"
    );

    // Enter på Vend-knappen skal vende kanten i concept graph
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_edge(),
        Some((n2, n1)),
        "Kanten skal være vendt til (n2, n1)"
    );

    let _ = app.update(Message::FocusNext); // CONCEPT_EDGE_BTN_KIND_ASSOC
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_KIND_ASSOC)
    );

    let _ = app.update(Message::FocusNext); // CONCEPT_EDGE_BTN_KIND_GEN
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_KIND_GEN)
    );

    let _ = app.update(Message::FocusNext); // CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX)
    );

    let _ = app.update(Message::FocusNext); // CONCEPT_EDGE_BTN_DELETE
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_DELETE)
    );

    // Wrap-around: næste Tab skal gå til input (focused_button er None)
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.concept_inspector_focused_button(),
        None,
        "Wrap around skal fokusere tekstfeltet"
    );

    // Tab tilbage til KIND_GEN og skift relationstype
    let _ = app.update(Message::FocusNext); // REVERSE
    let _ = app.update(Message::FocusNext); // KIND_ASSOC
    let _ = app.update(Message::FocusNext); // KIND_GEN
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_KIND_GEN)
    );
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.project()
            .concept_graph()
            .find_edge(n2, n1)
            .unwrap()
            .kind(),
        RelationKind::Generalization,
        "Enter på KIND_GEN skal ændre relationstype til Generalisering"
    );

    // Nu da kanten er Generalisering, skal Tab fra KIND_GEN springe direkte til DELETE
    let _ = app.update(Message::FocusNext);
    assert_eq!(
        app.concept_inspector_focused_button(),
        Some(CONCEPT_EDGE_BTN_DELETE),
        "Directed checkbox skal udelades fra Tab-cyklus ved Generalisering"
    );

    // Enter på Delete
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_edge(),
        None,
        "Kant skal være slettet og fravælges"
    );
}

#[test]
fn test_task_075_information_model_edge_tab_cycle_and_multiplicities() {
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::{ClassRelation, InformationClass};
    use kant::ui::app::{
        App, Message, Tab, INFO_EDGE_BTN_DELETE, INFO_EDGE_BTN_REVERSE, INFO_EDGE_BTN_SRC_MULT,
    };

    let mut app = App::new();
    let class_a = InformationClass::new("Kunde");
    let class_b = InformationClass::new("Ordre");
    let a_id = class_a.id();
    let b_id = class_b.id();
    app.project_mut().information_model_mut().add_class(class_a);
    app.project_mut().information_model_mut().add_class(class_b);
    let node_a = app.project_mut().information_graph_mut().add_node(a_id, 0);
    let node_b = app.project_mut().information_graph_mut().add_node(b_id, 0);

    let rel = ClassRelation::new(a_id, b_id, RelationKind::Association, None);
    app.project_mut().information_model_mut().add_relation(rel);
    app.project_mut().information_graph_mut().add_relation(
        node_a,
        node_b,
        RelationKind::Association,
        None,
    );

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::InfoEdgeSelected(Some((node_a, node_b))));
    assert_eq!(app.selected_info_edge(), Some((node_a, node_b)));

    let _ = app.update(Message::FocusNext); // INFO_EDGE_BTN_REVERSE
    assert_eq!(
        app.info_inspector_focused_button(),
        Some(INFO_EDGE_BTN_REVERSE),
        "Tab fra input skal ramme Vend-knappen"
    );

    // Enter vender relationen
    let _ = app.update(Message::EnterPressed);
    assert_eq!(app.selected_info_edge(), Some((node_b, node_a)));

    // Tab til SRC_MULT
    let mut found_src_mult = false;
    for _ in 0..10 {
        let _ = app.update(Message::FocusNext);
        if app.info_inspector_focused_button() == Some(INFO_EDGE_BTN_SRC_MULT) {
            found_src_mult = true;
            break;
        }
    }
    assert!(
        found_src_mult,
        "Kilde-multiplicitet skal kunne fokuseres med Tab"
    );

    // Enter cykler multiplicity presets
    let initial_mult = app
        .project()
        .information_model()
        .find_relation(b_id, a_id)
        .unwrap()
        .source_multiplicity();
    let _ = app.update(Message::EnterPressed);
    let new_mult = app
        .project()
        .information_model()
        .find_relation(b_id, a_id)
        .unwrap()
        .source_multiplicity();
    assert_ne!(
        initial_mult, new_mult,
        "Enter på multiplicitet skal cykle presets"
    );

    // Tab til DELETE
    let mut found_delete = false;
    for _ in 0..10 {
        let _ = app.update(Message::FocusNext);
        if app.info_inspector_focused_button() == Some(INFO_EDGE_BTN_DELETE) {
            found_delete = true;
            break;
        }
    }
    assert!(found_delete, "Slet-knap for kanten skal kunne nås med Tab");
    let _ = app.update(Message::EnterPressed);
    assert_eq!(
        app.selected_info_edge(),
        None,
        "Enter på slet kant skal fravælge kanten og slette relationen"
    );
}

#[test]
fn test_task_075_information_model_class_relations_delete_cycle() {
    use kant::features::concept_model::RelationKind;
    use kant::features::information_model::{ClassRelation, InformationClass};
    use kant::ui::app::{App, Message, Tab, INFO_INSPECTOR_REL_DELETE_START};

    let mut app = App::new();
    let class_a = InformationClass::new("Kunde");
    let class_b = InformationClass::new("Ordre");
    let a_id = class_a.id();
    let b_id = class_b.id();
    app.project_mut().information_model_mut().add_class(class_a);
    app.project_mut().information_model_mut().add_class(class_b);
    let node_a = app.project_mut().information_graph_mut().add_node(a_id, 0);
    let node_b = app.project_mut().information_graph_mut().add_node(b_id, 0);

    let rel = ClassRelation::new(a_id, b_id, RelationKind::Association, None);
    app.project_mut().information_model_mut().add_relation(rel);
    app.project_mut().information_graph_mut().add_relation(
        node_a,
        node_b,
        RelationKind::Association,
        None,
    );

    let _ = app.update(Message::SelectTab(Tab::InformationModel));
    let _ = app.update(Message::SelectInformationClass(Some(a_id)));
    assert_eq!(app.selected_info_class_id(), Some(a_id));

    // Tab gennem klasse-egenskaber indtil slet-knappen for relationen nås
    let mut found_rel_delete = false;
    for _ in 0..25 {
        let _ = app.update(Message::FocusNext);
        if app.info_inspector_focused_button() == Some(INFO_INSPECTOR_REL_DELETE_START) {
            found_rel_delete = true;
            break;
        }
    }
    assert!(
        found_rel_delete,
        "Slet-knap for tilknyttet relation skal nås i klasse-inspektorens Tab-cyklus"
    );

    // Enter skal slette relationen
    let _ = app.update(Message::EnterPressed);
    assert!(
        app.project().information_model().relations().is_empty(),
        "Enter på relationens slet-knap skal slette relationen fra modellen"
    );
}

// =======================================================================
// Task 076: Asynkron Git Synkroniseringsstatus & Diagnostik Log-udklipsholder
// =======================================================================

#[test]
fn test_task_076_autosave_does_not_touch_git_status_synchronously() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use kant::ui::app::{App, GitStatusRefresh, Message, Tab};

    let temp_dir = std::env::temp_dir().join(format!("kant_test_076_ac1_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let meta = ModelMetadata::new(
        "Autosave Sync Test",
        "Beskrivelse",
        "https://example.com/autosave",
        "TestOrg",
        "Test",
        "1.0.0",
        ModelStatus::Draft,
    );
    let project = ModelProject::new(meta);
    let model_file = temp_dir.join("model.kant");
    ProjectStorage::save(&project, &model_file).expect("save model");
    let _ = GitService::publish_model(&temp_dir, &project, "Første commit").expect("publish");

    let mut app = App::new_with_path(Some(model_file));
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "App skal starte i Uninitialized status"
    );

    // Kør trigger_autosave()
    app.trigger_autosave();

    // AC1: trigger_autosave() må IKKE ændre git_sync_status synkront på UI-tråden
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "trigger_autosave må IKKE ændre git_sync_status synkront på UI-tråden (forhindrer 2s UI-frys)"
    );

    // Efter et efterfølgende update() skal tilstandsmaskinen være gået i InFlight
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlight { generation: 0 },
        "Efter update() skal tilstandsmaskinen have registreret tjekket som InFlight"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_076_coalescing_state_machine() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use kant::ui::app::{App, GitStatusRefresh, Message, Tab};

    let temp_dir = std::env::temp_dir().join(format!("kant_test_076_ac2_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let meta = ModelMetadata::new(
        "Coalescing Test",
        "Beskrivelse",
        "https://example.com/coalesce",
        "TestOrg",
        "Test",
        "1.0.0",
        ModelStatus::Draft,
    );
    let project = ModelProject::new(meta);
    let model_file = temp_dir.join("model.kant");
    ProjectStorage::save(&project, &model_file).expect("save model");

    let mut app = App::new_with_path(Some(model_file));
    assert_eq!(app.git_status_refresh_state(), GitStatusRefresh::Idle);

    // 1. Første autosave starter tjek: Idle -> InFlight { generation: 0 }
    app.trigger_autosave();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlight { generation: 0 },
        "Første autosave skal sætte tilstanden til InFlight"
    );

    // 2. Andet autosave mens InFlight: InFlight -> InFlightDirty { generation: 0 }
    app.trigger_autosave();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlightDirty { generation: 0 },
        "Yderligere autosave under InFlight skal markere tilstanden InFlightDirty"
    );

    // 3. Tredje autosave koalesceres i InFlightDirty uden ekstra tasks
    app.trigger_autosave();
    let _ = app.update(Message::SelectTab(Tab::ConceptModel));
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlightDirty { generation: 0 },
        "Flere på hinanden følgende saves forbliver i InFlightDirty (koalescering)"
    );

    // 4. Når svar på første tjek ankommer under InFlightDirty:
    // Tilstanden skal straks genstarte et nyt tjek: InFlightDirty -> InFlight { generation: 0 }
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: 0,
        result: Ok(RepoSyncStatus::Synced),
        elapsed: std::time::Duration::from_millis(50),
    });
    assert_eq!(*app.git_sync_status(), RepoSyncStatus::Synced);
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlight { generation: 0 },
        "Svar modtaget under InFlightDirty skal overgå til InFlight for det opfølgende tjek"
    );

    // 5. Når det opfølgende svar ankommer og ingen nye saves er sket: InFlight -> Idle
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: 0,
        result: Ok(RepoSyncStatus::Synced),
        elapsed: std::time::Duration::from_millis(40),
    });
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::Idle,
        "Når opfølgende tjek er fuldført uden dirty flag, skal tilstanden vende tilbage til Idle"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_076_stale_git_status_result_is_discarded() {
    use kant::features::git::service::RepoSyncStatus;
    use kant::ui::app::{App, Message};

    let mut app = App::new();
    assert_eq!(*app.git_sync_status(), RepoSyncStatus::Uninitialized);
    let initial_gen = app.git_status_generation();

    // 1. Et svar med matchende generation skal anvendes
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: initial_gen,
        result: Ok(RepoSyncStatus::Synced),
        elapsed: std::time::Duration::from_millis(30),
    });
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Synced,
        "Svar med aktuel generation skal opdatere git_sync_status"
    );

    // 2. Handling der skifter model/repo skal bumpe generationen og sætte status til Uninitialized
    let _ = app.update(Message::NewProject);
    let bumped_gen = app.git_status_generation();
    assert!(
        bumped_gen > initial_gen,
        "NewProject skal have inkrementeret git_status_generation"
    );
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "NewProject skal have nulstillet git_sync_status til Uninitialized"
    );

    // 3. Et forsinket/forældet svar fra den gamle generation (f.eks. fra forrige repo) ankommer nu
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: initial_gen,
        result: Ok(RepoSyncStatus::PendingChanges),
        elapsed: std::time::Duration::from_millis(150),
    });

    // AC3: Forældet resultat skal kasseres fuldstændigt
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "Forældet svar med generation {} må IKKE overskrive status i generation {}",
        initial_gen,
        bumped_gen
    );
}

#[test]
fn test_task_076_truthful_git_status_transitions() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use kant::ui::app::{App, Message};

    let temp_dir = std::env::temp_dir().join(format!("kant_test_076_ac4_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let meta = ModelMetadata::new(
        "Køretøj Model",
        "Test for git transitions",
        "https://example.com/vehicle",
        "TestOrg",
        "Transport",
        "1.0.0",
        ModelStatus::Draft,
    );
    let project = ModelProject::new(meta);
    let model_file = temp_dir.join("model.kant");
    ProjectStorage::save(&project, &model_file).expect("save model");
    let _ = GitService::publish_model(&temp_dir, &project, "Initial commit").expect("publish");

    let mut app = App::new_with_path(Some(model_file));

    // Antag et initielt statustjek færdiggøres og finder 1 unpublished commit
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: app.git_status_generation(),
        result: Ok(RepoSyncStatus::UnpublishedCommits(1)),
        elapsed: std::time::Duration::from_millis(20),
    });
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::UnpublishedCommits(1),
        "Initial status efter opstarts-refresh skal være UnpublishedCommits(1)"
    );

    // Autosave UDEN modelændringer (deterministisk serialisering)
    app.trigger_autosave();
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::UnpublishedCommits(1),
        "Autosave alene må IKKE overskrive UnpublishedCommits med PendingChanges eller Synced"
    );

    // Når statustjekket afvikles på uændret repo, forbliver det UnpublishedCommits(1)
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: app.git_status_generation(),
        result: Ok(RepoSyncStatus::UnpublishedCommits(1)),
        elapsed: std::time::Duration::from_millis(20),
    });
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::UnpublishedCommits(1)
    );

    // Nu ændres modellen reelt
    app.project_mut()
        .metadata_mut()
        .set_name("Ændret Køretøj Model");
    app.trigger_autosave();

    // Når statustjekket efter gemningen fuldføres med ændringer på disk
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: app.git_status_generation(),
        result: Ok(RepoSyncStatus::PendingChanges),
        elapsed: std::time::Duration::from_millis(25),
    });
    assert_eq!(*app.git_sync_status(), RepoSyncStatus::PendingChanges);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_076_no_ui_thread_get_sync_status_calls() {
    let app_code = include_str!("../src/ui/app.rs");
    let mut illegal_calls = Vec::new();

    for (idx, line) in app_code.lines().enumerate() {
        let line_num = idx + 1;
        if line.contains("get_sync_status(") {
            illegal_calls.push((line_num, line.trim().to_string()));
        }
    }

    // Invariant: GitService::get_sync_status må kun optræde i baggrundstråde (spawn_blocking).
    // I src/ui/app.rs er der højst 3 legitime forekomster: 2 i execute_publish_workflow og 1 i perform_git_status_refresh.
    // De 6 synkrone UI-tråds call-sites fra Task 076 inventaret (linje 2738, 3031, 3244, 3274, 3383, 4220) skal alle være fjernet.
    assert!(
        illegal_calls.len() <= 3,
        "Der må IKKE være synkrone UI-tråds kald til get_sync_status i src/ui/app.rs! Fandt {} forekomster: {:?}",
        illegal_calls.len(),
        illegal_calls
    );
}

#[test]
fn test_task_076_report_contains_full_log_buffer() {
    use kant::features::diagnostics::{get_system_diagnostics, log_error, log_info, log_warn};

    let id = uuid::Uuid::new_v4().to_string();
    let msg1 = format!("Acceptance 076 start alpha {}", id);
    let msg2 = format!("Acceptance 076 middle beta {}", id);
    let msg3 = format!("Acceptance 076 final gamma {}", id);

    log_info("clipboard_test", &msg1);
    log_warn("clipboard_test", &msg2);
    log_error("clipboard_test", &msg3);

    let diag = get_system_diagnostics();
    assert!(
        diag.recent_logs.iter().any(|l| l.contains(&msg1)),
        "recent_logs i SystemDiagnostics skal indeholde de loggede hændelser"
    );

    let report = diag.formatted_report();
    assert!(
        report.contains("Seneste loghændelser (In-memory buffer"),
        "formatted_report skal indeholde sektion med overskriften 'Seneste loghændelser'"
    );

    let pos1 = report
        .find(&msg1)
        .expect("msg1 skal findes i den formaterede rapport");
    let pos2 = report
        .find(&msg2)
        .expect("msg2 skal findes i den formaterede rapport");
    let pos3 = report
        .find(&msg3)
        .expect("msg3 skal findes i den formaterede rapport");

    assert!(
        pos1 < pos2 && pos2 < pos3,
        "Loghændelser i rapporten skal fremtræde i streng kronologisk rækkefølge (første før sidste)"
    );
}

#[test]
fn test_task_076_boot_returns_initial_refresh_task() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use kant::ui::app::{App, GitStatusRefresh, Message};

    let temp_dir = std::env::temp_dir().join(format!("kant_test_076_ac6_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    GitService::init_repository(&temp_dir).expect("init repo");
    let meta = ModelMetadata::new(
        "Boot Test Model",
        "Beskrivelse",
        "https://example.com/boot",
        "TestOrg",
        "Test",
        "1.0.0",
        ModelStatus::Draft,
    );
    let project = ModelProject::new(meta);
    let model_file = temp_dir.join("model.kant");
    ProjectStorage::save(&project, &model_file).expect("save model");
    let _ = GitService::publish_model(&temp_dir, &project, "Initial commit").expect("publish");

    // AC6: Boot skal returnere (App, Task<Message>) uden at blokere første frame
    let (mut app, _task) = App::boot(Some(model_file));

    // Status skal være Uninitialized med det samme, og tjekket skal være InFlight
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "App skal starte i Uninitialized uden synkrone git-kald ved første frame"
    );
    assert_eq!(
        app.git_status_refresh_state(),
        GitStatusRefresh::InFlight { generation: 0 },
        "Boot skal have sat tilstanden til InFlight for det initiale baggrundstjek"
    );

    // Når det initiale statustjek fuldføres asynkront
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: 0,
        result: Ok(RepoSyncStatus::Synced),
        elapsed: std::time::Duration::from_millis(45),
    });
    assert_eq!(*app.git_sync_status(), RepoSyncStatus::Synced);
    assert_eq!(app.git_status_refresh_state(), GitStatusRefresh::Idle);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
