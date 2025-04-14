pub mod chain;
pub mod choice;
pub mod game_step;
mod getter;
mod helper;
pub mod phase;
pub mod turn_manager;

use std::collections::HashMap;

use chain::Chain;
use phase::{Phase, PhaseState};
use turn_manager::Turn;
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, take::BottomTake, types::PlayerType, Card},
    enums::DeckCode,
    exception::GameError,
    selector::TargetCount,
    server::input_handler::InputWaiter,
    unit::player::{Player, Resoruce},
    utils::deckcode_to_cards,
    zone::zone::Zone,
    OptArc,
};

pub struct GameConfig {
    /// Player's Deckcode
    pub player_1_deckcode: DeckCode,
    pub player_2_deckcode: DeckCode,

    /// 1 : Player 1,
    /// 2 : Player 2
    pub attacker: usize,
}

/// 게임의 상태를 관리/저장 하는 구조체
/// Card 로 인한 모든 변경 사항은 Task 로써 저장되며,
/// 그것을 담은 Tasks 를 Procedure 에게 전달하여 게임 결과를 계산한다.
#[derive(Clone)]
pub struct Game {
    pub player1: OptArc<Player>,
    pub player2: OptArc<Player>,
    pub phase_state: PhaseState,
    pub turn: Turn,
    pub chain: Chain,
    pub input_waiter: InputWaiter,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), GameError> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;

        // TODO: Limit 을 const 로 빼야함.
        let cost = Resoruce::new(0, 10);
        let mana = Resoruce::new(0, 3);
        self.player1 = OptArc::new(Player::new(
            OptArc::none(),
            PlayerType::Player1,
            cards[0].clone(),
            cost.clone(),
            mana.clone(),
        ));
        self.player2 = OptArc::new(Player::new(
            OptArc::none(),
            PlayerType::Player2,
            cards[1].clone(),
            cost,
            mana,
        ));

        self.player1
            .get()
            .get_deck_mut()
            .get_cards_mut()
            .extend(cards[0].clone());
        self.player2
            .get()
            .get_deck_mut()
            .get_cards_mut()
            .extend(cards[1].clone());
        Ok(())
    }
}

impl Game {
    pub fn get_player_by_type<T: Into<PlayerType> + Copy>(
        &self,
        player_type: T,
    ) -> &OptArc<Player> {
        match player_type.into() {
            PlayerType::Player1 => &self.player1,
            PlayerType::Player2 => &self.player2,
        }
    }

    pub fn get_turn(&self) -> &Turn {
        &self.turn
    }

    pub fn get_phase(&self) -> Phase {
        self.phase_state.get_phase()
    }

    pub fn get_phase_state_mut(&mut self) -> &mut PhaseState {
        &mut self.phase_state
    }

    pub fn get_phase_state(&mut self) -> &PhaseState {
        &self.phase_state
    }

    pub fn get_turn_mut(&mut self) -> &mut Turn {
        &mut self.turn
    }

    pub fn move_phase(&mut self) {
        self.phase_state.get_phase().move_to_next_phase();
    }

    pub fn get_player(&self) -> &OptArc<Player> {
        &self.player1
    }

    pub fn get_opponent(&self) -> &OptArc<Player> {
        &self.player2
    }

    pub fn get_chain_mut(&mut self) -> &mut Chain {
        &mut self.chain
    }

    pub fn get_input_waiter_mut(&mut self) -> &mut InputWaiter {
        &mut self.input_waiter
    }

    // pub fn resolve_chain(&mut self) -> Result<(), GameError> {
    //     self.chain.resolve(self)?;
    //     Ok(())
    // }

    pub fn get_chain(&self) -> &Chain {
        &self.chain
    }

    /// 플레이어의 덱에서 카드를 뽑아 손에 추가합니다.
    /// # Parameters
    /// * `player_type` - 덱에서 카드를 뽑을 플레이어의 종류입니다.
    /// # Returns
    /// * 뽑은 카드를 반환합니다.
    /// # Errors
    /// * 덱에 카드가 없을 경우 NoCardsLeft 에러를 반환합니다.
    pub fn draw_card(&mut self, player_type: PlayerType) -> Result<Card, GameError> {
        let result = self
            .get_player_by_type(player_type)
            .get()
            .get_deck_mut()
            .take_card(Box::new(BottomTake(TargetCount::Exact(1))))?;

        // TODO: 이 확인이 필요한가?
        if result.is_empty() {
            return Err(GameError::NoCardsLeft);
        }

        Ok(result[0].clone())
    }

