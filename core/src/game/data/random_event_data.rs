use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{
        abnormality_data::AbnormalityMetadata, bonus_data::BonusMetadata, build_string_index,
        build_uuid_index, once_lock_with, shop_data::ShopMetadata, GameDataBase,
    },
    enums::RiskLevel,
    events::event_selection::random::RandomEventType,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl RandomEventDatabase {
    pub fn new(events: Vec<RandomEventMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&events, "random event id", |event| {
            &event.id
        }));
        let by_uuid = once_lock_with(build_uuid_index(&events, "random event uuid", |event| {
            event.uuid
        }));

        Self {
            events,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.events, "random event id", |event| &event.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.events, "random event uuid", |event| event.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&RandomEventMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.events.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&RandomEventMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.events.get(index))
    }
}
