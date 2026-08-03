use super::*;
use crate::game::ability::{
    DeliveryDef, SkillCastTargetingDef, SkillDef, SkillId, SkillStepDef, SkillTarget,
};
use crate::game::battle::{event_log::BattleEventLog, tile_range::FacingDirection};
use crate::game::combat_preview::{
    ThreatWarning, ThreatWarningSource, ThreatWarningStatus, ThreatWarningTag,
};
use crate::game::data::{
    abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
    consumable_data::{
        ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
        ConsumableTargetPolicy, ConsumableTier,
    },
    corroded_employee_data::{
        CorrodedEmployeeProfileDatabase, CorrodedEmployeeProfileMetadata,
        CorrodedEmployeeProfileRole,
    },
    employee_data::EmployeeDefinitionDatabase,
    equipment_data::{
        EquipmentDatabase, EquipmentDismantleRecipeMetadata, EquipmentEnhancementRecipeMetadata,
        EquipmentMaterialMetadata, EquipmentMetadata, EquipmentType, WeaponArchetype,
        WeaponCombatProfile, WeaponRangeRole,
    },
    pve_data::{
        PveBattlefieldOverrideData, PveEncounter, PveEncounterClass, PveEncounterDatabase,
        PveWaveData, PveWaveEnemyData, PveWaveSource,
    },
    reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata},
    run_policy_data::RunPolicyData,
    shop_data::{
        ShopContext, ShopDatabase, ShopMerchandise, ShopMerchandiseEntry, ShopMetadata,
        ShopPoolMetadata, ShopSellPolicy,
    },
    skill_data::SkillDatabase,
    skill_fragment_data::{
        SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
        SkillFragmentEquipLimit, SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
    },
    visual_novel_data::{
        VisualNovelBackground, VisualNovelDatabase, VisualNovelDefinition, VisualNovelId,
        VisualNovelPresentationMode, VisualNovelSceneDefinition, VisualNovelSceneId,
        VisualNovelSceneNext, VisualNovelTextKind,
    },
    visual_novel_trigger_data::{
        VisualNovelTrigger, VisualNovelTriggerCheckpointPolicy, VisualNovelTriggerDatabase,
        VisualNovelTriggerId, VisualNovelTriggerRepeat, VisualNovelTriggerSource,
    },
    GameDataBase, GameDataBuilder,
};
use crate::game::employee::{
    CombatAptitude, EmployeeBaseCombatStats, EmployeeCondition, EmployeeDefinition,
    EmployeeInitialLoadout, EmployeePersonality, EmployeeProfile, EmployeeRiskDisposition,
    EmployeeSocialDisposition, EventAptitude, MAX_PHYSICAL_DAMAGE,
};
use crate::game::enums::RewardMode;
use crate::game::map::{
    test_facility_map, FacilityTerminalResolution, GameMode, MapNode, MapNodeCategory, MapNodeId,
    MapNodeKindId, MapNodePayload, MapNodeState, MapNodeVisibility, MapSlotId, MapTemplateId,
    MapTerminalNode, MapViewDto, RunMap, FACILITY_TEMPLATE_VERSION,
};
use crate::game::resources::item_slot::EquippedRef;
use crate::game::resources::{EquipItemOutcomeDto, OwnedEquipment, Position, ShopSessionState};
use crate::game::reward::{ExperienceTargetPolicy, RewardEffect, RewardOption};
use crate::game::skill_fragment::starter_basic_attack_fragment_id;
use crate::game::stats::{StatId, StatModifier, StatModifierKind};
use crate::game::visual_novel::{VisualNovelTriggerConsumedKey, VisualNovelTriggerConsumedScope};
use serde_json::{json, Value};
use std::sync::Arc;

fn action_capability(
    core: &GameCore,
    action: ActionKind,
) -> Option<crate::game::behavior::ActionCapabilityDto> {
    core.get_action_capabilities()
        .into_iter()
        .find(|capability| capability.action == action)
}

fn action_capability_enabled(core: &GameCore, action: ActionKind) -> bool {
    action_capability(core, action).is_some_and(|capability| capability.enabled)
}

fn execute_current_battle_mutation(
    core: &mut GameCore,
    player_id: Uuid,
    behavior: PlayerBehavior,
) -> Result<BehaviorResult, GameError> {
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("battle mutation test requires active battle");
    let authority = crate::game::behavior::BattleStateAuthorityDto {
        battle_uuid: active.battle_uuid,
        battle_state_revision: active.battle_state_revision,
    };
    core.execute_with_transport_context(player_id, behavior, None, 0, Some(authority))
}

