use actix::{Handler, Message};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    exception::{GameError, StateError},
    game::{phase::Phase, GameActor},
};

/// `RequestPlayCard` 메시지.
///
/// 플레이어가 카드를 사용하기 위한 요청을 나타냅니다.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use crate::card::types::PlayerKind;
/// use simulator_core::game::msg::gameplay::RequestPlayCard;
///
/// let request = RequestPlayCard {
///     player_type: PlayerKind::User,
///     card_id: Uuid::new_v4(),
/// };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestPlayCard {
    /// 카드를 사용하려는 플레이어의 종류.
    pub player_type: PlayerKind,
    /// 사용하려는 카드의 UUID.
    pub card_id: Uuid,
}

/// `SubmitInput` 메시지.
///
/// 플레이어가 입력을 제출하기 위한 요청을 나타냅니다.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use crate::card::types::PlayerKind;
/// use simulator_core::game::msg::gameplay::SubmitInput;
///
/// let submit = SubmitInput {
///     player_type: PlayerKind::User,
///     request_id: Uuid::new_v4(),
/// };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct SubmitInput {
    /// 입력을 제출하려는 플레이어의 종류.
    pub player_type: PlayerKind,
    /// 제출하려는 입력의 UUID.
    pub request_id: Uuid,
}

/// `RequestInput` 메시지.
///
/// 플레이어에게 입력을 요청하기 위한 메시지입니다.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use crate::card::types::PlayerKind;
/// use simulator_core::game::msg::gameplay::RequestInput;
///
/// let request = RequestInput {
///     player_type: PlayerKind::User,
///     request_id: Uuid::new_v4(),
/// };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestInput {
    /// 입력을 요청받을 플레이어의 종류.
    pub player_type: PlayerKind,
    /// 요청의 UUID.
    pub request_id: Uuid,
}

/// `IsCorrectPhase` 메시지.
///
/// 현재 게임 단계가 예상 단계와 일치하는지 확인하는 데 사용됩니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::phase::Phase;
/// use simulator_core::game::msg::gameplay::IsCorrectPhase;
///
/// let is_correct = IsCorrectPhase {
///     phase: Phase::Start,
/// };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct IsCorrectPhase {
    /// 확인하려는 단계.
    pub phase: Phase,
}

/// `ChoiceCardRequestPayload` 구조체.
///
/// 카드 선택 요청에 필요한 데이터를 담는 구조체입니다.
///
/// # Fields
///
/// * `player`: 선택을 요청하는 플레이어의 이름.
/// * `choice_type`: 선택의 종류. (예: "카드 선택", "대상 선택")
/// * `source_card_id`: 선택의 근원이 되는 카드의 UUID.
/// * `min_selections`: 최소 선택 횟수.
/// * `max_selections`: 최대 선택 횟수.
/// * `destination`: 선택 결과를 저장할 위치.
/// * `is_open`: 선택이 공개적인지 여부.
/// * `is_hidden_from_opponent`: 상대방에게 숨겨지는지 여부.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use simulator_core::game::msg::gameplay::ChoiceCardRequestPayload;
///
/// let payload = ChoiceCardRequestPayload {
///     player: "Player1".to_string(),
///     choice_type: "CardSelection".to_string(),
///     source_card_id: Uuid::new_v4(),
///     min_selections: 1,
///     max_selections: 3,
///     destination: "Hand".to_string(),
///     is_open: true,
///     is_hidden_from_opponent: false,
/// };
/// ```
pub struct ChoiceCardRequestPayload {
    /// 선택을 요청하는 플레이어의 이름.
    pub player: String,
    /// 선택의 종류. (예: "카드 선택", "대상 선택")
    pub choice_type: String,
    /// 선택의 근원이 되는 카드의 UUID.
    pub source_card_id: Uuid,
    /// 최소 선택 횟수.
    pub min_selections: usize,
    /// 최대 선택 횟수.
    pub max_selections: usize,
    /// 선택 결과를 저장할 위치.
    pub destination: String,
    /// 선택이 공개적인지 여부.
    pub is_open: bool,
    /// 상대방에게 숨겨지는지 여부.
    pub is_hidden_from_opponent: bool,
}

impl Handler<RequestPlayCard> for GameActor {
    type Result = Result<(), GameError>;

