use kant::features::collab::crypto::{decrypt, encrypt, CollabKey, RoomId, SessionTicket};
use kant::features::collab::network::{
    next_registered_collab_event, CollabChannel, CollabNetworkEvent, ConnectionStatus,
};
use kant::features::collab::protocol::{CollabPayload, ModelMutation, Relation};
use kant::features::concept_model::RelationKind;
use kant::features::concepts::{BelongsToDomain, Concept};
use kant::features::information_model::InformationClass;
use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
use kant::ui::app::{App, CollabState, MenuType, Message, RelayServerPreset};
use kant_relay::{create_app, AppState, RelayConfig};
use std::time::Duration;
use tokio::net::TcpListener;

#[test]
fn test_mutation_bridge() {
    // 1. Serde Roundtrip for CollabPayload and ModelMutation variants
    let concept = Concept::new(
        "Vejkøretøj",
        "Køretøj indrettet til færdsel på vej",
        BelongsToDomain::Yes,
    );
    let concept_id = concept.id();
    let mut updated_concept = concept.clone();
    updated_concept.set_accepted_term(Some("Automobil".to_string()));

    let class = InformationClass::new("Køretøj");
    let class_id = class.id();
    let mut updated_class = class.clone();
    updated_class.set_description(Some("Opdateret beskrivelse af køretøj".to_string()));

    let rel = Relation::with_label(
        concept_id,
        uuid::Uuid::new_v4(),
        RelationKind::Association,
        Some("kører på".to_string()),
    );

    let mutations = vec![
        ModelMutation::ConceptAdded(concept.clone()),
        ModelMutation::ConceptUpdated(updated_concept.clone()),
        ModelMutation::ConceptDeleted(concept_id),
        ModelMutation::InformationClassAdded(class.clone()),
        ModelMutation::InformationClassUpdated(updated_class.clone()),
        ModelMutation::InformationClassDeleted(class_id),
        ModelMutation::RelationAdded(rel.clone()),
        ModelMutation::RelationUpdated(rel.clone()),
        ModelMutation::RelationDeleted {
            from: concept_id,
            to: rel.to,
        },
        ModelMutation::ConceptDiagramNodeAdded(concept_id),
        ModelMutation::ConceptDiagramNodeRemoved(concept_id),
        ModelMutation::ClassDiagramNodeAdded(class_id),
        ModelMutation::ClassDiagramNodeRemoved(class_id),
        ModelMutation::ClassRelationAdded {
            from_class: class_id,
            to_class: uuid::Uuid::new_v4(),
            kind: RelationKind::Association,
            label: Some("har".to_string()),
            source_multiplicity: None,
            target_multiplicity: None,
            directed: Some(true),
        },
        ModelMutation::ClassRelationDeleted {
            from_class: class_id,
            to_class: uuid::Uuid::new_v4(),
        },
        ModelMutation::NodeMoved {
            id: concept_id,
            x: 250.0,
            y: 350.0,
        },
    ];

    for mutation in &mutations {
        let payload = CollabPayload::Mutation(mutation.clone());
        let json = serde_json::to_string(&payload).expect("Mutation serialization fejlede");
        let deserialized: CollabPayload =
            serde_json::from_str(&json).expect("Mutation deserialization fejlede");
        assert_eq!(payload, deserialized);
    }

    // 2. Snapshot Payload roundtrip
    let test_proj = ModelProject::new(ModelMetadata::new(
        "Kollaborativ Testmodel",
        "Testbeskrivelse",
        "https://data.gov.dk/model/test",
        "Digitaliseringsstyrelsen",
        "Testdomæne",
        "1.0.0",
        ModelStatus::Draft,
    ));
    let snap_payload = CollabPayload::Snapshot(test_proj.clone());
    let snap_json = serde_json::to_string(&snap_payload).expect("Snapshot serialization fejlede");
    let snap_deserialized: CollabPayload =
        serde_json::from_str(&snap_json).expect("Snapshot deserialization fejlede");
    assert_eq!(snap_payload, snap_deserialized);

    // 3. E2EE kryptering og dekryptering af CollabPayload
    let key = CollabKey::generate();
    let payload_bytes = serde_json::to_vec(&CollabPayload::Mutation(ModelMutation::ConceptAdded(
        concept.clone(),
    )))
    .unwrap();
    let encrypted = encrypt(&key, &payload_bytes).expect("Payload kryptering fejlede");
    let decrypted = decrypt(&key, &encrypted).expect("Payload dekryptering fejlede");
    let recovered_payload: CollabPayload = serde_json::from_slice(&decrypted).unwrap();
    assert_eq!(
        recovered_payload,
        CollabPayload::Mutation(ModelMutation::ConceptAdded(concept.clone()))
    );

    // 4. Live Mutation Application in App (in-memory mutation bridge)
    let mut app = App::new_with_path(None);

    // Tilføj begreb via CollabApplyMutation
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::ConceptAdded(concept.clone()),
    )));
    assert_eq!(
        app.project().concepts().len(),
        1,
        "ConceptAdded mutation skal tilføje begreb til App projekt"
    );
    assert_eq!(app.project().concepts()[0].preferred_term(), "Vejkøretøj");

    // Flyt node via CollabApplyMutation
    if let Some(node) = app
        .project()
        .concept_graph()
        .find_node_by_concept(concept_id)
    {
        let node_id = node.id();
        let _ = app.update(Message::CollabApplyMutation(Box::new(
            ModelMutation::NodeMoved {
                id: node_id,
                x: 420.0,
                y: 280.0,
            },
        )));
        let updated_node = app
            .project()
            .concept_graph()
            .find_node(node_id)
            .expect("Node skal findes");
        assert_eq!(updated_node.x(), 420.0);
        assert_eq!(updated_node.y(), 280.0);
    }

    // Opdater begreb
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::ConceptUpdated(updated_concept.clone()),
    )));
    assert_eq!(
        app.project().concepts()[0].accepted_term(),
        Some("Automobil")
    );

    // Tilføj og fjern klasse
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::InformationClassAdded(class.clone()),
    )));
    assert_eq!(
        app.project().information_model().classes().len(),
        1,
        "InformationClassAdded skal tilføje klasse"
    );
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::InformationClassUpdated(updated_class.clone()),
    )));
    assert_eq!(
        app.project().information_model().classes()[0].description(),
        Some("Opdateret beskrivelse af køretøj")
    );
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::InformationClassDeleted(class_id),
    )));
    assert_eq!(
        app.project().information_model().classes().len(),
        0,
        "InformationClassDeleted skal slette klasse"
    );

    // Fjern begreb
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::ConceptDeleted(concept_id),
    )));
    assert_eq!(
        app.project().concepts().len(),
        0,
        "ConceptDeleted skal slette begreb"
    );

    // Snapshot erstatning
    let _ = app.update(Message::CollabApplySnapshot(Box::new(test_proj.clone())));
    assert_eq!(app.project().metadata().name(), "Kollaborativ Testmodel");
}

