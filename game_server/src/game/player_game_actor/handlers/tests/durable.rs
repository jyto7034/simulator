use super::*;

async fn u08_host_relic_state(actor: &actix::Addr<PlayerGameActor>, session: Uuid) -> Value {
    let result = actor
        .send(command_for(session, command!(RequestRelicState)))
        .await
        .unwrap()
        .unwrap();
    let PlayerGameServerMessage::CommandResult {
        ok: true,
        result_type,
        payload,
        ..
    } = result.response
    else {
        panic!("relic query result")
    };
    assert_eq!(result_type, "RelicState");
    payload
}

#[actix_web::test]
async fn u08_nonempty_relics_and_current_live_battle_survive_real_detach_attach_without_continue() {
    use game_core::{
        application::ApplicationAdminCommand,
        game::{
            battle::{damage::DamageType, tile_range::TileRangePattern},
            data::{relic_data::RelicDatabase, ValidatedGameData},
        },
    };
    let base = defense_game_data_with_survive_timer(Some(60_000));
    let mut relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core_enhanced/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    let template = relics.relics[0].clone();
    for index in 1..=2 {
        let mut definition = template.clone();
        definition.id = format!("u08_host_{index}");
        definition.uuid = Uuid::from_u128(0x808_500 + index);
        relics.relics.push(definition);
    }
    let mut rows = vec!["XXXXXXXXXXXXXXX"; 15];
    rows[7] = "XXXXXXX@XXXXXXX";
    let range: TileRangePattern =
        serde_json::from_value(serde_json::json!({"include_anchor_tile":false,"rows":rows}))
            .unwrap();
    let mut equipment = (*base.equipment_data).clone();
    let weapon = equipment
        .items
        .iter_mut()
        .find(|item| item.id == "standard_training_sword")
        .unwrap()
        .weapon_profile
        .as_mut()
        .unwrap();
    weapon.damage_type = DamageType::True;
    weapon.range_role = game_core::game::data::equipment_data::WeaponRangeRole::Ranged;
    weapon.defense_tile_range = range;
    weapon.interval_ms = 1_000;
    weapon.windup_ms = 10;
    let data = GameDataBuilder::from_validated(&ValidatedGameData::from(base))
        .with_relics(relics)
        .with_equipment_data(Arc::new(equipment))
        .build_arc();
    let player = Uuid::from_u128(0x808_501);
    let mut app = game_core_waiting_on_live_defense_node_with_data(player, data);
    let snapshot: RunSnapshotDto = serde_json::from_value(
        app.snapshot(ApplicationSnapshotKind::Run)
            .unwrap()
            .read_model()
            .as_value()
            .clone(),
    )
    .unwrap();
    let employee = snapshot.roster.combat_eligible_employee_ids[0];
    // Internal fixture acquisition only. All equip/corrupt/battle/session
    // operations below use their ordinary application or host commands.
    for id in ["u08_probe", "u08_host_1", "u08_host_2"] {
        app.execute_admin(
            ApplicationAdminCommand::from_document(
                serde_json::json!({"type":"admin_grant_relic","definition_id":id,"count":1}).into(),
            )
            .unwrap(),
        )
        .unwrap();
    }
    app.execute_admin(
        ApplicationAdminCommand::from_document(
            serde_json::json!({"type":"admin_grant_relic_corruption_catalyst","amount":3}).into(),
        )
        .unwrap(),
    )
    .unwrap();
    let query = app
        .execute_fixture(player, command!(RequestRelicState))
        .unwrap();
    let state: game_core::game::relic::RelicStateDto =
        serde_json::from_value(query.payload().as_value().clone()).unwrap();
    assert_eq!(state.copies.len(), 3);
    for (slot, copy) in state.copies.iter().enumerate() {
        app.execute_fixture(
            player,
            command!(EquipRelic {
                copy_uuid: copy.copy_uuid,
                employee_uuid: employee,
                slot: slot as u8
            }),
        )
        .unwrap();
    }
    app.execute_fixture(
        player,
        command!(CorruptRelic {
            copy_uuid: state.copies[0].copy_uuid
        }),
    )
    .unwrap();
    let entered = app
        .execute_fixture(player, command!(ConfirmEnterNode))
        .unwrap();
    let setup: game_core::game::behavior::LiveBattleSetupSnapshotDto =
        serde_json::from_value(entered.payload().as_value()["battle_setup_snapshot"].clone())
            .unwrap();
    let position = setup
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == game_core::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells[0];
    let initial = app.active_battle_reconnect(0).unwrap();
    let update = &initial.payload().as_value()["battle_update"];
    let deployed = app
        .execute(
            player,
            command!(DeployUnit {
                employee_uuid: employee,
                position,
                facing: game_core::game::battle::tile_range::FacingDirection::Right
            }),
            ApplicationExecutionContext {
                expected_battle_authority: Some(ApplicationBattleAuthority {
                    battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(),
                    battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let unit: game_core::game::battle::ids::UnitInstanceId =
        serde_json::from_value(deployed.payload().as_value()["unit_instance_id"].clone()).unwrap();
    let mut actual_damage = None;
    for _ in 0..20 {
        app.advance_active_battle_by(1_000, 0).unwrap();
        actual_damage = app
            .battle_runtime_facts()
            .unwrap()
            .unwrap()
            .actors()
            .find(|(actor, _)| **actor == unit)
            .and_then(|(_, summary)| summary.caused.latest_damage.clone())
            .filter(|hit| hit.actual_hp_loss() > 0);
        if actual_damage.is_some() {
            break;
        }
    }
    let actual_damage =
        actual_damage.expect("real deployed actor must consume damage before reconnect");
    assert!(actual_damage.time_ms > 0);
    assert!(actual_damage.hp_after < actual_damage.hp_before);
    let actor = PlayerGameActor::new(player, app, LoadBalanceActor::new().start())
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    let first = Uuid::from_u128(0x808_502);
    attach(&actor, first).await;
    let relic_before = u08_host_relic_state(&actor, first).await;
    assert_eq!(relic_before["resources"]["relic_corruption_catalysts"], 2);
    let before = actor.send(U05LiveProbe { advance_ms: 0 }).await.unwrap();
    let durable_before = actor.send(DurableProbe { tick: false }).await.unwrap();
    assert_eq!(before["update"]["checkpoint"]["playback"]["paused"], false);
    assert!(before["update"]["server_battle_time_ms"].as_u64().unwrap() > 0);
    let live = before["update"]["checkpoint"]["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["unit_instance_id"] == serde_json::to_value(unit).unwrap())
        .unwrap();
    assert!(live["stats"]["current_health"].as_u64().unwrap() > 0);
    actor
        .send(DetachSession { session_id: first })
        .await
        .unwrap();
    let detached = actor
        .send(U05LiveProbe { advance_ms: 5_000 })
        .await
        .unwrap();
    assert_eq!(
        detached["update"]["checkpoint"]["playback"]["paused"], true,
        "real DetachSession must auto-pause"
    );
    let second = Uuid::from_u128(0x808_503);
    let attached = attach(&actor, second).await;
    assert_eq!(attached.side_messages.len(), 2);
    let after = actor
        .send(U05LiveProbe { advance_ms: 5_000 })
        .await
        .unwrap();
    assert_eq!(
        after["update"]["battle_uuid"],
        before["update"]["battle_uuid"]
    );
    assert_eq!(
        after["update"]["server_battle_time_ms"],
        before["update"]["server_battle_time_ms"]
    );
    assert_eq!(
        after["update"]["checkpoint"]["units"],
        before["update"]["checkpoint"]["units"]
    );
    assert_eq!(
        after["update"]["checkpoint"]["deployment"],
        before["update"]["checkpoint"]["deployment"]
    );
    assert_eq!(u08_host_relic_state(&actor, second).await, relic_before);
    let durable_after = actor.send(DurableProbe { tick: false }).await.unwrap();
    assert_eq!(durable_after["checkpoint"], durable_before["checkpoint"]);
    assert_eq!(
        durable_after["snapshot"]["game_state"],
        durable_before["snapshot"]["game_state"]
    );
    assert!(durable_after["fault"].is_null());
    let mut resume = command_for(second, command!(ResumeBattle));
    resume.battle_authority = Some(serde_json::from_value(serde_json::json!({"battle_uuid":after["update"]["battle_uuid"],"battle_state_revision":after["update"]["battle_state_revision"]})).unwrap());
    actor.send(resume).await.unwrap().unwrap();
    let resumed = actor.send(U05LiveProbe { advance_ms: 100 }).await.unwrap();
    assert!(
        resumed["update"]["server_battle_time_ms"].as_u64().unwrap()
            > after["update"]["server_battle_time_ms"].as_u64().unwrap()
    );
    assert_eq!(
        resumed["update"]["battle_uuid"],
        before["update"]["battle_uuid"]
    );
    assert_eq!(u08_host_relic_state(&actor, second).await, relic_before);
    println!(
        "U08_HOST_RECONNECT actor={unit:?} damage={} retained_time={} resumed_time={}",
        actual_damage.actual_hp_loss(),
        after["update"]["server_battle_time_ms"],
        resumed["update"]["server_battle_time_ms"]
    );
}

#[derive(actix::Message)]
#[rtype(result = "Value")]
struct DurableProbe {
    tick: bool,
}
impl Handler<DurableProbe> for PlayerGameActor {
    type Result = actix::MessageResult<DurableProbe>;
    fn handle(&mut self, message: DurableProbe, ctx: &mut Context<Self>) -> Self::Result {
        if message.tick {
            self.tick_live_battle(ctx);
        }
        actix::MessageResult(serde_json::json!({
            "snapshot": self.build_state_snapshot().unwrap(),
            "checkpoint": self.game_core.export_active_run_checkpoint().unwrap().map(|checkpoint| checkpoint.core_payload_json().to_vec()),
            "fault": self.game_core.battle_fault(),
        }))
    }
}
fn command_for(
    session_id: Uuid,
    behavior: game_core::application::ApplicationCommand,
) -> ExecutePlayerBehavior {
    ExecutePlayerBehavior {
        session_id,
        request_id: "durable-test".into(),
        behavior,
        battle_authority: None,
        battle_side_message: BattleSideMessageKind::BattleUpdate,
    }
}
async fn attach(actor: &actix::Addr<PlayerGameActor>, session: Uuid) -> AttachSessionResult {
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

#[actix_web::test]
async fn u04_protected_profile_exposes_retry_and_refuses_forced_reset_without_writing_originals() {
    for filename in [
        "encounter_compendium.json",
        "standard_completion.pending.json",
        "player_profile.json",
    ] {
        let root = tempfile::tempdir().unwrap();
        let original = br#"{"version":99,"original":true}"#;
        std::fs::write(root.path().join(filename), original).unwrap();
        let profile = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(root.path()).unwrap(),
        )));
        let player = Uuid::new_v4();
        let actor = PlayerGameActor::new(
            player,
            CoreApplication::new(empty_game_data(), 0),
            LoadBalanceActor::new().start(),
        )
        .with_player_profile_persistence(profile.clone())
        .start();
        let session = Uuid::new_v4();
        let state = attach(&actor, session).await.state_snapshot;
        assert_eq!(state["player_profile_persistence"]["state"], "protected");
        assert_eq!(state["player_profile_persistence"]["reset_required"], false);
        let actions = state["action_capabilities"].as_array().unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0]["action"], "RetryPlayerProfile");
        for command in [
            command!(ResetPlayerProfile),
            command!(RetryPlayerProfile),
            command!(StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: true
            }),
        ] {
            assert!(actor
                .send(command_for(session, command))
                .await
                .unwrap()
                .is_err());
            assert_eq!(std::fs::read(root.path().join(filename)).unwrap(), original);
            assert!(!root.path().join("player_profile.backup.json").exists());
            assert!(!root.path().join("application_commit.pending.json").exists());
        }
        assert_eq!(
            actor.send(DurableProbe { tick: false }).await.unwrap()["snapshot"],
            state
        );
    }
}

