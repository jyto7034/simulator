use actix::{ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite};
use tracing::{error, info, warn};

use crate::observer_actor::{
    message::{ExpectEvent, InternalEvent, StartObservation},
    EventStreamMessage, ObserverActor,
};

impl Handler<ExpectEvent> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: ExpectEvent, _ctx: &mut Self::Context) {
        let player_id = msg.player_id.expect("ExpectEvent must have a player_id for per-player tracking");
        
        info!(
            "[{}] Received expectation for player {}: {:?}",
            self.test_name, player_id, msg.event_type
        );
        
        // 해당 플레이어의 기대 이벤트 목록에 추가
        self.player_expectations
            .entry(player_id)
            .or_insert_with(Vec::new)
            .push(msg);
            
        // 플레이어 step 초기화 (아직 없다면)
        self.player_steps.entry(player_id).or_insert(0);
    }
}

impl Handler<StartObservation> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: StartObservation, ctx: &mut Self::Context) {
        info!("[{}] Starting event observation...", self.test_name);

        let url = if let Some(pid) = msg.player_id_filter {
            format!("{}/events/stream?player_id={}", self.match_server_url, pid)
        } else {
            format!("{}/events/stream", self.match_server_url)
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
                        // 받은 이벤트를 내부 메시지로 변환하여 자신에게 보냄
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

        // 이벤트에 player_id가 있는 경우만 처리
        if let Some(event_player_id) = event.player_id {
            self.check_player_expectations(event_player_id, &event, ctx);
        }
        
        // 모든 플레이어의 기대 이벤트가 완료되었는지 확인
        self.check_all_players_completed(ctx);
    }
}

impl ObserverActor {
    /// 특정 플레이어의 기대 이벤트를 확인
    fn check_player_expectations(&mut self, player_id: uuid::Uuid, event: &EventStreamMessage, _ctx: &mut actix::Context<Self>) {
        // 해당 플레이어의 기대 이벤트 목록과 현재 step 가져오기
        let player_expectations = match self.player_expectations.get(&player_id) {
            Some(expectations) => expectations,
            None => return, // 이 플레이어에 대한 기대 이벤트가 없음
        };
        
        let current_step = *self.player_steps.get(&player_id).unwrap_or(&0);
        
        // 현재 step에 해당하는 기대 이벤트가 있는지 확인
        if current_step >= player_expectations.len() {
            // 이미 모든 기대 이벤트를 완료한 플레이어
            return;
        }
        
        let expected_event = &player_expectations[current_step];
        
        // 이벤트가 기대와 일치하는지 확인
        if expected_event.matches(event) {
            info!(
                "✓ [{}] Player {} step {} matched: {}",
                self.test_name, player_id, current_step, event.event_type
            );
            
            // 해당 플레이어의 step 증가
            self.player_steps.insert(player_id, current_step + 1);
        } else {
            warn!(
                "Event doesn't match expected for player {}: {:?}",
                player_id, event
            );
        }
    }
    
    /// 모든 플레이어의 기대 이벤트가 완료되었는지 확인
    fn check_all_players_completed(&self, ctx: &mut actix::Context<Self>) {
        let mut all_completed = true;
        
        for (player_id, expectations) in &self.player_expectations {
            let current_step = *self.player_steps.get(player_id).unwrap_or(&0);
            if current_step < expectations.len() {
                all_completed = false;
                break;
            }
        }
        
        if all_completed && !self.player_expectations.is_empty() {
            info!("[{}] All players completed their expected events.", self.test_name);
            // 모든 시나리오 완료. 성공 메시지 전송
            // ctx.address().do_send(ObservationCompleted(ObservationResult::Success { ... }));
            ctx.stop();
        }
    }
}
