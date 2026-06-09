use actix::{ActorContext, AsyncContext, Handler};
use serde_json::Value;

use tracing::{info, warn};

use crate::game::player_game_actor::{
    messages::{
        AttachSession, CommandExecutionResult, DetachSession, ExecuteAdminCommand,
        ExecutePlayerBehavior, ForceDisconnect, PlayerGameServerMessage, PushServerMessage,
        QuitPlayerActor,
    },
    state::{
        behavior_result_payload, behavior_result_to_command_result, compress_battle_event_log_payload,
        PlayerGameActorError,
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

        self.active_session_id = Some(msg.session_id);
        self.socket = Some(msg.socket);
        self.session_control = Some(msg.control);
        self.ensure_live_battle_tick(ctx);
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

    fn handle(&mut self, msg: ExecutePlayerBehavior, ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        let result = self
            .game_core
            .execute(self.player_id, msg.behavior)
            .map_err(PlayerGameActorError::from)?;
        let response = behavior_result_to_command_result(msg.request_id, result)?;
        let state_snapshot = self.build_state_snapshot()?;
        self.ensure_live_battle_tick(ctx);

        Ok(CommandExecutionResult {
            response,
            state_snapshot,
        })
    }
}

impl Handler<ExecuteAdminCommand> for PlayerGameActor {
    type Result = Result<CommandExecutionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: ExecuteAdminCommand, ctx: &mut Self::Context) -> Self::Result {
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
        self.ensure_live_battle_tick(ctx);

        Ok(CommandExecutionResult {
            response,
            state_snapshot,
        })
    }
}

