use std::time::Duration;

use actix::{ActorContext, AsyncContext, Context, Handler, Message, ResponseFuture};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    enums::CLIENT_TIMEOUT,
    exception::{GameError, StateError, ConnectionError, SystemError},
    game::{state::GamePhase, GameActor},
};

use super::GameEvent;

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RegisterConnection {
    pub player_id: Uuid,
    pub recipient: actix::Recipient<GameEvent>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleOpponentWaitTimer {
    // 기다리는 상대의 종류
    pub opponent_kind: PlayerKind,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelOpponentWaitTimer;

impl Handler<RegisterConnection> for GameActor {
    type Result = ResponseFuture<Result<(), GameError>>;

    fn handle(&mut self, msg: RegisterConnection, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "GAME ACTOR [{}]: Handling RegisterConnection for player {}",
            self.game_id, msg.player_id
        );

        let game_id_clone = self.game_id.clone();
        let player_id = msg.player_id;
        let connection_recipient = msg.recipient.clone();

        let game_state = self.game_state.clone();
        let game_actor_addr = ctx.address().clone();
        let players = self.players.clone();
        let connections = self.connections.clone();
        let gsm = self.game_state.clone();

        let player_kind = self.get_player_type_by_uuid(player_id);
        let opponent_kind = self.opponent_player_kind.clone();
        let opponent_wait_timer_handle = self.opponent_wait_timer_handle.clone();

        Box::pin(async move {
            // --- 0. 기존 연결 확인 및 connections 맵 업데이트 ---
            {
                let gsm = gsm.lock().await;
                if gsm.current_phase() == GamePhase::Aborted {
                    info!(
                        "GAME ACTOR [{}]: Game is already aborted. Rejecting connection for player {}.",
                        game_id_clone, player_id
                    );
                    return Err(GameError::State(StateError::GameAborted));
                }
            }

            {
                let mut connections_guard = connections.lock().await;

                if connections_guard.contains_key(&player_id) {
                    info!(
                        "GAME ACTOR [{}]: Player {} already has an active connection. Rejecting new connection.",
                        game_id_clone, player_id
                    );
                    return Err(GameError::Connection(ConnectionError::SessionExists(player_id)));
                }

                connections_guard.insert(player_id, connection_recipient.clone());
                info!(
                    "GAME ACTOR [{}]: Connection for player {} registered successfully. Total connections: {}",
                    game_id_clone, player_id, connections_guard.len()
                );
            }

            // --- 1. GameState 업데이트 ---
            let is_all_players_connected;
            let mut current_phase;
            {
                let mut gsm = game_state.lock().await;
                info!(
                    "GAME ACTOR [{}]: Game state locked for player {}",
                    game_id_clone, player_id
                );

                gsm.add_connected_player(player_kind);
                info!(
                    "GAME ACTOR [{}]: Player {} connection status updated in GameStateManager.",
                    game_id_clone, player_id
                );

                is_all_players_connected = gsm.is_all_players_connected();
                current_phase = gsm.current_phase();

                if current_phase == GamePhase::Initial && is_all_players_connected {
                    info!(
                        "GAME ACTOR [{}]: All players connected. Transitioning to Mulligan phase.",
                        game_id_clone
                    );
                    gsm.transition_to_phase(GamePhase::Mulligan);
                    current_phase = gsm.current_phase();
                }
            }

            // --- 2. WaitTimer 처리 ---
            if let Some(kind) = opponent_kind {
                if kind != player_kind {
                    warn!(
                        "GAME ACTOR [{}]: Opponent kind mismatch: expected {:?}, got {:?}.",
                        game_id_clone, kind, player_kind
                    );
                    return Err(GameError::Connection(ConnectionError::AuthenticationFailed("Player kind mismatch".to_string())));
                }

                if let Some(_) = opponent_wait_timer_handle {
                    info!(
                        "GAME ACTOR [{}]: Cancelling opponent wait timer for player {}.",
                        game_id_clone, kind
                    );
                    game_actor_addr.do_send(CancelOpponentWaitTimer);
                } else {
                    warn!(
                        "GAME ACTOR [{}]: No opponent wait timer to cancel for player {}.",
                        game_id_clone, kind
                    );
                    return Err(GameError::System(SystemError::Internal("Timer handle not found".to_string())));
                }
            }

            // --- 3. 게임 로직 진행 ---
            // TODO: callback 으로 바꾸면 좋을 것 같음.
            // 현재 RegisterConnection 핸들러가 두 개 이상의 책임을 가지고 있음
            // 1. 플레이어 연결 등록
            // 2. 멀리건 카드 전송
            // then 같은 메소드를 활용해서 분리하면 좋을듯.
            if current_phase == GamePhase::Mulligan && is_all_players_connected {
                info!(
                    "GAME ACTOR [{}]: Proceeding with Mulligan card distribution.",
                    game_id_clone
                );

                use crate::game::state::PlayerMulliganStatus;
                use crate::player::message::GetMulliganDealCards;

                for (player_identity, player_addr) in players.iter() {
                    let connection_addr = {
                        let connections_snapshot = connections.lock().await;
                        connections_snapshot
                            .get(&player_identity.id)
                            .ok_or_else(|| {
                                error!(
                                    "Connection not found for player {} in connections_snapshot",
                                    player_identity.id
                                );
                                GameError::System(SystemError::Internal("Connection not found".to_string()))
                            })?
                            .clone()
                    };

                    match player_addr.send(GetMulliganDealCards).await {
                        Ok(cards) => {
                            let card_uuids: Vec<Uuid> =
                                cards.iter().map(|c| c.get_uuid()).collect();
                            if let Err(e) = connection_addr
                                .send(GameEvent::SendMulliganDealCards {
                                    cards: card_uuids.clone(),
                                })
                                .await
                            {
                                warn!(
                                    "Failed to send mulligan cards to {}: {:?}",
                                    player_identity.id, e
                                );
                            } else {
                                info!(
                                    "Sent mulligan cards ({} count) to player {}",
                                    card_uuids.len(),
                                    player_identity.id
                                );
                                let mut gsm_update = game_state.lock().await;
                                gsm_update.update_player_mulligan_status(
                                    player_identity.kind,
                                    PlayerMulliganStatus::CardsDealt,
                                );
                            }
                        }
                        Err(mailbox_err) => {
                            error!(
                                "Mailbox error getting mulligan cards for {}: {:?}",
                                player_identity.id, mailbox_err
                            );
                        }
                    }
                }
            } else if current_phase != GamePhase::Initial {
                warn!(
                    "GAME ACTOR [{}]: Player {} connected, but not all players are ready or in an unexpected game phase: {:?}.",
                    game_id_clone, player_id, current_phase
                );
            } else if is_all_players_connected == false {
                info!(
                    "GAME ACTOR [{}]: Player {} connected, but not all players are ready. Waiting for others.",
                    game_id_clone, player_id
                );
                if current_phase == GamePhase::Aborted {
                    return Err(GameError::State(StateError::GameAborted));
                }
                game_actor_addr.do_send(HandleOpponentWaitTimer {
                    opponent_kind: player_kind.reverse(),
                });
            }

            Ok(())
        })
    }
}

impl Handler<HandleOpponentWaitTimer> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: HandleOpponentWaitTimer, ctx: &mut Context<Self>) {
        if self.opponent_wait_timer_handle.is_some() {
            warn!("Opponent wait timer already started. Ignoring new request.");
            return;
        }
        info!(
            "Starting opponent wait timer for first player: {}",
            msg.opponent_kind
        );