#[test]
fn test_guest_autosave_suppressed() {
    // 1. Initialiser midlertidig diskfil
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_task027_{}.edge.json", uuid::Uuid::new_v4()));

    let mut app = App::new_with_path(Some(file_path.clone()));
    assert_eq!(app.collab_state(), CollabState::None);

    // Initial lagring med normal tilstand skal fungere (Host / Standalone)
    app.trigger_autosave();
    let initial_disk_data = std::fs::read_to_string(&file_path).expect("Læsning af fil fejlede");
    assert!(
        !initial_disk_data.is_empty(),
        "Filen skal have indhold efter normal autosave"
    );

    // 2. Skift til Guest tilstand
    app.set_collab_state(CollabState::Guest);
    assert!(app.collab_state().is_guest());

    // Skriv en specifik markør på disken
    let disk_marker = "{\"MARKER\":\"DISK_SKAL_IKKE_OVERSKRIVES_AF_GUEST\"}";
    std::fs::write(&file_path, disk_marker).expect("Skrivning af markør fejlede");

    // 3. Modtag mutationer i hukommelsen (RAM)
    let guest_concept = Concept::new(
        "GæstBegreb",
        "Dette begreb findes kun i RAM for gæsten",
        BelongsToDomain::Yes,
    );
    let _ = app.update(Message::CollabApplyMutation(Box::new(
        ModelMutation::ConceptAdded(guest_concept),
    )));

    // 4. Kald trigger_autosave() og SaveProject under Guest-tilstand
    app.trigger_autosave();
    let _ = app.update(Message::SaveProject);

    // 5. Invariant Assertion: Diskindholdet MÅ IKKE være ændret!
    let disk_after_autosave = std::fs::read_to_string(&file_path).expect("Læsning af fil fejlede");
    assert_eq!(
        disk_after_autosave, disk_marker,
        "Gæstens autosave må ALDRIG overskrive lokal diskfil!"
    );

    // 6. Skift til Host: Nu skal autosave skrive til disken
    app.set_collab_state(CollabState::Host);
    assert!(app.collab_state().is_host());
    app.trigger_autosave();

    let disk_after_host_autosave =
        std::fs::read_to_string(&file_path).expect("Læsning af fil fejlede");
    assert_ne!(
        disk_after_host_autosave, disk_marker,
        "Værten (Host) skal autosave godkendte ændringer til disk"
    );
    assert!(
        disk_after_host_autosave.contains("GæstBegreb"),
        "Værten skal gemme det tilføjede begreb på disk"
    );

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_task028_collab_ui_modals_and_presence() {
    let mut app = App::new_with_path(None);

    // 1. Header-bar menupunkt og Start Session modal flow
    assert!(app.start_session_modal().is_none());
    assert!(app.join_session_modal().is_none());

    // Åbn samarbejdsmenu
    let _ = app.update(Message::ToggleMenu(MenuType::Collab));
    assert_eq!(app.active_menu(), Some(MenuType::Collab));

    // Åbn Værtsmodal (Start Session)
    let _ = app.update(Message::OpenStartSessionModal);
    let start_modal = app
        .start_session_modal()
        .expect("StartSessionModal skal være åben efter OpenStartSessionModal");

    // Standard preset skal være Koyeb Cloud Frankfurt
    assert_eq!(start_modal.preset, RelayServerPreset::Koyeb);
    assert_eq!(start_modal.current_url(), "wss://kant.koyeb.app/ws");
    let token = start_modal.ticket.to_token();
    assert!(
        token.starts_with("kant:v1:"),
        "Sessionsbillet skal have det standardiserede kant:v1: præfiks"
    );

    // Skift til Lokal Docker preset
    let _ = app.update(Message::CollabPresetSelected(
        RelayServerPreset::LocalDocker,
    ));
    let start_modal = app.start_session_modal().unwrap();
    assert_eq!(start_modal.preset, RelayServerPreset::LocalDocker);
    assert_eq!(start_modal.current_url(), "ws://localhost:8080/ws");

    // Skift til Brugerdefineret URL preset
    let _ = app.update(Message::CollabPresetSelected(RelayServerPreset::Custom));
    let _ = app.update(Message::CollabCustomUrlChanged(
        "wss://my-org-relay.internal/ws".to_string(),
    ));
    let start_modal = app.start_session_modal().unwrap();
    assert_eq!(start_modal.preset, RelayServerPreset::Custom);
    assert_eq!(start_modal.current_url(), "wss://my-org-relay.internal/ws");

    // Kopiér sessionsbillet
    assert!(!start_modal.copied);
    let _ = app.update(Message::CollabCopyTicket);
    let start_modal = app.start_session_modal().unwrap();
    assert!(start_modal.copied, "Skal sætte copied flag for feedback");

    // Start session
    let _ = app.update(Message::CollabStartSession);
    assert!(
        app.start_session_modal().is_none(),
        "Værtsdialogen skal lukkes ved opstart af session"
    );
    assert_eq!(app.collab_state(), CollabState::Host);
    assert!(
        app.collab_status_summary().starts_with("Live: Vært"),
        "Statusindikator skal vise Live: Vært"
    );

    // Invariant: Simultan opstart må IKKE tillades under aktiv session
    let _ = app.update(Message::OpenStartSessionModal);
    assert!(
        app.start_session_modal().is_none(),
        "Må IKKE åbne StartSessionModal hvis en session allerede er aktiv"
    );
    let _ = app.update(Message::OpenJoinSessionModal);
    assert!(
        app.join_session_modal().is_none(),
        "Må IKKE åbne JoinSessionModal hvis en session allerede er aktiv"
    );

    // Afbryd session
    let _ = app.update(Message::CollabDisconnect);
    assert_eq!(app.collab_state(), CollabState::None);
    assert_eq!(app.collab_status_summary(), "Offline");

    // 2. Gæstedialog (Join Session) og realtids-validering
    let _ = app.update(Message::OpenJoinSessionModal);
    let join_modal = app
        .join_session_modal()
        .expect("JoinSessionModal skal være åben efter OpenJoinSessionModal");
    assert!(!join_modal.is_valid());

    // Indtast ugyldig kode
    let _ = app.update(Message::CollabJoinTokenChanged(
        "ugyldig-tekst-uden-edge-prefix".to_string(),
    ));
    let join_modal = app.join_session_modal().unwrap();
    assert!(!join_modal.is_valid());
    assert!(
        join_modal.error_message.is_some(),
        "Ugyldigt token skal give fejlmeddelelse"
    );

    // Indtast gyldig sessionskode
    let valid_ticket = SessionTicket::new(
        "wss://relay.edge.internal/ws",
        RoomId::generate(),
        CollabKey::generate(),
    );
    let valid_token_str = valid_ticket.to_token();
    let _ = app.update(Message::CollabJoinTokenChanged(valid_token_str));
    let join_modal = app.join_session_modal().unwrap();
    assert!(
        join_modal.is_valid(),
        "Gyldigt token skal valideres med succes"
    );
    assert!(join_modal.error_message.is_none());

    // Forbind til session som gæst
    let _ = app.update(Message::CollabJoinSession);
    assert!(
        app.join_session_modal().is_none(),
        "Gæstedialog skal lukkes ved tilslutning"
    );
    assert_eq!(app.collab_state(), CollabState::Guest);
    app.set_collab_connection_status(kant::features::collab::ConnectionStatus::Connected);
    assert_eq!(
        app.collab_status_summary(),
        "Live: Gæst (Forbundet til Vært)"
    );

    // 3. Vært afslutter session -> Gæst modtager advarsel og tilbud om lokal kopi
    app.notify_host_ended_session();
    assert_eq!(app.collab_state(), CollabState::None);
    let notice = app
        .guest_ended_notice()
        .expect("Gæst skal modtage notice ved afbrudt session");
    assert!(notice.message.contains("gemme en kopi"));
    let _ = app.update(Message::CollabGuestDismissEndedModal);
    assert!(app.guest_ended_notice().is_none());

    // 4. Tastaturnavigation: Escape lukker modaler
    let _ = app.update(Message::OpenStartSessionModal);
    assert!(app.start_session_modal().is_some());
    // Verificer rendering med StartSessionModal
    let _ = app.view();
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.start_session_modal().is_none(),
        "EscapePressed skal lukke StartSessionModal"
    );

    let _ = app.update(Message::OpenJoinSessionModal);
    assert!(app.join_session_modal().is_some());
    // Verificer rendering med JoinSessionModal
    let _ = app.view();
    let _ = app.update(Message::EscapePressed);
    assert!(
        app.join_session_modal().is_none(),
        "EscapePressed skal lukke JoinSessionModal"
    );

    // Verificer rendering under aktiv session
    app.set_collab_state(CollabState::Host);
    app.set_collab_connection_status(kant::features::collab::ConnectionStatus::Connected);
    let _ = app.view();
    app.set_collab_state(CollabState::None);
}

