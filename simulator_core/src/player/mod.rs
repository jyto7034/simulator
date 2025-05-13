use actix::{Actor, Addr, Context};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    card::{cards::Cards, take::TopTake, types::PlayerKind, Card},
    enums::ZoneType,
    exception::GameError,
    game::helper::Resoruce,
    selector::{mulligan::MulliganState, TargetCount},
    utils::deckcode_to_cards_single,
    zone::{
        deck::Deck, effect::Effect, field::Field, graveyard::Graveyard, hand::Hand, zone::Zone,
    },
};

pub mod message;

pub struct PlayerActor {
    opponent: Option<Addr<PlayerActor>>,
    player_type: PlayerKind,
    mulligan_state: MulliganState,
    cards: Cards,
    cost: Resoruce,
    mana: Resoruce,

    hand: Hand,
    deck: Deck,
    graveyard: Graveyard,
    effect: Effect,
    field: Field,
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
            cost: Resoruce::new(0, 0),
            mana: Resoruce::new(0, 0),
            hand: Hand::new(),
            deck: Deck::new(cards),
            graveyard: Graveyard::new(),
            effect: Effect::new(),
            field: Field::new(),
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