fn execute_current_battle_mutation_with_source(
    core: &mut GameCore,
    player_id: Uuid,
    behavior: PlayerBehavior,
    source_command_id: &str,
) -> Result<BehaviorResult, GameError> {
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("battle mutation test requires active battle");
    let authority = crate::game::behavior::BattleStateAuthorityDto {
        battle_uuid: active.battle_uuid,
        battle_state_revision: active.battle_state_revision,
    };
    core.execute_with_transport_context(
        player_id,
        behavior,
        Some(source_command_id),
        0,
        Some(authority),
    )
}

fn empty_game_data() -> Arc<GameDataBase> {
    test_game_data_builder().build_arc()
}

fn test_game_data_builder() -> GameDataBuilder {
    GameDataBuilder::empty()
        .with_employee_definitions(test_employee_definition_database())
        .with_visual_novels(test_hq_briefing_visual_novel_database())
        .with_visual_novel_triggers(test_hq_briefing_visual_novel_trigger_database())
        .with_equipment(vec![starter_equipment_meta()])
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
}

fn test_hq_briefing_visual_novel_database() -> VisualNovelDatabase {
    VisualNovelDatabase::new(vec![
        test_hq_briefing_visual_novel_definition("hq_briefing_standard_intro"),
        test_hq_briefing_visual_novel_definition("hq_briefing_endless_intro"),
    ])
}

fn test_hq_briefing_visual_novel_trigger_database() -> VisualNovelTriggerDatabase {
    VisualNovelTriggerDatabase::new(vec![
        test_hq_briefing_visual_novel_trigger("run_start_standard_hq_briefing", GameMode::Standard),
        test_hq_briefing_visual_novel_trigger("run_start_endless_hq_briefing", GameMode::Endless),
    ])
}

fn test_hq_briefing_visual_novel_trigger(id: &str, game_mode: GameMode) -> VisualNovelTrigger {
    let vn_id = match game_mode {
        GameMode::Standard => "hq_briefing_standard_intro",
        GameMode::Endless => "hq_briefing_endless_intro",
    };
    VisualNovelTrigger {
        id: VisualNovelTriggerId::new(id),
        source: VisualNovelTriggerSource::RunStart { game_mode },
        vn_id: VisualNovelId::new(vn_id),
        repeat: VisualNovelTriggerRepeat::OncePerRun,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad,
        priority: 100,
        blocking: true,
    }
}

fn test_hq_briefing_visual_novel_definition(id: &str) -> VisualNovelDefinition {
    VisualNovelDefinition {
        id: VisualNovelId::new(id),
        entry_scene_id: VisualNovelSceneId::new("start"),
        default_presentation_mode: VisualNovelPresentationMode::Cinematic,
        scenes: vec![VisualNovelSceneDefinition {
            id: VisualNovelSceneId::new("start"),
            text_kind: VisualNovelTextKind::Narration,
            background: VisualNovelBackground::CurrentScreenDimmed,
            script_id: id.to_string(),
            speakers: vec![],
            next: VisualNovelSceneNext::End,
        }],
    }
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

fn test_employee_definition_database() -> EmployeeDefinitionDatabase {
    let profile = |role: &str, background: &str| EmployeeProfile {
        role: role.to_string(),
        background: background.to_string(),
        personality: EmployeePersonality {
            risk: EmployeeRiskDisposition::Cautious,
            social: EmployeeSocialDisposition::Obedient,
        },
        combat_aptitude: CombatAptitude::Balanced,
        event_aptitude: EventAptitude::Maintenance,
        base_combat_stats: EmployeeBaseCombatStats::default(),
        basic_attack_speed_percent: 100,
    };
    let starters = (0..6).map(|index| EmployeeDefinition {
        employee_definition_id: format!("candidate_{index}"),
        name: format!("Candidate {index}"),
        starter_eligible: true,
        initial_loadout: EmployeeInitialLoadout {
            equipment_ids: vec!["standard_armor".to_string()],
            baseline_skill_fragment_ids: vec![starter_basic_attack_fragment_id()],
        },
        profile: profile("테스트 후보", "테스트용 시작 직원 후보"),
    });
    let recruits = (0..6).map(|index| EmployeeDefinition {
        employee_definition_id: format!("recruit_{index}"),
        name: format!("Recruit {index}"),
        starter_eligible: false,
        initial_loadout: EmployeeInitialLoadout::default(),
        profile: profile("테스트 채용 후보", "테스트용 런 중 채용 후보"),
    });
    EmployeeDefinitionDatabase::new(starters.chain(recruits).collect())
}

fn start_new_game_with_mode_and_default_starters(
    core: &mut GameCore,
    player_id: Uuid,
    game_mode: GameMode,
) -> BehaviorResult {
    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode,
                replace_active_run: false,
            },
        )
        .expect("start new game should open pre-run briefing");
    let BehaviorResult::RunStartFlowStarted {
        game_mode: returned_game_mode,
        visual_novel: Some(visual_novel),
    } = start
    else {
        panic!("start new game should return run start flow with VN");
    };
    assert_eq!(returned_game_mode, game_mode);
    let current_scene_id = crate::game::data::visual_novel_data::VisualNovelSceneId::new(
        visual_novel.current_scene_id,
    );
    let advance = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: visual_novel.session_id,
                current_scene_id,
            },
        )
        .expect("pre-run briefing should advance to starter selection");
    let BehaviorResult::VisualNovelState { visual_novel: None } = advance else {
        panic!("ending pre-run briefing should close the VN overlay");
    };
    let employee_definition_ids = core
        .state
        .starter_candidates
        .iter()
        .take(run_policy().setup.starter_employee_count)
        .map(|definition| definition.employee_definition_id.clone())
        .collect::<Vec<_>>();
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids,
        },
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
    GameDataBase::load_live_embedded().expect("embedded live gameplay RON must be valid")
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
                test_shop_metadata("map_shop", shop_uuid, ShopContext::UnknownExploration),
                test_shop_metadata(
                    "hq_supply_shop",
                    Uuid::from_u128(10_006),
                    ShopContext::HeadquartersSupport,
                ),
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
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    }
}

