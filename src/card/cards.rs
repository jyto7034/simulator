use crate::enums::UUID;
use super::Card;
use rand::seq::SliceRandom;

/// Vec<Card> 타입의 별칭
pub type Cards = Vec<Card>;

/// Vec<Card> 확장 트레이트
pub trait CardVecExt {
    fn contains_uuid(&self, uuid: UUID) -> bool;
    fn find_by_uuid(&self, uuid: UUID) -> Option<&Card>;
    fn find_by_uuid_mut(&mut self, uuid: UUID) -> Option<&mut Card>;
    fn find_all<F>(&self, predicate: F) -> Vec<&Card> where F: Fn(&Card) -> bool;
    fn find_all_mut<F>(&mut self, predicate: F) -> Vec<&mut Card> where F: Fn(&Card) -> bool;
    fn shuffle(&mut self);
    fn count<F>(&self, predicate: F) -> usize where F: Fn(&Card) -> bool;
}

impl CardVecExt for Vec<Card> {
    fn contains_uuid(&self, uuid: UUID) -> bool {
        self.iter().any(|card| card.uuid == uuid)
    }

    fn find_by_uuid(&self, uuid: UUID) -> Option<&Card> {
        self.iter().find(|card| card.get_uuid() == uuid)
    }

    fn find_by_uuid_mut(&mut self, uuid: UUID) -> Option<&mut Card> {
        self.iter_mut().find(|card| card.get_uuid() == uuid)
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