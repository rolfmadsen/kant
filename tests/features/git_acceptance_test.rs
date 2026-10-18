use kant::ui::app::{App, Message};

#[test]
fn test_task_046_git_service_and_domain_event_mapping() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::git::{ChangeAction, DomainEventMapper, GitService, RepoSyncStatus};
    use kant::features::information_model::InformationClass;
    use kant::features::model::{ModelMetadata, ModelProject, ModelStatus};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_046_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Initialiser Git-lager
    GitService::init_repository(&temp_dir).expect("init_repository skal lykkes");
    assert!(
        temp_dir.join(".git").is_dir(),
        ".git mappen skal være oprettet"
    );

    // 2. Tjek status på tomt repo
    let status = GitService::get_sync_status(&temp_dir).expect("status check skal lykkes");
    assert!(
        matches!(
            status,
            RepoSyncStatus::Synced | RepoSyncStatus::UnpublishedCommits(0)
        ),
        "Nyt repo skal være synkroniseret eller have 0 uudgivne commits"
    );

    // 3. Opret modelprojekt og udgiv (første commit)
    let meta = ModelMetadata::new(
        "Køretøjsmodellen Git",
        "Test for Git integration",
        "https://data.gov.dk/model/vehicle",
        "Motorstyrelsen",
        "Transport",
        "1.0.0",
        ModelStatus::Draft,
    );
    let mut project = ModelProject::new(meta);

    let c1 = Concept::new(
        "Køretøj",
        "Transportmiddel til personer eller gods",
        BelongsToDomain::Yes,
    );
    let c1_id = project.add_concept(c1).unwrap();

    let commit1_oid =
        GitService::publish_model(&temp_dir, &project, "Oprettet grundmodel med Køretøj")
            .expect("publish_model skal lykkes");
    assert!(!commit1_oid.is_empty(), "Commit OID må ikke være tom");

    // 4. Hent historik og verificer domænehændelser
    let history =
        GitService::get_commit_history(&temp_dir, 10).expect("get_commit_history skal lykkes");
    assert_eq!(history.len(), 1);
    let first_commit = &history[0];
    assert_eq!(first_commit.oid, commit1_oid);
    assert_eq!(first_commit.message, "Oprettet grundmodel med Køretøj");

    // Skal have en hændelse: 🟢 Oprettet Begreb: Køretøj
    let has_c1_added = first_commit.changes.iter().any(|e| {
        e.action == ChangeAction::Added && e.entity_type == "Begreb" && e.entity_name == "Køretøj"
    });
    assert!(
        has_c1_added,
        "Commit skal indeholde en Added Begreb Køretøj hændelse"
    );

    // 5. Rediger begreb og tilføj klasse, og test auto-genereret commit-summary
    let mut c1_updated = project.get_concept(c1_id).unwrap().clone();
    c1_updated.set_definition("Opdateret definition af motordrevet transportmiddel");
    project.update_concept(c1_updated).unwrap();

    let cls1 = InformationClass::new("Køretøj");
    let cls1_id = project.information_model_mut().add_class(cls1);

    // Udgiv med tom besked -> skal bruge auto-genereret besked
    let commit2_oid = GitService::publish_model(&temp_dir, &project, "")
        .expect("publish_model med auto-summary skal lykkes");

    let history_after =
        GitService::get_commit_history(&temp_dir, 10).expect("historik skal hentes");
    assert_eq!(history_after.len(), 2);
    let second_commit = &history_after[0];
    assert_eq!(second_commit.oid, commit2_oid);
    assert!(
        !second_commit.message.is_empty(),
        "Auto-genereret commitbesked må ikke være tom"
    );
    let expected_summary = DomainEventMapper::generate_commit_summary(&second_commit.changes);
    assert_eq!(second_commit.message, expected_summary);

    // 6. Test element-specifik historik ("Time Travel" / Audit)
    let c1_history =
        GitService::get_element_history(&temp_dir, c1_id).expect("element historik for c1");
    assert_eq!(
        c1_history.len(),
        2,
        "c1 skal have præcis 2 commits (oprettelse og ændring)"
    );

    let cls1_history =
        GitService::get_element_history(&temp_dir, cls1_id).expect("element historik for cls1");
    assert_eq!(
        cls1_history.len(),
        1,
        "cls1 skal have præcis 1 commit (oprettelse)"
    );

    // Oprydning
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_047_git_ui_topbar_status_and_publish_dialog() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::git::{GitService, RepoSyncStatus};
    use kant::ui::app::{App, Message};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_047_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Initialiser repo
    GitService::init_repository(&temp_dir).expect("init repo");

    // 2. Opret App instans med projektmappen
    let mut app = App::new_with_path(Some(temp_dir.clone()));

    // 3. Tjek indledende status
    let _ = app.update(Message::RefreshGitStatus);
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: app.git_status_generation(),
        result: Ok(RepoSyncStatus::Synced),
        elapsed: std::time::Duration::from_millis(10),
    });
    assert!(
        matches!(
            app.git_sync_status(),
            RepoSyncStatus::Synced | RepoSyncStatus::UnpublishedCommits(0)
        ),
        "Initial status skal være synkroniseret eller 0 uudgivne commits"
    );

    // 4. Foretag modelændring (tilføj begreb)
    let c1 = Concept::new("Køretøj", "Transportmiddel", BelongsToDomain::Yes);
    app.project_mut().add_concept(c1).unwrap();

    // 5. Åbn Udgiv Model modal
    assert!(app.publish_modal().is_none());
    let _ = app.update(Message::OpenPublishModal);
    assert!(app.publish_modal().is_some(), "Udgiv modal skal være åben");

    // Modalen skal have en forhåndsvisning eller forslag
    let modal = app.publish_modal().unwrap();
    assert!(
        !modal.message.is_empty() || !modal.preview_events.is_empty(),
        "Modal skal indeholde forslag eller forhåndsvisning af ændringer"
    );

    // 6. Skriv kommentar og bekræft udgivelse
    let _ = app.update(Message::UpdatePublishMessage(
        "Første officielle version af køretøjsmodellen".into(),
    ));
    assert_eq!(
        app.publish_modal().unwrap().message,
        "Første officielle version af køretøjsmodellen"
    );

    let _ = app.update(Message::ConfirmPublish);
    assert!(
        app.publish_modal().is_none(),
        "Modal skal lukkes efter udgivelse"
    );

    // 7. Valider opdateret status og badge-tekst
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::UnpublishedCommits(1)
    );
    let badge_text = app.git_status_badge_text();
    assert!(
        badge_text.contains('1') || badge_text.contains("klar") || badge_text.contains("Udgivet"),
        "Badgetekst skal afspejle uudgivne commits: {}",
        badge_text
    );

    // 8. Oprydning
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_048_model_graph_timeline_element_history_and_conflict_resolver() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::git::GitService;
    use kant::features::model::merge::ModelConflict;
    use kant::ui::app::{App, ConflictChoice, Message};
    use uuid::Uuid;

    let temp_dir = std::env::temp_dir().join(format!("kant_test_048_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Initialiser repo og app
    GitService::init_repository(&temp_dir).expect("init repo");
    let mut app = App::new_with_path(Some(temp_dir.clone()));

    // 2. Opret og udgiv version 1 (Begreb Køretøj)
    let c1 = Concept::new("Køretøj", "Transportmiddel", BelongsToDomain::Yes);
    let c1_id = app.project_mut().add_concept(c1).unwrap();
    let _ = app.update(Message::OpenPublishModal);
    let _ = app.update(Message::UpdatePublishMessage(
        "Version 1: Oprettet Køretøj".into(),
    ));
    let _ = app.update(Message::ConfirmPublish);

    // 3. Opdater begreb og udgiv version 2
    let mut c1_updated = app.project().get_concept(c1_id).unwrap().clone();
    c1_updated.set_definition("Opdateret transportmiddel");
    app.project_mut().update_concept(c1_updated).unwrap();
    let _ = app.update(Message::OpenPublishModal);
    let _ = app.update(Message::UpdatePublishMessage(
        "Version 2: Opdateret definition".into(),
    ));
    let _ = app.update(Message::ConfirmPublish);

    // 4. Åbn Modelhistorik (Tidslinje)
    assert!(app.history_modal().is_none());
    let _ = app.update(Message::OpenModelHistoryModal);
    assert!(
        app.history_modal().is_some(),
        "Modelhistorik modal skal være åben"
    );
    let history = app.history_modal().unwrap();
    assert_eq!(
        history.commits.len(),
        2,
        "Historik skal indeholde 2 commits"
    );
    assert_eq!(
        history.commits[0].message,
        "Version 2: Opdateret definition"
    );
    assert_eq!(history.commits[1].message, "Version 1: Oprettet Køretøj");
    let _ = app.update(Message::CloseModelHistoryModal);
    assert!(app.history_modal().is_none());

    // 5. Åbn Element-specifik historik ("Time Travel" / Audit)
    let _ = app.update(Message::OpenElementHistoryModal(c1_id));
    assert!(
        app.history_modal().is_some(),
        "Elementhistorik modal skal være åben"
    );
    let elem_hist = app.history_modal().unwrap();
    assert_eq!(
        elem_hist.commits.len(),
        2,
        "c1 historik skal indeholde 2 commits"
    );
    assert_eq!(elem_hist.target_element_id, Some(c1_id));
    let _ = app.update(Message::CloseModelHistoryModal);

    // 6. Test Visuel Konfliktløser Modal (Conflict Resolver)
    assert!(app.conflict_resolver_modal().is_none());
    let conflict = ModelConflict {
        entity_kind: kant::features::model::merge::ConflictEntityKind::Concept(c1_id),
        entity_name: "Køretøj".into(),
        field_name: "definition".into(),
        base_value: "Transportmiddel".into(),
        our_value: "Lokalt ændret transportmiddel".into(),
        their_value: "Server ændret transportmiddel".into(),
    };
    let _ = app.update(Message::OpenConflictResolverModal(vec![conflict.clone()]));
    assert!(
        app.conflict_resolver_modal().is_some(),
        "Konfliktløser modal skal være åben"
    );
    let resolver = app.conflict_resolver_modal().unwrap();
    assert_eq!(resolver.conflicts.len(), 1);

    // Vælg "Theirs" (serverens version) og løs konflikten
    let _ = app.update(Message::ResolveConflict {
        index: 0,
        choice: ConflictChoice::Theirs,
    });
    assert_eq!(
        app.conflict_resolver_modal()
            .unwrap()
            .resolved_choices
            .get(&0),
        Some(&ConflictChoice::Theirs)
    );

    // Bekræft fletning
    let _ = app.update(Message::ApplyResolvedConflicts);
    assert!(
        app.conflict_resolver_modal().is_none(),
        "Konfliktløser skal lukkes efter anvendelse"
    );

    // Valider at begrebets definition nu er opdateret til 'theirs_value'
    assert_eq!(
        app.project().get_concept(c1_id).unwrap().definition(),
        "Server ændret transportmiddel"
    );

    // 7. Oprydning
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_task_049_git_connection_remote_configuration_and_clone_ui() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::git::service::GitService;
    use kant::ui::app::{EdgeApp, Message};
    use uuid::Uuid;

    if !GitService::is_git_installed() {
        eprintln!("Skipping test: git binary not found");
        return;
    }

    let temp_root = std::env::temp_dir().join(format!("kant_test_049_{}", Uuid::new_v4()));
    let repo_dir = temp_root.join("local_repo");
    let clone_dir = temp_root.join("cloned_repo");
    let _ = std::fs::create_dir_all(&repo_dir);

    // 1. Initialiser repo og test direkte GitService remote & identity funktioner
    assert!(GitService::init_repository(&repo_dir).is_ok());

    // Før remote tilføjes skal den returnere None
    let initial_remote = GitService::get_remote_url(&repo_dir).unwrap();
    assert_eq!(initial_remote, None);

    // Tilføj fjernlager URL (origin)
    let test_remote_url = "https://github.com/kant-fda/model-arkiv.git";
    GitService::set_remote_url(&repo_dir, test_remote_url).expect("set_remote_url skal lykkes");
    let remote_after_set = GitService::get_remote_url(&repo_dir).unwrap();
    assert_eq!(remote_after_set, Some(test_remote_url.to_string()));

    // Sæt forfatteridentitet
    GitService::set_user_identity(&repo_dir, "Søren Arkitekt", "soren@fda.dk")
        .expect("set_user_identity skal lykkes");
    let (name, email) = GitService::get_user_identity(&repo_dir).unwrap();
    assert_eq!(name, Some("Søren Arkitekt".to_string()));
    assert_eq!(email, Some("soren@fda.dk".to_string()));

    // Fjern fjernlager
    GitService::remove_remote(&repo_dir).expect("remove_remote skal lykkes");
    let remote_after_remove = GitService::get_remote_url(&repo_dir).unwrap();
    assert_eq!(remote_after_remove, None);

    // 2. Klargør en commit i repo_dir så det kan klones
    let mut initial_project = kant::features::model::ModelProject::default();
    initial_project.metadata_mut().set_name("Klonbar Model");
    let c = Concept::new("Kunde", "En person der køber ydelser", BelongsToDomain::Yes);
    initial_project.add_concept(c).unwrap();
    kant::features::model::storage::ProjectStorage::save_to_directory(&initial_project, &repo_dir)
        .unwrap();
    GitService::publish_model(&repo_dir, &initial_project, "Initiel model før klon").unwrap();

    // Test GitService::clone_repository
    GitService::clone_repository(repo_dir.to_str().unwrap(), &clone_dir)
        .expect("Git clone skal lykkes");
    assert!(clone_dir.join(".kant").exists() || clone_dir.join(".git").exists());

    // 3. Test UI App modaler for Git-forbindelse og Klon
    let mut app = EdgeApp::new();
    app.set_active_file_path(Some(repo_dir.clone()));

    // Åbn forbindelsesdialog
    assert!(app.git_connection_modal().is_none());
    let _ = app.update(Message::OpenGitConnectionModal);
    assert!(
        app.git_connection_modal().is_some(),
        "GitConnectionModal skal være åben"
    );

    // Udfyld formularfelter
    let _ = app.update(Message::UpdateGitRemoteUrl(
        "https://github.com/kant-fda/central.git".into(),
    ));
    let _ = app.update(Message::UpdateGitAuthorName("Mette Modeller".into()));
    let _ = app.update(Message::UpdateGitAuthorEmail("mette@fda.dk".into()));

    let modal = app.git_connection_modal().unwrap();
    assert_eq!(modal.remote_url, "https://github.com/kant-fda/central.git");
    assert_eq!(modal.author_name, "Mette Modeller");
    assert_eq!(modal.author_email, "mette@fda.dk");

    // Gem forbindelsen
    let _ = app.update(Message::SaveGitConnection);
    assert!(
        app.git_connection_modal().is_none(),
        "Modal skal lukkes efter gem"
    );

    // Bekræft at GitService afspejler ændringerne på disk
    assert_eq!(
        GitService::get_remote_url(&repo_dir).unwrap(),
        Some("https://github.com/kant-fda/central.git".into())
    );
    let (saved_name, saved_email) = GitService::get_user_identity(&repo_dir).unwrap();
    assert_eq!(saved_name, Some("Mette Modeller".into()));
    assert_eq!(saved_email, Some("mette@fda.dk".into()));

    // Test Klon modal i UI
    assert!(app.git_clone_modal().is_none());
    let _ = app.update(Message::OpenGitCloneModal);
    assert!(
        app.git_clone_modal().is_some(),
        "GitCloneModal skal være åben"
    );

    let ui_clone_target = temp_root.join("ui_cloned");
    let _ = app.update(Message::UpdateGitCloneUrl(
        repo_dir.to_str().unwrap().into(),
    ));
    let _ = app.update(Message::UpdateGitCloneTargetDir(
        ui_clone_target.to_str().unwrap().into(),
    ));

    // Udfør klon i appen
    let _ = app.update(Message::ExecuteGitClone);
    assert!(
        app.git_clone_modal().is_none(),
        "Klon modal skal lukke ved succes"
    );
    assert!(ui_clone_target.exists(), "Klonet mappe skal eksistere");
    assert_eq!(app.project().metadata().name(), "Klonbar Model");

    // 4. Test håndtering af relativ filsti uden mappe (f.eks. standarden "model.kant.json")
    let mut rel_app = EdgeApp::new();
    rel_app.set_active_file_path(Some(std::path::PathBuf::from("model.kant.json")));
    assert_eq!(
        rel_app.repo_dir(),
        None,
        "Relativ fil uden dedikeret mappe må ikke returnere '.'"
    );
    let _ = rel_app.update(Message::OpenGitConnectionModal);
    assert!(rel_app.git_connection_modal().is_some());

    // 5. Oprydning
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_task_050_coarchi_publish_sync_workflow_and_token_authentication() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::ui::app::{EdgeApp, Message};
    use uuid::Uuid;

    if !GitService::is_git_installed() {
        eprintln!("Skipping test: git binary not found");
        return;
    }

    let temp_root = std::env::temp_dir().join(format!("kant_test_050_{}", Uuid::new_v4()));
    let remote_bare_dir = temp_root.join("remote.git");
    let local_dir = temp_root.join("local_model");
    let _ = std::fs::create_dir_all(&local_dir);

    // 1. Opret et bare git repository som vores 'remote origin'
    let _ = std::process::Command::new("git")
        .args([
            "init",
            "--bare",
            "-b",
            "main",
            remote_bare_dir.to_str().unwrap(),
        ])
        .output();

    // 2. Initialiser lokalt repository og sæt remote til bare repo
    assert!(GitService::init_repository(&local_dir).is_ok());
    GitService::set_remote_url(&local_dir, remote_bare_dir.to_str().unwrap()).unwrap();
    GitService::set_user_identity(&local_dir, "Test Modeller", "modeller@kant.local").unwrap();

    // 3. Opret en model og udgiv lokalt med publish_model
    let mut project = kant::features::model::ModelProject::default();
    project.metadata_mut().set_name("coArchi Model");
    let c = Concept::new("Vejafgift", "Gebyr for passage", BelongsToDomain::Yes);
    project.add_concept(c).unwrap();

    let commit_oid =
        GitService::publish_model(&local_dir, &project, "Første modeludgivelse").unwrap();
    assert!(!commit_oid.is_empty());

    // 4. Test direkte push_model fra GitService til remote
    GitService::push_model(&local_dir, "origin")
        .expect("push_model skal skubbe commits til remote");

    // Bekræft at remote har modtaget committet
    let remote_head = std::process::Command::new("git")
        .args([
            "--git-dir",
            remote_bare_dir.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap();
    assert_eq!(remote_head, commit_oid);

    // 5. Test UI Token håndtering i GitConnectionModal
    let mut app = EdgeApp::new();
    app.set_active_file_path(Some(local_dir.clone()));

    let _ = app.update(Message::OpenGitConnectionModal);
    assert!(app.git_connection_modal().is_some());

    // Indtast URL og token
    let _ = app.update(Message::UpdateGitRemoteUrl(
        "https://github.com/organisation/fda-model.git".into(),
    ));
    let _ = app.update(Message::UpdateGitToken("ghp_secret_token_12345".into()));

    let modal = app.git_connection_modal().unwrap();
    assert_eq!(
        modal.remote_url,
        "https://github.com/organisation/fda-model.git"
    );
    assert_eq!(modal.token, "ghp_secret_token_12345");

    // Gem forbindelsen: Token skal indlejres sikkert i remote URL
    let _ = app.update(Message::SaveGitConnection);
    assert!(app.git_connection_modal().is_none());

    let saved_remote = GitService::get_remote_url(&local_dir).unwrap().unwrap();
    assert!(
        saved_remote.contains("ghp_secret_token_12345@github.com"),
        "Gemt remote skal indeholde token til autentifikation: {}",
        saved_remote
    );

    // Når modal genåbnes, skal URL vises ren UDEN synligt token i URL-feltet, men i token-feltet
    let _ = app.update(Message::OpenGitConnectionModal);
    let modal_reopened = app.git_connection_modal().unwrap();
    assert_eq!(
        modal_reopened.remote_url, "https://github.com/organisation/fda-model.git",
        "URL felt skal være renset for token"
    );
    assert_eq!(
        modal_reopened.token, "ghp_secret_token_12345",
        "Token felt skal indeholde det udtrukne token"
    );
    let _ = app.update(Message::CloseGitConnectionModal);

    // 6. Test coArchi udgivelseskæde i UI (OpenPublishModal -> ConfirmPublish -> Auto-Pull & Push)
    // Sæt remoten tilbage til vores bare repo
    GitService::set_remote_url(&local_dir, remote_bare_dir.to_str().unwrap()).unwrap();

    let _ = app.update(Message::OpenPublishModal);
    assert!(app.publish_modal().is_some());

    // Udfør udgivelse
    let _ = app.update(Message::ConfirmPublish);
    assert!(
        app.publish_modal().is_none(),
        "Publish modal skal lukke efter udgivelse"
    );

    // Status skal være Synced
    assert_eq!(app.git_status_badge_text(), "Synkroniseret");

    // 7. Test reaktiv status pill ved tilføjelse/ændring af model-elementer
    let concept = kant::features::concepts::Concept::new(
        "Reaktiv Begreb",
        "En test af status pill",
        kant::features::concepts::BelongsToDomain::Yes,
    );
    app.project_mut().add_concept(concept).unwrap();
    app.trigger_autosave();
    let _ = app.update(Message::GitSyncStatusRefreshed {
        generation: app.git_status_generation(),
        result: Ok(RepoSyncStatus::PendingChanges),
        elapsed: std::time::Duration::from_millis(10),
    });
    assert_eq!(
        app.git_status_badge_text(),
        "Lokale ændringer • Commit og send",
        "Status pill skal prompte skifte til 'Lokale ændringer • Commit og send' når elementer tilføjes"
    );

    // Oprydning
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_task_051_model_isolation_and_git_provider_agnosticism() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::ui::app::ConceptFormField;
    use std::path::Path;

    // 1. Repo name parsing fra URL
    assert_eq!(
        GitService::parse_repo_name(
            "https://github.com/rolfmadsen/kant_begrebs_og_informationsmodeller.git"
        ),
        Some("kant_begrebs_og_informationsmodeller".to_string())
    );
    assert_eq!(
        GitService::parse_repo_name("https://gitlab.com/organisation/fda-vejafgift"),
        Some("fda-vejafgift".to_string())
    );
    assert_eq!(
        GitService::parse_repo_name("git@gitea.local:models/begreber.git"),
        Some("begreber".to_string())
    );

    // 2. Provider-agnostisk URL opbygning (GitLab, Gitea, GitHub, Custom)
    let gitlab_url = GitService::build_authenticated_url(
        "https://gitlab.com/org/fda-model.git",
        "glpat-token123",
    );
    assert_eq!(
        gitlab_url,
        "https://oauth2:glpat-token123@gitlab.com/org/fda-model.git"
    );

    let gitea_url = GitService::build_authenticated_url(
        "https://gitea.internal/org/fda-model.git",
        "my_gitea_token",
    );
    assert!(
        gitea_url.contains("my_gitea_token@gitea.internal"),
        "Gitea / intern Git URL skal indeholde token: {}",
        gitea_url
    );

    let github_url =
        GitService::build_authenticated_url("https://github.com/org/fda-model.git", "ghp_token456");
    assert_eq!(
        github_url,
        "https://x-access-token:ghp_token456@github.com/org/fda-model.git"
    );

    // 3. Sikkerhedsguardrail: Isoleret modelkatalog
    // Roden af vores edge repo indeholder Cargo.toml og må ALDRIG betragtes som en sikker modelmappe
    assert!(
        !GitService::is_safe_model_repo_dir(Path::new(".")),
        "Aktuel software-kildekodemappe med Cargo.toml må IKKE tillades som model-repo"
    );

    let temp_root =
        std::env::temp_dir().join(format!("kant_test_051_iso_{}", uuid::Uuid::new_v4()));
    let model_dir = temp_root.join("dedicated_model_project");
    std::fs::create_dir_all(&model_dir).unwrap();
    assert!(
        GitService::is_safe_model_repo_dir(&model_dir),
        "En dedikeret tom modelmappe skal være tilladt"
    );

    // 4. App state isolation:
    // Uden eksplicit filsti eller ved "Nyt projekt" skal repo_dir være None
    let mut app = App::new_with_path(None);
    assert_eq!(
        app.repo_dir(),
        None,
        "Nyt projekt uden valgt sti må ALDRIG falde tilbage til '.'!"
    );

    // Opret et begreb i hukommelsen
    let _ = app.update(Message::StartNewConcept);
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::PreferredTerm,
        "TestBegreb".to_string(),
    ));
    let _ = app.update(Message::UpdateConceptField(
        ConceptFormField::Definition,
        "En definition af testbegrebet".to_string(),
    ));
    let _ = app.update(Message::SaveConcept);
    assert_eq!(app.project().concepts().len(), 1);

    // Tryk "Nyt projekt" -> Skal nulstille til ren tavle og repo_dir() skal være None
    let _ = app.update(Message::NewProject);
    assert_eq!(app.project().concepts().len(), 0);
    assert_eq!(app.repo_dir(), None);
    assert_eq!(app.git_sync_status(), &RepoSyncStatus::Uninitialized);

    // 5. Kloning af tomt repository udleder automatisk projektnavn og initialiserer rent projekt
    let remote_dir = temp_root.join("remote_bare_repo.git");
    let _ = std::process::Command::new("git")
        .args(["init", "--bare", "-b", "main", remote_dir.to_str().unwrap()])
        .output();

    let nested_parent = temp_root.join("new_models_folder");
    assert!(
        !nested_parent.exists(),
        "Forældremappe må ikke eksistere forud for testen"
    );
    let clone_dest = nested_parent.join("cloned_fda_vejafgift");
    let remote_url = format!("file://{}", remote_dir.display());

    let _ = app.update(Message::OpenGitCloneModal);
    let _ = app.update(Message::UpdateGitCloneUrl(
        "https://gitlab.com/org/fda-vejafgift.git".into(),
    ));
    let _ = app.update(Message::UpdateGitCloneToken(
        "glpat-secret-token-123".into(),
    ));
    assert_eq!(
        app.git_clone_modal().unwrap().token,
        "glpat-secret-token-123"
    );

    // Simulér valg af destinationsmappe via folder-dialog
    let _ = app.update(Message::GitCloneTargetDirSelected(
        kant::ui::file_dialog::DialogResult::Selected(clone_dest.clone()),
    ));
    assert_eq!(
        app.git_clone_modal().unwrap().target_dir,
        clone_dest.display().to_string()
    );

    // Verificer at ændring i URL ikke overskriver brugerens valgte mappe
    let _ = app.update(Message::UpdateGitCloneUrl(remote_url));
    assert_eq!(
        app.git_clone_modal().unwrap().target_dir,
        clone_dest.display().to_string()
    );

    // Udfør klon - skal automatisk oprette nested_parent og lykkes
    let _ = app.update(Message::ExecuteGitClone);

    // Verificer at projektet er indlæst, forældremappen oprettet og modalen lukket
    assert!(
        nested_parent.exists(),
        "Forældremappen skal være oprettet automatisk"
    );
    assert_eq!(app.repo_dir(), Some(clone_dest.clone()));
    assert!(app.git_clone_modal().is_none());

    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_task_052_recent_models_and_decomposed_path_resolution() {
    use kant::features::concepts::{BelongsToDomain, Concept};
    use kant::features::model::recent::RecentStore;
    use kant::features::model::storage::ProjectStorage;
    use kant::features::model::ModelProject;
    use kant::ui::app::{EdgeApp, Message};
    use uuid::Uuid;

    let temp_root = std::env::temp_dir().join(format!("kant_test_052_{}", Uuid::new_v4()));
    let model_dir = temp_root.join("test_model_repo");
    let _ = std::fs::create_dir_all(&model_dir);

    // 1. Opret og gem et dekomponeret modelprojekt
    let mut proj = ModelProject::default();
    proj.metadata_mut().set_name("FDA Sundhedsmodel");
    let concept = Concept::new(
        "Patient",
        "En person modtager behandling",
        BelongsToDomain::Yes,
    );
    proj.add_concept(concept).unwrap();
    ProjectStorage::save_to_directory(&proj, &model_dir).expect("skal gemme dekomponeret model");

    let metadata_file = model_dir.join(".kant").join("metadata.json");
    assert!(metadata_file.exists(), ".kant/metadata.json skal eksistere");

    // 2. Test effective_model_path normalisering
    let eff1 = ProjectStorage::effective_model_path(&metadata_file);
    assert_eq!(
        eff1, model_dir,
        "metadata.json skal normaliseres til model-roden"
    );

    let eff2 = ProjectStorage::effective_model_path(&model_dir.join(".kant"));
    assert_eq!(eff2, model_dir.join(".kant"), "mappesti forbliver intakt");

    // 3. Test at ProjectStorage::load indlæser den fulde model selvom stien er .kant/metadata.json
    let loaded_from_metadata_file = ProjectStorage::load(&metadata_file)
        .expect("load fra metadata.json skal automatisk detektere .kant/ og hente hele modellen");
    assert_eq!(
        loaded_from_metadata_file.metadata().name(),
        "FDA Sundhedsmodel"
    );
    assert_eq!(loaded_from_metadata_file.concepts().len(), 1);
    assert_eq!(
        loaded_from_metadata_file.concepts()[0].preferred_term(),
        "Patient"
    );

    // 4. Test RecentStore normalisering og tracking
    let mut store = RecentStore::default();
    store.record_opened(&metadata_file);
    assert_eq!(store.last_opened, Some(model_dir.clone()));
    assert_eq!(store.recent_paths, vec![model_dir.clone()]);
    assert_eq!(store.get_auto_open_candidate(), Some(model_dir.clone()));

    // 5. Test i UI App: Åbning af metadata_file sætter roden korrekt og indlæser begreber
    let mut app = EdgeApp::new_with_path(None);
    let _ = app.update(Message::OpenProjectFile(metadata_file));
    assert_eq!(app.current_file_path(), Some(&model_dir));
    assert_eq!(app.project().concepts().len(), 1);
    assert_eq!(app.project().metadata().name(), "FDA Sundhedsmodel");

    // 6. Oprydning
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_task_053_windows_git_candidates_and_pathbuf_resolution() {
    use kant::features::git::GitService;
    use std::path::PathBuf;

    // 1. Uden LOCALAPPDATA skal standard Program Files stier returneres
    let candidates_default = GitService::windows_git_candidates(None);
    assert!(
        candidates_default.contains(&PathBuf::from(r"C:\Program Files\Git\cmd\git.exe")),
        "skal indeholde 64-bit Program Files sti"
    );
    assert!(
        candidates_default.contains(&PathBuf::from(r"C:\Program Files (x86)\Git\cmd\git.exe")),
        "skal indeholde 32-bit Program Files (x86) sti"
    );
    assert_eq!(candidates_default.len(), 2);

    // 2. Med LOCALAPPDATA skal brugerens lokale sti tilføjes
    let mock_local_appdata = r"C:\Users\Developer\AppData\Local";
    let candidates_with_local = GitService::windows_git_candidates(Some(mock_local_appdata));
    assert_eq!(candidates_with_local.len(), 3);
    assert_eq!(
        candidates_with_local[2],
        PathBuf::from(mock_local_appdata).join(r"Programs\Git\cmd\git.exe"),
        "skal tilføje lokal appdata kandidat"
    );

    // 3. Tom LOCALAPPDATA må ikke tilføje en tom relativ sti
    let candidates_empty = GitService::windows_git_candidates(Some("   "));
    assert_eq!(candidates_empty.len(), 2);
}

#[test]
fn test_task_054_windows_rendering_and_git_async_performance() {
    use kant::features::git::service::{GitService, RepoSyncStatus};
    use kant::ui::app::{EdgeApp, Message, PublishAsyncResult};
    use std::time::Instant;

    // 1. AC1: Test at OnceLock caching af Git-detektering gør gentagne kald lynhurtige (<10ms for 50 kald)
    if GitService::is_git_installed() {
        let t0 = Instant::now();
        for _ in 0..50 {
            assert!(GitService::is_git_installed());
            let _ = GitService::resolve_git_binary();
        }
        let elapsed = t0.elapsed();
        assert!(
            elapsed.as_millis() < 20,
            "50 gentagne kald til is_git_installed og resolve_git_binary skal være cached via OnceLock (<20ms), men tog: {:?}",
            elapsed
        );
    }

    // 2. AC3: Test at App::new_with_path starter op uden at blokere på remote sync status
    let app = EdgeApp::new();
    assert_eq!(
        *app.git_sync_status(),
        RepoSyncStatus::Uninitialized,
        "Opstart skal ikke synkront køre tunge git-kommandoer, men starte med Uninitialized"
    );

    // 3. AC4: Test at PublishCompleted opdaterer tilstand korrekt
    let temp_root = std::env::temp_dir().join(format!("kant_test_054_{}", uuid::Uuid::new_v4()));
    let local_dir = temp_root.join("local_model");
    let _ = std::fs::create_dir_all(&local_dir);
    let _ = GitService::init_repository(&local_dir);

    let mut app = EdgeApp::new();
    app.set_active_file_path(Some(local_dir.clone()));
    let _ = app.update(Message::OpenPublishModal);
    assert!(app.publish_modal().is_some());

    // Simulér asynkron færdiggørelse via Message::PublishCompleted
    let _ = app.update(Message::PublishCompleted(PublishAsyncResult::Success {
        sync_status: RepoSyncStatus::Synced,
    }));
    assert!(
        app.publish_modal().is_none(),
        "Modal skal lukke efter vellykket asynkron publish"
    );
    assert_eq!(*app.git_sync_status(), RepoSyncStatus::Synced);

    let _ = std::fs::remove_dir_all(&temp_root);
}