fn starter_equipment_meta() -> EquipmentMetadata {
    let mut equipment = equipment_meta(
        0x650e_8400_e29b_41d4_a716_4466_5544_0031,
        "standard_armor",
        EquipmentType::Armor,
    );
    equipment.price = 80;
    equipment
}

fn test_shop_metadata(id: &str, uuid: Uuid, context: ShopContext) -> ShopMetadata {
    ShopMetadata {
        id: id.to_string(),
        name: id.to_string(),
        uuid,
        context,
        offer_count: 1,
        default_buy_percent: 100,
        preset_refs: vec![],
        local_entries: vec![ShopMerchandiseEntry {
            merchandise: ShopMerchandise::Equipment {
                equipment_id: "standard_armor".to_string(),
            },
            quantity: 1,
            weight: 1,
            repeatable: false,
            buy_percent: None,
            tags: vec!["equipment".to_string()],
        }],
        refresh_policy: None,
        sell_policy: Some(ShopSellPolicy {
            price_percent: 40,
            accepted_tags: None,
        }),
    }
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
            WeaponArchetype::Bow | WeaponArchetype::Gun | WeaponArchetype::Staff => {
                WeaponRangeRole::Ranged
            }
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

fn game_data_with_display_items(equipment: Vec<EquipmentMetadata>) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_equipment(equipment_with_starter_assets(equipment))
        .build_arc()
}

