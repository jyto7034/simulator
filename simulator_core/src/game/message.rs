use actix::{Addr, Context, Handler, Message, Response, ResponseFuture};
use tracing::info;
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, take::RandomTake, types::PlayerKind, Card},
    exception::GameError,
    game::phase::PlayerPhaseProgress,
    player::{
        message::{AddCardsToDeck, GetCardFromDeck, RequestMulliganReroll},
        PlayerActor,
    },
    selector::TargetCount,
    server::input_handler::{InputAnswer, InputRequest},
};

use super::{phase::Phase, GameActor, GameConfig};

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct InitializeGame(pub GameConfig);

impl Handler<InitializeGame> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: InitializeGame, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPlayer {
    pub player_type: PlayerKind,
    pub player_addr: Addr<PlayerActor>,
}

impl Handler<RegisterPlayer> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: RegisterPlayer, _: &mut Self::Context) -> Self::Result {}
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerReady(pub PlayerKind);

impl Handler<PlayerReady> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: PlayerReady, _: &mut Self::Context) -> Self::Result {}
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestPlayCard {
    pub player_type: PlayerKind,
    pub card_id: Uuid,
}

impl Handler<RequestPlayCard> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: RequestPlayCard, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct SubmitInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
    pub answer: InputAnswer,
}

impl Handler<SubmitInput> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: SubmitInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
    pub request: InputRequest,
}

impl Handler<RequestInput> for GameActor {
    type Result = Result<(), GameError>;
    fn handle(&mut self, msg: RequestInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct IsCorrectPhase {
    pub phase: Phase,
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

#[derive(Message)]
#[rtype(result = "Result<Vec<Card>, GameError>")]
pub struct RerollRequestMulliganCard {
    pub player_type: PlayerKind,
    pub cards: Vec<Uuid>,
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

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct CheckReEntry {
    pub player_type: PlayerKind,
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
