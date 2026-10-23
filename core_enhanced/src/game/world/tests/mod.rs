use super::*;
use crate::game::ability::{
    DeliveryDef, SkillCastSelection, SkillCastTargetingDef, SkillDef, SkillId, SkillStepDef,
    SkillTarget,
};
use crate::game::battle::{
    buffs::{BuffDatabase, BuffKind, BuffMetadata, BuffReapplyPolicy},
    event_log::{BattleEventLog, SkillCastTarget},
    tile_range::FacingDirection,
};
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
        PveEncounterRewardSource, PveWaveData, PveWaveEnemyData, PveWaveSource,
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
    EmployeeInitialLoadout, EmployeePersonality, EmployeeProfile, EmployeeRiskDisposition, EmployeeRole,
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

fn refresh_active_run_checksum(document: &mut Value) {
    use sha2::{Digest, Sha256};
    let (payload, checksum) = if document.get("continuation").is_some() {
        ("continuation", "continuation_sha256")
    } else {
        ("checkpoint", "checkpoint_sha256")
    };
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(map) => serde_json::to_value(
                map.iter()
                    .map(|(key, value)| (key, sorted(value)))
                    .collect::<std::collections::BTreeMap<_, _>>(),
            )
            .unwrap(),
            Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
            value => value.clone(),
        }
    }
    document[checksum] = json!(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&sorted(&document[payload])).unwrap())
    ));
}

impl GameCore {
    // Test fixtures mutate roots directly. Publish that fixture explicitly;
    // production SavePoint and Continue capture remain independent operations.
    fn save_run_checkpoint_for_test(&mut self) -> Result<(), GameError> {
        self.save_run_checkpoint()?;
        self.refresh_continue_after_operation(super::application::ApplicationRoute::Behavior(
            ActionKind::CompleteNode,
        ))
    }
}

fn action_capability(
    core: &GameCore,
    action: ActionKind,
) -> Option<crate::game::behavior::ActionCapabilityDto> {
    core.get_action_capabilities()
        .into_iter()
        .find(|capability| capability.action == action)
}

fn assert_persisted_active_shop_session_equal(core: &GameCore, node_id: MapNodeId) {
    let persisted = core
        .state
        .run
        .as_ref()
        .expect("paired Shop session requires a run")
        .shop_sessions
        .get(&node_id)
        .expect("persisted Shop session");
    let active = core
        .state
        .active_node_content
        .as_ref()
        .expect("active Shop session")
        .as_shop()
        .expect("active content should be Shop");
    assert_eq!(
        serde_json::to_value(persisted).expect("persisted Shop session serializes"),
        serde_json::to_value(active).expect("active Shop session serializes"),
        "persisted and active Shop sessions must be complete-value equal"
    );
}

fn assert_persisted_active_event_session_equal(core: &GameCore, node_id: MapNodeId) {
    let persisted = core
        .state
        .run
        .as_ref()
        .expect("paired Event session requires a run")
        .event_sessions
        .get(&node_id)
        .expect("persisted Event session");
    let active = core
        .state
        .active_node_content
        .as_ref()
        .expect("active Event session")
        .as_event()
        .expect("active content should be Event");
    assert_eq!(
        persisted, active,
        "persisted and active Event sessions must be complete-value equal"
    );
}

fn non_battle_state_without_profile_fingerprint(core: &GameCore) -> Value {
    let run_payload = core.state.run.as_ref().map(|run| {
        crate::game::world::state::RunCheckpointPayload::capture(
            &core.state,
            run,
            core.game_data.identity(),
        )
        .unwrap()
    });
    let run_combat_recovery_restriction = core
        .state
        .run
        .as_ref()
        .map(|run| format!("{:?}", run.combat_recovery_restriction));
    serde_json::json!({
        "player": &core.state.player,
        "game_state": core.state.lifecycle.phase(),
        "active_node_content": core
            .get_selected_event_snapshot_json()
            .expect("active node content should project while fingerprinting"),
        "pending_mastered_elite_sweep": core
            .staged_mastered_elite_sweep_result()
            .map(|result| serde_json::to_value(result).expect("sweep result serializes")),
        "node_session": &core.state.node_session,
        "run_payload": run_payload,
        "run_combat_recovery_restriction": run_combat_recovery_restriction,
        "visual_novel": &core.state.visual_novel,
        "visual_novel_triggers": &core.state.visual_novel_triggers,
        "uuid_manager": &core.state.uuid_manager,
        "enkephalin": &core.state.enkephalin,
        "inventory": &core.state.inventory,
        "roster": &core.state.roster,
        "starter_candidates": &core.state.starter_candidates,
        "pending_game_mode": &core.state.pending_game_mode,
        "skill_fragments": &core.state.skill_fragments,
        "skill_fragment_policy": format!("{:?}", core.state.skill_fragment_policy),
        "active_battle_present": core.state.active_battle.is_some(),
        "run_checkpoint": &core.state.run_checkpoint,
        "run_power_authority": &core.state.run_power_authority,
    })
}

