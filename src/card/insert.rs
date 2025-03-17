use uuid::Uuid;

use crate::{enums::HAND_ZONE_SIZE, exception::GameError};

use super::Card;

pub trait Insert {
    fn insert(&self, cards: &mut Vec<Card>, card: Card) -> Result<(), GameError>;
    fn clone_box(&self) -> Box<dyn Insert>;
}

pub struct TopInsert;
pub struct BottomInsert;
pub struct RandomInsert;
pub struct SpecificPositionInsert {
    target_card_uuid: Uuid,
    is_above: bool,
}

// Top 구현
impl Insert for TopInsert {
    fn insert(&self, cards: &mut Vec<Card>, card: Card) -> Result<(), GameError> {
        // TODO: Hand 에 자리가 없거나 등 오류 처리해야함.
        let zone_card_size = cards.len();
        if HAND_ZONE_SIZE <= zone_card_size {
            return Err(GameError::ExceededCardLimit);
        }

        cards.push(card);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(TopInsert)
    }
}

// Bottom 구현
impl Insert for BottomInsert {
    fn insert(&self, cards: &mut Vec<Card>, card: Card) -> Result<(), GameError> {
        cards.insert(0, card);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(BottomInsert)
    }
}

// Random 구현
impl Insert for RandomInsert {
    fn insert(&self, cards: &mut Vec<Card>, card: Card) -> Result<(), GameError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let position = rng.gen_range(0..=cards.len());
        cards.insert(position, card);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(RandomInsert)
    }
}

// 특정 위치 구현
impl SpecificPositionInsert {
    pub fn new(target_card_uuid: Uuid, is_above: bool) -> Self {
        Self {
            target_card_uuid,
            is_above,
        }
    }
}

impl Insert for SpecificPositionInsert {
    fn insert(&self, cards: &mut Vec<Card>, card: Card) -> Result<(), GameError> {
        if let Some(pos) = cards
            .iter()
            .position(|c| c.get_uuid() == self.target_card_uuid)
        {
            let insert_pos = if self.is_above { pos } else { pos + 1 };
            cards.insert(insert_pos, card);
            Ok(())
        } else {
            Err(GameError::CardNotFound)
        }
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(Self {
            target_card_uuid: self.target_card_uuid.clone(),
            is_above: self.is_above,
        })
    }
}
