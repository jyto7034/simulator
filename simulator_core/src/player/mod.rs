use actix::{Actor, Addr, Context};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    card::{cards::Cards, take::TopTake, types::PlayerKind, Card},
    enums::ZoneType,
    exception::GameError,
    selector::{mulligan::MulliganState, TargetCount},
    utils::deckcode_to_cards_single,
    zone::{
        deck::Deck, effect::Effect, field::Field, graveyard::Graveyard, hand::Hand, zone::Zone,
    },
};

pub mod message;

/// 플레이어의 소모 가능한 자원 (마나, 코스트 등)을 관리하는 구조체
#[derive(Clone, Debug)]
pub struct Resoruce {
    current: i32,  // 현재 보유량
    max: i32,      // 최대 보유 가능량
    per_turn: i32, // 턴당 자동 충전량
}

impl Resoruce {
    /// 새로운 자원 생성
    /// current: 현재 보유량, max: 최대 보유량
    pub fn new(current: i32, max: i32) -> Self {
        Self {
            current,
            max,
            per_turn: 1, // 기본값: 턴당 1씩 충전
        }
    }

    /// 턴당 충전량을 설정한 자원 생성
    pub fn new_with_per_turn(current: i32, max: i32, per_turn: i32) -> Self {
        Self {
            current,
            max,
            per_turn,
        }
    }

    /// 현재 보유량 반환
    pub fn get_current(&self) -> i32 {
        self.current
    }

    /// 최대 보유량 반환
    pub fn get_max(&self) -> i32 {
        self.max
    }

    /// 턴당 충전량 반환
    pub fn get_per_turn(&self) -> i32 {
        self.per_turn
    }

    /// 자원 소모 (성공 여부 반환)
    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// 자원 소모 가능 여부 확인
    pub fn can_spend(&self, amount: i32) -> bool {
        self.current >= amount
    }

    /// 자원 충전 (최대값 초과 불가)
    pub fn refill(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// 턴 시작시 자동 충전
    pub fn turn_refill(&mut self) {
        self.refill(self.per_turn);
    }

    /// 최대값 증가 (마나 크리스탈 증가 등)
    pub fn increase_max(&mut self, amount: i32) {
        self.max += amount;
        // 최대값 증가시 현재값도 증가 (하스스톤 스타일)
        self.current += amount;
    }

    /// 자원 완전 충전
    pub fn full_refill(&mut self) {
        self.current = self.max;
    }

    /// 현재 자원이 비어있는지 확인
    pub fn is_empty(&self) -> bool {
        self.current <= 0
    }

    /// 현재 자원이 가득 찬지 확인
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// 현재 자원 직접 설정 (최대값 초과 불가)
    pub fn set_current(&mut self, amount: i32) {
        self.current = amount.clamp(0, self.max);
    }

    /// 최대값 직접 설정
    pub fn set_max(&mut self, max: i32) {
        self.max = max;
        // 현재값이 새로운 최대값을 초과하면 조정
        if self.current > self.max {
            self.current = self.max;
        }
    }

    /// 턴당 충전량 설정
    pub fn set_per_turn(&mut self, per_turn: i32) {
        self.per_turn = per_turn;
    }
}

pub struct PlayerActor {
    pub opponent: Option<Addr<PlayerActor>>,
    pub player_type: PlayerKind,
    pub mulligan_state: MulliganState,
    pub cards: Cards,
    pub cost: Resoruce,
    pub mana: Resoruce,
    pub health: i32,

    pub hand: Hand,
    pub deck: Deck,
    pub graveyard: Graveyard,
    pub effect: Effect,
    pub field: Field,
}

impl Actor for PlayerActor {
    type Context = Context<Self>;
}

impl PlayerActor {
    pub fn new(player_type: PlayerKind, deck_code: String) -> Self {
        let cards = deckcode_to_cards_single(deck_code).unwrap();
        Self {
            player_type,
            opponent: None,
            mulligan_state: MulliganState::new(),
            cards: cards.clone(),
            cost: Resoruce::new(0, 0), // 코스트는 보통 카드별로 다르므로 0으로 시작
            mana: Resoruce::new(1, 1), // 게임 시작시 마나 1/1으로 시작 (하스스톤 스타일)
            hand: Hand::new(),
            deck: Deck::new(cards),
            graveyard: Graveyard::new(),
            effect: Effect::new(),
            field: Field::new(),
            // TODO: Player Health 를 정확히 구현해야함.
            health: 20,
        }
    }

    pub fn restore_cards(&mut self, card: &[Card], target_zone: ZoneType) -> Result<(), GameError> {
        match target_zone {
            ZoneType::Hand => todo!(),
            ZoneType::Deck => todo!(),
            ZoneType::Graveyard => todo!(),
            ZoneType::Effect => todo!(),
            ZoneType::Field => todo!(),
            ZoneType::None => todo!(),
        }
    }

    pub fn get_cards_by_uuids(&self, uuids: &[Uuid]) -> Result<Vec<Card>, GameError> {
        todo!()
    }

    #[instrument(skip(self), fields(player_type = ?player_type.into()))]
    pub fn get_new_mulligan_cards<T: Into<PlayerKind> + Copy>(
        &mut self,
        player_type: T,
        count: usize,
    ) -> Result<Vec<Uuid>, GameError> {
        let player_type = player_type.into();
        debug!(
            "멀리건 카드 뽑기 시도: player={:?}, count={}",
            player_type, count
        );

        let take_result = self
            .deck
            .take_card(Box::new(TopTake(TargetCount::Exact(count))))?;

        let uuids = take_result
            .iter()
            .map(|card| card.get_uuid())
            .collect::<Vec<_>>();

        debug!(
            "멀리건 카드 뽑기 완료: player={:?}, card_count={}",
            player_type,
            uuids.len()
        );

        Ok(uuids)
    }

