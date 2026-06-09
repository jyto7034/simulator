use super::*;
use crate::game::ability::{DeliveryDef, SkillDef, SkillId, SkillStepDef, SkillTarget};
use crate::game::battle::{buffs::BuffDatabase, tile_range::FacingDirection, timeline::Timeline};
use crate::game::combat_preview::{
    ThreatWarning, ThreatWarningSource, ThreatWarningStatus, ThreatWarningTag,
};
use crate::game::data::{
    abnormality_data::{
        AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef,
    },
    artifact_data::{ArtifactDatabase, ArtifactMetadata},
    consumable_data::{
        ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
        ConsumableTargetPolicy, ConsumableTier,
    },
    corroded_employee_data::CorrodedEmployeeProfileDatabase,
    corroded_wave_data::CorrodedWavePresetDatabase,
    employee_data::{RecruitmentEmployeeCandidateDatabase, StarterEmployeeCandidateDatabase},
    equipment_data::{
        EquipmentDatabase, EquipmentDismantleRecipeMetadata, EquipmentEnhancementRecipeMetadata,
        EquipmentMaterialCost, EquipmentMaterialMetadata, EquipmentMaterialType, EquipmentMetadata,
        EquipmentRecipeMetadata, EquipmentType, WeaponArchetype, WeaponCombatProfile,
        WeaponRangeRole,
    },
    pve_data::{
        PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase, PveWaveData,
        PveWaveEnemyData,
    },
    reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata, RewardTag},
    shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType},
    skill_data::SkillDatabase,
    skill_fragment_data::{
        SkillFragmentAcquisitionSource, SkillFragmentCompatibilityFailureCode,
        SkillFragmentCompatibilityRequirements, SkillFragmentDatabase, SkillFragmentEffectDef,
        SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
    },
    GameDataBase, GameDataBuilder,
};
use crate::game::employee::{
    EmployeeAvailability, EmployeeGrade, EmployeeLifeState, StarterEmployeeCandidate,
};
use crate::game::enums::{RewardMode, Side};
use crate::game::map::{
    MapGenerationConfig, MapGenerator, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
    MapNodePayload, MedicalTreatmentKind, RunMap, SupportNodeMode, SupportNodeType,
};
use crate::game::resources::{
    EquipItemOutcomeDto, OwnedConsumable, OwnedEquipment, Position, ShopSessionState,
};
use crate::game::reward::RewardEffect;
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
}

fn test_starter_candidate_database() -> StarterEmployeeCandidateDatabase {
    StarterEmployeeCandidateDatabase::new(
        (0..6)
            .map(|index| StarterEmployeeCandidate {
                id: format!("candidate_{index}"),
                name: format!("Candidate {index}"),
                grade: EmployeeGrade::Junior,
                role: "테스트 후보".to_string(),
                background: "테스트용 시작 직원 후보".to_string(),
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
                grade: EmployeeGrade::Junior,
                role: "테스트 채용 후보".to_string(),
                background: "테스트용 런 중 채용 후보".to_string(),
            })
            .collect(),
    )
}

fn start_new_game_with_default_starters(core: &mut GameCore, player_id: Uuid) -> BehaviorResult {
    let start = core
        .execute(player_id, PlayerBehavior::StartNewGame)
        .expect("start new game should open starter selection");
    let BehaviorResult::StartNewGame {
        candidates,
        required_count,
    } = start
    else {
        panic!("start new game should return starter candidates");
    };
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

fn game_data_with_pve_encounters() -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![
            test_abnormality_meta("low_risk_abno", 20_001),
            test_abnormality_meta("boss_risk_abno", 20_002),
            test_abnormality_meta("elite_risk_abno", 20_003),
            test_abnormality_meta("ambush_risk_abno", 20_004),
            test_abnormality_meta("defense_risk_abno", 20_005),
            test_abnormality_meta("defense_route_abno", 20_006),
            test_abnormality_meta("defense_corridor_abno", 20_007),
        ])
        .with_pve(PveEncounterDatabase::new(vec![
            PveEncounter {
                id: "low_risk_encounter".to_string(),
                abnormality_id: "low_risk_abno".to_string(),
                difficulty: 1,
                risk_level: crate::game::enums::RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::OpenHall),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("low_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "boss_risk_encounter".to_string(),
                abnormality_id: "boss_risk_abno".to_string(),
                difficulty: 9,
                risk_level: crate::game::enums::RiskLevel::ALEPH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Boss),
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("boss_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "elite_risk_encounter".to_string(),
                abnormality_id: "elite_risk_abno".to_string(),
                difficulty: 4,
                risk_level: crate::game::enums::RiskLevel::WAW,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("elite_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "elite_encirclement_encounter".to_string(),
                abnormality_id: "ambush_risk_abno".to_string(),
                difficulty: 4,
                risk_level: crate::game::enums::RiskLevel::WAW,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: Some(
                    crate::game::combat_preview::CombatMissionVariant::Encirclement,
                ),
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("ambush_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_encounter".to_string(),
                abnormality_id: "defense_risk_abno".to_string(),
                difficulty: 3,
                risk_level: crate::game::enums::RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    route_id: Some("black_box_breach_main".to_string()),
                    ..test_pve_wave("defense_risk_abno")
                }],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_route_encounter".to_string(),
                abnormality_id: "defense_route_abno".to_string(),
                difficulty: 3,
                risk_level: crate::game::enums::RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("defense_route_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "defense_corridor_encounter".to_string(),
                abnormality_id: "defense_corridor_abno".to_string(),
                difficulty: 2,
                risk_level: crate::game::enums::RiskLevel::TETH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("defense_corridor_abno")],
                static_obstacles: vec![],
            },
        ]))
        .build_arc()
}

fn live_game_data_from_ron() -> Arc<GameDataBase> {
    let shops_db: ShopDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/events/shops/base.ron"
    ))
    .expect("shops/base.ron should deserialize");
    let rewards_db: RewardDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/events/rewards/base.ron"
    ))
    .expect("rewards/base.ron should deserialize");
    let abnormalities_db: AbnormalityDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/abnormalities/base.ron"
    ))
    .expect("abnormalities/base.ron should deserialize");
    let corroded_employee_db: CorrodedEmployeeProfileDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/enemies/corroded_employees.ron"
    ))
    .expect("corroded_employees.ron should deserialize");
    let corroded_wave_db: CorrodedWavePresetDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/enemies/corroded_wave_presets.ron"
    ))
    .expect("corroded_wave_presets.ron should deserialize");
    let starter_employee_db: StarterEmployeeCandidateDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/employees/starter_candidates.ron"
    ))
    .expect("starter_candidates.ron should deserialize");
    let recruitment_employee_db: RecruitmentEmployeeCandidateDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/employees/recruitment_candidates.ron"),
    )
    .expect("recruitment_candidates.ron should deserialize");
    let equipments_db: EquipmentDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/equipments/base.ron"
    ))
    .expect("equipments/base.ron should deserialize");
    let artifacts_db: ArtifactDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/artifacts/base.ron"
    ))
    .expect("artifacts/base.ron should deserialize");
    let buffs_db: BuffDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/buffs/base.ron"
    ))
    .expect("buffs/base.ron should deserialize");
    let skill_db: SkillDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/skills/base.ron"
    ))
    .expect("skills/base.ron should deserialize");
    let skill_fragment_db: SkillFragmentDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/skill_fragments/base.ron"
    ))
    .expect("skill_fragments/base.ron should deserialize");
    let pve_db: PveEncounterDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/pve/encounters.ron"
    ))
    .expect("pve/encounters.ron should deserialize");

    GameDataBuilder::empty()
        .with_abnormality_data(Arc::new(abnormalities_db))
        .with_corroded_employee_data(Arc::new(corroded_employee_db))
        .with_corroded_wave_data(Arc::new(corroded_wave_db))
        .with_starter_employee_data(Arc::new(starter_employee_db))
        .with_recruitment_employee_data(Arc::new(recruitment_employee_db))
        .with_artifact_data(Arc::new(artifacts_db))
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