fn assert_non_battle_fingerprint_unchanged_except(
    before: &Value,
    after: &Value,
    mutable_roots: &[&str],
) {
    let before = before
        .as_object()
        .expect("before non-battle fingerprint must be an object");
    let after = after
        .as_object()
        .expect("after non-battle fingerprint must be an object");
    assert_eq!(before.len(), after.len(), "fingerprint root count changed");
    for (root, before_value) in before {
        assert!(
            after.contains_key(root),
            "fingerprint root '{root}' disappeared"
        );
        if !mutable_roots.contains(&root.as_str()) {
            assert_eq!(
                after.get(root),
                Some(before_value),
                "non-battle root '{root}' changed outside the branch contract"
            );
        }
    }
}

fn assert_run_payload_fingerprint_unchanged_except(
    before: &Value,
    after: &Value,
    mutable_fields: &[&str],
) {
    let before = before["run_payload"]
        .as_object()
        .expect("before run payload fingerprint must be an object");
    let after = after["run_payload"]
        .as_object()
        .expect("after run payload fingerprint must be an object");
    assert_eq!(before.len(), after.len(), "run payload field count changed");
    for (field, before_value) in before {
        assert!(
            after.contains_key(field),
            "run payload field '{field}' disappeared"
        );
        if !mutable_fields.contains(&field.as_str()) {
            assert_eq!(
                after.get(field),
                Some(before_value),
                "run payload field '{field}' changed outside the branch contract"
            );
        }
    }
}

fn action_capability_enabled(core: &GameCore, action: ActionKind) -> bool {
    action_capability(core, action).is_some_and(|capability| capability.enabled)
}

