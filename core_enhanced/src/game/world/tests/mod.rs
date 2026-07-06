use super::*;
use crate::game::ability::{
    DeliveryDef, SkillCastTargetingDef, SkillDef, SkillId, SkillStepDef, SkillTarget,
};
use crate::game::battle::{
    buffs::BuffDatabase, event_log::BattleEventLog, tile_range::FacingDirection,
};
use crate::game::combat_preview::{
    ThreatWarning, ThreatWarningSource, ThreatWarningStatus, ThreatWarningTag,
};
use crate::game::data::{
    abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
    artifact_data::ArtifactMetadata,
    consumable_data::{
        ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
        ConsumableTargetPolicy, ConsumableTier,
    },
    corroded_employee_data::{
        CorrodedEmployeeProfileDatabase, CorrodedEmployeeProfileMetadata,
        CorrodedEmployeeProfileRole,
    },
    employee_data::{RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase},
    equipment_data::{
        EquipmentDatabase, EquipmentDismantleRecipeMetadata, EquipmentEnhancementRecipeMetadata,
        EquipmentMaterialMetadata, EquipmentMetadata, EquipmentRecipeMetadata, EquipmentType,
        WeaponArchetype, WeaponCombatProfile, WeaponRangeRole,
    },
    pve_data::{
        PveBattlefieldOverrideData, PveEncounter, PveEncounterClass, PveEncounterDatabase,
        PveWaveData, PveWaveEnemyData, PveWaveSource,
    },
    reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata},
    run_policy_data::RunPolicyData,
    shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType},
    skill_data::SkillDatabase,
    skill_fragment_data::{
        SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
        SkillFragmentEquipLimit, SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
    },
    GameDataBase, GameDataBuilder,
};
use crate::game::employee::{
    EmployeeAvailability, EmployeeLifeState, StarterEmployeeCandidate, StarterEmployeeLoadout,
};
use crate::game::enums::RewardMode;
use crate::game::map::{
    GameMode, MapGenerationConfig, MapGenerator, MapNode, MapNodeCategory, MapNodeId,
    MapNodeKindId, MapNodePayload, MapNodeState, MapNodeVisibility, MapSlotId, MapTemplateId,
    MapViewDto, RunMap, SupportNodeMode, SupportNodeType, DEFAULT_MAP_TEMPLATE_ID,
};
use crate::game::resources::item_slot::EquippedRef;
use crate::game::resources::{
    EquipItemOutcomeDto, OwnedConsumable, OwnedEquipment, Position, ShopSessionState,
};
use crate::game::reward::{ExperienceTargetPolicy, RewardEffect, RewardOption};
use crate::game::skill_fragment::starter_basic_attack_fragment_id;
use crate::game::stats::{StatId, StatModifier, StatModifierKind};
use serde_json::{json, Value};
use std::sync::Arc;

fn empty_game_data() -> Arc<GameDataBase> {
    test_game_data_builder().build_arc()
}

fn test_game_data_builder() -> GameDataBuilder {
    GameDataBuilder::empty()
        .with_starter_employee_candidates(test_starter_candidate_database())
        .with_recruitment_employee_candidates(test_recruitment_candidate_database())
        .with_equipment(vec![starter_equipment_meta()])
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
}

fn run_policy() -> RunPolicyData {
    RunPolicyData::builtin()
}

fn map_node_ids_by_state(map: &MapViewDto, state: MapNodeState) -> Vec<MapNodeId> {
    map.nodes
        .iter()
        .filter(|node| node.state == state)
        .map(|node| node.id)
        .collect()
}

fn test_starter_candidate_database() -> StarterEmployeeCandidateDatabase {
    StarterEmployeeCandidateDatabase::new(
        (0..6)
            .map(|index| StarterEmployeeCandidate {
                id: format!("candidate_{index}"),
                name: format!("Candidate {index}"),
                role: "테스트 후보".to_string(),
                background: "테스트용 시작 직원 후보".to_string(),
                starter_loadout: StarterEmployeeLoadout {
                    equipment_ids: vec!["standard_armor".to_string()],
                    baseline_skill_fragment_ids: vec![starter_basic_attack_fragment_id()],
                },
            })
            .collect(),
    )
}

