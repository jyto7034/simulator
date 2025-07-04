use super::Card;
use uuid::Uuid;

/// `Card` 타입의 `Vec`에 대한 별칭입니다.
///
/// 카드들의 목록을 나타내는 데 사용됩니다.
///
/// # Examples
///
/// ```
/// use simulator_core::card::Card;
/// use simulator_core::card::cards::Cards;
/// use uuid::Uuid;
///
/// let mut cards: Cards = Vec::new();
/// let card1 = Card { uuid: Uuid::new_v4() };
/// let card2 = Card { uuid: Uuid::new_v4() };
/// cards.push(card1);
/// cards.push(card2);
///
/// assert_eq!(cards.len(), 2);
/// ```
pub type Cards = Vec<Card>;

/// `Vec<Card>`를 확장하는 트레이트입니다.
///
/// 카드 벡터에 대한 추가적인 기능들을 제공합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::card::Card;
/// use simulator_core::card::cards::{Cards, CardVecExt};
/// use uuid::Uuid;
///
/// let mut cards: Cards = Vec::new();
/// let card1 = Card { uuid: Uuid::new_v4() };
/// let card2 = Card { uuid: Uuid::new_v4() };
/// cards.push(card1);
/// cards.push(card2);
///
/// let uuid_to_find = card1.uuid;
///
/// assert_eq!(cards.contains_uuid(uuid_to_find), true);
/// ```
pub trait CardVecExt {
    /// 벡터에 특정 UUID를 가진 카드가 존재하는지 확인합니다.
    ///
    /// # Arguments
    ///
    /// * `uuid` - 확인할 UUID
    ///
    /// # Returns
    ///
    /// * `true` - 벡터에 해당 UUID를 가진 카드가 존재하는 경우
    /// * `false` - 벡터에 해당 UUID를 가진 카드가 존재하지 않는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// assert_eq!(cards.contains_uuid(uuid_to_find), true);
    /// ```
    fn contains_uuid<U: Into<Uuid>>(&self, uuid: U) -> bool;

    /// 벡터에서 특정 UUID를 가진 카드를 찾아 반환합니다 (불변 참조).
    ///
    /// # Arguments
    ///
    /// * `uuid` - 찾을 UUID
    ///
    /// # Returns
    ///
    /// * `Some(&Card)` - 해당 UUID를 가진 카드를 찾은 경우, 해당 카드에 대한 불변 참조를 반환합니다.
    /// * `None` - 해당 UUID를 가진 카드를 찾지 못한 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// if let Some(found_card) = cards.find_by_uuid(uuid_to_find) {
    ///     assert_eq!(found_card.uuid, uuid_to_find);
    /// } else {
    ///     panic!("Card not found");
    /// }
    /// ```
    fn find_by_uuid<U: Into<Uuid>>(&self, uuid: U) -> Option<&Card>;

    /// 벡터에서 특정 UUID를 가진 카드를 찾아 반환합니다 (가변 참조).
    ///
    /// # Arguments
    ///
    /// * `uuid` - 찾을 UUID
    ///
    /// # Returns
    ///
    /// * `Some(&mut Card)` - 해당 UUID를 가진 카드를 찾은 경우, 해당 카드에 대한 가변 참조를 반환합니다.
    /// * `None` - 해당 UUID를 가진 카드를 찾지 못한 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// if let Some(found_card) = cards.find_by_uuid_mut(uuid_to_find) {
    ///     found_card.uuid = Uuid::new_v4(); // Modify the card
    /// } else {
    ///     panic!("Card not found");
    /// }
    /// ```
    fn find_by_uuid_mut<U: Into<Uuid>>(&mut self, uuid: U) -> Option<&mut Card>;

    /// 주어진 조건을 만족하는 모든 카드를 찾아 벡터로 반환합니다 (불변 참조).
    ///
    /// # Arguments
    ///
    /// * `predicate` - 각 카드에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<&Card>` - 조건을 만족하는 카드들의 벡터
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let found_cards = cards.find_all(|card| true); // Find all cards
    ///
    /// assert_eq!(found_cards.len(), 2);
    /// ```
    fn find_all<F>(&self, predicate: F) -> Vec<&Card>
    where
        F: Fn(&Card) -> bool;

