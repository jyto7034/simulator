use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::enums::{EquipmentType, RiskLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,

    #[serde(skip)]
    uuid_map: HashMap<Uuid, EquipmentMetadata>,

    #[serde(skip)]
    id_map: HashMap<String, EquipmentMetadata>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        let uuid_map = items.iter().map(|item| (item.uuid, item.clone())).collect();
        let id_map = items.iter().map(|item| (item.id.clone(), item.clone())).collect();
        Self { items, uuid_map, id_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.uuid_map = self.items.iter().map(|item| (item.uuid, item.clone())).collect();
        self.id_map = self.items.iter().map(|item| (item.id.clone(), item.clone())).collect();
    }

    /// UUID로 장비 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&EquipmentMetadata> {
        self.uuid_map.get(uuid)
    }

    /// ID로 장비 조회 (O(1))
    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.id_map.get(id)
    }
}

pub type EquipmentItem = EquipmentMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentMetadata {
    pub id: String,       // 고유 식별자 (예: "justitia", "paradise_lost")
    pub uuid: Uuid,       // 클라이언트 통신용
    pub name: String,
    pub equipment_type: EquipmentType,
    pub rarity: RiskLevel,
    pub price: u32,
}
