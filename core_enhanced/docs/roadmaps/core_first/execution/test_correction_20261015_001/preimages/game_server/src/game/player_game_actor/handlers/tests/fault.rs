use super::*;
use crate::game::load_balance_actor::messages::{FindPlayer, GetOrCreatePlayerActor};
use game_core::application::ApplicationAdminCommand;

#[derive(actix::Message)]
#[rtype(result = "Value")]
struct FaultProbe {
    tick: bool,
}
impl Handler<FaultProbe> for PlayerGameActor {
    type Result = actix::MessageResult<FaultProbe>;
    fn handle(&mut self, msg: FaultProbe, ctx: &mut Context<Self>) -> Self::Result {
        if msg.tick {
            self.tick_live_battle(ctx);
        }
        actix::MessageResult(serde_json::json!({
            "snapshot": self.build_state_snapshot().unwrap(),
            "fault": self.game_core.battle_fault(),
            "tick_scheduled": self.live_battle_tick.is_some(),
            "ttl_scheduled": self.disconnect_timer.is_some(),
        }))
    }
}
fn fault_data() -> Arc<GameDataBase> {
    let raw = std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
        .unwrap()
        .replacen("max_work: 10000000", "max_work: 1000", 1);
    defense_game_data_with_profile(
        None,
        Arc::new(
            game_core::runtime_safety::RuntimeSafetySettings::from_ron(
                "fault-host",
                raw.as_bytes(),
            )
            .unwrap(),
        ),
    )
}
fn file_bytes(root: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        path: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().into(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut out = Default::default();
    visit(root, root, &mut out);
    out
}
async fn attach_fault_probe(
    actor: &actix::Addr<PlayerGameActor>,
    session: Uuid,
) -> AttachSessionResult {
    let (probe, _, _) = spawn_probe();
    actor
        .send(AttachSession {
            session_id: session,
            socket: probe.clone().recipient(),
            control: probe.recipient(),
        })
        .await
        .unwrap()
        .unwrap()
}
fn fault_command(
    session_id: Uuid,
    behavior: game_core::application::ApplicationCommand,
) -> ExecutePlayerBehavior {
    ExecutePlayerBehavior {
        session_id,
        request_id: "fault-check".into(),
        behavior,
        battle_authority: None,
        battle_side_message: BattleSideMessageKind::BattleUpdate,
    }
}

#[actix_web::test]
async fn real_tick_fault_blocks_late_ticks_management_reconnect_and_stop_writes_then_existing_continue(
) {
    let dir = tempfile::tempdir().unwrap();
    let active = Arc::new(Mutex::new(
        ActiveRunStore::open(dir.path().join("active")).unwrap(),
    ));
    let mut profile = PlayerProfilePersistence::bootstrap(
        PlayerProfileStore::open(dir.path().join("profile")).unwrap(),
    );
    let initial_profile = game_core::game::encounter_compendium::PlayerProfileState {
        highest_floor_reached: 2,
        ..Default::default()
    };
    profile.merge_and_persist(initial_profile);
    let profile = Arc::new(Mutex::new(profile));
    let data = fault_data();
    let player = Uuid::from_u128(0xf003);
    let mut core = game_core_waiting_on_live_defense_node_with_data(player, data.clone());
    // Capture a genuine save-point checkpoint through the existing public admin port.
    let mut saved = game_core_waiting_on_live_defense_node_with_data(player, data.clone());
    saved
        .execute_fixture(player, command!(CancelSelectedNode))
        .unwrap();
    saved
        .execute_admin(
            ApplicationAdminCommand::from_document(
                serde_json::json!({"type":"admin_enter_save_point"}).into(),
            )
            .unwrap(),
        )
        .unwrap();
    saved
        .execute_fixture(player, command!(CompleteNode))
        .unwrap();
    let checkpoint = saved
        .export_active_run_checkpoint()
        .unwrap()
        .expect("save point creates checkpoint");
    active
        .lock()
        .unwrap()
        .publish(&StoredCheckpoint::from(&checkpoint))
        .unwrap();
    core.prepare_non_battle_command(player, command!(ConfirmEnterNode), Default::default())
        .unwrap()
        .commit();
    // An unpersisted monotonic value distinguishes a forbidden stopped writer.
    core.merge_player_profile(
        serde_json::to_value(game_core::game::encounter_compendium::PlayerProfileState {
            highest_floor_reached: 99,
            ..Default::default()
        })
        .unwrap()
        .into(),
    )
    .unwrap();
    let balance = LoadBalanceActor::new().start();
    let actor = PlayerGameActor::new(player, core, balance.clone())
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile.clone())
        .with_disconnect_ttl(Duration::from_millis(10))
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    balance
        .send(Register {
            player_id: player,
            addr: actor.clone(),
        })
        .await
        .unwrap();
    let first_session = Uuid::from_u128(1);
    attach_fault_probe(&actor, first_session).await;
    actor.send(EnsureLiveBattleTick).await.unwrap();
    assert_eq!(
        actor.send(FaultProbe { tick: false }).await.unwrap()["tick_scheduled"],
        true
    );
    let before = file_bytes(dir.path());
    let failed = actor.send(FaultProbe { tick: true }).await.unwrap();
    assert_eq!(failed["fault"]["cause"]["fault"]["code"], "work_limit");
    assert_eq!(failed["tick_scheduled"], false);
    assert_eq!(failed["snapshot"]["game_state"], "battle_faulted");
    let first_fault = failed["fault"].clone();
    for _ in 0..4 {
        actor.send(EnsureLiveBattleTick).await.unwrap();
        let repeated = actor.send(FaultProbe { tick: true }).await.unwrap();
        assert_eq!(repeated, failed);
        for behavior in [
            command!(PauseBattle),
            command!(ResumeBattle),
            command!(RequestBattleState),
            command!(ContinueRun),
            command!(ResetActiveRun),
            command!(RetryPlayerProfile),
            command!(ResetPlayerProfile),
            command!(StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: true
            }),
        ] {
            assert_eq!(
                actor
                    .send(fault_command(first_session, behavior))
                    .await
                    .unwrap()
                    .unwrap_err()
                    .code,
                "battle_faulted"
            );
        }
        let admin = ApplicationAdminCommand::from_document(
            serde_json::json!({"type":"admin_set_enkephalin","amount":1}).into(),
        )
        .unwrap();
        assert_eq!(
            actor
                .send(ExecuteAdminCommand {
                    session_id: first_session,
                    request_id: "admin".into(),
                    command: admin
                })
                .await
                .unwrap()
                .unwrap_err()
                .code,
            "battle_faulted"
        );
    }
    let second_session = Uuid::from_u128(2);
    let replaced = attach_fault_probe(&actor, second_session).await;
    assert_eq!(replaced.state_snapshot["battle_fault"], first_fault);
    assert!(replaced.side_messages.is_empty());
    assert_eq!(
        actor
            .send(fault_command(first_session, command!(PauseBattle)))
            .await
            .unwrap()
            .unwrap_err()
            .code,
        "stale_session"
    );
    actor
        .send(DetachSession {
            session_id: second_session,
        })
        .await
        .unwrap();
    time::sleep(Duration::from_millis(30)).await;
    assert_eq!(
        balance
            .send(FindPlayer { player_id: player })
            .await
            .unwrap(),
        Some(actor.clone())
    );
    let detached = actor.send(FaultProbe { tick: false }).await.unwrap();
    assert_eq!(detached["ttl_scheduled"], false);
    assert_eq!(detached["fault"], first_fault);
    let third_session = Uuid::from_u128(3);
    assert_eq!(
        attach_fault_probe(&actor, third_session)
            .await
            .state_snapshot["battle_fault"],
        first_fault
    );
    actor.send(QuitPlayerActor).await.unwrap();
    for _ in 0..20 {
        if balance
            .send(FindPlayer { player_id: player })
            .await
            .unwrap()
            .is_none()
        {
            break;
        }
        time::sleep(Duration::from_millis(2)).await;
    }
    assert_eq!(file_bytes(dir.path()), before);
    // Reuse the same real store owners for the production new-actor factory.
    drop(actor);
    for _ in 0..20 {
        if Arc::strong_count(&active) == 1 && Arc::strong_count(&profile) == 1 {
            break;
        }
        time::sleep(Duration::from_millis(2)).await;
    }
    let balance = LoadBalanceActor::with_persistence(
        Arc::try_unwrap(active)
            .ok()
            .expect("old actor released active store")
            .into_inner()
            .unwrap(),
        Arc::try_unwrap(profile)
            .ok()
            .expect("old actor released profile store")
            .into_inner()
            .unwrap(),
    )
    .start();
    let fresh = balance
        .send(GetOrCreatePlayerActor {
            player_id: player,
            game_data: data.into(),
            run_seed: 23,
        })
        .await
        .unwrap();
    let new_session = Uuid::from_u128(4);
    assert!(attach_fault_probe(&fresh, new_session).await.state_snapshot["battle_fault"].is_null());
    let resumed = fresh
        .send(fault_command(new_session, command!(ContinueRun)))
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        resumed.response,
        PlayerGameServerMessage::CommandResult { ok: true, .. }
    ));
    assert_eq!(
        resumed.state_snapshot.as_ref().unwrap()["run_checkpoint"]["loads_used"],
        checkpoint.loads_used()
    );
    assert!(fresh.send(FaultProbe { tick: false }).await.unwrap()["fault"].is_null());
}

