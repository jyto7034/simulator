// 환상체 (기물) 정보
pub mod abnormality_data;

// 아티팩트 정보
pub mod artifact_data;

// 보너스 이벤트 정보
pub mod bonus_data;

// 아이템 ( 장착 장비 ) 정보
pub mod equipment_data;

// 이벤트 생성을 위한 pools
pub mod event_pools;

// 랜덤 인카운트 이벤트 정보
pub mod random_event_data;

// 상점 정보
pub mod shop_data;

use std::{collections::HashMap, sync::Arc};
use serde::{Deserialize, Serialize};

use abnormality_data::AbnormalityDatabase;
use artifact_data::ArtifactDatabase;
use bonus_data::BonusDatabase;
use equipment_data::EquipmentDatabase;
use random_event_data::RandomEventDatabase;
use shop_data::ShopDatabase;
use uuid::Uuid;

use crate::game::data::{
    abnormality_data::AbnormalityMetadata, artifact_data::ArtifactMetadata,
    equipment_data::EquipmentMetadata, event_pools::EventPoolConfig,
};

/// 모든 게임 데이터를 담는 구조체
///
/// NOTE: 이 데이터는 game_server에서 로드되어 GameCore에 전달됩니다.
/// game_server에서 Arc<GameData>로 공유하여 메모리 효율적으로 사용합니다.
pub struct GameData {
    pub shops_db: ShopDatabase,
    pub event_pools: EventPoolConfig,
    pub bonuses: BonusDatabase,
    pub random_events: RandomEventDatabase,
    pub abnormalities: AbnormalityDatabase,
    pub equipments: EquipmentDatabase,
    pub artifacts: ArtifactDatabase,

    pub item_uuid_map: HashMap<Uuid, ItemReference>,
}

impl GameData {
    pub fn build_item_uuid_map(&mut self) {
        // 1. Abnormality DB에서 추가
        for abnormality in &self.abnormalities.items {
            let arc_data = Arc::new(abnormality.clone());
            self.item_uuid_map
                .insert(abnormality.uuid, ItemReference::Abnormality(arc_data));
        }

        // 2. Equipment DB에서 추가
        for equipment in &self.equipments.items {
            let arc_data = Arc::new(equipment.clone());
            self.item_uuid_map
                .insert(equipment.uuid, ItemReference::Equipment(arc_data));
        }

        // 3. Artifact DB에서 추가
        for artifact in &self.artifacts.items {
            let arc_data = Arc::new(artifact.clone());
            self.item_uuid_map
                .insert(artifact.uuid, ItemReference::Artifact(arc_data));
        }

        tracing::info!(
            "ItemReference UUID Map 구축 완료: {} items",
            self.item_uuid_map.len()
        );
    }

    /// 어떤 타입의 아이템인지 모를 때 사용
    pub fn get_item_by_uuid(&self, uuid: &Uuid) -> Option<&ItemReference> {
        self.item_uuid_map.get(uuid)
    }

    /// UUID로 아이템 가격 조회 (O(1))
    pub fn get_item_price(&self, uuid: &Uuid) -> Option<u32> {
        self.get_item_by_uuid(uuid).map(|item| item.price())
    }

    /// UUID로 아이템 이름 조회 (O(1))
    pub fn get_item_name(&self, uuid: &Uuid) -> Option<&str> {
        self.get_item_by_uuid(uuid).map(|item| item.name())
    }

    // TODO: 아래 3개 함수 굳이 필요한가 싶음. 리팩토링 여지 있음

    /// ShopProduct에서 환상체 데이터 조회 (레거시 - 필요시 사용)
    pub fn get_abnormality_from_product(
        &self,
        id: &str,
    ) -> Option<&abnormality_data::AbnormalityMetadata> {
        self.abnormalities.get_by_id(id)
    }

    /// ShopProduct에서 장비 데이터 조회 (레거시)
    pub fn get_equipment_from_product(
        &self,
        id: &str,
    ) -> Option<&equipment_data::EquipmentMetadata> {
        self.equipments.get_by_id(id)
    }

    /// ShopProduct에서 아티팩트 데이터 조회 (레거시)
    pub fn get_artifact_from_product(&self, id: &str) -> Option<&artifact_data::ArtifactMetadata> {
        self.artifacts.get_by_id(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemReference {
    Abnormality(Arc<AbnormalityMetadata>),
    Equipment(Arc<EquipmentMetadata>),
    Artifact(Arc<ArtifactMetadata>),
}
impl ItemReference {
    /// 공통 메타데이터 접근자들

    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Abnormality(data) => data.uuid,
            Self::Equipment(data) => data.uuid,
            Self::Artifact(data) => data.uuid,
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Self::Abnormality(data) => data.price,
            Self::Equipment(data) => data.price,
            Self::Artifact(data) => data.price,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Abnormality(data) => &data.name,
            Self::Equipment(data) => &data.name,
            Self::Artifact(data) => &data.name,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Abnormality(data) => &data.id,
            Self::Equipment(data) => &data.id,
            Self::Artifact(data) => &data.id,
        }
    }

    /// 타입 확인 헬퍼
    pub fn is_abnormality(&self) -> bool {
        matches!(self, Self::Abnormality(_))
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self, Self::Equipment(_))
    }

    pub fn is_artifact(&self) -> bool {
        matches!(self, Self::Artifact(_))
    }

    /// 타입별 상세 데이터 접근 (필요시)
    pub fn as_abnormality(&self) -> Option<&Arc<AbnormalityMetadata>> {
        match self {
            Self::Abnormality(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_equipment(&self) -> Option<&Arc<EquipmentMetadata>> {
        match self {
            Self::Equipment(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_artifact(&self) -> Option<&Arc<ArtifactMetadata>> {
        match self {
            Self::Artifact(data) => Some(data),
            _ => None,
        }
    }
}
