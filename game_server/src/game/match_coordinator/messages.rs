use crate::GameMode;
use actix::Message;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct EnqueuePlayer {
    pub player_id: Uuid,
    pub game_mode: GameMode,
    // 클라이언트가 보낸 데이터는 여기서 검증 후 metadata로 변환
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct DequeuePlayer {
    pub player_id: Uuid,
    pub game_mode: GameMode,
}
