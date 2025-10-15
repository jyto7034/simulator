use actix::{ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture};
use futures_util::StreamExt;
use std::collections::HashSet;
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::observer_actor::{
    message::{EventStreamMessage, EventType, InternalEvent, PlayerFinished, StartObservation},
    EventRequirement, ObserverActor, Phase,
};
use crate::BehaviorOutcome;

impl Handler<StartObservation> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: StartObservation, ctx: &mut Self::Context) {
        info!("[{}] Starting event observation...", self.test_name);

        // 관찰 시작 시, 모든 플레이어를 초기 단계(Enqueuing)로 설정
        for player_id in &msg.player_ids {
            self.players_phase.insert(*player_id, Phase::Enqueuing);
            self.player_satisfied_requirements
                .insert(*player_id, HashSet::new());
        }

        let mut params: Vec<String> = Vec::new();

        // session_id 필터 (특정 테스트 세션만 격리)
        params.push(format!("session_id={}", self.test_session_id));

        // 선택적 환경 변수 필터
        if let Ok(kind) = std::env::var("OBSERVER_STREAM_KIND") {
            if !kind.is_empty() {
                params.push(format!("kind={}", kind));
            }
        }
        if let Ok(gm) = std::env::var("OBSERVER_FILTER_GAME_MODE") {
            if !gm.is_empty() {
                params.push(format!("game_mode={}", gm));
            }
        }
        if let Ok(et) = std::env::var("OBSERVER_FILTER_EVENT_TYPE") {
            if !et.is_empty() {
                params.push(format!("event_type={}", et));
            }
        }

        let url = format!(
            "{}/events/stream?{}",
            self.match_server_url,
            params.join("&")
        );

        let actor_future = async move {
            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    let (_sink, stream) = ws_stream.split();
                    Some(stream)
                }
                Err(e) => {
                    error!("Failed to connect to event stream: {}", e);
                    None
                }
            }
        }
        .into_actor(self)
        .map(|stream_opt, act, ctx| {
            if let Some(stream) = stream_opt {
                info!(
                    "[{}] Successfully connected to event stream.",
                    act.test_name
                );
                ctx.add_stream(stream);
            } else {
                error!("[{}] Failed to add event stream.", act.test_name);
                ctx.stop();
            }
        });

        ctx.spawn(actor_future);
    }
}

impl actix::Handler<PlayerFinished> for ObserverActor {
    type Result = ();
    fn handle(&mut self, msg: PlayerFinished, ctx: &mut Self::Context) {
        let player_id = msg.player_id;

        info!(
            "Test [{}] Player {} finished with outcome: {:?}",
            self.test_name, player_id, msg.result
        );

        // Error outcome이면 실패 처리
        if matches!(msg.result, BehaviorOutcome::Error(_)) {
            self.mark_player_failed(
                player_id,
                &format!("PlayerActor finished with Error outcome: {:?}", msg.result),
                ctx,
            );
            return;
        }

        // IntendError는 의도된 에러이므로 즉시 테스트 성공으로 종료
        if matches!(msg.result, BehaviorOutcome::IntendError) {
            let duration = self.started_at.elapsed();
            info!(
                "✓ Test [{}] Player {} returned IntendError (expected behavior) - completing test immediately. Duration: {:?}",
                self.test_name, player_id, duration
            );

            // 즉시 성공 신호 전송
            if let Some(tx) = self.completion_tx.take() {
                let _ = tx.send(true);
            }
            ctx.stop();
            actix::System::current().stop();
            return;
        }

        // 정상 종료인 경우, phase 상태 확인
        let current_phase = self.players_phase.get(&player_id);

        // 아직 terminal state가 아니면 Finished로 설정
        if let Some(phase) = current_phase {
            if !matches!(phase, Phase::Matched | Phase::Dequeued | Phase::Finished) {
                info!(
                    "Test [{}] Player {} actor finished before reaching terminal phase, setting to Finished",
                    self.test_name, player_id
                );
                self.players_phase.insert(player_id, Phase::Finished);
            }
        }

        // 모든 플레이어가 완료되었는지 확인
        self.check_all_players_finished(ctx);
    }
}

// WebSocket 스트림으로부터 메시지를 받는 핸들러
impl StreamHandler<Result<tungstenite::Message, tungstenite::Error>> for ObserverActor {
    fn handle(
        &mut self,
        item: Result<tungstenite::Message, tungstenite::Error>,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(tungstenite::Message::Text(text)) => {
                match serde_json::from_str::<EventStreamMessage>(&text) {
                    Ok(event) => {
                        ctx.address().do_send(InternalEvent(event));
                    }
                    Err(_e) => {
                        error!("Failed to parse event stream message");
                    }
                }
            }
            Ok(_) => { /* 다른 메시지 타입은 무시 */ }
            Err(e) => {
                error!("Event stream error: {}. Stopping observer.", e);
                ctx.stop();
            }
        }
    }
}

