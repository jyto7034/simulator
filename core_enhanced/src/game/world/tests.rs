use super::*;
use crate::game::ability::{DeliveryDef, SkillDef, SkillId, SkillStepDef, SkillTarget};
use crate::game::battle::timeline::Timeline;
use crate::game::data::{
    abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
    artifact_data::ArtifactMetadata,
    employee_data::StarterEmployeeCandidateDatabase,
    equipment_data::{
        EquipmentDatabase, EquipmentDismantleRecipeMetadata, EquipmentEnhancementRecipeMetadata,
        EquipmentMaterialCost, EquipmentMaterialMetadata, EquipmentMaterialType, EquipmentMetadata,
        EquipmentRecipeMetadata, EquipmentRestorationRecipeMetadata, EquipmentType,
    },
    pve_data::{
        PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase, PveWaveData,
        PveWaveEnemyData,
    },
    random_event_data::{
        RandomEventDatabase, RandomEventInnerMetadata, RandomEventMetadata, RandomEventPoolMetadata,
    },
    reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata, RewardTag},
    shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType},
    skill_data::SkillDatabase,
    skill_fragment_data::{
        SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
        SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
    },
    GameDataBase, GameDataBuilder,
};
use crate::game::employee::{
    Employee, EmployeeAvailability, EmployeeGrade, EmployeeLifeState, StarterEmployeeCandidate,
};
use crate::game::enums::{RewardMode, Side};
use crate::game::map::{
    MapGenerationConfig, MapGenerator, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
    MapNodePayload, MedicalTreatmentKind, RunMap, SupportNodeMode, SupportNodeType,
};
use crate::game::resources::{
    EquipItemOutcomeDto, OwnedEquipment, SelectedEvent, ShopSessionState,
};
use crate::game::reward::{RewardEffect, RewardOption};
use crate::game::skill_fragment::starter_basic_attack_fragment_id;
use crate::game::stats::{StatId, StatModifier, StatModifierKind};
use serde_json::{json, Value};
use std::sync::Arc;

fn empty_game_data() -> Arc<GameDataBase> {
    test_game_data_builder().build_arc()
}

fn test_game_data_builder() -> GameDataBuilder {
    GameDataBuilder::empty().with_starter_employee_candidates(test_starter_candidate_database())
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
            test_abnormality_meta("recovery_risk_abno", 20_006),
            test_abnormality_meta("frontline_risk_abno", 20_007),
        ])
        .with_pve(PveEncounterDatabase::new(vec![
            PveEncounter {
                id: "low_risk_encounter".to_string(),
                abnormality_id: "low_risk_abno".to_string(),
                difficulty: 1,
                risk_level: crate::game::enums::RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Suppression),
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
                node_type: Some(crate::game::combat_preview::CombatNodeType::Suppression),
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
                node_type: Some(crate::game::combat_preview::CombatNodeType::Encirclement),
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
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("defense_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "recovery_encounter".to_string(),
                abnormality_id: "recovery_risk_abno".to_string(),
                difficulty: 3,
                risk_level: crate::game::enums::RiskLevel::HE,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Recovery),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("recovery_risk_abno")],
                static_obstacles: vec![],
            },
            PveEncounter {
                id: "frontline_encounter".to_string(),
                abnormality_id: "frontline_risk_abno".to_string(),
                difficulty: 2,
                risk_level: crate::game::enums::RiskLevel::TETH,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(crate::game::combat_preview::CombatNodeType::Frontline),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![test_pve_wave("frontline_risk_abno")],
                static_obstacles: vec![],
            },
        ]))
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
    }
}

