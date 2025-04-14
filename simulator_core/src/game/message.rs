use actix::{Addr, Context, Handler, Message, Response};
use tracing::info;
use uuid::Uuid;

use crate::{
    card::types::PlayerType,
    exception::GameError,
    game::phase::PlayerPhaseProgress,
    player::{message::RequestMulliganReroll, PlayerActor},
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
    pub player_type: PlayerType,
    pub player_addr: Addr<PlayerActor>,
}

impl Handler<RegisterPlayer> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: RegisterPlayer, _: &mut Self::Context) -> Self::Result {}
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerReady(pub PlayerType);

impl Handler<PlayerReady> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: PlayerReady, _: &mut Self::Context) -> Self::Result {}
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestPlayCard {
    pub player_type: PlayerType,
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
    pub player_type: PlayerType,
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
    pub player_type: PlayerType,
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
#[rtype(result = "Result<Vec<Uuid>, GameError>")]
pub struct ProcessRerollRequest {
    pub player_type: PlayerType,
    pub cards_to_reroll: Vec<Uuid>,
}

impl Handler<ProcessRerollRequest> for GameActor {
    type Result = Response<Result<Vec<Uuid>, GameError>>; // Uuid 반환

    fn handle(&mut self, msg: ProcessRerollRequest, ctx: &mut Context<Self>) -> Self::Result {
        println!(
            "GAME ACTOR: Received reroll request from {:?}",
            msg.player_type
        );

        // 1. 요청 유효성 검사 (예: 멀리건 페이즈인가?)
        if self.phase_state.get_phase() != Phase::Mulligan {
            return Response::reply(Err(GameError::WrongPhase));
        }
        // TODO: 해당 플레이어가 리롤 가능한 상태인지 추가 검사 (예: 이미 완료하지 않았는지)

        // 2. 해당 PlayerActor 주소 가져오기
        let player_addr = match self.players.get(&msg.player_type) {
            Some(addr) => addr.clone(),
            None => return Response::reply(Err(GameError::PlayerNotFound)),
        };

        // 3. PlayerActor에게 RequestMulliganReroll 메시지 보내기
        let fut = async move {
            match player_addr
                .send(RequestMulliganReroll {
                    cards_to_restore: msg.cards_to_reroll,
                })
                .await
            {
                Ok(Ok(new_card_uuids)) => {
                    // 성공 시 새로 뽑은 카드의 UUID 목록 반환
                    Ok(new_card_uuids)
                }
                Ok(Err(e)) => {
                    // PlayerActor가 에러 반환
                    eprintln!(
                        "GAME ACTOR: Reroll failed for {:?}: {:?}",
                        msg.player_type, e
                    );
                    Err(e)
                }
                Err(mailbox_error) => {
                    // 메시지 전송 실패
                    eprintln!(
                        "GAME ACTOR: MailboxError during reroll for {:?}: {}",
                        msg.player_type, mailbox_error
                    );
                    Err(GameError::InternalServerError) // 내부 서버 오류로 처리
                }
            }
        };

        // 비동기 작업 결과를 Response로 감싸서 반환
        Response::fut(fut)
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
#[rtype(result = "Result<(), GameError>")]
pub struct CheckReEntry {
    pub player_type: PlayerType,
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
