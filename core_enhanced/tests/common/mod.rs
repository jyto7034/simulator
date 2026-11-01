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
use game_core::game::battle::event_log::BattleEventLog;
use game_core::game::battle::scenario::{TerminalCondition, TerminalOutcome};
use game_core::game::battle::types::{BattleOutcomeId, BattleVerdict};
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase;
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::pve_data::{
    PveEncounter, PveEncounterClass, PveEncounterDatabase, PveOutcomePolicyData,
    PveTerminalConditionData, PveTerminalOutcomeData, PveWaveData, PveWaveEnemyData, PveWaveSource,
};
use game_core::game::data::reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata};
use game_core::game::data::shop_data::{
    ShopContext, ShopDatabase, ShopMerchandise, ShopMerchandiseEntry, ShopMetadata,
    ShopPoolMetadata, ShopRefreshPolicy, ShopSellPolicy,
};
use game_core::game::data::skill_data::SkillDatabase;
use game_core::game::data::{GameDataBase, GameDataBuilder};
use game_core::game::enums::{RewardMode, RiskLevel};
use game_core::game::reward::RewardEffect;
use uuid::Uuid;

pub fn required_group_terminal_outcomes() -> Vec<TerminalOutcome> {
    vec![
        TerminalOutcome {
            id: BattleOutcomeId::new("victory"),
            verdict: BattleVerdict::Victory,
            condition: TerminalCondition::AllRequiredEnemyGroupsDefeated,
            priority: 0,
        },
        TerminalOutcome {
            id: BattleOutcomeId::new("defeat"),
            verdict: BattleVerdict::Defeat,
            condition: TerminalCondition::AllPlayerCombatantsDefeated,
            priority: 0,
        },
    ]
}

fn pve_terminal_outcomes() -> Vec<PveTerminalOutcomeData> {
    vec![
        PveTerminalOutcomeData {
            id: "victory".to_string(),
            verdict: BattleVerdict::Victory,
            condition: PveTerminalConditionData::AllRequiredEnemyGroupsDefeated,
            priority: 0,
        },
        PveTerminalOutcomeData {
            id: "defeat".to_string(),
            verdict: BattleVerdict::Defeat,
            condition: PveTerminalConditionData::AllPlayerCombatantsDefeated,
            priority: 0,
        },
    ]
}

fn pve_outcome_policies(
    reward_mode: RewardMode,
    reward_uuids: Vec<Uuid>,
) -> Vec<PveOutcomePolicyData> {
    vec![
        PveOutcomePolicyData {
            outcome_id: "victory".to_string(),
            reward_mode,
            reward_uuids,
            grant_node_bonus: true,
            advance_encounter_progression: true,
        },
        PveOutcomePolicyData {
            outcome_id: "defeat".to_string(),
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: Vec::new(),
            grant_node_bonus: false,
            advance_encounter_progression: false,
        },
    ]
}

pub fn debug_event_log_exports_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug_event_log_exports")
}

pub fn write_debug_event_log_export(name: &str, event_log: &BattleEventLog) -> PathBuf {
    let out_dir = debug_event_log_exports_dir();
    let out_path = out_dir.join(format!("{name}.json"));
    let parent_dir = out_path.parent().unwrap_or_else(|| {
        panic!(
            "debug event log export path must have parent: {}",
            out_path.display()
        )
    });
    std::fs::create_dir_all(parent_dir).expect("create debug_event_log_exports directory");
    let file = std::fs::File::create(&out_path).expect("create debug event log json");
    serde_json::to_writer_pretty(std::io::BufWriter::new(file), event_log)
        .expect("write debug event log json");
    out_path
}

pub fn empty_game_data() -> Arc<GameDataBase> {
    GameDataBuilder::empty().build_arc()
}