#[actix_web::test]
async fn u04_newer_profile_blocks_stale_restored_actor_terminal_disposition() {
    let root = tempfile::tempdir().unwrap();
    let active = Arc::new(Mutex::new(ActiveRunStore::open(root.path()).unwrap()));
    let player = Uuid::new_v4();
    let initial = game_core_waiting_on_live_defense_node(player);
    active
        .lock()
        .unwrap()
        .publish(&StoredCheckpoint::from(
            &initial.export_active_run_checkpoint().unwrap().unwrap(),
        ))
        .unwrap();
    let profile = persisted_profile_fixture(root.path(), &initial);
    let balance = LoadBalanceActor::new().start();
    let make_actor = || {
        PlayerGameActor::new(
            player,
            CoreApplication::new(defense_game_data(), 0),
            balance.clone(),
        )
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile.clone())
        .start()
    };
    let a = make_actor();
    let b = make_actor();
    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();
    attach(&a, session_a).await;
    attach(&b, session_b).await;
    a.send(command_for(session_a, command!(ContinueRun)))
        .await
        .unwrap()
        .unwrap();
    b.send(command_for(session_b, command!(ContinueRun)))
        .await
        .unwrap()
        .unwrap();
    let before_b = b.send(DurableProbe { tick: false }).await.unwrap();
    a.send(command_for(session_a, command!(AbandonRun)))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        active.lock().unwrap().load().unwrap(),
        ActiveRunLoad::Missing
    );
    let confirmed = std::fs::read(root.path().join("active_run.json")).unwrap();
    let error = b
        .send(command_for(session_b, command!(AbandonRun)))
        .await
        .unwrap()
        .unwrap_err();
    // Abandon clears the authoritative expedition identity. The stale actor
    // must reject that profile before preparing another terminal publication.
    // Store-level revision conflicts are covered by the conditional-intent test.
    assert!(profile.lock().unwrap().profile().unwrap().lifecycle.active_expedition.is_none());
    assert_eq!(error.code, "invalid_action");
    assert_eq!(
        std::fs::read(root.path().join("active_run.json")).unwrap(),
        confirmed
    );
    assert_eq!(
        b.send(DurableProbe { tick: false }).await.unwrap(),
        before_b
    );
    assert!(!root.path().join("application_commit.pending.json").exists());
}