    /// 파라미터로 들어오는 카드들을 덱의 맨 밑으로 복원합니다.
    ///
    /// # Parameters
    /// * `player_type` - 카드를 복원할 플레이어 타입
    /// * `src_cards` - 복원할 카드들의 UUID 목록
    ///
    /// # Returns
    /// * `Ok(())` - 모든 카드가 성공적으로 덱의 맨 밑에 추가됨
    /// * `Err(GameError)` - 카드 복원 중 오류 발생
    ///
    /// # Errors
    /// * `GameError::CardNotFound` - 지정된 UUID를 가진 카드를 플레이어가 소유하지 않은 경우
    /// * `GameError::ExceededCardLimit` - 덱에 자리가 없어 카드를 추가할 수 없는 경우
    ///
    pub fn restore_card(
        &mut self,
        player_type: PlayerType,
        src_cards: &Vec<Uuid>,
    ) -> Result<(), GameError> {
        for card_uuid in src_cards {
            let card = {
                let player = self.get_player_by_type(player_type).get();
                match player.get_cards().find_by_uuid(card_uuid.clone()) {
                    Some(card) => card.clone(),
                    None => return Err(GameError::CardNotFound),
                }
            };
            self.get_player_by_type(player_type)
                .get()
                .get_deck_mut()
                .add_card(vec![card.clone()], Box::new(BottomInsert))?;
        }
        Ok(())
    }

    /// 두 플레이어의 카드 목록에서 입력받은 UUID에 해당하는 카드를 순서대로 찾아 반환합니다.
    ///
    /// # 설명
    /// - 플레이어와 상대방의 모든 카드 목록을 합쳐서, 각 카드의 고유한 UUID를 key로 하는 HashMap을 생성합니다.
    /// - 입력받은 UUID 리스트의 순서대로 해당 카드들을 찾아 Vec<Card>로 반환합니다.
    ///
    /// # Parameters
    /// - `uuids`: 찾고자 하는 카드의 고유 식별자(UUID)들이 담긴 벡터입니다.
    ///             각 UUID는 고유하다고 가정합니다.
    ///
    /// # Returns
    /// - 입력받은 순서대로 찾은 카드들을 담은 Vec<Card>를 반환합니다.
    ///
    /// # Panics
    /// - 만약 입력받은 UUID 중 하나라도 플레이어와 상대방의 카드 목록에서 찾지 못하면,
    ///   GameError::CardsNotFound 에러를 반환합니다.
    pub fn get_cards_by_uuids(&self, uuids: Vec<Uuid>) -> Result<Vec<Card>, GameError> {
        let player = self.get_player().get();
        let opponent = self.get_opponent().get();

        // 두 카드 리스트를 하나의 iterator로 합칩니다.
        // UUID가 고유하다고 가정하므로, (uuid, card) 쌍을 HashMap에 저장할 수 있습니다.
        let card_map: HashMap<Uuid, Card> = player
            .get_cards()
            .iter()
            .chain(opponent.get_cards().iter())
            .map(|card| (card.get_uuid(), card.clone()))
            .collect();

        // 입력한 uuid 순서대로 카드들을 찾아서 반환합니다.
        // 입력 uuid 중 하나라도 매칭되는 카드가 없으면 panic! 합니다.
        let mut results = Vec::with_capacity(uuids.len());
        for uuid in uuids {
            if let Some(card) = card_map.get(&uuid) {
                results.push(card.clone());
            } else {
                return Err(GameError::CardsNotFound);
            }
        }
        Ok(results)
    }

    /// 두 플레이어의 카드 목록에서 입력받은 UUID에 해당하는 카드를 순서대로 찾아 반환합니다.
    ///
    /// # 설명
    /// - 플레이어와 상대방의 모든 카드 목록을 합쳐서, 각 카드의 고유한 UUID를 key로 하는 HashMap을 생성합니다.
    /// - 입력받은 UUID 리스트의 순서대로 해당 카드들을 찾아 Card로 반환합니다.
    ///
    /// # Parameters
    /// - `uuids`: 찾고자 하는 카드의 고유 식별자(UUID)들이 담긴 벡터입니다.
    ///             각 UUID는 고유하다고 가정합니다.
    ///
    /// # Returns
    /// - 입력받은 순서대로 찾은 카드들을 담은 Card를 반환합니다.
    ///
    /// # Panics
    /// - 만약 입력받은 UUID 를 가지고 플레이어와 상대방의 카드 목록에서 찾지 못하면,
    ///   GameError::CardNotFound 에러를 반환합니다.
    pub fn get_cards_by_uuid(&self, uuid: Uuid) -> Result<Card, GameError> {
        let player = self.get_player().get();
        let opponent = self.get_opponent().get();

        // 두 카드 리스트를 하나의 iterator로 합칩니다.
        // UUID가 고유하다고 가정하므로, (uuid, card) 쌍을 HashMap에 저장할 수 있습니다.
        let card_map: HashMap<Uuid, Card> = player
            .get_cards()
            .iter()
            .chain(opponent.get_cards().iter())
            .map(|card| (card.get_uuid(), card.clone()))
            .collect();

        // 입력한 uuid 순서대로 카드들을 찾아서 반환합니다.
        // 입력 uuid 중 하나라도 매칭되는 카드가 없으면 panic! 합니다.
        card_map.get(&uuid).cloned().ok_or(GameError::CardNotFound)
    }
}

// TODO: 게임의 상태를 hash 로 변환해서 제공해야함.
