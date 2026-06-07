#![allow(dead_code)]

pub const BOARD_WIDTH: u8 = 7;
pub const BOARD_HEIGHT: u8 = 8;
pub const BOARD_SIZE: (u8, u8) = (BOARD_WIDTH, BOARD_HEIGHT);

use std::path::PathBuf;
use std::sync::Arc;

use game_core::game::ability::{
    DeliveryDef, SkillCastTargetingDef, SkillDef, SkillId, SkillKind, SkillPresentationDef,
    SkillStepDef, SkillTarget, StepTargetingMode,
};
use game_core::game::battle::{buffs::BuffDatabase, timeline::Timeline};
use game_core::game::combat_preview::EnemyKind;
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::{ArtifactDatabase, ArtifactMetadata};
use game_core::game::data::consumable_data::ConsumableDatabase;
use game_core::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase;
use game_core::game::data::employee_data::{
    RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase,
};
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::pve_data::{
    PveEncounter, PveEncounterDatabase, PveWaveData, PveWaveEnemyData,
};
use game_core::game::data::reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata};
use game_core::game::data::shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType};
use game_core::game::data::skill_data::SkillDatabase;
use game_core::game::data::skill_fragment_data::SkillFragmentDatabase;
use game_core::game::data::{GameDataBase, GameDataBuilder};
use game_core::game::enums::{RewardMode, RiskLevel};
use game_core::game::reward::RewardEffect;
use uuid::Uuid;

pub fn debug_event_log_exports_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("debug_event_log_exports")
}

pub fn write_debug_event_log_export(name: &str, timeline: &Timeline) -> PathBuf {
    let out_dir = debug_event_log_exports_dir();
    let out_path = out_dir.join(format!("{name}.json"));
    let parent_dir = out_path.parent().unwrap_or_else(|| {
        panic!(
            "debug event log export path must have parent: {}",
            out_path.display()
        )
    });
    std::fs::create_dir_all(parent_dir).expect("create debug_event_log_exports directory");
    timeline
        .write_pretty_json(&out_path)
        .expect("write debug event log json");
    out_path
}

pub fn empty_game_data() -> Arc<GameDataBase> {
    GameDataBuilder::empty().build_arc()
}