#[actix_web::test]
async fn fault_is_per_actor_and_other_player_ticks_commands_and_real_saves_continue() {
    let dir = tempfile::tempdir().unwrap();
    let active = Arc::new(Mutex::new(
        ActiveRunStore::open(dir.path().join("active")).unwrap(),
    ));
    let profile = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
        PlayerProfileStore::open(dir.path().join("profile")).unwrap(),
    )));
    let balance = LoadBalanceActor::new().start();
    let player_a = Uuid::from_u128(0xf031);
    let mut a = game_core_waiting_on_live_defense_node_with_data(player_a, fault_data());
    a.execute_fixture(player_a, command!(ConfirmEnterNode))
        .unwrap();
    assert_eq!(
        a.advance_active_battle_by(60_000, 0).unwrap_err().code(),
        "battle_runtime_work_limit"
    );
    let actor_a = PlayerGameActor::new(player_a, a, balance.clone())
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile.clone())
        .start();
    let player_b = Uuid::from_u128(0xf032);
    let mut b = game_core_waiting_on_live_defense_node(player_b);
    b.execute_fixture(player_b, command!(ConfirmEnterNode))
        .unwrap();
    let actor_b = PlayerGameActor::new(player_b, b, balance)
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile)
        .with_live_battle_tick_interval(Duration::from_millis(1))
        .start();
    let session_a = Uuid::from_u128(0xf033);
    let session_b = Uuid::from_u128(0xf034);
    let fault = attach_fault_probe(&actor_a, session_a).await.state_snapshot;
    let before_b = attach_fault_probe(&actor_b, session_b).await.state_snapshot;
    let stepped_b = actor_b.send(FaultProbe { tick: true }).await.unwrap();
    assert!(stepped_b["fault"].is_null());
    assert!(
        stepped_b["snapshot"]["game_state_context"]["battle_state_revision"]
            .as_u64()
            .unwrap()
            > before_b["game_state_context"]["battle_state_revision"]
                .as_u64()
                .unwrap()
    );
    let context = &stepped_b["snapshot"]["game_state_context"];
    let mut retreat = fault_command(session_b, command!(RetreatBattle));
    retreat.battle_authority = Some(game_core::game::behavior::BattleStateAuthorityDto {
        battle_uuid: serde_json::from_value(context["battle_uuid"].clone()).unwrap(),
        battle_state_revision: context["battle_state_revision"].as_u64().unwrap(),
    });
    actor_b.send(retreat).await.unwrap().unwrap();
    let before_save = file_bytes(dir.path());
    actor_b
        .send(ExecuteAdminCommand {
            session_id: session_b,
            request_id: "savepoint".into(),
            command: ApplicationAdminCommand::from_document(
                serde_json::json!({"type":"admin_enter_save_point"}).into(),
            )
            .unwrap(),
        })
        .await
        .unwrap()
        .unwrap();
    actor_b
        .send(fault_command(session_b, command!(CompleteNode)))
        .await
        .unwrap()
        .unwrap();
    let after_save = file_bytes(dir.path());
    assert_ne!(before_save, after_save);
    assert!(matches!(
        active.lock().unwrap().load().unwrap(),
        ActiveRunLoad::Loaded { .. }
    ));
    assert_eq!(
        actor_a.send(FaultProbe { tick: true }).await.unwrap()["snapshot"],
        fault
    );
    assert_eq!(
        actor_a
            .send(fault_command(session_a, command!(ContinueRun)))
            .await
            .unwrap()
            .unwrap_err()
            .code,
        "battle_faulted"
    );
    actor_a.send(QuitPlayerActor).await.unwrap();
    time::sleep(Duration::from_millis(10)).await;
    assert_eq!(after_save, file_bytes(dir.path()));
}