#[actix_web::test]
async fn u04_real_terminal_host_tick_preintent_failure_does_not_publish_end_then_retry_settles_once(
) {
    let root = tempfile::tempdir().unwrap();
    let active = Arc::new(Mutex::new(ActiveRunStore::open(root.path()).unwrap()));
    let player = Uuid::new_v4();
    let core = game_core_waiting_on_live_defense_node_with_data(
        player,
        defense_game_data_with_survive_timer(Some(10)),
    );
    active
        .lock()
        .unwrap()
        .publish(&StoredCheckpoint::from(
            &core.export_active_run_checkpoint().unwrap().unwrap(),
        ))
        .unwrap();
    let profile = persisted_profile_fixture(root.path(), &core);
    let actor = PlayerGameActor::new(player, core, LoadBalanceActor::new().start())
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile.clone())
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    let session = Uuid::new_v4();
    let (probe, _, messages) = spawn_probe();
    actor
        .send(AttachSession {
            session_id: session,
            socket: probe.clone().recipient(),
            control: probe.recipient(),
        })
        .await
        .unwrap()
        .unwrap();
    actor
        .send(command_for(session, command!(ConfirmEnterNode)))
        .await
        .unwrap()
        .unwrap();
    let before = actor.send(DurableProbe { tick: false }).await.unwrap();
    let before_file = std::fs::read(root.path().join("active_run.json")).unwrap();
    time::sleep(Duration::from_millis(5)).await;
    messages.lock().unwrap().clear();
    std::fs::create_dir(root.path().join("application_commit.pending.tmp")).unwrap();
    for _ in 0..2 {
        assert_eq!(
            actor.send(DurableProbe { tick: true }).await.unwrap(),
            before
        );
        assert_eq!(
            std::fs::read(root.path().join("active_run.json")).unwrap(),
            before_file
        );
        assert!(!root.path().join("application_commit.pending.json").exists());
    }
    time::sleep(Duration::from_millis(5)).await;
    assert!(messages
        .lock()
        .unwrap()
        .iter()
        .all(|message| !matches!(message, PlayerGameServerMessage::BattleUpdate(_))));
    std::fs::remove_dir(root.path().join("application_commit.pending.tmp")).unwrap();
    let settled = actor.send(DurableProbe { tick: true }).await.unwrap();
    assert_eq!(settled["snapshot"]["game_state"], "combat_result");
    assert!(settled["fault"].is_null());
    let confirmed = std::fs::read(root.path().join("active_run.json")).unwrap();
    assert_ne!(confirmed, before_file);
    assert_eq!(
        std::fs::read(root.path().join("active_run.backup.json")).unwrap(),
        confirmed
    );
    assert_eq!(
        actor.send(DurableProbe { tick: true }).await.unwrap(),
        settled
    );
    let checkpoint = active
        .lock()
        .unwrap()
        .snapshot()
        .unwrap()
        .checkpoint
        .unwrap();
    let mut restarted = CoreApplication::new(defense_game_data_with_survive_timer(Some(10)), 0);
    restarted.install_authoritative_player_profile(
        serde_json::to_value(profile.lock().unwrap().profile().unwrap()).unwrap().into()
    ).unwrap();
    restarted
        .continue_from_active_run_checkpoint(player, &checkpoint.to_application())
        .unwrap();
    assert_eq!(restarted.run_snapshot_fixture().game_state, "combat_result");
    actor
        .send(command_for(session, command!(CompleteCombatResult)))
        .await
        .unwrap()
        .unwrap();
    assert!(actor
        .send(command_for(session, command!(CompleteCombatResult)))
        .await
        .unwrap()
        .is_err());
}

