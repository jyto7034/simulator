pub mod game_step;
mod getter;
pub mod turn_manager;

use std::collections::HashMap;

use turn_manager::Turn;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, types::PlayerType, Card},
    enums::{phase::Phase, DeckCode, UUID},
    exception::GameError,
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
    pub phase: Phase,
    pub turn: Turn,
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
            PlayerType::None => todo!(),
        }
    }

    pub fn get_turn(&self) -> &Turn {
        &self.turn
    }

    pub fn get_phase(&self) -> Phase {
        self.phase
    }

    pub fn get_turn_mut(&mut self) -> &mut Turn {
        &mut self.turn
    }

    pub fn get_phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn move_phase(&mut self) -> Phase {
        self.phase = self.phase.next_phase();
        self.phase
    }

    pub fn get_player(&self) -> &OptArc<Player> {
        &self.player1
    }

    pub fn get_opponent(&self) -> &OptArc<Player> {
        &self.player2
    }

    pub fn draw_card(&self, player_type: PlayerType) -> Result<(), GameError> {
        todo!()
    }

    pub fn restore_card(
        &mut self,
        player_type: PlayerType,
        src_cards: &Vec<UUID>,
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
    /// # 파라미터
    /// - `uuids`: 찾고자 하는 카드의 고유 식별자(UUID)들이 담긴 벡터입니다.
    ///             각 UUID는 고유하다고 가정합니다.
    ///
    /// # 반환값
    /// - 입력받은 순서대로 찾은 카드들을 담은 Vec<Card>를 반환합니다.
    ///
    /// # 패닉
    /// - 만약 입력받은 UUID 중 하나라도 플레이어와 상대방의 카드 목록에서 찾지 못하면,
    ///   해당 UUID와 함께 panic!이 발생합니다.
    ///
    pub fn get_cards_by_uuid(&self, uuids: Vec<UUID>) -> Vec<Card> {
        let player = self.get_player().get();
        let opponent = self.get_opponent().get();

        // 두 카드 리스트를 하나의 iterator로 합칩니다.
        // UUID가 고유하다고 가정하므로, (uuid, card) 쌍을 HashMap에 저장할 수 있습니다.
        let card_map: HashMap<UUID, Card> = player
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
                panic!("No card found with uuid: {:?}", uuid);
            }
        }
        results
    }
}
