use actix::Addr;
use redis::aio::ConnectionManager;
use tracing::info;

use crate::{
    game::{battle_actor, load_balance_actor::LoadBalanceActor},
    matchmaking::{
        matchmaker::{
            operations::{
                notify::{self, MessageRoutingDeps},
                try_match::PlayerCandidate,
            },
            MatchmakerDeps,
        },
        subscript::SubScriptionManager,
    },
    shared::{protocol::ServerMessage, redis_events},
    GameMode,
};

use metrics::{MATCHES_CROSS_POD_TOTAL, MATCHES_SAME_POD_TOTAL};

/// 매칭된 2명 처리 - Battle 실행 후 결과와 함께 MatchFound 전송
pub async fn process_match_pair(
    player1: &PlayerCandidate,
    player2: &PlayerCandidate,
    game_mode: GameMode,
    _queue_suffix: &str,
    deps: &MatchmakerDeps,
    redis: &mut ConnectionManager,
    subscription_addr: Addr<SubScriptionManager>,
    load_balance_addr: &Addr<LoadBalanceActor>,
) {
    info!(
        "Processing match pair: {} vs {} (mode: {:?})",
        player1.player_id, player2.player_id, game_mode
    );

    // Pod ID 가져오기
    let my_pod_id = PlayerCandidate::current_pod_id();

    // Battle 실행 (동기적으로 결과 대기)
    let battle_result = battle_actor::execute_battle(player1, player2, game_mode).await;

    // 플레이어들에게 MatchFound 알림 (battle_result 포함)
    notify_match_found_with_result(
        player1,
        player2,
        &battle_result,
        redis,
        subscription_addr,
        load_balance_addr,
        deps,
    )
    .await;

    // 메트릭 기록
    metrics::MATCHES_CREATED_TOTAL.inc();

    // Same-pod, Cross-pod 구분
    let both_same_pod = player1.pod_id == my_pod_id && player2.pod_id == my_pod_id;
    if both_same_pod {
        MATCHES_SAME_POD_TOTAL.inc();
        info!("Same-pod match completed");
    } else {
        MATCHES_CROSS_POD_TOTAL.inc();
        info!("Cross-pod match completed");
    }

    metrics::MATCHED_PLAYERS_TOTAL_BY_MODE
        .with_label_values(&[&format!("{:?}", game_mode)])
        .inc_by(2);

    // Redis 이벤트 발행
    publish_match_events(player1, player2, game_mode, redis).await;
}

/// Redis 이벤트 발행
async fn publish_match_events(
    player1: &PlayerCandidate,
    player2: &PlayerCandidate,
    game_mode: GameMode,
    redis: &mut ConnectionManager,
) {
    if let Ok(metadata1_str) = serde_json::to_string(&player1.metadata) {
        redis_events::try_publish_test_event(
            redis,
            &metadata1_str,
            "player.match_found",
            &player1.pod_id,
            vec![
                ("player_id", player1.player_id.clone()),
                ("opponent_id", player2.player_id.clone()),
                ("game_mode", format!("{:?}", game_mode)),
            ],
        )
        .await;
    }

    if let Ok(metadata2_str) = serde_json::to_string(&player2.metadata) {
        redis_events::try_publish_test_event(
            redis,
            &metadata2_str,
            "player.match_found",
            &player2.pod_id,
            vec![
                ("player_id", player2.player_id.clone()),
                ("opponent_id", player1.player_id.clone()),
                ("game_mode", format!("{:?}", game_mode)),
            ],
        )
        .await;
    }
}

/// 플레이어들에게 MatchFound 알림 (battle_result 포함)
async fn notify_match_found_with_result(
    player1: &PlayerCandidate,
    player2: &PlayerCandidate,
    battle_result: &battle_actor::BattleResult,
    redis: &mut ConnectionManager,
    subscription_addr: Addr<SubScriptionManager>,
    load_balance_addr: &Addr<LoadBalanceActor>,
    deps: &MatchmakerDeps,
) {
    // MessageRoutingDeps 생성
    let routing_deps = MessageRoutingDeps {
        subscription_addr: subscription_addr.clone(),
        load_balance_addr: Some(load_balance_addr.clone()),
        redis: redis.clone(),
        metrics: deps.metrics.clone(),
    };

    // Player 1에게 전송
    notify::send_message_to_player(
        player1,
        ServerMessage::MatchFound {
            winner_id: battle_result.winner_id.clone(),
            opponent_id: player2.player_id.clone(),
            battle_data: battle_result.battle_data.clone(),
        },
        &routing_deps,
    )
    .await;

    // Player 2에게 전송
    notify::send_message_to_player(
        player2,
        ServerMessage::MatchFound {
            winner_id: battle_result.winner_id.clone(),
            opponent_id: player1.player_id.clone(),
            battle_data: battle_result.battle_data.clone(),
        },
        &routing_deps,
    )
    .await;
}
