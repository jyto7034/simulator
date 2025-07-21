use actix::{
    ActorContext, ActorFutureExt, AsyncContext, Handler, StreamHandler, WrapFuture,
};
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
        info!(
            "[{}] Received expectation: {:?}",
            self.test_name,
            msg.event_type
        );
        self.expected_sequence.push(msg);
    }
}

impl Handler<StartObservation> for ObserverActor {
    type Result = ();

    fn handle(&mut self, msg: StartObservation, ctx: &mut Self::Context) {
        info!(
            "[{}] Starting event observation...",
            self.test_name
        );

        let url = if let Some(pid) = msg.player_id_filter {
            format!(
                "{}/events/stream?player_id={}",
                self.match_server_url,
                pid
            )
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
                info!("[{}] Successfully connected to event stream.", act.test_name);
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

        if self.current_step >= self.expected_sequence.len() {
            // 모든 예상을 완료했으면 더 이상 확인할 필요 없음
            return;
        }

        let current_expected = &self.expected_sequence[self.current_step];

        // TODO: `matches` 로직 복원. data_matcher를 어떻게 처리할지 결정해야 함.
        let is_match = current_expected.event_type == event.event_type
            && current_expected.player_id == event.player_id;

        if is_match {
            info!(
                "✓ [{}] Step {} matched: {}",
                self.test_name,
                self.current_step,
                event.event_type
            );
            self.current_step += 1;

            if self.current_step >= self.expected_sequence.len() {
                info!("[{}] All expected events received.", self.test_name);
                // 모든 시나리오 완료. 성공 메시지 전송
                // ctx.address().do_send(ObservationCompleted(ObservationResult::Success { ... }));
                ctx.stop();
            }
        } else {
            warn!("Event doesn't match expected: {:?}", event);
        }
    }
}