/// 테스트용 `GameDataBase` 생성 (작고 결정적인 데이터).
///
/// - 리롤 가능한 상점 1개 (visible/hidden 구성)
/// - Equipment 최소 1개
/// - Abnormality combat metadata 최소 1개
/// - Reward/PvE encounter 최소 1개
pub fn create_test_game_data() -> Arc<GameDataBase> {
    let shop_uuid = Uuid::from_u128(1);
    let reward_uuid = Uuid::from_u128(2);

    let equipment1 = EquipmentMetadata {
        content_family: None,
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
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let equipment2 = EquipmentMetadata {
        content_family: None,
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
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };

    let skill_id = SkillId::from("test_skill");
    let skills_db = SkillDatabase::new(vec![SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: game_core::game::ability::SkillUseLifetime::Action,
        flow: None,
        id: skill_id.clone(),
        name: "test_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::SelfUnit,
            Default::default(),
            None,
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
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
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: None,
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
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: None,
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
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: None,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    };

    let shop = ShopMetadata {
        id: "test_shop".to_string(),
        name: "Test Refreshable Shop".to_string(),
        uuid: shop_uuid,
        context: ShopContext::UnknownExploration,
        offer_count: 2,
        default_buy_percent: 100,
        preset_refs: vec![],
        local_entries: [&equipment1, &equipment2]
            .into_iter()
            .map(|equipment| ShopMerchandiseEntry {
                merchandise: ShopMerchandise::Equipment {
                    equipment_id: equipment.id.clone(),
                },
                quantity: 1,
            base_price: None,
            guaranteed: false,
                weight: 1,
                repeatable: false,
                buy_percent: None,
                tags: vec!["equipment".to_string()],
            })
            .collect(),
        refresh_policy: Some(ShopRefreshPolicy {
            base_cost: 0,
            max_uses: 1,
        }),
        sell_policy: Some(ShopSellPolicy {
            price_percent: 40,
            accepted_tags: None,
        }),
    };

    let reward = RewardMetadata {
        id: "test_reward_enkephalin".to_string(),
        uuid: reward_uuid,
        name: "Test Enkephalin Reward".to_string(),
        description: "Grants fixed Enkephalin".to_string(),
        icon: "enkephalin_icon.png".to_string(),
        effects: vec![RewardEffect::GrantEnkephalin { amount: 100 }],
    };

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
            deterrence_tuning: Default::default(),
            bonus_objectives: Vec::new(),
            id: "pve_test_1".to_string(),
            encounter_class: PveEncounterClass::Elite,
            primary_abnormality_id: Some(abnormality1.id.clone()),
            risk_level: RiskLevel::ZAYIN,
            outcome_policies: pve_outcome_policies(RewardMode::ChooseOne, vec![reward_uuid]),
            node_type: None,
            mission_variant: None,
            terminal_outcomes: pve_terminal_outcomes(),
            battlefield: None,
            tactical_plan: None,
            boss_runtime: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                activation: game_core::game::data::pve_data::PveWaveActivationData::at_time(
                    "wave_0", 0,
                ),
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                    abnormality_id: abnormality1.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }]),
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            deterrence_tuning: Default::default(),
            bonus_objectives: Vec::new(),
            id: "pve_test_2".to_string(),
            encounter_class: PveEncounterClass::Elite,
            primary_abnormality_id: Some(abnormality2.id.clone()),
            risk_level: RiskLevel::TETH,
            outcome_policies: pve_outcome_policies(RewardMode::ChooseOne, vec![reward_uuid]),
            node_type: None,
            mission_variant: None,
            terminal_outcomes: pve_terminal_outcomes(),
            battlefield: None,
            tactical_plan: None,
            boss_runtime: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                activation: game_core::game::data::pve_data::PveWaveActivationData::at_time(
                    "wave_0", 0,
                ),
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                    abnormality_id: abnormality2.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }]),
            }],
            static_obstacles: vec![],
        },
        PveEncounter {
            deterrence_tuning: Default::default(),
            bonus_objectives: Vec::new(),
            id: "pve_test_3".to_string(),
            encounter_class: PveEncounterClass::Elite,
            primary_abnormality_id: Some(abnormality3.id.clone()),
            risk_level: RiskLevel::ZAYIN,
            outcome_policies: pve_outcome_policies(RewardMode::ChooseOne, vec![reward_uuid]),
            node_type: None,
            mission_variant: None,
            terminal_outcomes: pve_terminal_outcomes(),
            battlefield: None,
            tactical_plan: None,
            boss_runtime: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                activation: game_core::game::data::pve_data::PveWaveActivationData::at_time(
                    "wave_0", 0,
                ),
                spawn_zone_ids: Vec::new(),
                route_id: None,
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                    abnormality_id: abnormality3.id.clone(),
                    tier: game_core::game::enums::Tier::I,
                    count: 1,
                }]),
            }],
            static_obstacles: vec![],
        },
    ]);

    GameDataBuilder::empty()
        .with_abnormality_data(Arc::new(abnormalities_db))
        .with_corroded_employee_data(Arc::new(CorrodedEmployeeProfileDatabase::new(vec![])))
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
/// - 상점 데이터 (events/shops/base.ron)
/// - 보상 데이터 (rewards.ron)
/// - 환상체 데이터 (abnormalities.ron)
/// - 장비 데이터 (equipments.ron)
/// - 섭취 아이템 데이터 (consumables/base.ron)
/// - 통합 직원 definition 데이터 (employees/definitions.ron)
/// - 스킬 파편 데이터 (skill_fragments/base.ron)
/// - 이벤트 노드 데이터 (events/story/base.ron)
/// - 보스 전조 체인 데이터 (boss_omen/chains.ron)
#[allow(dead_code)]
pub fn load_game_data_from_ron() -> Arc<GameDataBase> {
    game_core::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid")
        .into_fixture_database_arc()
}
