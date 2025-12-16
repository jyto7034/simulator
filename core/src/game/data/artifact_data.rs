use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{enums::RiskLevel, stats::TriggeredEffects};

pub type ArtifactItem = ArtifactMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub rarity: RiskLevel,
    pub price: u32,
    /// 트리거 기반 효과 (Permanent = 상시 적용)
    #[serde(default)]
    pub triggered_effects: TriggeredEffects,
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