fn test_abnormality_meta(id: &str, uuid: u128) -> AbnormalityMetadata {
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
        movement: MovementDef::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn test_pve_wave(abnormality_id: &str) -> PveWaveData {
    PveWaveData {
        id: "wave_0".to_string(),
        time_ms: 0,
        spawn_zone_ids: Vec::new(),
        route_id: None,
        required_for_victory: true,
        source: None,
        enemies: vec![PveWaveEnemyData {
            kind: crate::game::combat_preview::EnemyKind::Abnormality,
            profile_id: None,
            abnormality_id: abnormality_id.to_string(),
            tier: crate::game::enums::Tier::I,
            count: 1,
        }],
    }
}

fn game_data_with_map_content() -> Arc<GameDataBase> {
    let shop_uuid = Uuid::from_u128(10_001);
    let reward_uuid = Uuid::from_u128(10_002);
    let forbidden_reward_uuid = Uuid::from_u128(10_004);

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
            vec![
                RewardMetadata {
                    id: "map_reward".to_string(),
                    uuid: reward_uuid,
                    name: "Map Reward".to_string(),
                    description: "Map reward".to_string(),
                    icon: "test".to_string(),
                    tags: Vec::new(),
                    effects: vec![RewardEffect::GrantEnkephalin { amount: 7 }],
                },
                RewardMetadata {
                    id: "map_forbidden_reward".to_string(),
                    uuid: forbidden_reward_uuid,
                    name: "Forbidden Map Reward".to_string(),
                    description: "Forbidden reward must not be reachable from map reward pools."
                        .to_string(),
                    icon: "test".to_string(),
                    tags: vec![RewardTag::Forbidden],
                    effects: vec![RewardEffect::ForbiddenAbnormalityGrant],
                },
            ],
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
        .with_equipment(equipment)
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
            equipment, recipes,
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
                equipment,
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
            equipment,
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
                cast_targeting: Default::default(),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
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
                cast_targeting: Default::default(),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
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
        .with_equipment_data(Arc::new(EquipmentDatabase::new(equipment)))
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
                cast_targeting: Default::default(),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![SkillStepDef {
                    id: "test_step".to_string(),
                    delay_ms: 0,
                    range_units: 1.0,
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
            equipment, recipes,
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
        .with_equipment_data(Arc::new(EquipmentDatabase::new(equipment)))
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
        .build_arc()
}

fn active_skill_fragment(id: &str, uuid: u128, skill_id: &str) -> SkillFragmentMetadata {
    SkillFragmentMetadata {
        id: SkillFragmentId::from(id),
        uuid: Uuid::from_u128(uuid),
        name: id.to_string(),
        description: "test active fragment".to_string(),
        rarity: SkillFragmentRarity::Rare,
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

mod snapshots_and_start {
    use super::*;

    #[test]
    fn selected_shop_snapshot_includes_display_item_metadata() {
        let blade = equipment_meta(0x10, "raw_blade_core", EquipmentType::Weapon);
        let lens = artifact_meta(0x20, "artifact_echo_lens");
        let game_data = game_data_with_display_items(vec![blade.clone()], vec![lens.clone()]);
        let mut core = GameCore::new(game_data, 123);

        core.state.active_node_content = Some(ActiveNodeContent::Shop(ShopSessionState {
            id: "artifact_shop".to_string(),
            name: "Artifact Merchant".to_string(),
            uuid: Uuid::from_u128(0x30),
            shop_type: crate::game::data::shop_data::ShopType::Shop,
            can_reroll: true,
            visible_items: vec![blade.uuid],
            hidden_items: vec![lens.uuid],
        }));

        let selected = core
            .get_selected_event_snapshot_json()
            .unwrap()
            .expect("selected event snapshot");

        assert_eq!(selected["visible_items"][0]["uuid"], json!(blade.uuid));
        assert_eq!(selected["visible_items"][0]["kind"], "equipment");
        assert_eq!(
            selected["visible_items"][0]["definition_id"],
            "raw_blade_core"
        );
        assert_eq!(selected["visible_items"][0]["id"], "raw_blade_core");
        assert_eq!(selected["visible_items"][0]["equipment_type"], "weapon");
        assert!(selected["visible_items"][0]["icon"].is_null());
        assert_eq!(selected["hidden_items"][0]["kind"], "artifact");
        assert_eq!(
            selected["hidden_items"][0]["definition_id"],
            "artifact_echo_lens"
        );
        assert_eq!(selected["hidden_items"][0]["id"], "artifact_echo_lens");
        assert_eq!(
            selected["hidden_items"][0]["description"],
            "artifact_echo_lens description"
        );
        assert!(selected["hidden_items"][0]["icon"].is_null());
        assert_eq!(selected["visible_item_uuids"][0], json!(blade.uuid));
        assert_eq!(selected["hidden_item_uuids"][0], json!(lens.uuid));
    }

    #[test]
    fn game_core_rejects_disallowed_actions_and_updates_allowed_actions_on_state_transition() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);

        assert!(matches!(core.get_state(), GameState::NotStarted));
        assert!(core.is_action_allowed(&PlayerBehavior::StartNewGame));
        assert!(!core.is_action_allowed(&PlayerBehavior::RequestMapData));

        let err = core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let res = core
            .execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        let BehaviorResult::StartNewGame {
            candidates,
            required_count,
        } = res
        else {
            panic!("start should expose starter candidates");
        };
        assert_eq!(
            required_count,
            RUN_SYSTEM_POLICY.setup.starter_employee_count
        );
        assert_eq!(candidates.len(), 6);
        assert!(matches!(
            core.get_state(),
            GameState::SelectingStarterEmployees
        ));
        assert_eq!(
            core.get_allowed_actions(),
            vec![ActionKind::SelectStarterEmployees]
        );

        let selected_ids = candidates
            .into_iter()
            .take(required_count)
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>();
        let selected = core
            .execute(
                player_id,
                PlayerBehavior::SelectStarterEmployees {
                    candidate_ids: selected_ids,
                },
            )
            .unwrap();
        assert!(matches!(
            selected,
            BehaviorResult::StarterEmployeesSelected { .. }
        ));
        assert!(matches!(core.get_state(), GameState::ViewingMap));

        let allowed = core.get_allowed_actions();
        assert!(allowed.contains(&ActionKind::RequestMapData));
        assert!(allowed.contains(&ActionKind::SelectMapNode));
        assert!(allowed.contains(&ActionKind::EquipItem));
        assert!(allowed.contains(&ActionKind::MoveRosterUnit));
    }

    #[test]
    fn start_new_game_creates_starter_employee_roster_and_order() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);

        start_new_game_with_default_starters(&mut core, player_id);

        let roster = core.roster().expect("employee roster exists");
        assert_eq!(roster.len(), RUN_SYSTEM_POLICY.setup.starter_employee_count);

        let employee_ids = roster.available_employee_ids();
        assert_eq!(
            employee_ids.len(),
            RUN_SYSTEM_POLICY.setup.starter_employee_count
        );
        let roster_order = core.roster_order().expect("roster order exists");
        for employee_id in employee_ids {
            assert!(roster_order.slot_of(employee_id).is_some());
        }
    }

    #[test]
    fn employee_roster_snapshot_exposes_status_loadout_and_roster_slot() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_id = core.roster().unwrap().available_employee_ids()[0];

        let snapshot = core.get_employee_roster_snapshot_json().unwrap();
        let employees = snapshot["employees"].as_array().unwrap();
        let employee = employees
            .iter()
            .find(|employee| employee["uuid"] == json!(employee_id))
            .expect("starter employee is exposed in roster snapshot");

        assert_eq!(employee["life_state"], "Alive");
        assert_eq!(employee["availability"], "Available");
        assert_eq!(employee["available_for_combat"], true);
        assert_eq!(employee["trauma"], 0);
        assert!(employee.get("field_position").is_none());
        assert_eq!(employee["roster_slot"], 0);
        assert_eq!(employee["equipped_items"].as_array().unwrap().len(), 0);
        assert_eq!(
            employee["combat_profile"]["deployment_affinity"],
            "ground_only"
        );
        assert_eq!(
            employee["combat_profile"]["effective_deployment_affinity"],
            "ground_only"
        );
        assert!(snapshot["available_employee_ids"]
            .as_array()
            .unwrap()
            .contains(&json!(employee_id)));
    }

    #[test]
    fn employee_roster_snapshot_surfaces_effective_profile_errors() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_id = core.roster().unwrap().available_employee_ids()[0];
        let stale_item_uuid = Uuid::from_u128(0xEFFE_C710);

        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_id).unwrap();
            employee
                .loadout
                .item_slot
                .equip(
                    crate::game::resources::item_slot::EquippedRef {
                        instance_uuid: stale_item_uuid,
                        base_uuid: Uuid::from_u128(0xBADC_0DE),
                        equipment_type: EquipmentType::Weapon,
                    },
                    true,
                )
                .unwrap();
        }

        let snapshot = core.get_employee_roster_snapshot_json().unwrap();
        let employee = snapshot["employees"]
            .as_array()
            .unwrap()
            .iter()
            .find(|employee| employee["uuid"] == json!(employee_id))
            .expect("starter employee is exposed in roster snapshot");
        let combat_profile = &employee["combat_profile"];

        assert_eq!(
            combat_profile["effective_profile_error"]["code"],
            "inventory_item_not_found"
        );
        assert!(combat_profile["effective_profile_error"]["message"]
            .as_str()
            .unwrap()
            .contains("InventoryItemNotFound"));
        assert_eq!(combat_profile["effective_stats"], Value::Null);
        assert_eq!(combat_profile["effective_weapon_profile"], Value::Null);
        assert_eq!(combat_profile["effective_deployment_affinity"], Value::Null);
    }

    #[test]
    fn run_snapshot_exposes_current_flow_after_start() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);

        let snapshot = core.get_run_snapshot_json().unwrap();

        assert_eq!(snapshot["game_state"], "viewing_map");
        assert_eq!(snapshot["game_state_context"]["type"], "viewing_map");
        assert_eq!(snapshot["run_progression"]["act_index"], 0);
        assert_eq!(
            snapshot["run_progression"]["max_acts"],
            RUN_SYSTEM_POLICY.setup.default_max_acts
        );
        assert!(snapshot["allowed_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("SelectMapNode")));
        assert!(snapshot["map"]["nodes"].as_array().unwrap().len() > 0);
        assert!(
            snapshot["map"]["available_node_ids"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
        assert_eq!(
            snapshot["roster"]["employees"].as_array().unwrap().len(),
            RUN_SYSTEM_POLICY.setup.starter_employee_count
        );
        assert_eq!(snapshot["current_node_session"], Value::Null);
        assert_eq!(snapshot["resources"]["enkephalin"], 500);
    }

    #[test]
    fn run_snapshot_exposes_current_node_session_after_node_entry() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_choice",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::LimitedChoice,
                choices: vec![SupportNodeType::Medical, SupportNodeType::Rest],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let snapshot = core.get_run_snapshot_json().unwrap();

        assert_eq!(snapshot["game_state"], "node_confirm");
        assert_eq!(snapshot["game_state_context"]["node_id"], json!(node_id));
        assert_eq!(snapshot["current_node_session"]["node_id"], json!(node_id));
        assert_eq!(
            snapshot["current_node_session"]["payload"]["Support"]["support_mode"],
            "LimitedChoice"
        );
        assert!(snapshot["allowed_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("ConfirmEnterNode")));
        assert!(snapshot["allowed_actions"]
            .as_array()
            .unwrap()
            .contains(&json!("SelectMapNode")));
    }
}

mod map_flow {
    use super::*;
    use crate::game::map::MapNodeState;

    #[test]
    fn start_new_game_exposes_initial_map_without_entering_a_node_session() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);

        start_new_game_with_default_starters(&mut core, player_id);
        let map = match core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap()
        {
            BehaviorResult::MapState { map } => map,
            other => panic!("expected map state, got {other:?}"),
        };

        assert_eq!(map.act_index, 0);
        assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.setup.default_max_acts);
        assert_eq!(map.current_node_id, None);
        assert!(!map.nodes.is_empty());
        assert!(!map.edges.is_empty());
        assert!(!map.available_node_ids.is_empty());
        assert!(map.nodes.iter().any(|node| node.id == map.boss_node_id));
        assert!(map.available_node_ids.iter().all(|node_id| {
            map.nodes
                .iter()
                .any(|node| node.id == *node_id && node.state == MapNodeState::Available)
        }));
        assert!(core.state.node_session.is_none());
        assert!(matches!(core.get_state(), GameState::ViewingMap));
    }

    #[test]
    fn combat_preview_seed_uses_full_generated_node_uuid() {
        const MAP_NS: u64 = 0x524D_4150; // "RMAP"
        const PREVIEW_NS: u64 = 0x5052_4556; // "PREV"

        let core = GameCore::new(empty_game_data(), 123);
        let map_seed = crate::game::determinism::seed_with_namespace(123, MAP_NS);
        let node_ids = (0..32)
            .map(|index| {
                MapNodeId::new(crate::game::determinism::uuid_v4_from_seed(
                    map_seed, MAP_NS, index,
                ))
            })
            .collect::<Vec<_>>();

        assert!(
            node_ids
                .windows(2)
                .all(|pair| pair[0].0.as_bytes()[..8] == pair[1].0.as_bytes()[..8]),
            "this regression fixture must match MapGenerator's shared UUID prefix family"
        );

        let pair = node_ids
            .iter()
            .copied()
            .flat_map(|left| node_ids.iter().copied().map(move |right| (left, right)))
            .find(|(left, right)| {
                left != right
                    && core.node_seed(*left, PREVIEW_NS) % 7
                        != core.node_seed(*right, PREVIEW_NS) % 7
            })
            .expect("full UUID seed mixing should vary combat archetype buckets");

        let left_seed = core.node_seed(pair.0, PREVIEW_NS);
        let right_seed = core.node_seed(pair.1, PREVIEW_NS);
        let game_data = empty_game_data();
        let left = crate::game::combat_preview::CombatPreview::try_generate_for_node(
            pair.0,
            MapNodeCategory::Combat,
            None,
            game_data.as_ref(),
            left_seed,
        )
        .expect("left combat preview should generate");
        let right = crate::game::combat_preview::CombatPreview::try_generate_for_node(
            pair.1,
            MapNodeCategory::Combat,
            None,
            game_data.as_ref(),
            right_seed,
        )
        .expect("right combat preview should generate");

        assert_ne!(left.archetype, right.archetype);
        assert_ne!(left.battlefield_template_id, right.battlefield_template_id);
    }

    #[test]
    fn generated_map_node_ids_feed_distinct_node_seeds() {
        const PREVIEW_NS: u64 = 0x5052_4556; // "PREV"

        let core = GameCore::new(empty_game_data(), 123);
        let map = MapGenerator::generate(123, MapGenerationConfig::default());
        let mut seeds = map
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.category,
                    MapNodeCategory::Combat | MapNodeCategory::Boss
                )
            })
            .map(|node| core.node_seed(node.id, PREVIEW_NS))
            .collect::<Vec<_>>();

        seeds.sort_unstable();
        seeds.dedup();

        assert!(
            seeds.len() > 1,
            "generated map combat nodes should not collapse to one preview seed"
        );
    }

    #[test]
    fn combat_node_preview_exposes_basic_briefing() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );

        let result = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();

        let BehaviorResult::NodePreview {
            combat_preview: Some(preview),
            ..
        } = result
        else {
            panic!("expected combat node preview");
        };
        assert_eq!(preview.node_id, node_id);
        assert_eq!(preview.encounter_id.as_deref(), Some("low_risk_encounter"));
        assert!(!preview.deployment_zones.is_empty());
        assert!(!preview.spawn_zones.is_empty());
        assert!(!preview.spawn_waves.is_empty());
        assert_eq!(preview.spawn_waves[0].time_ms, 0);
    }

    #[test]
    fn node_confirm_allows_reselecting_another_available_node_before_entering() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let available_node_ids = core
            .state
            .run
            .as_ref()
            .expect("run state")
            .map_progression
            .available_node_ids
            .clone();
        assert!(
            available_node_ids.len() >= 2,
            "map fixture should expose at least two available nodes"
        );
        let first_node_id = available_node_ids[0];
        let second_node_id = available_node_ids[1];
        {
            let map = &mut core.state.run.as_mut().expect("run state").map;
            for (node_id, kind_id) in [
                (first_node_id, "support_first_preview"),
                (second_node_id, "support_second_preview"),
            ] {
                let node = map.node_mut(node_id).expect("node exists");
                node.category = MapNodeCategory::Support;
                node.kind_id = MapNodeKindId::new(kind_id);
                node.payload = MapNodePayload::Support {
                    support_type: SupportNodeType::Rest,
                    support_mode: SupportNodeMode::Known,
                    choices: vec![],
                };
            }
        }

        let first_preview = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: first_node_id,
                },
            )
            .unwrap();
        assert!(matches!(
            first_preview,
            BehaviorResult::NodePreview {
                node_id,
                ..
            } if node_id == first_node_id
        ));
        assert!(matches!(
            core.get_state(),
            GameState::NodeConfirm {
                node_id,
                ..
            } if node_id == first_node_id
        ));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::SelectMapNode));

        let second_preview = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: second_node_id,
                },
            )
            .unwrap();
        assert!(matches!(
            second_preview,
            BehaviorResult::NodePreview {
                node_id,
                ..
            } if node_id == second_node_id
        ));
        assert!(matches!(
            core.get_state(),
            GameState::NodeConfirm {
                node_id,
                ..
            } if node_id == second_node_id
        ));
        assert_eq!(
            core.state
                .node_session
                .as_ref()
                .expect("preview session should be staged")
                .node_id,
            second_node_id
        );

        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        match entered {
            BehaviorResult::NodeEntered { node_id, .. }
            | BehaviorResult::SupportState { node_id, .. }
            | BehaviorResult::HeadquartersContactState { node_id, .. } => {
                assert_eq!(node_id, second_node_id);
            }
            other => panic!("expected second node to be entered, got {other:?}"),
        }
    }

    #[test]
    fn combat_node_confirm_starts_live_battle() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(result, BehaviorResult::BattleAdvanced { .. }));
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        let allowed = core.get_allowed_actions();
        assert!(allowed.contains(&ActionKind::DeployUnit));
        assert!(allowed.contains(&ActionKind::WithdrawUnit));
    }

    #[test]
    fn safe_node_entry_delivers_pending_research_fragments() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let fragment_id = starter_basic_attack_fragment_id();
        let starting_count = core.state.skill_fragments.count(&fragment_id);
        core.state
            .skill_fragments
            .add_research_progress(&fragment_id, 100)
            .expect("research completion should be queued");
        assert_eq!(
            core.state
                .skill_fragments
                .pending_research_deliveries()
                .collect::<Vec<_>>(),
            vec![(&fragment_id, 1)]
        );

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_rest",
            MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: crate::game::map::SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let BehaviorResult::SupportState {
            research_deliveries,
            ..
        } = result
        else {
            panic!("expected support state");
        };
        assert_eq!(research_deliveries.len(), 1);
        assert_eq!(research_deliveries[0].fragment_id, fragment_id);
        assert_eq!(research_deliveries[0].count, 1);
        assert_eq!(research_deliveries[0].total_count, starting_count + 1);
        assert_eq!(
            core.state.skill_fragments.count(&fragment_id),
            starting_count + 1
        );
        assert!(core
            .state
            .skill_fragments
            .pending_research_deliveries()
            .next()
            .is_none());
    }

    #[test]
    fn map_progression_selects_completes_and_unlocks_next_nodes() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);

        let map = match core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap()
        {
            BehaviorResult::MapState { map } => map,
            other => panic!("expected map state, got {other:?}"),
        };
        assert!(!map.available_node_ids.is_empty());
        let first_node_id = map.available_node_ids[0];
        {
            let node = core
                .state
                .run
                .as_mut()
                .expect("run state")
                .map
                .node_mut(first_node_id)
                .expect("node exists");
            node.category = MapNodeCategory::Support;
            node.kind_id = crate::game::map::MapNodeKindId::new("support_rest");
            node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };
        }

        let preview = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: first_node_id,
                },
            )
            .unwrap();
        assert!(matches!(
            preview,
            BehaviorResult::NodePreview {
                node_id,
                ..
            } if node_id == first_node_id
        ));
        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        match entered {
            BehaviorResult::NodeEntered { node_id, .. }
            | BehaviorResult::SupportState { node_id, .. }
            | BehaviorResult::HeadquartersContactState { node_id, .. } => {
                assert_eq!(node_id, first_node_id);
            }
            BehaviorResult::ShopState { .. } | BehaviorResult::RewardState { .. } => {}
            other => panic!("expected entered map node content, got {other:?}"),
        }
        let session = core
            .state
            .node_session
            .as_ref()
            .expect("node session is staged after node entry");
        assert_eq!(session.node_id, first_node_id);
        assert!(matches!(core.get_state(), GameState::InNode { .. }));

        let completed = core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();
        let map = match completed {
            BehaviorResult::NodeCompleted { map, .. } => map,
            other => panic!("expected node completed, got {other:?}"),
        };
        assert!(map.completed_node_ids.contains(&first_node_id));
        assert!(!map.available_node_ids.contains(&first_node_id));
        assert!(!map.available_node_ids.is_empty());
        assert!(core.state.node_session.is_none());
        assert!(matches!(core.get_state(), GameState::ViewingMap));

        let err = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: first_node_id,
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn generated_map_assigns_pve_encounters_to_combat_and_boss_nodes() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);

        let map = match core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap()
        {
            BehaviorResult::MapState { map } => map,
            other => panic!("expected map state, got {other:?}"),
        };

        let boss = map
            .nodes
            .iter()
            .find(|node| node.id == map.boss_node_id)
            .expect("boss node exists");
        assert!(matches!(
            &boss.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if id == "boss_risk_encounter"
        ));

        assert!(map.nodes.iter().any(|node| {
            node.category == MapNodeCategory::Combat
                && matches!(
                    &node.payload,
                    MapNodePayload::Encounter {
                        encounter_id: Some(_)
                    }
                )
        }));
    }

    #[test]
    fn map_encounter_assignment_respects_node_kind_and_boss_purpose() {
        let core = GameCore::new(game_data_with_pve_encounters(), 123);
        let normal_node_id = MapNodeId::new(Uuid::from_u128(0xA001));
        let elite_node_id = MapNodeId::new(Uuid::from_u128(0xA002));
        let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA003));
        let mut map = RunMap {
            nodes: vec![
                MapNode {
                    id: normal_node_id,
                    depth: 0,
                    lane: 0,
                    kind_id: MapNodeKindId::new("combat_monster"),
                    category: MapNodeCategory::Combat,
                    state: MapNodeState::Available,
                    outgoing: vec![elite_node_id],
                    payload: MapNodePayload::Encounter { encounter_id: None },
                },
                MapNode {
                    id: elite_node_id,
                    depth: 3,
                    lane: 0,
                    kind_id: MapNodeKindId::new("combat_elite"),
                    category: MapNodeCategory::Combat,
                    state: MapNodeState::Hidden,
                    outgoing: vec![boss_node_id],
                    payload: MapNodePayload::Encounter { encounter_id: None },
                },
                MapNode {
                    id: boss_node_id,
                    depth: 7,
                    lane: 0,
                    kind_id: MapNodeKindId::new("boss_abnormality"),
                    category: MapNodeCategory::Boss,
                    state: MapNodeState::Hidden,
                    outgoing: vec![],
                    payload: MapNodePayload::Encounter { encounter_id: None },
                },
            ],
            start_node_ids: vec![normal_node_id],
            boss_node_id,
        };
        let run_progression = RunProgression::new(123, 3);

        super::map_encounters::assign_map_encounters(
            &core.game_data.pve_data,
            &mut map,
            &run_progression,
        );

        let normal = map.node(normal_node_id).expect("normal node exists");
        assert!(matches!(
            &normal.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
                encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
                    && encounter.node_type != Some(crate::game::combat_preview::CombatNodeType::Boss)
            })
        ));
        let elite = map.node(elite_node_id).expect("elite node exists");
        assert!(matches!(
            &elite.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
                encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
                    && encounter.node_type != Some(crate::game::combat_preview::CombatNodeType::Boss)
            })
        ));
        let boss = map.node(boss_node_id).expect("boss node exists");
        assert!(matches!(
            &boss.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if id == "boss_risk_encounter"
        ));
    }

    #[test]
    fn boss_completion_advances_acts_until_run_complete() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);

        let mut act_complete_count = 0_u8;
        loop {
            let map = match core
                .execute(player_id, PlayerBehavior::RequestMapData)
                .unwrap()
            {
                BehaviorResult::MapState { map } => map,
                other => panic!("expected map state, got {other:?}"),
            };
            let next_node = map
                .available_node_ids
                .first()
                .copied()
                .expect("available node");
            {
                let run = core.state.run.as_mut().expect("run state");
                let node = run.map.node_mut(next_node).expect("node exists");
                node.category = MapNodeCategory::Support;
                node.kind_id = crate::game::map::MapNodeKindId::new("support_rest");
                node.payload = MapNodePayload::Support {
                    support_type: SupportNodeType::Rest,
                    support_mode: SupportNodeMode::Known,
                    choices: vec![],
                };
            }
            core.execute(
                player_id,
                PlayerBehavior::SelectMapNode { node_id: next_node },
            )
            .unwrap();
            core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
                .unwrap();

            match core
                .execute(player_id, PlayerBehavior::CompleteNode)
                .unwrap()
            {
                BehaviorResult::NodeCompleted { .. } => {}
                BehaviorResult::ActComplete { act_index, map } => {
                    act_complete_count = act_complete_count.saturating_add(1);
                    assert_eq!(act_index, act_complete_count);
                    assert_eq!(map.act_index, act_index);
                    assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.setup.default_max_acts);
                    assert!(matches!(core.get_state(), GameState::ViewingMap));
                }
                BehaviorResult::RunComplete { map } => {
                    assert_eq!(
                        act_complete_count,
                        RUN_SYSTEM_POLICY.setup.default_max_acts - 1
                    );
                    assert_eq!(map.act_index, RUN_SYSTEM_POLICY.setup.default_max_acts - 1);
                    assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.setup.default_max_acts);
                    assert!(matches!(core.get_state(), GameState::RunComplete));
                    break;
                }
                other => panic!("unexpected map progression result: {other:?}"),
            }
        }
    }
}

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
    let available_node_ids = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map_progression
        .available_node_ids
        .clone();
    let map = &mut core.state.run.as_mut().expect("run state").map;
    for node_id in available_node_ids {
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

fn write_world_debug_event_log_export(name: &str, timeline: &Timeline) -> std::path::PathBuf {
    let out_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("debug_event_log_exports");
    let out_path = out_dir.join(format!("{name}.json"));
    std::fs::create_dir_all(&out_dir).expect("create debug_event_log_exports directory");
    timeline
        .write_pretty_json(&out_path)
        .expect("write debug event log json");
    out_path
}

mod support {
    use super::*;

    #[test]
    fn medical_support_node_emergency_care_restores_hp() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.health.set_current_hp(10);
        }
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_medical",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            core.execute(player_id, PlayerBehavior::CompleteNode)
                .unwrap_err(),
            GameError::InvalidAction
        ));
        core.execute(
            player_id,
            PlayerBehavior::SelectSupportTarget { employee_uuid },
        )
        .unwrap();
        assert!(matches!(
            core.execute(player_id, PlayerBehavior::CompleteNode)
                .unwrap_err(),
            GameError::InvalidAction
        ));
        core.execute(
            player_id,
            PlayerBehavior::SelectMedicalTreatment {
                treatment: MedicalTreatmentKind::EmergencyCare,
            },
        )
        .unwrap();
        core.execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee.health.current_hp, 60);
        assert_eq!(employee.trauma, 0);
    }

    #[test]
    fn medical_support_node_counseling_restores_trauma_only() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.health.set_current_hp(40);
            employee.trauma = 80;
        }
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_medical",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::SelectSupportTarget { employee_uuid },
        )
        .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::SelectMedicalTreatment {
                treatment: MedicalTreatmentKind::Counseling,
            },
        )
        .unwrap();
        core.execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee.health.current_hp, 40);
        assert_eq!(employee.trauma, 40);
    }

    #[test]
    fn medical_support_node_balanced_care_restores_hp_and_trauma() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.health.set_current_hp(40);
            employee.trauma = 80;
        }
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_medical",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::SelectSupportTarget { employee_uuid },
        )
        .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::SelectMedicalTreatment {
                treatment: MedicalTreatmentKind::BalancedCare,
            },
        )
        .unwrap();
        core.execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee.health.current_hp, 65);
        assert_eq!(employee.trauma, 60);
    }

    #[test]
    fn medical_support_node_with_no_target_wastes_node() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_medical",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::SupportState {
                target_candidates,
                ..
            } if target_candidates.is_empty()
        ));
        let result = core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(core.state.active_node_content.is_none());
    }

    #[test]
    fn rest_support_node_restores_all_living_employee_trauma() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        {
            let mut ids = core.roster().unwrap().available_employee_ids();
            ids.sort();
            let roster = core.roster_mut().unwrap();
            roster.get_mut(&ids[0]).unwrap().trauma = 30;
            roster.get_mut(&ids[1]).unwrap().trauma = 5;
            roster.get_mut(&ids[2]).unwrap().life_state = EmployeeLifeState::Dead;
            roster.get_mut(&ids[2]).unwrap().trauma = 50;
        }
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_rest",
            MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        let mut ids = core
            .roster()
            .unwrap()
            .iter()
            .map(|e| e.uuid)
            .collect::<Vec<_>>();
        ids.sort();
        let roster = core.roster().unwrap();
        assert_eq!(roster.get(&ids[0]).unwrap().trauma, 20);
        assert_eq!(roster.get(&ids[1]).unwrap().trauma, 0);
        assert_eq!(roster.get(&ids[2]).unwrap().trauma, 50);
    }

    #[test]
    fn limited_choice_support_node_requires_choice_before_completion() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_choice",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::LimitedChoice,
                choices: vec![SupportNodeType::Medical, SupportNodeType::Rest],
            },
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);
        assert!(matches!(
            result,
            BehaviorResult::SupportState {
                support_mode: SupportNodeMode::LimitedChoice,
                support_type: None,
                ..
            }
        ));
        assert!(matches!(
            core.execute(player_id, PlayerBehavior::CompleteNode)
                .unwrap_err(),
            GameError::InvalidAction
        ));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::ChooseSupport {
                    support_type: SupportNodeType::Rest,
                },
            )
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::SupportState {
                selected_support_type: Some(SupportNodeType::Rest),
                ..
            }
        ));
        let result = core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(core.state.active_node_content.is_none());
    }

    #[test]
    fn support_choice_does_not_expose_maintenance_actions() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_choice",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::LimitedChoice,
                choices: vec![SupportNodeType::Medical, SupportNodeType::Rest],
            },
        );

        select_and_confirm_map_node(&mut core, player_id, node_id);
        core.execute(
            player_id,
            PlayerBehavior::ChooseSupport {
                support_type: SupportNodeType::Rest,
            },
        )
        .unwrap();
        assert!(!core
            .get_allowed_actions()
            .contains(&ActionKind::DismantleSkillFragment));
    }

    #[test]
    fn independent_maintenance_node_exposes_maintenance_actions() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);

        assert!(matches!(
            result,
            BehaviorResult::MaintenanceState {
                maintenance_options,
                ..
            } if !maintenance_options.materials.is_empty()
        ));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::DismantleSkillFragment));
    }

    #[test]
    fn full_choice_support_node_exposes_all_active_support_effects() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_full_choice",
            MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::FullChoice,
                choices: vec![],
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::SupportState {
                support_mode: SupportNodeMode::FullChoice,
                choices,
                ..
            } if choices == vec![
                SupportNodeType::Medical,
                SupportNodeType::Rest,
            ]
        ));
    }
}

