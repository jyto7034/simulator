use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Client to Server Messages ---

#[derive(Deserialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 플레이어가 매칭 대기열에 들어가기를 요청합니다.
    #[serde(rename = "enqueue")]
    Enqueue { player_id: Uuid, game_mode: String },
    /// 클라이언트가 에셋 로딩을 완료했음을 서버에 알립니다.
    #[serde(rename = "loading_complete")]
    LoadingComplete { loading_session_id: Uuid },
}

// --- Server to Client Messages ---

#[derive(Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 대기열에 성공적으로 등록되었음을 알립니다.
    #[serde(rename = "enqueued")]
    EnQueued,

    /// 최종적으로 매칭이 성사되었고, 게임 서버 접속 정보를 전달합니다.
    #[serde(rename = "match_found")]
    MatchFound {
        session_id: Uuid, // dedicated_server의 게임 세션 ID
        server_address: String,
    },

    /// 클라이언트에게 에셋 로딩을 시작하라고 지시합니다.
    #[serde(rename = "start_loading")]
    StartLoading { loading_session_id: Uuid },

    /// 에러가 발생했음을 알립니다.
    #[serde(rename = "error")]
    Error { message: String },
}
