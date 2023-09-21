use crate::exception::exception::Exception;

pub trait Entity {
    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }

    fn run(&self) -> Result<(), Exception>;
}
