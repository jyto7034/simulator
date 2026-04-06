use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::{build_string_index, build_uuid_index, once_lock_with};

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
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl BonusDatabase {
    pub fn new(bonuses: Vec<BonusMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&bonuses, "bonus id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&bonuses, "bonus uuid", |item| item.uuid));

        Self {
            bonuses,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.bonuses, "bonus id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.bonuses, "bonus uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&BonusMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.bonuses.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&BonusMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.bonuses.get(index))
    }
}
