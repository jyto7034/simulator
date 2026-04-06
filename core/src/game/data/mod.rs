use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    sync::{Arc, OnceLock},
};

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

pub(crate) fn once_lock_with<T>(value: T) -> OnceLock<T> {
    let once_lock = OnceLock::new();
    if once_lock.set(value).is_err() {
        unreachable!("OnceLock should be empty during initialization");
    }
    once_lock
}

fn build_unique_index<K, T>(
    items: &[T],
    index_name: &str,
    mut key_fn: impl FnMut(&T) -> K,
) -> HashMap<K, usize>
where
    K: Eq + Hash + Clone + Display,
{
    let mut index = HashMap::with_capacity(items.len());

    for (item_index, item) in items.iter().enumerate() {
        let key = key_fn(item);
        if let Some(previous_index) = index.insert(key.clone(), item_index) {
            panic!(
                "duplicate {index_name} '{key}' found at indices {previous_index} and {item_index}"
            );
        }
    }

    index
}

pub(crate) fn build_uuid_index<T>(
    items: &[T],
    index_name: &str,
    key_fn: impl FnMut(&T) -> Uuid,
) -> HashMap<Uuid, usize> {
    build_unique_index(items, index_name, key_fn)
}

pub(crate) fn build_string_index<T>(
    items: &[T],
    index_name: &str,
    mut key_fn: impl FnMut(&T) -> &str,
) -> HashMap<String, usize> {
    build_unique_index(items, index_name, |item| key_fn(item).to_string())
}

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

pub struct GameDataBaseParts {
    pub abnormality_data: Arc<AbnormalityDatabase>,
    pub artifact_data: Arc<ArtifactDatabase>,
    pub equipment_data: Arc<EquipmentDatabase>,
    pub shop_data: Arc<ShopDatabase>,
    pub bonus_data: Arc<BonusDatabase>,
    pub random_event_data: Arc<RandomEventDatabase>,
    pub pve_data: Arc<PveEncounterDatabase>,
    pub skill_data: Arc<SkillDatabase>,
    pub event_pools: EventPoolConfig,
}

