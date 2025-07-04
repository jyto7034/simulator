use uuid::Uuid;

use crate::{exception::{GameError, SystemError, GameplayError, DeckError}, selector::TargetCount, zone::zone::Zone};

use super::Card;
use crate::card::cards::CardVecExt;

pub trait Take: Send + Sync {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError>;
    fn clone_box(&self) -> Box<dyn Take>;
}

// TopTake: 덱/존의 위에서 카드를 가져옴
#[derive(Clone)]
pub struct TopTake(pub TargetCount);

/// `TopTake` 구조체를 위한 `Take` 트레이트 구현입니다.
///
/// 덱이나 영역의 위에서부터 지정된 개수의 카드를 가져옵니다.
///
/// # Errors
///
/// * `GameError::System(SystemError::Internal)`: 사용 가능한 카드 수가 요청한 카드 수보다 적을 경우 발생합니다.
// TODO: 오류 발생 조건에 대한 더 자세한 설명이 필요합니다. 예를 들어, `TargetCount`의 설정에 따라 어떤 경우에 오류가 발생하는지 명확히 기술해야 합니다.
// TODO: 성능 개선을 위해 `drain` 대신 다른 방법을 사용할 수 있는지 검토해 볼 수 있습니다.
impl Take for TopTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::System(SystemError::Internal("Not enough cards available".to_string())));
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

/// `BottomTake` 구조체를 위한 `Take` 트레이트 구현입니다.
///
/// 덱이나 영역의 아래에서부터 지정된 개수의 카드를 가져옵니다.
///
/// # Errors
///
/// * `GameError::System(SystemError::Internal)`: 사용 가능한 카드 수가 요청한 카드 수보다 적을 경우 발생합니다.
// TODO: 오류 발생 조건에 대한 더 자세한 설명이 필요합니다. 예를 들어, `TargetCount`의 설정에 따라 어떤 경우에 오류가 발생하는지 명확히 기술해야 합니다.
// TODO: 성능 개선을 위해 `drain` 대신 다른 방법을 사용할 수 있는지 검토해 볼 수 있습니다.
impl Take for BottomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::System(SystemError::Internal("Not enough cards available".to_string())));
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

/// `RandomTake` 구조체를 위한 `Take` 트레이트 구현입니다.
///
/// 덱이나 영역에서 무작위로 지정된 개수의 카드를 가져옵니다.
///
/// # Errors
///
/// * `GameError::System(SystemError::Internal)`: 사용 가능한 카드 수가 요청한 카드 수보다 적을 경우 발생합니다.
// TODO: 오류 발생 조건에 대한 더 자세한 설명이 필요합니다. 예를 들어, `TargetCount`의 설정에 따라 어떤 경우에 오류가 발생하는지 명확히 기술해야 합니다.
// TODO: 무작위 선택 알고리즘의 시간 복잡도에 대한 언급이 필요합니다. 큰 덱에서 성능 문제가 발생할 수 있는지 고려해야 합니다.
// TODO: 현재 구현은 무작위로 선택된 카드를 제거된 순서의 역순으로 반환합니다. 이는 의도된 동작인지 명확히 해야 하며, 필요에 따라 원래 뽑힌 순서대로 반환하도록 수정할 수 있습니다.
impl Take for RandomTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();
        let available = cards.len();

        let count = calculate_take_count(self.0, available)?;

        if count == 0 {
            return Ok(Vec::new());
        }
        if count > available {
            return Err(GameError::System(SystemError::Internal("Not enough cards available".to_string())));
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

/// `SpecificTake` 구조체를 위한 `Take` 트레이트 구현입니다.
///
/// 특정 UUID를 가진 카드를 덱이나 영역에서 가져옵니다.
///
/// # Errors
///
/// * `GameError::Gameplay(GameplayError::ResourceNotFound)`: 지정된 UUID를 가진 카드가 없을 경우 발생합니다.
// TODO: 만약 동일한 UUID를 가진 카드가 여러 장 존재할 경우, 어떤 카드가 선택되는지에 대한 설명이 필요합니다.
// TODO: 만약 특정 UUID를 가진 카드가 여러 장 존재할 경우, 모든 카드를 가져오는 기능을 추가할 수 있습니다.
impl Take for SpecificTake {
    fn take(&mut self, zone: &mut dyn Zone) -> Result<Vec<Card>, GameError> {
        let cards = zone.get_cards_mut();

        match cards.remove_by_uuid(self.0) {
            Some(card) => Ok(vec![card]),
            None => Err(GameError::Gameplay(GameplayError::ResourceNotFound { kind: "card", id: self.0.to_string() })),
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
                Err(GameError::Gameplay(GameplayError::DeckError(DeckError::NoCardsLeftToDraw)))
            } else {
                Ok(n)
            }
        }
        TargetCount::Range(low, high) => {
            if available < low {
                // 최소 요구량보다 적으면 에러 또는 0 반환
                Err(GameError::Gameplay(GameplayError::DeckError(DeckError::NoCardsLeftToDraw)))
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