fn test_recruitment_candidate_database() -> RecruitmentEmployeeCandidateDatabase {
    RecruitmentEmployeeCandidateDatabase::new(
        (0..6)
            .map(|index| StarterEmployeeCandidate {
                id: format!("recruit_{index}"),
                name: format!("Recruit {index}"),
                role: "테스트 채용 후보".to_string(),
                background: "테스트용 런 중 채용 후보".to_string(),
                starter_loadout: Default::default(),
            })
            .collect(),
    )
}

fn start_new_game_with_mode_and_default_starters(
    core: &mut GameCore,
    player_id: Uuid,
    game_mode: GameMode,
) -> BehaviorResult {
    let start = core
        .execute(player_id, PlayerBehavior::StartNewGame { game_mode })
        .expect("start new game should open starter selection");
    let BehaviorResult::StartNewGame {
        game_mode: returned_game_mode,
        candidates,
        required_count,
    } = start
    else {
        panic!("start new game should return starter candidates");
    };
    assert_eq!(returned_game_mode, game_mode);
    let candidate_ids = candidates
        .into_iter()
        .take(required_count)
        .map(|candidate| candidate.id)
        .collect::<Vec<_>>();
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees { candidate_ids },
    )
    .expect("default starter employee selection should start the run")
}

fn start_new_game_with_default_starters(core: &mut GameCore, player_id: Uuid) -> BehaviorResult {
    start_new_game_with_mode_and_default_starters(core, player_id, GameMode::Standard)
}

fn game_data_with_pve_encounters() -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![
            test_abnormality_meta("low_risk_abno", 20_001),
            test_boss_abnormality_meta("boss_risk_abno", 20_002),
            test_abnormality_meta("elite_risk_abno", 20_003),
            test_abnormality_meta("ambush_risk_abno", 20_004),
            test_abnormality_meta("defense_risk_abno", 20_005),
            test_abnormality_meta("defense_route_abno", 20_006),
            test_abnormality_meta("defense_corridor_abno", 20_007),
            test_boss_abnormality_meta("final_boss_risk_abno", 20_008),
        ])
        .with_corroded_employee_data(Arc::new(CorrodedEmployeeProfileDatabase::new(vec![
            test_corroded_employee_profile("corroded_guard", 21_001),
        ])))
        .with_pve(PveEncounterDatabase::new(vec![
            PveEncounter {
                id: "normal_corroded_encounter".to_string(),
                encounter_class: PveEncounterClass::Normal,
                primary_abnormality_id: None,
                risk_level: crate::game::enums::RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::OpenHall),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_corroded_pve_wave("corroded_guard")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "low_risk_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("low_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::OpenHall),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_defense_pve_wave("low_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "boss_risk_encounter".to_string(),
                encounter_class: PveEncounterClass::NormalBoss,
                primary_abnormality_id: Some("boss_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::ALEPH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Boss),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("boss_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "final_boss_risk_encounter".to_string(),
                encounter_class: PveEncounterClass::FinalBoss,
                primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::ALEPH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Boss),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("final_boss_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "elite_risk_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("elite_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::WAW,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_defense_pve_wave("elite_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "elite_survival_timer_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("ambush_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::WAW,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: Some(45_000),
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_defense_pve_wave("ambush_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    route_id: Some("defense_main".to_string()),
                    ..test_pve_wave("defense_risk_abno")
                }],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_route_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("defense_route_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_defense_pve_wave("defense_route_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_corridor_encounter".to_string(),
                encounter_class: PveEncounterClass::Elite,
                primary_abnormality_id: Some("defense_corridor_abno".to_string()),
                risk_level: crate::game::enums::RiskLevel::TETH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                suppression_research: None,
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_defense_pve_wave("defense_corridor_abno")],
                static_obstacles: vec![],
            },
        ]))
        .build_arc()
}

fn live_game_data_from_ron() -> Arc<GameDataBase> {
    GameDataBase::load_live_embedded()
}

