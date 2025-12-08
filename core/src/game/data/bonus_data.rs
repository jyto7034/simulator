use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    Enkephalin,
    Experience,
    Item,
    Abnormality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusMetadata {
    pub id: String,
    pub bonus_type: BonusType,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusDatabase {
    pub bonuses: Vec<BonusMetadata>,
}

impl BonusDatabase {
    pub fn new(bonuses: Vec<BonusMetadata>) -> Self {
        Self { bonuses }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&BonusMetadata> {
        self.bonuses.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&BonusMetadata> {
        self.bonuses.iter().find(|item| item.uuid == *uuid)
    }
}
