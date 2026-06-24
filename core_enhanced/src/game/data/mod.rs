use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    sync::{Arc, OnceLock},
};

use uuid::Uuid;

use crate::game::{
    battle::buffs::BuffDatabase,
    data::{
        abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
        artifact_data::{ArtifactDatabase, ArtifactMetadata},
        consumable_data::{ConsumableDatabase, ConsumableMetadata},
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::{RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase},
        equipment_data::{EquipmentDatabase, EquipmentMetadata},
        pve_data::PveEncounterDatabase,
        reward_data::RewardDatabase,
        run_policy_data::RunPolicyData,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::SkillFragmentDatabase,
    },
};

// 환상체 (기물) 정보
pub mod abnormality_data;

// 아티팩트 정보
pub mod artifact_data;

// 섭취 아이템 정보
pub mod consumable_data;

// 아이템 ( 장착 장비 ) 정보
pub mod equipment_data;

// 침식된 전 탐사 직원 적 프로필
pub mod corroded_employee_data;

// 침식 직원 웨이브 생성 preset
pub mod corroded_wave_data;

// 시작 직원 후보 데이터
pub mod employee_data;

// PvE 전투 데이터
pub mod pve_data;

// 보상 정보
pub mod reward_data;

// 런/월드 진행 정책
pub mod run_policy_data;

// 스킬 정보
pub mod skill_data;

// 직원 스킬 파편 정보
pub mod skill_fragment_data;

// 상점 정보
pub mod shop_data;

mod validation;

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
    pub corroded_employee_data: Arc<CorrodedEmployeeProfileDatabase>,
    pub corroded_wave_data: Arc<CorrodedWavePresetDatabase>,
    pub starter_employee_data: Arc<StarterEmployeeCandidateDatabase>,
    pub recruitment_employee_data: Arc<RecruitmentEmployeeCandidateDatabase>,
    pub artifact_data: Arc<ArtifactDatabase>,
    pub consumable_data: Arc<ConsumableDatabase>,
    pub equipment_data: Arc<EquipmentDatabase>,

    /// 마스터 테이블을 참고하여 상인, 보상 등을 구성하여 저장하는 게임 데이터베이스
    pub shop_data: Arc<ShopDatabase>,
    pub reward_data: Arc<RewardDatabase>,

    /// PvE 전투(Suppress) 데이터
    pub pve_data: Arc<PveEncounterDatabase>,

    /// 런/월드 진행 정책 데이터
    pub run_policy: Arc<RunPolicyData>,

    /// 스킬 메타데이터 DB
    pub skill_data: Arc<SkillDatabase>,

    /// 전투 중 상태이상 buff 메타데이터 DB
    pub buff_data: Arc<BuffDatabase>,

    /// 직원 스킬 파편 메타데이터 DB
    pub skill_fragment_data: Arc<SkillFragmentDatabase>,

    /// UUID 기반 아이템 조회 레지스트리
    pub item_registry: ItemRegistry,
}

pub struct GameDataBaseParts {
    pub abnormality_data: Arc<AbnormalityDatabase>,
    pub corroded_employee_data: Arc<CorrodedEmployeeProfileDatabase>,
    pub corroded_wave_data: Arc<CorrodedWavePresetDatabase>,
    pub starter_employee_data: Arc<StarterEmployeeCandidateDatabase>,
    pub recruitment_employee_data: Arc<RecruitmentEmployeeCandidateDatabase>,
    pub artifact_data: Arc<ArtifactDatabase>,
    pub consumable_data: Arc<ConsumableDatabase>,
    pub equipment_data: Arc<EquipmentDatabase>,
    pub shop_data: Arc<ShopDatabase>,
    pub reward_data: Arc<RewardDatabase>,
    pub pve_data: Arc<PveEncounterDatabase>,
    pub run_policy: Arc<RunPolicyData>,
    pub skill_data: Arc<SkillDatabase>,
    pub buff_data: Arc<BuffDatabase>,
    pub skill_fragment_data: Arc<SkillFragmentDatabase>,
}

