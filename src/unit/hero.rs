use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::entity::Entity;
pub struct Hero {}

impl Entity for Hero {
    fn get_entity_type(&self) -> String {
        "Hero".to_string()
    }

    fn run(&self, game: &mut Game) -> Result<(), Exception> {
        Ok(())
    }
}

impl Hero {
    pub fn new() -> Hero {
        Hero {}
    }
}
