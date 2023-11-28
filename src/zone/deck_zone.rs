use crate::deck::{Card, Cards};
use crate::enums::constant;
use crate::enums::constant::CardType;
use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::Entity;
use crate::zone::Zone;

pub struct DeckZone {
    zone_cards: Cards,
    zone_size: usize,
}
impl Zone for DeckZone {
    /// 현재 Zone 에 존재하는 모든 카드를 반환합니다.
    fn get_cards(&mut self) -> &mut Cards {
        &mut self.zone_cards
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    /// TODO: 무슨 방식으로(eg. 랜덤, 맨 위, 맨 아래) 넣을지 구현해야함.
    fn add_card(&mut self, card: Card) -> Result<(), Exception> {
        if card.get_card_type() != &CardType::Unit {
            panic!("DifferentCardTypes");
        }

        // Zone 에 존재할 수 있는 카드의 갯수를 넘어갈 때
        if self.zone_cards.len() < self.zone_size + 1 {
            return Err(Exception::ExceededCardLimit);
        }

        self.zone_cards.add_card(card.clone());
        Ok(())
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    fn remove_card(&mut self, card: Card) -> Result<(), Exception> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl DeckZone {
    pub fn new() -> DeckZone {
        DeckZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: constant::UNIT_ZONE_SIZE,
        }
    }
}

impl Entity for DeckZone {
    fn run(&self, game: &mut Game) -> Result<(), Exception> {
        todo!()
    }

    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }
}
