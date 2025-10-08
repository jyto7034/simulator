use actix::{ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture};
use futures_util::StreamExt;
use std::collections::HashSet;
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{debug, error, info, warn};

use crate::observer_actor::{
    message::{
        EventType, InternalEvent, ObservationCompleted, PlayerFinishedFromActor,
        SetSingleScenarioAddr, StartObservation, StopObservation,
    },
    EventStreamMessage, ObservationResult, ObserverActor, Phase,
};

impl Handler<StartObservation> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: StartObservation, ctx: &mut Self::Context) {
        info!("[{}] Starting event observation...", self.test_name);

        // 관찰 시작 시, 모든 플레이어를 초기 단계(Matching)로 설정
        for player_id in &msg.player_ids {
            self.players_phase.insert(*player_id, Phase::Matching);
            self.player_received_events_in_phase
                .insert(*player_id, HashSet::new());
        }

        // 기본 스트림 URL에 선택적으로 kind 필터를 추가(환경 변수로 제어)
        // Build stream URL with optional filters from env (kind, game_mode, session_id, event_type)
        let mut params: Vec<String> = Vec::new();

        // test_session_id는 항상 포함
        params.push(format!("session_id={}", self.test_session_id));

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
        // 환경 변수로도 session_id를 오버라이드할 수 있음 (테스트용)
        if let Ok(sid) = std::env::var("OBSERVER_FILTER_SESSION_ID") {
            if !sid.is_empty() {
                params.push(format!("session_id={}", sid));
            }
        }
        if let Ok(et) = std::env::var("OBSERVER_FILTER_EVENT_TYPE") {
            if !et.is_empty() {
                params.push(format!("event_type={}", et));
            }
        }
        if let Ok(run_id) = std::env::var("OBSERVER_RUN_ID") {
            if !run_id.is_empty() {
                params.push(format!("run_id={}", run_id));
            }
        }
        let url = if params.is_empty() {
            format!("{}/events/stream", self.match_server_url)
        } else {
            format!(
                "{}/events/stream?{}",
                self.match_server_url,
                params.join("&")
            )
        };

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

        // 스웜 환경에서 대량 이벤트를 수용하기 위해 메일박스 용량 증가
        ctx.set_mailbox_capacity(50_000);

        ctx.spawn(actor_future);
    }
}

impl actix::Handler<PlayerFinishedFromActor> for ObserverActor {
    type Result = ();
    fn handle(&mut self, msg: PlayerFinishedFromActor, ctx: &mut Self::Context) {
        // 플레이어 이벤트 기반 검증 기능을 살리되
        // 플레이어의 실패는 테스트 실패로 즉시 처리 되어야함
        // 처음에 작성해둔 명세가 타이트 하지 않았음, 플레이어의 실패는 의도된 행동이기 때문에 Ok() 로 처리하자고 했는데
        // 실제로 Ok() 로 처리를 하니 테스트가 timeout 되는 경우가 발생 ( 플레이어 한 쪽이 실패했는데 Failed 가 아닌 Ok() 를 반환해서 다른 한 쪽이 끝날 때까지 대기. )
        let current_phase = self.players_phase.get(&msg.player_id);

        match msg.result {
            // Player reported a failure - wait for phase validation via events
            Err(ref failure) => {
                // Phase 검증: Failed 또는 Finished가 될 때까지 대기
                match current_phase {
                    Some(&Phase::Failed) | Some(&Phase::Finished) => {
                        info!(
                            "[{}] Player {} failure confirmed via phase validation (phase: {:?})",
                            self.test_name, msg.player_id, current_phase
                        );
                        self.check_all_players_finished(ctx);
                    }
                    _ => {
                        warn!(
                            "[{}] Player {} reported failure: {:?}, but phase is {:?}, waiting for phase validation via events",
                            self.test_name, msg.player_id, failure, current_phase
                        );
                    }
                }
            }

            // Player reported success - wait for phase validation via events
            Ok(_) => {
                // Phase 검증: Finished가 될 때까지 대기
                if current_phase != Some(&Phase::Finished) {
                    warn!(
                        "[{}] Player {} reported completion but phase is {:?}, waiting for phase validation via events",
                        self.test_name, msg.player_id, current_phase
                    );
                    return;
                }

                info!(
                    "[{}] Player {} confirmed finished via phase validation",
                    self.test_name, msg.player_id
                );
                self.check_all_players_finished(ctx);
            }
        }
    }
}