fn test_abnormality_meta(id: &str, uuid: u128) -> AbnormalityMetadata {
    test_abnormality_meta_with_threat_class(
        id,
        uuid,
        crate::game::battle::types::BattleUnitThreatClass::Elite,
    )
}

fn test_boss_abnormality_meta(id: &str, uuid: u128) -> AbnormalityMetadata {
    test_abnormality_meta_with_threat_class(
        id,
        uuid,
        crate::game::battle::types::BattleUnitThreatClass::Boss,
    )
}

fn test_abnormality_meta_with_threat_class(
    id: &str,
    uuid: u128,
    threat_class: crate::game::battle::types::BattleUnitThreatClass,
) -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: id.to_string(),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        risk_level: crate::game::enums::RiskLevel::ZAYIN,
        price: 0,
        max_health: 10,
        attack: 0,
        defense: 0,
        magic_resist: 0,
        threat_class,
        response_complete_skill_fragment_id: Some(starter_basic_attack_fragment_id()),
        omen_chain_id: None,
        movement: MovementDef::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn test_corroded_employee_profile(id: &str, uuid: u128) -> CorrodedEmployeeProfileMetadata {
    CorrodedEmployeeProfileMetadata {
        id: id.to_string(),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        profile_role: CorrodedEmployeeProfileRole::Special,
        basic_attack_range_preset: None,
        max_health: 10,
        attack: 1,
        defense: 0,
        magic_resist: 0,
        movement: MovementDef::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
    }
}

fn test_pve_wave(abnormality_id: &str) -> PveWaveData {
    PveWaveData {
        id: "wave_0".to_string(),
        time_ms: 0,
        spawn_zone_ids: Vec::new(),
        route_id: None,
        required_for_victory: true,
        source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
            abnormality_id: abnormality_id.to_string(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        }]),
    }
}

fn test_corroded_pve_wave(profile_id: &str) -> PveWaveData {
    PveWaveData {
        id: "wave_0".to_string(),
        time_ms: 0,
        spawn_zone_ids: Vec::new(),
        route_id: Some("defense_main".to_string()),
        required_for_victory: true,
        source: PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
            profile_id: profile_id.to_string(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        }]),
    }
}

fn test_defense_pve_wave(abnormality_id: &str) -> PveWaveData {
    PveWaveData {
        route_id: Some("defense_main".to_string()),
        ..test_pve_wave(abnormality_id)
    }
}

fn game_data_with_map_content() -> Arc<GameDataBase> {
    let shop_uuid = Uuid::from_u128(10_001);
    let reward_uuid = Uuid::from_u128(10_002);

    test_game_data_builder()
        .with_shops(ShopDatabase::new_with_pools(
            vec![
                ShopMetadata {
                    id: "map_shop".to_string(),
                    name: "Map Shop".to_string(),
                    uuid: shop_uuid,
                    shop_type: ShopType::Shop,
                    can_reroll: false,
                    visible_items: vec![],
                    hidden_items: vec![],
                },
                ShopMetadata {
                    id: "hq_supply_shop".to_string(),
                    name: "HQ Supply Shop".to_string(),
                    uuid: Uuid::from_u128(10_006),
                    shop_type: ShopType::Shop,
                    can_reroll: false,
                    visible_items: vec![],
                    hidden_items: vec![],
                },
            ],
            vec![
                ShopPoolMetadata {
                    id: "default_shops".to_string(),
                    shop_ids: vec!["map_shop".to_string()],
                },
                ShopPoolMetadata {
                    id: "headquarters_basic_supplies".to_string(),
                    shop_ids: vec!["hq_supply_shop".to_string()],
                },
            ],
        ))
        .with_rewards(RewardDatabase::new_with_pools(
            vec![RewardMetadata {
                id: "map_reward".to_string(),
                uuid: reward_uuid,
                name: "Map Reward".to_string(),
                description: "Map reward".to_string(),
                icon: "test".to_string(),
                effects: vec![RewardEffect::GrantEnkephalin { amount: 7 }],
            }],
            vec![RewardPoolMetadata {
                id: "default_treasures".to_string(),
                reward_ids: vec!["map_reward".to_string()],
            }],
        ))
        .build_arc()
}

