use actix::Handler;
use serde_json::json;
use tracing::info;

use super::messages::*;
use super::MatchCoordinator;
use crate::matchmaking::matchmaker::messages::Enqueue;

impl Handler<EnqueuePlayer> for MatchCoordinator {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: EnqueuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        // 1. GameMode에 맞는 Matchmaker 찾기
        let matchmaker = self
            .matchmakers
            .get(&msg.game_mode)
            .ok_or_else(|| format!("Unsupported game mode: {:?}", msg.game_mode))?;

        // 2. 서버에서 metadata 생성 (중요!)
        let pod_id = std::env::var("POD_ID").unwrap_or_else(|_| "default-pod".to_string());
        let metadata = json!({
            "pod_id": pod_id,
            "player_id": msg.player_id.to_string(),
            // 추가 서버 측 데이터...
        });

        // 3. Matchmaker에 Enqueue 요청
        matchmaker.do_send_enqueue(Enqueue {
            player_id: msg.player_id,
            game_mode: msg.game_mode,
            metadata: metadata.to_string(),
        });

        info!("Player {} enqueued for {:?}", msg.player_id, msg.game_mode);
        Ok(())
    }
}

impl Handler<DequeuePlayer> for MatchCoordinator {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: DequeuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        let matchmaker = self
            .matchmakers
            .get(&msg.game_mode)
            .ok_or_else(|| format!("Unsupported game mode: {:?}", msg.game_mode))?;

        matchmaker.do_send_dequeue(crate::matchmaking::matchmaker::messages::Dequeue {
            player_id: msg.player_id,
            game_mode: msg.game_mode,
        });

        info!("Player {} dequeued from {:?}", msg.player_id, msg.game_mode);
        Ok(())
    }
}
