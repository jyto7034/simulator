use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::enums::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDatabase {
    pub items: Vec<ArtifactMetadata>,

    #[serde(skip)]
    uuid_map: HashMap<Uuid, ArtifactMetadata>,

    #[serde(skip)]
    id_map: HashMap<String, ArtifactMetadata>,
}

impl ArtifactDatabase {
    pub fn new(items: Vec<ArtifactMetadata>) -> Self {
        let uuid_map = items.iter().map(|item| (item.uuid, item.clone())).collect();
        let id_map = items.iter().map(|item| (item.id.clone(), item.clone())).collect();
        Self { items, uuid_map, id_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.uuid_map = self.items.iter().map(|item| (item.uuid, item.clone())).collect();
        self.id_map = self.items.iter().map(|item| (item.id.clone(), item.clone())).collect();
    }

    /// UUID로 아티팩트 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ArtifactMetadata> {
        self.uuid_map.get(uuid)
    }

    /// ID로 아티팩트 조회 (O(1))
    pub fn get_by_id(&self, id: &str) -> Option<&ArtifactMetadata> {
        self.id_map.get(id)
    }
}

pub type ArtifactItem = ArtifactMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub id: String,       // 고유 식별자 (예: "one_sin", "fairy_festival")
    pub uuid: Uuid,       // 클라이언트 통신용
    pub name: String,
    pub description: String,
    pub rarity: RiskLevel,
    pub price: u32,
}