impl Handler<PushServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: PushServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.push_to_active_socket(msg.message);
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
        let mut snapshot = self
            .game_core
            .get_run_snapshot_json()
            .map_err(PlayerGameActorError::from)?;

        if let Some((winner, event_log)) = self.game_core.get_combat_result_event_log() {
            if let Some(root) = snapshot.as_object_mut() {
                if let Some(Value::Object(selected_event)) = root.get_mut("selected_event") {
                    selected_event.insert(
                        "compressed_timeline".to_string(),
                        serde_json::to_value(compress_battle_event_log_payload(winner, &event_log)?)
                            .map_err(|error| {
                                PlayerGameActorError::new("serialization_failed", error.to_string())
                            })?,
                    );
                }
            }
        }

        Ok(snapshot)
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

        match behavior_result_payload(result) {
            Ok((result_type, payload)) => {
                self.push_to_active_socket(PlayerGameServerMessage::Notification {
                    notification_type: "battle_delta".to_string(),
                    payload: serde_json::json!({
                        "result_type": result_type,
                        "payload": payload,
                    }),
                });
            }
            Err(error) => {
                self.push_to_active_socket(error.to_server_message(None));
                self.stop_live_battle_tick(ctx);
                return;
            }
        }

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
            messages::{AttachSession, DetachSession, ForceDisconnect, PlayerGameServerMessage},
            PlayerGameActor,
        },
    };
    use actix::{Actor, Context};
    use actix_web::rt::time;
    use game_core::game::{
        behavior::{BehaviorResult, PlayerBehavior},
        combat_preview::{BattlefieldArchetype, BattlefieldSizeClass, CombatNodeType, EnemyKind},
        data::{
            abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
            employee_data::{
                RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase,
            },
            pve_data::{
                PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase, PveWaveData,
                PveWaveEnemyData,
            },
            GameDataBase, GameDataBuilder,
        },
        employee::{EmployeeGrade, StarterEmployeeCandidate},
        enums::{RewardMode, RiskLevel, Tier},
        map::MapNodeCategory,
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

    fn empty_game_data() -> Arc<GameDataBase> {
        let candidates = (0..5)
            .map(|index| StarterEmployeeCandidate {
                id: format!("candidate_{index}"),
                name: format!("Candidate {index}"),
                grade: EmployeeGrade::Junior,
                role: "Test Role".to_string(),
                background: "Test Background".to_string(),
            })
            .collect();

        GameDataBuilder::empty()
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
            route_id: Some("black_box_breach_main".to_string()),
            required_for_victory: true,
            source: None,
            enemies: vec![PveWaveEnemyData {
                kind: EnemyKind::Abnormality,
                profile_id: None,
                abnormality_id: abnormality_id.to_string(),
                tier: Tier::I,
                count: 1,
            }],
        }
    }

    fn defense_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty()
            .with_starter_employee_candidates(StarterEmployeeCandidateDatabase::new(
                (0..6)
                    .map(|index| StarterEmployeeCandidate {
                        id: format!("candidate_{index}"),
                        name: format!("Candidate {index}"),
                        grade: EmployeeGrade::Junior,
                        role: "Test Role".to_string(),
                        background: "Test Background".to_string(),
                    })
                    .collect(),
            ))
            .with_recruitment_employee_candidates(RecruitmentEmployeeCandidateDatabase::new(vec![]))
            .with_abnormalities(vec![test_abnormality_meta("defense_risk_abno", 20_005)])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "defense_encounter".to_string(),
                abnormality_id: "defense_risk_abno".to_string(),
                difficulty: 3,
                risk_level: RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(BattlefieldArchetype::ChokePoint),
                    size_class: Some(BattlefieldSizeClass::Small),
                }),
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
            .execute(player_id, PlayerBehavior::StartNewGame)
            .expect("start new game should open starter selection");
        let BehaviorResult::StartNewGame {
            candidates,
            required_count,
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
        let game_data = defense_game_data();
        for seed in 0..1_000 {
            let mut core = GameCore::new(game_data.clone(), seed);
            let BehaviorResult::StarterEmployeesSelected { mut map, .. } =
                start_new_game_with_default_starters(&mut core, player_id)
            else {
                panic!("starter selection should enter the run map");
            };

            for _ in 0..3 {
                for node_id in map.available_node_ids.clone() {
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

                let Some(start_node_id) = map.available_node_ids.iter().copied().find(|node_id| {
                    map.nodes
                        .iter()
                        .any(|node| node.id == *node_id && node.category == MapNodeCategory::Start)
                }) else {
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
                behavior: PlayerBehavior::StartNewGame,
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
            result.state_snapshot["game_state"],
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
    async fn live_battle_tick_pushes_delta_and_snapshot_after_confirm_enter() {
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
                assert_eq!(result_type, "BattleAdvanced");
                assert_eq!(payload["node_type"], "Defense");
                assert!(payload["battle_uuid"].is_string());
                assert!(payload["encounter_id"].is_string());
            }
            other => panic!("expected BattleAdvanced command result, got {other:?}"),
        }
        assert_eq!(
            result.state_snapshot["game_state_context"]["type"],
            "in_battle"
        );

        time::sleep(Duration::from_millis(60)).await;

        let messages = messages.lock().expect("messages mutex should be lockable");
        let battle_delta = messages.iter().find_map(|message| match message {
            PlayerGameServerMessage::Notification {
                notification_type,
                payload,
            } if notification_type == "battle_delta" => Some(payload),
            _ => None,
        });
        let battle_delta = battle_delta.expect("live battle tick should push battle_delta");
        assert_eq!(battle_delta["result_type"], "BattleAdvanced");
        assert_eq!(battle_delta["payload"]["node_type"], "Defense");
        assert!(battle_delta["payload"]["battle_uuid"].is_string());
        assert!(battle_delta["payload"]["encounter_id"].is_string());

        assert!(messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::StateSnapshot { state }
                if state["game_state_context"]["type"] == "in_battle"
                    || state["game_state_context"]["type"] == "combat_result"
        )));
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
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        let paused = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "pause-live-defense".to_string(),
                behavior: PlayerBehavior::PauseBattle,
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
                assert_eq!(result_type, "BattlePlaybackChanged");
                assert_eq!(payload["playback"]["paused"], true);
            }
            other => panic!("expected BattlePlaybackChanged command result, got {other:?}"),
        }

        time::sleep(Duration::from_millis(120)).await;
        {
            let messages = messages.lock().expect("messages mutex should be lockable");
            assert!(!messages.iter().any(|message| matches!(
                message,
                PlayerGameServerMessage::Notification {
                    notification_type,
                    ..
                } if notification_type == "battle_delta"
            )));
        }

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resume-live-defense".to_string(),
                behavior: PlayerBehavior::ResumeBattle,
            })
            .await
            .expect("resume request should complete")
            .expect("resume should succeed");

        time::sleep(Duration::from_millis(120)).await;
        let messages = messages.lock().expect("messages mutex should be lockable");
        assert!(messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::Notification {
                notification_type,
                payload,
            } if notification_type == "battle_delta"
                && payload["result_type"] == "BattleAdvanced"
        )));
    }
}
