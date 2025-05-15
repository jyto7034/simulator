use actix::{Addr, AsyncContext, Context, Handler, Message, ResponseFuture};
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, take::RandomTake, types::PlayerKind, Card},
    enums::ZoneType,
    exception::GameError,
    game::{phase::PlayerPhaseProgress, state::PlayerMulliganStatus},
    player::{
        message::{
            AddCardsToDeck, GetCardFromDeck, GetDeckCards, GetFieldCards, GetGraveyardCards,
            GetHandCards, GetMulliganDealCards,
        },
        PlayerActor,
    },
    selector::TargetCount,
    server::{
        actor::{
            connection::ConnectionActor, messages::SendMulliganDealCards,
            types::PlayerInputResponse, UserAction,
        },
        input_handler::{InputAnswer, InputRequest},
    },
    PlayerHashMapExt,
};

use super::{phase::Phase, GameActor, GameConfig};

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
    pub answer: InputAnswer,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
    pub request: InputRequest,
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

#[derive(Message, Deserialize, Debug, Clone)]
#[rtype(result = "Result<PlayerInputResponse, GameError>")]
pub struct HandleUserAction {
    pub player_id: Uuid,
    pub action: UserAction,
}

impl Handler<HandleUserAction> for GameActor {
    type Result = ResponseFuture<Result<PlayerInputResponse, GameError>>;

    fn handle(&mut self, msg: HandleUserAction, ctx: &mut Self::Context) -> Self::Result {
        match msg.action {
            UserAction::PlayCard { card_id, target_id } => {
                todo!()
            }
            UserAction::Attack {
                attacker_id,
                defender_id,
            } => todo!(),
            UserAction::EndTurn => todo!(),
            UserAction::SubmitInput {
                request_id,
                response_data,
            } => todo!(),
            UserAction::RerollRequestMulliganCard { card_id } => {
                let player_type = self.get_player_type_by_uuid(msg.player_id);
                let addr = ctx.address();
                Box::pin(async move {
                    let rerolled_cards = addr
                        .send(RerollRequestMulliganCard {
                            player_type,
                            cards: card_id.clone(),
                        })
                        .await??
                        .iter()
                        .map(|card| card.get_uuid())
                        .collect();

                    Ok(PlayerInputResponse::MulliganRerollAnswer(rerolled_cards))
                })
            }
            UserAction::CompleteMulligan => todo!(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RegisterConnection {
    pub player_id: Uuid,
    pub addr: Addr<ConnectionActor>,
}

impl Handler<RegisterConnection> for GameActor {
    type Result = ResponseFuture<Result<(), GameError>>;

    fn handle(&mut self, msg: RegisterConnection, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "GAME ACTOR [{}]: Handling RegisterConnection for player {}",
            self.game_id, msg.player_id
        );

        if let Some(_) = self.connections.insert(msg.player_id, msg.addr.clone()) {
            info!(
                "GAME ACTOR [{}]: Player {} already registered, updating connection.",
                self.game_id, msg.player_id
            );
            return Box::pin(async { Err(GameError::ActiveSessionExists) });
        } else {
            info!(
                "GAME ACTOR [{}]: Player {} registered successfully.",
                self.game_id, msg.player_id
            );

            if let Ok(mut gsm) = self.game_state.try_lock() {
                debug!(
                    "GAME ACTOR [{}]: Locking game state for player {}",
                    self.game_id, msg.player_id
                );
                gsm.update_player_connection_status(
                    self.get_player_type_by_uuid(msg.player_id),
                    true,
                );
                drop(gsm);
            } else {
                error!(
                    "GAME ACTOR [{}]: Failed to lock game state for player {}",
                    self.game_id, msg.player_id
                );
                return Box::pin(async { Err(GameError::GameStateLockFailed) });
            }
        }

        let kind = self.get_player_type_by_uuid(msg.player_id);
        self.player_connection_ready.insert(kind, true);

        let game_id = self.game_id.clone();
        let player_id = msg.player_id.clone();
        let is_all_ready = self.all_players_ready();

        let players_uuid = self
            .players
            .iter()
            .map(|(identity, _)| identity.id)
            .collect::<Vec<_>>();
        let players_addr = self.players.clone();
        let connections = self.connections.clone();
        let gsm = self.game_state.clone();

        if connections.is_empty() {
            error!(
                "GAME ACTOR [{}]: No connections available to send cards",
                game_id
            );
            return Box::pin(async { Err(GameError::NoConnections) });
        }

        Box::pin(async move {
            if is_all_ready {
                info!("GAME ACTOR [{}]: All players are ready", game_id);

                for uuid in players_uuid {
                    let connection = connections.get(&uuid).ok_or(GameError::NoConnections)?;
                    let player_addr = players_addr
                        .get_by_uuid(&uuid)
                        .ok_or(GameError::NoConnections)?;

                    let player_cards = player_addr
                        .send(GetMulliganDealCards)
                        .await?
                        .iter()
                        .map(|card| card.get_uuid())
                        .collect::<Vec<_>>();

                    println!(
                        "GAME ACTOR [{}]: Sending cards to connection: {:?}",
                        game_id,
                        connections.get(&uuid)
                    );
                    if let Err(err) = connection
                        .send(SendMulliganDealCards {
                            cards: player_cards,
                        })
                        .await?
                    {
                        warn!(
                            "GAME ACTOR [{}]: Failed to send cards to connection: {}",
                            game_id, err
                        );
                    } else {
                        info!(
                            "GAME ACTOR [{}]: Successfully sent cards to connection",
                            game_id
                        );

                        if let Ok(mut gsm) = gsm.try_lock() {
                            debug!(
                                "GAME ACTOR [{}]: Locking game state for player {}",
                                game_id, uuid
                            );
                            gsm.update_player_mulligan_status(
                                kind,
                                PlayerMulliganStatus::CardsDealt,
                            );
                            drop(gsm);
                        } else {
                            error!(
                                "GAME ACTOR [{}]: Failed to lock game state for player {}",
                                game_id, uuid
                            );
                            return Err(GameError::GameStateLockFailed);
                        }
                    }
                }
            } else {
                info!("GAME ACTOR [{}]: Not all players are ready yet", game_id);
            }

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
