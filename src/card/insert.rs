use uuid::Uuid;

use crate::{exception::GameError, zone::zone::Zone};

use super::Card;

pub trait Insert: Send + Sync {
    /// 카드를 지정된 영역에 삽입합니다.
    ///
    /// # Arguments
    /// * `zone` - 카드를 삽입할 영역
    /// * `card` - 삽입할 카드
    ///
    /// # Returns
    /// * `Result<(), GameError>` - 삽입 성공 여부
    fn insert(&self, zone: &mut dyn Zone, card: Card) -> Result<(), GameError>;

    /// 자기 자신의 복제본을 Box로 반환합니다.
    fn clone_box(&self) -> Box<dyn Insert>;
}
pub struct GeneralInsert;
pub struct TopInsert;
pub struct BottomInsert;
pub struct RandomInsert;
pub struct SpecificPositionInsert {
    target_card_uuid: Uuid,
    is_above: bool,
}

// Top 구현
impl Insert for TopInsert {
    fn insert(&self, zone: &mut dyn Zone, card: Card) -> Result<(), GameError> {
        let cards = zone.get_cards_mut();

        // 영역 용량 확인 (Zone 타입에 따라 다르게 처리할 수 있음)

        cards.push(card);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(TopInsert)
    }
}

// Bottom 구현
impl Insert for BottomInsert {
    fn insert(&self, zone: &mut dyn Zone, card: Card) -> Result<(), GameError> {
        let cards = zone.get_cards_mut();

        // 영역 용량 확인

        cards.insert(0, card);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(BottomInsert)
    }
}

// Random 구현
impl Insert for RandomInsert {
    fn insert(&self, zone: &mut dyn Zone, card: Card) -> Result<(), GameError> {
        use rand::Rng;
        let cards = zone.get_cards_mut();

        // 영역 용량 확인

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
    fn insert(&self, zone: &mut dyn Zone, card: Card) -> Result<(), GameError> {
        let cards = zone.get_cards_mut();

        // 영역 용량 확인

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
            target_card_uuid: self.target_card_uuid,
            is_above: self.is_above,
        })
    }
}