fn serialized_grant_roots(core: &GameCore) -> Value {
    json!({
        "inventory": &core.state.inventory,
        "skill_fragments": &core.state.skill_fragments,
        "roster": &core.state.roster,
        "uuid_manager": &core.state.uuid_manager,
        "enkephalin": &core.state.enkephalin,
    })
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

// Continue consumes a separately accepted profile. Never infer permanent
// ownership from the active-run bytes under test.
fn core_with_profile_for_continue(source: &GameCore, seed: u64) -> GameCore {
    let mut destination = GameCore::new(source.game_data.clone(), seed);
    destination.install_authoritative_player_profile(source.state.player_profile.clone()).unwrap();
    destination
}

fn install_application_profile_for_continue(
    destination: &mut crate::application::CoreApplication,
    source: &crate::application::CoreApplication,
) {
    let profile = source.snapshot(crate::application::ApplicationSnapshotKind::PlayerProfile)
        .unwrap().read_model().clone();
    destination.install_authoritative_player_profile(profile).unwrap();
}

fn test_game_data_builder() -> GameDataBuilder {
    let mut policy = run_policy();
    policy.tutorial_employee_definition_ids = vec!["candidate_0".into(), "candidate_2".into(), "candidate_4".into()];
    // These owner fixtures synthesize legacy floor indices and nodes. U13's
    // authored room routes are separately loaded and traversed in u13_facility.
    let mut catalogs=crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
    std::sync::Arc::make_mut(&mut catalogs.facility_templates).layouts.clear();
    GameDataBuilder::empty_with_map_catalogs(catalogs)
        .with_run_policy(policy)
        .with_employee_definitions(test_employee_definition_database())
        .with_visual_novels(test_hq_briefing_visual_novel_database())
        .with_visual_novel_triggers(test_hq_briefing_visual_novel_trigger_database())
        .with_equipment(equipment_with_starter_assets(Vec::new()))
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
}

fn test_control_buff_database() -> Arc<BuffDatabase> {
    Arc::new(BuffDatabase::new(vec![
        BuffMetadata {
            name: "stun".to_string(),
            kind: BuffKind::Stun,
            tick_interval_ms: 0,
            max_stacks: 1,
            reapply_policy: BuffReapplyPolicy::RefreshDuration,
        },
        BuffMetadata {
            name: "freeze".to_string(),
            kind: BuffKind::Freeze,
            tick_interval_ms: 0,
            max_stacks: 1,
            reapply_policy: BuffReapplyPolicy::RefreshDuration,
        },
        BuffMetadata {
            name: "silence".to_string(),
            kind: BuffKind::Silence,
            tick_interval_ms: 0,
            max_stacks: 1,
            reapply_policy: BuffReapplyPolicy::RefreshDuration,
        },
    ]))
}

fn test_hq_briefing_visual_novel_database() -> VisualNovelDatabase {
    VisualNovelDatabase::new(vec![
        test_hq_briefing_visual_novel_definition("hq_briefing_standard_intro"),
        test_hq_briefing_visual_novel_definition("hq_briefing_endless_intro"),
    ])
}

fn test_hq_briefing_visual_novel_trigger_database() -> VisualNovelTriggerDatabase {
    VisualNovelTriggerDatabase::new(vec![
        test_hq_briefing_visual_novel_trigger("run_start_standard_hq_briefing", GameMode::Expedition),
        test_hq_briefing_visual_novel_trigger("run_start_endless_hq_briefing", GameMode::Tutorial),
    ])
}

fn test_hq_briefing_visual_novel_trigger(id: &str, game_mode: GameMode) -> VisualNovelTrigger {
    let vn_id = match game_mode {
        GameMode::Expedition => "hq_briefing_standard_intro",
        GameMode::Tutorial => "hq_briefing_endless_intro",
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
        role: match index / 2 { 0 => EmployeeRole::Dealer, 1 => EmployeeRole::Tank, _ => EmployeeRole::Healer },
        initial_loadout: EmployeeInitialLoadout {
            equipment_ids: vec![
                match index / 2 {
                    0 => "standard_training_sword",
                    1 => "test_starter_shield",
                    _ => "test_starter_healing_staff",
                }.to_string(),
                "standard_armor".to_string(),
            ],
            baseline_skill_fragment_ids: vec![starter_basic_attack_fragment_id()],
            active_skill_fragment_id: None,
        },
        profile: profile("테스트 후보", "테스트용 시작 직원 후보"),
    });
    let recruits = (0..6).map(|index| EmployeeDefinition {
        employee_definition_id: format!("recruit_{index}"),
        name: format!("Recruit {index}"),
        starter_eligible: false,
        role: match index / 2 { 0 => EmployeeRole::Dealer, 1 => EmployeeRole::Tank, _ => EmployeeRole::Healer },
        initial_loadout: EmployeeInitialLoadout {
            equipment_ids: vec![match index / 2 {
                0 => "standard_training_sword",
                1 => "test_starter_shield",
                _ => "test_starter_healing_staff",
            }.into(), "standard_armor".into()],
            baseline_skill_fragment_ids: vec![starter_basic_attack_fragment_id()],
            active_skill_fragment_id: None,
        },
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
    if game_mode == GameMode::Tutorial {
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert_eq!(core.state.roster.len(), 3, "tutorial must install its authored preset");
        return advance;
    }
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
            loan_copy_ids: vec![],
        },
    )
    .expect("default starter employee selection should start the run")
}

fn start_new_game_with_default_starters(core: &mut GameCore, player_id: Uuid) -> BehaviorResult {
    start_new_game_with_mode_and_default_starters(core, player_id, GameMode::Expedition)
}

