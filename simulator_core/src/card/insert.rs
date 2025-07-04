use uuid::Uuid;

use crate::{exception::{GameError, GameplayError}, zone::zone::Zone};

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
/// 카드를 영역 내 임의의 위치에 삽입하는 전략을 나타내는 구조체입니다.
pub struct RandomInsert;
/// 카드를 특정 카드 위 또는 아래에 삽입하는 전략을 나타내는 구조체입니다.
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
    /// `SpecificPositionInsert` 구조체의 생성자입니다.
    ///
    /// # Arguments
    ///
    /// * `target_card_uuid` - 삽입 위치를 결정하는 대상 카드의 UUID
    /// * `is_above` - `true`이면 대상 카드 위에, `false`이면 대상 카드 아래에 삽입합니다.
    ///
    /// # Returns
    ///
    /// * `SpecificPositionInsert` - 새로 생성된 `SpecificPositionInsert` 인스턴스
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
            Err(GameError::Gameplay(GameplayError::ResourceNotFound { kind: "card", id: self.target_card_uuid.to_string() }))
        }
    }

    fn clone_box(&self) -> Box<dyn Insert> {
        Box::new(Self {
            target_card_uuid: self.target_card_uuid,
            is_above: self.is_above,
        })
    }
}
