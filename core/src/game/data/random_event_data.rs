use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{
        abnormality_data::AbnormalityMetadata, bonus_data::BonusMetadata,
        shop_data::ShopMetadata, GameDataBase,
    },
    enums::RiskLevel,
    events::event_selection::random::RandomEventType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomEventInnerMetadata {
    Shop(Uuid),
    Bonus(Uuid),
    Suppress(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventMetadata {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub event_type: RandomEventType,
    pub risk_level: RiskLevel,
    pub description: String,
    pub image: String,
    pub inner_metadata: RandomEventInnerMetadata,
}

/// RandomEvent 가 실제로 참조하는 도메인 타겟
#[derive(Debug, Clone)]
pub enum RandomEventTarget<'a> {
    Shop(&'a ShopMetadata),
    Bonus(&'a BonusMetadata),
    Suppress(&'a AbnormalityMetadata),
}

impl RandomEventInnerMetadata {
    /// RandomEventInnerMetadata 를 실제 도메인 메타데이터로 해석
    pub fn resolve<'a>(
        &self,
        data: &'a GameDataBase,
    ) -> Result<RandomEventTarget<'a>, crate::game::behavior::GameError> {
        use crate::game::behavior::GameError;

        match self {
            RandomEventInnerMetadata::Shop(uuid) => {
                let shop = data
                    .shop_data
                    .get_by_uuid(uuid)
                    .ok_or(GameError::EventNotFound)?;
                Ok(RandomEventTarget::Shop(shop))
            }
            RandomEventInnerMetadata::Bonus(uuid) => {
                let bonus = data
                    .bonus_data
                    .get_by_uuid(uuid)
                    .ok_or(GameError::EventNotFound)?;
                Ok(RandomEventTarget::Bonus(bonus))
            }
            RandomEventInnerMetadata::Suppress(uuid) => {
                let abnormality = data
                    .abnormality_data
                    .get_by_uuid(uuid)
                    .ok_or(GameError::EventNotFound)?;
                Ok(RandomEventTarget::Suppress(abnormality))
            }
        }
    }
}

/// RON 파일 최상위 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventDatabase {
    pub events: Vec<RandomEventMetadata>,

    #[serde(skip)]
    event_map: HashMap<Uuid, RandomEventMetadata>,
}

impl RandomEventDatabase {
    /// Database 생성 (HashMap 초기화)
    pub fn new(events: Vec<RandomEventMetadata>) -> Self {
        let event_map = events.iter().map(|e| (e.uuid, e.clone())).collect();

        Self { events, event_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.event_map = self.events.iter().map(|e| (e.uuid, e.clone())).collect();
    }

    /// ID로 메타데이터 조회 (여전히 O(n), 자주 사용 안함)
    pub fn get_by_id(&self, id: &str) -> Option<&RandomEventMetadata> {
        self.events.iter().find(|e| e.id == id)
    }

    /// UUID로 메타데이터 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&RandomEventMetadata> {
        self.event_map.get(uuid)
    }
}
