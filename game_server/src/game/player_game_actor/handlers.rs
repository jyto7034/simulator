use actix::{ActorContext, AsyncContext, Handler};
use serde_json::Value;

use tracing::{info, warn};

use crate::game::player_game_actor::{
    messages::{
        AttachSession, CommandExecutionResult, DetachSession, EnsureLiveBattleTick,
        ExecuteAdminCommand, ExecutePlayerBehavior, ForceDisconnect, PlayerGameServerMessage,
        PushServerMessage, QuitPlayerActor,
    },
    state::{
        battle_update_from_behavior_result, behavior_result_to_command_result,
        compress_combat_result_event_log_attachment, PlayerGameActorError, PlayerStateSnapshotDto,
    },
    PlayerGameActor,
};

impl Handler<AttachSession> for PlayerGameActor {
    type Result = Result<Value, PlayerGameActorError>;

    fn handle(&mut self, msg: AttachSession, ctx: &mut Self::Context) -> Self::Result {
        if let Some(timer) = self.disconnect_timer.take() {
            ctx.cancel_future(timer);
        }

        if self.active_session_id != Some(msg.session_id) {
            if let Some(control) = self.session_control.take() {
                control.do_send(ForceDisconnect {
                    code: "session_replaced".to_string(),
                    message: "A newer connection replaced this session".to_string(),
                });
            }
        }

        if matches!(
            self.game_core.get_state(),
            game_core::game::resources::GameState::InBattle { .. }
        ) {
            self.stop_live_battle_tick(ctx);
            self.game_core
                .execute(
                    self.player_id,
                    game_core::game::behavior::PlayerBehavior::RecoverBattleSetupLoss,
                )
                .map_err(PlayerGameActorError::from)?;
        }

        self.active_session_id = Some(msg.session_id);
        self.socket = Some(msg.socket);
        self.session_control = Some(msg.control);
        self.build_state_snapshot()
    }
}

impl Handler<DetachSession> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: DetachSession, ctx: &mut Self::Context) -> Self::Result {
        if self.active_session_id == Some(msg.session_id) {
            self.active_session_id = None;
            self.socket = None;
            self.session_control = None;
            self.stop_live_battle_tick(ctx);

            if let Some(timer) = self.disconnect_timer.take() {
                ctx.cancel_future(timer);
            }

            let ttl = self.disconnect_ttl;
            self.disconnect_timer = Some(ctx.run_later(ttl, |act, ctx| {
                act.disconnect_timer = None;
                if act.active_session_id.is_none() {
                    ctx.stop();
                }
            }));
        }
    }
}

impl Handler<ExecutePlayerBehavior> for PlayerGameActor {
    type Result = Result<CommandExecutionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: ExecutePlayerBehavior, _ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        let result = self
            .game_core
            .execute_with_source_command_id(self.player_id, msg.behavior, Some(&msg.request_id))
            .map_err(PlayerGameActorError::from)?;
        let command_result =
            behavior_result_to_command_result(msg.request_id, result, msg.battle_side_message)?;
        let state_snapshot = command_result
            .send_state_snapshot
            .then(|| self.build_state_snapshot())
            .transpose()?;
        Ok(CommandExecutionResult {
            response: command_result.response,
            side_messages: command_result.side_messages,
            state_snapshot,
        })
    }
}

impl Handler<ExecuteAdminCommand> for PlayerGameActor {
    type Result = Result<CommandExecutionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: ExecuteAdminCommand, _ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        let result = self
            .game_core
            .execute_admin_command(msg.command)
            .map_err(PlayerGameActorError::from)?;
        let response = PlayerGameServerMessage::AdminResult {
            request_id: msg.request_id,
            ok: true,
            result_type: result.result_type.to_string(),
            payload: result.payload,
        };
        let state_snapshot = self.build_state_snapshot()?;
        Ok(CommandExecutionResult {
            response,
            side_messages: Vec::new(),
            state_snapshot: Some(state_snapshot),
        })
    }
}

impl Handler<PushServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: PushServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.push_to_active_socket(msg.message);
    }
}

impl Handler<EnsureLiveBattleTick> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, _msg: EnsureLiveBattleTick, ctx: &mut Self::Context) -> Self::Result {
        self.ensure_live_battle_tick(ctx);
    }
}

