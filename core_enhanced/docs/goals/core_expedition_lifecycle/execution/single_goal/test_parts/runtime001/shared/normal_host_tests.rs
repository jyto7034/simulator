//! Actual filesystem crash tests, selected only with the runner's approved
//! same-executable configuration. The child is not counted as a test result.
use crate as host_server;
#[path = "normal_host_support.rs"]
mod support;
#[path = "normal_return_cycle_tests.rs"]
mod return_cycle;
use support::*;
use crate::mod_admission::ApprovedPersistence;
use game_core::application::ApplicationSnapshotKind;
use serde_json::{json, Value};
use std::{fs, path::{Path, PathBuf}, process::Command, sync::Arc};

#[test]
#[ignore = "subprocess-only helper; run u12_normal_abandon_crash_boundaries_recover_exact_pair"]
fn u12_normal_lifecycle_crash_child() {
    let run = || -> Result<()> {
        let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("actual approved config required")?);
        let out = PathBuf::from(std::env::var_os("U12_CHILD_OUTPUT").ok_or("new child output required")?);
        let stage = std::env::var("U12_CHILD_CRASH_STAGE")?;
        let session = open(&config)?;
        let root = out.join("save");
        let persistence = ApprovedPersistence::open(&root, Arc::clone(&session.host))?;
        let (mut app, mut required, anchored) = new_application(&session, &persistence)?;
        let mut revision = Some(anchored.revision);
        execute(&session, &persistence, &mut app, &mut required, &mut revision,
            json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
        finish_start(&session, &persistence, &mut app, &mut required, &mut revision, &out)?;
        let profile = snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?;
        assert_eq!(profile["lifecycle"]["expedition_sequence"], 1);
        assert!(profile["lifecycle"]["active_expedition"].is_object());
        save(&out.join("before.json"), &json!({"profile":profile,"roots":roots(&app)?,"revision":revision,"required":required}))?;
        std::env::set_var("U04_DURABLE_CRASH_AT", stage);
        execute(&session, &persistence, &mut app, &mut required, &mut revision, json!({"type":"abandon_run"}))?;
        Err("requested actual durable crash stage was not reached".into())
    };
    run().unwrap();
}

#[test]
#[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
fn u12_normal_abandon_crash_boundaries_recover_exact_pair() {
    let run = || -> Result<()> {
        let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("actual approved config required")?);
        let parent = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("exclusive evidence parent required")?);
        let out = parent.join("abandon_crash"); fs::create_dir(&out)?;
        for stage in ["intent_temp_synced", "intent_renamed", "intent_synced", "before_intent_clear", "intent_removed", "intent_clear_synced"] {
            let case = out.join(stage); fs::create_dir(&case)?;
            let child = Command::new(std::env::current_exe()?).args(["--exact",
                "mod_admission::u12_normal_host_tests::u12_normal_lifecycle_crash_child",
                "--ignored", "--nocapture", "--test-threads=1"])
                .env("U12_NORMAL_CONFIG", &config).env("U12_CHILD_OUTPUT", &case)
                .env("U12_CHILD_CRASH_STAGE", stage).env_remove("U04_DURABLE_CRASH_AT").output()?;
            fs::write(case.join("child.stdout"), &child.stdout)?;
            fs::write(case.join("child.stderr"), &child.stderr)?;
            assert_eq!(child.status.code(), Some(86), "actual filesystem stage {stage}, no filter-zero or ordinary exit");
            save(&case.join("child_exit.json"), &json!({"exit":child.status.code(),"stage":stage}))?;
            let before: Value = serde_json::from_slice(&fs::read(case.join("before.json"))?)?;
            let root = case.join("save");
            let session = open(&config)?;
            let accepted = stage != "intent_temp_synced";
            for recovery in 0..2 {
                let persistence = ApprovedPersistence::open(&root, Arc::clone(&session.host))?;
                let profile = serde_json::to_value(persistence.player_profile()?)?;
                let active = persistence.snapshot()?;
                assert_eq!(profile["lifecycle"]["expedition_sequence"], 1);
                assert_eq!(profile["lifecycle"]["vault"], before["profile"]["lifecycle"]["vault"]);
                assert_eq!(profile["lifecycle"]["first_return_story"], "Never");
                if accepted {
                    assert!(profile["lifecycle"]["active_expedition"].is_null());
                    assert!(active.checkpoint.is_none());
                } else {
                    assert_eq!(profile, before["profile"]);
                    assert!(active.checkpoint.is_some());
                }
                let (app, _, _) = restore(&session, &persistence)?;
                if accepted { assert!(app.is_not_started()); }
                else { assert_eq!(snapshot(&app, ApplicationSnapshotKind::Run)?["roster"]["employees"].as_array().unwrap().len(), 3); }
                save(&case.join(format!("recovery_{recovery}.json")), &json!({"accepted":accepted,"profile":profile,"roots":roots(&app)?}))?;
                drop(persistence);
                if recovery == 0 { save(&case.join("first_recovery_bytes.json"), &file_bytes(&root)?)?; }
                else {
                    let first = fs::read(case.join("first_recovery_bytes.json"))?;
                    let first: std::collections::BTreeMap<String, Vec<u8>> = serde_json::from_slice(&first)?;
                    assert_eq!(file_bytes(&root)?, first, "second recovery cannot reapply or rewrite the accepted pair");
                }
            }
        }
        save(&out.join("RESULT.json"), &json!({"passed":true,"actual_crash_processes":6,"recovery_passes_per_case":2}))?;
        Ok(())
    };
    run().unwrap();
}

