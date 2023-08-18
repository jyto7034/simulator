use crate::unit::entity::Entity;

pub struct Agent {}

impl Entity for Agent {
    fn get_entity_type(&self) -> String {
        "Agent".to_string()
    }
}

impl Agent {
    pub fn new() -> Agent {
        Agent {}
    }
}
