use crate::exception::exception::Exception;
use crate::unit::entity::Entity;
pub struct Hero {}

impl Entity for Hero {
    fn get_entity_type(&self) -> String {
        "Hero".to_string()
    }

    fn run(&self) -> Result<(), Exception> {
        Ok(())
    }
}

impl Hero {
    pub fn new() -> Hero {
        Hero {}
    }
}