fn test_pve_wave(abnormality_id: &str) -> PveWaveData {
    PveWaveData {
        id: "wave_0".to_string(),
        time_ms: 0,
        spawn_zone_ids: Vec::new(),
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
    let random_event_uuid = Uuid::from_u128(10_003);
    let forbidden_reward_uuid = Uuid::from_u128(10_004);
    let suppress_event_uuid = Uuid::from_u128(10_005);

    test_game_data_builder()
        .with_shops(ShopDatabase::new_with_pools(
            vec![ShopMetadata {
                id: "map_shop".to_string(),
                name: "Map Shop".to_string(),
                uuid: shop_uuid,
                shop_type: ShopType::Shop,
                can_reroll: false,
                visible_items: vec![],
                hidden_items: vec![],
            }],
            vec![ShopPoolMetadata {
                id: "default_shops".to_string(),
                shop_ids: vec!["map_shop".to_string()],
            }],
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
        .with_random_events(RandomEventDatabase::new_with_pools(
            vec![
                RandomEventMetadata {
                    id: "map_random_reward".to_string(),
                    name: "Map Random Reward".to_string(),
                    uuid: random_event_uuid,
                    event_type:
                        crate::game::events::event_selection::random::RandomEventType::Reward,
                    risk_level: crate::game::enums::RiskLevel::ZAYIN,
                    description: "A deterministic random event".to_string(),
                    image: String::new(),
                    inner_metadata: RandomEventInnerMetadata::Reward(reward_uuid),
                },
                RandomEventMetadata {
                    id: "map_suppress_event".to_string(),
                    name: "Map Suppress Event".to_string(),
                    uuid: suppress_event_uuid,
                    event_type:
                        crate::game::events::event_selection::random::RandomEventType::Suppress,
                    risk_level: crate::game::enums::RiskLevel::HE,
                    description: "Suppress event is not a live map event target.".to_string(),
                    image: String::new(),
                    inner_metadata: RandomEventInnerMetadata::Suppress(Uuid::from_u128(0xBEEF)),
                },
            ],
            vec![RandomEventPoolMetadata {
                id: "default_random_events".to_string(),
                event_ids: vec!["map_random_reward".to_string()],
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
        triggered_effects: Default::default(),
        ability_activations: vec![],
    }
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

fn game_data_with_equipment_restoration(
    abnormality: Arc<AbnormalityMetadata>,
    equipment: Vec<EquipmentMetadata>,
    materials: Vec<EquipmentMaterialMetadata>,
    restoration_recipes: Vec<EquipmentRestorationRecipeMetadata>,
) -> Arc<GameDataBase> {
    test_game_data_builder()
        .with_abnormalities(vec![(*abnormality).clone()])
        .with_equipment_data(Arc::new(
            EquipmentDatabase::with_materials_recipes_and_restorations(
                equipment,
                materials,
                vec![],
                restoration_recipes,
            ),
        ))
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
            EquipmentDatabase::with_materials_recipes_restorations_and_dismantles(
                equipment,
                materials,
                vec![],
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

        core.state.selected_event = Some(SelectedEvent::new(SelectedEventState::Shop(
            ShopSessionState {
                id: "artifact_shop".to_string(),
                name: "Artifact Merchant".to_string(),
                uuid: Uuid::from_u128(0x30),
                shop_type: crate::game::data::shop_data::ShopType::Shop,
                can_reroll: true,
                visible_items: vec![blade.uuid],
                hidden_items: vec![lens.uuid],
            },
        )));

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
        assert_eq!(required_count, RUN_SYSTEM_POLICY.starter_employee_count);
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
        assert!(allowed.contains(&ActionKind::MoveBenchUnit));
        assert!(!allowed.contains(&ActionKind::MoveUnit));
    }

    #[test]
    fn start_new_game_creates_starter_employee_roster_and_bench() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);

        start_new_game_with_default_starters(&mut core, player_id);

        let roster = core.roster().expect("employee roster exists");
        assert_eq!(roster.len(), RUN_SYSTEM_POLICY.starter_employee_count);

        let employee_ids = roster.available_employee_ids();
        assert_eq!(employee_ids.len(), RUN_SYSTEM_POLICY.starter_employee_count);
        let bench = core.bench().expect("bench exists");
        for employee_id in employee_ids {
            assert!(bench.slot_of(employee_id).is_some());
        }
    }

    #[test]
    fn move_unit_is_only_valid_for_node_combat_deployment() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_id = core
            .roster()
            .unwrap()
            .available_employee_ids()
            .first()
            .copied()
            .expect("starter employee");

        let err = core
            .execute(
                player_id,
                PlayerBehavior::MoveUnit {
                    target_unit_uuid: employee_id,
                    dest_pos: Position::new(1, 1),
                    swap_with_unit_uuid: None,
                },
            )
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn employee_roster_snapshot_exposes_status_loadout_and_bench_slot() {
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
        assert_eq!(employee["bench_slot"], 0);
        assert_eq!(employee["equipped_items"].as_array().unwrap().len(), 0);
        assert!(snapshot["available_employee_ids"]
            .as_array()
            .unwrap()
            .contains(&json!(employee_id)));
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
            RUN_SYSTEM_POLICY.default_max_acts
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
            RUN_SYSTEM_POLICY.starter_employee_count
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
        assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.default_max_acts);
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
        const RECON_NS: u64 = 0x5245_434F_4E; // "RECON"

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
                    && core.node_seed(*left, RECON_NS) % 7 != core.node_seed(*right, RECON_NS) % 7
            })
            .expect("full UUID seed mixing should vary combat archetype buckets");

        let left_seed = core.node_seed(pair.0, RECON_NS);
        let right_seed = core.node_seed(pair.1, RECON_NS);
        let game_data = empty_game_data();
        let left = crate::game::combat_preview::CombatPreview::try_generate_for_node(
            pair.0,
            MapNodeCategory::Combat,
            None,
            game_data.as_ref(),
            false,
            left_seed,
        )
        .expect("left combat preview should generate");
        let right = crate::game::combat_preview::CombatPreview::try_generate_for_node(
            pair.1,
            MapNodeCategory::Combat,
            None,
            game_data.as_ref(),
            false,
            right_seed,
        )
        .expect("right combat preview should generate");

        assert_ne!(left.archetype, right.archetype);
        assert_ne!(left.battlefield_template_id, right.battlefield_template_id);
    }

    #[test]
    fn generated_map_node_ids_feed_distinct_node_seeds() {
        const RECON_NS: u64 = 0x5245_434F_4E; // "RECON"

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
            .map(|node| core.node_seed(node.id, RECON_NS))
            .collect::<Vec<_>>();

        seeds.sort_unstable();
        seeds.dedup();

        assert!(
            seeds.len() > 1,
            "generated map combat nodes should not collapse to one preview seed"
        );
    }

    #[test]
    fn combat_node_preview_exposes_basic_briefing_without_spending_recon() {
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
            combat_deployment: Some(deployment),
            recon_charge,
            ..
        } = result
        else {
            panic!("expected combat node preview");
        };
        assert_eq!(recon_charge, 2);
        assert!(!preview.recon_revealed);
        assert_eq!(preview.node_id, node_id);
        assert_eq!(preview.encounter_id.as_deref(), Some("low_risk_encounter"));
        assert!(!preview.deployment_zones.is_empty());
        assert_eq!(deployment.node_id, node_id);
        assert!(deployment.placements.is_empty());
        assert!(!preview.spawn_zones.is_empty());
        assert!(!preview.spawn_waves.is_empty());
        assert_eq!(preview.spawn_waves[0].time_ms, 0);
        assert_eq!(core.state.run.as_ref().unwrap().recon_charge, 2);
    }

    #[test]
    fn combat_node_requires_explicit_node_deployment_before_confirm() {
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
        let err = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    }

    #[test]
    fn combat_node_confirm_revalidates_stale_node_deployment() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );

        let preview = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = preview
        else {
            panic!("expected combat preview");
        };
        core.execute(
            player_id,
            PlayerBehavior::MoveUnit {
                target_unit_uuid: employee_uuid,
                dest_pos: combat_preview.deployment_zones[0].cells[0],
                swap_with_unit_uuid: None,
            },
        )
        .unwrap();
        core.roster_mut()
            .unwrap()
            .get_mut(&employee_uuid)
            .unwrap()
            .availability = EmployeeAvailability::Unavailable;

        let err = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    }

    #[test]
    fn recon_scan_spends_charge_allows_rescan_and_persists_after_cancel() {
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
            .execute(player_id, PlayerBehavior::UseReconScan)
            .unwrap();
        let BehaviorResult::ReconScanUsed {
            remaining_recon_charge,
            combat_preview,
            ..
        } = result
        else {
            panic!("expected recon scan result");
        };
        assert_eq!(remaining_recon_charge, 1);
        assert!(combat_preview.recon_revealed);

        let result = core
            .execute(player_id, PlayerBehavior::UseReconScan)
            .unwrap();
        assert!(matches!(
            result,
            BehaviorResult::ReconScanUsed {
                remaining_recon_charge: 0,
                ..
            }
        ));
        assert!(core
            .execute(player_id, PlayerBehavior::UseReconScan)
            .is_err());

        core.execute(player_id, PlayerBehavior::CancelSelectedNode)
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        let BehaviorResult::NodePreview {
            combat_preview: Some(preview),
            recon_charge,
            ..
        } = result
        else {
            panic!("expected combat node preview");
        };
        assert_eq!(recon_charge, 0);
        assert!(preview.recon_revealed);
    }

    #[test]
    fn recon_scan_rejects_non_combat_node_without_spending_charge() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
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

        let err = core
            .execute(player_id, PlayerBehavior::UseReconScan)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert_eq!(core.state.run.as_ref().unwrap().recon_charge, 2);
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
        deploy_first_available_employee_if_combat_preview(&mut core, player_id, &preview);
        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(
            entered,
            BehaviorResult::NodeEntered {
                node_id,
                ..
            } | BehaviorResult::SupportState {
                node_id,
                ..
            } if node_id == first_node_id
        ));
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

        core.assign_map_encounters(&mut map, &run_progression);

        let normal = map.node(normal_node_id).expect("normal node exists");
        assert!(matches!(
            &normal.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if id == "low_risk_encounter"
        ));
        let elite = map.node(elite_node_id).expect("elite node exists");
        assert!(matches!(
            &elite.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if id == "elite_encirclement_encounter"
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
                    assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.default_max_acts);
                    assert!(matches!(core.get_state(), GameState::ViewingMap));
                }
                BehaviorResult::RunComplete { map } => {
                    assert_eq!(act_complete_count, RUN_SYSTEM_POLICY.default_max_acts - 1);
                    assert_eq!(map.act_index, RUN_SYSTEM_POLICY.default_max_acts - 1);
                    assert_eq!(map.max_acts, RUN_SYSTEM_POLICY.default_max_acts);
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

fn select_and_confirm_map_node(
    core: &mut GameCore,
    player_id: Uuid,
    node_id: MapNodeId,
) -> BehaviorResult {
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    assert!(matches!(preview, BehaviorResult::NodePreview { .. }));
    deploy_first_available_employee_if_combat_preview(core, player_id, &preview);
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap()
}

fn deploy_first_available_employee_if_combat_preview(
    core: &mut GameCore,
    player_id: Uuid,
    preview: &BehaviorResult,
) {
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = preview
    else {
        return;
    };
    let Some(employee_uuid) = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
    else {
        return;
    };
    let dest_pos = combat_preview.deployment_zones[0].cells[0];
    core.execute(
        player_id,
        PlayerBehavior::MoveUnit {
            target_unit_uuid: employee_uuid,
            dest_pos,
            swap_with_unit_uuid: None,
        },
    )
    .unwrap();
}

fn write_world_timeline_export(name: &str, timeline: &Timeline) -> std::path::PathBuf {
    let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("timeline_exports");
    let out_path = out_dir.join(format!("{name}.json"));
    std::fs::create_dir_all(&out_dir).expect("create timeline_exports directory");
    timeline
        .write_pretty_json(&out_path)
        .expect("write timeline json");
    out_path
}

mod system_flow {
    use super::*;
    use crate::game::battle::types::BattleWinner;

    #[test]
    fn official_run_slice_covers_starter_selection_combat_result_and_safe_node_delivery() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);

        let start = core
            .execute(player_id, PlayerBehavior::StartNewGame)
            .expect("new game should open starter selection");
        let BehaviorResult::StartNewGame {
            candidates,
            required_count,
        } = start
        else {
            panic!("expected starter candidate selection");
        };
        let candidate_ids = candidates
            .into_iter()
            .take(required_count)
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>();
        let selected = core
            .execute(
                player_id,
                PlayerBehavior::SelectStarterEmployees { candidate_ids },
            )
            .expect("starter selection should enter the run map");
        let BehaviorResult::StarterEmployeesSelected { map, .. } = selected else {
            panic!("expected run map after starter selection");
        };
        assert!(!map.available_node_ids.is_empty());
        assert!(matches!(core.get_state(), GameState::ViewingMap));

        let combat_node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );
        let safe_node_id = {
            let run = core.state.run.as_mut().unwrap();
            let safe_node_id = run
                .map
                .node(combat_node_id)
                .unwrap()
                .outgoing
                .first()
                .copied()
                .expect("combat node should unlock a next node");
            let safe_node = run.map.node_mut(safe_node_id).unwrap();
            safe_node.category = MapNodeCategory::Support;
            safe_node.kind_id = MapNodeKindId::new("support_rest");
            safe_node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };
            safe_node_id
        };

        let preview = core
            .execute(
                player_id,
                PlayerBehavior::SelectMapNode {
                    node_id: combat_node_id,
                },
            )
            .expect("combat node preview should open");
        let BehaviorResult::NodePreview {
            combat_preview: Some(combat_preview),
            ..
        } = preview
        else {
            panic!("expected combat preview");
        };
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        let deployment_cell = combat_preview.deployment_zones[0].cells[0];
        let moved = core
            .execute(
                player_id,
                PlayerBehavior::MoveUnit {
                    target_unit_uuid: employee_uuid,
                    dest_pos: deployment_cell,
                    swap_with_unit_uuid: None,
                },
            )
            .expect("combat deployment move should be accepted");
        assert!(matches!(moved, BehaviorResult::MoveUnit));

        let combat_started = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .expect("combat node should start battle");
        assert!(matches!(
            combat_started,
            BehaviorResult::CombatResolved {
                winner: BattleWinner::Player,
                ..
            }
        ));
        assert!(matches!(core.get_state(), GameState::InCombatReplay { .. }));

        let combat_finished = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .expect("combat replay should finish");
        let BehaviorResult::CombatRewardsGranted {
            outcome,
            completion,
            ..
        } = combat_finished
        else {
            panic!("expected combat rewards and node completion");
        };
        assert!(outcome.mission_success);
        assert_eq!(outcome.node_id, combat_node_id);
        assert_eq!(outcome.combat.unwrap().winner, BattleWinner::Player);
        assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .available_node_ids
            .contains(&safe_node_id));

        let fragment_id = starter_basic_attack_fragment_id();
        let starting_count = core.state.skill_fragments.count(&fragment_id);
        core.state
            .skill_fragments
            .add_research_progress(&fragment_id, 100)
            .expect("research completion should be queued for safe node delivery");

        core.execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: safe_node_id,
            },
        )
        .expect("safe node preview should open");
        let safe_entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .expect("safe node should be entered");
        let BehaviorResult::SupportState {
            research_deliveries,
            ..
        } = safe_entered
        else {
            panic!("expected support state with research deliveries");
        };
        assert_eq!(research_deliveries.len(), 1);
        assert_eq!(research_deliveries[0].fragment_id, fragment_id);
        assert_eq!(research_deliveries[0].total_count, starting_count + 1);

        let completed = core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .expect("safe node completion should return to map");
        let BehaviorResult::NodeCompleted { map, .. } = completed else {
            panic!("expected safe node completion");
        };
        assert!(map.completed_node_ids.contains(&combat_node_id));
        assert!(map.completed_node_ids.contains(&safe_node_id));
        assert!(!map.available_node_ids.contains(&safe_node_id));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
    }
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
        assert!(core.state.selected_event.is_none());
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
        assert!(core.state.selected_event.is_none());
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
                SupportNodeType::Maintenance,
            ]
        ));
    }
}