mod node_sessions {
    use super::*;

    fn headquarters_payload() -> MapNodePayload {
        MapNodePayload::HeadquartersContact {
            shop_pool_id: Some("headquarters_basic_supplies".to_string()),
            candidate_count: 3,
        }
    }

    #[test]
    fn headquarters_contact_enters_safe_choice_state_with_recruitment_candidates() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::HeadquartersContact,
            "headquarters_contact",
            headquarters_payload(),
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);

        assert!(matches!(
            result,
            BehaviorResult::HeadquartersContactState {
                options,
                recruitment_candidates,
                shop_pool_id: Some(pool_id),
                ..
            } if options.contains(&crate::game::map::HeadquartersContactOption::RecruitEmployee)
                && options.contains(&crate::game::map::HeadquartersContactOption::RequestEmergencySupplies)
                && options.contains(&crate::game::map::HeadquartersContactOption::OpenHeadquartersShop)
                && recruitment_candidates.len() == 3
                && pool_id == "headquarters_basic_supplies"
        ));
        assert!(matches!(core.get_state(), GameState::InNode { .. }));
        assert!(core.state.node_session.is_some());
        assert!(core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .is_err());
    }

    #[test]
    fn headquarters_recruit_employee_completes_node_and_adds_roster_member() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let initial_roster_size = core.roster().unwrap().len();
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::HeadquartersContact,
            "headquarters_contact",
            headquarters_payload(),
        );
        let result = select_and_confirm_map_node(&mut core, player_id, node_id);
        let BehaviorResult::HeadquartersContactState {
            recruitment_candidates,
            ..
        } = result
        else {
            panic!("headquarters node should expose recruitment candidates");
        };
        let candidate_id = recruitment_candidates[0].id.clone();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::RecruitEmployee {
                    candidate_id: candidate_id.clone(),
                },
            )
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::EmployeeRecruited {
                candidate_id: id,
                completion,
                ..
            } if id == candidate_id
                && matches!(*completion, BehaviorResult::NodeCompleted { .. })
        ));
        assert_eq!(core.roster().unwrap().len(), initial_roster_size + 1);
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
    }

    #[test]
    fn headquarters_shop_uses_headquarters_pool_and_exit_completes_node() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::HeadquartersContact,
            "headquarters_contact",
            headquarters_payload(),
        );
        select_and_confirm_map_node(&mut core, player_id, node_id);

        let result = core
            .execute(player_id, PlayerBehavior::OpenHeadquartersShop)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::ShopState { shop, .. } if shop.id == "hq_supply_shop"
        ));
        assert!(matches!(core.get_state(), GameState::InShop { .. }));

        let result = core.execute(player_id, PlayerBehavior::ExitShop).unwrap();
        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
    }

    #[test]
    fn headquarters_emergency_supplies_completes_node_without_shop_or_recruitment() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let initial_enkephalin = core.get_enkephalin();
        let initial_roster_size = core.roster().unwrap().len();
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::HeadquartersContact,
            "headquarters_contact",
            headquarters_payload(),
        );
        select_and_confirm_map_node(&mut core, player_id, node_id);

        let result = core
            .execute(player_id, PlayerBehavior::RequestEmergencySupplies)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::EmergencySuppliesGranted {
                enkephalin,
                completion,
                ..
            } if enkephalin == initial_enkephalin + RUN_SYSTEM_POLICY.headquarters.emergency_enkephalin
                && matches!(*completion, BehaviorResult::NodeCompleted { .. })
        ));
        assert_eq!(core.roster().unwrap().len(), initial_roster_size);
        assert!(matches!(core.get_state(), GameState::ViewingMap));
    }

    #[test]
    fn shop_map_node_enters_shop_and_exit_completes_node() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Shop,
            "shop_general",
            MapNodePayload::Shop {
                shop_id: Some("map_shop".to_string()),
                shop_pool_id: None,
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::ShopState { shop, .. } if shop.id == "map_shop"
        ));
        assert!(matches!(core.get_state(), GameState::InShop { .. }));
        assert!(core.state.node_session.is_some());

        let result = core.execute(player_id, PlayerBehavior::ExitShop).unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
    }

    #[test]
    fn reward_map_node_enters_claimable_reward_and_exit_completes_node() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Reward,
            "reward_treasure",
            MapNodePayload::Reward {
                reward_pool_id: Some("default_treasures".to_string()),
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(result, BehaviorResult::RewardState { .. }));
        assert!(matches!(core.get_state(), GameState::InReward { .. }));

        let result = core
            .execute(player_id, PlayerBehavior::ClaimReward)
            .unwrap();
        assert!(matches!(result, BehaviorResult::RewardGranted { .. }));
        assert!(matches!(
            core.get_state(),
            GameState::InRewardClaimed { .. }
        ));

        let result = core.execute(player_id, PlayerBehavior::ExitReward).unwrap();
        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
    }

    #[test]
    fn reward_map_node_uses_declared_pool_and_avoids_forbidden_rewards() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Reward,
            "reward_treasure",
            MapNodePayload::Reward {
                reward_pool_id: Some("default_treasures".to_string()),
            },
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::RewardState { rewards, .. }
                if rewards.len() == 1
                    && rewards[0].id == "map_reward"
                    && !rewards[0].tags.contains(&RewardTag::Forbidden)
        ));
    }
}

