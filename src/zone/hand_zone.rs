use crate::deck::{Card, Cards};
use crate::enums::constant::CardType;
use crate::enums::constant::{self, UUID};
use crate::exception::exception::Exception;
use crate::unit::Entity;
use crate::zone::Zone;

pub struct HandZone {
    zone_cards: Cards,
    zone_size: usize,
}
impl Zone for HandZone {
    /// 현재 Zone 에 존재하는 모든 카드를 반환합니다.
    fn get_cards(&self) -> &Cards {
        &self.zone_cards
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    fn add_card(&mut self, card: &Card) -> Result<(), Exception> {
        if card.get_card_type() != &CardType::Unit {
            return Err(Exception::DifferentCardTypes);
        }
        // Zone 에 존재할 수 있는 카드의 갯수를 넘어갈 때
        if self.zone_cards.len() < self.zone_size + 1 {
            return Err(Exception::ExceededCardLimit);
        }
        self.zone_cards.push(card);
        Ok(())
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    fn remove_card(&mut self, card: &Card) -> Result<(), Exception> {
        let prev_len = self.zone_cards.len();
        self.zone_cards
            .v_card
            .retain(|item| item.get_uuid() == card.get_uuid());
        if self.zone_cards.len() != prev_len {
            Ok(())
        } else {
            Err(Exception::NothingToRemove)
        }
    }
}

impl HandZone {
    pub fn new() -> HandZone {
        HandZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: constant::UNIT_ZONE_SIZE,
        }
    }

    pub fn test(&mut self) {
        self.zone_size = 1;
    }

    // game.player_1.get_hand_zone().draw();
    pub fn draw(
        &mut self,
        draw_type: constant::CardDrawType,
        count_of_card: Option<usize>,
    ) -> Result<&UUID, Exception> {
        let cards = self.zone_cards.draw(draw_type, Some(1));
        if !cards.is_empty() {
            Ok(&cards[0].get_uuid())
        } else {
            Err(Exception::FailedToDrawCard)
        }
    }
}

impl Entity for HandZone {
    fn run(&self) -> Result<(), Exception> {
        todo!()
    }

    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }
}