#[derive(Debug, Clone)]
pub enum Item {
    Equipment(Arc<EquipmentMetadata>),
    Artifact(Arc<ArtifactMetadata>),
    Abnormality(Arc<AbnormalityMetadata>),
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a> {
    Equipment(&'a EquipmentMetadata),
    Artifact(&'a ArtifactMetadata),
    Abnormality(&'a AbnormalityMetadata),
}

impl<'a> ItemRef<'a> {
    pub fn price(&self) -> u32 {
        match self {
            ItemRef::Equipment(meta) => meta.price,
            ItemRef::Artifact(meta) => meta.price,
            ItemRef::Abnormality(meta) => meta.price,
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            ItemRef::Equipment(meta) => meta.uuid,
            ItemRef::Artifact(meta) => meta.uuid,
            ItemRef::Abnormality(meta) => meta.uuid,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            ItemRef::Equipment(meta) => &meta.id,
            ItemRef::Artifact(meta) => &meta.id,
            ItemRef::Abnormality(meta) => &meta.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ItemRef::Equipment(meta) => &meta.name,
            ItemRef::Artifact(meta) => &meta.name,
            ItemRef::Abnormality(meta) => &meta.name,
        }
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self, ItemRef::Equipment(_))
    }

    pub fn is_artifact(&self) -> bool {
        matches!(self, ItemRef::Artifact(_))
    }

    pub fn is_abnormality(&self) -> bool {
        matches!(self, ItemRef::Abnormality(_))
    }

    pub fn to_owned_item(self) -> Item {
        Item::from(self)
    }
}

impl<'a> From<ItemRef<'a>> for Item {
    fn from(value: ItemRef<'a>) -> Self {
        match value {
            ItemRef::Equipment(meta) => Item::Equipment(Arc::new(meta.clone())),
            ItemRef::Artifact(meta) => Item::Artifact(Arc::new(meta.clone())),
            ItemRef::Abnormality(meta) => Item::Abnormality(Arc::new(meta.clone())),
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemIndex {
    Abnormality(usize),
    Artifact(usize),
    Equipment(usize),
}

impl ItemIndex {
    fn kind(self) -> &'static str {
        match self {
            ItemIndex::Abnormality(_) => "abnormality",
            ItemIndex::Artifact(_) => "artifact",
            ItemIndex::Equipment(_) => "equipment",
        }
    }

    fn index(self) -> usize {
        match self {
            ItemIndex::Abnormality(index)
            | ItemIndex::Artifact(index)
            | ItemIndex::Equipment(index) => index,
        }
    }
}

#[derive(Debug, Default)]
pub struct ItemRegistry {
    by_uuid: HashMap<Uuid, ItemIndex>,
}

impl ItemRegistry {
    pub fn new(
        abnormality_db: &AbnormalityDatabase,
        artifact_db: &ArtifactDatabase,
        equipment_db: &EquipmentDatabase,
    ) -> Self {
        let mut by_uuid = HashMap::new();

        for (index, meta) in abnormality_db.items.iter().enumerate() {
            insert_item_index(&mut by_uuid, meta.uuid, ItemIndex::Abnormality(index));
        }

        for (index, meta) in artifact_db.items.iter().enumerate() {
            insert_item_index(&mut by_uuid, meta.uuid, ItemIndex::Artifact(index));
        }

        for (index, meta) in equipment_db.items.iter().enumerate() {
            insert_item_index(&mut by_uuid, meta.uuid, ItemIndex::Equipment(index));
        }

        Self { by_uuid }
    }

    fn get_index(&self, uuid: &Uuid) -> Option<ItemIndex> {
        self.by_uuid.get(uuid).copied()
    }
}

fn insert_item_index(by_uuid: &mut HashMap<Uuid, ItemIndex>, uuid: Uuid, new_index: ItemIndex) {
    if let Some(previous_index) = by_uuid.insert(uuid, new_index) {
        panic!(
            "duplicate item uuid '{uuid}' found across item registries: {}[{}] and {}[{}]",
            previous_index.kind(),
            previous_index.index(),
            new_index.kind(),
            new_index.index(),
        );
    }
}

impl GameDataBase {
    pub fn new(parts: GameDataBaseParts) -> Self {
        let GameDataBaseParts {
            abnormality_data,
            artifact_data,
            equipment_data,
            shop_data,
            bonus_data,
            random_event_data,
            pve_data,
            skill_data,
            event_pools,
        } = parts;

        abnormality_data.validate_indexes();
        artifact_data.validate_indexes();
        equipment_data.validate_indexes();
        shop_data.validate_indexes();
        bonus_data.validate_indexes();
        random_event_data.validate_indexes();
        pve_data.validate_indexes();
        skill_data.validate_indexes();

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
    pub fn item(&self, uuid: &Uuid) -> Option<ItemRef<'_>> {
        match self.item_registry.get_index(uuid)? {
            ItemIndex::Abnormality(index) => self
                .abnormality_data
                .items
                .get(index)
                .map(ItemRef::Abnormality),
            ItemIndex::Artifact(index) => {
                self.artifact_data.items.get(index).map(ItemRef::Artifact)
            }
            ItemIndex::Equipment(index) => {
                self.equipment_data.items.get(index).map(ItemRef::Equipment)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        artifact_data::ArtifactMetadata,
        bonus_data::BonusDatabase,
        equipment_data::{EquipmentMetadata, EquipmentType},
        event_pools::{EventPhasePool, EventPoolConfig},
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
    };
    use crate::game::enums::RiskLevel;
    use std::collections::HashMap;

    #[derive(Clone, Copy)]
    struct DummyKey {
        uuid: Uuid,
    }

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
    #[should_panic(expected = "duplicate dummy uuid")]
    fn build_uuid_index_panics_on_duplicate_keys() {
        let key = Uuid::from_u128(1);
        let items = [DummyKey { uuid: key }, DummyKey { uuid: key }];

        let _ = build_uuid_index(&items, "dummy uuid", |item| item.uuid);
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
            ability_activations: vec![],
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
            ability_activations: vec![],
        };

        let game_data = GameDataBase::new(GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![abno])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![artifact])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![equipment])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });

        assert!(matches!(
            game_data.item(&abno_uuid),
            Some(ItemRef::Abnormality(_))
        ));
        assert!(matches!(
            game_data.item(&art_uuid),
            Some(ItemRef::Artifact(_))
        ));
        assert!(matches!(
            game_data.item(&equip_uuid),
            Some(ItemRef::Equipment(_))
        ));
        assert!(game_data.item(&Uuid::from_u128(999)).is_none());
    }

    #[test]
    #[should_panic(expected = "duplicate item uuid")]
    fn game_data_base_panics_on_duplicate_item_uuid_across_categories() {
        let shared_uuid = Uuid::from_u128(1);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: shared_uuid,
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
            uuid: shared_uuid,
            name: "Art".to_string(),
            description: "desc".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 20,
            triggered_effects: Default::default(),
            ability_activations: vec![],
        };

        let _ = GameDataBase::new(GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![abno])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![artifact])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });
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
            ability_activations: vec![],
        };
        let item = Item::Equipment(Arc::new(equipment));

        assert_eq!(item.uuid(), uuid);
        assert_eq!(item.id(), "equip");
        assert_eq!(item.name(), "Equip");
        assert_eq!(item.price(), 123);
    }
}