mod equipment {
    use super::*;

    #[test]
    fn use_consumable_item_is_safezone_action_and_exposes_active_modifier_snapshot() {
        let consumable = consumable_meta(
            0xC001,
            "stabilizing_ampoule",
            ConsumableTier::Common,
            ConsumableEffect::TraumaMitigation { percent: 25 },
        );
        let owned_uuid = Uuid::from_u128(0xC0FFEE);
        let game_data = game_data_with_consumables(vec![consumable.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        core.inventory_mut()
            .unwrap()
            .consumables
            .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
            .unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_rest",
            MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::UseConsumableItem));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::UseConsumableItem {
                    item_uuid: owned_uuid,
                    target_employee_uuid: employee_uuid,
                },
            )
            .unwrap();

        let BehaviorResult::ConsumableItemUsed {
            item_uuid,
            target_employee_uuid,
            replaced_modifier,
            applied_modifier,
            inventory_diff,
        } = result
        else {
            panic!("expected consumable use result");
        };
        assert_eq!(item_uuid, owned_uuid);
        assert_eq!(target_employee_uuid, employee_uuid);
        assert!(replaced_modifier.is_none());
        assert_eq!(applied_modifier.definition_id, "stabilizing_ampoule");
        assert_eq!(inventory_diff.removed, vec![owned_uuid]);
        assert!(core
            .inventory()
            .unwrap()
            .consumables
            .get_item(&owned_uuid)
            .is_none());

