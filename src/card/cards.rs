use super::Card;
use uuid::Uuid;

/// Vec<Card> 타입의 별칭
pub type Cards = Vec<Card>;

/// Vec<Card> 확장 트레이트
pub trait CardVecExt {
    fn contains_uuid<U: Into<Uuid>>(&self, uuid: U) -> bool;
    fn find_by_uuid<U: Into<Uuid>>(&self, uuid: U) -> Option<&Card>;
    fn find_by_uuid_mut<U: Into<Uuid>>(&mut self, uuid: U) -> Option<&mut Card>;
    fn find_all<F>(&self, predicate: F) -> Vec<&Card>
    where
        F: Fn(&Card) -> bool;
    fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card>
    where
        F: Fn(&Card) -> bool;
    fn shuffle(&mut self);
    fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Card) -> bool;
    // 새로 추가할 메소드들
    fn remove_by_uuid<U: Into<Uuid>>(&mut self, uuid: U) -> Option<Card>;
    fn remove_all<F>(&mut self, predicate: F) -> Vec<Card>
    where
        F: Fn(&Card) -> bool;
}

impl CardVecExt for Vec<Card> {
    /// 특정 UUID를 가진 카드를 찾아 벡터에서 제거하고 반환합니다.
    /// # Returns
    /// * `Some(Card)` - 해당 UUID를 가진 카드가 발견되어 제거된 경우
    /// * `None` - 해당 UUID를 가진 카드가 없는 경우
    fn remove_by_uuid<U: Into<Uuid>>(&mut self, uuid: U) -> Option<Card> {
        let uuid = uuid.into();
        let position = self.iter().position(|card| card.uuid == uuid)?;
        Some(self.remove(position))
    }

    /// 특정 조건을 만족하는 모든 카드를 벡터에서 제거하고 반환합니다.
    /// # Returns
    /// * `Vec<Card>` - 제거된 카드들의 벡터 (조건을 만족하는 카드가 없으면 빈 벡터)
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
    /// # RETURNS
    /// * `true` - 존재하는 경우
    /// * `false` - 존재하지 않는 경우
    fn contains_uuid<U: Into<Uuid>>(&self, uuid: U) -> bool {
        let uuid = uuid.into();
        self.iter().any(|card| card.uuid == uuid)
    }

    fn find_by_uuid<U: Into<Uuid>>(&self, uuid: U) -> Option<&Card> {
        let uuid = uuid.into();
        self.iter().find(|card| card.uuid == uuid)
    }

    fn find_by_uuid_mut<U: Into<Uuid>>(&mut self, uuid: U) -> Option<&mut Card> {
        let uuid = uuid.into();
        self.iter_mut().find(|card| card.uuid == uuid)
    }

    fn find_all<F>(&self, predicate: F) -> Vec<&Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.iter().filter(|card| predicate(card)).collect()
    }

    fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card>
    where
        F: Fn(&Card) -> bool,
    {
        self.iter_mut().filter(|card| predicate(card)).collect()
    }

    fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.as_mut_slice().shuffle(&mut rng);
    }

    fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Card) -> bool,
    {
        self.iter().filter(|card| predicate(card)).count()
    }
}
