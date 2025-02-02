use crate::{enums::UUID, zone::zone::Zone};

use super::Card;

pub trait Take{
    fn take(&self, zone: Box<dyn Zone>) -> Vec<Card>;
    fn clone_box(&self) -> Box<dyn Take>;
}

pub struct TopTake;
pub struct BottomTake;
pub struct RandomTake;
pub struct SpecificTake(UUID);

impl Take for TopTake{
    fn take(&self, zone: Box<dyn Zone>) -> Vec<Card> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for BottomTake{
    fn take(&self, zone: Box<dyn Zone>) -> Vec<Card> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for RandomTake{
    fn take(&self, zone: Box<dyn Zone>) -> Vec<Card> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}

impl Take for SpecificTake{
    fn take(&self, zone: Box<dyn Zone>) -> Vec<Card> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Take> {
        todo!()
    }
}