        self.opponent_player_kind = Some(msg.opponent_kind);
        let handle = ctx.run_later(Duration::from_secs(CLIENT_TIMEOUT), move |act, ctx_later| {
            // TODO: try_lock -> lock 변경 해야함
            if let Ok(mut gsm) = act.game_state.try_lock() {

                // 현재 GamePhase 가 Initial이고, 연결된 플레이어가 1명이며( count_connected_players ), 상대방이 미접속 상태인( is_player_connected_by_kind ) 경우
                if gsm.current_phase() == GamePhase::Initial
                    && gsm.count_connected_players() == 1
                    && gsm.is_player_connected_by_kind(msg.opponent_kind) == None
                {
                    warn!(
                        "GAME ACTOR [{}]: Opponent wait timeout for player {}. Aborting game.",
                        act.game_id,
                        msg.opponent_kind
                    );
                    gsm.transition_to_phase(GamePhase::Aborted);
                    ctx_later.stop();

                } else if gsm.current_phase() == GamePhase::Aborted {
                    info!(
                        "GAME ACTOR [{}]: OpponentWaitTimeout for player {} triggered, but situation already resolved.",
                        act.game_id, msg.opponent_kind
                    );
                    gsm.transition_to_phase(GamePhase::AlreadyCancelled);
                } else {
                    warn!(
                        "GAME ACTOR [{}]: Opponent wait timeout for player {} but game is in unexpected phase: {:?}.",
                        act.game_id, msg.opponent_kind, gsm.current_phase()
                    );
                    gsm.transition_to_phase(GamePhase::UnexpectedGamePhase);
                    ctx_later.stop();
                }

            } else {
                error!(
                    "GAME ACTOR [{}]: Failed to lock game state during opponent wait timeout logic.",
                    act.game_id
                );
                act.unexpected_stop = true;
                ctx_later.stop();
            }
        });

        self.opponent_wait_timer_handle = Some(handle);
    }
}

// 이미 접속된 플레이어에게 게임 종료 알림 전송
// for (player_identity, player_addr) in act.players.iter() {
//     if let Some(connection) = act.connections.lock().await.get(&player_identity.id) {
//         if let Err(e) = connection.send(GameEvent::GameAborted).await {
//             warn!(
//                 "Failed to send GameAborted event to player {}: {:?}",
//                 player_identity.id, e
//             );
//         } else {
//             info!(
//                 "Sent GameAborted event to player {}",
//                 player_identity.id
//             );
//         }
//     } else {
//         warn!(
//             "No connection found for player {} when sending GameAborted event.",
//             player_identity.id
//         );
//     }
// }

impl Handler<CancelOpponentWaitTimer> for GameActor {
    type Result = ();

    fn handle(&mut self, _msg: CancelOpponentWaitTimer, ctx: &mut Context<Self>) {
        if let Some(handle) = self.opponent_wait_timer_handle.take() {
            info!("Cancelling opponent wait timer.");
            if ctx.cancel_future(handle) {
                info!("Opponent wait timer cancelled successfully.");
            } else {
                warn!("Failed to cancel opponent wait timer, it may have already been cancelled or expired.");
            }
            self.opponent_player_kind = None;
            self.opponent_wait_timer_handle = None;
        } else {
            info!("No opponent wait timer to cancel, or already cancelled.");
        }
    }
}