#[cfg(feature = "runtime-observation")]
#[test]
fn u04_prepared_terminal_tick_observes_discard_and_commit_without_premature_end() {
    use game_core::runtime_observation::{self as obs, Outcome};
    let player = Uuid::from_u128(0xC004_F015);
    let data = defense_game_data_with_survive_timer(Some(10));
    let mut app = game_core_waiting_on_live_defense_node_with_data(player, data.clone());
    let mut baseline = game_core_waiting_on_live_defense_node_with_data(player, data);
    let (sender, receiver) = obs::channel();
    let (observer, reader) = sender.session(15);
    app.set_runtime_observer(Some(observer));
    let entered = app
        .prepare_command(player, command!(ConfirmEnterNode), Default::default())
        .unwrap()
        .commit();
    assert_eq!(
        entered,
        baseline
            .execute_fixture(player, command!(ConfirmEnterNode))
            .unwrap()
    );
    assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Accepted);
    let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let live = reader.snapshot(false).unwrap();
    assert_eq!(live.commits, 1);
    assert!(live.current.battle.is_some());
    let prepared = app.prepare_active_battle_for_host_tick(50, 0).unwrap();
    assert!(
        prepared.result().as_ref().unwrap().payload().as_value()["finished"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(reader.snapshot(false).unwrap().current, live.current);
    drop(prepared);
    assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Discarded);
    assert_eq!(reader.snapshot(false).unwrap().current, live.current);
    assert_eq!(reader.snapshot(false).unwrap().commits, 1);
    assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
    let expected = baseline.advance_active_battle_for_host_tick(50, 0).unwrap();
    let committed = app
        .prepare_active_battle_for_host_tick(50, 0)
        .unwrap()
        .commit();
    assert_eq!(committed, expected);
    let accepted = receiver.try_recv().unwrap();
    assert_eq!(accepted.outcome, Outcome::Accepted);
    assert!(accepted.work > 0 && accepted.admitted_bytes > 0);
    assert_eq!(reader.snapshot(false).unwrap().commits, 2);
    assert!(reader.snapshot(false).unwrap().current.battle.is_none());
    assert_eq!(
        app.snapshot(ApplicationSnapshotKind::Run).unwrap(),
        baseline.snapshot(ApplicationSnapshotKind::Run).unwrap()
    );
    assert_eq!(
        app.export_active_run_checkpoint().unwrap(),
        baseline.export_active_run_checkpoint().unwrap()
    );
}

