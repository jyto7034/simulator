use actix::{Addr, Context, Handler, Message, Recipient, ResponseFuture};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, take::RandomTake, types::PlayerKind, Card},
    enums::ZoneType,
    exception::GameError,
    game::{
        phase::PlayerPhaseProgress,
        state::{GamePhase, PlayerMulliganStatus},
    },
    player::{
        message::{
            AddCardsToDeck, GetCardFromDeck, GetDeckCards, GetFieldCards, GetGraveyardCards,
            GetHandCards, GetMulliganDealCards,
        },
        PlayerActor,
    },
    selector::TargetCount,
};

use super::{phase::Phase, GameActor, GameConfig};

#[derive(Message)]
#[rtype(result = "()")]
pub enum GameEvent {
    SendMulliganDealCards { cards: Vec<Uuid> },
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct InitializeGame(pub GameConfig);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPlayer {
    pub player_type: PlayerKind,
    pub player_addr: Addr<PlayerActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerReady(pub PlayerKind);

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestPlayCard {
    pub player_type: PlayerKind,
    pub card_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct SubmitInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct IsCorrectPhase {
    pub phase: Phase,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Card>, GameError>")]
pub struct RerollRequestMulliganCard {
    pub player_type: PlayerKind,
    pub cards: Vec<Uuid>,
}

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetPlayerZoneCards {
    pub player_type: PlayerKind,
    pub zone: ZoneType,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct CheckReEntry {
    pub player_type: PlayerKind,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RegisterConnection {
    pub player_id: Uuid,
    pub recipient: Recipient<GameEvent>,
}

impl Handler<RegisterConnection> for GameActor {
    type Result = ResponseFuture<Result<(), GameError>>;

    fn handle(&mut self, msg: RegisterConnection, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "GAME ACTOR [{}]: Handling RegisterConnection for player {}",
            self.game_id, msg.player_id
        );

        // async 블록에서 사용될 값들을 미리 클론하거나 준비합니다.
        let game_id_clone = self.game_id.clone();
        let player_id = msg.player_id;
        let connection_addr = msg.recipient.clone();

        let game_state = self.game_state.clone();
        let players = self.players.clone();
        let connections = self.connections.clone();

        let player_kind = self.get_player_type_by_uuid(player_id);

        Box::pin(async move {
            // --- 0. 기존 연결 확인 및 connections 맵 업데이트 (connections_map_arc 사용) ---
            {
                // Mutex 잠금 범위 시작
                let mut connections_guard = connections.lock().await;
                if connections_guard.contains_key(&player_id) {
                    info!(
                        "GAME ACTOR [{}]: Player {} already has an active connection. Rejecting new connection.",
                        game_id_clone, player_id
                    );
                    return Err(GameError::ActiveSessionExists);
                }
                connections_guard.insert(player_id, connection_addr.clone());
                info!(
                    "GAME ACTOR [{}]: Connection for player {} registered successfully. Total connections: {}",
                    game_id_clone, player_id, connections_guard.len()
                );
            } // Mutex drop

            // --- 1. GameState 업데이트 ---
            let is_all_players_connected;
            let mut current_phase; // mut로 변경
            {
                // Mutex 잠금 범위 시작
                let mut gsm = game_state.lock().await;
                info!(
                    "GAME ACTOR [{}]: Game state locked for player {}",
                    game_id_clone, player_id
                );

                gsm.update_player_connection_status(player_kind, true);
                info!(
                    "GAME ACTOR [{}]: Player {} connection status updated in GameStateManager.",
                    game_id_clone, player_id
                );

                is_all_players_connected = gsm.is_all_players_connected(); // is_로 변경 가정
                current_phase = gsm.current_phase(); // 초기 값 할당

                // 초기 페이즈 전환 로직 (모든 플레이어 연결 시)
                if current_phase == GamePhase::Initial && is_all_players_connected {
                    info!(
                        "GAME ACTOR [{}]: All players connected. Transitioning to Mulligan phase.",
                        game_id_clone
                    );
                    gsm.transition_to_phase(GamePhase::Mulligan);
                    current_phase = gsm.current_phase();
                }
            } // Mutex drop

            // --- 2. 게임 로직 진행 ---
            if current_phase == GamePhase::Mulligan && is_all_players_connected {
                info!(
                    "GAME ACTOR [{}]: Proceeding with Mulligan card distribution.",
                    game_id_clone
                );

                // players 와 connections (읽기 접근)

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
                                GameError::InternalServerError
                            })?
                            .clone()
                    }; // Mutex drop

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
                                // 멀리건 상태 업데이트 위해 다시 락
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
            } else if current_phase == GamePhase::Mulligan {
                // 이미 멀리건 중 재접속
                warn!(
                    "GAME ACTOR [{}]: Player {} reconnected during Mulligan phase.",
                    game_id_clone, player_id
                );
                // ... 재진입 로직 ...
            } else if current_phase != GamePhase::Initial {
                // 아직 모든 플레이어가 연결되지 않은 초기 상태가 아닐 때
                warn!(
                    "GAME ACTOR [{}]: Player {} connected, but not all players are ready or in an unexpected game phase: {:?}.",
                    game_id_clone, player_id, current_phase
                );
            }
            // current_phase_after_update가 GamePhase::Initial인데 all_players_connected_after_update가 false인 경우는
            // "Waiting for other players"에 해당하므로 별도 처리가 필요 없을 수 있음 (또는 알림 전송)

            Ok(())
        })
    }
}

impl Handler<InitializeGame> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: InitializeGame, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RegisterPlayer> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: RegisterPlayer, _: &mut Self::Context) -> Self::Result {}
}