//   1. WebSocket 메시지 (ServerMessage) - /ws/ 엔드포인트
//   // protocol.rs:30-54
//   pub enum ServerMessage {
//       EnQueued { pod_id: String },
//       DeQueued,
//       MatchFound { session_id, server_address },
//       Error { code, message },
//   }
//   - 수신: PlayerActor (매칭 WebSocket 연결)
//   - 목적: 플레이어에게 매칭 결과 즉시 알림
//   - 전송: session/mod.rs에서 ctx.text() 직접 전송

//   2. Redis Stream 이벤트 - /events/stream 엔드포인트
//   // event_stream.rs:14-21
//   pub struct EventStreamMessage {
//       pub event_type: String,  // "enqueued", "dequeued", "match_found", "error"
//       pub player_id: Option<Uuid>,
//       pub timestamp: DateTime<Utc>,
//       pub data: serde_json::Value,
//   }
//   - 수신: ObserverActor (테스트 검증용)
//   - 목적: 테스트 시나리오의 Phase 검증
//   - 전송: Redis Stream → EventStreamSession이 WebSocket으로 중계

//   차이점:
//   - ServerMessage: 플레이어 실시간 통신
//   - Redis Event: 테스트 관찰/검증 (metadata에 test_session_id 있을 때만 발행)

// Match Server 로부터 온 메시지를 처리하는 핸들러.
impl Handler<InternalEvent> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: InternalEvent, ctx: &mut Self::Context) {
        let event = msg.0;

        // event_type 기반으로 scope 구분
        let is_player_event = matches!(
            event.event_type,
            EventType::PlayerEnqueued
                | EventType::PlayerDequeued
                | EventType::PlayerMatchFound
                | EventType::PlayerError
        );

        if is_player_event {
            // Player 이벤트는 player_id 필수
            if let Some(player_id) = event.player_id {
                self.check_phase_completion(player_id, event, ctx);
            } else {
                warn!(
                    "Test [{}] Received player event without player_id: type={:?}, data={:?}",
                    self.test_name, event.event_type, event.data
                );
            }
        } else {
            // Global 이벤트 처리
            self.handle_global_event(event, ctx);
        }
    }
}