fn game_data_with_equipment(
    abnormality: Arc<AbnormalityMetadata>,
    equipment: Vec<EquipmentMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![(*abnormality).clone()])
        .with_equipment_data(Arc::new(EquipmentDatabase::new(
            equipment_with_starter_assets(equipment),
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
        .with_equipment_data(Arc::new(EquipmentDatabase::with_materials_and_dismantles(
            equipment_with_starter_assets(equipment),
            materials,
            dismantle_recipes,
        )))
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
        .with_employee_definition_data(base.employee_definition_data.clone())
        .with_equipment_data(Arc::new(EquipmentDatabase::new(
            equipment_with_starter_assets(equipment),
        )))
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_visual_novel_data(base.visual_novel_data.clone())
        .with_visual_novel_trigger_data(base.visual_novel_trigger_data.clone())
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
        .with_employee_definition_data(base.employee_definition_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_visual_novel_data(base.visual_novel_data.clone())
        .with_visual_novel_trigger_data(base.visual_novel_trigger_data.clone())
        .with_pve_data(base.pve_data.clone())
        .with_consumable_data(Arc::new(ConsumableDatabase::new(consumables)))
        .with_skill_fragment_data(base.skill_fragment_data.clone())
        .build_arc()
}

fn game_data_with_visual_novel_triggers(
    base: Arc<GameDataBase>,
    triggers: VisualNovelTriggerDatabase,
) -> Arc<GameDataBase> {
    GameDataBuilder::empty()
        .with_abnormality_data(base.abnormality_data.clone())
        .with_corroded_employee_data(base.corroded_employee_data.clone())
        .with_corroded_wave_data(base.corroded_wave_data.clone())
        .with_employee_definition_data(base.employee_definition_data.clone())
        .with_consumable_data(base.consumable_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_event_data(base.event_data.clone())
        .with_visual_novel_data(base.visual_novel_data.clone())
        .with_visual_novel_triggers(triggers)
        .with_pve_data(base.pve_data.clone())
        .with_boss_omen_data(base.boss_omen_data.clone())
        .with_run_policy_data(base.run_policy.clone())
        .with_skill_data(base.skill_data.clone())
        .with_buff_data(base.buff_data.clone())
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
    let current_template_id = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map
        .map_template_id
        .clone();
    let current_template_supports_category = core
        .game_data
        .map_catalogs
        .facility_templates
        .template(&current_template_id)
        .is_some_and(|template| {
            template
                .slots
                .iter()
                .any(|slot| slot.allowed_categories.contains(&category))
        });

    if !current_template_supports_category {
        let floor_index = match category {
            MapNodeCategory::SavePoint
            | MapNodeCategory::Maintenance
            | MapNodeCategory::HeadquartersContact
            | MapNodeCategory::Shop => 1,
            MapNodeCategory::Boss => 2,
            MapNodeCategory::Gate => 0,
            MapNodeCategory::Start
            | MapNodeCategory::Combat
            | MapNodeCategory::Event
            | MapNodeCategory::Reward => 0,
        };
        let run_seed = core
            .state
            .run
            .as_ref()
            .expect("run state")
            .run_progression
            .run_seed;
        let generated = crate::game::map::FacilityMapGenerator::generate(
            run_seed,
            GameMode::Standard,
            floor_index,
            &crate::game::map::FacilityRunState::default(),
            &core.game_data.map_catalogs,
            core.run_policy(),
            100,
        )
        .expect("test category must resolve to an authored facility template");
        let run = core.state.run.as_mut().expect("run state");
        run.map = generated.map;
        run.map_progression = crate::game::map::MapProgression::from_map(&run.map);
        run.facility_run_state = generated.facility_run_state;
    }

    let run = core.state.run.as_ref().expect("run state");
    let template = core
        .game_data
        .map_catalogs
        .facility_templates
        .template(&run.map.map_template_id)
        .expect("run map template")
        .clone();
    let target_slot_id = template
        .slots
        .iter()
        .find(|slot| slot.allowed_categories.contains(&category))
        .expect("authored slot for forced category")
        .id
        .clone();
    let start_slot_id = template
        .slots
        .iter()
        .find(|slot| slot.role == crate::game::map::FacilitySlotRole::Start)
        .map(|slot| slot.id.clone())
        .or_else(|| {
            template
                .navigation_start_slot_ids()
                .into_iter()
                .find(|slot_id| *slot_id == target_slot_id)
        })
        .expect("facility test path start");

    let mut parents = std::collections::HashMap::<MapSlotId, MapSlotId>::new();
    let mut queue = std::collections::VecDeque::from([start_slot_id.clone()]);
    let mut visited = std::collections::HashSet::from([start_slot_id.clone()]);
    while let Some(slot_id) = queue.pop_front() {
        if slot_id == target_slot_id {
            break;
        }
        for neighbor in template.neighbor_slot_ids(&slot_id) {
            if visited.insert(neighbor.clone()) {
                parents.insert(neighbor.clone(), slot_id.clone());
                queue.push_back(neighbor);
            }
        }
    }
    assert!(
        visited.contains(&target_slot_id),
        "forced category slot must be reachable"
    );
    let mut path = vec![target_slot_id.clone()];
    while path.last() != Some(&start_slot_id) {
        path.push(
            parents
                .get(path.last().expect("path tail"))
                .expect("reachable path parent")
                .clone(),
        );
    }
    path.reverse();

    let run = core.state.run.as_mut().expect("run state");
    let node_id = run
        .map
        .nodes
        .iter()
        .find(|node| node.slot_id == target_slot_id)
        .expect("node assigned to forced category slot")
        .id;
    let mut completed_node_ids = Vec::new();
    for slot_id in path.iter().take(path.len().saturating_sub(1)) {
        let node = run
            .map
            .nodes
            .iter_mut()
            .find(|node| node.slot_id == *slot_id)
            .expect("path node");
        node.state = MapNodeState::Completed;
        node.visibility = MapNodeVisibility::Revealed;
        completed_node_ids.push(node.id);
    }
    let node = run.map.node_mut(node_id).expect("node exists");
    node.category = category;
    node.kind_id = crate::game::map::MapNodeKindId::new(kind_id);
    node.payload = payload;
    node.state = MapNodeState::Available;
    node.visibility = MapNodeVisibility::Revealed;
    run.map_progression = crate::game::map::MapProgression {
        current_node_id: completed_node_ids.last().copied(),
        available_node_ids: vec![node_id],
        completed_node_ids,
    };
    node_id
}

fn normalize_other_available_save_points(core: &mut GameCore, except_node_id: MapNodeId) {
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
        if node.category != MapNodeCategory::SavePoint {
            continue;
        }
        node.kind_id = crate::game::map::MapNodeKindId::new("save_point");
        node.payload = MapNodePayload::SavePoint;
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

mod facility;

mod node_sessions;

mod equipment;

mod combat;
