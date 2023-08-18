pub trait Entity {
    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }
}
