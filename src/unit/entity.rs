use crate::exception::exception::Exception;
use crate::game::Game;
pub trait Entity {
    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }

    fn run(&self, game: &mut Game) -> Result<(), Exception>;
}