/// 테스트용 `GameDataBase` 생성 (작고 결정적인 데이터).
///
/// - 리롤 가능한 상점 1개 (visible/hidden 구성)
/// - Equipment/Artifact/Abnormality 최소 1개
/// - Reward/PvE encounter 최소 1개
pub fn create_test_game_data() -> Arc<GameDataBase> {
    let shop_uuid = Uuid::from_u128(1);
    let reward_uuid = Uuid::from_u128(2);

    let artifact1 = ArtifactMetadata {
        id: "test_artifact_1".to_string(),
        uuid: Uuid::from_u128(0xA000_0001),
        name: "Test Artifact 1".to_string(),
        description: "Test artifact for testing".to_string(),
        rarity: RiskLevel::HE,
        price: 100,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };
    let artifact2 = ArtifactMetadata {
        id: "test_artifact_2".to_string(),
        uuid: Uuid::from_u128(0xA000_0002),
        name: "Test Artifact 2".to_string(),
        description: "Another test artifact".to_string(),
        rarity: RiskLevel::WAW,
        price: 200,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    };

    let equipment1 = EquipmentMetadata {
        id: "test_weapon_1".to_string(),
        uuid: Uuid::from_u128(0xE000_0001),
        name: "Test Weapon".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::HE,
        price: 150,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: Default::default(),
        ability_activations: vec![],
        weapon_profile: Some(Default::default()),
    };
    let equipment2 = EquipmentMetadata {
        id: "test_armor_1".to_string(),
        uuid: Uuid::from_u128(0xE000_0002),
        name: "Test Armor".to_string(),
        equipment_type: EquipmentType::Armor,
        rarity: RiskLevel::TETH,
        price: 80,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: Default::default(),
        ability_activations: vec![],
        weapon_profile: None,
    };

    let skill_id = SkillId::from("test_skill");
    let skills_db = SkillDatabase::new(vec![SkillDef {
        id: skill_id.clone(),
        name: "test_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
            range_units: 1.0,
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::SelfUnit,
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![],
            presentation: SkillPresentationDef::default(),
        }],
    }]);

    let abnormality1 = AbnormalityMetadata {
        id: "test_abnorm_1".to_string(),
        uuid: Uuid::from_u128(0xB000_0001),
        name: "Test Abnormality".to_string(),
        risk_level: RiskLevel::HE,
        price: 120,
        max_health: 100,
        attack: 30,
        defense: 5,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: Some(skill_id),
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    };
    let abnormality2 = AbnormalityMetadata {
        id: "test_abnorm_2".to_string(),
        uuid: Uuid::from_u128(0xB000_0002),
        name: "Test Abnormality 2".to_string(),
        risk_level: RiskLevel::TETH,
        price: 90,
        max_health: 80,
        attack: 20,
        defense: 4,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    };
    let abnormality3 = AbnormalityMetadata {
        id: "test_abnorm_3".to_string(),
        uuid: Uuid::from_u128(0xB000_0003),
        name: "Test Abnormality 3".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 70,
        max_health: 70,
        attack: 18,
        defense: 3,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    };

    let shop = ShopMetadata {
        id: "test_shop".to_string(),
        name: "Test Rerollable Shop".to_string(),
        uuid: shop_uuid,
        shop_type: ShopType::Shop,
        can_reroll: true,
        visible_items: vec![artifact1.uuid, equipment1.uuid],
        hidden_items: vec![artifact2.uuid, equipment2.uuid, abnormality1.uuid],
    };

    let reward = RewardMetadata {
        id: "test_reward_enkephalin".to_string(),
        uuid: reward_uuid,
        name: "Test Enkephalin Reward".to_string(),
        description: "Grants fixed Enkephalin".to_string(),
        icon: "enkephalin_icon.png".to_string(),
        tags: Vec::new(),
        effects: vec![RewardEffect::GrantEnkephalin { amount: 100 }],
    };

    let artifacts_db = ArtifactDatabase::new(vec![artifact1, artifact2]);
    let equipments_db = EquipmentDatabase::new(vec![equipment1, equipment2]);
    let abnormalities_db = AbnormalityDatabase::new(vec![
        abnormality1.clone(),
        abnormality2.clone(),
        abnormality3.clone(),
    ]);
    let shops_db = ShopDatabase::new_with_pools(
        vec![shop],
        vec![ShopPoolMetadata {
            id: "default_shops".to_string(),
            shop_ids: vec!["test_shop".to_string()],
        }],
    );
    let rewards_db = RewardDatabase::new_with_pools(
        vec![reward],
        vec![RewardPoolMetadata {
            id: "default_treasures".to_string(),
            reward_ids: vec!["test_reward_enkephalin".to_string()],
        }],
    );

    let pve_db = PveEncounterDatabase::new(vec![
        PveEncounter {
            id: "pve_test_1".to_string(),
            abnormality_id: abnormality1.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::ZAYIN,
            reward_mode: RewardMode::ChooseOne,
            reward_uuids: vec![reward_uuid],
            node_type: None,
            mission_variant: None,
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: None,
                enemies: vec![PveWaveEnemyData {
                    kind: EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: abnormality1.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }],
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            id: "pve_test_2".to_string(),
            abnormality_id: abnormality2.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::TETH,
            reward_mode: RewardMode::ChooseOne,
            reward_uuids: vec![reward_uuid],
            node_type: None,
            mission_variant: None,
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: None,
                enemies: vec![PveWaveEnemyData {
                    kind: EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: abnormality2.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }],
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            id: "pve_test_3".to_string(),
            abnormality_id: abnormality3.id.clone(),
            difficulty: 1,
            risk_level: RiskLevel::ZAYIN,
            reward_mode: RewardMode::ChooseOne,
            reward_uuids: vec![reward_uuid],
            node_type: None,
            mission_variant: None,
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: None,
                enemies: vec![PveWaveEnemyData {
                    kind: EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: abnormality3.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }],
            }],
            static_obstacles: vec![],
        },
    ]);

    GameDataBuilder::empty()
        .with_abnormality_data(Arc::new(abnormalities_db))
        .with_corroded_employee_data(Arc::new(CorrodedEmployeeProfileDatabase::new(vec![])))
        .with_artifact_data(Arc::new(artifacts_db))
        .with_equipment_data(Arc::new(equipments_db))
        .with_shop_data(Arc::new(shops_db))
        .with_reward_data(Arc::new(rewards_db))
        .with_pve_data(Arc::new(pve_db))
        .with_skill_data(Arc::new(skills_db))
        .build_arc()
}

