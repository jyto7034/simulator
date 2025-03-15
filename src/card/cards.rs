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
}

impl CardVecExt for Vec<Card> {
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
