use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::game::data::{
    abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
    artifact_data::{ArtifactDatabase, ArtifactMetadata},
    bonus_data::BonusDatabase,
    equipment_data::{EquipmentDatabase, EquipmentMetadata},
    event_pools::EventPoolConfig,
    pve_data::PveEncounterDatabase,
    random_event_data::RandomEventDatabase,
    skill_data::SkillDatabase,
    shop_data::ShopDatabase,
};

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

// PvE 전투 데이터
pub mod pve_data;

// 랜덤 인카운트 이벤트 정보
pub mod random_event_data;

// 스킬 정보
pub mod skill_data;

// 상점 정보
pub mod shop_data;

pub struct GameDataBase {
    /// 환상체, 장비, 아티팩트 Raw 데이터를 저장하는 마스터 테이블
    pub abnormality_data: Arc<AbnormalityDatabase>,
    pub artifact_data: Arc<ArtifactDatabase>,
    pub equipment_data: Arc<EquipmentDatabase>,

    /// 마스터 테이블을 참고하여 상인, 랜덤 이벤트, 보너스 등을 구성하여 저장하는 게임 데이터베이스
    pub shop_data: Arc<ShopDatabase>,
    pub bonus_data: Arc<BonusDatabase>,
    pub random_event_data: Arc<RandomEventDatabase>,

    /// PvE 전투(Suppress) 데이터
    pub pve_data: Arc<PveEncounterDatabase>,

    /// 스킬 메타데이터 DB
    pub skill_data: Arc<SkillDatabase>,

    /// 이벤트 생성을 위한 가중치 풀 (Ordeal별 이벤트 확률)
    pub event_pools: EventPoolConfig,

    /// UUID 기반 아이템 조회 레지스트리
    pub item_registry: ItemRegistry,
}

#[derive(Debug, Clone)]
pub enum Item {
    Equipment(Arc<EquipmentMetadata>),
    Artifact(Arc<ArtifactMetadata>),
    Abnormality(Arc<AbnormalityMetadata>),
}

impl Item {
    // ============================================================
    // 공통 속성 접근자
    // ============================================================

    /// 아이템 가격 반환
    pub fn price(&self) -> u32 {
        match self {
            Item::Equipment(meta) => meta.price,
            Item::Artifact(meta) => meta.price,
            Item::Abnormality(meta) => meta.price,
        }
    }

    /// 아이템 UUID 반환
    pub fn uuid(&self) -> Uuid {
        match self {
            Item::Equipment(meta) => meta.uuid,
            Item::Artifact(meta) => meta.uuid,
            Item::Abnormality(meta) => meta.uuid,
        }
    }

    /// 아이템 ID 반환
    pub fn id(&self) -> &str {
        match self {
            Item::Equipment(meta) => &meta.id,
            Item::Artifact(meta) => &meta.id,
            Item::Abnormality(meta) => &meta.id,
        }
    }

    /// 아이템 이름 반환
    pub fn name(&self) -> &str {
        match self {
            Item::Equipment(meta) => &meta.name,
            Item::Artifact(meta) => &meta.name,
            Item::Abnormality(meta) => &meta.name,
        }
    }

    // ============================================================
    // 타입 확인
    // ============================================================

    /// Equipment 타입인지 확인
    pub fn is_equipment(&self) -> bool {
        matches!(self, Item::Equipment(_))
    }

    /// Artifact 타입인지 확인
    pub fn is_artifact(&self) -> bool {
        matches!(self, Item::Artifact(_))
    }

    /// Abnormality 타입인지 확인
    pub fn is_abnormality(&self) -> bool {
        matches!(self, Item::Abnormality(_))
    }

    // ============================================================
    // 타입 변환 (참조)
    // ============================================================

    /// Equipment로 변환 (참조)
    pub fn as_equipment(&self) -> Option<Arc<EquipmentMetadata>> {
        match self {
            Item::Equipment(meta) => Some(meta.clone()),
            _ => None,
        }
    }

    /// Artifact로 변환 (참조)
    pub fn as_artifact(&self) -> Option<Arc<ArtifactMetadata>> {
        match self {
            Item::Artifact(meta) => Some(meta.clone()),
            _ => None,
        }
    }

    /// Abnormality로 변환 (참조)
    pub fn as_abnormality(&self) -> Option<Arc<AbnormalityMetadata>> {
        match self {
            Item::Abnormality(meta) => Some(meta.clone()),
            _ => None,
        }
    }

    // ============================================================
    // 유틸리티
    // ============================================================

    /// Arc 복제 (메모리 효율적)
    pub fn clone_arc(&self) -> Self {
        match self {
            Item::Equipment(meta) => Item::Equipment(Arc::clone(meta)),
            Item::Artifact(meta) => Item::Artifact(Arc::clone(meta)),
            Item::Abnormality(meta) => Item::Abnormality(Arc::clone(meta)),
        }
    }
}

/// Uuid -> Item 매핑을 제공하는 전역 레지스트리
#[derive(Debug, Default)]
pub struct ItemRegistry {
    by_uuid: HashMap<Uuid, Item>,
}

impl ItemRegistry {
    pub fn new(
        abnormality_db: &AbnormalityDatabase,
        artifact_db: &ArtifactDatabase,
        equipment_db: &EquipmentDatabase,
    ) -> Self {
        let mut by_uuid = HashMap::new();

        // 환상체 아이템 등록
        for meta in &abnormality_db.items {
            by_uuid.insert(meta.uuid, Item::Abnormality(Arc::new(meta.clone())));
        }

        // 아티팩트 아이템 등록
        for meta in &artifact_db.items {
            by_uuid.insert(meta.uuid, Item::Artifact(Arc::new(meta.clone())));
        }

        // 장비 아이템 등록
        for meta in &equipment_db.items {
            by_uuid.insert(meta.uuid, Item::Equipment(Arc::new(meta.clone())));
        }

        Self { by_uuid }
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Item> {
        self.by_uuid.get(uuid)
    }
}

impl GameDataBase {
    pub fn new(
        abnormality_data: Arc<AbnormalityDatabase>,
        artifact_data: Arc<ArtifactDatabase>,
        equipment_data: Arc<EquipmentDatabase>,
        shop_data: Arc<ShopDatabase>,
        bonus_data: Arc<BonusDatabase>,
        random_event_data: Arc<RandomEventDatabase>,
        pve_data: Arc<PveEncounterDatabase>,
        skill_data: Arc<SkillDatabase>,
        event_pools: EventPoolConfig,
    ) -> Self {
        let item_registry = ItemRegistry::new(&abnormality_data, &artifact_data, &equipment_data);

        Self {
            abnormality_data,
            artifact_data,
            equipment_data,
            shop_data,
            bonus_data,
            random_event_data,
            pve_data,
            skill_data,
            event_pools,
            item_registry,
        }
    }

    /// UUID로 아이템 메타데이터 조회
    pub fn item(&self, uuid: &Uuid) -> Option<&Item> {
        self.item_registry.get(uuid)
    }
}