    /// `RequestPlayCard` 메시지를 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `msg`: 처리할 `RequestPlayCard` 메시지.
    /// * `_`: 액터 컨텍스트 (사용되지 않음).
    ///
    /// # Returns
    ///
    /// 성공하면 `Ok(())`, 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```
    /// // GameActor의 handle 메서드를 직접 호출하는 예제 (실제로는 Actix 프레임워크를 통해 호출됨)
    /// use actix::Context;
    /// use uuid::Uuid;
    /// use crate::card::types::PlayerKind;
    /// use simulator_core::game::msg::gameplay::RequestPlayCard;
    /// use simulator_core::game::GameActor;
    ///
    /// let mut actor = GameActor::new();
    /// let mut ctx = Context::new();
    /// let msg = RequestPlayCard {
    ///     player_type: PlayerKind::User,
    ///     card_id: Uuid::new_v4(),
    /// };
    /// let result = actor.handle(msg, &mut ctx);
    /// assert!(result.is_ok());
    /// ```
    fn handle(&mut self, msg: RequestPlayCard, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<SubmitInput> for GameActor {
    type Result = Result<(), GameError>;

    /// `SubmitInput` 메시지를 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `msg`: 처리할 `SubmitInput` 메시지.
    /// * `_`: 액터 컨텍스트 (사용되지 않음).
    ///
    /// # Returns
    ///
    /// 성공하면 `Ok(())`, 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```
    /// // GameActor의 handle 메서드를 직접 호출하는 예제 (실제로는 Actix 프레임워크를 통해 호출됨)
    /// use actix::Context;
    /// use uuid::Uuid;
    /// use crate::card::types::PlayerKind;
    /// use simulator_core::game::msg::gameplay::SubmitInput;
    /// use simulator_core::game::GameActor;
    ///
    /// let mut actor = GameActor::new();
    /// let mut ctx = Context::new();
    /// let msg = SubmitInput {
    ///     player_type: PlayerKind::User,
    ///     request_id: Uuid::new_v4(),
    /// };
    /// let result = actor.handle(msg, &mut ctx);
    /// assert!(result.is_ok());
    /// ```
    fn handle(&mut self, msg: SubmitInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RequestInput> for GameActor {
    type Result = Result<(), GameError>;

    /// `RequestInput` 메시지를 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `msg`: 처리할 `RequestInput` 메시지.
    /// * `_`: 액터 컨텍스트 (사용되지 않음).
    ///
    /// # Returns
    ///
    /// 성공하면 `Ok(())`, 실패하면 `Err(GameError)`.
    ///
    /// # Examples
    ///
    /// ```
    /// // GameActor의 handle 메서드를 직접 호출하는 예제 (실제로는 Actix 프레임워크를 통해 호출됨)
    /// use actix::Context;
    /// use uuid::Uuid;
    /// use crate::card::types::PlayerKind;
    /// use simulator_core::game::msg::gameplay::RequestInput;
    /// use simulator_core::game::GameActor;
    ///
    /// let mut actor = GameActor::new();
    /// let mut ctx = Context::new();
    /// let msg = RequestInput {
    ///     player_type: PlayerKind::User,
    ///     request_id: Uuid::new_v4(),
    /// };
    /// let result = actor.handle(msg, &mut ctx);
    /// assert!(result.is_ok());
    /// ```
    fn handle(&mut self, msg: RequestInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<IsCorrectPhase> for GameActor {
    type Result = Result<(), GameError>;

    /// `IsCorrectPhase` 메시지를 처리합니다.
    ///
    /// # Arguments
    ///
    /// * `msg`: 처리할 `IsCorrectPhase` 메시지.
    /// * `_`: 액터 컨텍스트 (사용되지 않음).
    ///
    /// # Returns
    ///
    /// 현재 게임 단계가 메시지의 `phase`와 같으면 `Ok(())`를 반환합니다.
    /// 그렇지 않으면 `Err(GameError)`를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// // GameActor의 handle 메서드를 직접 호출하는 예제 (실제로는 Actix 프레임워크를 통해 호출됨)
    /// use actix::Context;
    /// use simulator_core::game::phase::Phase;
    /// use simulator_core::game::msg::gameplay::IsCorrectPhase;
    /// use simulator_core::game::GameActor;
    ///
    /// let mut actor = GameActor::new();
    /// actor.turn.current_phase = Phase::Start; // 현재 단계를 Start로 설정
    /// let mut ctx = Context::new();
    /// let msg = IsCorrectPhase {
    ///     phase: Phase::Start,
    /// };
    /// let result = actor.handle(msg, &mut ctx);
    /// assert!(result.is_ok());
    ///
    /// let msg_wrong_phase = IsCorrectPhase {
    ///     phase: Phase::Draw,
    /// };
    /// let result_wrong_phase = actor.handle(msg_wrong_phase, &mut ctx);
    /// assert!(result_wrong_phase.is_err());
    /// ```
    fn handle(&mut self, msg: IsCorrectPhase, _: &mut Self::Context) -> Self::Result {
        if self.turn.current_phase == msg.phase {
            Ok(())
        } else {
            Err(GameError::State(StateError::InvalidActionForPhase { current_phase: format!("{:?}", self.turn.current_phase), action: format!("{:?}", msg.phase) }))
        }
    }
}