impl Handler<QuitPlayerActor> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, _msg: QuitPlayerActor, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "PlayerGameActor quitting on client request: player {}",
            self.player_id
        );
        if let Some(timer) = self.disconnect_timer.take() {
            ctx.cancel_future(timer);
        }
        self.stop_live_battle_tick(ctx);
        self.active_session_id = None;
        self.socket = None;
        self.session_control = None;
        // stopped() 에서 LoadBalance Deregister 가 호출됨 → 다음 접속 시 새 Actor 생성.
        ctx.stop();
    }
}

impl PlayerGameActor {
    fn ensure_active_session(&self, session_id: uuid::Uuid) -> Result<(), PlayerGameActorError> {
        if self.active_session_id == Some(session_id) {
            return Ok(());
        }

        Err(PlayerGameActorError::new(
            "stale_session",
            "This socket is no longer bound to the active player session",
        ))
    }

    fn build_state_snapshot(&self) -> Result<Value, PlayerGameActorError> {
        let snapshot = self
            .game_core
            .get_run_snapshot_dto()
            .map_err(PlayerGameActorError::from)?;
        let compressed_event_log = self
            .game_core
            .get_combat_result_event_log_attachment()
            .map(|attachment| compress_combat_result_event_log_attachment(&attachment))
            .transpose()?;

        serde_json::to_value(PlayerStateSnapshotDto::new(snapshot, compressed_event_log))
            .map_err(|error| PlayerGameActorError::new("serialization_failed", error.to_string()))
    }

    fn push_to_active_socket(&self, message: super::messages::PlayerGameServerMessage) {
        if let Some(socket) = &self.socket {
            socket.do_send(message);
        }
    }

    fn ensure_live_battle_tick(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if self.live_battle_tick.is_some()
            || self.socket.is_none()
            || !self.has_active_live_battle()
        {
            return;
        }

        let interval = self.live_battle_tick_interval;
        self.live_battle_tick = Some(ctx.run_interval(interval, |act, ctx| {
            act.tick_live_battle(ctx);
        }));
    }

    fn stop_live_battle_tick(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if let Some(handle) = self.live_battle_tick.take() {
            ctx.cancel_future(handle);
        }
    }