enum EventKind<'a> {
    Required(usize, &'a EventRequirement), // idx와 requirement 참조
    Transition(&'a EventRequirement),      // requirement 참조 (matcher 검증용)
    Invalid,
}

impl ObserverActor {
    /// 플레이어를 Failed 상태로 전환하고 테스트 종료를 확인합니다.
    fn mark_player_failed(
        &mut self,
        player_id: Uuid,
        reason: &str,
        ctx: &mut actix::Context<Self>,
    ) {
        warn!(
            "Test [{}] Player {} marked as Failed: {}",
            self.test_name, player_id, reason
        );
        self.players_phase.insert(player_id, Phase::Failed);
        self.player_satisfied_requirements
            .insert(player_id, HashSet::new());
        self.check_all_players_finished(ctx);
    }

    // 플레이어의 현재 단계(Phase)의 완료 조건을 확인하고, 충족 시 다음 단계로 전환합니다.
    fn check_phase_completion(
        &mut self,
        player_id: Uuid,
        event: EventStreamMessage,
        ctx: &mut actix::Context<Self>,
    ) {
        let event_type = &event.event_type;
        let data = &event.data;

        let current_phase = match self.players_phase.get(&player_id) {
            Some(phase) => phase.clone(),
            None => {
                warn!(
                    "Test [{}] Received event for untracked player {}",
                    self.test_name, player_id
                );
                return;
            }
        };

        // 현재 페이즈의 조건 가져오기
        let condition = match self
            .players_schedule
            .get(&player_id)
            .and_then(|schedule| schedule.get(&current_phase))
        {
            Some(cond) => cond,
            None => return,
        };

        // 현재 이벤트가 required_events 에 속한지 확인하여 이벤트의 종류를 알아냄.
        let event_kind = if let Some((idx, req)) = condition
            .required_events
            .iter()
            .enumerate()
            .find(|(_, req)| req.event_type == *event_type)
        {
            EventKind::Required(idx, req)
        } else if condition.transition_event.event_type == *event_type {
            EventKind::Transition(&condition.transition_event)
        } else {
            EventKind::Invalid
        };

        // Error Event 도 matches 로 처리함 ( 의도된 Error 일 수 도 있어서. )
        match event_kind {
            EventKind::Required(idx, req) => {
                // matcher 에서 부적합 결과가 나온 경우 실패 처리
                if !req.matches(event_type, data) {
                    self.mark_player_failed(
                        player_id,
                        &format!(
                            "failed matcher for required event {:?} in phase {:?}",
                            event_type, current_phase
                        ),
                        ctx,
                    );
                    return;
                }

                // matches 통과한 이벤트 등록
                self.player_satisfied_requirements
                    .entry(player_id)
                    .or_default()
                    .insert(idx);
            }
            EventKind::Transition(req) => {
                // matcher 에서 부적합 결과가 나온 경우 실패 처리
                if !req.matches(event_type, data) {
                    self.mark_player_failed(
                        player_id,
                        &format!(
                            "failed matcher for transition event {:?} in phase {:?}",
                            event_type, current_phase
                        ),
                        ctx,
                    );
                    return;
                }

                let satisfied = self.player_satisfied_requirements.get(&player_id).unwrap();

                // 모든 required_events가 만족되었는지 확인
                // 만약 만족되지 않았다면 실패 처리
                let all_required_satisfied =
                    (0..condition.required_events.len()).all(|idx| satisfied.contains(&idx));

                if all_required_satisfied {
                    // 단계 전환
                    let next_phase = condition.next_phase.clone();
                    info!(
                        "Test [{}] Player {} completed phase {:?} -> transitioning to {:?}",
                        self.test_name, player_id, current_phase, next_phase
                    );
                    self.players_phase.insert(player_id, next_phase);
                    self.player_satisfied_requirements
                        .insert(player_id, HashSet::new());
                } else {
                    self.mark_player_failed(
                        player_id,
                        &format!(
                            "received transition event {:?} but not all required events satisfied",
                            event_type
                        ),
                        ctx,
                    );
                    return;
                }
            }
            EventKind::Invalid => {
                // Special case: PlayerReEnqueued는 InQueue phase에서 무시 (정상적인 re-enqueue)
                if current_phase == Phase::InQueue && *event_type == EventType::PlayerReEnqueued {
                    info!(
                        "Test [{}] Player {} received PlayerReEnqueued in InQueue phase (re-enqueued after failed match), ignoring",
                        self.test_name, player_id
                    );
                    return;
                }

                self.mark_player_failed(
                    player_id,
                    &format!(
                        "received unexpected event {:?} in phase {:?}",
                        event_type, current_phase
                    ),
                    ctx,
                );
                return;
            }
        }

        self.check_all_players_finished(ctx);
    }

    fn handle_global_event(&mut self, event: EventStreamMessage, _ctx: &mut actix::Context<Self>) {
        match event.event_type {
            EventType::GlobalQueueSizeChanged => {
                info!(
                    "Test [{}] Global event: queue_size_changed - {:?}",
                    self.test_name, event.data
                );

                // GlobalQueueSizeChanged를 현재 InQueue Phase의 모든 플레이어에게 적용
                let players_in_queue: Vec<Uuid> = self
                    .players_phase
                    .iter()
                    .filter(|(_, phase)| **phase == Phase::InQueue)
                    .map(|(player_id, _)| *player_id)
                    .collect();

                for player_id in players_in_queue {
                    // 해당 플레이어의 InQueue Phase 조건 확인
                    if let Some(schedule) = self.players_schedule.get(&player_id) {
                        if let Some(condition) = schedule.get(&Phase::InQueue) {
                            // required_events에서 GlobalQueueSizeChanged의 인덱스 찾기
                            if let Some((idx, _)) = condition
                                .required_events
                                .iter()
                                .enumerate()
                                .find(|(_, req)| {
                                    req.event_type == EventType::GlobalQueueSizeChanged
                                })
                            {
                                // player_satisfied_requirements에 등록
                                self.player_satisfied_requirements
                                    .entry(player_id)
                                    .or_default()
                                    .insert(idx);

                                info!(
                                    "Test [{}] Player {} satisfied GlobalQueueSizeChanged requirement in InQueue phase",
                                    self.test_name, player_id
                                );
                            }
                        }
                    }
                }
            }
            _ => {
                // Unknown global event
            }
        }
    }

    fn check_all_players_finished(&mut self, ctx: &mut actix::Context<Self>) {
        if self.players_phase.is_empty() {
            return;
        }

        // 실패한 플레이어가 있는지 먼저 확인
        let has_failed = self
            .players_phase
            .values()
            .any(|phase| *phase == Phase::Failed);

        if has_failed {
            warn!(
                "✗ Test [{}] failed - at least one player is in Failed state.",
                self.test_name
            );
            // 실패 신호 전송
            if let Some(tx) = self.completion_tx.take() {
                let _ = tx.send(false);
            }
            ctx.stop();
            actix::System::current().stop();
            return;
        }

        // 모든 플레이어가 Phase 기준으로 완료 (Matched, Dequeued, Finished)
        let all_phases_finished = self
            .players_phase
            .values()
            .all(|phase| matches!(phase, Phase::Matched | Phase::Dequeued | Phase::Finished));

        if all_phases_finished {
            let duration = self.started_at.elapsed();
            info!(
                "✓ Test [{}] All players finished successfully. Duration: {:?}",
                self.test_name, duration
            );
            info!("  - Completed players: {}", self.players_phase.len());
            // 성공 신호 전송
            if let Some(tx) = self.completion_tx.take() {
                let _ = tx.send(true);
            }
            ctx.stop();
            actix::System::current().stop();
        }
    }
}
