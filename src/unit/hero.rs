use crate::unit::entity::Entity;

pub struct Hero {}

impl Entity for Hero {
    fn get_entity_type(&self) -> String {
        "Hero".to_string()
    }
}

impl Hero {
    pub fn new() -> Hero {
        Hero {}
    }
}
