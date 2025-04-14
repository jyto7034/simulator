use uuid::Uuid;

use crate::{exception::GameError, selector::TargetCount, zone::zone::Zone};

use super::Card;
use crate::card::cards::CardVecExt;

pub trait Take: Send + Sync {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError>;
    fn clone_box(&self) -> Box<dyn Take>;
}

// TopTake: 덱/존의 위에서 카드를 가져옴
#[derive(Clone)]
pub struct TopTake(pub TargetCount);

impl Take for TopTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::InternalServerError);
        }

        // drain은 앞에서부터 제거하므로, 뒤에서부터 가져오려면 인덱스 계산 필요
        let start_index = available.saturating_sub(count);
        Ok(cards.drain(start_index..).collect()) // drain은 역순으로 반환하지 않음, 순서 유지됨
    }

    fn clone_box(&self) -> Box<dyn Take> {
        Box::new(self.clone()) // Clone을 이용해 간단히 구현
    }
}

// BottomTake: 덱/존의 아래에서 카드를 가져옴
#[derive(Clone)]
pub struct BottomTake(pub TargetCount);

impl Take for BottomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::InternalServerError);
        }

        // 앞에서부터 count만큼 제거
        Ok(cards.drain(0..count).collect())
    }

    fn clone_box(&self) -> Box<dyn Take> {
        Box::new(self.clone())
    }
}

// RandomTake: 덱/존에서 무작위로 카드를 가져옴
#[derive(Clone)]
pub struct RandomTake(pub TargetCount);

impl Take for RandomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::InternalServerError);
        }

        // 1. 무작위 인덱스를 count만큼 중복 없이 뽑기
        let mut rng = rand::thread_rng();
        let indices_to_take: Vec<usize> =
            rand::seq::index::sample(&mut rng, available, count).into_vec();

        // 2. 인덱스를 내림차순으로 정렬하여 제거 시 다른 인덱스에 영향을 주지 않도록 함
        let mut sorted_indices = indices_to_take;
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a)); // 내림차순 정렬

        // 3. 정렬된 인덱스를 사용하여 카드 제거 및 수집
        let mut taken_cards = Vec::with_capacity(count);
        for index in sorted_indices {
            // remove는 요소를 제거하고 뒤의 요소들을 앞으로 당김
            taken_cards.push(cards.remove(index));
        }

        // 4. 원래 뽑힌 순서대로 돌려주려면 taken_cards를 reverse 해야 함 (선택적)
        // 현재는 제거된 순서의 역순으로 반환됨. 무작위 선택이므로 순서가 중요하지 않을 수 있음.
        // taken_cards.reverse();

        Ok(taken_cards)
    }

    fn clone_box(&self) -> Box<dyn Take> {
        Box::new(self.clone())
    }
}

// SpecificTake: 특정 UUID의 카드를 가져옴
#[derive(Clone)]
pub struct SpecificTake(pub Uuid);

impl Take for SpecificTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();

        match cards.remove_by_uuid(self.0) {
            Some(card) => Ok(vec![card]),
            None => Err(GameError::CardNotFound),
        }
    }

    fn clone_box(&self) -> Box<dyn Take> {
        Box::new(self.clone())
    }
}

// --- Helper Function ---

/// 가져올 카드의 수를 계산하는 헬퍼 함수
fn calculate_take_count(target_count: TargetCount, available: usize) -> Result<usize, GameError> {
    use std::cmp::min; // 함수 내에서만 사용

    match target_count {
        TargetCount::Exact(n) => {
            if n > available {
                // 정확히 n개를 가져와야 하는데 부족한 경우 -> 에러 처리 또는 정책 결정 필요
                // 여기서는 일단 에러로 처리 (혹은 0개를 반환할 수도 있음)
                Err(GameError::NotEnoughCards) // 더 구체적인 에러 타입 정의 필요
            } else {
                Ok(n)
            }
        }
        TargetCount::Range(low, high) => {
            if available < low {
                // 최소 요구량보다 적으면 에러 또는 0 반환
                Err(GameError::NotEnoughCards) // 또는 Ok(0)
            } else {
                // low 이상 high 이하, 그리고 available 이하의 값을 반환
                Ok(min(high, available))
            }
        }
        TargetCount::Any => {
            // 가능한 모든 카드
            Ok(available)
        }
        TargetCount::None => {
            // 0개
            Ok(0)
        }
    }
}

// --- 새로운 에러 타입 정의 (선택적) ---
// exception/mod.rs 에 추가
// #[derive(Debug, PartialEq, Clone)]
// pub enum GameError {
//     // ... 기존 에러들 ...
//     NotEnoughCards(usize, usize), // 필요한 카드 수, 실제 있는 카드 수
//     // ...
// }
//
// impl fmt::Display for GameError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             // ...
//             Self::NotEnoughCards(needed, available) => write!(f, "Not enough cards: needed {}, available {}", needed, available),
//             // ...
//         }
//     }
// }