fn abnormality_meta(uuid: u128) -> Arc<AbnormalityMetadata> {
    Arc::new(AbnormalityMetadata {
        id: format!("abno-{uuid}"),
        uuid: Uuid::from_u128(uuid),
        name: format!("Abno {uuid}"),
        risk_level: crate::game::enums::RiskLevel::TETH,
        price: 100,
        max_health: 10,
        attack: 3,
        defense: 1,
        magic_resist: 0,
        threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
        response_complete_skill_fragment_id: Some(starter_basic_attack_fragment_id()),
        omen_chain_id: None,
        movement: MovementDef::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    })
}

fn equipment_meta(uuid: u128, id: &str, equipment_type: EquipmentType) -> EquipmentMetadata {
    EquipmentMetadata {
        id: id.to_string(),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        equipment_type,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: Default::default(),
        ability_activations: vec![],
        weapon_profile: (equipment_type == EquipmentType::Weapon).then(Default::default),
    }
}

fn starter_equipment_meta() -> EquipmentMetadata {
    equipment_meta(
        0x650e_8400_e29b_41d4_a716_4466_5544_0031,
        "standard_armor",
        EquipmentType::Armor,
    )
}

fn equipment_with_starter_assets(mut equipment: Vec<EquipmentMetadata>) -> Vec<EquipmentMetadata> {
    if !equipment.iter().any(|item| item.id == "standard_armor") {
        equipment.push(starter_equipment_meta());
    }
    equipment
}

fn weapon_equipment(uuid: u128, id: &str, archetype: WeaponArchetype) -> EquipmentMetadata {
    let mut equipment = equipment_meta(uuid, id, EquipmentType::Weapon);
    equipment.weapon_profile = Some(WeaponCombatProfile {
        weapon_archetype: archetype,
        range_role: match archetype {
            WeaponArchetype::Bow
            | WeaponArchetype::Gun
            | WeaponArchetype::GrenadeLauncher
            | WeaponArchetype::Staff => WeaponRangeRole::Ranged,
            WeaponArchetype::Sword | WeaponArchetype::Spear | WeaponArchetype::Shield => {
                WeaponRangeRole::Melee
            }
        },
        ..WeaponCombatProfile::default()
    });
    equipment
}

fn grant_and_equip_weapon(
    core: &mut GameCore,
    employee_uuid: Uuid,
    weapon: EquipmentMetadata,
    owned_uuid: Uuid,
) {
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(owned_uuid, Arc::new(weapon)))
        .unwrap();
    core.handle_equip_item(owned_uuid, employee_uuid).unwrap();
}

fn artifact_meta(uuid: u128, id: &str) -> ArtifactMetadata {
    ArtifactMetadata {
        id: id.to_string(),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        description: format!("{id} description"),
        rarity: crate::game::enums::RiskLevel::TETH,
        price: 5,
        triggered_effects: Default::default(),
        ability_activations: vec![],
    }
}

fn consumable_meta(
    uuid: u128,
    id: &str,
    tier: ConsumableTier,
    effect: ConsumableEffect,
) -> ConsumableMetadata {
    ConsumableMetadata {
        id: id.to_string(),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        description: format!("{id} description"),
        tier,
        rarity: crate::game::enums::RiskLevel::TETH,
        price: 10,
        target_policy: ConsumableTargetPolicy::SingleEmployee,
        duration_policy: ConsumableDurationPolicy::NextCombatNode,
        effect,
        live_pool: true,
    }
}

fn game_data_with_consumables(consumables: Vec<ConsumableMetadata>) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_consumable_data(Arc::new(ConsumableDatabase::new(consumables)))
        .build_arc()
}

fn game_data_with_display_items(
    equipment: Vec<EquipmentMetadata>,
    artifacts: Vec<ArtifactMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_equipment(equipment_with_starter_assets(equipment))
        .with_artifacts(artifacts)
        .build_arc()
}

