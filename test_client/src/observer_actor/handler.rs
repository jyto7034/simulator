use actix::{ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture};
use futures_util::StreamExt;
use std::collections::HashSet;
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{debug, error, info, warn};

use crate::observer_actor::{
    message::{InternalEvent, ObservationCompleted, SetSingleScenarioAddr, StartObservation},
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
        if let Ok(kind) = std::env::var("OBSERVER_STREAM_KIND") {
            if !kind.is_empty() { params.push(format!("kind={}", kind)); }
        }
        if let Ok(gm) = std::env::var("OBSERVER_FILTER_GAME_MODE") {
            if !gm.is_empty() { params.push(format!("game_mode={}", gm)); }
        }
        if let Ok(sid) = std::env::var("OBSERVER_FILTER_SESSION_ID") {
            if !sid.is_empty() { params.push(format!("session_id={}", sid)); }
        }
        if let Ok(et) = std::env::var("OBSERVER_FILTER_EVENT_TYPE") {
            if !et.is_empty() { params.push(format!("event_type={}", et)); }
        }
        if let Ok(run_id) = std::env::var("OBSERVER_RUN_ID") {
            if !run_id.is_empty() { params.push(format!("run_id={}", run_id)); }
        }
        let url = if params.is_empty() {
            format!("{}/events/stream", self.match_server_url)
        } else {
            format!("{}/events/stream?{}", self.match_server_url, params.join("&"))
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


impl actix::Handler<crate::observer_actor::message::PlayerFinishedFromActor> for ObserverActor {
    type Result = ();
    fn handle(&mut self, msg: crate::observer_actor::message::PlayerFinishedFromActor, ctx: &mut Self::Context) {
        // Mark this player as Finished in phase model
        self.players_phase.insert(msg.player_id, crate::observer_actor::Phase::Finished);
        self.player_received_events_in_phase.insert(msg.player_id, std::collections::HashSet::new());
        // For abnormal scenarios we consider the scenario concluded once any player finishes
        if let Some(single) = &self.single_scenario_addr {
            single.do_send(ObservationCompleted(ObservationResult::Success {
                events: self.received_events.clone().into(),
                duration: self.started_at.elapsed(),
            }));
        }
        ctx.stop();
    }
}

impl actix::Handler<crate::observer_actor::message::StopObservation> for super::ObserverActor {
    type Result = ();
    fn handle(&mut self, _msg: crate::observer_actor::message::StopObservation, ctx: &mut Self::Context) {
        // Stop the event stream by stopping the actor
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
                                        evt.get("game_mode").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
        // Process state events inside actor context (message-forwarded)
        self.process_state_event(&event);
        debug!("[{}] Received event: {:?}", self.test_name, event);
        // Shard-level filtering: drop unrelated player-scoped events
        if let Some(pid) = event.player_id {
            if !self.players_phase.contains_key(&pid) {
                // Allowlist non-player state events always pass through
                match event.event_type {
                    crate::observer_actor::message::EventType::QueueSizeChanged
                    | crate::observer_actor::message::EventType::DedicatedSessionFailed
                    | crate::observer_actor::message::EventType::LoadingSessionCreated
                    | crate::observer_actor::message::EventType::LoadingSessionCompleted
                    | crate::observer_actor::message::EventType::LoadingSessionTimeout
                    | crate::observer_actor::message::EventType::PlayersRequeued
                    | crate::observer_actor::message::EventType::LoadingSessionCanceled
                    | crate::observer_actor::message::EventType::ServerMessage
                    | crate::observer_actor::message::EventType::Error
                    | crate::observer_actor::message::EventType::StateViolation => {}
                    _ => {
                        // Drop unrelated per-player event
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
        } else if event.event_type == crate::observer_actor::message::EventType::DedicatedSessionFailed {
            // 최종 실패(reason=max_retries_exceeded)만 실패로 간주
            let reason = event
                .data
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            if reason.contains("max_retries_exceeded") {
                let keys: Vec<_> = self.players_phase.keys().cloned().collect();
                for pid in keys {
                    self.players_phase
                        .insert(pid, crate::observer_actor::Phase::Failed);
                }
                self.check_all_players_finished(ctx);
            } else {
                warn!(
                    "[{}] Non-final dedicated_session_failed observed: {}",
                    self.test_name, reason
                );
            }
        } else if event.event_type == crate::observer_actor::message::EventType::StateViolation {
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

        // 전용 서버 할당 실패와 같은 치명적 에러가 클라이언트에 전달되면 해당 플레이어를 종료로 전환
        if *event_type == crate::observer_actor::message::EventType::Error {
            if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
                let m = msg.to_lowercase();
                if m.contains("failed") && m.contains("attempt") {
                    self.players_phase
                        .insert(player_id, crate::observer_actor::Phase::Failed);
                    self.player_received_events_in_phase
                        .insert(player_id, std::collections::HashSet::new());
                    self.check_all_players_finished(ctx);
                    return;
                }
            }
            // 그 외 에러 메시지는 테스트 상 종료로 간주 (성공/실패 판단은 상위에서 수행)
            self.players_phase
                .insert(player_id, crate::observer_actor::Phase::Finished);
            self.player_received_events_in_phase
                .insert(player_id, std::collections::HashSet::new());
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

    /// 모든 플레이어가 Finished 상태에 도달했는지 확인합니다.
    fn check_all_players_finished(&self, ctx: &mut actix::Context<Self>) {
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
                "✗ [{}] Scenario failed - at least one player is in Failed state.",
                self.test_name
            );

            // SingleScenarioActor에게 실패 결과 전송
            if let Some(single_scenario_addr) = &self.single_scenario_addr {
                let failure_reason = format!("One or more players failed phase validation");
                single_scenario_addr.do_send(ObservationCompleted(ObservationResult::Error {
                    failed_step: 0,
                    reason: failure_reason,
                    events: self.received_events.clone().into(),
                }));
            }
            ctx.stop();
            return;
        }

        // 모든 플레이어가 완료했는지 확인
        let all_finished = self
            .players_phase
            .values()
            .all(|phase| *phase == Phase::Finished);

        if all_finished {
            info!(
                "✓ [{}] All players have finished the scenario successfully.",
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
