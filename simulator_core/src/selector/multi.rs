use actix::Addr;

use crate::{
    card::{types::OwnerType, Card},
    exception::GameError,
    game::GameActor,
};

use super::{TargetCondition, TargetCount, TargetSelector};

// 다중 카드 선택기
pub struct MultiCardSelector {
    condition: TargetCondition,
    count: TargetCount,
}

impl TargetSelector for MultiCardSelector {
    /// 다중 카드 선택기 생성자
    ///
    /// # Parameters
    /// * `game` - 게임 객체
    /// * `source` - 이벤트를 발생시킨 카드
    ///
    /// # Returns
    /// * `Ok(Vec<Card>)` - 선택된 카드 목록
    /// * `Err(GameError)` - 카드 선택 중 오류 발생
    ///
    /// # Errors
    fn select_targets(&self, game: Addr<GameActor>, source: &Card) -> Result<Vec<Card>, GameError> {
        todo!()
    }

    fn has_valid_targets(&self, game: Addr<GameActor>, source: &Card) -> bool {
        todo!()
    }

    fn get_target_count(&self) -> TargetCount {
        todo!()
    }

    fn clone_selector(&self) -> Box<dyn TargetSelector> {
        todo!()
    }

    fn get_owner(&self) -> OwnerType {
        todo!()
    }

    fn get_locations(&self) -> Vec<crate::enums::ZoneType> {
        todo!()
    }

    fn is_valid_target(&self, card: &Card, game: Addr<GameActor>, source: &Card) -> bool {
        todo!()
    }
}
