use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{ability::AbilityId, enums::RiskLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub risk_level: RiskLevel,
    pub price: u32,
    /// 전투용 기본 최대 체력
    pub max_health: u32,
    /// 전투용 기본 공격력
    pub attack: u32,
    /// 전투용 기본 방어력
    pub defense: u32,
    /// 전투용 기본 공격 주기(ms)
    pub attack_interval_ms: u64,
    /// 이 기물이 보유한 어빌리티 목록
    #[serde(default)]
    pub abilities: Vec<AbilityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityDatabase {
    pub items: Vec<AbnormalityMetadata>,
}

impl AbnormalityDatabase {
    pub fn new(items: Vec<AbnormalityMetadata>) -> Self {
        Self { items }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&AbnormalityMetadata> {
        self.items.iter().find(|item| item.uuid == *uuid)
    }
}