pub struct GameDataBuilder {
    abnormality_data: Arc<AbnormalityDatabase>,
    corroded_employee_data: Arc<CorrodedEmployeeProfileDatabase>,
    corroded_wave_data: Arc<CorrodedWavePresetDatabase>,
    starter_employee_data: Arc<StarterEmployeeCandidateDatabase>,
    recruitment_employee_data: Arc<RecruitmentEmployeeCandidateDatabase>,
    artifact_data: Arc<ArtifactDatabase>,
    consumable_data: Arc<ConsumableDatabase>,
    equipment_data: Arc<EquipmentDatabase>,
    shop_data: Arc<ShopDatabase>,
    reward_data: Arc<RewardDatabase>,
    pve_data: Arc<PveEncounterDatabase>,
    run_policy: Arc<RunPolicyData>,
    skill_data: Arc<SkillDatabase>,
    buff_data: Arc<BuffDatabase>,
    skill_fragment_data: Arc<SkillFragmentDatabase>,
}

impl GameDataBuilder {
    pub fn empty() -> Self {
        Self {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            corroded_employee_data: Arc::new(CorrodedEmployeeProfileDatabase::new(vec![])),
            corroded_wave_data: Arc::new(CorrodedWavePresetDatabase::new(vec![])),
            starter_employee_data: Arc::new(StarterEmployeeCandidateDatabase::new(vec![])),
            recruitment_employee_data: Arc::new(RecruitmentEmployeeCandidateDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            consumable_data: Arc::new(ConsumableDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            reward_data: Arc::new(RewardDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            run_policy: Arc::new(RunPolicyData::builtin()),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            buff_data: Arc::new(BuffDatabase::new(vec![])),
            skill_fragment_data: Arc::new(SkillFragmentDatabase::new(vec![])),
        }
    }

    pub fn live_defaults() -> Self {
        Self::empty().with_buffs(BuffDatabase::live_default())
    }

    pub fn with_abnormalities(mut self, items: Vec<AbnormalityMetadata>) -> Self {
        self.abnormality_data = Arc::new(AbnormalityDatabase::new(items));
        self
    }

    pub fn with_abnormality_data(mut self, data: Arc<AbnormalityDatabase>) -> Self {
        self.abnormality_data = data;
        self
    }

    pub fn with_corroded_employee_profiles(
        mut self,
        data: CorrodedEmployeeProfileDatabase,
    ) -> Self {
        self.corroded_employee_data = Arc::new(data);
        self
    }

    pub fn with_corroded_employee_data(
        mut self,
        data: Arc<CorrodedEmployeeProfileDatabase>,
    ) -> Self {
        self.corroded_employee_data = data;
        self
    }

    pub fn with_corroded_wave_presets(mut self, data: CorrodedWavePresetDatabase) -> Self {
        self.corroded_wave_data = Arc::new(data);
        self
    }

    pub fn with_corroded_wave_data(mut self, data: Arc<CorrodedWavePresetDatabase>) -> Self {
        self.corroded_wave_data = data;
        self
    }

    pub fn with_starter_employee_candidates(
        mut self,
        data: StarterEmployeeCandidateDatabase,
    ) -> Self {
        self.starter_employee_data = Arc::new(data);
        self
    }

    pub fn with_starter_employee_data(
        mut self,
        data: Arc<StarterEmployeeCandidateDatabase>,
    ) -> Self {
        self.starter_employee_data = data;
        self
    }

    pub fn with_recruitment_employee_candidates(
        mut self,
        data: RecruitmentEmployeeCandidateDatabase,
    ) -> Self {
        self.recruitment_employee_data = Arc::new(data);
        self
    }

    pub fn with_recruitment_employee_data(
        mut self,
        data: Arc<RecruitmentEmployeeCandidateDatabase>,
    ) -> Self {
        self.recruitment_employee_data = data;
        self
    }

    pub fn with_artifacts(mut self, items: Vec<ArtifactMetadata>) -> Self {
        self.artifact_data = Arc::new(ArtifactDatabase::new(items));
        self
    }

    pub fn with_artifact_data(mut self, data: Arc<ArtifactDatabase>) -> Self {
        self.artifact_data = data;
        self
    }

    pub fn with_consumables(mut self, items: Vec<ConsumableMetadata>) -> Self {
        self.consumable_data = Arc::new(ConsumableDatabase::new(items));
        self
    }

    pub fn with_consumable_data(mut self, data: Arc<ConsumableDatabase>) -> Self {
        self.consumable_data = data;
        self
    }

    pub fn with_equipment(mut self, items: Vec<EquipmentMetadata>) -> Self {
        self.equipment_data = Arc::new(EquipmentDatabase::new(items));
        self
    }

    pub fn with_equipment_data(mut self, data: Arc<EquipmentDatabase>) -> Self {
        self.equipment_data = data;
        self
    }

    pub fn with_shops(mut self, data: ShopDatabase) -> Self {
        self.shop_data = Arc::new(data);
        self
    }

    pub fn with_shop_data(mut self, data: Arc<ShopDatabase>) -> Self {
        self.shop_data = data;
        self
    }

    pub fn with_rewards(mut self, data: RewardDatabase) -> Self {
        self.reward_data = Arc::new(data);
        self
    }

    pub fn with_reward_data(mut self, data: Arc<RewardDatabase>) -> Self {
        self.reward_data = data;
        self
    }

    pub fn with_pve(mut self, data: PveEncounterDatabase) -> Self {
        self.pve_data = Arc::new(data);
        self
    }

    pub fn with_pve_data(mut self, data: Arc<PveEncounterDatabase>) -> Self {
        self.pve_data = data;
        self
    }

    pub fn with_run_policy(mut self, policy: RunPolicyData) -> Self {
        self.run_policy = Arc::new(policy);
        self
    }

    pub fn with_run_policy_data(mut self, data: Arc<RunPolicyData>) -> Self {
        self.run_policy = data;
        self
    }

    pub fn with_skills(mut self, data: SkillDatabase) -> Self {
        self.skill_data = Arc::new(data);
        self
    }

    pub fn with_skill_data(mut self, data: Arc<SkillDatabase>) -> Self {
        self.skill_data = data;
        self
    }

    pub fn with_buffs(mut self, data: BuffDatabase) -> Self {
        self.buff_data = Arc::new(data);
        self
    }

    pub fn with_buff_data(mut self, data: Arc<BuffDatabase>) -> Self {
        self.buff_data = data;
        self
    }

    pub fn with_skill_fragments(mut self, data: SkillFragmentDatabase) -> Self {
        self.skill_fragment_data = Arc::new(data);
        self
    }

    pub fn with_skill_fragment_data(mut self, data: Arc<SkillFragmentDatabase>) -> Self {
        self.skill_fragment_data = data;
        self
    }

    pub fn build(self) -> GameDataBase {
        GameDataBase::new(GameDataBaseParts {
            abnormality_data: self.abnormality_data,
            corroded_employee_data: self.corroded_employee_data,
            corroded_wave_data: self.corroded_wave_data,
            starter_employee_data: self.starter_employee_data,
            recruitment_employee_data: self.recruitment_employee_data,
            artifact_data: self.artifact_data,
            consumable_data: self.consumable_data,
            equipment_data: self.equipment_data,
            shop_data: self.shop_data,
            reward_data: self.reward_data,
            pve_data: self.pve_data,
            run_policy: self.run_policy,
            skill_data: self.skill_data,
            buff_data: self.buff_data,
            skill_fragment_data: self.skill_fragment_data,
        })
    }

    pub fn build_arc(self) -> Arc<GameDataBase> {
        Arc::new(self.build())
    }
}

impl Default for GameDataBuilder {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Equipment(Arc<EquipmentMetadata>),
    Artifact(Arc<ArtifactMetadata>),
    Consumable(Arc<ConsumableMetadata>),
}

#[derive(Debug, Clone, Copy)]
pub enum ItemRef<'a> {
    Equipment(&'a EquipmentMetadata),
    Artifact(&'a ArtifactMetadata),
    Consumable(&'a ConsumableMetadata),
}

impl<'a> ItemRef<'a> {
    pub fn price(&self) -> u32 {
        match self {
            ItemRef::Equipment(meta) => meta.price,
            ItemRef::Artifact(meta) => meta.price,
            ItemRef::Consumable(meta) => meta.price,
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            ItemRef::Equipment(meta) => meta.uuid,
            ItemRef::Artifact(meta) => meta.uuid,
            ItemRef::Consumable(meta) => meta.uuid,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            ItemRef::Equipment(meta) => &meta.id,
            ItemRef::Artifact(meta) => &meta.id,
            ItemRef::Consumable(meta) => &meta.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ItemRef::Equipment(meta) => &meta.name,
            ItemRef::Artifact(meta) => &meta.name,
            ItemRef::Consumable(meta) => &meta.name,
        }
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self, ItemRef::Equipment(_))
    }

    pub fn is_artifact(&self) -> bool {
        matches!(self, ItemRef::Artifact(_))
    }

    pub fn is_consumable(&self) -> bool {
        matches!(self, ItemRef::Consumable(_))
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
            ItemRef::Consumable(meta) => Item::Consumable(Arc::new(meta.clone())),
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
            Item::Consumable(meta) => meta.price,
        }
    }

    /// 아이템 UUID 반환
    pub fn uuid(&self) -> Uuid {
        match self {
            Item::Equipment(meta) => meta.uuid,
            Item::Artifact(meta) => meta.uuid,
            Item::Consumable(meta) => meta.uuid,
        }
    }

    /// 아이템 ID 반환
    pub fn id(&self) -> &str {
        match self {
            Item::Equipment(meta) => &meta.id,
            Item::Artifact(meta) => &meta.id,
            Item::Consumable(meta) => &meta.id,
        }
    }

    /// 아이템 이름 반환
    pub fn name(&self) -> &str {
        match self {
            Item::Equipment(meta) => &meta.name,
            Item::Artifact(meta) => &meta.name,
            Item::Consumable(meta) => &meta.name,
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

    pub fn is_consumable(&self) -> bool {
        matches!(self, Item::Consumable(_))
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

    pub fn as_consumable(&self) -> Option<Arc<ConsumableMetadata>> {
        match self {
            Item::Consumable(meta) => Some(meta.clone()),
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
            Item::Consumable(meta) => Item::Consumable(Arc::clone(meta)),
        }
    }
}

/// Uuid -> Item 매핑을 제공하는 전역 레지스트리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemIndex {
    Artifact(usize),
    Consumable(usize),
    Equipment(usize),
}

impl ItemIndex {
    fn kind(self) -> &'static str {
        match self {
            ItemIndex::Artifact(_) => "artifact",
            ItemIndex::Consumable(_) => "consumable",
            ItemIndex::Equipment(_) => "equipment",
        }
    }

    fn index(self) -> usize {
        match self {
            ItemIndex::Artifact(index)
            | ItemIndex::Consumable(index)
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
        artifact_db: &ArtifactDatabase,
        consumable_db: &ConsumableDatabase,
        equipment_db: &EquipmentDatabase,
    ) -> Self {
        let mut by_uuid = HashMap::new();

        for (index, meta) in artifact_db.items.iter().enumerate() {
            insert_item_index(&mut by_uuid, meta.uuid, ItemIndex::Artifact(index));
        }

        for (index, meta) in consumable_db.items.iter().enumerate() {
            insert_item_index(&mut by_uuid, meta.uuid, ItemIndex::Consumable(index));
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
            corroded_employee_data,
            corroded_wave_data,
            starter_employee_data,
            recruitment_employee_data,
            artifact_data,
            consumable_data,
            equipment_data,
            shop_data,
            reward_data,
            pve_data,
            run_policy,
            skill_data,
            buff_data,
            skill_fragment_data,
        } = parts;

        abnormality_data.validate_indexes();
        corroded_employee_data.validate_indexes();
        corroded_wave_data.validate_indexes();
        starter_employee_data.validate_indexes();
        recruitment_employee_data.validate_indexes();
        artifact_data.validate_indexes();
        consumable_data.validate_indexes();
        equipment_data.validate_indexes();
        shop_data.validate_indexes();
        reward_data.validate_indexes();
        pve_data.validate_indexes();
        run_policy
            .validate_contract()
            .expect("run policy data must satisfy contract");
        skill_data.validate_indexes();
        buff_data.validate_indexes();
        skill_fragment_data.validate_indexes();
        validation::validate_data_references(
            &abnormality_data,
            &corroded_employee_data,
            &corroded_wave_data,
            &artifact_data,
            &consumable_data,
            &equipment_data,
            &reward_data,
            &pve_data,
            &skill_data,
            &buff_data,
            &skill_fragment_data,
        );

        let item_registry = ItemRegistry::new(&artifact_data, &consumable_data, &equipment_data);
        validation::validate_shop_item_references(&shop_data, &item_registry);
        validation::validate_starter_employee_loadout_references(
            &starter_employee_data,
            &equipment_data,
            &skill_fragment_data,
        );

        Self {
            abnormality_data,
            corroded_employee_data,
            corroded_wave_data,
            starter_employee_data,
            recruitment_employee_data,
            artifact_data,
            consumable_data,
            equipment_data,
            shop_data,
            reward_data,
            pve_data,
            run_policy,
            skill_data,
            buff_data,
            skill_fragment_data,
            item_registry,
        }
    }

    /// UUID로 아이템 메타데이터 조회
    pub fn item(&self, uuid: &Uuid) -> Option<ItemRef<'_>> {
        match self.item_registry.get_index(uuid)? {
            ItemIndex::Artifact(index) => {
                self.artifact_data.items.get(index).map(ItemRef::Artifact)
            }
            ItemIndex::Consumable(index) => self
                .consumable_data
                .items
                .get(index)
                .map(ItemRef::Consumable),
            ItemIndex::Equipment(index) => {
                self.equipment_data.items.get(index).map(ItemRef::Equipment)
            }
        }
    }

    pub fn validate_generated_combat_preview_contracts(&self) {
        validation::validate_generated_combat_preview_contracts(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId, SkillKind,
        SkillStepDef, SkillTarget,
    };
    use crate::game::battle::buffs::{BuffDatabase, BuffId};
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        artifact_data::ArtifactMetadata,
        equipment_data::{EquipmentMetadata, EquipmentType},
        shop_data::{ShopDatabase, ShopMetadata},
        skill_data::SkillDatabase,
        skill_fragment_data::{
            SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
            SkillFragmentId, SkillFragmentMetadata, SkillFragmentOrigin, SkillFragmentRarity,
        },
        GameDataBuilder,
    };
    use crate::game::enums::RiskLevel;
    use std::{collections::HashMap, sync::Arc};

    #[derive(Clone, Copy)]
    struct DummyKey {
        uuid: Uuid,
    }

    #[test]
    fn game_data_builder_empty_does_not_load_live_buffs() {
        let game_data = GameDataBuilder::empty().build();

        assert!(game_data
            .buff_data
            .get(BuffId::from_name("poison"))
            .is_none());
    }

    #[test]
    fn game_data_builder_live_defaults_loads_live_buffs_explicitly() {
        let game_data = GameDataBuilder::live_defaults().build();

        assert!(game_data
            .buff_data
            .get(BuffId::from_name("poison"))
            .is_some());
    }

    fn minimal_skill(id: &str) -> SkillDef {
        SkillDef {
            id: SkillId::from(id),
            name: id.to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::SelfUnit,
                Default::default(),
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::SelfUnit,
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![],
                presentation: Default::default(),
            }],
        }
    }

    fn minimal_abnormality(id: &str, uuid: u128, skill_id: Option<&str>) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 10,
            max_health: 10,
            attack: 1,
            defense: 1,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            mobility_kind: Default::default(),
            skill_id: skill_id.map(SkillId::from),
            target_traits: Vec::new(),
        }
    }