mod node_sessions {
    use super::*;

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

    #[test]
    fn random_map_node_routes_resolved_reward_event_directly_to_reward() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Event,
            "event_random",
            MapNodePayload::Event {
                event_id: Some("map_random_reward".to_string()),
                event_pool_id: None,
            },
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);
        assert!(matches!(result, BehaviorResult::RewardState { .. }));
        assert!(matches!(core.get_state(), GameState::InReward { .. }));
        core.execute(player_id, PlayerBehavior::ClaimReward)
            .unwrap();
        let result = core.execute(player_id, PlayerBehavior::ExitReward).unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
    }

    #[test]
    fn event_map_node_uses_declared_pool_without_suppress_targets() {
        let mut core = GameCore::new(game_data_with_map_content(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Event,
            "event_random",
            MapNodePayload::Event {
                event_id: None,
                event_pool_id: Some("default_random_events".to_string()),
            },
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);

        assert!(matches!(result, BehaviorResult::RewardState { .. }));
        assert!(matches!(core.get_state(), GameState::InReward { .. }));
    }

    #[test]
    fn game_core_uses_metagame_field_dimensions() {
        let core = GameCore::new(empty_game_data(), 123);
        let field = core.field().unwrap();

        assert_eq!(field.width, METAGAME_FIELD_WIDTH);
        assert_eq!(field.height, METAGAME_FIELD_HEIGHT);
    }
}