fn game_data_with_equipment(
    abnormality: Arc<AbnormalityMetadata>,
    equipment: Vec<EquipmentMetadata>,
    recipes: Vec<EquipmentRecipeMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![(*abnormality).clone()])
        .with_equipment_data(Arc::new(EquipmentDatabase::with_recipes(
            equipment_with_starter_assets(equipment),
            recipes,
        )))
        .build_arc()
}

fn game_data_with_equipment_dismantle(
    abnormality: Arc<AbnormalityMetadata>,
    equipment: Vec<EquipmentMetadata>,
    materials: Vec<EquipmentMaterialMetadata>,
    dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![(*abnormality).clone()])
        .with_equipment_data(Arc::new(
            EquipmentDatabase::with_materials_recipes_and_dismantles(
                equipment_with_starter_assets(equipment),
                materials,
                vec![],
                dismantle_recipes,
            ),
        ))
        .build_arc()
}

fn game_data_with_equipment_enhancement(
    abnormality: Arc<AbnormalityMetadata>,
    equipment: Vec<EquipmentMetadata>,
    materials: Vec<EquipmentMaterialMetadata>,
    enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![(*abnormality).clone()])
        .with_equipment_data(Arc::new(EquipmentDatabase::with_all(
            equipment_with_starter_assets(equipment),
            materials,
            vec![],
            vec![],
            enhancement_recipes,
        )))
        .build_arc()
}

fn game_data_with_skill_fragments(fragments: Vec<SkillFragmentMetadata>) -> Arc<GameDataBase> {
    let skills = fragments
        .iter()
        .filter_map(|fragment| match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id, ..
            } => Some(SkillDef {
                id: imitation_skill_id.clone(),
                name: imitation_skill_id.to_string(),
                kind: Default::default(),
                cast_targeting: SkillCastTargetingDef::explicit(
                    SkillTarget::SelfUnit,
                    Default::default(),
                    None,
                    false,
                ),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
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
            }),
            _ => None,
        })
        .collect();

    test_game_data_builder()
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
        .build_arc()
}

fn game_data_with_equipment_and_skill_fragments(
    equipment: Vec<EquipmentMetadata>,
    fragments: Vec<SkillFragmentMetadata>,
) -> Arc<GameDataBase> {
    let skills = fragments
        .iter()
        .filter_map(|fragment| match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id, ..
            } => Some(SkillDef {
                id: imitation_skill_id.clone(),
                name: imitation_skill_id.to_string(),
                kind: Default::default(),
                cast_targeting: SkillCastTargetingDef::explicit(
                    SkillTarget::SelfUnit,
                    Default::default(),
                    None,
                    false,
                ),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
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
            }),
            _ => None,
        })
        .collect();

    test_game_data_builder()
        .with_equipment_data(Arc::new(EquipmentDatabase::new(
            equipment_with_starter_assets(equipment),
        )))
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
        .build_arc()
}

fn game_data_with_equipment_recipes_and_skill_fragments(
    equipment: Vec<EquipmentMetadata>,
    recipes: Vec<EquipmentRecipeMetadata>,
    fragments: Vec<SkillFragmentMetadata>,
) -> Arc<GameDataBase> {
    let skills = fragments
        .iter()
        .filter_map(|fragment| match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id, ..
            } => Some(SkillDef {
                id: imitation_skill_id.clone(),
                name: imitation_skill_id.to_string(),
                kind: Default::default(),
                cast_targeting: SkillCastTargetingDef::explicit(
                    SkillTarget::SelfUnit,
                    Default::default(),
                    None,
                    false,
                ),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
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
            }),
            _ => None,
        })
        .collect();

    test_game_data_builder()
        .with_equipment_data(Arc::new(EquipmentDatabase::with_recipes(
            equipment_with_starter_assets(equipment),
            recipes,
        )))
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
        .build_arc()
}