#[actix_web::test]
async fn new_owner_continue_keeps_existing_missing_and_invalid_save_refusals() {
    for invalid in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(dir.path()).unwrap();
        if invalid {
            store
                .publish(&StoredCheckpoint {
                    core_payload_json: br#"{"version":12,"invalid_core_checkpoint":true}"#.to_vec(),
                    loads_used: 2,
                    run_peak_certified_power: 1,
                })
                .unwrap();
        }
        let before = file_bytes(dir.path());
        let player = Uuid::from_u128(0xf041);
        let balance = LoadBalanceActor::with_active_run_store(store).start();
        let actor = balance
            .send(GetOrCreatePlayerActor {
                player_id: player,
                game_data: empty_game_data().into(),
                run_seed: 41,
            })
            .await
            .unwrap();
        let session = Uuid::from_u128(0xf042);
        attach_fault_probe(&actor, session).await;
        let error = actor
            .send(fault_command(session, command!(ContinueRun)))
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(
            error.code,
            if invalid {
                "invalid_static_data"
            } else {
                "active_run_unavailable"
            }
        );
        assert!(actor.send(FaultProbe { tick: false }).await.unwrap()["fault"].is_null());
        assert_eq!(before, file_bytes(dir.path()));
    }
}

#[actix_web::test]
async fn real_manual_retreat_failure_publishes_fault_and_cancels_the_scheduled_timer() {
    // Entry uses zero native clone slots; the first Retreat clones 85. A legal
    // immutable 84-slot profile isolates a real manual precommit failure.
    let player = Uuid::from_u128(0xf051);
    let raw = std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
        .unwrap()
        .replacen("max_clone_slots: 1048576", "max_clone_slots: 84", 1);
    let data = defense_game_data_with_profile(
        None,
        Arc::new(
            game_core::runtime_safety::RuntimeSafetySettings::from_ron(
                "manual-fault",
                raw.as_bytes(),
            )
            .unwrap(),
        ),
    );
    let mut core = game_core_waiting_on_live_defense_node_with_data(player, data);
    core.execute_fixture(player, command!(ConfirmEnterNode))
        .unwrap();
    let balance = LoadBalanceActor::new().start();
    let actor = PlayerGameActor::new(player, core, balance)
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    let session = Uuid::from_u128(0xf052);
    let attached = attach_fault_probe(&actor, session).await;
    actor.send(EnsureLiveBattleTick).await.unwrap();
    assert_eq!(
        actor.send(FaultProbe { tick: false }).await.unwrap()["tick_scheduled"],
        true
    );
    let context = &attached.state_snapshot["game_state_context"];
    let mut retreat = fault_command(session, command!(RetreatBattle));
    retreat.battle_authority = Some(game_core::game::behavior::BattleStateAuthorityDto {
        battle_uuid: serde_json::from_value(context["battle_uuid"].clone()).unwrap(),
        battle_state_revision: context["battle_state_revision"].as_u64().unwrap(),
    });
    let result = actor.send(retreat).await.unwrap();
    let error = result
        .err()
        .expect("manual command must reach the actual native limit");
    assert_eq!(error.code, "battle_runtime_clone_limit");
    let after = actor.send(FaultProbe { tick: false }).await.unwrap();
    assert_eq!(after["fault"]["operation"]["command"], "retreat_battle");
    assert_eq!(after["tick_scheduled"], false);
    actor.send(EnsureLiveBattleTick).await.unwrap();
    assert_eq!(actor.send(FaultProbe { tick: true }).await.unwrap(), after);
}