    pub fn get_cards_mut(&mut self) -> &mut Cards {
        &mut self.cards
    }

    pub fn get_cards(&self) -> &Cards {
        &self.cards
    }
}

//         #[instrument(skip(self), fields(player_type = ?player_type.into()))]
//         pub fn add_select_cards<T: Into<PlayerType> + Copy>(
//             &mut self,
//             cards: Vec<Uuid>,
//             player_type: T,
//         ) {
//             let player_type = player_type.into();
//             debug!(
//                 "멀리건 상태에 카드 추가 시작: player={:?}, cards={:?}",
//                 player_type, cards
//             );

//             let mut player = self.get_player_by_type(player_type).get();

//             player
//                 .get_mulligan_state_mut()
//                 .add_select_cards(cards.clone());
//             debug!("멀리건 상태에 카드 추가 완료: player={:?}", player_type);
//         }

//         pub fn add_reroll_cards<T: Into<PlayerType> + Copy>(
//             &mut self,
//             player_type: T,
//             payload_cards: Vec<Uuid>,
//             rerolled_cards: Vec<Uuid>,
//         ) {
//             let player_type = player_type.into();
//             debug!("선택 카드 제거: player={:?}", player_type);
//             self.get_player_by_type(player_type)
//                 .get()
//                 .get_mulligan_state_mut()
//                 .remove_select_cards(payload_cards);

//             debug!("리롤된 카드 추가: player={:?}", player_type);
//             self.get_player_by_type(player_type)
//                 .get()
//                 .get_mulligan_state_mut()
//                 .add_select_cards(rerolled_cards);
//         }

//         pub fn reroll_request<T: Into<PlayerType> + Copy>(
//             &mut self,
//             player_type: T,
//             cards: Vec<Uuid>,
//         ) -> Result<Vec<Uuid>, GameError> {
//             let player_type = player_type.into();
//             // 플레이어가 이미 준비 상태인 경우
//             if self
//                 .get_player_by_type(player_type)
//                 .get()
//                 .get_mulligan_state_mut()
//                 .is_ready()
//             {
//                 warn!("플레이어가 이미 준비 상태: player={:?}", player_type);
//                 return Err(GameError::AlreadyReady);
//                 // try_send_error!(session, GameError::AlreadyReady, retry 3);
//             }

//             // 플레이어가 선택한 카드가 유효한지 확인합니다.
//             debug!("선택한 카드 유효성 검사: player={:?}", player_type);
//             if let Err(e) = self.get_cards_by_uuids(cards.clone()) {
//                 error!("유효하지 않은 카드 선택: player={:?}", player_type);
//                 return Err(e);
//             }

//             // 기존 카드를 덱의 최하단에 위치 시킨 뒤, 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장하고 json 으로 변환하여 전송합니다.
//             info!("카드 리롤 시작: player={:?}", player_type);
//             let rerolled_card = match self.restore_then_reroll_mulligan_cards(player_type, cards) {
//                 Ok(cards) => {
//                     debug!("카드 리롤 성공: card_count={}", cards.len());
//                     cards
//                 }
//                 Err(e) => {
//                     error!("카드 리롤 실패: player={:?}, error={:?}", player_type, e);
//                     panic!("카드 리롤 실패: player={:?}, error={:?}", player_type, e);
//                 }
//             };

//             Ok(rerolled_card)
//         }

//         /// 멀리건 완료 처리 함수
//         /// - 게임 객체를 받아서, 플레이어의 멀리건 상태를 완료로 변경하고, 선택한 카드들을 손으로 이동시킵니다.
//         /// - 선택한 카드들의 UUID를 반환합니다.
//         /// # Arguments
//         /// * `game` - 게임 객체
//         /// * `player_type` - 플레이어 타입
//         /// # Returns
//         /// * `Vec<Uuid>` - 선택한 카드들의 UUID

//         pub fn process_mulligan_completion<T: Into<PlayerType> + Copy>(
//             &mut self,
//             player_type: T,
//         ) -> Result<Vec<Uuid>, GameError> {
//             let player_type = player_type.into();

//             // 선택된 멀리건 카드들의 UUID 를 얻습니다.
//             let selected_cards = self
//                 .get_player_by_type(player_type)
//                 .get()
//                 .get_mulligan_state_mut()
//                 .get_select_cards();

//             // UUID -> Card 객체로 변환하는 과정입니다.
//             let cards = self.get_cards_by_uuids(selected_cards.clone())?;

//             // add_card 함수를 통해 선택된 카드들을 손으로 이동시킵니다.
//             self.get_player_by_type(player_type)
//                 .get()
//                 .get_hand_mut()
//                 .add_card(cards, Box::new(TopInsert))
//                 .map_err(|_| GameError::InternalServerError)?;

//             // 멀리건 상태를 "완료" 상태로 변경합니다.
//             self.get_player_by_type(player_type)
//                 .get()
//                 .get_mulligan_state_mut()
//                 .confirm_selection();

//             // 그런 뒤, 선택한 카드들을 반환합니다.
//             Ok(selected_cards)
//         }

//         pub fn check_player_ready_state<T: Into<PlayerType> + Copy>(&self, player_type: T) -> bool {
//             let player_type = player_type.into();
//             self.get_player_by_type(player_type.reverse())
//                 .get()
//                 .get_mulligan_state_mut()
//                 .is_ready()
//         }
//     }
// }
