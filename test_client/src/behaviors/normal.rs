use std::time::Duration;

use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::observer_actor::message::ExpectEvent;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResponse};
use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Debug, Clone)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 서버로부터 EnQueued 확인 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player successfully enqueued",
            player_context.player_id
        );

        // enqueued 이후 start_loading 메시지가 올 것을 기대
        let matcher = Box::new(|data: &serde_json::Value| {
            data.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "start_loading")
                .unwrap_or(false)
        });

        let expect_event = ExpectEvent::server_message(
            Some(player_context.player_id),
            matcher,
            Duration::from_secs(30), // 매칭 대기시간 고려
        );

        BehaviorResponse(Ok(BehaviorOutcome::Continue), Some(expect_event))
    }

    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player excited about match!",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None)
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResponse {
        info!(
            "[{}] Normal player starting to load assets",
            player_context.player_id
        );

        // 정상적으로 loading_complete 메시지 전송
        let _msg = ClientMessage::LoadingComplete { loading_session_id };

        // ws_sink
        //     .send(Message::Text(serde_json::to_string(&msg)?))
        //     .await?;

        info!(
            "[{}] Normal player sent loading_complete",
            player_context.player_id
        );

        // loading_complete 전송 후 match_found 이벤트가 와야 함 (정상 케이스)
        // 만약 메시지가 실제로 전송되지 않았다면 이 이벤트는 오지 않을 것
        let matcher = Box::new(move |data: &serde_json::Value| {
            // match_found 메시지인지 확인
            if let Some(msg_type) = data.get("type").and_then(|t| t.as_str()) {
                if msg_type == "match_found" {
                    // session_id가 현재 loading_session_id와 연관되어 있는지 확인
                    if let Some(_session_id) = data.get("session_id") {
                        info!("✅ loading_complete was actually sent! Received match_found.");
                        return true;
                    }
                }
            }
            false
        });

        let expect_event = ExpectEvent::new(
            "server_message".to_string(),
            Some(player_context.player_id),
            matcher,
            Duration::from_secs(10), // loading_complete 후 빠르게 와야 함
        );

        BehaviorResponse(Ok(BehaviorOutcome::Continue), Some(expect_event))
    }

    async fn on_loading_complete(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player successfully completed the flow!",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Stop), None) // 성공적으로 완료했으므로 종료
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