#[actix_web::test]
async fn u04_admin_prepared_drop_and_io_failures_share_application_acceptance() {
    use game_core::application::ApplicationAdminCommand;
    let admin = |amount| {
        ApplicationAdminCommand::from_document(
            serde_json::json!({"type":"admin_set_enkephalin", "amount":amount}).into(),
        )
        .unwrap()
    };
    let root = tempfile::tempdir().unwrap();
    let player = Uuid::new_v4();
    let mut core = game_core_waiting_on_live_defense_node(player);
    let before = core.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let checkpoint = core.export_active_run_checkpoint().unwrap().unwrap();
    drop(core.prepare_admin_command(admin(777)).unwrap());
    assert_eq!(core.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
    assert_eq!(
        core.export_active_run_checkpoint().unwrap().unwrap(),
        checkpoint
    );
    let active = Arc::new(Mutex::new(ActiveRunStore::open(root.path()).unwrap()));
    active
        .lock()
        .unwrap()
        .publish(&StoredCheckpoint::from(&checkpoint))
        .unwrap();
    let profile = persisted_profile_fixture(root.path(), &core);
    let actor = PlayerGameActor::new(player, core, LoadBalanceActor::new().start())
        .with_active_run_store(active.clone())
        .with_player_profile_persistence(profile.clone())
        .start();
    let session = Uuid::new_v4();
    attach(&actor, session).await;
    let before = actor.send(DurableProbe { tick: false }).await.unwrap();
    let execute = |amount| ExecuteAdminCommand {
        session_id: session,
        request_id: "durable-admin".into(),
        command: admin(amount),
    };
    std::fs::create_dir(root.path().join("application_commit.pending.tmp")).unwrap();
    assert_eq!(
        actor.send(execute(777)).await.unwrap().unwrap_err().code,
        "application_persistence_failed"
    );
    assert_eq!(
        actor.send(DurableProbe { tick: false }).await.unwrap(),
        before
    );
    std::fs::remove_dir(root.path().join("application_commit.pending.tmp")).unwrap();
    std::fs::create_dir(root.path().join("player_profile.tmp")).unwrap();
    let accepted = actor.send(execute(777)).await.unwrap().unwrap();
    assert!(accepted.side_messages.iter().any(|message| matches!(message,
        PlayerGameServerMessage::Notification { notification_type, .. } if notification_type == "application_recovery_pending")));
    let pending = actor.send(DurableProbe { tick: false }).await.unwrap();
    assert_ne!(pending, before);
    assert!(actor.send(execute(888)).await.unwrap().is_err());
    assert_eq!(
        actor.send(DurableProbe { tick: false }).await.unwrap(),
        pending
    );
    std::fs::remove_dir(root.path().join("player_profile.tmp")).unwrap();
    actor
        .send(command_for(session, command!(RetryPlayerProfile)))
        .await
        .unwrap()
        .unwrap();
    let saved = active
        .lock()
        .unwrap()
        .snapshot()
        .unwrap()
        .checkpoint
        .unwrap();
    let mut restored = CoreApplication::new(defense_game_data(), 0);
    restored.install_authoritative_player_profile(
        serde_json::to_value(profile.lock().unwrap().profile().unwrap()).unwrap().into()
    ).unwrap();
    restored
        .continue_from_active_run_checkpoint(player, &saved.to_application())
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(saved.core_payload_json.as_slice()).unwrap()
            ["continuation"]["current"]["enkephalin"]["amount"],
        777
    );
}

#[derive(actix::Message)]
#[rtype(result = "Value")]
struct U05LiveProbe {
    advance_ms: u64,
}
impl Handler<U05LiveProbe> for PlayerGameActor {
    type Result = actix::MessageResult<U05LiveProbe>;
    fn handle(&mut self, message: U05LiveProbe, _: &mut Context<Self>) -> Self::Result {
        if message.advance_ms > 0 {
            self.game_core
                .advance_active_battle_for_host_tick(message.advance_ms, 0)
                .unwrap();
        }
        let result = self.game_core.active_battle_reconnect(0).unwrap();
        let spent: i64 = self
            .game_core
            .battle_runtime_facts()
            .unwrap()
            .unwrap()
            .actors()
            .map(|(_, summary)| summary.caused.resource_spent)
            .sum();
        actix::MessageResult(
            serde_json::json!({"update": result.payload().as_value()["battle_update"], "spent": spent}),
        )
    }
}