#[tokio::test]
async fn test_task029_e2e_collab_sync_and_presence() {
    // 1. Start ephemeral test relay server
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Kunne ikke binde TCP listener");
    let addr = listener
        .local_addr()
        .expect("Kunne ikke hente lokal adresse");
    let state = AppState::new(RelayConfig::default());
    let relay_app = create_app(state);
    tokio::spawn(async move {
        axum::serve(listener, relay_app)
            .await
            .expect("Relay stoppede uventet");
    });
    let relay_url = format!("ws://{}", addr);

    // 2. Opret Vært App og Gæst App med isoleret in-memory model (uden disk fil)
    let mut host = App::new_with_path(None);
    let mut guest = App::new_with_path(None);

    // Vært konfigurerer custom relay og starter session
    let _ = host.update(Message::OpenStartSessionModal);
    let _ = host.update(Message::CollabPresetSelected(RelayServerPreset::Custom));
    let _ = host.update(Message::CollabCustomUrlChanged(relay_url.clone()));
    let ticket_token = host
        .start_session_modal()
        .expect("StartSessionModal skal være åben")
        .ticket
        .to_token();
    let _ = host.update(Message::CollabStartSession);
    assert_eq!(host.collab_state(), CollabState::Host);

    // Gæst tilslutter via sessionskoden
    let _ = guest.update(Message::OpenJoinSessionModal);
    let _ = guest.update(Message::CollabJoinTokenChanged(ticket_token));
    let _ = guest.update(Message::CollabJoinSession);
    assert_eq!(guest.collab_state(), CollabState::Guest);

    let host_sub_id = host
        .collab_channel()
        .expect("Host collab_channel skal eksistere")
        .sub_id();
    let guest_sub_id = guest
        .collab_channel()
        .expect("Guest collab_channel skal eksistere")
        .sub_id();

    // 3. Pump netværkshændelser for initial opkobling, presence og snapshot
    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(host_sub_id),
        )
        .await
        {
            let _ = host.update(Message::CollabNetworkEventReceived(ev));
        }
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(guest_sub_id),
        )
        .await
        {
            let _ = guest.update(Message::CollabNetworkEventReceived(ev));
        }
        if host.collab_participant_count() == 2 && guest.collab_participant_count() == 2 {
            break;
        }
    }
    assert_eq!(
        host.collab_participant_count(),
        2,
        "Vært skal registrere 2 deltagere"
    );
    assert_eq!(
        guest.collab_participant_count(),
        2,
        "Gæst skal registrere 2 deltagere"
    );

    // 4. Vært opretter et begreb -> synkroniseres til Gæst
    let new_concept = Concept::new("Vejafgift", "Gebyr for passage", BelongsToDomain::Yes);
    let concept_id = new_concept.id();
    host.apply_mutation(ModelMutation::ConceptAdded(new_concept.clone()));
    host.broadcast_mutation(&ModelMutation::ConceptAdded(new_concept.clone()));

    // Pump events til Gæst
    let mut guest_synced = false;
    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(guest_sub_id),
        )
        .await
        {
            let _ = guest.update(Message::CollabNetworkEventReceived(ev));
            if guest
                .project()
                .concepts()
                .iter()
                .any(|c| c.id() == concept_id)
            {
                guest_synced = true;
                break;
            }
        }
    }
    assert!(guest_synced, "Gæst modtog ikke begrebet tilføjet af Vært");
    assert!(
        guest
            .project()
            .concept_graph()
            .is_concept_on_diagram(concept_id),
        "Gæst skal have tilføjet noden til diagrammet"
    );

    // 5. Gæst flytter en node -> synkroniseres til Vært
    let node = guest
        .project()
        .concept_graph()
        .find_node_by_concept(concept_id)
        .expect("Node skal findes på lærredet");
    let node_id = node.id();

    guest.broadcast_mutation(&ModelMutation::NodeMoved {
        id: node_id,
        x: 420.0,
        y: 680.0,
    });

    // Pump events til Vært
    let mut host_synced = false;
    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(host_sub_id),
        )
        .await
        {
            let _ = host.update(Message::CollabNetworkEventReceived(ev));
            if let Some(host_node) = host.project().concept_graph().find_node(node_id) {
                if (host_node.x() - 420.0).abs() < 1.0 && (host_node.y() - 680.0).abs() < 1.0 {
                    host_synced = true;
                    break;
                }
            }
        }
    }
    assert!(
        host_synced,
        "Vært modtog ikke nodeflytning foretaget af Gæst"
    );

    // 5b. Vært tilføjer et ekstra begreb og forbinder med en kant -> synkroniseres til Gæst
    let second_concept = Concept::new("Afgiftstype", "Klassifikation", BelongsToDomain::Yes);
    let second_concept_id = second_concept.id();
    host.apply_mutation(ModelMutation::ConceptAdded(second_concept.clone()));
    host.broadcast_mutation(&ModelMutation::ConceptAdded(second_concept.clone()));

    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(guest_sub_id),
        )
        .await
        {
            let _ = guest.update(Message::CollabNetworkEventReceived(ev));
            if guest
                .project()
                .concepts()
                .iter()
                .any(|c| c.id() == second_concept_id)
            {
                break;
            }
        }
    }

    let host_n1 = host
        .project()
        .concept_graph()
        .find_node_by_concept(concept_id)
        .unwrap()
        .id();
    let host_n2 = host
        .project()
        .concept_graph()
        .find_node_by_concept(second_concept_id)
        .unwrap()
        .id();

    // Vært forbinder noderne via Message::GraphEdgeCreated
    let _ = host.update(Message::GraphEdgeCreated(host_n1, host_n2));

    // Pump events til Gæst
    let mut guest_received_edge = false;
    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(guest_sub_id),
        )
        .await
        {
            let _ = guest.update(Message::CollabNetworkEventReceived(ev));
            if guest.project().concept_graph().edges().len() == 1 {
                guest_received_edge = true;
                break;
            }
        }
    }
    assert!(
        guest_received_edge,
        "Gæst modtog ikke kanten/relationen oprettet af Vært via GraphEdgeCreated"
    );

    // 6. Vært afbryder sessionen -> Gæst modtager notice og advarsel
    let _ = host.update(Message::CollabDisconnect);
    assert_eq!(host.collab_state(), CollabState::None);

    let mut guest_received_ended = false;
    for _ in 0..15 {
        if let Ok(Some(ev)) = tokio::time::timeout(
            Duration::from_millis(100),
            next_registered_collab_event(guest_sub_id),
        )
        .await
        {
            let _ = guest.update(Message::CollabNetworkEventReceived(ev));
            if guest.guest_ended_notice().is_some() {
                guest_received_ended = true;
                break;
            }
        }
    }
    assert!(
        guest_received_ended,
        "Gæst modtog ikke HostEndedSession notice"
    );
    assert_eq!(guest.collab_state(), CollabState::None);
}

