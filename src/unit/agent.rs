use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::entity::Entity;

pub struct Agent {}

impl Entity for Agent {
    fn get_entity_type(&self) -> String {
        "Agent".to_string()
    }

    fn run(&self, _game: &mut Game) -> Result<(), Exception> {
        Ok(())
    }
}

impl Agent {
    pub fn new() -> Agent {
        Agent {}
    }
}
