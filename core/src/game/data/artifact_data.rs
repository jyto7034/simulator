use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{enums::RiskLevel, stats::StatModifier};

pub type ArtifactItem = ArtifactMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub rarity: RiskLevel,
    pub price: u32,
    /// 이 아티팩트가 부여하는 스탯 변경 목록 (덱 전체 또는 특정 조건)
    #[serde(default)]
    pub modifiers: Vec<StatModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDatabase {
    pub items: Vec<ArtifactMetadata>,
}

impl ArtifactDatabase {
    pub fn new(items: Vec<ArtifactMetadata>) -> Self {
        Self { items }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&ArtifactMetadata> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ArtifactMetadata> {
        self.items.iter().find(|item| item.uuid == *uuid)
    }
}
