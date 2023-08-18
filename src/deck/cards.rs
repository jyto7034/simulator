use crate::deck::Card;
use crate::enums::constant;

/// 다수의 카드를 보다 더 효율적으로 관리하기 위한 구조체입니다.
/// 예를 들어 카드 서치, 수정 등이 있습니다.
pub struct Cards {
    pub v_card: Vec<Card>,
}

impl Cards {
    pub fn dummy() -> Cards {
        Cards { v_card: vec![] }
    }

    pub fn get_card_count(&self) -> u32 {
        constant::MAX_CARD_SIZE
    }

    pub fn empty(&self) -> bool {
        false
    }
}