    fn tick_live_battle(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if self.socket.is_none() || !self.has_active_live_battle() {
            self.stop_live_battle_tick(ctx);
            return;
        }

        let delta_ms = self.live_battle_tick_interval.as_millis();
        let delta_ms = u64::try_from(delta_ms).unwrap_or(u64::MAX).max(1);
        let result = match self
            .game_core
            .advance_active_battle_for_server_tick(delta_ms)
        {
            Ok(result) => result,
            Err(error) => {
                warn!(
                    "Live battle tick failed for player {}: {:?}",
                    self.player_id, error
                );
                self.push_to_active_socket(
                    PlayerGameActorError::from(error).to_server_message(None),
                );
                self.stop_live_battle_tick(ctx);
                return;
            }
        };
        let Some(result) = result else {
            return;
        };

        let finished = matches!(
            result,
            game_core::game::behavior::BehaviorResult::BattleAdvanced { finished: true, .. }
        );

        if let Some(battle_update) = battle_update_from_behavior_result(&result) {
            self.push_to_active_socket(PlayerGameServerMessage::battle_update(battle_update));
        } else {
            self.push_to_active_socket(
                PlayerGameActorError::new(
                    "missing_battle_update",
                    "Live battle tick did not produce a battle_update payload",
                )
                .to_server_message(None),
            );
            self.stop_live_battle_tick(ctx);
            return;
        }

        if finished {
            match self.build_state_snapshot() {
                Ok(state) => {
                    self.push_to_active_socket(PlayerGameServerMessage::StateSnapshot { state });
                }
                Err(error) => {
                    self.push_to_active_socket(error.to_server_message(None));
                    self.stop_live_battle_tick(ctx);
                    return;
                }
            }
        }

        if finished || !self.has_active_live_battle() {
            self.stop_live_battle_tick(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        load_balance_actor::{messages::Register, LoadBalanceActor},
        player_game_actor::{
            messages::{
                AttachSession, BattleSideMessageKind, DetachSession, EnsureLiveBattleTick,
                ForceDisconnect, PlayerGameServerMessage,
            },
            PlayerGameActor,
        },
    };
    use actix::{Actor, Context};
    use actix_web::rt::time;
    use game_core::game::{
        behavior::{BehaviorResult, PlayerBehavior},
        combat_preview::{BattlefieldArchetype, BattlefieldSizeClass, CombatNodeType},
        data::{
            abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
            employee_data::{
                RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase,
            },
            equipment_data::{EquipmentMetadata, EquipmentType},
            pve_data::{
                PveBattlefieldOverrideData, PveEncounter, PveEncounterClass, PveEncounterDatabase,
                PveWaveData, PveWaveEnemyData, PveWaveSource,
            },
            skill_fragment_data::{
                SkillFragmentAcquisitionSource, SkillFragmentCompatibilityRequirements,
                SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentEquipLimit,
                SkillFragmentId, SkillFragmentMetadata, SkillFragmentOrigin, SkillFragmentRarity,
            },
            GameDataBase, GameDataBuilder,
        },
        employee::{StarterEmployeeCandidate, StarterEmployeeLoadout},
        enums::{RewardMode, RiskLevel, Tier},
        map::{GameMode, MapNodeCategory, MapNodeState},
        world::GameCore,
    };
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    use uuid::Uuid;

    struct ProbeSession {
        disconnects: Arc<Mutex<Vec<(String, String)>>>,
        messages: Arc<Mutex<Vec<PlayerGameServerMessage>>>,
    }

    impl Actor for ProbeSession {
        type Context = Context<Self>;
    }

    impl Handler<ForceDisconnect> for ProbeSession {
        type Result = ();

        fn handle(&mut self, msg: ForceDisconnect, _ctx: &mut Self::Context) -> Self::Result {
            self.disconnects
                .lock()
                .expect("disconnects mutex should be lockable")
                .push((msg.code, msg.message));
        }
    }

    impl Handler<PlayerGameServerMessage> for ProbeSession {
        type Result = ();

        fn handle(
            &mut self,
            msg: PlayerGameServerMessage,
            _ctx: &mut Self::Context,
        ) -> Self::Result {
            self.messages
                .lock()
                .expect("messages mutex should be lockable")
                .push(msg);
        }
    }

    fn spawn_probe() -> (
        actix::Addr<ProbeSession>,
        Arc<Mutex<Vec<(String, String)>>>,
        Arc<Mutex<Vec<PlayerGameServerMessage>>>,
    ) {
        let disconnects = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let addr = ProbeSession {
            disconnects: disconnects.clone(),
            messages: messages.clone(),
        }
        .start();

        (addr, disconnects, messages)
    }

    fn starter_test_loadout() -> StarterEmployeeLoadout {
        StarterEmployeeLoadout {
            equipment_ids: vec!["standard_armor".to_string()],
            baseline_skill_fragment_ids: vec![SkillFragmentId::from(
                "starter_basic_attack_enhancement",
            )],
        }
    }

    fn starter_test_equipment() -> EquipmentMetadata {
        EquipmentMetadata {
            id: "standard_armor".to_string(),
            uuid: Uuid::parse_str("650e8400-e29b-41d4-a716-446655440031")
                .expect("standard armor uuid should be valid"),
            name: "Standard Training E.G.O Armor".to_string(),
            equipment_type: EquipmentType::Armor,
            rarity: RiskLevel::TETH,
            price: 80,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: Default::default(),
            ability_activations: Vec::new(),
            weapon_profile: None,
        }
    }

    fn starter_test_skill_fragment() -> SkillFragmentMetadata {
        SkillFragmentMetadata {
            id: SkillFragmentId::from("starter_basic_attack_enhancement"),
            uuid: Uuid::parse_str("53544152-5445-525f-4652-414700000001")
                .expect("starter basic attack fragment uuid should be valid"),
            name: "Starter Basic Attack Enhancement".to_string(),
            description:
                "A baseline fragment that lets employees perform reinforced basic attacks."
                    .to_string(),
            rarity: SkillFragmentRarity::Common,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: Some(SkillFragmentOrigin::Concept {
                concept_id: "employee_baseline_training".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::DependentConcept {
                concept_id: "employee_baseline_training".to_string(),
            }],
            dependencies: Vec::new(),
            compatibility: SkillFragmentCompatibilityRequirements::default(),
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 2,
                attack_interval_ms_reduction: 0,
            },
        }
    }

    fn starter_policy_game_data_builder() -> GameDataBuilder {
        GameDataBuilder::empty()
            .with_equipment(vec![starter_test_equipment()])
            .with_skill_fragments(SkillFragmentDatabase::new(vec![
                starter_test_skill_fragment(),
            ]))
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        let candidates = (0..5)
            .map(|index| StarterEmployeeCandidate {
                id: format!("candidate_{index}"),
                name: format!("Candidate {index}"),
                role: "Test Role".to_string(),
                background: "Test Background".to_string(),
                starter_loadout: starter_test_loadout(),
            })
            .collect();

        starter_policy_game_data_builder()
            .with_starter_employee_candidates(StarterEmployeeCandidateDatabase::new(candidates))
            .build_arc()
    }

    fn test_abnormality_meta(id: &str, uuid: u128) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            target_traits: Vec::new(),
            mobility_kind: Default::default(),
            threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: Some(SkillFragmentId::from(
                "starter_basic_attack_enhancement",
            )),
            omen_chain_id: None,
            movement: MovementDef::default(),
            basic_attack: BasicAttackDef::default(),
            resonance: ResonanceDef::default(),
            skill_id: None,
        }
    }