/// 실제 RON 파일에서 GameDataBase 로드
///
/// 통합 테스트나 실제 서버에서 사용합니다.
/// - 상점 데이터 (shops.ron)
/// - 보상 데이터 (rewards.ron)
/// - 환상체 데이터 (abnormalities.ron)
/// - 장비 데이터 (equipments.ron)
/// - 아티팩트 데이터 (artifacts.ron)
/// - 섭취 아이템 데이터 (consumables/base.ron)
/// - 시작 직원 후보 데이터 (employees/starter_candidates.ron)
/// - 런 중 채용 후보 데이터 (employees/recruitment_candidates.ron)
/// - 스킬 파편 데이터 (skill_fragments/base.ron)
#[allow(dead_code)]
pub fn load_game_data_from_ron() -> Arc<GameDataBase> {
    // Given: RON 파일 include_str! 로 포함 (컴파일 타임)
    let shops_ron = include_str!("../../../game_resources/data/events/shops/base.ron");
    let rewards_ron = include_str!("../../../game_resources/data/events/rewards/base.ron");
    let abnormalities_ron = include_str!("../../../game_resources/data/abnormalities/base.ron");
    let corroded_employees_ron =
        include_str!("../../../game_resources/data/enemies/corroded_employees.ron");
    let corroded_wave_presets_ron =
        include_str!("../../../game_resources/data/enemies/corroded_wave_presets.ron");
    let starter_candidates_ron =
        include_str!("../../../game_resources/data/employees/starter_candidates.ron");
    let recruitment_candidates_ron =
        include_str!("../../../game_resources/data/employees/recruitment_candidates.ron");
    let equipments_ron = include_str!("../../../game_resources/data/equipments/base.ron");
    let artifacts_ron = include_str!("../../../game_resources/data/artifacts/base.ron");
    let consumables_ron = include_str!("../../../game_resources/data/consumables/base.ron");
    let buffs_ron = include_str!("../../../game_resources/data/buffs/base.ron");
    let skills_ron = include_str!("../../../game_resources/data/skills/base.ron");
    let skill_fragments_ron = include_str!("../../../game_resources/data/skill_fragments/base.ron");
    let pve_ron = include_str!("../../../game_resources/data/pve/encounters.ron");

    // When: RON 역직렬화
    let shops_db: ShopDatabase =
        ron::de::from_str(shops_ron).expect("Failed to deserialize shops.ron");

    let rewards_db: RewardDatabase =
        ron::de::from_str(rewards_ron).expect("Failed to deserialize rewards.ron");

    let abnormalities_db: AbnormalityDatabase =
        ron::de::from_str(abnormalities_ron).expect("Failed to deserialize abnormalities.ron");
    let corroded_employee_db: CorrodedEmployeeProfileDatabase =
        ron::de::from_str(corroded_employees_ron)
            .expect("Failed to deserialize corroded_employees.ron");
    let corroded_wave_db: game_core::game::data::corroded_wave_data::CorrodedWavePresetDatabase =
        ron::de::from_str(corroded_wave_presets_ron)
            .expect("Failed to deserialize corroded_wave_presets.ron");
    let starter_employee_db: StarterEmployeeCandidateDatabase =
        ron::de::from_str(starter_candidates_ron)
            .expect("Failed to deserialize starter_candidates.ron");
    let recruitment_employee_db: RecruitmentEmployeeCandidateDatabase =
        ron::de::from_str(recruitment_candidates_ron)
            .expect("Failed to deserialize recruitment_candidates.ron");

    let equipments_db: EquipmentDatabase =
        ron::de::from_str(equipments_ron).expect("Failed to deserialize equipments.ron");

    let artifacts_db: ArtifactDatabase =
        ron::de::from_str(artifacts_ron).expect("Failed to deserialize artifacts.ron");

    let consumables_db: ConsumableDatabase =
        ron::de::from_str(consumables_ron).expect("Failed to deserialize consumables/base.ron");

    let buffs_db: BuffDatabase =
        ron::de::from_str(buffs_ron).expect("Failed to deserialize buffs/base.ron");

    let skill_db: SkillDatabase =
        ron::de::from_str(skills_ron).expect("Failed to deserialize skills.ron");
    let skill_fragment_db: SkillFragmentDatabase = ron::de::from_str(skill_fragments_ron)
        .expect("Failed to deserialize skill_fragments/base.ron");

    let pve_db: PveEncounterDatabase =
        ron::de::from_str(pve_ron).expect("Failed to deserialize pve encounters.ron");

    GameDataBuilder::empty()
        .with_abnormality_data(Arc::new(abnormalities_db))
        .with_corroded_employee_data(Arc::new(corroded_employee_db))
        .with_corroded_wave_data(Arc::new(corroded_wave_db))
        .with_starter_employee_data(Arc::new(starter_employee_db))
        .with_recruitment_employee_data(Arc::new(recruitment_employee_db))
        .with_artifact_data(Arc::new(artifacts_db))
        .with_consumable_data(Arc::new(consumables_db))
        .with_equipment_data(Arc::new(equipments_db))
        .with_shop_data(Arc::new(shops_db))
        .with_reward_data(Arc::new(rewards_db))
        .with_pve_data(Arc::new(pve_db))
        .with_buff_data(Arc::new(buffs_db))
        .with_skill_data(Arc::new(skill_db))
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::with_builtin_starter(
            skill_fragment_db.fragments,
        )))
        .build_arc()
}