mod equipment {
    use super::*;

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
    fn restore_equipment_is_maintenance_only_and_consumes_material_stacks() {
        let unit_meta = abnormality_meta(1);
        let restored = equipment_meta(30, "restored_weapon", EquipmentType::Weapon);
        let material = EquipmentMaterialMetadata {
            id: "test_weapon_fragment".to_string(),
            uuid: Uuid::from_u128(31),
            name: "Test Weapon Fragment".to_string(),
            description: "A test restoration material".to_string(),
            material_type: EquipmentMaterialType::Fragment,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            equipment_type: Some(EquipmentType::Weapon),
        };
        let recipe = EquipmentRestorationRecipeMetadata {
            id: "restore_test_weapon".to_string(),
            result_equipment_id: restored.id.clone(),
            costs: vec![EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 3,
            }],
        };
        let game_data = game_data_with_equipment_restoration(
            Arc::clone(&unit_meta),
            vec![restored.clone()],
            vec![material.clone()],
            vec![recipe.clone()],
        );
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.inventory_mut()
            .unwrap()
            .equipment_materials
            .add(&material.id, 3)
            .unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::RestoreEquipment {
                    recipe_id: recipe.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::RestoreEquipment));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::RestoreEquipment {
                    recipe_id: recipe.id.clone(),
                },
            )
            .unwrap();

        let BehaviorResult::EquipmentRestored {
            recipe_id,
            result_equipment_id,
            inventory_diff,
        } = result
        else {
            panic!("expected equipment restoration result");
        };
        assert_eq!(recipe_id, recipe.id);
        assert_eq!(result_equipment_id, restored.id);
        assert_eq!(inventory_diff.added.len(), 1);
        assert_eq!(inventory_diff.material_stacks.len(), 1);
        assert_eq!(inventory_diff.material_stacks[0].material_id, material.id);
        assert_eq!(inventory_diff.material_stacks[0].amount, 0);
        assert_eq!(
            core.inventory()
                .unwrap()
                .equipment_materials
                .amount(&material.id),
            0
        );
        assert!(core
            .inventory()
            .unwrap()
            .equipments
            .iter()
            .any(|owned| owned.meta.id == restored.id));
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
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
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
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
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
    fn skill_fragment_upgrade_consumes_same_rarity_material_and_updates_progress() {
        let target = active_skill_fragment("upgrade_active_fragment", 0xF00F, "upgrade_skill");
        let material = active_skill_fragment("upgrade_material_fragment", 0xF010, "material_skill");
        let game_data = game_data_with_skill_fragments(vec![target.clone(), material.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.state.skill_fragments.add(&target).unwrap();
        core.state.skill_fragments.add(&material).unwrap();
        core.state.skill_fragments.add(&material).unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::UpgradeSkillFragment {
                    target_fragment_id: target.id.clone(),
                    material_fragment_id: material.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::UpgradeSkillFragment));

        let result = core
            .execute(
                player_id,
                PlayerBehavior::UpgradeSkillFragment {
                    target_fragment_id: target.id.clone(),
                    material_fragment_id: material.id.clone(),
                },
            )
            .unwrap();

        let BehaviorResult::SkillFragmentUpgraded {
            target_fragment_id,
            material_fragment_id,
            material_remaining_count,
            progress,
        } = result
        else {
            panic!("expected skill fragment upgrade result");
        };
        assert_eq!(target_fragment_id, target.id);
        assert_eq!(material_fragment_id, material.id);
        assert_eq!(material_remaining_count, 1);
        assert_eq!(progress.upgrade_level, 1);
        assert_eq!(progress.awakening_progress, 1);
        assert_eq!(core.state.skill_fragments.count(&target.id), 1);
        assert_eq!(core.state.skill_fragments.count(&material.id), 1);
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
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
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
    fn skill_fragment_dismantle_rejects_equipped_or_last_copy() {
        let fragment =
            active_skill_fragment("equipped_dismantle_fragment", 0xF012, "equipped_skill");
        let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
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
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        );
        core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn skill_fragment_dismantle_rejects_last_copy() {
        let fragment =
            active_skill_fragment("last_copy_dismantle_fragment", 0xF013, "last_copy_skill");
        let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
        let mut core = GameCore::new(game_data, 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        core.state.skill_fragments.add(&fragment).unwrap();
        core.state.skill_fragments.add(&fragment).unwrap();

        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Support,
            "support_maintenance",
            MapNodePayload::Support {
                support_type: SupportNodeType::Maintenance,
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
            PlayerBehavior::DismantleSkillFragment {
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

        let err = core
            .execute(
                player_id,
                PlayerBehavior::DismantleSkillFragment {
                    fragment_id: fragment.id.clone(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));
        assert_eq!(core.state.skill_fragments.count(&fragment.id), 1);
    }
}

mod combat {
    use super::*;
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

    fn start_forced_map_combat(
        core: &mut GameCore,
        player_id: Uuid,
        category: MapNodeCategory,
        kind_id: &str,
        encounter_id: &str,
    ) -> MapNodeId {
        let node_id = force_map_combat_node(core, category, kind_id, encounter_id);
        let result = select_and_confirm_map_node(core, player_id, node_id);
        assert!(matches!(result, BehaviorResult::CombatResolved { .. }));
        assert!(core.state.node_session.is_some());
        node_id
    }

    fn first_player_participant(core: &GameCore) -> ParticipantBattleResult {
        let roster = core.roster().expect("roster");
        core.state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .participant_results
            .iter()
            .find(|participant| {
                participant.side == Side::Player && roster.get(&participant.owned_uuid).is_some()
            })
            .cloned()
            .expect("player participant")
    }

    fn set_active_battle_result_for_failed_replay_test(core: &mut GameCore) -> Uuid {
        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        let mut participant = first_player_participant(core);
        participant.survived = false;
        participant.final_hp = 0;
        participant.became_incapacitated = true;
        let employee_uuid = participant.owned_uuid;
        battle.winner = BattleWinner::Opponent;
        battle.participant_results = vec![participant];
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));
        employee_uuid
    }

    #[test]
    fn combat_replay_without_node_session_is_rejected() {
        let mut core = GameCore::new(empty_game_data(), 123);
        let abnormality_uuid = Uuid::from_u128(0xBEEF);

        core.transition_to(GameState::InCombatReplay {
            battle_uuid: abnormality_uuid,
        })
        .unwrap();
        core.state.selected_event = Some(SelectedEvent::new(SelectedEventState::CombatBattle(
            CombatBattleState {
                abnormality_id: "abno".to_string(),
                encounter_id: "encounter".to_string(),
                node_type: crate::game::combat_preview::CombatNodeType::Suppression,
                abnormality_uuid,
                winner: BattleWinner::Opponent,
                timeline: Timeline::default(),
                reward_mode: RewardMode::ChooseOne,
                rewards: vec![],
                participant_results: vec![],
            },
        )));

        let err = core.handle_finish_combat_replay().unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));

        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Player;
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let err = core.handle_finish_combat_replay().unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn failed_combat_opens_outgoing_medical_before_no_deployable_run_failure() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_low_risk",
            "low_risk_encounter",
        );
        let medical_node_id = {
            let run = core.state.run.as_mut().unwrap();
            let next_node_id = run
                .map
                .node(node_id)
                .unwrap()
                .outgoing
                .first()
                .copied()
                .expect("combat node should have outgoing node");
            let next_node = run.map.node_mut(next_node_id).unwrap();
            next_node.category = MapNodeCategory::Support;
            next_node.kind_id = crate::game::map::MapNodeKindId::new("support_medical");
            next_node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Medical,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };
            next_node_id
        };
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.availability = EmployeeAvailability::Unavailable;
            }
        }
        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Opponent;
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        let BehaviorResult::NodeCompleted {
            outcome: Some(outcome),
            ..
        } = result
        else {
            panic!("expected failed combat node completion with outcome summary");
        };
        assert!(!outcome.mission_success);
        assert_eq!(outcome.node_id, node_id);
        assert_eq!(outcome.category, MapNodeCategory::Combat);
        assert_eq!(outcome.combat.unwrap().winner, BattleWinner::Opponent);
        assert!(outcome.inventory_diff.added.is_empty());
        assert!(outcome.research_deliveries.is_empty());
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        let run = core.state.run.as_ref().unwrap();
        assert!(run
            .map_progression
            .available_node_ids
            .contains(&medical_node_id));
        assert!(run
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
    }

    #[test]
    fn failed_combat_without_outgoing_medical_fails_when_no_deployable_employee_remains() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_low_risk",
            "low_risk_encounter",
        );
        {
            let run = core.state.run.as_mut().unwrap();
            let next_node_id = run
                .map
                .node(node_id)
                .unwrap()
                .outgoing
                .first()
                .copied()
                .expect("combat node should have outgoing node");
            let next_node = run.map.node_mut(next_node_id).unwrap();
            next_node.category = MapNodeCategory::Combat;
            next_node.kind_id = crate::game::map::MapNodeKindId::new("combat_low_risk");
            next_node.payload = MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            };
        }
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.availability = EmployeeAvailability::Unavailable;
            }
        }
        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Opponent;
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        let BehaviorResult::RunFailed {
            reason: RunFailureReason::NoDeployableEmployees,
            outcome: Some(outcome),
        } = result
        else {
            panic!("expected no-deployable run failure with combat outcome summary");
        };
        assert!(!outcome.mission_success);
        assert_eq!(outcome.node_id, node_id);
        assert_eq!(outcome.combat.unwrap().winner, BattleWinner::Opponent);
        assert!(matches!(
            core.get_state(),
            GameState::RunFailed {
                reason: RunFailureReason::NoDeployableEmployees
            }
        ));
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
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
            RUN_SYSTEM_POLICY.post_battle_incapacitation_trauma
        );
        assert_eq!(employee.injuries.len(), 1);
        assert_eq!(employee.injuries[0].id, "battle_incapacitation");
        assert_eq!(
            employee.life_state,
            crate::game::employee::EmployeeLifeState::Alive
        );
    }

    #[test]
    fn repeated_post_battle_incapacitation_can_kill_employee_and_remove_from_bench_pool() {
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
        let bench = core.bench().unwrap();
        assert!(bench.slot_of(employee_uuid).is_none());
    }

    #[test]
    fn combat_replay_fails_run_when_no_living_employee_remains() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_low_risk",
            "low_risk_encounter",
        );
        let employee_ids = core.roster().unwrap().available_employee_ids();
        assert!(!employee_ids.is_empty());
        {
            let roster = core.roster_mut().unwrap();
            for employee_id in &employee_ids {
                let employee = roster.get_mut(employee_id).unwrap();
                employee.trauma = Employee::TRAUMA_DEATH_THRESHOLD - 1;
            }
        }

        let participant_results = employee_ids
            .iter()
            .enumerate()
            .map(|(index, employee_id)| ParticipantBattleResult {
                unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                    0xCAFE + index as u128,
                )),
                owned_uuid: *employee_id,
                side: Side::Player,
                survived: false,
                final_hp: 0,
                max_hp: 30,
                became_incapacitated: true,
            })
            .collect::<Vec<_>>();

        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Opponent;
        battle.participant_results = participant_results;
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        assert!(matches!(
            result,
            BehaviorResult::RunFailed {
                reason: RunFailureReason::NoLivingEmployees,
                ..
            }
        ));
        assert!(matches!(
            core.get_state(),
            GameState::RunFailed {
                reason: RunFailureReason::NoLivingEmployees
            }
        ));
        assert!(core.get_allowed_actions().is_empty());
        assert!(core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .is_err());
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
        core.sync_bench_with_owned_units().unwrap();
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
        core.sync_bench_with_owned_units().unwrap();
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

        assert!(matches!(
            result,
            BehaviorResult::RunFailed {
                reason: RunFailureReason::NoDeployableEmployees,
                ..
            }
        ));
    }

    #[test]
    fn combat_map_node_is_blocked_when_recovery_node_is_still_available() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        {
            let roster = core.roster_mut().unwrap();
            for employee in roster.iter_mut() {
                employee.health.set_current_hp(0);
            }
        }
        core.sync_bench_with_owned_units().unwrap();

        let available_nodes = core
            .state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .available_node_ids
            .clone();
        assert!(available_nodes.len() >= 2);
        let recovery_node_id = available_nodes[0];
        let combat_node_id = available_nodes[1];
        {
            let map = &mut core.state.run.as_mut().unwrap().map;
            let recovery_node = map.node_mut(recovery_node_id).unwrap();
            recovery_node.category = MapNodeCategory::Support;
            recovery_node.kind_id = crate::game::map::MapNodeKindId::new("support_medical");
            recovery_node.payload = MapNodePayload::Support {
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
            .contains(&recovery_node_id));
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

        let result = select_and_confirm_map_node(&mut core, player_id, recovery_node_id);
        assert!(matches!(result, BehaviorResult::SupportState { .. }));
    }

    #[test]
    fn combat_map_node_uses_run_hp_as_battle_start_hp_without_writing_survivor_battle_hp_back() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
        {
            let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
            employee.health.set_current_hp(40);
        }
        let node_id = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            MapNodePayload::Encounter {
                encounter_id: Some("low_risk_encounter".to_string()),
            },
        );

        select_and_confirm_map_node(&mut core, player_id, node_id);
        let participant = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .participant_results
            .iter()
            .find(|participant| participant.owned_uuid == employee_uuid)
            .expect("employee participant");
        assert_eq!(participant.final_hp, 40);

        core.apply_post_battle_resolution(&[ParticipantBattleResult {
            unit_instance_id: participant.unit_instance_id,
            owned_uuid: employee_uuid,
            side: Side::Player,
            survived: true,
            final_hp: 1,
            max_hp: participant.max_hp,
            became_incapacitated: false,
        }])
        .unwrap();
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee.health.current_hp, 40);
    }

    #[test]
    fn combat_map_node_smoke_completes_battle_reward_and_map_progression() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_low_risk",
            "low_risk_encounter",
        );

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);
        assert!(matches!(
            result,
            BehaviorResult::CombatResolved {
                winner: BattleWinner::Player,
                ..
            }
        ));
        assert!(matches!(
            core.get_state(),
            GameState::InCombatReplay {
                battle_uuid
            } if battle_uuid == node_id.0
        ));
        let battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap();
        assert_eq!(
            battle.node_type,
            crate::game::combat_preview::CombatNodeType::Suppression
        );
        let selected_snapshot = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(selected_snapshot["node_type"], json!("Suppression"));
        assert!(core.state.node_session.is_some());

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();
        let BehaviorResult::CombatRewardsGranted {
            outcome,
            completion,
            ..
        } = result
        else {
            panic!("expected combat reward result");
        };
        assert!(outcome.mission_success);
        assert_eq!(outcome.node_id, node_id);
        assert_eq!(outcome.category, MapNodeCategory::Combat);
        assert_eq!(outcome.combat.unwrap().winner, BattleWinner::Player);
        assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
        assert!(core.state.selected_event.is_none());
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
    }

    #[test]
    fn combat_experience_reward_applies_to_surviving_participants_and_outcome() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_low_risk",
            "low_risk_encounter",
        );
        let participant = first_player_participant(&core);
        let employee_uuid = participant.owned_uuid;
        let starting_experience = core
            .roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .experience;
        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.reward_mode = RewardMode::ClaimAll;
        battle.rewards = vec![RewardOption {
            id: "combat_xp_reward".to_string(),
            uuid: Uuid::from_u128(0xC0_0001),
            name: "Combat XP".to_string(),
            description: "test xp reward".to_string(),
            icon: "xp".to_string(),
            tags: vec![],
            effects: vec![RewardEffect::GrantExperience { amount: 20 }],
        }];
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        let BehaviorResult::CombatRewardsGranted { outcome, .. } = result else {
            panic!("expected combat rewards");
        };
        let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(
            employee.experience,
            starting_experience + RUN_SYSTEM_POLICY.post_battle_survival_xp + 20
        );
        let change = outcome
            .employee_changes
            .iter()
            .find(|change| change.employee_uuid == employee_uuid)
            .expect("employee change should be summarized");
        assert_eq!(change.experience_before, starting_experience);
        assert_eq!(change.experience_after, employee.experience);
    }

    #[test]
    fn finish_failed_defense_replay_consumes_node_and_returns_to_map_without_run_failure() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        );
        assert_eq!(
            core.state
                .selected_event
                .as_ref()
                .unwrap()
                .as_combat_battle()
                .unwrap()
                .node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );

        let incapacitated_employee = set_active_battle_result_for_failed_replay_test(&mut core);
        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.node_session.is_none());
        assert!(core.state.selected_event.is_none());
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
        let employee = core.roster().unwrap().get(&incapacitated_employee).unwrap();
        assert_eq!(
            employee.trauma,
            RUN_SYSTEM_POLICY.post_battle_incapacitation_trauma
        );
        assert!(employee.health.current_hp < employee.health.max_hp);
    }

    #[test]
    fn finish_failed_recovery_replay_consumes_node_and_returns_to_map_without_run_failure() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Combat,
            "combat_recovery",
            "recovery_encounter",
        );
        assert_eq!(
            core.state
                .selected_event
                .as_ref()
                .unwrap()
                .as_combat_battle()
                .unwrap()
                .node_type,
            crate::game::combat_preview::CombatNodeType::Recovery
        );

        set_active_battle_result_for_failed_replay_test(&mut core);
        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
    }

    #[test]
    fn finish_boss_replay_with_party_wipe_fails_run() {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        start_forced_map_combat(
            &mut core,
            player_id,
            MapNodeCategory::Boss,
            "boss_abnormality",
            "boss_risk_encounter",
        );
        let employee_ids = core.roster().unwrap().available_employee_ids();
        assert!(!employee_ids.is_empty());
        {
            let roster = core.roster_mut().unwrap();
            for employee_id in &employee_ids {
                let employee = roster.get_mut(employee_id).unwrap();
                employee.trauma = Employee::TRAUMA_DEATH_THRESHOLD - 1;
            }
        }
        let mut battle = core
            .state
            .selected_event
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap()
            .clone();
        battle.winner = BattleWinner::Opponent;
        battle.participant_results = employee_ids
            .iter()
            .enumerate()
            .map(|(index, employee_id)| ParticipantBattleResult {
                unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                    0xB055 + index as u128,
                )),
                owned_uuid: *employee_id,
                side: Side::Player,
                survived: false,
                final_hp: 0,
                max_hp: 30,
                became_incapacitated: true,
            })
            .collect();
        core.state.selected_event =
            Some(SelectedEvent::new(SelectedEventState::CombatBattle(battle)));

        let result = core
            .execute(player_id, PlayerBehavior::FinishCombatReplay)
            .unwrap();

        let BehaviorResult::RunFailed {
            reason: RunFailureReason::BossDefeated,
            outcome: Some(outcome),
        } = result
        else {
            panic!("expected boss run failure with combat outcome summary");
        };
        assert!(!outcome.mission_success);
        assert_eq!(outcome.combat.unwrap().winner, BattleWinner::Opponent);
        assert!(matches!(
            core.get_state(),
            GameState::RunFailed {
                reason: RunFailureReason::BossDefeated
            }
        ));
    }

    #[test]
    fn map_combat_node_smoke_exports_timeline() {
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

        let result = select_and_confirm_map_node(&mut core, player_id, node_id);
        let BehaviorResult::CombatResolved { winner, timeline } = result else {
            panic!("expected combat resolved timeline");
        };

        assert_eq!(winner, BattleWinner::Player);
        assert_eq!(
            timeline.version,
            crate::game::battle::timeline::TIMELINE_VERSION
        );
        assert!(timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleStart { .. }
        )));
        assert!(timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::UnitSpawned { .. }
        )));
        assert!(timeline.entries.iter().any(|entry| matches!(
            entry.event,
            crate::game::battle::timeline::TimelineEvent::BattleEnd { .. }
        )));

        let path = write_world_timeline_export("map_combat_node_smoke", &timeline);
        println!("wrote timeline: {}", path.display());
        assert!(path.exists());
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
    fn move_unit_rejects_run_persistent_field_placement() {
        let (mut core, employee_ids) = core_with_starter_employee_ids();
        let owned_uuid = employee_ids[0];

        let err = core
            .execute(
                Uuid::from_u128(1),
                PlayerBehavior::MoveUnit {
                    target_unit_uuid: owned_uuid,
                    dest_pos: Position::new(1, 1),
                    swap_with_unit_uuid: None,
                },
            )
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        let field = core.field().unwrap();
        assert_eq!(field.get_position(owned_uuid), None);
    }

    #[test]
    fn move_bench_unit_reorders_and_swaps_bench_slots() {
        let (mut core, employee_ids) = core_with_starter_employee_ids();
        let left_uuid = employee_ids[0];
        let right_uuid = employee_ids[1];

        let result = core
            .execute(
                Uuid::from_u128(1),
                PlayerBehavior::MoveBenchUnit {
                    target_unit_uuid: left_uuid,
                    dest_slot: 1,
                    swap_with_unit_uuid: Some(right_uuid),
                },
            )
            .unwrap();

        assert!(matches!(result, BehaviorResult::MoveBenchUnit));
        let bench = core.bench().unwrap();
        assert_eq!(bench.slot_of(left_uuid), Some(1));
        assert_eq!(bench.slot_of(right_uuid), Some(0));
    }
}
