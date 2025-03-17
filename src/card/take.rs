use uuid::Uuid;

use crate::{exception::GameError, selector::TargetCount, zone::zone::Zone};

use super::Card;

pub trait Take {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError>;
    fn clone_box(&self) -> Box<dyn Take>;
}

pub struct TopTake(pub TargetCount);
pub struct BottomTake(pub TargetCount);
pub struct RandomTake(pub TargetCount);
pub struct SpecificTake(pub Uuid);

use std::cmp::min;
impl Take for TopTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        // TargetCount variant에 따라 실제로 가져올 카드의 수를 결정합니다.
        let count = match self.0 {
            TargetCount::Exact(n) => min(n, available),
            TargetCount::Range(low, high) => {
                if available < low {
                    // 만약 최소 필요한 카드 수보다 적으면 아무것도 가져오지 않음
                    0
                } else {
                    min(high, available)
                }
            }
            TargetCount::Any => available,
            TargetCount::None => 0,
        };

        // 카드 집합의 앞부분에서 결정된 개수만큼 카드들을 drainage 하여 소유권을 가져옵니다.
        Ok(cards.drain(available - count..).collect())
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for BottomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = match self.0 {
            TargetCount::Exact(n) => min(n, available),
            TargetCount::Range(low, high) => {
                if available < low {
                    0
                } else {
                    min(high, available)
                }
            }
            TargetCount::Any => available,
            TargetCount::None => 0,
        };

        Ok(cards.drain(0..count).collect())
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for RandomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for SpecificTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}
