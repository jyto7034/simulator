use crate::{
    card::{Card, cards::Cards},
    enums::{CardType, InsertType, UNIT_ZONE_SIZE},
    exception::Exception,
};

use super::zone::Zone;

#[derive(Clone)]
pub struct EffectZone {
    zone_cards: Cards,
    zone_size: usize,
}

impl EffectZone {
    pub fn new() -> EffectZone {
        EffectZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: UNIT_ZONE_SIZE,
        }
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    pub fn remove_card(&mut self, _card: Card) -> Result<(), Exception> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl Zone for EffectZone {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    /// TODO: 무슨 방식으로(eg. 랜덤, 맨 위, 맨 아래) 넣을지 구현해야함.
    fn add_card(&mut self, card: Card, insert_type: InsertType) -> Result<(), Exception> {
        if card.get_card_type() != &CardType::Unit {
            panic!("DifferentCardTypes");
        }

        // Zone 에 존재할 수 있는 카드의 갯수를 넘어갈 때
        if self.zone_cards.len() > self.zone_size {
            return Err(Exception::ExceededCardLimit);
        }

        self.zone_cards.add_card(card, insert_type)?;
        Ok(())
    }

    fn get_cards(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: crate::enums::UUID) {
        todo!()
    }
}