        let snapshot = core.get_run_snapshot_json().unwrap();
        assert!(snapshot["inventory"]["consumables"]
            .as_array()
            .unwrap()
            .is_empty());
        let employees = snapshot["roster"]["employees"].as_array().unwrap();
        let employee = employees
            .iter()
            .find(|value| value["uuid"] == json!(employee_uuid))
            .unwrap();
        assert_eq!(
            employee["active_consumable_modifier"]["definition_id"],
            "stabilizing_ampoule"
        );
        assert_eq!(
            employee["active_consumable_modifier"]["remaining_combat_nodes"],
            1
        );
    }

    #[test]
    fn use_consumable_item_allows_alive_but_combat_unavailable_target() {
        let consumable = consumable_meta(
            0xC020,
            "field_tonic",
            ConsumableTier::Common,
            ConsumableEffect::TraumaMitigation { percent: 15 },
        );
        let owned_uuid = Uuid::from_u128(0xC020_0001);
        let game_data = game_data_with_consumables(vec![consumable.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.availability = EmployeeAvailability::Unavailable;
        }
        core.inventory_mut()
            .unwrap()
            .consumables
            .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
            .unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_rest",
            MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();

        let result = core.execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: owned_uuid,
                target_employee_uuid: employee_uuid,
            },
        );

        assert!(matches!(
            result,
            Ok(BehaviorResult::ConsumableItemUsed { .. })
        ));
        assert_eq!(
            core.roster()
                .unwrap()
                .get(&employee_uuid)
                .unwrap()
                .active_consumable_modifier
                .as_ref()
                .unwrap()
                .definition_id,
            "field_tonic"
        );
    }

    #[test]
    fn use_consumable_item_rejects_dead_target() {
        let consumable = consumable_meta(
            0xC021,
            "dead_target_tonic",
            ConsumableTier::Common,
            ConsumableEffect::TraumaMitigation { percent: 15 },
        );
        let owned_uuid = Uuid::from_u128(0xC021_0001);
        let game_data = game_data_with_consumables(vec![consumable.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.life_state = EmployeeLifeState::Dead;
        }
        core.inventory_mut()
            .unwrap()
            .consumables
            .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
            .unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_rest",
            MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();

        let result = core.execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: owned_uuid,
                target_employee_uuid: employee_uuid,
            },
        );

        assert!(matches!(result, Err(GameError::InvalidAction)));
        assert!(core
            .inventory()
            .unwrap()
            .consumables
            .get_item(&owned_uuid)
            .is_some());
    }

    #[test]
    fn use_consumable_item_replaces_existing_modifier_without_refund() {
        let first = consumable_meta(
            0xC010,
            "first_ampoule",
            ConsumableTier::Common,
            ConsumableEffect::TraumaMitigation { percent: 10 },
        );
        let second = consumable_meta(
            0xC011,
            "second_ampoule",
            ConsumableTier::Uncommon,
            ConsumableEffect::BattleHpSetup { bonus_percent: 20 },
        );
        let first_owned = Uuid::from_u128(0xC010_0001);
        let second_owned = Uuid::from_u128(0xC011_0001);
        let game_data = game_data_with_consumables(vec![first.clone(), second.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        {
            let inventory = core.inventory_mut().unwrap();
            inventory
                .consumables
                .add_item(OwnedConsumable::new(first_owned, Arc::new(first)))
                .unwrap();
            inventory
                .consumables
                .add_item(OwnedConsumable::new(second_owned, Arc::new(second)))
                .unwrap();
        }

        core.execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: first_owned,
                target_employee_uuid: employee_uuid,
            },
        )
        .unwrap();
        let result = core
            .execute(
                player_id,
                PlayerBehavior::UseConsumableItem {
                    item_uuid: second_owned,
                    target_employee_uuid: employee_uuid,
                },
            )
            .unwrap();

        let BehaviorResult::ConsumableItemUsed {
            replaced_modifier,
            inventory_diff,
            ..
        } = result
        else {
            panic!("expected consumable use result");
        };
        assert_eq!(replaced_modifier.unwrap().definition_id, "first_ampoule");
        assert_eq!(inventory_diff.removed, vec![second_owned]);
        assert!(core
            .inventory()
            .unwrap()
            .consumables
            .get_item(&first_owned)
            .is_none());
        assert!(core
            .inventory()
            .unwrap()
            .consumables
            .get_item(&second_owned)
            .is_none());
    }

    #[test]
    fn equip_item_targets_employee_loadout_after_roster_initialization() {
        let unit_meta = abnormality_meta(1);
        let weapon = equipment_meta(10, "employee_weapon", EquipmentType::Weapon);
        let weapon_owned_uuid = Uuid::from_u128(200);
        let game_data =
            game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");

        {
            let inventory = core.inventory_mut().unwrap();
            inventory
                .equipments
                .add_item(OwnedEquipment::new(
                    weapon_owned_uuid,
                    Arc::new(weapon.clone()),
                ))
                .unwrap();
        }

        let result = core
            .handle_equip_item(weapon_owned_uuid, employee_uuid)
            .unwrap();

        let BehaviorResult::EquipItem { result } = result else {
            panic!("expected equip item result");
        };
        assert_eq!(result.target_unit, employee_uuid);
        assert_eq!(result.equipped_items.len(), 1);
        assert_eq!(result.equipped_items[0].base_uuid, weapon.uuid);

        let roster = core.roster().unwrap();
        let employee = roster.get(&employee_uuid).unwrap();
        let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
        assert_eq!(equipped.len(), 1);
        assert_eq!(equipped[0].instance_uuid, weapon_owned_uuid);

        let inventory = core.inventory().unwrap();
        let equipped_item = inventory.equipments.get_item(&weapon_owned_uuid).unwrap();
        assert_eq!(equipped_item.equipped_to, Some(employee_uuid));

        let snapshot = core.get_employee_roster_snapshot_json().unwrap();
        let employees = snapshot["employees"].as_array().unwrap();
        let employee_snapshot = employees
            .iter()
            .find(|employee| employee["uuid"] == json!(employee_uuid))
            .expect("equipped employee is exposed in roster snapshot");
        assert_eq!(
            employee_snapshot["combat_profile"]["effective_weapon_profile"]["weapon_archetype"],
            "Sword"
        );
    }

    #[test]
    fn unequip_item_allows_unbound_equipment_in_safezone() {
        let unit_meta = abnormality_meta(1);
        let weapon = equipment_meta(10, "employee_weapon", EquipmentType::Weapon);
        let weapon_owned_uuid = Uuid::from_u128(200);
        let game_data =
            game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(OwnedEquipment::new(
                weapon_owned_uuid,
                Arc::new(weapon.clone()),
            ))
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipItem {
                item_uuid: weapon_owned_uuid,
                target_unit: employee_uuid,
            },
        )
        .unwrap();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::UnEquipItem {
                    item_uuid: weapon_owned_uuid,
                    target_unit: employee_uuid,
                },
            )
            .unwrap();

        let BehaviorResult::UnEquipItem { result } = result else {
            panic!("expected unequip item result");
        };
        assert_eq!(result.item_uuid, weapon_owned_uuid);
        assert!(result.equipped_items.is_empty());
        let inventory = core.inventory().unwrap();
        let equipment = inventory.equipments.get_item(&weapon_owned_uuid).unwrap();
        assert_eq!(equipment.equipped_to, None);
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert!(employee.loadout.item_slot.iter().next().is_none());
    }

    #[test]
    fn unequip_item_rejects_bound_equipment_and_snapshot_exposes_reason() {
        let unit_meta = abnormality_meta(1);
        let mut weapon = equipment_meta(10, "bound_weapon", EquipmentType::Weapon);
        weapon.bound = true;
        weapon.cannot_unequip_reason = "story_bound".to_string();
        let weapon_owned_uuid = Uuid::from_u128(201);
        let game_data =
            game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(OwnedEquipment::new(
                weapon_owned_uuid,
                Arc::new(weapon.clone()),
            ))
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipItem {
                item_uuid: weapon_owned_uuid,
                target_unit: employee_uuid,
            },
        )
        .unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::UnEquipItem {
                    item_uuid: weapon_owned_uuid,
                    target_unit: employee_uuid,
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(
            snapshot["inventory"]["equipments"][0]["item"]["can_unequip"],
            false
        );
        assert_eq!(
            snapshot["inventory"]["equipments"][0]["item"]["cannot_unequip_reason"],
            "story_bound"
        );
        let employees = snapshot["roster"]["employees"].as_array().unwrap();
        let employee = employees
            .iter()
            .find(|value| value["uuid"] == json!(employee_uuid))
            .unwrap();
        assert_eq!(employee["equipped_items"][0]["can_unequip"], false);
        assert_eq!(
            employee["equipped_items"][0]["cannot_unequip_reason"],
            "story_bound"
        );
    }

    #[test]
    fn equip_item_combines_equipped_component_before_slot_rejection() {
        let unit_meta = abnormality_meta(1);
        let component_a = equipment_meta(10, "weapon_component_a", EquipmentType::Weapon);
        let component_b = equipment_meta(11, "weapon_component_b", EquipmentType::Weapon);
        let completed = equipment_meta(12, "completed_weapon", EquipmentType::Weapon);
        let game_data = game_data_with_equipment(
            Arc::clone(&unit_meta),
            vec![component_a.clone(), component_b.clone(), completed.clone()],
            vec![EquipmentRecipeMetadata {
                ingredients: vec![component_a.uuid, component_b.uuid],
                result: completed.uuid,
            }],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        let component_a_owned_uuid = Uuid::from_u128(200);
        let component_b_owned_uuid = Uuid::from_u128(201);

        {
            let inventory = core.inventory_mut().unwrap();
            inventory
                .equipments
                .add_item(OwnedEquipment::new(
                    component_a_owned_uuid,
                    Arc::new(component_a),
                ))
                .unwrap();
            inventory
                .equipments
                .add_item(OwnedEquipment::new(
                    component_b_owned_uuid,
                    Arc::new(component_b),
                ))
                .unwrap();
        }

        core.handle_equip_item(component_a_owned_uuid, employee_uuid)
            .unwrap();
        let result = core
            .handle_equip_item(component_b_owned_uuid, employee_uuid)
            .unwrap();

        let BehaviorResult::EquipItem { result } = result else {
            panic!("expected equip item result");
        };
        assert_eq!(result.requested_item_uuid, component_b_owned_uuid);
        assert_eq!(result.target_unit, employee_uuid);
        assert_eq!(result.inventory_diff.removed.len(), 2);
        assert!(result
            .inventory_diff
            .removed
            .contains(&component_a_owned_uuid));
        assert!(result
            .inventory_diff
            .removed
            .contains(&component_b_owned_uuid));
        assert_eq!(result.inventory_diff.added.len(), 1);
        assert!(matches!(
            result.outcome,
            EquipItemOutcomeDto::Combined {
                result_base_uuid,
                ..
            } if result_base_uuid == completed.uuid
        ));
        assert_eq!(result.equipped_items.len(), 1);
        assert_eq!(result.equipped_items[0].base_uuid, completed.uuid);

        let inventory = core.inventory().unwrap();
        assert!(inventory
            .equipments
            .get_item(&component_a_owned_uuid)
            .is_none());
        assert!(inventory
            .equipments
            .get_item(&component_b_owned_uuid)
            .is_none());

        let roster = core.roster().unwrap();
        let employee = roster.get(&employee_uuid).unwrap();
        let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
        assert_eq!(equipped.len(), 1);
        assert_eq!(equipped[0].base_uuid, completed.uuid);

        let completed_item = inventory
            .equipments
            .iter()
            .find(|item| item.meta.uuid == completed.uuid)
            .expect("completed item should be created");
        assert_eq!(completed_item.equipped_to, Some(employee_uuid));
    }

    #[test]
    fn dismantle_equipment_is_maintenance_only_and_grants_material_stacks() {
        let unit_meta = abnormality_meta(1);
        let equipment = equipment_meta(40, "dismantle_weapon", EquipmentType::Weapon);
        let material = EquipmentMaterialMetadata {
            id: "dismantled_weapon_fragment".to_string(),
            uuid: Uuid::from_u128(41),
            name: "Dismantled Weapon Fragment".to_string(),
            description: "A test dismantle material".to_string(),
            material_type: EquipmentMaterialType::Fragment,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            equipment_type: Some(EquipmentType::Weapon),
        };
        let recipe = EquipmentDismantleRecipeMetadata {
            equipment_id: equipment.id.clone(),
            yields: vec![EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 2,
            }],
        };
        let game_data = game_data_with_equipment_dismantle(
            Arc::clone(&unit_meta),
            vec![equipment.clone()],
            vec![material.clone()],
            vec![recipe],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let owned_uuid = Uuid::from_u128(401);
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
            .unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::DismantleEquipment {
                    item_uuid: owned_uuid,
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::DismantleEquipment));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::DismantleEquipment {
                    item_uuid: owned_uuid,
                },
            )
            .unwrap();

        let BehaviorResult::EquipmentDismantled {
            item_uuid,
            equipment_id,
            inventory_diff,
        } = result
        else {
            panic!("expected equipment dismantle result");
        };
        assert_eq!(item_uuid, owned_uuid);
        assert_eq!(equipment_id, equipment.id);
        assert!(inventory_diff.added.is_empty());
        assert_eq!(inventory_diff.removed, vec![owned_uuid]);
        assert_eq!(inventory_diff.material_stacks.len(), 1);
        assert_eq!(inventory_diff.material_stacks[0].material_id, material.id);
        assert_eq!(inventory_diff.material_stacks[0].amount, 2);
        assert!(core
            .inventory()
            .unwrap()
            .equipments
            .get_item(&owned_uuid)
            .is_none());
        assert_eq!(
            core.inventory()
                .unwrap()
                .equipment_materials
                .amount(&material.id),
            2
        );
    }

    #[test]
    fn dismantle_equipment_automatically_unequips_equipped_item() {
        let unit_meta = abnormality_meta(1);
        let equipment = equipment_meta(45, "equipped_dismantle_weapon", EquipmentType::Weapon);
        let material = EquipmentMaterialMetadata {
            id: "equipment_dust".to_string(),
            uuid: Uuid::from_u128(46),
            name: "Equipment Dust".to_string(),
            description: "A test equipment dust material".to_string(),
            material_type: EquipmentMaterialType::Generic,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            equipment_type: None,
        };
        let recipe = EquipmentDismantleRecipeMetadata {
            equipment_id: equipment.id.clone(),
            yields: vec![EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 1,
            }],
        };
        let game_data = game_data_with_equipment_dismantle(
            Arc::clone(&unit_meta),
            vec![equipment.clone()],
            vec![material.clone()],
            vec![recipe],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let owned_uuid = Uuid::from_u128(451);
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipItem {
                item_uuid: owned_uuid,
                target_unit: employee_uuid,
            },
        )
        .unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let BehaviorResult::MaintenanceState {
            maintenance_options,
            ..
        } = entered
        else {
            panic!("expected maintenance state with options");
        };
        let preview = maintenance_options
            .items
            .iter()
            .find(|item| item.target_id == owned_uuid.to_string())
            .expect("equipped equipment should appear in maintenance preview");
        assert!(preview.operations.dismantle.can_execute);
        assert!(preview.operations.dismantle.will_unequip);

        core.execute(
            player_id,
            PlayerBehavior::DismantleEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap();

        assert!(core
            .inventory()
            .unwrap()
            .equipments
            .get_item(&owned_uuid)
            .is_none());
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert!(!employee.loadout.item_slot.contains_instance(owned_uuid));
        assert_eq!(
            core.inventory()
                .unwrap()
                .equipment_materials
                .amount(&material.id),
            1
        );
    }

    #[test]
    fn enhance_equipment_is_maintenance_only_and_updates_owned_instance() {
        let unit_meta = abnormality_meta(1);
        let equipment = equipment_meta(50, "enhance_weapon", EquipmentType::Weapon);
        let material = EquipmentMaterialMetadata {
            id: "enhance_weapon_fragment".to_string(),
            uuid: Uuid::from_u128(51),
            name: "Enhance Weapon Fragment".to_string(),
            description: "A test enhancement material".to_string(),
            material_type: EquipmentMaterialType::Fragment,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            equipment_type: Some(EquipmentType::Weapon),
        };
        let recipe = EquipmentEnhancementRecipeMetadata {
            equipment_id: equipment.id.clone(),
            max_level: 2,
            costs_per_level: vec![EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 2,
            }],
            modifiers_per_level: vec![StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 3,
            }],
        };
        let game_data = game_data_with_equipment_enhancement(
            Arc::clone(&unit_meta),
            vec![equipment.clone()],
            vec![material.clone()],
            vec![recipe],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let owned_uuid = Uuid::from_u128(501);
        {
            let inventory = core.inventory_mut().unwrap();
            inventory
                .equipments
                .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
                .unwrap();
            inventory.equipment_materials.add(&material.id, 2).unwrap();
        }

        let err = core
            .execute(
                player_id,
                PlayerBehavior::EnhanceEquipment {
                    item_uuid: owned_uuid,
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let BehaviorResult::MaintenanceState {
            maintenance_options,
            ..
        } = entered
        else {
            panic!("expected maintenance state with options");
        };
        let maintenance_item = maintenance_options
            .items
            .iter()
            .find(|item| item.target_id == owned_uuid.to_string())
            .expect("equipment should be present in maintenance preview");
        assert!(maintenance_item.operations.enhance.can_execute);
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::EnhanceEquipment));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::EnhanceEquipment {
                    item_uuid: owned_uuid,
                },
            )
            .unwrap();

        let BehaviorResult::EquipmentEnhanced {
            item_uuid,
            equipment_id,
            enhancement_level,
            inventory_diff,
        } = result
        else {
            panic!("expected equipment enhanced result");
        };
        assert_eq!(item_uuid, owned_uuid);
        assert_eq!(equipment_id, equipment.id);
        assert_eq!(enhancement_level, 1);
        assert_eq!(inventory_diff.updated.len(), 1);
        assert_eq!(inventory_diff.material_stacks[0].amount, 0);
        let owned = core
            .inventory()
            .unwrap()
            .equipments
            .get_item(&owned_uuid)
            .unwrap();
        assert_eq!(owned.enhancement_level, 1);
    }

    #[test]
    fn skill_fragment_actions_equip_and_unequip_employee_loadout() {
        let fragment = active_skill_fragment("test_active_fragment", 0xF00D, "test_active_skill");
        let weapon = weapon_equipment(0xE001, "starter_test_sword", WeaponArchetype::Sword);
        let game_data = game_data_with_equipment_and_skill_fragments(
            vec![weapon.clone()],
            vec![fragment.clone()],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            weapon,
            Uuid::from_u128(0xE001_0001),
        );
        core.state.skill_fragments.add(&fragment).unwrap();

        let equip_result = core
            .execute(
                player_id,
                PlayerBehavior::EquipSkillFragment {
                    employee_uuid,
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();

        let BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid: updated_employee,
            equipped_fragment_ids,
        } = equip_result
        else {
            panic!("expected skill fragment loadout result");
        };
        assert_eq!(updated_employee, employee_uuid);
        assert!(equipped_fragment_ids.iter().any(|id| id == &fragment.id));

        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        let profile = employee
            .combat_profile_for_battle(
                &core.game_data.skill_fragment_data,
                &core.state.skill_fragments,
            )
            .unwrap();
        assert_eq!(profile.skill_id.as_deref(), Some("test_active_skill"));

        let unequip_result = core
            .execute(
                player_id,
                PlayerBehavior::UnequipSkillFragment {
                    employee_uuid,
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();
        let BehaviorResult::SkillFragmentLoadoutUpdated {
            equipped_fragment_ids,
            ..
        } = unequip_result
        else {
            panic!("expected skill fragment loadout result");
        };
        assert!(!equipped_fragment_ids.iter().any(|id| id == &fragment.id));
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        let profile = employee
            .combat_profile_for_battle(
                &core.game_data.skill_fragment_data,
                &core.state.skill_fragments,
            )
            .unwrap();
        assert_eq!(profile.skill_id, None);
    }

    #[test]
    fn skill_fragment_equip_rejects_unowned_fragment() {
        let fragment = active_skill_fragment("unowned_active_fragment", 0xF00E, "unowned_skill");
        let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");

        let err = core
            .execute(
                player_id,
                PlayerBehavior::EquipSkillFragment {
                    employee_uuid,
                    fragment_id: fragment.id,
                },
            )
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn skill_fragment_equip_rejects_incompatible_weapon_profile() {
        let mut fragment = active_skill_fragment("gun_locked_fragment", 0xF013, "gun_locked_skill");
        fragment.compatibility = SkillFragmentCompatibilityRequirements {
            allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
            ..SkillFragmentCompatibilityRequirements::default()
        };
        let sword = weapon_equipment(0xE002, "test_sword", WeaponArchetype::Sword);
        let game_data = game_data_with_equipment_and_skill_fragments(
            vec![sword.clone()],
            vec![fragment.clone()],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            sword,
            Uuid::from_u128(0xE002_0001),
        );
        core.state.skill_fragments.add(&fragment).unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::EquipSkillFragment {
                    employee_uuid,
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap_err();

        assert!(matches!(
            err,
            GameError::SkillFragmentIncompatible {
                fragment_id,
                failure_codes,
            } if fragment_id == fragment.id
                && failure_codes == vec![
                    SkillFragmentCompatibilityFailureCode::WeaponArchetypeMismatch
                ]
        ));

        let snapshot = core.get_employee_roster_snapshot_json().unwrap();
        let employee_snapshot = snapshot["employees"]
            .as_array()
            .unwrap()
            .iter()
            .find(|employee| employee["uuid"] == json!(employee_uuid))
            .unwrap();
        let compatibility = employee_snapshot["skill_fragments"]["compatibility"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == json!(fragment.id))
            .unwrap();
        assert_eq!(compatibility["is_compatible"], false);
        assert_eq!(
            compatibility["failure_codes"],
            json!(["weapon_archetype_mismatch"])
        );
    }

    #[test]
    fn skill_fragment_equip_accepts_matching_weapon_profile() {
        let mut fragment =
            active_skill_fragment("matching_gun_fragment", 0xF014, "matching_gun_skill");
        fragment.compatibility = SkillFragmentCompatibilityRequirements {
            allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
            allowed_range_roles: vec![WeaponRangeRole::Ranged],
            ..SkillFragmentCompatibilityRequirements::default()
        };
        let gun = weapon_equipment(0xE003, "test_gun", WeaponArchetype::Gun);
        let game_data =
            game_data_with_equipment_and_skill_fragments(vec![gun.clone()], vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(&mut core, employee_uuid, gun, Uuid::from_u128(0xE003_0001));
        core.state.skill_fragments.add(&fragment).unwrap();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::EquipSkillFragment {
                    employee_uuid,
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::SkillFragmentLoadoutUpdated { .. }
        ));
    }

    #[test]
    fn equipment_combination_rejects_result_that_invalidates_active_fragment() {
        let mut fragment =
            active_skill_fragment("gun_combo_locked_fragment", 0xF015, "gun_combo_skill");
        fragment.compatibility = SkillFragmentCompatibilityRequirements {
            allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
            ..SkillFragmentCompatibilityRequirements::default()
        };
        let gun_component = weapon_equipment(0xE008, "gun_component", WeaponArchetype::Gun);
        let catalyst = weapon_equipment(0xE009, "weapon_catalyst", WeaponArchetype::Gun);
        let sword_result = weapon_equipment(0xE00A, "sword_result", WeaponArchetype::Sword);
        let game_data = game_data_with_equipment_recipes_and_skill_fragments(
            vec![
                gun_component.clone(),
                catalyst.clone(),
                sword_result.clone(),
            ],
            vec![EquipmentRecipeMetadata {
                ingredients: vec![gun_component.uuid, catalyst.uuid],
                result: sword_result.uuid,
            }],
            vec![fragment.clone()],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            gun_component,
            Uuid::from_u128(0xE008_0001),
        );
        core.state.skill_fragments.add(&fragment).unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();
        let catalyst_owned_uuid = Uuid::from_u128(0xE009_0001);
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(OwnedEquipment::new(catalyst_owned_uuid, Arc::new(catalyst)))
            .unwrap();

        let err = core
            .handle_equip_item(catalyst_owned_uuid, employee_uuid)
            .unwrap_err();

        assert!(matches!(
            err,
            GameError::SkillFragmentIncompatible {
                fragment_id,
                failure_codes,
            } if fragment_id == fragment.id
                && failure_codes == vec![
                    SkillFragmentCompatibilityFailureCode::WeaponArchetypeMismatch
                ]
        ));
    }

    #[test]
    fn skill_fragment_upgrade_consumes_fragment_dust_and_updates_progress() {
        let target = active_skill_fragment("upgrade_active_fragment", 0xF00F, "upgrade_skill");
        let game_data = game_data_with_skill_fragments(vec![target.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.state.skill_fragments.add(&target).unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::UpgradeSkillFragment {
                    target_fragment_id: target.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::UpgradeSkillFragment));
        core.state.skill_fragments.add_fragment_dust(4).unwrap();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::UpgradeSkillFragment {
                    target_fragment_id: target.id.clone(),
                },
            )
            .unwrap();

        let BehaviorResult::SkillFragmentUpgraded {
            target_fragment_id,
            dust_spent,
            remaining_dust,
            progress,
        } = result
        else {
            panic!("expected skill fragment upgrade result");
        };
        assert_eq!(target_fragment_id, target.id);
        assert_eq!(dust_spent, 4);
        assert_eq!(remaining_dust, 0);
        assert_eq!(progress.upgrade_level, 1);
        assert_eq!(progress.awakening_progress, 1);
        assert_eq!(core.state.skill_fragments.count(&target.id), 1);
        assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
    }

    #[test]
    fn skill_fragment_dismantle_is_maintenance_only_and_grants_dust() {
        let fragment = active_skill_fragment("dismantle_fragment", 0xF011, "dismantle_skill");
        let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.state.skill_fragments.add(&fragment).unwrap();
        core.state.skill_fragments.add(&fragment).unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::DismantleSkillFragment));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();

        let BehaviorResult::SkillFragmentDismantled {
            fragment_id,
            remaining_count,
            dust_gained,
            total_dust,
        } = result
        else {
            panic!("expected skill fragment dismantle result");
        };
        assert_eq!(fragment_id, fragment.id);
        assert_eq!(remaining_count, 1);
        assert_eq!(dust_gained, 2);
        assert_eq!(total_dust, 2);
        assert_eq!(core.state.skill_fragments.count(&fragment.id), 1);
        assert_eq!(core.state.skill_fragments.fragment_dust(), 2);
    }

    #[test]
    fn skill_fragment_dismantle_unequips_equipped_fragment() {
        let fragment =
            active_skill_fragment("equipped_dismantle_fragment", 0xF012, "equipped_skill");
        let weapon = weapon_equipment(0xE004, "dismantle_test_sword", WeaponArchetype::Sword);
        let game_data = game_data_with_equipment_and_skill_fragments(
            vec![weapon.clone()],
            vec![fragment.clone()],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            weapon,
            Uuid::from_u128(0xE004_0001),
        );
        core.state.skill_fragments.add(&fragment).unwrap();
        core.state.skill_fragments.add(&fragment).unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::SkillFragmentDismantled {
                remaining_count: 1,
                ..
            }
        ));
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee.skill_fragments.active_fragment_id(), None);
        assert_eq!(core.state.skill_fragments.fragment_dust(), 2);
    }

    #[test]
    fn skill_fragment_dismantle_allows_last_copy() {
        let fragment =
            active_skill_fragment("last_copy_dismantle_fragment", 0xF013, "last_copy_skill");
        let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.state.skill_fragments.add(&fragment).unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Maintenance,
            "maintenance",
            MapNodePayload::Maintenance,
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let result = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::SkillFragmentDismantled {
                remaining_count: 0,
                ..
            }
        ));
        assert_eq!(core.state.skill_fragments.count(&fragment.id), 0);
    }
}