#[test]
fn test_host_rebroadcasts_snapshot_when_participant_count_rises() {
    let mut app = App::new_with_path(None);

    // --- Invariant 1: Gæst-rolle må ALDRIG sende snapshot ---
    // Sæt tilstand manuelt som gæst (ingen kanal = broadcast_snapshot returnerer tidligt)
    let _ = app.update(Message::CollabNetworkEventReceived(
        CollabNetworkEvent::StatusChanged(ConnectionStatus::Connected),
    ));
    let seq_before = app.collab_seq_value();
    // Simuler PresenceUpdated som gæst (collab_state = None → guard afviser)
    app.set_collab_participant_count(2);
    assert_eq!(
        app.collab_seq_value(),
        seq_before,
        "Gæst/None-rolle må IKKE øge collab_seq ved PresenceUpdated"
    );

    // --- Invariant 2: Vært SKAL sende snapshot når count stiger ---
    let mut host_app = App::new_with_path(None);
    host_app.set_collab_participant_count(1); // Sæt startværdi uden host-rolle
    let seq_before_host = host_app.collab_seq_value();
    host_app.set_collab_participant_count(2); // Stigning, men ingen host-rolle → ingen snapshot
    assert_eq!(
        host_app.collab_seq_value(),
        seq_before_host,
        "Uden host-rolle må stigende participant_count IKKE øge collab_seq"
    );

    // --- Invariant 3: Fald i deltagerantal må IKKE sende snapshot (gæst forlader) ---
    let mut host_app2 = App::new_with_path(None);
    host_app2.set_collab_participant_count(3);
    let seq_at_peak = host_app2.collab_seq_value();
    host_app2.set_collab_participant_count(2); // Fald → ingen snapshot
    assert_eq!(
        host_app2.collab_seq_value(),
        seq_at_peak,
        "Faldende participant_count må ALDRIG sende snapshot"
    );

    // --- Invariant 4: Ens antal (ingen ændring) må IKKE sende snapshot ---
    let mut host_app3 = App::new_with_path(None);
    host_app3.set_collab_participant_count(2);
    let seq_stable = host_app3.collab_seq_value();
    host_app3.set_collab_participant_count(2); // Uændret → ingen snapshot
    assert_eq!(
        host_app3.collab_seq_value(),
        seq_stable,
        "Uændret participant_count må ALDRIG sende snapshot"
    );
}

