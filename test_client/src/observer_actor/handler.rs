use actix::{ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture};
use futures_util::StreamExt;
use std::collections::HashSet;
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{error, info};

use crate::observer_actor::{
    message::{InternalEvent, StartObservation},
    EventStreamMessage, ObserverActor, Phase,
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

        let url = format!("{}/events/stream", self.match_server_url);

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
                    Err(e) => {
                        error!("Failed to parse event stream message: {}", e);
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
        info!("[{}] Received event: {:?}", self.test_name, event);
        self.received_events.push(event.clone());

        if let Some(player_id) = event.player_id {
            self.check_phase_completion(player_id, &event, ctx);
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
        // let event_type = &event.event_type;
        // let data = &event.data;

        // let current_phase = match self.players_phase.get(&player_id) {
        //     Some(phase) => phase.clone(),
        //     None => {
        //         warn!(
        //             "[{}] Received event for untracked player {}",
        //             self.test_name, player_id
        //         );
        //         return;
        //     }
        // };

        // // 현재 단계에서 받은 이벤트로 기록
        // self.player_received_events_in_phase
        //     .entry(player_id)
        //     .or_default()
        //     .insert(event_type.clone());

        // 현재 단계의 완료 조건 가져오기
        // if let Some(condition) = self.scenario_schedule.get(&current_phase) {
        //     // 전환 이벤트가 발생했는지 확인
        //     if *event_type == condition.transition_event {
        //         let received_events = self
        //             .player_received_events_in_phase
        //             .get(&player_id)
        //             .unwrap();

        //         // 필수 이벤트들을 모두 받았는지 확인
        //         if condition.required_events.is_subset(received_events) {
        //             // Matcher가 존재하면 실행하고, 없으면 통과로 간주
        //             let matcher_passed = condition
        //                 .transition_matcher
        //                 .as_ref()
        //                 .map_or(true, |matcher| matcher(data));

        //             if matcher_passed {
        //                 // --- 단계 전환 ---
        //                 let next_phase = condition.next_phase.clone();
        //                 info!(
        //                     "[{}] Player {} completed phase {:?} -> transitioning to {:?}",
        //                     self.test_name, player_id, current_phase, next_phase
        //                 );
        //                 self.players_phase.insert(player_id, next_phase);
        //                 self.player_received_events_in_phase
        //                     .insert(player_id, HashSet::new()); // 다음 단계를 위해 초기화
        //             } else {
        //                 warn!(
        //                     "[{}] Player {} failed matcher for event {:?}",
        //                     self.test_name, player_id, event_type
        //                 );
        //                 // TODO: 실패 처리 로직 추가
        //             }
        //         }
        //     }
        // }

        // 모든 플레이어가 최종 단계에 도달했는지 확인
        self.check_all_players_finished(ctx);
    }

    /// 모든 플레이어가 Finished 상태에 도달했는지 확인합니다.
    fn check_all_players_finished(&self, ctx: &mut actix::Context<Self>) {
        if self.players_phase.is_empty() {
            return;
        }

        let all_finished = self
            .players_phase
            .values()
            .all(|phase| *phase == Phase::Finished);

        if all_finished {
            info!(
                "✓ [{}] All players have finished the scenario successfully.",
                self.test_name
            );
            // TODO: 성공 결과 전송
            ctx.stop();
        }

        // TODO: 실패 조건 처리 (e.g., 한 명이라도 Failed 상태가 되면 즉시 중단)
    }
}