#[test]
#[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
fn u12_normal_legacy_profile_active_and_journal_headers_protect_originals() {
    let run = || -> Result<()> {
        let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
        let out = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("output")?).join("legacy_originals");
        fs::create_dir(&out)?;
        let session = open(&config)?;
        for (name, bytes) in [
            ("active_run.json", br#"{"version":5}"#.as_slice()),
            ("active_run.json", br#"{"version":6}"#.as_slice()),
            ("player_profile.json", br#"{"version":3}"#.as_slice()),
            ("application_commit.pending.json", br#"{"intent":{"version":1}}"#.as_slice()),
            ("application_commit.pending.json", br#"{"intent":{"version":2}}"#.as_slice()),
        ].into_iter().enumerate().map(|(index, pair)| (format!("{index:02}"), pair)) {
            let (file, bytes) = bytes;
            let root = out.join(name); fs::create_dir(&root)?;
            fs::write(root.join(file), bytes)?;
            let before = file_bytes(&root)?;
            assert!(ApprovedPersistence::open(&root, Arc::clone(&session.host)).is_err());
            assert_eq!(file_bytes(&root)?, before);
        }
        save(&out.join("RESULT.json"), &json!({"passed":true,"scope":"version preflight protects originals; checksum-valid old-pair cases are separate"}))?;
        Ok(())
    };
    run().unwrap();
}

#[test]
fn u12_profile_byte_ceiling_is_symmetric_and_preaccept_pair_rejection_preserves_files() {
    use crate::persistence::{active_run_store::ActiveRunStore,
        player_profile_store::{PlayerProfileLoad, PlayerProfilePersistence, PlayerProfileStore}};
    use game_core::game::encounter_compendium::PlayerProfileState;
    let run = || -> Result<()> {
        const LIMIT: usize = 16 * 1024 * 1024;
        let root = tempfile::tempdir()?;
        let store = PlayerProfileStore::open(root.path())?;
        let mut profile = PlayerProfileState::default();
        profile.encounter_compendium.registered_ids.insert("x".into());
        store.publish(&profile)?;
        let small_len = fs::read(root.path().join("player_profile.json"))?.len();
        let id = "x".repeat(1 + LIMIT - small_len);
        profile.encounter_compendium.registered_ids = std::collections::BTreeSet::from([id.clone()]);
        profile.validate_persisted()?;
        store.publish(&profile)?;
        assert_eq!(fs::metadata(root.path().join("player_profile.json"))?.len(), LIMIT as u64);
        let PlayerProfileLoad::Loaded { profile: decoded, .. } = store.load()? else { panic!("at-limit writer output must reload"); };
        assert_eq!(decoded, profile);
        let mut oversized = profile.clone();
        oversized.encounter_compendium.registered_ids = std::collections::BTreeSet::from([format!("{id}x")]);
        oversized.validate_persisted()?;
        let files_before = file_bytes(root.path())?;
        assert!(store.publish(&oversized).is_err());
        assert_eq!(file_bytes(root.path())?, files_before);
        // Start the paired path with empty knowledge. Otherwise merging two
        // different long IDs would test approximately twice the byte limit.
        let paired_root = tempfile::tempdir()?;
        let active = ActiveRunStore::open(paired_root.path())?;
        let mut persistence = PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(paired_root.path())?);
        let paired_before = persistence.profile()?;
        let paired_files = file_bytes(paired_root.path())?;
        let revision = active.snapshot()?.revision;
        assert!(persistence.accept_application(&active, revision, None, &oversized).is_err());
        assert_eq!(persistence.profile()?, paired_before);
        assert_eq!(active.snapshot()?.revision, revision);
        assert_eq!(file_bytes(paired_root.path())?, paired_files);
        let accepted = persistence.accept_application(&active, revision, None, &profile)?;
        assert!(!accepted.recovery_pending);
        assert_eq!(active.snapshot()?.revision, accepted.revision);
        assert_eq!(persistence.profile()?, profile);
        assert_eq!(fs::metadata(paired_root.path().join("player_profile.json"))?.len(), LIMIT as u64);
        assert!(!paired_root.path().join("application_commit.pending.json").exists());
        Ok(())
    };
    run().unwrap();
}

#[test]
#[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
fn u12_normal_unanchored_and_stale_initial_drafts_cannot_overwrite_newer_lifecycle() {
    use game_core::application::ApplicationCommand;
    let run = || -> Result<()> {
        let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
        let out = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("output")?).join("initial_stale_cas");
        fs::create_dir(&out)?;
        let session = open(&config)?;
        for anchored in [false, true] {
            let case = out.join(if anchored { "some_initial_revision" } else { "none_initial_revision" });
            fs::create_dir(&case)?;
            let root = case.join("save");
            let persistence = ApprovedPersistence::open(&root, Arc::clone(&session.host))?;
            let (mut draft_a, required_a, anchored_a) = new_application(&session, &persistence)?;
            let a_before = roots(&draft_a)?;
            let initial_revision = anchored_a.revision;
            let mut expected_a = if anchored { Some(initial_revision) } else { None };
            let command = ApplicationCommand::from_document(json!({"type":"start_new_game",
                "game_mode":"Expedition","replace_active_run":false}).into())?;
            let (prepared_a, next_a) = session.host.prepare_with_future_generation(
                &mut draft_a, &required_a, PLAYER, command, Default::default())
                .map_err(|error| format!("approved competing draft preparation failed: {error:?}"))?;
            // B genuinely publishes a full departure after A's draft exists.
            let (mut actor_b, mut required_b, anchored_b) = new_application(&session, &persistence)?;
            let mut expected_b = Some(anchored_b.revision);
            execute(&session, &persistence, &mut actor_b, &mut required_b, &mut expected_b,
                json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
            finish_start(&session, &persistence, &mut actor_b, &mut required_b, &mut expected_b, &case)?;
            let b_before = roots(&actor_b)?;
            let profile_before = persistence.player_profile()?;
            assert!(profile_before.lifecycle.active_expedition.is_some());
            let files_before = file_bytes(&root)?;
            let revision_before = persistence.snapshot()?.revision;
            let result = persistence.publish(prepared_a, &next_a, &mut expected_a, None, true);
            assert!(result.is_err(), "A cannot acquire B's latest CAS merely by having no expected revision");
            assert_eq!(roots(&draft_a)?, a_before);
            assert_eq!(roots(&actor_b)?, b_before);
            assert_eq!(persistence.player_profile()?, profile_before);
            assert_eq!(persistence.snapshot()?.revision, revision_before);
            assert_eq!(file_bytes(&root)?, files_before);
            save(&case.join("RESULT.json"), &json!({"passed":true,"anchored":anchored,
                "initial_revision":initial_revision,"newer_revision":revision_before,
                "rejection":result.err(),"a_unchanged":true,"b_unchanged":true,"exact_files_unchanged":true}))?;
        }
        Ok(())
    };
    run().unwrap();
}

mod actor_management {
    use super::*;
    use actix::{Actor, Addr, Context, Handler, Message, MessageResult};
    use crate::game::{load_balance_actor::{LoadBalanceActor, messages::GetOrCreatePlayerActor},
        player_game_actor::{PlayerGameActor, messages::*}};
    use game_core::{application::ApplicationCommand, game::data::ValidatedGameData};
    use uuid::Uuid;

    struct Sink;
    impl Actor for Sink { type Context = Context<Self>; }
    impl Handler<PlayerGameServerMessage> for Sink {
        type Result = ();
        fn handle(&mut self, _: PlayerGameServerMessage, _: &mut Context<Self>) {}
    }
    impl Handler<ForceDisconnect> for Sink {
        type Result = ();
        fn handle(&mut self, _: ForceDisconnect, _: &mut Context<Self>) {}
    }
    #[derive(Message)]
    #[rtype(result = "Value")]
    struct ReadLifecycle;
    impl Handler<ReadLifecycle> for PlayerGameActor {
        type Result = MessageResult<ReadLifecycle>;
        fn handle(&mut self, _: ReadLifecycle, _: &mut Context<Self>) -> Self::Result {
            MessageResult(roots(&self.game_core).unwrap())
        }
    }
    #[derive(Message)]
    #[rtype(result = "Value")]
    struct ReadRun;
    impl Handler<ReadRun> for PlayerGameActor {
        type Result = MessageResult<ReadRun>;
        fn handle(&mut self, _: ReadRun, _: &mut Context<Self>) -> Self::Result {
            MessageResult(snapshot(&self.game_core, ApplicationSnapshotKind::Run).unwrap())
        }
    }
    #[derive(Message)]
    #[rtype(result = "Value")]
    struct ReadProfile;
    impl Handler<ReadProfile> for PlayerGameActor {
        type Result = MessageResult<ReadProfile>;
        fn handle(&mut self, _: ReadProfile, _: &mut Context<Self>) -> Self::Result {
            MessageResult(snapshot(&self.game_core, ApplicationSnapshotKind::PlayerProfile).unwrap())
        }
    }
    async fn actor(lb: &Addr<LoadBalanceActor>, admission: &crate::mod_admission::AdmissionSession)
        -> Result<(Addr<PlayerGameActor>, Uuid, Addr<Sink>)> {
        // The approved LoadBalanceActor creates through its installed host.
        // Supply the same selected generation to the message's required field;
        // the unapproved constructor branch is not used by these callers.
        let (game_data, _) = admission.host.future_generation()?;
        let actor = lb.send(GetOrCreatePlayerActor { player_id: PLAYER, run_seed: 12077,
            game_data }).await?;
        let session = Uuid::new_v4();
        let sink = Sink.start();
        actor.send(AttachSession { session_id: session, socket: sink.clone().recipient(), control: sink.clone().recipient() })
            .await?.map_err(|error| format!("attach: {} {}", error.code, error.message))?;
        Ok((actor, session, sink))
    }
    async fn command(actor: &Addr<PlayerGameActor>, session: Uuid, document: Value)
        -> Result<std::result::Result<CommandExecutionResult, String>> {
        let behavior = ApplicationCommand::from_document(document.into())?;
        Ok(actor.send(ExecutePlayerBehavior { session_id: session, request_id: Uuid::new_v4().to_string(),
            behavior, battle_authority: None, battle_side_message: BattleSideMessageKind::BattleUpdate }).await?
            .map_err(|error| format!("{} {}", error.code, error.message)))
    }

    #[actix_web::test]
    async fn u12_profile_only_actor_rejects_departure_before_commit_and_memory_actor_accepts() {
        use crate::persistence::player_profile_store::{PlayerProfilePersistence, PlayerProfileStore};
        use game_core::application::CoreApplication;
        use std::sync::Mutex;
        use game_core::game::data::content_pack::{self, ContentSelection, InputLimits};
        // This independent actor fixture uses current external official input;
        // it does not claim normal admission or enable embedded fixture APIs.
        let official_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources");
        let content_directory = tempfile::tempdir().unwrap();
        let limits = InputLimits::default();
        let selection = ContentSelection { packs: vec!["official".into()], winners: vec![] };
        let packaged = content_pack::package_official(&official_root,
            &content_directory.path().join("official"), &limits).unwrap();
        let game_data: ValidatedGameData = content_pack::load(content_directory.path(), &selection, &limits).unwrap();
        assert_eq!(game_data.identity(), &packaged.content_identity);
        assert_eq!(game_data.source_receipt(), &packaged.pack_receipt);
        println!("U12_PROFILE_ONLY_EXTERNAL_INPUT {}", json!({"official_source_root":official_root,
            "content_root":content_directory.path(),"selection":selection,"package_receipt":packaged,
            "runtime_safety_sha256":game_data.runtime_safety().effective_sha256(),"approval_claimed":false}));
        for durable_profile in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
                PlayerProfileStore::open(directory.path()).unwrap())));
            let lb = LoadBalanceActor::new().start();
            let core = CoreApplication::new(game_data.clone(), 12077);
            let mut actor = PlayerGameActor::new(PLAYER, core, lb);
            if durable_profile { actor = actor.with_player_profile_persistence(Arc::clone(&persistence)); }
            let actor = actor.start();
            let sink = Sink.start();
            let session = Uuid::new_v4();
            actor.send(AttachSession { session_id: session, socket: sink.clone().recipient(), control: sink.recipient() })
                .await.unwrap().unwrap();
            let start = command(&actor, session, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))
                .await.unwrap().unwrap();
            assert!(matches!(start.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            for _ in 0..128 {
                let run = actor.send(ReadRun).await.unwrap();
                if run["visual_novel"].is_null() { break; }
                let vn = &run["visual_novel"];
                let document = if let Some(choice) = vn["choices"].as_array().and_then(|choices| choices.iter().find(|choice| choice["selectable"] == true)) {
                    json!({"type":"select_visual_novel_choice","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"],"choice_id":choice["choice_id"]})
                } else {
                    json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]})
                };
                let result = command(&actor, session, document).await.unwrap().unwrap();
                assert!(matches!(result.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            }
            let selecting = actor.send(ReadRun).await.unwrap();
            assert_eq!(selecting["game_state"], "selecting_starter_employees");
            let ids: Vec<_> = selecting["game_state_context"]["candidates"].as_array().unwrap().iter()
                .take(3).map(|candidate| candidate["employee_definition_id"].clone()).collect();
            let before = actor.send(ReadLifecycle).await.unwrap();
            let files_before = file_bytes(directory.path()).unwrap();
            let profile_before = persistence.lock().unwrap().profile().unwrap();
            let result = command(&actor, session, json!({"type":"select_starter_employees",
                "employee_definition_ids":ids,"loan_copy_ids":[]})).await.unwrap();
            if durable_profile {
                assert!(result.as_ref().map_or(true, |result| matches!(&result.response, PlayerGameServerMessage::CommandResult { ok: false, .. })));
                assert_eq!(actor.send(ReadLifecycle).await.unwrap(), before);
                assert_eq!(file_bytes(directory.path()).unwrap(), files_before);
                assert_eq!(persistence.lock().unwrap().profile().unwrap(), profile_before);
            } else {
                assert!(matches!(result.unwrap().response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
                assert_ne!(actor.send(ReadLifecycle).await.unwrap(), before);
                let abandoned = command(&actor, session, json!({"type":"abandon_run"})).await.unwrap().unwrap();
                assert!(matches!(abandoned.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            }
            // Rejected departure remains at selection; use an independent title
            // actor for the profile-invariant tutorial positive control.
            let lb = LoadBalanceActor::new().start();
            let core = CoreApplication::new(game_data.clone(), 12077);
            let mut tutorial_actor = PlayerGameActor::new(PLAYER, core, lb);
            if durable_profile { tutorial_actor = tutorial_actor.with_player_profile_persistence(Arc::clone(&persistence)); }
            let actor = tutorial_actor.start();
            let sink = Sink.start();
            let session = Uuid::new_v4();
            actor.send(AttachSession { session_id: session, socket: sink.clone().recipient(), control: sink.recipient() })
                .await.unwrap().unwrap();
            let tutorial = command(&actor, session, json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":false}))
                .await.unwrap().unwrap();
            assert!(matches!(tutorial.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            let skipped = command(&actor, session, json!({"type":"skip_tutorial"})).await.unwrap().unwrap();
            assert!(matches!(skipped.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            if durable_profile { assert_eq!(persistence.lock().unwrap().profile().unwrap(), profile_before); }
        }
    }

    #[actix_web::test]
    #[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
    async fn u12_normal_title_profile_reset_cannot_orphan_saved_active_expedition() {
        let run = async {
            let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
            let out = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("output")?).join("title_profile_reset");
            fs::create_dir(&out)?;
            let admission = open(&config)?;
            let root = out.join("save");
            let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
            let (mut source, mut required, anchored) = new_application(&admission, &persistence)?;
            let mut revision = Some(anchored.revision);
            execute(&admission, &persistence, &mut source, &mut required, &mut revision,
                json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
            finish_start(&admission, &persistence, &mut source, &mut required, &mut revision, &out)?;
            let lb = LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host), persistence).start();
            let (actor, session, _sink) = actor(&lb, &admission).await?;
            let before = actor.send(ReadLifecycle).await?;
            let files = file_bytes(&root)?;
            let rejected = command(&actor, session, json!({"type":"reset_player_profile"})).await?;
            let rejected = match rejected {
                Err(error) => json!({"transport_error":error}),
                Ok(result) => {
                    assert!(matches!(&result.response, PlayerGameServerMessage::CommandResult { ok: false, .. }));
                    serde_json::to_value(result.response)?
                }
            };
            assert_eq!(actor.send(ReadLifecycle).await?, before);
            assert_eq!(file_bytes(&root)?, files);
            let continued = command(&actor, session, json!({"type":"continue_run"})).await?
                .map_err(|error| format!("same saved run still continues: {error}"))?;
            assert!(matches!(continued.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            save(&out.join("RESULT.json"), &json!({"passed":true,"actual_load_balancer":true,
                "rejected":rejected,"original_files_unchanged":true,"continued":actor.send(ReadLifecycle).await?}))?;
            Ok::<(), Box<dyn std::error::Error>>(())
        };
        run.await.unwrap();
    }

    #[actix_web::test]
    #[ignore = "subprocess-only helper; select the reset crash parent"]
    async fn u12_normal_corrupt_reset_crash_child() {
        use crate::persistence::player_profile_store::PlayerProfileStore;
        use game_core::game::{encounter_compendium::PlayerProfileState,
            expedition_lifecycle::VaultEquipment, data::equipment_data::EquipmentType};
        let run = async {
            let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
            let out = PathBuf::from(std::env::var_os("U12_CHILD_OUTPUT").ok_or("child output")?);
            let stage = std::env::var("U12_CHILD_CRASH_STAGE")?;
            let admission = open(&config)?;
            let root = out.join("save");
            fs::create_dir(&root)?;
            let mut fixture = PlayerProfileState::default();
            fixture.highest_floor_reached = 7;
            let id = Uuid::from_u128(0x12_5e8);
            fixture.lifecycle.vault.equipment.insert(id, VaultEquipment {
                weapon_stage: Default::default(),
                copy_uuid: id, definition_id: "dormant_crash_reset_weapon".into(),
                equipment_type: EquipmentType::Weapon, enhancement_level: 0,
            });
            { PlayerProfileStore::open(&root)?.publish(&fixture)?; }
            let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
            let (mut source, mut required, anchored) = new_application(&admission, &persistence)?;
            let mut revision = Some(anchored.revision);
            execute(&admission, &persistence, &mut source, &mut required, &mut revision,
                json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
            finish_start(&admission, &persistence, &mut source, &mut required, &mut revision, &out)?;
            let checkpoint = source.export_active_run_checkpoint()?.ok_or("genuine departure checkpoint")?;
            let expected = snapshot(&source, ApplicationSnapshotKind::PlayerProfile)?;
            let lb = LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host), persistence).start();
            let (actor, session, _sink) = actor(&lb, &admission).await?;
            fs::write(root.join("active_run.json"), b"{reset-crash-current-original")?;
            fs::write(root.join("active_run.backup.json"), b"{reset-crash-backup-original")?;
            save(&out.join("before_reset.json"), &json!({"profile":expected,
                "old_checkpoint":checkpoint.core_payload_json(),"loads_used":checkpoint.loads_used(),
                "run_peak_certified_power":checkpoint.run_peak_certified_power(),"files":file_bytes(&root)?}))?;
            std::env::set_var("U04_DURABLE_CRASH_AT", stage);
            let _ = command(&actor, session, json!({"type":"reset_active_run"})).await?;
            Err::<(), Box<dyn std::error::Error>>("accepted reset crash stage was not reached".into())
        };
        run.await.unwrap();
    }

    #[test]
    #[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
    fn u12_normal_corrupt_reset_crash_recovery_protects_exact_originals_and_next_departure() {
        use game_core::application::ApplicationCheckpoint;
        let run = || -> Result<()> {
            let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
            let out = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("output")?).join("reset_crash");
            fs::create_dir(&out)?;
            for (name, stage, tamper) in [
                ("accepted_intent", "intent_synced", false),
                ("profile_current", "profile_renamed", false),
                ("profile_backup", "profile_backup_synced", false),
                ("active_current", "active_renamed", false),
                ("active_backup", "active_backup_synced", false),
                ("changed_corrupt_original", "intent_synced", true),
            ] {
                let case = out.join(name); fs::create_dir(&case)?;
                let child = Command::new(std::env::current_exe()?).args(["--exact",
                    "mod_admission::u12_normal_host_tests::actor_management::u12_normal_corrupt_reset_crash_child",
                    "--ignored", "--nocapture", "--test-threads=1"])
                    .env("U12_NORMAL_CONFIG", &config).env("U12_CHILD_OUTPUT", &case)
                    .env("U12_CHILD_CRASH_STAGE", stage).env_remove("U04_DURABLE_CRASH_AT").output()?;
                fs::write(case.join("child.stdout"), &child.stdout)?;
                fs::write(case.join("child.stderr"), &child.stderr)?;
                assert_eq!(child.status.code(), Some(86), "actual reset stage {stage}");
                save(&case.join("child_exit.json"), &json!({"exit":child.status.code(),"stage":stage,"tamper":tamper}))?;
                let before: Value = serde_json::from_slice(&fs::read(case.join("before_reset.json"))?)?;
                let root = case.join("save");
                let admission = open(&config)?;
                assert!(root.join("application_commit.pending.json").is_file());
                if tamper {
                    fs::write(root.join("active_run.json"), b"{different-corrupt-original-after-acceptance")?;
                    let exact = file_bytes(&root)?;
                    for _ in 0..2 {
                        assert!(ApprovedPersistence::open(&root, Arc::clone(&admission.host)).is_err());
                        assert_eq!(file_bytes(&root)?, exact, "recovery cannot overwrite an original outside the accepted reset intent");
                    }
                    save(&case.join("RESULT.json"), &json!({"passed":true,"changed_original_protected":true}))?;
                    continue;
                }
                let mut expected = before["profile"].clone();
                expected["lifecycle"]["active_expedition"] = Value::Null;
                let mut first_recovery = None;
                for recovery in 0..2 {
                    let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
                    assert_eq!(serde_json::to_value(persistence.player_profile()?)?, expected);
                    assert!(persistence.snapshot()?.checkpoint.is_none());
                    let (app, _, _) = restore(&admission, &persistence)?;
                    assert!(app.is_not_started());
                    assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?, expected);
                    save(&case.join(format!("recovery_{recovery}.json")), &roots(&app)?)?;
                    drop(persistence);
                    let bytes = file_bytes(&root)?;
                    if let Some(first) = &first_recovery { assert_eq!(&bytes, first, "accepted reset replay is byte-idempotent"); }
                    else { first_recovery = Some(bytes); }
                }
                let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
                let (mut app, mut required, mut revision) = restore(&admission, &persistence)?;
                let old = ApplicationCheckpoint::new(serde_json::from_value(before["old_checkpoint"].clone())?,
                    serde_json::from_value(before["loads_used"].clone())?,
                    serde_json::from_value(before["run_peak_certified_power"].clone())?);
                let roots_before = roots(&app)?;
                assert!(app.continue_from_active_run_checkpoint(PLAYER, &old).is_err());
                assert_eq!(roots(&app)?, roots_before);
                execute(&admission, &persistence, &mut app, &mut required, &mut revision,
                    json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
                let next_out = case.join("next_departure"); fs::create_dir(&next_out)?;
                finish_start(&admission, &persistence, &mut app, &mut required, &mut revision, &next_out)?;
                let next = snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?;
                assert_eq!(next["lifecycle"]["expedition_sequence"].as_u64().unwrap(),
                    expected["lifecycle"]["expedition_sequence"].as_u64().unwrap() + 1);
                assert_ne!(next["lifecycle"]["active_expedition"], before["profile"]["lifecycle"]["active_expedition"]);
                assert_eq!(next["lifecycle"]["vault"], expected["lifecycle"]["vault"]);
                save(&case.join("RESULT.json"), &json!({"passed":true,"stage":stage,
                    "exact_reset_profile":expected,"old_continue_rejected":true,"next_departure":next}))?;
            }
            save(&out.join("RESULT.json"), &json!({"passed":true,"actual_crash_processes":6,
                "recovery_cases":5,"tamper_cases":1,"recoveries_per_case":2}))?;
            Ok(())
        };
        run().unwrap();
    }

    #[actix_web::test]
    #[ignore = "requires actual reviewed same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
    async fn u12_normal_admitted_corrupt_reset_is_atomic_and_preserves_vault_before_next_start() {
        use crate::persistence::player_profile_store::PlayerProfileStore;
        use game_core::game::{encounter_compendium::PlayerProfileState,
            expedition_lifecycle::VaultEquipment, data::equipment_data::EquipmentType};
        let run = async {
            let config = PathBuf::from(std::env::var_os("U12_NORMAL_CONFIG").ok_or("approved config")?);
            let out = PathBuf::from(std::env::var_os("U12_NORMAL_OUTPUT").ok_or("output")?).join("admitted_corrupt_reset");
            fs::create_dir(&out)?;
            let admission = open(&config)?;
            let root = out.join("save");
            fs::create_dir(&root)?;
            // Explicit durable boundary fixture, not a public acquisition claim.
            let mut fixture_profile = PlayerProfileState::default();
            let dormant = Uuid::from_u128(0x12_5e7);
            fixture_profile.lifecycle.vault.equipment.insert(dormant, VaultEquipment {
                weapon_stage: Default::default(),
                copy_uuid: dormant, definition_id: "dormant_reset_weapon".into(),
                equipment_type: EquipmentType::Weapon, enhancement_level: 0,
            });
            fixture_profile.highest_floor_reached = 7;
            {
                let store = PlayerProfileStore::open(&root)?;
                store.publish(&fixture_profile)?;
            }
            let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
            let (mut source, mut required, anchored) = new_application(&admission, &persistence)?;
            let mut revision = Some(anchored.revision);
            execute(&admission, &persistence, &mut source, &mut required, &mut revision,
                json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
            finish_start(&admission, &persistence, &mut source, &mut required, &mut revision, &out)?;
            let expected = snapshot(&source, ApplicationSnapshotKind::PlayerProfile)?;
            assert!(expected["lifecycle"]["active_expedition"].is_object());
            let lb = LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host), persistence).start();
            let (actor, session, _sink) = actor(&lb, &admission).await?;
            // The supported management case starts from already admitted owners.
            fs::write(root.join("active_run.json"), b"{corrupt-current")?;
            fs::write(root.join("active_run.backup.json"), b"{corrupt-backup")?;
            let before = actor.send(ReadLifecycle).await?;
            let files = file_bytes(&root)?;
            fs::create_dir(root.join("application_commit.pending.tmp"))?;
            let rejected = command(&actor, session, json!({"type":"reset_active_run"})).await?;
            assert!(rejected.as_ref().map_or(true, |result| matches!(&result.response, PlayerGameServerMessage::CommandResult { ok: false, .. })));
            assert_eq!(actor.send(ReadLifecycle).await?, before);
            assert_eq!(file_bytes(&root)?, files);
            fs::remove_dir(root.join("application_commit.pending.tmp"))?;
            let accepted = command(&actor, session, json!({"type":"reset_active_run"})).await?
                .map_err(|error| format!("reset retry: {error}"))?;
            assert!(matches!(accepted.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            let after = actor.send(ReadProfile).await?;
            let mut exact_after = expected.clone();
            exact_after["lifecycle"]["active_expedition"] = Value::Null;
            assert_eq!(after, exact_after, "reset loses only active expedition, preserving dormant vault and knowledge");
            let invalid_continue = command(&actor, session, json!({"type":"continue_run"})).await?;
            assert!(invalid_continue.as_ref().map_or(true, |result| matches!(&result.response, PlayerGameServerMessage::CommandResult { ok: false, .. })));
            let next = command(&actor, session, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false})).await?
                .map_err(|error| format!("new start after reset: {error}"))?;
            assert!(matches!(next.response, PlayerGameServerMessage::CommandResult { ok: true, .. }));
            assert_eq!(actor.send(ReadProfile).await?, exact_after, "briefing does not consume next expedition identity");
            save(&out.join("RESULT.json"), &json!({"passed":true,"scope":"already admitted owner only",
                "preaccept_unchanged":true,"profile_after_reset":after,"new_start_accepted":true}))?;
            Ok::<(), Box<dyn std::error::Error>>(())
        };
        run.await.unwrap();
    }
}