    fn test_pve_wave(abnormality_id: &str) -> PveWaveData {
        PveWaveData {
            id: "wave_0".to_string(),
            time_ms: 0,
            spawn_zone_ids: Vec::new(),
            route_id: Some("defense_main".to_string()),
            required_for_victory: true,
            source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                abnormality_id: abnormality_id.to_string(),
                tier: Tier::I,
                count: 1,
            }]),
        }
    }

    fn defense_game_data() -> Arc<GameDataBase> {
        defense_game_data_with_survive_timer(None)
    }

    fn defense_game_data_with_survive_timer(survive_timer_ms: Option<u64>) -> Arc<GameDataBase> {
        starter_policy_game_data_builder()
            .with_starter_employee_candidates(StarterEmployeeCandidateDatabase::new(
                (0..6)
                    .map(|index| StarterEmployeeCandidate {
                        id: format!("candidate_{index}"),
                        name: format!("Candidate {index}"),
                        role: "Test Role".to_string(),
                        background: "Test Background".to_string(),
                        starter_loadout: starter_test_loadout(),
                    })
                    .collect(),
            ))
            .with_recruitment_employee_candidates(RecruitmentEmployeeCandidateDatabase::new(vec![]))
            .with_abnormalities(vec![test_abnormality_meta("defense_risk_abno", 20_005)])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "defense_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
                risk_level: RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(BattlefieldArchetype::ChokePoint),
                    size_class: Some(BattlefieldSizeClass::Small),
                }),
                survive_timer_ms,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("defense_risk_abno")],
                static_obstacles: vec![],
            }]))
            .build_arc()
    }

    fn start_new_game_with_default_starters(
        core: &mut GameCore,
        player_id: Uuid,
    ) -> BehaviorResult {
        let start = core
            .execute(
                player_id,
                PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                },
            )
            .expect("start new game should open starter selection");
        let BehaviorResult::StartNewGame {
            candidates,
            required_count,
            ..
        } = start
        else {
            panic!("start new game should return starter candidates");
        };
        let candidate_ids = candidates
            .into_iter()
            .take(required_count)
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>();
        core.execute(
            player_id,
            PlayerBehavior::SelectStarterEmployees { candidate_ids },
        )
        .expect("default starter employee selection should start the run")
    }

    fn game_core_waiting_on_live_defense_node(player_id: Uuid) -> GameCore {
        game_core_waiting_on_live_defense_node_with_data(player_id, defense_game_data())
    }

    fn game_core_waiting_on_live_survival_defense_node(player_id: Uuid) -> GameCore {
        game_core_waiting_on_live_defense_node_with_data(
            player_id,
            defense_game_data_with_survive_timer(Some(1)),
        )
    }

    fn game_core_waiting_on_live_defense_node_with_data(
        player_id: Uuid,
        game_data: Arc<GameDataBase>,
    ) -> GameCore {
        for seed in 0..1_000 {
            let mut core = GameCore::new(game_data.clone(), seed);
            let BehaviorResult::StarterEmployeesSelected { mut map, .. } =
                start_new_game_with_default_starters(&mut core, player_id)
            else {
                panic!("starter selection should enter the run map");
            };

            for _ in 0..3 {
                let available_node_ids = map
                    .nodes
                    .iter()
                    .filter(|node| node.state == MapNodeState::Available)
                    .map(|node| node.id)
                    .collect::<Vec<_>>();
                for node_id in available_node_ids {
                    let preview = core
                        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
                        .expect("available node preview should succeed");
                    let is_live_defense = matches!(
                        &preview,
                        BehaviorResult::NodePreview {
                            combat_preview: Some(combat_preview),
                            ..
                        } if combat_preview.node_type == CombatNodeType::Defense
                    );
                    if is_live_defense {
                        return core;
                    }

                    core.execute(player_id, PlayerBehavior::CancelSelectedNode)
                        .expect("non-defense node preview should be cancellable");
                }

                let Some(start_node_id) = map
                    .nodes
                    .iter()
                    .find(|node| {
                        node.state == MapNodeState::Available
                            && node.category == MapNodeCategory::Start
                    })
                    .map(|node| node.id)
                else {
                    break;
                };
                let preview = core
                    .execute(
                        player_id,
                        PlayerBehavior::SelectMapNode {
                            node_id: start_node_id,
                        },
                    )
                    .expect("start node preview should succeed");
                assert!(matches!(
                    preview,
                    BehaviorResult::NodePreview {
                        category: MapNodeCategory::Start,
                        ..
                    }
                ));
                let entered = core
                    .execute(player_id, PlayerBehavior::ConfirmEnterNode)
                    .expect("start node confirm should succeed");
                assert!(matches!(
                    entered,
                    BehaviorResult::NodeEntered {
                        category: MapNodeCategory::Start,
                        ..
                    }
                ));
                let completed = core
                    .execute(player_id, PlayerBehavior::CompleteNode)
                    .expect("start node complete should succeed");
                let BehaviorResult::NodeCompleted { map: next_map, .. } = completed else {
                    panic!("start node completion should return updated map");
                };
                map = next_map;
            }
        }

        panic!("expected at least one generated run seed to start near a live Defense node");
    }

    #[test]
    fn player_state_snapshot_preserves_core_snapshot_root_fields() {
        let core = GameCore::new(empty_game_data(), 7);
        let core_snapshot = core
            .get_run_snapshot_dto()
            .expect("core snapshot should build");
        let core_json = serde_json::to_value(&core_snapshot).expect("core snapshot should serialize");
        let player_json = serde_json::to_value(PlayerStateSnapshotDto::new(core_snapshot, None))
            .expect("player snapshot should serialize");

        let core_fields = core_json
            .as_object()
            .expect("core snapshot should be an object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let player_fields = player_json
            .as_object()
            .expect("player snapshot should be an object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            player_fields, core_fields,
            "server snapshot wrapper must not maintain a second root field list"
        );

        for field in core_fields {
            assert_eq!(
                player_json.get(&field),
                core_json.get(&field),
                "server snapshot should preserve core field `{field}` when no transport attachment is added"
            );
        }
    }

    #[actix_web::test]
    async fn execute_behavior_returns_updated_state_snapshot() {
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
        .start();

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        let snapshot = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        assert_eq!(snapshot["game_state"], "not_started");

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "req-1".to_string(),
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                },
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("execute request should complete")
            .expect("execute should succeed");

        match result.response {
            PlayerGameServerMessage::CommandResult {
                request_id,
                result_type,
                ..
            } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(result_type, "StartNewGame");
            }
            other => panic!("expected command_result, got {other:?}"),
        }

        assert_eq!(
            result
                .state_snapshot
                .expect("non-battle commands should send state_snapshot")["game_state"],
            "selecting_starter_employees"
        );
    }

    #[actix_web::test]
    async fn attach_replaces_previous_session_immediately() {
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 9),
            load_balance,
        )
        .start();

        let (probe_one, disconnects_one, _) = spawn_probe();
        let (probe_two, _, _) = spawn_probe();

        actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe_one.clone().recipient(),
                control: probe_one.recipient(),
            })
            .await
            .expect("first attach request should complete")
            .expect("first attach should succeed");

        actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe_two.clone().recipient(),
                control: probe_two.recipient(),
            })
            .await
            .expect("second attach request should complete")
            .expect("second attach should succeed");

        time::sleep(Duration::from_millis(25)).await;

        let disconnects = disconnects_one
            .lock()
            .expect("disconnects mutex should be lockable");
        assert_eq!(disconnects.len(), 1);
        assert_eq!(disconnects[0].0, "session_replaced");
    }

    #[actix_web::test]
    async fn detached_actor_stops_after_ttl_and_deregisters() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::new_v4();
        let actor = PlayerGameActor::new(
            player_id,
            GameCore::new(empty_game_data(), 13),
            load_balance.clone(),
        )
        .with_disconnect_ttl(Duration::from_millis(20))
        .start();

        load_balance
            .send(Register {
                player_id,
                addr: actor.clone(),
            })
            .await
            .expect("register should succeed");

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor.do_send(DetachSession { session_id });

        time::sleep(Duration::from_millis(80)).await;

        assert_eq!(
            load_balance
                .send(crate::game::load_balance_actor::messages::GetPlayerCount)
                .await
                .expect("player count query should succeed"),
            0
        );
    }

    #[actix_web::test]
    async fn live_battle_tick_pushes_update_without_state_snapshot_after_confirm_enter() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51DE);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(5))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: PlayerBehavior::ConfirmEnterNode,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        match result.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "enter-live-defense");
                assert_eq!(payload["accepted_at_battle_time_ms"], 0);
            }
            other => panic!("expected CommandAccepted command result, got {other:?}"),
        }
        assert!(matches!(
            result.side_messages.as_slice(),
            [
                PlayerGameServerMessage::BattleSetupSnapshot {
                    battle_uuid: setup_battle_uuid,
                    setup_version: 1,
                    routes,
                    deployment_zones,
                    ..
                },
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    server_battle_time_ms: 0,
                    events_delta,
                    checkpoint,
                }
            ] if !battle_uuid.is_nil()
                && setup_battle_uuid == battle_uuid
                && !routes.is_empty()
                && !deployment_zones.is_empty()
                && events_delta.after_seq == 0
                && events_delta.to_seq == checkpoint.at_seq
                && checkpoint.battle_time_ms == 0
        ));
        assert!(
            result.state_snapshot.is_none(),
            "battle start is represented by battle_setup_snapshot plus battle_update, not full state_snapshot"
        );
        time::sleep(Duration::from_millis(20)).await;
        {
            let messages = messages.lock().expect("messages mutex should be lockable");
            assert!(!messages
                .iter()
                .any(|message| matches!(message, PlayerGameServerMessage::BattleUpdate { .. })));
        }
        actor.do_send(EnsureLiveBattleTick);

        let resync = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resync-live-defense".to_string(),
                behavior: PlayerBehavior::RequestBattleState { since_seq: Some(0) },
                battle_side_message: BattleSideMessageKind::BattleResync,
            })
            .await
            .expect("resync request should complete")
            .expect("resync should succeed");
        match resync.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "resync-live-defense");
            }
            other => panic!("expected CommandAccepted resync result, got {other:?}"),
        }
        assert!(resync.side_messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleResync {
                update,
                ..
            } if update.events_delta.after_seq == 0
                && update.events_delta.to_seq == update.checkpoint.at_seq
                && !update.events_delta.events.is_empty()
        )));
        assert!(
            resync.state_snapshot.is_none(),
            "catch-up battle_resync is a battle side message and must not be followed by full state_snapshot"
        );

        time::sleep(Duration::from_millis(60)).await;

        let messages = messages.lock().expect("messages mutex should be lockable");
        let battle_update = messages.iter().find_map(|message| match message {
            PlayerGameServerMessage::BattleUpdate {
                battle_uuid,
                events_delta,
                checkpoint,
                ..
            } => Some((battle_uuid, events_delta, checkpoint)),
            _ => None,
        });
        let (battle_uuid, events_delta, checkpoint) =
            battle_update.expect("live battle tick should push battle_update");
        assert!(!battle_uuid.is_nil());
        assert_eq!(events_delta.to_seq, checkpoint.at_seq);

        assert!(!messages
            .iter()
            .any(|message| matches!(message, PlayerGameServerMessage::StateSnapshot { .. })));
    }

    #[actix_web::test]
    async fn finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E2);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_survival_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(5))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-survival-defense".to_string(),
                behavior: PlayerBehavior::ConfirmEnterNode,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        actor.do_send(EnsureLiveBattleTick);

        for _ in 0..20 {
            time::sleep(Duration::from_millis(20)).await;
            let messages = messages.lock().expect("messages mutex should be lockable");
            if messages
                .iter()
                .any(|message| matches!(message, PlayerGameServerMessage::StateSnapshot { .. }))
            {
                break;
            }
        }

        let messages = messages.lock().expect("messages mutex should be lockable");
        let final_snapshot_index = messages
            .iter()
            .position(|message| {
                matches!(
                    message,
                    PlayerGameServerMessage::StateSnapshot { state }
                        if state["game_state_context"]["type"] == "combat_result"
                            && state["selected_event"]["type"] == "combat_battle"
                            && state["selected_event"]["result_stats"].is_object()
                            && state["selected_event"]["compressed_event_log"].is_object()
                )
            })
            .expect("finished live tick should push combat_result state_snapshot");
        let final_update_index = messages
            .iter()
            .position(|message| matches!(message, PlayerGameServerMessage::BattleUpdate { .. }))
            .expect("finished live tick should push final battle_update");

        assert!(
            final_update_index < final_snapshot_index,
            "final battle_update must arrive before combat_result state_snapshot"
        );
    }

    #[actix_web::test]
    async fn live_battle_resync_rejects_future_since_seq() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E0);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .start();

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: PlayerBehavior::ConfirmEnterNode,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        let err = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "future-resync".to_string(),
                behavior: PlayerBehavior::RequestBattleState {
                    since_seq: Some(u64::MAX),
                },
                battle_side_message: BattleSideMessageKind::BattleResync,
            })
            .await
            .expect("future resync request should complete")
            .expect_err("future since_seq should be rejected");

        assert_eq!(err.code, "invalid_battle_resync_seq");
    }

    #[actix_web::test]
    async fn attaching_to_active_battle_recovers_to_node_confirm_snapshot() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E1);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(50))
        .start();

        let (probe_one, disconnects_one, _) = spawn_probe();
        let session_one = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id: session_one,
                socket: probe_one.clone().recipient(),
                control: probe_one.recipient(),
            })
            .await
            .expect("first attach request should complete")
            .expect("first attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id: session_one,
                request_id: "enter-live-defense".to_string(),
                behavior: PlayerBehavior::ConfirmEnterNode,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        let (probe_two, _, _) = spawn_probe();
        let session_two = Uuid::new_v4();
        let snapshot = actor
            .send(AttachSession {
                session_id: session_two,
                socket: probe_two.clone().recipient(),
                control: probe_two.recipient(),
            })
            .await
            .expect("second attach request should complete")
            .expect("second attach should succeed");

        assert_eq!(snapshot["game_state_context"]["type"], "node_confirm");
        assert!(snapshot["game_state_context"]["combat_preview"].is_object());
        assert!(snapshot["allowed_actions"]
            .as_array()
            .is_some_and(|actions| actions.iter().any(|action| action == "ConfirmEnterNode")));

        let disconnects = disconnects_one
            .lock()
            .expect("disconnects mutex should be lockable");
        assert!(disconnects
            .iter()
            .any(|(code, _)| code == "session_replaced"));
    }

    #[actix_web::test]
    async fn paused_live_battle_tick_does_not_push_delta_until_resumed() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51DF);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(50))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: PlayerBehavior::ConfirmEnterNode,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");
        actor.do_send(EnsureLiveBattleTick);

        let paused = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "pause-live-defense".to_string(),
                behavior: PlayerBehavior::PauseBattle,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("pause request should complete")
            .expect("pause should succeed");
        match paused.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "pause-live-defense");
            }
            other => panic!("expected CommandAccepted command result, got {other:?}"),
        }
        assert!(paused.side_messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleUpdate { checkpoint, .. }
                if checkpoint.playback.paused
        )));
        assert!(
            paused.state_snapshot.is_none(),
            "battle playback commands must not be followed by full state_snapshot"
        );
        assert!(!paused
            .side_messages
            .iter()
            .any(|message| matches!(message, PlayerGameServerMessage::BattleSetupSnapshot { .. })));

        time::sleep(Duration::from_millis(120)).await;
        {
            let messages = messages.lock().expect("messages mutex should be lockable");
            assert!(!messages
                .iter()
                .any(|message| matches!(message, PlayerGameServerMessage::BattleUpdate { .. })));
        }

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resume-live-defense".to_string(),
                behavior: PlayerBehavior::ResumeBattle,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("resume request should complete")
            .expect("resume should succeed");

        time::sleep(Duration::from_millis(120)).await;
        let messages = messages.lock().expect("messages mutex should be lockable");
        assert!(messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleUpdate {
                events_delta,
                checkpoint,
                ..
            } if events_delta.to_seq == checkpoint.at_seq
        )));
    }
}