impl actix::Handler<StopObservation> for super::ObserverActor {
    type Result = ();
    fn handle(&mut self, _msg: StopObservation, ctx: &mut Self::Context) {
        ctx.stop();
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
                        // events:* 스타일의 상태 이벤트는 값 파싱으로 캐시 갱신만 시도
                        if let Ok(evt) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(t) = evt.get("event").and_then(|v| v.as_str()) {
                                if t == "queue_size_changed" {
                                    if let (Some(mode), Some(size)) = (
                                        evt.get("game_mode")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string()),
                                        evt.get("size").and_then(|v| v.as_i64()),
                                    ) {
                                        self.last_queue_size
                                            .insert(mode, (size, chrono::Utc::now()));
                                    }
                                }
                            }
                        } else {
                            error!("Failed to parse event stream message");
                        }
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

// 내부 이벤트 메시지를 처리하여 검증 로직을 수행하는 핸들러
impl Handler<InternalEvent> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: InternalEvent, ctx: &mut Self::Context) {
        let event = msg.0;
        self.process_state_event(&event);
        debug!("[{}] Received event: {:?}", self.test_name, event);
        if let Some(pid) = event.player_id {
            if !self.players_phase.contains_key(&pid) {
                match event.event_type {
                    EventType::QueueSizeChanged
                    | EventType::PlayersRequeued
                    | EventType::ServerMessage
                    | EventType::Error
                    | EventType::StateViolation => {}
                    _ => {
                        return;
                    }
                }
            }
        }

        // 링버퍼 append
        if self.received_events.len() == self.max_events_kept {
            self.received_events.pop_front();
        }
        self.received_events.push_back(event.clone());

        if let Some(player_id) = event.player_id {
            self.check_phase_completion(player_id, &event, ctx);
        } else {
            match event.event_type {
                EventType::QueueSizeChanged | EventType::PlayersRequeued => {
                    let player_ids: Vec<_> = self.players_phase.keys().cloned().collect();
                    for player_id in &player_ids {
                        self.player_received_events_in_phase
                            .entry(*player_id)
                            .or_default()
                            .insert(event.event_type.clone());
                    }
                    info!(
                        "[{}] Global event {:?} broadcasted to {} players",
                        self.test_name,
                        event.event_type,
                        player_ids.len()
                    );

                    for player_id in player_ids {
                        self.check_phase_completion(player_id, &event, ctx);
                    }
                }
                EventType::StateViolation => {
                    // 서버 불변식 위반은 스웜 검증에서 즉시 실패로 처리
                    let code = event
                        .data
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    warn!(
                        "[{}] State violation observed: {} — failing scenario",
                        self.test_name, code
                    );
                    let keys: Vec<_> = self.players_phase.keys().cloned().collect();
                    for pid in keys {
                        self.players_phase
                            .insert(pid, crate::observer_actor::Phase::Failed);
                    }
                    self.check_all_players_finished(ctx);
                }
                _ => {}
            }
        }
    }
}

impl ObserverActor {
    /// 플레이어의 현재 단계(Phase)의 완료 조건을 확인하고, 충족 시 다음 단계로 전환합니다.
    fn check_phase_completion(
        &mut self,
        player_id: uuid::Uuid,
        event: &EventStreamMessage,
        ctx: &mut actix::Context<Self>,
    ) {
        let event_type = &event.event_type;
        let data = &event.data;

        // 치명적 에러가 클라이언트에 전달되면 해당 플레이어를 종료로 전환
        if *event_type == EventType::Error {
            if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
                let m = msg.to_lowercase();
                if m.contains("failed") && m.contains("attempt") {
                    self.players_phase.insert(player_id, Phase::Failed);
                    self.player_received_events_in_phase
                        .insert(player_id, HashSet::new());
                    self.check_all_players_finished(ctx);
                    return;
                }
            }
            // 그 외 에러 메시지는 테스트 상 종료로 간주 (성공/실패 판단은 상위에서 수행)
            self.players_phase.insert(player_id, Phase::Finished);
            self.player_received_events_in_phase
                .insert(player_id, HashSet::new());
            self.check_all_players_finished(ctx);
            return;
        }

        let current_phase = match self.players_phase.get(&player_id) {
            Some(phase) => phase.clone(),
            None => {
                warn!(
                    "[{}] Received event for untracked player {}",
                    self.test_name, player_id
                );
                return;
            }
        };