    /// 주어진 조건을 만족하는 모든 카드를 찾아 벡터로 반환합니다 (가변 참조).
    ///
    /// # Arguments
    ///
    /// * `predicate` - 각 카드에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<&mut Card>` - 조건을 만족하는 카드들의 벡터
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let mut found_cards = cards.find_all_mut(|card| true); // Find all cards
    ///
    /// for card in &mut found_cards {
    ///     card.uuid = Uuid::new_v4(); // Modify the cards
    /// }
    /// ```
    fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card>
    where
        F: Fn(&Card) -> bool;

    /// 벡터의 카드들을 무작위로 섞습니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// cards.shuffle();
    /// ```
    fn shuffle(&mut self);

    /// 주어진 조건을 만족하는 카드들의 개수를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate` - 각 카드에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `usize` - 조건을 만족하는 카드들의 개수
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let count = cards.count(|card| true); // Count all cards
    ///
    /// assert_eq!(count, 2);
    /// ```
    fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Card) -> bool;
    // 새로 추가할 메소드들

    /// 특정 UUID를 가진 카드를 찾아 벡터에서 제거하고 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `uuid` - 제거할 카드의 UUID
    ///
    /// # Returns
    ///
    /// * `Some(Card)` - 해당 UUID를 가진 카드가 발견되어 제거된 경우
    /// * `None` - 해당 UUID를 가진 카드가 없는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_remove = card1.uuid;
    ///
    /// if let Some(removed_card) = cards.remove_by_uuid(uuid_to_remove) {
    ///     assert_eq!(removed_card.uuid, uuid_to_remove);
    /// } else {
    ///     panic!("Card not found");
    /// }
    /// ```
    fn remove_by_uuid<U: Into<Uuid>>(&mut self, uuid: U) -> Option<Card>;

    /// 특정 조건을 만족하는 모든 카드를 벡터에서 제거하고 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate` - 제거할 카드를 선택하는 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<Card>` - 제거된 카드들의 벡터 (조건을 만족하는 카드가 없으면 빈 벡터)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let removed_cards = cards.remove_all(|card| true); // Remove all cards
    ///
    /// assert_eq!(removed_cards.len(), 2);
    /// ```
    fn remove_all<F>(&mut self, predicate: F) -> Vec<Card>
    where
        F: Fn(&Card) -> bool;
}

impl CardVecExt for Vec<Card> {
    /// 특정 UUID를 가진 카드를 찾아 벡터에서 제거하고 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `uuid` - 제거할 카드의 UUID
    ///
    /// # Returns
    ///
    /// * `Some(Card)` - 해당 UUID를 가진 카드가 발견되어 제거된 경우
    /// * `None` - 해당 UUID를 가진 카드가 없는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_remove = card1.uuid;
    ///
    /// if let Some(removed_card) = cards.remove_by_uuid(uuid_to_remove) {
    ///     assert_eq!(removed_card.uuid, uuid_to_remove);
    /// } else {
    ///     panic!("Card not found");
    /// }
    /// ```
    fn remove_by_uuid<U: Into<Uuid>>(&mut self, uuid: U) -> Option<Card> {
        let uuid = uuid.into();
        let position = self.iter().position(|card| card.uuid == uuid)?;
        Some(self.remove(position))
    }

    /// 특정 조건을 만족하는 모든 카드를 벡터에서 제거하고 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate` - 제거할 카드를 선택하는 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<Card>` - 제거된 카드들의 벡터 (조건을 만족하는 카드가 없으면 빈 벡터)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let removed_cards = cards.remove_all(|card| true); // Remove all cards
    ///
    /// assert_eq!(removed_cards.len(), 2);
    /// ```
    fn remove_all<F>(&mut self, predicate: F) -> Vec<Card>
    where
        F: Fn(&Card) -> bool,
    {
        // 제거할 카드 인덱스들 수집 (역순으로 정렬해야 함)
        let mut indices: Vec<usize> = self
            .iter()
            .enumerate()
            .filter(|(_, card)| predicate(card))
            .map(|(i, _)| i)
            .collect();

        // 역순으로 정렬 (뒤에서부터 제거해야 인덱스가 변하지 않음)
        indices.sort_by(|a, b| b.cmp(a));

        // 카드 제거하고 수집
        let mut removed = Vec::with_capacity(indices.len());
        for index in indices {
            removed.push(self.remove(index));
        }

        // 원래 순서대로 뒤집기
        removed.reverse();
        removed
    }
    /// Vec<Card> 에서 특정 Uuid 를 가진 Card 가 존재하는지 확인합니다.
    ///
    /// # Arguments
    ///
    /// * `uuid`: 확인할 UUID
    ///
    /// # Returns
    ///
    /// * `true` - 존재하는 경우
    /// * `false` - 존재하지 않는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// assert_eq!(cards.contains_uuid(uuid_to_find), true);
    /// ```
    fn contains_uuid<U: Into<Uuid>>(&self, uuid: U) -> bool {
        let uuid = uuid.into();
        self.iter().any(|card| card.uuid == uuid)
    }