mod combat {
    use super::*;
    use crate::game::ability::SkillEffectDef;
    use crate::game::battle::types::{BattleWinner, ParticipantBattleResult};
    use crate::game::resources::{CombatBattleState, RunFailureReason};

    fn force_map_combat_node(
        core: &mut GameCore,
        category: MapNodeCategory,
        kind_id: &str,
        encounter_id: &str,
    ) -> MapNodeId {
        force_first_available_node(
            core,
            category,
            kind_id,
            MapNodePayload::Encounter {
                encounter_id: Some(encounter_id.to_string()),
            },
        )
    }

    fn first_ground_deployment_cell(preview: &BehaviorResult) -> Position {
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = preview
        else {
            panic!("expected combat preview");
        };
        combat_preview
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
            .and_then(|zone| zone.cells.first().copied())
            .expect("expected ground deployment cell")
    }

    #[test]
    fn combat_result_without_node_session_is_rejected() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let abnormality_uuid = Uuid::from_u128(0xBEEF);

        core.transition_to(GameState::CombatResult {
            battle_uuid: abnormality_uuid,
        })
        .unwrap();
        core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(CombatBattleState {
            abnormality_id: "abno".to_string(),
            encounter_id: "encounter".to_string(),
            node_type: crate::game::combat_preview::CombatNodeType::Defense,
            mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
            abnormality_uuid,
            winner: BattleWinner::Opponent,
            timeline: Timeline::default(),
            reward_mode: RewardMode::ChooseOne,
            rewards: vec![],
            participant_results: vec![],
        }));

        let err = core.handle_complete_combat_result().unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));

        let mut battle = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Player;
        core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));

        let err = core.handle_complete_combat_result().unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn post_battle_resolution_applies_employee_incapacitation_trauma() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");

        core.apply_post_battle_resolution(&[ParticipantBattleResult {
            unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                0xCAFE,
            )),
            owned_uuid: employee_uuid,
            side: Side::Player,
            survived: false,
            final_hp: 0,
            max_hp: 30,
            became_incapacitated: true,
        }])
        .unwrap();

        let roster = core.roster().unwrap();
        let employee = roster.get(&employee_uuid).unwrap();
        assert_eq!(
            employee.trauma,
            RUN_SYSTEM_POLICY.post_battle.incapacitation_trauma
        );
        assert_eq!(employee.injuries.len(), 1);
        assert_eq!(employee.injuries[0].id, "battle_incapacitation");
        assert_eq!(
            employee.life_state,
            crate::game::employee::EmployeeLifeState::Alive
        );
    }

    #[test]
    fn repeated_post_battle_incapacitation_can_kill_employee_and_remove_from_roster_order() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");

        let participant = ParticipantBattleResult {
            unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                0xCAFE,
            )),
            owned_uuid: employee_uuid,
            side: Side::Player,
            survived: false,
            final_hp: 0,
            max_hp: 30,
            became_incapacitated: true,
        };
        core.apply_post_battle_resolution(&[participant.clone(), participant.clone(), participant])
            .unwrap();

        let roster = core.roster().unwrap();
        let employee = roster.get(&employee_uuid).unwrap();
        assert_eq!(
            employee.life_state,
            crate::game::employee::EmployeeLifeState::Dead
        );
        assert!(!roster.available_employee_ids().contains(&employee_uuid));
        let roster_order = core.roster_order().unwrap();
        assert!(roster_order.slot_of(employee_uuid).is_none());
    }

    #[test]
    fn combat_map_node_fails_run_when_no_employee_can_be_deployed() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.availability = EmployeeAvailability::Unavailable;
            }
        }
        core.sync_roster_order_with_owned_units().unwrap();
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );
        force_other_available_support_nodes_to_rest(&mut core, node_id);

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::RunFailed {
                reason: RunFailureReason::NoDeployableEmployees,
                ..
            }
        ));
        assert!(matches!(
            core.get_state(),
            GameState::RunFailed {
                reason: RunFailureReason::NoDeployableEmployees
            }
        ));
        assert!(core.state.node_session.is_none());
        assert!(core.get_allowed_actions().is_empty());
    }

    #[test]
    fn combat_map_node_fails_run_when_all_living_employees_have_zero_hp() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.health.set_current_hp(0);
            }
        }
        core.sync_roster_order_with_owned_units().unwrap();
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );
        force_other_available_support_nodes_to_rest(&mut core, node_id);

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::RunFailed {
                reason: RunFailureReason::NoDeployableEmployees,
                ..
            }
        ));
    }

    #[test]
    fn combat_map_node_is_blocked_when_medical_support_node_is_still_available() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.health.set_current_hp(0);
            }
        }
        core.sync_roster_order_with_owned_units().unwrap();

        let available_nodes = core
            .state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .available_node_ids
            .clone();
        assert!(available_nodes.len() >= 2);
        let medical_node_id = available_nodes[0];
        let combat_node_id = available_nodes[1];
        {
            let map = &mut core.state.run.as_mut().unwrap().map;
            let medical_node = map.node_mut(medical_node_id).unwrap();
            medical_node.category = MapNodeCategory::Support;
            medical_node.kind_id = crate::game::map::MapNodeKindId::new("support_medical");
            medical_node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };

            let combat_node = map.node_mut(combat_node_id).unwrap();
            combat_node.category = MapNodeCategory::Combat;
            combat_node.kind_id = crate::game::map::MapNodeKindId::new("combat_low_risk");
            combat_node.payload = MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            };
        }

        let preview = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: combat_node_id,
                },
            )
            .unwrap();
        assert!(matches!(preview, BehaviorResult::NodePreview { .. }));

        let err = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .available_node_ids
            .contains(&medical_node_id));
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .available_node_ids
            .contains(&combat_node_id));
        core.execute(player_id, PlayerBehavior::CancelSelectedNode)
            .unwrap();

        let result = select_and_confirm_map_node(&mut core, player_id, medical_node_id);
        assert!(matches!(result, BehaviorResult::SupportState { .. }));
    }

    #[test]
    fn defense_combat_node_smoke_writes_debug_event_log_export() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0101);
        let player_id = Uuid::from_u128(0xD3F3_0101);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = &preview
        else {
            panic!("expected defense combat preview");
        };
        assert_eq!(
            combat_preview.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert!(combat_preview
            .routes
            .iter()
            .any(|route| route.id == "black_box_breach_main"));
        assert!(combat_preview.spawn_waves.iter().all(|wave| {
            wave.route_id.as_deref() == Some("black_box_breach_main") && wave.required_for_victory
        }));
        assert!(combat_preview
            .deployment_zones
            .iter()
            .any(|zone| { zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground }));
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];

        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let BehaviorResult::BattleAdvanced {
            battle_uuid,
            finished: false,
            ..
        } = result
        else {
            panic!("expected defense combat to enter live battle");
        };
        assert!(matches!(
            core.get_state(),
            GameState::InBattle {
                battle_uuid: state_uuid,
            } if state_uuid == battle_uuid
        ));
        let deploy_result = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid,
                    position: deploy_position,
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        let BehaviorResult::BattleUnitDeployed {
            timeline_delta,
            deployment,
            ..
        } = deploy_result
        else {
            panic!("expected live deploy result");
        };
        assert!(timeline_delta.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::UnitSpawned { .. }
        )));
        assert_eq!(deployment.deployed_units.len(), 1);

        let mut finished = false;
        for _ in 0..10_000 {
            let result = core.advance_active_battle_to_next_event_bucket().unwrap();
            if let BehaviorResult::BattleAdvanced { finished: true, .. } = result {
                finished = true;
                break;
            }
        }
        assert!(finished, "defense live battle should eventually finish");

        let battle = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap();
        assert_eq!(
            battle.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert_eq!(
            battle.timeline.version,
            crate::game::battle::timeline::TIMELINE_VERSION
        );
        assert!(battle.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleStart { .. }
        )));
        assert!(battle.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::UnitSpawned { .. }
        )));
        assert!(battle.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleEnd { .. }
        )));

        let path =
            write_world_debug_event_log_export("defense_combat_node_smoke", &battle.timeline);
        println!("wrote debug event log: {}", path.display());
        assert!(path.exists());
    }

    #[test]
    fn live_ron_defense_route_playable_path_runs_to_combat_result() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0001);
        let player_id = Uuid::from_u128(0xD3F3);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = &preview
        else {
            panic!("expected live defense combat preview");
        };
        assert_eq!(
            combat_preview.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        let route = combat_preview
            .routes
            .iter()
            .find(|route| route.id == "black_box_breach_main")
            .expect("live defense route should be authored");
        assert!(combat_preview.spawn_waves.iter().all(|wave| {
            wave.route_id.as_deref() == Some("black_box_breach_main") && wave.required_for_victory
        }));

        let mut deploy_positions = combat_preview
            .deployment_zones
            .iter()
            .filter(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
            .flat_map(|zone| zone.cells.iter().copied())
            .collect::<Vec<_>>();
        deploy_positions.sort_by_key(|position| {
            (
                position.manhattan(&route.end),
                (position.x - route.end.x).abs(),
                position.y,
                position.x,
            )
        });
        deploy_positions.dedup();
        assert!(
            deploy_positions.len() >= 2,
            "live defense should expose multiple ground deployment cells near the route endpoint"
        );
        let employee_ids = core.roster().unwrap().available_employee_ids();
        assert!(
            employee_ids.len() >= 2,
            "starter selection should provide enough employees for live defense smoke"
        );

        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::BattleAdvanced {
                finished: false,
                ..
            }
        ));
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        for (employee_uuid, position) in employee_ids.iter().take(2).zip(deploy_positions.iter()) {
            let result = core
                .execute(
                    player_id,
                    PlayerBehavior::DeployUnit {
                        employee_uuid: *employee_uuid,
                        position: *position,
                        facing: FacingDirection::Right,
                    },
                )
                .unwrap();
            assert!(matches!(result, BehaviorResult::BattleUnitDeployed { .. }));
        }

        let mut third_employee = employee_ids.get(2).copied();
        let third_position = deploy_positions.get(2).copied();
        let mut finished = false;
        for tick_index in 0..20_000 {
            let result = match core.advance_active_battle_to_next_event_bucket() {
                Ok(result) => result,
                Err(error) => {
                    let bodies = core
                        .state
                        .active_battle
                        .as_ref()
                        .map(|active| active.battle.live_unit_bodies())
                        .unwrap_or_default();
                    panic!(
                        "live RON defense tick {tick_index} failed with {error:?}; bodies={bodies:?}"
                    );
                }
            };
            let BehaviorResult::BattleAdvanced {
                finished: battle_finished,
                deployment,
                ..
            } = result
            else {
                panic!("expected live battle tick result");
            };
            if !battle_finished {
                if let (Some(employee_uuid), Some(position), Some(deployment)) =
                    (third_employee, third_position, deployment.as_ref())
                {
                    if deployment.current_cost >= deployment.base_deploy_cost {
                        let result = core
                            .execute(
                                player_id,
                                PlayerBehavior::DeployUnit {
                                    employee_uuid,
                                    position,
                                    facing: FacingDirection::Right,
                                },
                            )
                            .unwrap();
                        assert!(matches!(result, BehaviorResult::BattleUnitDeployed { .. }));
                        third_employee = None;
                    }
                }
            }
            if battle_finished {
                finished = true;
                break;
            }
        }
        assert!(
            finished,
            "live RON defense battle should finish deterministically"
        );
        assert!(matches!(core.get_state(), GameState::CombatResult { .. }));

        let battle = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap();
        assert_eq!(
            battle.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert!(battle.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleEnd { .. }
        )));
        assert!(battle.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::UnitSpawned { .. }
        )));

        let completion = core
            .execute(player_id, PlayerBehavior::CompleteCombatResult)
            .unwrap();
        assert!(matches!(
            completion,
            BehaviorResult::NodeCompleted {
                outcome: Some(_),
                ..
            }
        ));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
    }

    #[test]
    fn defense_live_battle_state_request_returns_timeline_delta_without_advancing_cursor() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0102);
        let player_id = Uuid::from_u128(0xD3F3_0102);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::BattleAdvanced {
                finished: false,
                ..
            }
        ));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::RequestBattleState));
        assert!(core.get_allowed_actions().contains(&ActionKind::DeployUnit));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::WithdrawUnit));
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(snapshot["game_state_context"]["type"], "in_battle");
        assert_eq!(
            snapshot["game_state_context"]["combat_preview"]["node_type"],
            "Defense"
        );
        assert!(snapshot["game_state_context"]["combat_preview"]["routes"]
            .as_array()
            .is_some_and(|routes| routes.iter().any(|route| {
                route["id"] == "black_box_breach_main"
                    && route["cells"]
                        .as_array()
                        .is_some_and(|cells| !cells.is_empty())
            })));
        assert!(
            snapshot["game_state_context"]["combat_preview"]["deployment_zones"]
                .as_array()
                .is_some_and(|zones| zones.iter().any(|zone| {
                    zone["kind"] == "Ground"
                        && zone["cells"]
                            .as_array()
                            .is_some_and(|cells| !cells.is_empty())
                }))
        );

        let state = core
            .execute(
                player_id,
                PlayerBehavior::RequestBattleState { since_seq: None },
            )
            .unwrap();
        let BehaviorResult::BattleState {
            node_type,
            combat_preview,
            timeline_delta,
            last_timeline_seq,
            finished,
            deployment,
            ..
        } = state
        else {
            panic!("expected battle state");
        };
        assert_eq!(
            node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert_eq!(
            combat_preview.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert!(combat_preview
            .deployment_zones
            .iter()
            .any(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground));
        assert!(!combat_preview.routes.is_empty());
        assert!(!finished);
        let deployment = deployment.expect("live defense should expose deployment state");
        assert!(deployment.deployed_units.is_empty());
        assert_eq!(deployment.battle_time_ms, 0);
        assert_eq!(
            deployment.current_cost,
            RUN_SYSTEM_POLICY.live_deployment.initial_cost
        );
        assert!(timeline_delta.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleStart { .. }
        )));
        assert_eq!(
            last_timeline_seq,
            timeline_delta.last().map(|entry| entry.seq).unwrap_or(0)
        );

        let repeated = core
            .execute(
                player_id,
                PlayerBehavior::RequestBattleState {
                    since_seq: Some(last_timeline_seq),
                },
            )
            .unwrap();
        let BehaviorResult::BattleState { timeline_delta, .. } = repeated else {
            panic!("expected repeated battle state");
        };
        assert!(timeline_delta.is_empty());
    }

    #[test]
    fn live_defense_playback_pause_freezes_server_tick_and_resume_advances() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0101);
        let player_id = Uuid::from_u128(0xD3F3_0101);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let BehaviorResult::BattleAdvanced {
            battle_time_ms,
            playback,
            ..
        } = result
        else {
            panic!("expected live battle start");
        };
        assert_eq!(battle_time_ms, 0);
        assert!(!playback.paused);
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X1
        );
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::PauseBattle));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::ResumeBattle));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::SetBattleSpeed));
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();

        let paused = core
            .execute(player_id, PlayerBehavior::PauseBattle)
            .unwrap();
        let BehaviorResult::BattlePlaybackChanged {
            playback,
            battle_time_ms: paused_time_ms,
            ..
        } = paused
        else {
            panic!("expected playback changed");
        };
        assert!(playback.paused);
        assert_eq!(paused_time_ms, 0);
        let paused_snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(
            paused_snapshot["game_state_context"]["playback"]["paused"],
            true
        );

        let paused_tick = core.advance_active_battle_for_server_tick(10).unwrap();
        assert!(paused_tick.is_none());
        let state = core
            .execute(
                player_id,
                PlayerBehavior::RequestBattleState { since_seq: None },
            )
            .unwrap();
        let BehaviorResult::BattleState {
            playback,
            battle_time_ms,
            ..
        } = state
        else {
            panic!("expected paused battle state");
        };
        assert!(playback.paused);
        assert_eq!(battle_time_ms, paused_time_ms);

        let resumed = core
            .execute(player_id, PlayerBehavior::ResumeBattle)
            .unwrap();
        let BehaviorResult::BattlePlaybackChanged { playback, .. } = resumed else {
            panic!("expected playback changed");
        };
        assert!(!playback.paused);

        let advanced = core
            .advance_active_battle_for_server_tick(10)
            .unwrap()
            .expect("resume should advance the live battle");
        let BehaviorResult::BattleAdvanced {
            battle_time_ms,
            playback,
            finished,
            ..
        } = advanced
        else {
            panic!("expected battle advanced");
        };
        assert!(!finished);
        assert!(!playback.paused);
        assert_eq!(battle_time_ms, 10);
    }

    #[test]
    fn live_defense_playback_speed_scales_server_tick_delta() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0102);
        let player_id = Uuid::from_u128(0xD3F3_0102);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_ids = core.roster().unwrap().available_employee_ids();
        let employee_uuid = employee_ids[0];
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();

        let changed = core
            .execute(
                player_id,
                PlayerBehavior::SetBattleSpeed {
                    speed: crate::game::behavior::BattlePlaybackSpeed::X2,
                },
            )
            .unwrap();
        let BehaviorResult::BattlePlaybackChanged { playback, .. } = changed else {
            panic!("expected playback changed");
        };
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X2
        );

        let x2_tick = core
            .advance_active_battle_for_server_tick(10)
            .unwrap()
            .expect("x2 tick should advance");
        let BehaviorResult::BattleAdvanced {
            battle_time_ms,
            playback,
            finished,
            ..
        } = x2_tick
        else {
            panic!("expected x2 battle advanced");
        };
        assert!(!finished);
        assert_eq!(battle_time_ms, 20);
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X2
        );

        core.execute(
            player_id,
            PlayerBehavior::SetBattleSpeed {
                speed: crate::game::behavior::BattlePlaybackSpeed::X3,
            },
        )
        .unwrap();
        let x3_tick = core
            .advance_active_battle_for_server_tick(10)
            .unwrap()
            .expect("x3 tick should advance");
        let BehaviorResult::BattleAdvanced {
            battle_time_ms,
            playback,
            finished,
            ..
        } = x3_tick
        else {
            panic!("expected x3 battle advanced");
        };
        assert!(!finished);
        assert_eq!(battle_time_ms, 50);
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X3
        );
    }

    #[test]
    fn live_defense_half_speed_accumulates_fractional_server_ticks() {
        let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0103);
        let player_id = Uuid::from_u128(0xD3F3_0103);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            "defend_black_box_relay",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_ids = core.roster().unwrap().available_employee_ids();
        let employee_uuid = employee_ids[0];
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let changed = core
            .execute(
                player_id,
                PlayerBehavior::SetBattleSpeed {
                    speed: crate::game::behavior::BattlePlaybackSpeed::X0_5,
                },
            )
            .unwrap();
        let BehaviorResult::BattlePlaybackChanged { playback, .. } = changed else {
            panic!("expected playback changed");
        };
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X0_5
        );

        let first_tick = core.advance_active_battle_for_server_tick(1).unwrap();
        assert!(
            first_tick.is_none(),
            "first 1ms tick at 0.5x should only accumulate fractional time"
        );

        let second_tick = core
            .advance_active_battle_for_server_tick(1)
            .unwrap()
            .expect("second 1ms tick at 0.5x should advance one simulation ms");
        let BehaviorResult::BattleAdvanced {
            battle_time_ms,
            playback,
            finished,
            ..
        } = second_tick
        else {
            panic!("expected half-speed battle advanced");
        };
        assert!(!finished);
        assert_eq!(battle_time_ms, 1);
        assert_eq!(
            playback.speed,
            crate::game::behavior::BattlePlaybackSpeed::X0_5
        );
    }

    #[test]
    fn live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_ids = core.roster().unwrap().available_employee_ids();
        let employee_uuid = employee_ids[0];
        let second_employee_uuid = employee_ids[1];
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let deployed = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid,
                    position: deploy_position,
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        let BehaviorResult::BattleUnitDeployed {
            unit_instance_id,
            deployment,
            ..
        } = deployed
        else {
            panic!("expected live deploy result");
        };
        assert_eq!(
            deployment.current_cost,
            RUN_SYSTEM_POLICY
                .live_deployment
                .initial_cost
                .saturating_sub(RUN_SYSTEM_POLICY.live_deployment.base_deploy_cost)
        );
        assert_eq!(deployment.deployed_units.len(), 1);
        assert_eq!(deployment.deployed_units[0].employee_uuid, employee_uuid);
        assert_eq!(
            deployment.deployed_units[0].unit_instance_id,
            unit_instance_id
        );
        assert_eq!(deployment.deployed_units[0].facing, FacingDirection::Right);
        if let Some(deployment) = core
            .state
            .active_battle
            .as_mut()
            .and_then(|battle| battle.live_deployment.as_mut())
        {
            deployment.current_cost = RUN_SYSTEM_POLICY.live_deployment.base_deploy_cost;
        }
        let occupied = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid: second_employee_uuid,
                    position: deploy_position,
                    facing: FacingDirection::Right,
                },
            )
            .expect_err("allied deployment must reject occupied positions");
        assert!(matches!(occupied, GameError::PositionOccupied));

        let withdrawn = core
            .execute(player_id, PlayerBehavior::WithdrawUnit { employee_uuid })
            .unwrap();
        let BehaviorResult::BattleUnitWithdrawn {
            unit_instance_id: withdrawn_unit,
            deployment,
            ..
        } = withdrawn
        else {
            panic!("expected live withdraw result");
        };
        assert_eq!(withdrawn_unit, unit_instance_id);
        assert!(deployment.deployed_units.is_empty());
        assert_eq!(deployment.redeploying_units.len(), 1);
        assert_eq!(deployment.redeploying_units[0].employee_uuid, employee_uuid);
        assert_eq!(
            deployment.redeploying_units[0].deploy_cost,
            RUN_SYSTEM_POLICY
                .live_deployment
                .base_deploy_cost
                .saturating_mul(
                    RUN_SYSTEM_POLICY
                        .live_deployment
                        .redeploy_cost_multiplier_pct
                )
                / 100
        );
    }

    #[test]
    fn deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let consumable = consumable_meta(
            0xD3F3_3001,
            "deploy_cost_capsule",
            ConsumableTier::Uncommon,
            ConsumableEffect::DeployCostReduction { percent: 50 },
        );
        core.roster_mut()
            .unwrap()
            .get_mut(&employee_uuid)
            .unwrap()
            .apply_consumable_modifier(Uuid::from_u128(0xD3F3_3002), &consumable);

        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let deployment = core
            .state
            .active_battle
            .as_ref()
            .and_then(|active| active.live_deployment_dto(&core.state.roster))
            .expect("live deployment dto");
        let reduced_cost = RUN_SYSTEM_POLICY
            .live_deployment
            .base_deploy_cost
            .saturating_sub(
                RUN_SYSTEM_POLICY
                    .live_deployment
                    .base_deploy_cost
                    .div_ceil(2),
            );
        let unit_cost = deployment
            .unit_deploy_costs
            .iter()
            .find(|cost| cost.employee_uuid == employee_uuid)
            .expect("employee deploy cost dto");
        assert_eq!(
            unit_cost.base_deploy_cost,
            RUN_SYSTEM_POLICY.live_deployment.base_deploy_cost
        );
        assert_eq!(unit_cost.effective_deploy_cost, reduced_cost);

        let deployed = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid,
                    position: deploy_position,
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();

        let BehaviorResult::BattleUnitDeployed { deployment, .. } = deployed else {
            panic!("expected live deploy result");
        };
        assert_eq!(
            deployment.current_cost,
            RUN_SYSTEM_POLICY
                .live_deployment
                .initial_cost
                .saturating_sub(reduced_cost)
        );
    }

    #[test]
    fn activate_skill_uses_equipped_manual_fragment_in_live_defense() {
        let skill_id = SkillId::from("manual_fragment_guard");
        let fragment = active_skill_fragment("manual_fragment", 0xD3F3_2001, skill_id.as_str());
        let skill = SkillDef {
            id: skill_id.clone(),
            name: skill_id.to_string(),
            kind: Default::default(),
            cast_targeting: Default::default(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "self_guard".to_string(),
                delay_ms: 0,
                range_units: 1.0,
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
        };
        let weapon = weapon_equipment(0xE005, "manual_fragment_sword", WeaponArchetype::Sword);
        let mut core = GameCore::new(
            game_data_with_pve_equipment_and_active_skill_fragments(
                vec![weapon.clone()],
                vec![skill],
                vec![fragment.clone()],
            ),
            123,
        );
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            weapon,
            Uuid::from_u128(0xE005_0001),
        );
        let consumable = consumable_meta(
            0xD3F3_3003,
            "skill_charge_serum",
            ConsumableTier::Critical,
            ConsumableEffect::InitialSkillCharge { percent: 100 },
        );
        core.roster_mut()
            .unwrap()
            .get_mut(&employee_uuid)
            .unwrap()
            .apply_consumable_modifier(Uuid::from_u128(0xD3F3_3004), &consumable);
        core.state.skill_fragments.add(&fragment).unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );
        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::ActivateSkill));
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let snapshot = core.get_run_snapshot_json().unwrap();
        let catalog_skills = snapshot["skill_catalog"]["skills"].as_array().unwrap();
        let catalog_skill = catalog_skills
            .iter()
            .find(|skill| skill["skill_id"] == json!(skill_id))
            .expect("equipped manual skill should be exposed in skill catalog");
        assert_eq!(catalog_skill["display_name"], skill_id.as_str());
        assert_eq!(catalog_skill["steps"][0]["delivery"], "instant");

        let deployed_units = snapshot["game_state_context"]["deployment"]["deployed_units"]
            .as_array()
            .unwrap();
        let deployed = deployed_units
            .iter()
            .find(|unit| unit["employee_uuid"] == json!(employee_uuid))
            .expect("deployed employee should be exposed in live deployment snapshot");
        assert_eq!(deployed["skill_readiness"]["skill_id"], json!(skill_id));
        assert_eq!(deployed["skill_readiness"]["activation_mode"], "manual");
        assert_eq!(
            deployed["skill_readiness"]["manual_activation_allowed"],
            true
        );
        assert_eq!(deployed["skill_readiness"]["target_required"], false);
        assert_eq!(deployed["skill_readiness"]["target_available"], true);
        assert_eq!(
            deployed["skill_readiness"]["resonance_current"],
            deployed["skill_readiness"]["resonance_max"]
        );

        let activated = core
            .execute(
                player_id,
                PlayerBehavior::ActivateSkill {
                    employee_uuid,
                    skill_id: skill_id.clone(),
                    target: None,
                },
            )
            .unwrap();
        let BehaviorResult::BattleSkillActivated {
            timeline_delta,
            unit_instance_id,
            ..
        } = activated
        else {
            panic!("expected live skill activation result");
        };
        assert!(timeline_delta.iter().any(|entry| {
            matches!(
                &entry.event,
                crate::game::battle::timeline::TimelineEvent::ManualCastStart {
                    caster_instance_id,
                    skill_id: actual_skill_id,
                    ..
                } if *caster_instance_id == unit_instance_id && actual_skill_id == &skill_id
            )
        }));

        let advanced = core.advance_active_battle_by(1).unwrap();
        let BehaviorResult::BattleAdvanced { timeline_delta, .. } = advanced else {
            panic!("expected battle advance after manual cast");
        };
        assert!(timeline_delta.iter().any(|entry| {
            matches!(
                &entry.event,
                crate::game::battle::timeline::TimelineEvent::AbilityCast {
                    caster_instance_id,
                    skill_id: actual_skill_id,
                    ..
                } if *caster_instance_id == unit_instance_id && actual_skill_id == &skill_id
            )
        }));
    }

    #[test]
    fn manual_fragment_readiness_keeps_button_enabled_when_target_is_missing() {
        let skill_id = SkillId::from("manual_fragment_mark_target");
        let fragment = active_skill_fragment("target_fragment", 0xD3F3_2005, skill_id.as_str());
        let skill = SkillDef {
            id: skill_id.clone(),
            name: skill_id.to_string(),
            kind: Default::default(),
            cast_targeting: Default::default(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "mark".to_string(),
                delay_ms: 0,
                range_units: 0.0,
                defense_tile_range: Some(crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec![".@X".to_string()],
                }),
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: Default::default(),
                },
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![],
                presentation: Default::default(),
            }],
        };
        let weapon = weapon_equipment(0xE006, "target_fragment_sword", WeaponArchetype::Sword);
        let mut core = GameCore::new(
            game_data_with_pve_equipment_and_active_skill_fragments(
                vec![weapon.clone()],
                vec![skill],
                vec![fragment.clone()],
            ),
            123,
        );
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            weapon,
            Uuid::from_u128(0xE006_0001),
        );
        core.roster_mut()
            .unwrap()
            .get_mut(&employee_uuid)
            .unwrap()
            .combat_profile
            .battle_profile
            .resonance
            .start = 100;
        core.state.skill_fragments.add(&fragment).unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );
        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let deploy_position = first_ground_deployment_cell(&preview);
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();

        let snapshot = core.get_run_snapshot_json().unwrap();
        let deployed_units = snapshot["game_state_context"]["deployment"]["deployed_units"]
            .as_array()
            .unwrap();
        let deployed = deployed_units
            .iter()
            .find(|unit| unit["employee_uuid"] == json!(employee_uuid))
            .expect("deployed employee should be exposed in live deployment snapshot");
        assert_eq!(deployed["skill_readiness"]["skill_id"], json!(skill_id));
        assert_eq!(
            deployed["skill_readiness"]["manual_activation_allowed"], true,
            "manual button should remain available so Unity can open target selection"
        );
        assert_eq!(deployed["skill_readiness"]["target_required"], true);
        assert_eq!(deployed["skill_readiness"]["target_available"], false);
        assert_eq!(
            deployed["skill_readiness"]["target_block_reason"],
            "no_valid_target"
        );
        assert!(deployed["skill_readiness"]["can_activate_reason"].is_null());
    }

    #[test]
    fn manual_fragment_can_restore_stabilization_and_enable_extra_deployment() {
        let skill_id = SkillId::from("manual_fragment_stabilize");
        let fragment =
            active_skill_fragment("stabilization_fragment", 0xD3F3_2002, skill_id.as_str());
        let skill = SkillDef {
            id: skill_id.clone(),
            name: skill_id.to_string(),
            kind: Default::default(),
            cast_targeting: Default::default(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "restore_stabilization".to_string(),
                delay_ms: 0,
                range_units: 1.0,
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::SelfUnit,
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStabilization {
                    amount: RUN_SYSTEM_POLICY.live_deployment.max_cost as i32 + 50,
                }],
                presentation: Default::default(),
            }],
        };
        let weapon = weapon_equipment(
            0xE007,
            "stabilization_fragment_sword",
            WeaponArchetype::Sword,
        );
        let mut core = GameCore::new(
            game_data_with_pve_equipment_and_active_skill_fragments(
                vec![weapon.clone()],
                vec![skill],
                vec![fragment.clone()],
            ),
            123,
        );
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_ids = core.roster().unwrap().available_employee_ids();
        let caster_uuid = employee_ids[0];
        let second_uuid = employee_ids[1];
        let third_uuid = employee_ids[2];
        grant_and_equip_weapon(&mut core, caster_uuid, weapon, Uuid::from_u128(0xE007_0001));
        core.roster_mut()
            .unwrap()
            .get_mut(&caster_uuid)
            .unwrap()
            .combat_profile
            .battle_profile
            .resonance
            .start = 100;
        core.state.skill_fragments.add(&fragment).unwrap();
        core.execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: caster_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );
        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = &preview
        else {
            panic!("expected node preview");
        };
        let mut deploy_positions = combat_preview
            .deployment_zones
            .iter()
            .flat_map(|zone| zone.cells.iter().copied())
            .collect::<Vec<_>>();
        deploy_positions.sort_by_key(|position| (position.y, position.x));
        deploy_positions.dedup();
        assert!(deploy_positions.len() >= 3);

        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        for (employee_uuid, position) in [
            (caster_uuid, deploy_positions[0]),
            (second_uuid, deploy_positions[1]),
        ] {
            core.execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid,
                    position,
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        }

        let insufficient = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid: third_uuid,
                    position: deploy_positions[2],
                    facing: FacingDirection::Right,
                },
            )
            .expect_err("third deployment should require restored stabilization");
        assert!(matches!(insufficient, GameError::InsufficientResources));

        core.execute(
            player_id,
            PlayerBehavior::ActivateSkill {
                employee_uuid: caster_uuid,
                skill_id: skill_id.clone(),
                target: None,
            },
        )
        .unwrap();
        let advanced = core.advance_active_battle_by(1).unwrap();
        let BehaviorResult::BattleAdvanced {
            deployment: Some(deployment),
            ..
        } = advanced
        else {
            panic!("expected live deployment after stabilization skill");
        };
        assert_eq!(
            deployment.current_cost,
            RUN_SYSTEM_POLICY.live_deployment.max_cost
        );

        let deployed = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid: third_uuid,
                    position: deploy_positions[2],
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        let BehaviorResult::BattleUnitDeployed { deployment, .. } = deployed else {
            panic!("expected third deployment after stabilization restore");
        };
        assert_eq!(deployment.deployed_units.len(), 3);
        assert_eq!(
            deployment.current_cost,
            RUN_SYSTEM_POLICY
                .live_deployment
                .max_cost
                .saturating_sub(RUN_SYSTEM_POLICY.live_deployment.base_deploy_cost)
        );
    }

    #[test]
    fn retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(snapshot["game_state_context"]["type"], "node_confirm");
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
            json!(3)
        );
        let result = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::BattleAdvanced {
                finished: false,
                ..
            }
        ));
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::RetreatBattle));
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(snapshot["game_state_context"]["type"], "in_battle");
        assert_eq!(snapshot["game_state_context"]["can_retreat"], json!(true));
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["attempts_started"],
            json!(1)
        );
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
            json!(2)
        );
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let employee_before = core.roster().unwrap().get(&employee_uuid).unwrap().clone();

        let result = core
            .execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();

        let BehaviorResult::NodePreview {
            node_id: retreated_node_id,
            ..
        } = result
        else {
            panic!("expected retreat to return to node confirm while attempts remain");
        };
        assert_eq!(retreated_node_id, node_id);
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["attempts_started"],
            json!(1)
        );
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
            json!(2)
        );
        assert!(core.state.active_battle.is_none());
        assert!(core.state.node_session.is_some());
        assert!(core.state.active_node_content.is_none());
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Revealed));
        let employee_after = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(
            employee_after.health.current_hp,
            employee_before.health.current_hp
        );
        assert_eq!(employee_after.trauma, employee_before.trauma);
        assert_eq!(employee_after.experience, employee_before.experience);

        for expected_remaining in [1, 0] {
            core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
                .unwrap();
            let result = core
                .execute(player_id, PlayerBehavior::RetreatBattle)
                .unwrap();

            if expected_remaining > 0 {
                let BehaviorResult::NodePreview { .. } = result else {
                    panic!("expected retreat to keep node alive before attempts are exhausted");
                };
                assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
                let snapshot = core.get_run_snapshot_json().unwrap();
                assert_eq!(
                    snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
                    json!(expected_remaining)
                );
                continue;
            }

            let BehaviorResult::NodeCompleted {
                outcome: Some(outcome),
                ..
            } = result
            else {
                panic!("expected third retreat to consume the exhausted abnormality");
            };
            assert!(!outcome.mission_success);
            assert_eq!(outcome.node_id, node_id);
            let combat = outcome.combat.expect("combat summary");
            assert_eq!(
                combat.node_type,
                crate::game::combat_preview::CombatNodeType::Defense
            );
            assert_eq!(combat.winner, BattleWinner::Draw);
            assert!(combat.retreated);
            assert!(outcome.employee_changes.is_empty());
            assert!(outcome.inventory_diff.added.is_empty());
            assert!(outcome.research_deliveries.is_empty());
            assert!(matches!(core.get_state(), GameState::ViewingMap));
            assert!(core.state.active_battle.is_none());
            assert!(core.state.node_session.is_none());
            assert!(core.state.active_node_content.is_none());
            assert!(core
                .state
                .run
                .as_ref()
                .unwrap()
                .map
                .node(node_id)
                .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
        }
    }

    #[test]
    fn retreat_marks_rumor_threat_warning_disproved_on_reentry_preview() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );

        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let active = core
            .state
            .active_battle
            .as_mut()
            .expect("battle should be active");
        active.combat_preview.threat_warnings = vec![
            ThreatWarning {
                tag: ThreatWarningTag::ArmoredEnemyPossible,
                status: ThreatWarningStatus::Unverified,
                source: ThreatWarningSource::Briefing,
            },
            ThreatWarning {
                tag: ThreatWarningTag::AirEnemyPossible,
                status: ThreatWarningStatus::Unverified,
                source: ThreatWarningSource::Rumor,
            },
        ];

        let result = core
            .execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(preview),
            ..
        } = result
        else {
            panic!("retreat should return node preview while attempts remain");
        };

        assert!(preview.threat_warnings.iter().any(|warning| {
            warning.source == ThreatWarningSource::Briefing
                && warning.status == ThreatWarningStatus::Unverified
        }));
        assert!(preview.threat_warnings.iter().any(|warning| {
            warning.source == ThreatWarningSource::Rumor
                && warning.status == ThreatWarningStatus::Disproved
        }));

        let cached = core
            .state
            .run
            .as_ref()
            .unwrap()
            .combat_previews
            .get(&node_id)
            .expect("retreat should update cached node preview");
        assert!(cached.threat_warnings.iter().any(|warning| {
            warning.source == ThreatWarningSource::Rumor
                && warning.status == ThreatWarningStatus::Disproved
        }));
    }

    #[test]
    fn consumable_modifier_survives_retreat_reentry_and_expires_when_abnormality_is_resolved() {
        let consumable = consumable_meta(
            0xC030,
            "reentry_ampoule",
            ConsumableTier::Common,
            ConsumableEffect::TraumaMitigation { percent: 20 },
        );
        let owned_uuid = Uuid::from_u128(0xC030_0001);
        let mut core = GameCore::new(
            game_data_with_pve_and_consumables(vec![consumable.clone()]),
            123,
        );
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        core.inventory_mut()
            .unwrap()
            .consumables
            .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
            .unwrap();

        core.execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: owned_uuid,
                target_employee_uuid: employee_uuid,
            },
        )
        .unwrap();
        assert_eq!(
            core.roster()
                .unwrap()
                .get(&employee_uuid)
                .unwrap()
                .active_consumable_modifier
                .as_ref()
                .unwrap()
                .remaining_combat_nodes,
            1
        );

        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();

        for expected_remaining_attempts in [2, 1] {
            core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
                .unwrap();
            core.execute(player_id, PlayerBehavior::RetreatBattle)
                .unwrap();
            assert_eq!(
                core.roster()
                    .unwrap()
                    .get(&employee_uuid)
                    .unwrap()
                    .active_consumable_modifier
                    .as_ref()
                    .unwrap()
                    .remaining_combat_nodes,
                1
            );
            let snapshot = core.get_run_snapshot_json().unwrap();
            assert_eq!(
                snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
                json!(expected_remaining_attempts)
            );
        }

        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::NodeCompleted {
                outcome: Some(_),
                ..
            }
        ));
        assert!(core
            .roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .active_consumable_modifier
            .is_none());
    }
}

