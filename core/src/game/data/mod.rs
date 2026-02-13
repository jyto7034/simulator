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
    shop_data::ShopDatabase,
    skill_data::SkillDatabase,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        artifact_data::ArtifactMetadata,
        equipment_data::{EquipmentMetadata, EquipmentType},
        event_pools::{EventPhasePool, EventPoolConfig},
    };
    use crate::game::enums::RiskLevel;
    use std::collections::HashMap;

    fn empty_event_pools() -> EventPoolConfig {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        }
    }

    #[test]
    fn item_registry_returns_items_by_uuid_across_categories() {
        let abno_uuid = Uuid::from_u128(1);
        let art_uuid = Uuid::from_u128(2);
        let equip_uuid = Uuid::from_u128(3);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 10,
            max_health: 10,
            attack: 1,
            defense: 1,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };

        let artifact = ArtifactMetadata {
            id: "art".to_string(),
            uuid: art_uuid,
            name: "Art".to_string(),
            description: "desc".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 20,
            triggered_effects: HashMap::new(),
        };

        let equipment = EquipmentMetadata {
            id: "equip".to_string(),
            uuid: equip_uuid,
            name: "Equip".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 30,
            allow_duplicate_equip: true,
            triggered_effects: HashMap::new(),
        };

        let game_data = GameDataBase::new(
            Arc::new(AbnormalityDatabase::new(vec![abno])),
            Arc::new(ArtifactDatabase::new(vec![artifact])),
            Arc::new(EquipmentDatabase::new(vec![equipment])),
            Arc::new(ShopDatabase::new(vec![])),
            Arc::new(BonusDatabase::new(vec![])),
            Arc::new(RandomEventDatabase::new(vec![])),
            Arc::new(PveEncounterDatabase::new(vec![])),
            Arc::new(SkillDatabase::new(vec![])),
            empty_event_pools(),
        );

        assert!(matches!(
            game_data.item(&abno_uuid),
            Some(Item::Abnormality(_))
        ));
        assert!(matches!(game_data.item(&art_uuid), Some(Item::Artifact(_))));
        assert!(matches!(
            game_data.item(&equip_uuid),
            Some(Item::Equipment(_))
        ));
        assert!(game_data.item(&Uuid::from_u128(999)).is_none());
    }

    #[test]
    fn item_accessors_reflect_metadata() {
        let uuid = Uuid::from_u128(1);
        let equipment = EquipmentMetadata {
            id: "equip".to_string(),
            uuid,
            name: "Equip".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 123,
            allow_duplicate_equip: true,
            triggered_effects: HashMap::new(),
        };
        let item = Item::Equipment(Arc::new(equipment));

        assert_eq!(item.uuid(), uuid);
        assert_eq!(item.id(), "equip");
        assert_eq!(item.name(), "Equip");
        assert_eq!(item.price(), 123);
    }
}