    /// Vec<Card> 에서 특정 Uuid 를 가진 Card 를 찾습니다.
    ///
    /// # Arguments
    ///
    /// * `uuid`: 찾을 UUID
    ///
    /// # Returns
    ///
    /// * `Some(&Card)` - 존재하는 경우 해당 Card 의 불변 참조
    /// * `None` - 존재하지 않는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// if let Some(found_card) = cards.find_by_uuid(uuid_to_find) {
    ///     assert_eq!(found_card.uuid, uuid_to_find);
    /// }
    /// ```
    fn find_by_uuid<U: Into<Uuid>>(&self, uuid: U) -> Option<&Card> {
        let uuid = uuid.into();
        self.iter().find(|card| card.uuid == uuid)
    }

    /// Vec<Card> 에서 특정 Uuid 를 가진 Card 를 찾고 가변 참조를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `uuid`: 찾을 UUID
    ///
    /// # Returns
    ///
    /// * `Some(&mut Card)` - 존재하는 경우 해당 Card 의 가변 참조
    /// * `None` - 존재하지 않는 경우
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let uuid_to_find = card1.uuid;
    ///
    /// if let Some(found_card) = cards.find_by_uuid_mut(uuid_to_find) {
    ///     found_card.uuid = Uuid::new_v4(); // modify the card
    /// }
    /// ```
    fn find_by_uuid_mut<U: Into<Uuid>>(&mut self, uuid: U) -> Option<&mut Card> {
        let uuid = uuid.into();
        self.iter_mut().find(|card| card.uuid == uuid)
    }

    /// 주어진 조건을 만족하는 모든 Card 를 찾아 Vec<&Card> 로 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate`: 각 Card 에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<&Card>` - 조건을 만족하는 Card 들의 Vec
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let found_cards = cards.find_all(|card| true); // Find all cards
    ///
    /// assert_eq!(found_cards.len(), 2);
    /// ```
    fn find_all<F>(&self, predicate: F) -> Vec<&Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.iter().filter(|card| predicate(card)).collect()
    }

    /// 주어진 조건을 만족하는 모든 Card 를 찾아 Vec<&mut Card> 로 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate`: 각 Card 에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `Vec<&mut Card>` - 조건을 만족하는 Card 들의 Vec
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let mut found_cards = cards.find_all_mut(|card| true); // Find all cards
    ///
    /// for card in &mut found_cards {
    ///     card.uuid = Uuid::new_v4(); // modify the card
    /// }
    /// ```
    fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.iter_mut().filter(|card| predicate(card)).collect()
    }

    /// 벡터의 카드들을 무작위로 섞습니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// cards.shuffle();
    /// ```
    fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.as_mut_slice().shuffle(&mut rng);
    }

    /// 주어진 조건을 만족하는 카드들의 개수를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `predicate`: 각 카드에 적용할 조건 함수
    ///
    /// # Returns
    ///
    /// * `usize`: 조건을 만족하는 카드들의 개수
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::card::Card;
    /// use simulator_core::card::cards::{Cards, CardVecExt};
    /// use uuid::Uuid;
    ///
    /// let mut cards: Cards = Vec::new();
    /// let card1 = Card { uuid: Uuid::new_v4() };
    /// let card2 = Card { uuid: Uuid::new_v4() };
    /// cards.push(card1);
    /// cards.push(card2);
    ///
    /// let count = cards.count(|card| true); // Count all cards
    ///
    /// assert_eq!(count, 2);
    /// ```
    fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Card) -> bool,
    {
        self.iter().filter(|card| predicate(card)).count()
    }
}