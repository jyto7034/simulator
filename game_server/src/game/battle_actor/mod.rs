use tracing::info;

use crate::{
    matchmaking::matchmaker::operations::try_match::PlayerCandidate,
    GameMode,
};

pub mod messages;
pub mod simulator;

/// Battle 결과 데이터
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BattleResult {
    pub winner_id: String,
    pub battle_data: Option<serde_json::Value>,
}

/// Battle 실행 로직 (순수 함수)
pub async fn execute_battle(
    player1: &PlayerCandidate,
    player2: &PlayerCandidate,
    game_mode: GameMode,
) -> BattleResult {
    info!(
        "Executing battle: {} vs {} (mode: {:?})",
        player1.player_id, player2.player_id, game_mode
    );

    // Battle 시뮬레이션 실행
    let winner_id = simulate_battle(player1, player2).await;

    info!(
        "Battle completed: {} vs {}, winner: {}",
        player1.player_id, player2.player_id, winner_id
    );

    BattleResult {
        winner_id,
        battle_data: Some(serde_json::json!({
            "mode": format!("{:?}", game_mode),
        })),
    }
}

/// Battle 시뮬레이션 로직 (승자 결정)
async fn simulate_battle(player1: &PlayerCandidate, player2: &PlayerCandidate) -> String {
    // TODO: 실제 battle 로직 구현
    // 임시로 player1을 승자로 반환
    info!(
        "Simulating battle (stub): {} vs {}",
        player1.player_id, player2.player_id
    );
    player1.player_id.clone()
}
