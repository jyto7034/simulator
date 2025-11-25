use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::enums::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityDatabase {
    pub items: Vec<AbnormalityMetadata>,

    #[serde(skip)]
    uuid_map: HashMap<Uuid, AbnormalityMetadata>,

    #[serde(skip)]
    id_map: HashMap<String, AbnormalityMetadata>,
}

impl AbnormalityDatabase {
    pub fn new(items: Vec<AbnormalityMetadata>) -> Self {
        let uuid_map = items.iter().map(|item| (item.uuid, item.clone())).collect();
        let id_map = items.iter().map(|item| (item.id.clone(), item.clone())).collect();
        Self { items, uuid_map, id_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.uuid_map = self.items.iter().map(|item| (item.uuid, item.clone())).collect();
        self.id_map = self.items.iter().map(|item| (item.id.clone(), item.clone())).collect();
    }

    /// UUID로 환상체 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&AbnormalityMetadata> {
        self.uuid_map.get(uuid)
    }

    /// ID로 환상체 조회 (O(1))
    pub fn get_by_id(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.id_map.get(id)
    }
}

pub type AbnormalityItem = AbnormalityMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityMetadata {
    pub uuid: Uuid,
    pub id: String,   // "F-01-02"
    pub name: String, // "Scorched Girl"
    pub risk_level: RiskLevel,
    pub price: u32,
}