fn game_data_with_pve_equipment_and_active_skill_fragments(
    equipment: Vec<EquipmentMetadata>,
    skills: Vec<SkillDef>,
    fragments: Vec<SkillFragmentMetadata>,
) -> Arc<GameDataBase> {
    let base = game_data_with_pve_encounters();
    GameDataBuilder::empty()
        .with_abnormality_data(base.abnormality_data.clone())
        .with_corroded_employee_data(base.corroded_employee_data.clone())
        .with_corroded_wave_data(base.corroded_wave_data.clone())
        .with_starter_employee_data(base.starter_employee_data.clone())
        .with_recruitment_employee_data(base.recruitment_employee_data.clone())
        .with_artifact_data(base.artifact_data.clone())
        .with_equipment_data(Arc::new(EquipmentDatabase::new(
            equipment_with_starter_assets(equipment),
        )))
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_pve_data(base.pve_data.clone())
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
        .build_arc()
}

fn game_data_with_pve_and_consumables(consumables: Vec<ConsumableMetadata>) -> Arc<GameDataBase> {
    let base = game_data_with_pve_encounters();
    GameDataBuilder::empty()
        .with_abnormality_data(base.abnormality_data.clone())
        .with_corroded_employee_data(base.corroded_employee_data.clone())
        .with_corroded_wave_data(base.corroded_wave_data.clone())
        .with_starter_employee_data(base.starter_employee_data.clone())
        .with_recruitment_employee_data(base.recruitment_employee_data.clone())
        .with_artifact_data(base.artifact_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_pve_data(base.pve_data.clone())
        .with_consumable_data(Arc::new(ConsumableDatabase::new(consumables)))
        .with_skill_fragment_data(base.skill_fragment_data.clone())
        .build_arc()
}

fn active_skill_fragment(id: &str, uuid: u128, skill_id: &str) -> SkillFragmentMetadata {
    SkillFragmentMetadata {
        id: SkillFragmentId::from(id),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        description: "test active fragment".to_string(),
        rarity: SkillFragmentRarity::Rare,
        equip_limit: crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
        origin: None,
        sources: vec![SkillFragmentAcquisitionSource::RareReward],
        dependencies: vec![],
        compatibility: Default::default(),
        effect: SkillFragmentEffectDef::ActiveSkill {
            imitation_skill_id: SkillId::from(skill_id),
            upgrade_skill_ids: Default::default(),
            awakened_skill_id: None,
        },
    }
}

mod snapshots_and_start;

mod map_flow;

fn force_first_available_node(
    core: &mut GameCore,
    category: MapNodeCategory,
    kind_id: &str,
    payload: MapNodePayload,
) -> MapNodeId {
    let node_id = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map_progression
        .available_node_ids
        .first()
        .copied()
        .expect("available node");
    let map = &mut core.state.run.as_mut().expect("run state").map;
    let node = map.node_mut(node_id).expect("node exists");
    node.category = category;
    node.kind_id = crate::game::map::MapNodeKindId::new(kind_id);
    node.payload = payload;
    node_id
}

fn force_other_available_support_nodes_to_rest(core: &mut GameCore, except_node_id: MapNodeId) {
    let available_progression_node_ids = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map_progression
        .available_node_ids
        .clone();
    let map = &mut core.state.run.as_mut().expect("run state").map;
    for node_id in available_progression_node_ids {
        if node_id == except_node_id {
            continue;
        }
        let Some(node) = map.node_mut(node_id) else {
            continue;
        };
        if node.category != MapNodeCategory::Support {
            continue;
        }
        node.kind_id = crate::game::map::MapNodeKindId::new("support_rest");
        node.payload = MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        };
    }
}

fn select_and_confirm_map_node(
    core: &mut GameCore,
    player_id: Uuid,
    node_id: MapNodeId,
) -> BehaviorResult {
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    assert!(matches!(preview, BehaviorResult::NodePreview { .. }));
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap()
}

fn write_world_debug_event_log_export(
    name: &str,
    event_log: &BattleEventLog,
) -> std::path::PathBuf {
    let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug_event_log_exports");
    let out_path = out_dir.join(format!("{name}.json"));
    std::fs::create_dir_all(&out_dir).expect("create debug_event_log_exports directory");
    event_log
        .write_pretty_json(&out_path)
        .expect("write debug event log json");
    out_path
}

mod support;

mod node_sessions;

mod equipment;

mod combat;

mod placement;
