use crate::enums::UUID;

use super::Card;

/// 다수의 카드를 보다 더 효율적으로 관리하기 위한 구조체입니다.
pub struct Cards {
    pub v_card: Vec<Card>,
}

impl Cards {
    pub fn new_with(cards: Vec<Card>) -> Self {
        Self { v_card: cards }
    }

    /// 새로운 Cards 인스턴스를 생성합니다.
    pub fn new() -> Self {
        Self { v_card: Vec::new() }
    }

    /// 특정 용량으로 Cards 인스턴스를 생성합니다.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            v_card: Vec::with_capacity(capacity),
        }
    }

    /// 카드를 추가합니다.
    pub fn add(&mut self, card: Card) {
        self.v_card.push(card);
    }

    /// 여러 카드를 한번에 추가합니다.
    pub fn add_multiple(&mut self, cards: Vec<Card>) {
        self.v_card.extend(cards);
    }

    /// 특정 인덱스의 카드를 제거하고 반환합니다.
    pub fn remove(&mut self, index: usize) -> Option<Card> {
        if index < self.v_card.len() {
            Some(self.v_card.remove(index))
        } else {
            None
        }
    }

    /// UUID로 카드를 찾습니다.
    pub fn find_by_uuid(&self, uuid: UUID) -> Option<&Card> {
        self.v_card.iter().find(|card| card.get_uuid() == uuid)
    }

    /// UUID로 카드를 찾아 수정 가능한 참조를 반환합니다.
    pub fn find_by_uuid_mut(&mut self, uuid: UUID) -> Option<&mut Card> {
        self.v_card.iter_mut().find(|card| card.get_uuid() == uuid)
    }

    /// 조건에 맞는 모든 카드를 찾습니다.
    pub fn find_all<F>(&self, predicate: F) -> Vec<&Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.v_card.iter().filter(|card| predicate(card)).collect()
    }

    /// 조건에 맞는 모든 카드의 수정 가능한 참조를 반환합니다.
    pub fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.v_card
            .iter_mut()
            .filter(|card| predicate(card))
            .collect()
    }

    /// 카드의 개수를 반환합니다.
    pub fn len(&self) -> usize {
        self.v_card.len()
    }

    /// 카드가 비어있는지 확인합니다.
    pub fn is_empty(&self) -> bool {
        self.v_card.is_empty()
    }

    /// 모든 카드를 제거합니다.
    pub fn clear(&mut self) {
        self.v_card.clear();
    }

    /// 카드들을 섞습니다.
    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.v_card.shuffle(&mut rng);
    }

    /// 카드들을 정렬합니다.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Card, &Card) -> std::cmp::Ordering,
    {
        self.v_card.sort_by(compare);
    }

    /// 특정 조건을 만족하는 카드의 개수를 반환합니다.
    pub fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Card) -> bool,
    {
        self.v_card.iter().filter(|card| predicate(card)).count()
    }

    /// 첫 번째 카드를 가져옵니다.
    pub fn first(&self) -> Option<&Card> {
        self.v_card.first()
    }

    /// 마지막 카드를 가져옵니다.
    pub fn last(&self) -> Option<&Card> {
        self.v_card.last()
    }

    /// 특정 범위의 카드들을 가져옵니다.
    pub fn get_range(&self, range: std::ops::Range<usize>) -> Option<&[Card]> {
        self.v_card.get(range)
    }
}

// FromIterator 구현
impl FromIterator<Card> for Cards {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        Self {
            v_card: iter.into_iter().collect(),
        }
    }
}

// Clone 구현 (Card가 Clone을 구현한다고 가정)
impl Clone for Cards {
    fn clone(&self) -> Self {
        Self {
            v_card: self.v_card.clone(),
        }
    }
}

// Default 구현
impl Default for Cards {
    fn default() -> Self {
        Self::new()
    }
}