fn game_data_with_pve_encounters() -> Arc<GameDataBase> {
    let sweep_base_reward_uuid = Uuid::from_u128(40_001);
    let mut abnormalities = vec![
        test_abnormality_meta("low_risk_abno", 20_001),
        test_boss_abnormality_meta("boss_risk_abno", 20_002),
        test_abnormality_meta("elite_risk_abno", 20_003),
        test_abnormality_meta("ambush_risk_abno", 20_004),
        test_abnormality_meta("defense_risk_abno", 20_005),
        test_abnormality_meta("defense_route_abno", 20_006),
        test_abnormality_meta("defense_corridor_abno", 20_007),
        test_boss_abnormality_meta("final_boss_risk_abno", 20_008),
    ];
    let fragments = abnormalities
        .iter_mut()
        .enumerate()
        .map(|(index, abnormality)| {
            let fragment_id = SkillFragmentId::from(format!("fragment_{}", abnormality.id));
            abnormality.response_complete_skill_fragment_id = Some(fragment_id.clone());
            SkillFragmentMetadata {
                corrosion_class:
                    crate::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
                id: fragment_id,
                uuid: Uuid::from_u128(30_000 + index as u128),
                name: format!("{} fragment", abnormality.id),
                description: "test encounter source fragment".to_string(),
                rarity: SkillFragmentRarity::Rare,
                equip_limit: SkillFragmentEquipLimit::OwnedCopies,
                origin: None,
                sources: Vec::new(),
                dependencies: Vec::new(),
                compatibility: Default::default(),
                effect: SkillFragmentEffectDef::BasicAttackModifier {
                    attack_bonus: 1,
                    attack_interval_ms_reduction: 0,
                },
            }
        })
        .collect::<Vec<_>>();
    test_game_data_builder()
        .with_equipment_data(Arc::new(EquipmentDatabase::with_materials(
            equipment_with_starter_assets(Vec::new()),
            [("revival_catalyst", crate::game::data::equipment_data::EquipmentMaterialType::RevivalCatalyst),
             ("part_big_bad_wolf", crate::game::data::equipment_data::EquipmentMaterialType::AbnormalityPart)]
                .into_iter().enumerate().map(|(i, (id, material_type))| EquipmentMaterialMetadata {
                    id: id.into(), uuid: Uuid::from_u128(171700 + i as u128), name: id.into(), description: id.into(),
                    material_type, rarity: crate::game::enums::RiskLevel::WAW, equipment_type: None,
                }).collect(),
        )))
        .with_buff_data(test_control_buff_database())
        .with_abnormalities(abnormalities)
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
        .with_rewards(RewardDatabase::new(vec![RewardMetadata {
            id: "sweep_base_reward".to_string(),
            uuid: sweep_base_reward_uuid,
            name: "Sweep base reward".to_string(),
            description: "Test authored combat reward".to_string(),
            icon: "test".to_string(),
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 7 },
                RewardEffect::GrantFragmentDust { amount: 3 },
            ],
        }]))
        .with_corroded_employee_data(Arc::new(CorrodedEmployeeProfileDatabase::new(vec![
            test_corroded_employee_profile("corroded_guard", 21_001),
        ])))
        .with_pve(PveEncounterDatabase::new_with_reward_sources(
            vec![
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "normal_corroded_encounter".to_string(),
                    encounter_class: PveEncounterClass::Normal,
                    primary_abnormality_id: None,
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::ZAYIN,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: Some(PveBattlefieldOverrideData {
                        archetype: Some(
                            crate::game::combat_preview::BattlefieldArchetype::OpenHall,
                        ),
                        size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                    }),
                    tactical_plan: None,
                    waves: vec![test_corroded_pve_wave("corroded_guard")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "low_risk_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("low_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::ZAYIN,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: Some(PveBattlefieldOverrideData {
                        archetype: Some(
                            crate::game::combat_preview::BattlefieldArchetype::OpenHall,
                        ),
                        size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                    }),
                    tactical_plan: None,
                    waves: vec![test_defense_pve_wave("low_risk_abno")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "boss_risk_encounter".to_string(),
                    encounter_class: PveEncounterClass::NormalBoss,
                    primary_abnormality_id: Some("boss_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::ALEPH,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Boss),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: None,
                    tactical_plan: None,
                    waves: vec![test_pve_wave("boss_risk_abno")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "final_boss_risk_encounter".to_string(),
                    encounter_class: PveEncounterClass::FinalBoss,
                    primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::ALEPH,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Boss),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: None,
                    tactical_plan: None,
                    waves: vec![test_pve_wave("final_boss_risk_abno")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "elite_risk_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("elite_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::WAW,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![sweep_base_reward_uuid],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: None,
                    tactical_plan: None,
                    waves: vec![test_defense_pve_wave("elite_risk_abno")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "elite_survival_timer_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("ambush_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::WAW,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(Some(45_000)),
                    battlefield: None,
                    tactical_plan: None,
                    waves: vec![PveWaveData {
                        required_for_victory: false,
                        ..test_defense_pve_wave("ambush_risk_abno")
                    }],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "defense_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("defense_risk_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::HE,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: Some(PveBattlefieldOverrideData {
                        archetype: Some(
                            crate::game::combat_preview::BattlefieldArchetype::ChokePoint,
                        ),
                        size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                    }),
                    tactical_plan: None,
                    waves: vec![PveWaveData {
                        route_id: Some("defense_main".to_string()),
                        ..test_pve_wave("defense_risk_abno")
                    }],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "defense_route_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("defense_route_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::HE,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: Some(PveBattlefieldOverrideData {
                        archetype: Some(
                            crate::game::combat_preview::BattlefieldArchetype::Corridor,
                        ),
                        size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                    }),
                    tactical_plan: None,
                    waves: vec![test_defense_pve_wave("defense_route_abno")],
                    static_obstacles: vec![],
                },
                PveEncounter {
                    deterrence_tuning: Default::default(),
                    bonus_objectives: Vec::new(),
                    id: "defense_corridor_encounter".to_string(),
                    encounter_class: PveEncounterClass::Elite,
                    primary_abnormality_id: Some("defense_corridor_abno".to_string()),
                    boss_runtime: None,
                    risk_level: crate::game::enums::RiskLevel::TETH,
                    outcome_policies: PveEncounter::fixture_outcome_policies(
                        RewardMode::ClaimAll,
                        vec![],
                    ),
                    node_type: Some(crate::game::combat_preview::CombatNodeType::Defense),
                    mission_variant: None,
                    terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                    battlefield: Some(PveBattlefieldOverrideData {
                        archetype: Some(
                            crate::game::combat_preview::BattlefieldArchetype::Corridor,
                        ),
                        size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                    }),
                    tactical_plan: None,
                    waves: vec![test_defense_pve_wave("defense_corridor_abno")],
                    static_obstacles: vec![],
                },
            ],
            vec![
                PveEncounterRewardSource {
                    reward_source_id: "source_low_risk".to_string(),
                    encounter_id: "low_risk_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_low_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_boss_risk".to_string(),
                    encounter_id: "boss_risk_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_boss_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 10 }],
                    revival: Some(crate::game::revival::BossRevivalPolicy {
                        catalyst_material_id: "revival_catalyst".into(), catalyst_amount: 1, part_material_id: "part_big_bad_wolf".into(),
                        stage_2: fixture_revival_stage("boss_risk_encounter", 140, 2),
                        stage_3: fixture_revival_stage("boss_risk_encounter", 180, 4),
                    }),
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_final_boss_risk".to_string(),
                    encounter_id: "final_boss_risk_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_final_boss_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 20 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_elite_risk".to_string(),
                    encounter_id: "elite_risk_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_elite_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_elite_survival".to_string(),
                    encounter_id: "elite_survival_timer_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_ambush_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_defense".to_string(),
                    encounter_id: "defense_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_defense_risk_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_defense_route".to_string(),
                    encounter_id: "defense_route_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_defense_route_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
                PveEncounterRewardSource {
                    reward_source_id: "source_defense_corridor".to_string(),
                    encounter_id: "defense_corridor_encounter".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_defense_corridor_abno"),
                    first_equipment_ids: vec!["standard_armor".to_owned()],
                    repeat_rewards: vec![RewardEffect::GrantEquipmentDust { amount: 5 }],
                    revival: None,
                },
            ],
        ))
        .build_arc()
}

// Explicit pre-U06 Sword/baseline numerical inputs are applied AFTER a valid
// role-compatible departure. Official definitions and starter validation stay
// intact; ordinary live-data and U06/U12 loadout tests never call this helper.
fn apply_legacy_sword_numerical_fixture(core: &mut GameCore, player: Uuid) {
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    let mut employees: Vec<_> = core.state.roster.iter().map(|employee| employee.uuid).collect();
    // Fixture-owned UUID allocation must not depend on HashMap iteration.
    employees.sort_by_key(|uuid| uuid.as_u128());
    let sword = core.game_data.equipment_data.get_by_id("standard_training_sword").unwrap().clone();
    for employee_uuid in employees {
        if let Some(copy_uuid) = core.state.roster.get(&employee_uuid).unwrap().skill_fragments.active_copy_uuid() {
            core.execute(player, PlayerBehavior::UnequipSkillFragment { employee_uuid, copy_uuid }).unwrap();
            // I: discard the now-unbound training skill rather than granting
            // dismantle currency or retaining an unrequested numerical input.
            core.state.skill_fragments.remove(copy_uuid).unwrap();
        }
        let weapon = core.state.inventory.equipments.iter().find(|copy|
            copy.equipped_to == Some(employee_uuid) && copy.meta.equipment_type == EquipmentType::Weapon)
            .expect("normal starter role has its authored weapon");
        let old_copy = weapon.instance_uuid;
        if weapon.meta.id != "standard_training_sword" {
            core.execute(player, PlayerBehavior::UnEquipItem { item_uuid: old_copy, target_unit: employee_uuid }).unwrap();
            core.state.inventory.equipments.remove_item(old_copy).unwrap();
            let item_uuid = core.state.uuid_manager.next_owned_equipment();
            core.state.inventory.equipments.add_item(OwnedEquipment::new(item_uuid, Arc::new(sword.clone()))).unwrap();
            core.execute(player, PlayerBehavior::EquipItem { item_uuid, target_unit: employee_uuid }).unwrap();
        }
        // I: preserve the inherited baseline damage input without changing the
        // catalog's authored starter roles, prices, effects or validators.
        core.state.roster.get_mut(&employee_uuid).unwrap().skill_fragments =
            crate::game::skill_fragment::SkillFragmentLoadout::with_baseline_ids(vec![starter_basic_attack_fragment_id()]);
    }
    core.refresh_continue_after_operation(super::application::ApplicationRoute::Behavior(ActionKind::EquipItem)).unwrap();
}

fn start_new_game_with_legacy_sword_fixture(core: &mut GameCore, player: Uuid) {
    start_new_game_with_default_starters(core, player);
    apply_legacy_sword_numerical_fixture(core, player);
}

fn legacy_pool_game_data_from_ron() -> Arc<GameDataBase> {
    let mut data=live_game_data_from_ron();
    Arc::make_mut(&mut Arc::get_mut(&mut data).unwrap().map_catalogs.facility_templates).layouts.clear();
    data
}

fn live_game_data_from_ron() -> Arc<GameDataBase> {
    crate::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid")
        .into_fixture_database_arc()
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
        activation: crate::game::data::pve_data::PveWaveActivationData::at_time("wave_0", 0),
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
        activation: crate::game::data::pve_data::PveWaveActivationData::at_time("wave_0", 0),
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
        content_family: None,
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

fn starter_weapon_meta() -> EquipmentMetadata {
    let mut equipment = equipment_meta(
        0x650e_8400_e29b_41d4_a716_4466_5544_0030,
        "standard_training_sword",
        EquipmentType::Weapon,
    );
    let profile = equipment
        .weapon_profile
        .as_mut()
        .expect("starter weapon fixture must author a weapon profile");
    profile.block_capacity = 1;
    let defense_focus = profile
        .ground_defense_focus
        .as_mut()
        .expect("starter weapon fixture must author ground defense focus");
    defense_focus.block_capacity_bonus = 1;
    defense_focus.incoming_damage_reduction_percent = 10;
    equipment
}

fn starter_role_weapon_meta(healer: bool) -> EquipmentMetadata {
    let mut equipment = starter_weapon_meta();
    equipment.id = if healer { "test_starter_healing_staff" } else { "test_starter_shield" }.into();
    equipment.uuid = Uuid::from_u128(if healer { 0x650e_8400_e29b_41d4_a716_4466_5544_0033 } else { 0x650e_8400_e29b_41d4_a716_4466_5544_0032 });
    let profile = equipment.weapon_profile.as_mut().unwrap();
    profile.weapon_archetype = if healer { WeaponArchetype::Staff } else { WeaponArchetype::Shield };
    if healer {
        profile.range_role = WeaponRangeRole::Ranged;
        profile.action = crate::game::data::abnormality_data::BasicAttackAction::Heal {
            flat: 8, spell_power_percent: 100, footprint: None,
        };
    }
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
            base_price: None,
            guaranteed: false,
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
    if !equipment
        .iter()
        .any(|item| item.id == "standard_training_sword")
    {
        equipment.push(starter_weapon_meta());
    }
    if !equipment.iter().any(|item| item.id == "standard_armor") {
        equipment.push(starter_equipment_meta());
    }
    for healer in [false, true] {
        let weapon = starter_role_weapon_meta(healer);
        if !equipment.iter().any(|item| item.id == weapon.id) {
            equipment.push(weapon);
        }
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

fn force_unequip_employee_weapon(core: &mut GameCore, employee_uuid: Uuid) -> Uuid {
    let weapon_instance_uuid = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .find(|equipped| equipped.equipment_type == EquipmentType::Weapon)
        .map(|equipped| equipped.instance_uuid)
        .expect("test employee must start armed");
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .remove_by_instance(weapon_instance_uuid)
        .unwrap();
    core.inventory_mut()
        .unwrap()
        .equipments
        .get_item_mut(&weapon_instance_uuid)
        .unwrap()
        .equipped_to = None;
    weapon_instance_uuid
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
            SkillFragmentEffectDef::ActiveSkill { forms, .. } => Some(SkillDef {
                action_policy: None,
                resonance_admission: None,
                use_lifetime: crate::game::ability::SkillUseLifetime::Action,
                flow: None,
                id: forms["base"].skill_id.clone(),
                name: forms["base"].skill_id.to_string(),
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
            SkillFragmentEffectDef::ActiveSkill { forms, .. } => Some(SkillDef {
                action_policy: None,
                resonance_admission: None,
                use_lifetime: crate::game::ability::SkillUseLifetime::Action,
                flow: None,
                id: forms["base"].skill_id.clone(),
                name: forms["base"].skill_id.to_string(),
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
    mut fragments: Vec<SkillFragmentMetadata>,
) -> Arc<GameDataBase> {
    let base = game_data_with_pve_encounters();
    fragments.extend(base.skill_fragment_data.fragments.iter().cloned());
    GameDataBuilder::empty()
        .with_abnormality_data(base.abnormality_data.clone())
        .with_corroded_employee_data(base.corroded_employee_data.clone())
        .with_corroded_wave_data(base.corroded_wave_data.clone())
        .with_employee_definition_data(base.employee_definition_data.clone())
        .with_equipment_data(Arc::new(EquipmentDatabase::with_materials(
            equipment_with_starter_assets(equipment), base.equipment_data.materials.clone(),
        )))
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_visual_novel_data(base.visual_novel_data.clone())
        .with_visual_novel_trigger_data(base.visual_novel_trigger_data.clone())
        .with_pve_data(base.pve_data.clone())
        .with_skills(SkillDatabase::new(skills))
        .with_skill_fragments(SkillFragmentDatabase::new(fragments))
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
        corrosion_class: crate::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
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
            forms: std::collections::BTreeMap::from([(
                "base".into(),
                crate::game::data::skill_fragment_data::SkillFragmentForm {
                    resonance_max: 100,
                    skill_id: SkillId::from(skill_id),
                    compatibility: None,
                    bloom_recipe: None,
                },
            )]),
            passive_guard: None,
            dismantle_recipe: Some(
                crate::game::data::skill_fragment_data::SkillFragmentRecipe {
                    enkephalin: 2,
                    materials: vec![],
                },
            ),
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

    let current_template_supports_category = current_template_supports_category && {
        let run = core.state.run.as_ref().unwrap();
        run.facility_run_state.rooms.is_none() || run.map.nodes.iter().any(|node|
            node.category == category && core.facility_node_support(node.id).map_or(true, |support| support.is_supported()))
    };

    if !current_template_supports_category {
        // I: install an actual authored graph that supports the requested
        // category. The current Expedition development floors intentionally
        // have no Event/Boss slots, so advancing an assumed floor index cannot
        // supply this input. Tutorial is only the generator's schedule selector;
        // the run's mode, lifecycle identity and depth remain unchanged.
        let catalogs = &core.game_data.map_catalogs;
        let schedule = catalogs.node_assignment_profiles.standard_schedule.iter()
            .filter(|entry| catalogs.facility_templates.pool(&entry.pool_id).is_some_and(|pool|
                !pool.template_ids.is_empty() && pool.template_ids.iter().all(|id|
                    catalogs.facility_templates.template(id).is_some_and(|template|
                        template.slots.iter().any(|slot| slot.allowed_categories.contains(&category))))))
            // Prefer a graph that also supports SavePoint: history fixtures
            // must be able to save without replacing their completed Event.
            .max_by_key(|entry| catalogs.facility_templates.pool(&entry.pool_id).unwrap()
                .template_ids.iter().all(|id| catalogs.facility_templates.template(id).unwrap()
                    .slots.iter().any(|slot| slot.allowed_categories.contains(&MapNodeCategory::SavePoint))))
            .expect("a current authored schedule must contain the requested test category");
        let floor_index = u32::from(schedule.floor).checked_sub(1).expect("authored floor is one-based");
        let run_seed = core
            .state
            .run
            .as_ref()
            .expect("run state")
            .run_progression
            .run_seed;
        let floor_power_anchor = core.state.run_power_authority.current_floor_power_anchor;
        let generated = crate::game::map::FacilityMapGenerator::generate(
            run_seed,
            GameMode::Tutorial,
            floor_index,
            &crate::game::map::FacilityRunState::default(),
            &core.game_data.map_catalogs,
            core.run_policy(),
            floor_power_anchor,
        )
        .expect("test category must resolve to an authored facility template");
        assert!(core.state.active_battle.is_none(), "location fixture cannot discard a live battle");
        let run = core.state.run.as_mut().expect("run state");
        // Historical event/shop receipts belong to their original node graph.
        // Tests needing them must choose their supported map before creating
        // those receipts; never erase history to make a checkpoint validate.
        assert!(run.event_sessions.is_empty(), "choose the fixture graph before event history exists");
        assert!(run.shop_sessions.is_empty(), "choose the fixture graph before shop history exists");
        if run.recruitment.active_session().is_some() {
            run.recruitment.return_active_session_to_tail().unwrap();
        }
        run.map = generated.map;
        run.map_progression = crate::game::map::MapProgression::from_map(&run.map);
        run.facility_run_state = generated.facility_run_state;
        run.run_progression.entry_receipts.clear();
        run.run_progression.pending_continuation = None;
        run.combat_previews.clear();
        run.abnormality_attempts.clear();
        run.combat_recovery_restriction = None;
        core.state.node_session = None;
        core.state.active_node_content = None;
        core.transition_to(GameState::ViewingMap).unwrap();
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
        .filter(|slot| slot.allowed_categories.contains(&category))
        .filter(|slot| run.map.nodes.iter().find(|node| node.slot_id == slot.id)
            .is_some_and(|node| !run.event_sessions.contains_key(&node.id)))
        .min_by_key(|slot| {
            let node = run.map.nodes.iter().find(|node| node.slot_id == slot.id).unwrap();
            // Preserve an existing authored difficulty when preparing a battle.
            if matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss) {
                (node.category != category, node.combat_difficulty.is_none())
            } else { (node.category != category, false) }
        })
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
    let mut current_node_id = None;
    for slot_id in path.iter().take(path.len().saturating_sub(1)) {
        let node = run
            .map
            .nodes
            .iter_mut()
            .find(|node| node.slot_id == *slot_id)
            .expect("path node");
        node.state = MapNodeState::Completed;
        node.visibility = MapNodeVisibility::Revealed;
        current_node_id = Some(node.id);
    }
    let node = run.map.node_mut(node_id).expect("node exists");
    node.category = category;
    node.kind_id = crate::game::map::MapNodeKindId::new(kind_id);
    node.payload = payload;
    node.state = MapNodeState::Available;
    node.visibility = MapNodeVisibility::Revealed;
    for other in &mut run.map.nodes {
        if other.id != node_id && other.state == MapNodeState::Available {
            other.state = MapNodeState::Unavailable;
        }
    }
    let mut completed_node_ids = run
        .map
        .nodes
        .iter()
        .filter(|node| node.state == MapNodeState::Completed)
        .map(|node| node.id)
        .collect::<Vec<_>>();
    completed_node_ids.sort_by_key(|id| id.0.as_u128());
    completed_node_ids.dedup();
    run.map_progression = crate::game::map::MapProgression {
        current_node_id,
        available_node_ids: vec![node_id],
        completed_node_ids,
    };
    run.map.validate_facility_template_contract(&template)
        .expect("explicit category fixture preserves the authored graph and slot contract");
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
    let file = std::fs::File::create(&out_path).expect("create debug event log json");
    serde_json::to_writer_pretty(std::io::BufWriter::new(file), event_log)
        .expect("write debug event log json");
    out_path
}

mod facility;

mod checkpoint_validation;
#[cfg(unix)]
mod content_generation;

mod node_sessions;
mod u06_starter;
mod u07_objects;

mod equipment;

mod combat;
mod combat_reward_quantity;

mod mastered_elite_sweep;

mod r07_copies;

mod u08_relics;

mod u08_query_owner;

mod u08_carriers;
mod u08_continue_owner;
mod u08_event_owner;
mod u08_lifecycle;

mod u09_structural;
mod u10_causal;

mod u12_lifecycle;

mod u13_facility;
mod u14_deterrence;

mod u15_economy;
mod u16_support;

mod u162_relic_rewards;

mod u163_dealer;
mod u164_stress;

fn fixture_revival_stage(encounter_id: &str, percent: u32, parts: u32) -> crate::game::revival::RevivalStagePolicy {
    crate::game::revival::RevivalStagePolicy { encounter_id: encounter_id.into(), stat_multiplier_percent: percent,
        rewards: vec![RewardEffect::GrantEquipmentMaterial { material_id: "part_big_bad_wolf".into(), amount: parts }],
        equipment_choices: vec!["standard_armor".into(), "standard_training_sword".into()] }
}

mod u17_revival;
mod u17_public;

mod continuation_contract;
