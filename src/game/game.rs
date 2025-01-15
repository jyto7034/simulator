use crate::{
    card::deck::deckcode_to_cards,
    enums::{DeckCode, InsertType, PlayerType, ZoneType, PLAYER_1, PLAYER_2},
    exception::exception::Exception,
    unit::player::{Player, Resoruce},
    OptRcRef, RcRef,
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
    pub player1: OptRcRef<Player>,
    pub player2: OptRcRef<Player>,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), Exception> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;

        // Player 설정
        self.player1 = OptRcRef::new(Player::new(
            OptRcRef::none(),
            PlayerType::Player1,
            cards[PLAYER_1].clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ));
        self.player2 = OptRcRef::new(Player::new(
            OptRcRef::none(),
            PlayerType::Player2,
            cards[PLAYER_2].clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ));

        // 순환 참조이긴 한데, 딱히 문제 없음. 정리만 수동적으로 잘 정리해주면 됨

        self.player1.get_mut().opponent = OptRcRef::clone(&self.player2);
        self.player2.get_mut().opponent = OptRcRef::clone(&self.player1);

        let cards = self.player1.get().get_cards().clone();
        for card in &cards {
            self.player1
                .get_mut()
                .add_card(ZoneType::DeckZone, card.clone(), InsertType::Top)?;
        }

        let cards = self.player2.get().get_cards().clone();
        for card in &cards {
            self.player2
                .get_mut()
                .add_card(ZoneType::DeckZone, card.clone(), InsertType::Top)?;
        }

        self.player1.get_mut().set_cost(0);
        self.player1.get_mut().set_mana(0);

        self.player2.get_mut().set_cost(0);
        self.player2.get_mut().set_mana(0);

        Ok(())
    }

    pub fn get_player(&self, player_type: PlayerType) -> &OptRcRef<Player> {
        match player_type {
            PlayerType::Player1 => &self.player1,
            PlayerType::Player2 => &self.player2,
            PlayerType::None => todo!(),
        }
    }
}