mod placement {
    use super::*;

    fn core_with_starter_employee_ids() -> (GameCore, Vec<Uuid>) {
        let mut core = GameCore::new(empty_game_data(), 123);
        start_new_game_with_default_starters(&mut core, Uuid::from_u128(1));
        let employee_ids = core.roster().unwrap().available_employee_ids();
        (core, employee_ids)
    }

    #[test]
    fn move_roster_unit_reorders_and_swaps_roster_slots() {
        let (mut core, employee_ids) = core_with_starter_employee_ids();
        let left_uuid = employee_ids[0];
        let right_uuid = employee_ids[1];

        let result = core
            .execute(
                Uuid::from_u128(1),
                PlayerBehavior::MoveRosterUnit {
                    target_unit_uuid: left_uuid,
                    dest_slot: 1,
                    swap_with_unit_uuid: Some(right_uuid),
                },
            )
            .unwrap();

        let BehaviorResult::MoveRosterUnit { roster_slots } = result else {
            panic!("expected roster order move result");
        };
        assert_eq!(
            roster_slots
                .iter()
                .find(|slot| slot.slot == 1)
                .and_then(|slot| slot.unit_uuid),
            Some(left_uuid)
        );
        let roster_order = core.roster_order().unwrap();
        assert_eq!(roster_order.slot_of(left_uuid), Some(1));
        assert_eq!(roster_order.slot_of(right_uuid), Some(0));
    }
}