    fn active_fragment(
        id: &str,
        origin_abnormality_id: &str,
        imitation_skill_id: &str,
    ) -> SkillFragmentMetadata {
        SkillFragmentMetadata {
            id: SkillFragmentId::from(id),
            uuid: Uuid::from_u128(0xF000),
            name: id.to_string(),
            description: "fragment".to_string(),
            rarity: SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: Some(SkillFragmentOrigin::Abnormality {
                abnormality_id: origin_abnormality_id.to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::AbnormalityContainment {
                abnormality_id: origin_abnormality_id.to_string(),
            }],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: SkillId::from(imitation_skill_id),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
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
    fn item_registry_returns_inventory_items_by_uuid_and_excludes_abnormalities() {
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
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
        };

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_artifacts(vec![artifact])
            .with_equipment(vec![equipment])
            .build();

        assert!(game_data.item(&abno_uuid).is_none());
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
    #[should_panic(expected = "must use Elite or Boss threat_class")]
    fn game_data_base_rejects_normal_abnormality_threat_class() {
        let mut abno = minimal_abnormality("normal_abnormality", 0xA991, None);
        abno.threat_class = crate::game::battle::types::BattleUnitThreatClass::Normal;

        let _ = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .build();
    }

    #[test]
    fn game_data_base_accepts_fragment_origin_and_independent_imitation_skill() {
        let abno = minimal_abnormality("t-02-43_freischutz", 1, Some("freischutz_magic_bullet"));
        let original_skill = minimal_skill("freischutz_magic_bullet");
        let imitation_skill = minimal_skill("fragment_freischutz_black_round");
        let fragment = active_fragment(
            "fragment_freischutz_black_round",
            "t-02-43_freischutz",
            "fragment_freischutz_black_round",
        );

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_skills(SkillDatabase::new(vec![original_skill, imitation_skill]))
            .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
            .build();

        assert!(game_data
            .skill_fragment_data
            .get_by_id_str("fragment_freischutz_black_round")
            .is_some());
    }

    #[test]
    #[should_panic(expected = "references unknown origin abnormality")]
    fn game_data_base_rejects_fragment_unknown_origin_abnormality() {
        let fragment = active_fragment(
            "fragment_freischutz_black_round",
            "missing_freischutz",
            "fragment_freischutz_black_round",
        );

        let _ = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![minimal_skill(
                "fragment_freischutz_black_round",
            )]))
            .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
            .build();
    }

    #[test]
    #[should_panic(expected = "references unknown imitation skill")]
    fn game_data_base_rejects_fragment_unknown_imitation_skill() {
        let abno = minimal_abnormality("t-02-43_freischutz", 1, Some("freischutz_magic_bullet"));
        let fragment = active_fragment(
            "fragment_freischutz_black_round",
            "t-02-43_freischutz",
            "fragment_freischutz_black_round",
        );

        let _ = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_skills(SkillDatabase::new(vec![minimal_skill(
                "freischutz_magic_bullet",
            )]))
            .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
            .build();
    }

    #[test]
    #[should_panic(expected = "references unknown buff")]
    fn game_data_base_rejects_skill_references_missing_buff_data() {
        let mut skill = minimal_skill("skill_with_missing_buff");
        skill.steps[0].effects.push(SkillEffectDef::ApplyBuff {
            buff_id: "missing_buff".to_string(),
            duration_ms: 100,
        });

        let _ = GameDataBuilder::empty()
            .with_buff_data(Arc::new(BuffDatabase::new(vec![])))
            .with_skills(SkillDatabase::new(vec![skill]))
            .build();
    }

    #[test]
    #[should_panic(expected = "duplicate item uuid")]
    fn game_data_base_panics_on_duplicate_item_uuid_across_inventory_item_categories() {
        let shared_uuid = Uuid::from_u128(1);

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

        let equipment = EquipmentMetadata {
            id: "equip".to_string(),
            uuid: shared_uuid,
            name: "Equip".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 30,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
        };

        let _ = GameDataBuilder::empty()
            .with_artifacts(vec![artifact])
            .with_equipment(vec![equipment])
            .build();
    }

    #[test]
    #[should_panic(expected = "references missing item uuid")]
    fn game_data_base_panics_on_shop_referencing_missing_item_uuid() {
        let shop = ShopMetadata {
            id: "shop".to_string(),
            name: "Shop".to_string(),
            uuid: Uuid::from_u128(100),
            shop_type: crate::game::data::shop_data::ShopType::Shop,
            can_reroll: false,
            visible_items: vec![Uuid::from_u128(999)],
            hidden_items: vec![],
        };

        let _ = GameDataBuilder::empty()
            .with_shops(ShopDatabase::new(vec![shop]))
            .build();
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
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
        };
        let item = Item::Equipment(Arc::new(equipment));

        assert_eq!(item.uuid(), uuid);
        assert_eq!(item.id(), "equip");
        assert_eq!(item.name(), "Equip");
        assert_eq!(item.price(), 123);
    }
}