#[actix_web::test]
async fn u05_paused_accepted_use_survives_same_actor_reconnect_and_spends_once() {
    use game_core::application::ApplicationAdminCommand;
    use game_core::game::data::{
        skill_data::SkillDatabase, skill_fragment_data::SkillFragmentDatabase, GameDataBuilder,
        ValidatedGameData,
    };
    let base = defense_game_data();
    let skill_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core_enhanced/tests/fixtures/r08/skills.ron"
    ))
    .replace("r08_strike", "u05_host_self")
    .replace("focus_time_ms: 100", "focus_time_ms: 1000")
    .replace("automatic_unit", "self")
    .replace("EnemySingle(rule: Nearest)", "SelfUnit")
    .replace("reach: global", "reach: pattern")
    .replace("delay_ms: 200", "delay_ms: 0")
    .replace("Damage(amount: 100, damage_type: True)", "Heal(amount: 1)");
    let mut skills = base.skill_data.skills.clone();
    skills.extend(
        ron::de::from_str::<SkillDatabase>(&skill_source)
            .unwrap()
            .skills,
    );
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core_enhanced/tests/fixtures/r09/fragments.ron"
    ))
    .replace("r09_fragment", "u05_host_fragment")
    .replace("r08_strike", "u05_host_self")
    .replace("resonance_max: 100", "resonance_max: 1");
    let mut fragments = base.skill_fragment_data.fragments.clone();
    fragments.extend(
        ron::de::from_str::<SkillFragmentDatabase>(&source)
            .unwrap()
            .fragments,
    );
    let data = GameDataBuilder::from_validated(&ValidatedGameData::from(base))
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::new(fragments))
        .build_arc();
    let player = Uuid::new_v4();
    let mut app = game_core_waiting_on_live_defense_node_with_data(player, data);
    app.execute_admin(
        ApplicationAdminCommand::from_document(
            serde_json::json!({
                "type": "admin_grant_skill_fragment", "fragment_id": "u05_host_fragment", "count": 1
            })
            .into(),
        )
        .unwrap(),
    )
    .unwrap();
    let snapshot = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let snapshot: game_core::game::behavior::RunSnapshotDto =
        serde_json::from_value(snapshot.read_model().as_value().clone()).unwrap();
    let employee = snapshot.roster.combat_eligible_employee_ids[0];
    let copy = snapshot
        .inventory
        .skill_fragments
        .iter()
        .find(|entry| entry.copy.definition_id.as_str() == "u05_host_fragment")
        .unwrap()
        .copy
        .copy_uuid;
    app.execute_fixture(
        player,
        command!(EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: copy
        }),
    )
    .unwrap();
    let entered = app
        .execute_fixture(player, command!(ConfirmEnterNode))
        .unwrap();
    let setup: game_core::game::behavior::LiveBattleSetupSnapshotDto =
        serde_json::from_value(entered.payload().as_value()["battle_setup_snapshot"].clone())
            .unwrap();
    let position = setup
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == game_core::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells[0];
    let execute = |app: &mut CoreApplication, behavior| {
        let projection = app.active_battle_reconnect(0).unwrap();
        let update = &projection.payload().as_value()["battle_update"];
        app.execute(
            player,
            behavior,
            ApplicationExecutionContext {
                expected_battle_authority: Some(ApplicationBattleAuthority {
                    battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(),
                    battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
    };
    execute(
        &mut app,
        command!(DeployUnit {
            employee_uuid: employee,
            position,
            facing: game_core::game::battle::tile_range::FacingDirection::Right
        }),
    );
    app.advance_active_battle_by(1000, 0).unwrap();
    execute(&mut app, command!(PauseBattle));
    let actor = PlayerGameActor::new(player, app, LoadBalanceActor::new().start())
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    let first = Uuid::new_v4();
    attach(&actor, first).await;
    let before = actor.send(U05LiveProbe { advance_ms: 0 }).await.unwrap();
    let mut command = command_for(
        first,
        command!(ActivateSkill {
            employee_uuid: employee,
            skill_id: "u05_host_self",
            target: serde_json::Value::Null
        }),
    );
    command.battle_authority = Some(serde_json::from_value(serde_json::json!({
        "battle_uuid": before["update"]["battle_uuid"], "battle_state_revision": before["update"]["battle_state_revision"]
    })).unwrap());
    actor.send(command).await.unwrap().unwrap();
    let accepted = actor.send(U05LiveProbe { advance_ms: 5000 }).await.unwrap();
    assert_eq!(accepted["update"]["server_battle_time_ms"], 1000);
    assert_eq!(
        accepted["spent"].as_i64().unwrap(),
        before["spent"].as_i64().unwrap() + 1
    );
    let readiness = |state: &Value| {
        let update: game_core::game::behavior::LiveBattleUpdateDto =
            serde_json::from_value(state["update"].clone()).unwrap();
        update
            .checkpoint
            .deployment
            .unwrap()
            .deployed_units
            .into_iter()
            .find(|unit| unit.employee_uuid == employee)
            .unwrap()
            .skill_readiness
    };
    let ready = readiness(&accepted);
    assert_eq!(ready.resonance_current, 0);
    assert_eq!(ready.resonance_use.as_ref().unwrap().consumed, 1);
    assert_eq!(
        ready.resonance_use.as_ref().unwrap().phase,
        game_core::game::behavior::ResonanceUsePhaseDto::Accepted
    );
    actor
        .send(DetachSession { session_id: first })
        .await
        .unwrap();
    let second = Uuid::new_v4();
    attach(&actor, second).await;
    let reconnected = actor.send(U05LiveProbe { advance_ms: 0 }).await.unwrap();
    assert_eq!(readiness(&reconnected), ready);
    assert_eq!(reconnected["spent"], accepted["spent"]);
    let mut duplicate = command_for(
        second,
        command!(ActivateSkill {
            employee_uuid: employee,
            skill_id: "u05_host_self",
            target: serde_json::Value::Null
        }),
    );
    let authority = || {
        serde_json::from_value(serde_json::json!({"battle_uuid": reconnected["update"]["battle_uuid"], "battle_state_revision": reconnected["update"]["battle_state_revision"]})).unwrap()
    };
    duplicate.battle_authority = Some(authority());
    assert!(actor.send(duplicate).await.unwrap().is_err());
    assert_eq!(
        actor.send(U05LiveProbe { advance_ms: 0 }).await.unwrap(),
        reconnected
    );
    let mut resume = command_for(second, command!(ResumeBattle));
    resume.battle_authority = Some(authority());
    actor.send(resume).await.unwrap().unwrap();
    let finished = actor.send(U05LiveProbe { advance_ms: 1100 }).await.unwrap();
    assert_eq!(finished["spent"], accepted["spent"]);
    assert!(readiness(&finished).resonance_use.is_none());
}

#[actix_web::test]
async fn u07_host_wall_cast_and_object_survive_paused_same_actor_reconnect() {
    use game_core::application::ApplicationAdminCommand;
    use game_core::game::{
        behavior::{
            LiveBattleSetupSnapshotDto, LiveBattleUpdateDto, RunSnapshotDto,
            SkillTargetPreviewResultDto,
        },
        data::{skill_fragment_data::SkillFragmentDatabase, GameDataBuilder, ValidatedGameData},
    };
    let base = defense_game_data();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core_enhanced/tests/fixtures/r09/fragments.ron"
    ))
    .replace("r09_fragment", "u07_host_wall")
    .replace("r08_strike", "common_place_ice_wall")
    .replace("resonance_max: 100", "resonance_max: 1");
    let mut fragments = base.skill_fragment_data.fragments.clone();
    fragments.extend(
        ron::de::from_str::<SkillFragmentDatabase>(&source)
            .unwrap()
            .fragments,
    );
    let data = GameDataBuilder::from_validated(&ValidatedGameData::from(base))
        .with_skill_fragments(SkillFragmentDatabase::new(fragments))
        .build_arc();
    let player = Uuid::from_u128(0xC007510);
    let mut app = game_core_waiting_on_live_defense_node_with_data(player, data);
    app.execute_admin(ApplicationAdminCommand::from_document(serde_json::json!({"type":"admin_grant_skill_fragment","fragment_id":"u07_host_wall","count":1}).into()).unwrap()).unwrap();
    let snapshot: RunSnapshotDto = serde_json::from_value(
        app.snapshot(ApplicationSnapshotKind::Run)
            .unwrap()
            .read_model()
            .as_value()
            .clone(),
    )
    .unwrap();
    let employee = snapshot.roster.combat_eligible_employee_ids[0];
    let copy = snapshot
        .inventory
        .skill_fragments
        .iter()
        .find(|entry| entry.copy.definition_id.as_str() == "u07_host_wall")
        .unwrap()
        .copy
        .copy_uuid;
    app.execute_fixture(
        player,
        command!(EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: copy
        }),
    )
    .unwrap();
    let entered = app
        .execute_fixture(player, command!(ConfirmEnterNode))
        .unwrap();
    let setup: LiveBattleSetupSnapshotDto =
        serde_json::from_value(entered.payload().as_value()["battle_setup_snapshot"].clone())
            .unwrap();
    let position = setup
        .deployment_zones
        .iter()
        .find(|z| z.kind == game_core::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells[0];
    let execute = |app: &mut CoreApplication, behavior| {
        let projection = app.active_battle_reconnect(0).unwrap();
        let update = &projection.payload().as_value()["battle_update"];
        app.execute(
            player,
            behavior,
            ApplicationExecutionContext {
                expected_battle_authority: Some(ApplicationBattleAuthority {
                    battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(),
                    battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
    };
    execute(
        &mut app,
        command!(DeployUnit {
            employee_uuid: employee,
            position,
            facing: game_core::game::battle::tile_range::FacingDirection::Right
        }),
    );
    app.advance_active_battle_by(1000, 0).unwrap();
    execute(
        &mut app,
        command!(BeginSkillTargeting {
            employee_uuid: employee,
            skill_id: "common_place_ice_wall"
        }),
    );
    let target = setup
        .battlefield
        .valid_tiles
        .iter()
        .copied()
        .find(|position| {
            let result = execute(
                &mut app,
                command!(PreviewSkillTarget {
                    employee_uuid: employee,
                    skill_id: "common_place_ice_wall",
                    target: serde_json::json!({"type":"Tile","position":position})
                }),
            );
            let preview: SkillTargetPreviewResultDto =
                serde_json::from_value(result.payload().as_value().clone()).unwrap();
            preview.eligibility.enabled
        })
        .unwrap();
    execute(&mut app, command!(PauseBattle));
    let actor = PlayerGameActor::new(player, app, LoadBalanceActor::new().start())
        .with_live_battle_tick_interval(Duration::from_secs(60))
        .start();
    let first = Uuid::from_u128(0xC007511);
    attach(&actor, first).await;
    let before = actor.send(U05LiveProbe { advance_ms: 0 }).await.unwrap();
    let authority = |state: &Value| {
        serde_json::from_value(serde_json::json!({"battle_uuid":state["update"]["battle_uuid"],"battle_state_revision":state["update"]["battle_state_revision"]})).unwrap()
    };
    let mut cast = command_for(
        first,
        command!(ActivateSkill {
            employee_uuid: employee,
            skill_id: "common_place_ice_wall",
            target: serde_json::json!({"type":"Tile","position":target})
        }),
    );
    cast.battle_authority = Some(authority(&before));
    actor.send(cast).await.unwrap().unwrap();
    let accepted = actor.send(U05LiveProbe { advance_ms: 5000 }).await.unwrap();
    assert_eq!(
        accepted["update"]["server_battle_time_ms"],
        before["update"]["server_battle_time_ms"]
    );
    assert_eq!(
        accepted["spent"].as_i64().unwrap(),
        before["spent"].as_i64().unwrap() + 1
    );
    assert!(
        accepted["update"]["checkpoint"]["independent_objects"]["active"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    actor
        .send(DetachSession { session_id: first })
        .await
        .unwrap();
    let second = Uuid::from_u128(0xC007512);
    attach(&actor, second).await;
    let resumed = actor.send(U05LiveProbe { advance_ms: 5000 }).await.unwrap();
    assert_eq!(resumed["spent"], accepted["spent"]);
    assert_eq!(
        resumed["update"]["checkpoint"]["independent_objects"],
        accepted["update"]["checkpoint"]["independent_objects"]
    );
    let mut resume = command_for(second, command!(ResumeBattle));
    resume.battle_authority = Some(authority(&resumed));
    actor.send(resume).await.unwrap().unwrap();
    let created = actor.send(U05LiveProbe { advance_ms: 600 }).await.unwrap();
    let created_update: LiveBattleUpdateDto =
        serde_json::from_value(created["update"].clone()).unwrap();
    let object = &created_update.checkpoint.independent_objects.active[0];
    assert_eq!(object.definition_id.as_str(), "common_ice_wall");
    assert_eq!(object.max_health, Some(120));
    assert_eq!(object.source.fragment_copy_uuid, Some(copy));
    assert_eq!(created["spent"], accepted["spent"]);
    actor
        .send(DetachSession { session_id: second })
        .await
        .unwrap();
    let third = Uuid::from_u128(0xC007513);
    attach(&actor, third).await;
    let reconnected = actor.send(U05LiveProbe { advance_ms: 5000 }).await.unwrap();
    assert_eq!(
        reconnected["update"]["server_battle_time_ms"],
        created["update"]["server_battle_time_ms"]
    );
    assert_eq!(
        reconnected["update"]["checkpoint"]["independent_objects"],
        created["update"]["checkpoint"]["independent_objects"]
    );
    let mut withdraw = command_for(
        third,
        command!(WithdrawUnit {
            employee_uuid: employee
        }),
    );
    withdraw.battle_authority = Some(authority(&reconnected));
    actor.send(withdraw).await.unwrap().unwrap();
    let withdrawn = actor.send(U05LiveProbe { advance_ms: 5000 }).await.unwrap();
    assert_eq!(
        withdrawn["update"]["checkpoint"]["independent_objects"]["active"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(withdrawn["spent"], created["spent"]);
}