        // 현재 단계에서 받은 이벤트로 기록
        self.player_received_events_in_phase
            .entry(player_id)
            .or_default()
            .insert(event_type.clone());

        // 현재 단계의 완료 조건 가져오기 - 플레이어별 스케줄에서 찾기
        if let Some(player_schedule) = self.players_schedule.get(&player_id) {
            if let Some(condition) = player_schedule.get(&current_phase) {
                // 전환 이벤트가 발생했는지 확인
                if *event_type == condition.transition_event {
                    let received_events = self
                        .player_received_events_in_phase
                        .get(&player_id)
                        .unwrap();

                    // 필수 이벤트들을 모두 받았는지 확인
                    if condition.required_events.is_subset(received_events) {
                        // Matcher가 존재하면 실행하고, 없으면 통과로 간주
                        let matcher_passed = condition
                            .transition_matcher
                            .as_ref()
                            .map_or(true, |matcher| matcher(data));

                        if matcher_passed {
                            // --- 단계 전환 ---
                            let next_phase = condition.next_phase.clone();
                            info!(
                                "[{}] Player {} completed phase {:?} -> transitioning to {:?}",
                                self.test_name, player_id, current_phase, next_phase
                            );
                            self.players_phase.insert(player_id, next_phase);
                            self.player_received_events_in_phase
                                .insert(player_id, HashSet::new()); // 다음 단계를 위해 초기화
                        } else {
                            warn!(
                                "[{}] Player {} failed matcher for event {:?}",
                                self.test_name, player_id, event_type
                            );
                            // 매처 실패 시 Failed 상태로 전환
                            self.players_phase.insert(player_id, Phase::Failed);
                        }
                    }
                }
            }
        }

        // 모든 플레이어가 최종 단계에 도달했는지 확인
        self.check_all_players_finished(ctx);
    }

    /// 모든 플레이어가 terminal state에 도달했는지 확인합니다.
    /// - Expected failures: Failed 상태가 예상됨
    /// - Normal players: Finished 상태가 예상됨
    fn check_all_players_finished(&mut self, ctx: &mut actix::Context<Self>) {
        if self.players_phase.is_empty() {
            return;
        }

        // 1. Check for unexpected failures (failures not in expected_failures set)
        let unexpected_failures: Vec<_> = self
            .players_phase
            .iter()
            .filter(|(player_id, phase)| {
                **phase == Phase::Failed && !self.expected_failures.contains(player_id)
            })
            .map(|(id, _)| *id)
            .collect();

        if !unexpected_failures.is_empty() {
            warn!(
                "✗ [{}] Scenario failed - unexpected failures detected: {:?}",
                self.test_name, unexpected_failures
            );

            // SingleScenarioActor에게 실패 결과 전송
            if let Some(single_scenario_addr) = &self.single_scenario_addr {
                let failure_reason = format!(
                    "Unexpected player failures: {} player(s)",
                    unexpected_failures.len()
                );
                single_scenario_addr.do_send(ObservationCompleted(ObservationResult::Error {
                    failed_step: 0,
                    reason: failure_reason,
                    events: self.received_events.clone().into(),
                }));
            }
            ctx.stop();
            return;
        }

        // 2. Check if all players reached terminal state
        // Terminal states:
        // - Finished (for normal players)
        // - Failed (only if expected)
        let all_terminal = self.players_phase.iter().all(|(player_id, phase)| {
            *phase == Phase::Finished
                || (*phase == Phase::Failed && self.expected_failures.contains(player_id))
        });

        if all_terminal {
            info!(
                "✓ [{}] All players reached terminal state - test complete.",
                self.test_name
            );

            // SingleScenarioActor에게 성공 결과 전송
            if let Some(single_scenario_addr) = &self.single_scenario_addr {
                let duration = self.started_at.elapsed();

                single_scenario_addr.do_send(ObservationCompleted(ObservationResult::Success {
                    events: self.received_events.clone().into(),
                    duration,
                }));
            }
            ctx.stop();
        }
    }
}

impl Handler<SetSingleScenarioAddr> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: SetSingleScenarioAddr, _ctx: &mut Self::Context) -> Self::Result {
        self.single_scenario_addr = Some(msg.addr);
        info!("[{}] SingleScenarioActor address set", self.test_name);
    }
}