impl Handler<PlayerReady> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: PlayerReady, _: &mut Self::Context) -> Self::Result {}
}

impl Handler<RequestPlayCard> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: RequestPlayCard, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<SubmitInput> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: SubmitInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RequestInput> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: RequestInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<IsCorrectPhase> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: IsCorrectPhase, _: &mut Self::Context) -> Self::Result {
        if self.phase_state.get_phase() == msg.phase {
            Ok(())
        } else {
            Err(GameError::WrongPhase)
        }
    }
}

impl Handler<GetPlayerZoneCards> for GameActor {
    type Result = ResponseFuture<Vec<Card>>;

    fn handle(&mut self, msg: GetPlayerZoneCards, _: &mut Context<Self>) -> Self::Result {
        let player_type = msg.player_type;
        let zone = msg.zone;

        let addr = self.get_player_addr_by_kind(player_type);
        Box::pin(async move {
            match zone {
                ZoneType::Deck => addr.send(GetDeckCards).await,
                ZoneType::Hand => addr.send(GetHandCards).await,
                ZoneType::Field => addr.send(GetFieldCards).await,
                ZoneType::Graveyard => addr.send(GetGraveyardCards).await,
                _ => panic!("Invalid zone type: {}", zone),
            }
            .unwrap()
        })
    }
}

impl Handler<RerollRequestMulliganCard> for GameActor {
    type Result = ResponseFuture<Result<Vec<Card>, GameError>>;

    fn handle(&mut self, msg: RerollRequestMulliganCard, _: &mut Context<Self>) -> Self::Result {
        let player_type = msg.player_type;

        let mut cards = vec![];
        let player_cards = self
            .all_cards
            .get(&player_type)
            .unwrap_or_else(|| panic!("Player cards not found for player type: {:?}", player_type));
        for uuid in msg.cards {
            if let Some(card) = player_cards.find_by_uuid(uuid.clone()) {
                cards.push(card.clone());
            } else {
                todo!()
                // return ResponseFuture:;
            }
        }
        let addr = self.get_player_addr_by_kind(player_type);
        Box::pin(async move {
            addr.do_send(AddCardsToDeck {
                cards,
                insert: Box::new(BottomInsert),
            });

            addr.send(GetCardFromDeck {
                take: Box::new(RandomTake(TargetCount::Exact(5))),
            })
            .await?
        })
    }
}

impl Handler<CheckReEntry> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: CheckReEntry, _: &mut Context<Self>) -> Self::Result {
        let current_phase = self.phase_state.get_phase();
        let player_progress = self.phase_state.get_player_progress(msg.player_type);

        info!(
            "GAME ACTOR: Handling CheckReEntry for {:?} in phase {:?}. Current progress: {:?}",
            msg.player_type, current_phase, player_progress
        );