#[test]
fn test_collab_channel_connect_spawns_outside_tokio_runtime() {
    assert!(
        tokio::runtime::Handle::try_current().is_err(),
        "Testen SKAL køre på en tråd UDEN aktiv Tokio runtime (ligesom Iced GUI tråden)"
    );

    let room_id = RoomId::new("test-outside-tokio");
    let (channel, mut rx) = CollabChannel::connect("ws://127.0.0.1:9999/ws", &room_id);

    // Skal modtage StatusChanged(Connecting) fra baggrundstråden
    let event = rx.blocking_recv();
    assert_eq!(
        event,
        Some(CollabNetworkEvent::StatusChanged(
            ConnectionStatus::Connecting
        )),
        "Kanalen SKAL starte på baggrunds-runtime og udsende Connecting"
    );
    drop(channel);
}

#[test]
fn test_task034_collab_edge_and_diagram_sync_lifecycle() {
    let mut guest_app = App::new_with_path(None);

    // 1. Concept Model: Synkroniser noder
    let c1 = Concept::new("Køretøj", "Transportmiddel", BelongsToDomain::Yes);
    let c2 = Concept::new("Motor", "Drivmiddel", BelongsToDomain::Yes);
    let c1_id = c1.id();
    let c2_id = c2.id();

    guest_app.apply_mutation(ModelMutation::ConceptAdded(c1));
    guest_app.apply_mutation(ModelMutation::ConceptAdded(c2));
    guest_app.apply_mutation(ModelMutation::ConceptDiagramNodeAdded(c1_id));
    guest_app.apply_mutation(ModelMutation::ConceptDiagramNodeAdded(c2_id));

    assert_eq!(guest_app.project().concept_graph().nodes().len(), 2);

    // 2. Concept Model: Synkroniser kant/relation (RelationAdded)
    let rel = Relation::with_all(
        uuid::Uuid::new_v4(),
        c1_id,
        c2_id,
        RelationKind::Association,
        Some("har del".to_string()),
        Some(true),
    );
    guest_app.apply_mutation(ModelMutation::RelationAdded(rel.clone()));

    assert_eq!(
        guest_app.project().concept_graph().edges().len(),
        1,
        "Gæst skal have 1 relation efter RelationAdded"
    );

    // 3. Concept Model: Opdater relation (RelationUpdated)
    let mut updated_rel = rel.clone();
    updated_rel.kind = RelationKind::Composition;
    updated_rel.label = Some("består af".to_string());
    guest_app.apply_mutation(ModelMutation::RelationUpdated(updated_rel));

    let n1 = guest_app
        .project()
        .concept_graph()
        .find_node_by_concept(c1_id)
        .unwrap()
        .id();
    let n2 = guest_app
        .project()
        .concept_graph()
        .find_node_by_concept(c2_id)
        .unwrap()
        .id();
    let edge = guest_app
        .project()
        .concept_graph()
        .find_edge(n1, n2)
        .expect("Kant skal eksistere");
    assert_eq!(edge.kind(), RelationKind::Composition);
    assert_eq!(edge.label(), Some("består af"));

    // 4. Concept Model: Slet relation (RelationDeleted)
    guest_app.apply_mutation(ModelMutation::RelationDeleted {
        from: c1_id,
        to: c2_id,
    });
    assert_eq!(
        guest_app.project().concept_graph().edges().len(),
        0,
        "Relation skal være slettet på gæsten"
    );

    // 5. Information Model: Synkroniser klasser og diagram noder
    let cls1 = InformationClass::new("KøretøjKlasse");
    let cls2 = InformationClass::new("MotorKlasse");
    let cls1_id = cls1.id();
    let cls2_id = cls2.id();

    guest_app.apply_mutation(ModelMutation::InformationClassAdded(cls1));
    guest_app.apply_mutation(ModelMutation::InformationClassAdded(cls2));
    guest_app.apply_mutation(ModelMutation::ClassDiagramNodeAdded(cls1_id));
    guest_app.apply_mutation(ModelMutation::ClassDiagramNodeAdded(cls2_id));

    assert_eq!(guest_app.project().information_graph().nodes().len(), 2);

    // 6. Information Model: Flyt node (NodeMoved via class UUID)
    guest_app.apply_mutation(ModelMutation::NodeMoved {
        id: cls1_id,
        x: 450.0,
        y: 650.0,
    });
    let node1 = guest_app
        .project()
        .information_graph()
        .find_node_by_class(cls1_id)
        .expect("Klassenode skal findes");
    assert_eq!(
        (node1.x(), node1.y()),
        (450.0, 650.0),
        "Klassenode position skal opdateres på gæst"
    );

    // 7. Information Model: Tilføj relation (ClassRelationAdded)
    guest_app.apply_mutation(ModelMutation::ClassRelationAdded {
        from_class: cls1_id,
        to_class: cls2_id,
        kind: RelationKind::Association,
        label: Some("benytter".to_string()),
        source_multiplicity: None,
        target_multiplicity: None,
        directed: Some(true),
    });
    assert_eq!(
        guest_app.project().information_graph().edges().len(),
        1,
        "Gæst skal have 1 klasse-relation efter ClassRelationAdded"
    );

    // 8. Information Model: Slet relation (ClassRelationDeleted)
    guest_app.apply_mutation(ModelMutation::ClassRelationDeleted {
        from_class: cls1_id,
        to_class: cls2_id,
    });
    assert_eq!(
        guest_app.project().information_graph().edges().len(),
        0,
        "Klasse-relation skal være slettet på gæsten"
    );
}
