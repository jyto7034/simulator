pub mod game_step;
mod getter;
pub mod turn_manager;

use turn_manager::TurnManager;

use crate::{
    card::{insert::{BottomInsert, TopInsert}, types::PlayerType},
    enums::{phase::Phase, DeckCode, UUID},
    exception::GameError,
    server::end_point::AuthPlayer,
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
pub struct Game {
    pub player1: OptArc<Player>,
    pub player2: OptArc<Player>,
    pub phase: Phase,
    pub turn: TurnManager,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), GameError> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;
        
        // TODO: Limit 을 const 로 빼야함.
        let cost = Resoruce::new(0, 10);
        let mana = Resoruce::new(0, 3);
        self.player1 = OptArc::new(Player::new(OptArc::none(), PlayerType::Player1, cards[0].clone(), cost.clone(), mana.clone()));
        self.player2 = OptArc::new(Player::new(OptArc::none(), PlayerType::Player2, cards[1].clone(), cost, mana));

        self.player1.get_mut().get_deck_mut().get_cards_mut().v_card.extend(cards[0].clone().v_card);
        self.player2.get_mut().get_deck_mut().get_cards_mut().v_card.extend(cards[1].clone().v_card);
        Ok(())
    }
}

impl Game {
    pub fn get_player_by_type<T: Into<PlayerType>>(&self, player_type: T) -> &OptArc<Player> {
        match player_type.into() {
            PlayerType::Player1 => &self.player1,
            PlayerType::Player2 => &self.player2,
            PlayerType::None => todo!(),
        }
    }

    pub fn get_turn(&self) -> &TurnManager {
        &self.turn
    }

    pub fn get_phase(&self) -> Phase {
        self.phase
    }

    pub fn get_turn_mut(&mut self) -> &mut TurnManager {
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
                .get_mut()
                .get_deck_mut()
                .add_card(vec![card.clone()], Box::new(BottomInsert))?;
        }
        Ok(())
    }
}