        // 페이즈별 재진입 규칙 정의
        match current_phase {
            Phase::Mulligan => {
                        // 멀리건: Entered 상태가 아니면 재진입 불가 (이미 시작했거나 완료)
                        if player_progress != PlayerPhaseProgress::NotStarted && player_progress != PlayerPhaseProgress::Entered {
                             println!("GAME ACTOR: Re-entry denied for {:?} in Mulligan (Progress: {:?})", msg.player_type, player_progress);
                            Err(GameError::NotAllowedReEntry)
                        } else {
                             // 첫 진입 시 상태 변경
                             self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::Entered);
                            Ok(())
                        }
                    }
            Phase::DrawPhase => {
                        // 드로우: Entered 상태가 아니면 재진입 불가 (이미 드로우 했음)
                        if player_progress != PlayerPhaseProgress::Entered {
                            println!("GAME ACTOR: Re-entry denied for {:?} in DrawPhase (Progress: {:?})", msg.player_type, player_progress);
                            Err(GameError::NotAllowedReEntry)
                        } else {
                             // 드로우 액션 수행 후 상태를 ActionTaken 등으로 변경하는 로직 필요
                             // 이 핸들러는 진입 가능 여부만 확인하므로 상태 변경은 다른 곳에서
                             // self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::ActionTaken); // 예시: 드로우 직후 호출
                            Ok(())
                        }
                    }
            Phase::StandbyPhase => {
                        // 스탠바이: Entered 상태가 아니면 재진입 불가 (이미 처리 시작)
                        if player_progress != PlayerPhaseProgress::Entered {
                            println!("GAME ACTOR: Re-entry denied for {:?} in StandbyPhase (Progress: {:?})", msg.player_type, player_progress);
                            Err(GameError::NotAllowedReEntry)
                        } else {
                            // 스탠바이 처리 시작 시 상태 변경 필요
                            // self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::ActionTaken);
                            Ok(())
                        }
                    }
            Phase::MainPhaseStart | Phase::MainPhase1 | Phase::MainPhase2 => {
                        // 여기서는 단순화를 위해 항상 허용.
                        // 만약 "MainPhaseStart 시" 효과 발동 여부를 체크하려면 다른 상태 필요.
                        if player_progress == PlayerPhaseProgress::NotStarted { // 페이즈가 막 바뀐 직후라면 Entered로 설정
                             self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::Entered);
                        }
                        Ok(())
                    }
            Phase::BattlePhaseStart | Phase::BattleStep | // ... 등등 ...
                    Phase::BattlePhaseEnd => {
                        // 여기서는 단순화를 위해 항상 허용.
                        // 실제로는 공격 선언 후 BattlePhaseStart 재진입 불가 등의 규칙 필요.
                         if player_progress == PlayerPhaseProgress::NotStarted {
                             self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::Entered);
                         }
                        Ok(())
                    }
            Phase::EndPhase => {
                        // 엔드: Entered 상태가 아니면 재진입 불가 (턴 종료 처리 시작됨)
                        if player_progress != PlayerPhaseProgress::Entered {
                            println!("GAME ACTOR: Re-entry denied for {:?} in EndPhase (Progress: {:?})", msg.player_type, player_progress);
                            Err(GameError::NotAllowedReEntry)
                        } else {
                            // 엔드 페이즈 처리 시작 시 상태 변경 필요
                            // self.phase_state.update_player_progress(msg.player_type, PlayerPhaseProgress::ActionTaken);
                            Ok(())
                        }
                    }
            Phase::BattleDamageStepStart => todo!(),
            Phase::BattleDamageStepCalculationBefore => todo!(),
            Phase::BattleDamageStepCalculationStart => todo!(),
            Phase::BattleDamageStepCalculationEnd => todo!(),
            Phase::BattleDamageStepEnd => todo!(),
        }
    }
}

pub struct ChoiceCardRequestPayload {
    pub player: String,
    pub choice_type: String,

    pub source_card_id: Uuid,

    // 선택 제한 설정
    pub min_selections: usize, // 최소 선택 개수
    pub max_selections: usize, // 최대 선택 개수
    pub destination: String,

    // 상태 관리
    pub is_open: bool,                 // 선택이 활성화되어 있는지
    pub is_hidden_from_opponent: bool, // 상대방에게 숨김 여부
}
