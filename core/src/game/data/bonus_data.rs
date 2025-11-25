use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::events::event_selection::bonus::BonusType;

/// 보너스 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusMetadata {
    pub bonus_type: BonusType,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub min_amount: u32,
    pub max_amount: u32,
}

/// RON 파일 최상위 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusDatabase {
    pub bonuses: Vec<BonusMetadata>,

    #[serde(skip)]
    bonus_map: HashMap<Uuid, BonusMetadata>,
}

impl BonusDatabase {
    /// Database 생성 (HashMap 초기화)
    pub fn new(bonuses: Vec<BonusMetadata>) -> Self {
        let bonus_map = bonuses.iter().map(|b| (b.uuid, b.clone())).collect();

        Self { bonuses, bonus_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.bonus_map = self.bonuses.iter().map(|b| (b.uuid, b.clone())).collect();
    }

    /// BonusType으로 메타데이터 조회 (여전히 O(n), 자주 사용 안함)
    pub fn get_by_type(&self, bonus_type: &BonusType) -> Option<&BonusMetadata> {
        self.bonuses.iter().find(|b| &b.bonus_type == bonus_type)
    }

    /// UUID로 메타데이터 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&BonusMetadata> {
        self.bonus_map.get(uuid)
    }
}
