use std::vec;

use crate::deck::{Card, Cards};
use crate::enums::constant::{self, CardType};
use crate::exception::exception::Exception;
use crate::zone::Zone;

pub struct UnitZone {
    zone_cards: Cards,
    zone_size: usize,
}
impl Zone for UnitZone {
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

impl UnitZone {
    pub fn new() -> UnitZone {
        UnitZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: constant::UNIT_ZONE_SIZE,
        }
    }

    /// 카드를 교체합니다.
    pub fn replace_card(&mut self, src_card: &Card, dst_card: &Card) -> Result<(), Exception> {
        self.remove_card(dst_card)?;
        self.add_card(src_card)?;
        Ok(())
    }
}
