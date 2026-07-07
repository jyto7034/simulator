use std::{collections::HashSet, sync::Arc};

use game_core::{
    game::resources::Position,
    game::{
        battle::{
            scenario::{
                BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
                ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef,
                ScenarioUnitSpawn, WinCondition,
            },
            tile_range::TileRangePattern,
            types::{BattleUnitDraft, BattleUnitSource},
        },
        data::{
            abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef},
            GameDataBase, GameDataBuilder,
        },
        enums::{RiskLevel, Side, Tier},
        growth::GrowthStack,
    },
};
use uuid::Uuid;

use super::{BoardScenario, PlacedUnitKind, RuntimeStartPatch, UnitSeed};

pub(crate) const SKILL_DUMMY_UUID: Uuid =
    Uuid::from_u128(0xD00D_0000_0000_0000_0000_0000_0000_0001);
const OVERRIDDEN_DUMMY_UUID_NS: u128 = 0xD00D_1000_0000_0000_0000_0000_0000_0000;
const OVERRIDDEN_ABNORMALITY_UUID_NS: u128 = 0xA880_1000_0000_0000_0000_0000_0000_0000;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedUnit {
    pub(crate) position: Position,
    pub(crate) base_uuid: Uuid,
    pub(crate) runtime_patch: RuntimeStartPatch,
}

pub(crate) struct ResolvedScenario {
    pub(crate) abnormality: AbnormalityMetadata,
    pub(crate) game_data: Arc<GameDataBase>,
    pub(crate) battle_scenario: BattleScenario,
    pub(crate) runtime_patches: Vec<(Position, RuntimeStartPatch)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioUnitRole {
    TargetCaster,
    Other,
}

fn broad_defense_tile_range() -> TileRangePattern {
    TileRangePattern {
        include_anchor_tile: false,
        rows: vec![
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXX@XXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
        ],
    }
}

fn apply_skill_test_basic_attack_contract(abnormality: &mut AbnormalityMetadata) {
    abnormality.basic_attack.defense_tile_range = Some(broad_defense_tile_range());
}

pub(crate) fn skill_test_dummy_metadata() -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: "skill_test_dummy".to_string(),
        uuid: SKILL_DUMMY_UUID,
        name: "Skill Test Dummy".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 2_000,
        attack: 10,
        defense: 0,
        magic_resist: 0,
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: Some(
            game_core::game::data::skill_fragment_data::SkillFragmentId::from(
                "starter_basic_attack_enhancement",
            ),
        ),
        movement: MovementDef {
            speed_units_per_ms: 0,
            radius_units: 350_000,
        },
        basic_attack: BasicAttackDef {
            range_units: 1.0,
            defense_tile_range: Some(broad_defense_tile_range()),
            interval_ms: 500,
            windup_ms: 0,
            delivery: game_core::game::ability::DeliveryDef::Instant,
            ..BasicAttackDef::default()
        },
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn make_overridden_dummy_uuid(position: Position) -> Uuid {
    let x = position.x as i128 as u128;
    let y = position.y as i128 as u128;
    Uuid::from_u128(OVERRIDDEN_DUMMY_UUID_NS | ((x & 0xFFFF) << 16) | (y & 0xFFFF))
}

fn make_overridden_abnormality_uuid(base_uuid: Uuid, position: Position) -> Uuid {
    let x = position.x as i128 as u128;
    let y = position.y as i128 as u128;
    let seed = base_uuid.as_u128().rotate_left(19) ^ ((x & 0xFFFF) << 16) ^ (y & 0xFFFF);
    Uuid::from_u128(OVERRIDDEN_ABNORMALITY_UUID_NS ^ seed)
}

fn default_test_resonance_start(abnormality: &AbnormalityMetadata, role: ScenarioUnitRole) -> u32 {
    match role {
        ScenarioUnitRole::TargetCaster if abnormality.skill_id.is_some() => {
            let max = abnormality.resonance.max.max(1);
            max.saturating_sub(10)
        }
        ScenarioUnitRole::TargetCaster | ScenarioUnitRole::Other => 0,
    }
}

fn resolve_metadata_by_kind(
    game_data: &GameDataBase,
    kind: &PlacedUnitKind,
) -> AbnormalityMetadata {
    match kind {
        PlacedUnitKind::SkillDummy => skill_test_dummy_metadata(),
        PlacedUnitKind::BaseUuid(uuid) => game_data
            .abnormality_data
            .items
            .iter()
            .find(|abnormality| abnormality.uuid == *uuid)
            .cloned()
            .unwrap_or_else(|| panic!("missing abnormality data for base uuid {uuid}")),
        PlacedUnitKind::Custom(abnormality) => abnormality.clone(),
        PlacedUnitKind::Abnormality(id) => game_data
            .abnormality_data
            .items
            .iter()
            .find(|abnormality| abnormality.id == *id)
            .cloned()
            .unwrap_or_else(|| panic!("missing abnormality data for scenario unit {id}")),
    }
}

fn materialize_unit_metadata(
    game_data: &GameDataBase,
    unit: &UnitSeed,
    role: ScenarioUnitRole,
) -> (AbnormalityMetadata, bool) {
    let original = resolve_metadata_by_kind(game_data, &unit.kind);
    let mut abnormality = original.clone();
    let has_static_override = !unit.patch.static_patch.is_empty();
    let explicit_resonance_override = unit.patch.static_patch.resonance.is_some();

    unit.patch.static_patch.apply_to_metadata(&mut abnormality);
    apply_skill_test_basic_attack_contract(&mut abnormality);

    let desired_resonance_start = if explicit_resonance_override {
        abnormality.resonance.start
    } else {
        default_test_resonance_start(&abnormality, role)
    };
    abnormality.resonance.start = desired_resonance_start;

    match &unit.kind {
        PlacedUnitKind::SkillDummy if has_static_override => {
            abnormality.uuid = make_overridden_dummy_uuid(unit.position);
            abnormality.id = format!("skill_test_dummy_{}_{}", unit.position.x, unit.position.y);
            abnormality.name = format!(
                "Skill Test Dummy ({}, {})",
                unit.position.x, unit.position.y
            );
        }
        PlacedUnitKind::SkillDummy => {
            abnormality.uuid = SKILL_DUMMY_UUID;
        }
        PlacedUnitKind::Abnormality(_) | PlacedUnitKind::BaseUuid(_) if has_static_override => {
            abnormality.uuid = make_overridden_abnormality_uuid(abnormality.uuid, unit.position);
            abnormality.id = format!(
                "{}_override_{}_{}",
                abnormality.id, unit.position.x, unit.position.y
            );
            abnormality.name = format!(
                "{} [{}, {}]",
                abnormality.name, unit.position.x, unit.position.y
            );
        }
        PlacedUnitKind::Custom(_)
        | PlacedUnitKind::Abnormality(_)
        | PlacedUnitKind::BaseUuid(_) => {}
    }

    let should_append_metadata = has_static_override
        || matches!(unit.kind, PlacedUnitKind::Custom(_))
        || original.resonance.start != desired_resonance_start;

    (abnormality, should_append_metadata)
}

fn build_game_data_with_units(
    base: Arc<GameDataBase>,
    metadata_overrides: &[AbnormalityMetadata],
) -> Arc<GameDataBase> {
    let mut abnormalities = base.abnormality_data.items.clone();
    for abnormality in &mut abnormalities {
        apply_skill_test_basic_attack_contract(abnormality);
    }
    let mut known_uuids: HashSet<Uuid> = abnormalities.iter().map(|meta| meta.uuid).collect();

    if known_uuids.insert(SKILL_DUMMY_UUID) {
        abnormalities.push(skill_test_dummy_metadata());
    }

    for abnormality in metadata_overrides {
        if let Some(existing) = abnormalities
            .iter_mut()
            .find(|existing| existing.uuid == abnormality.uuid)
        {
            *existing = abnormality.clone();
            continue;
        }

        if known_uuids.insert(abnormality.uuid) {
            abnormalities.push(abnormality.clone());
        }
    }

    GameDataBuilder::empty()
        .with_abnormalities(abnormalities)
        .with_corroded_employee_data(Arc::clone(&base.corroded_employee_data))
        .with_corroded_wave_data(Arc::clone(&base.corroded_wave_data))
        .with_artifact_data(Arc::clone(&base.artifact_data))
        .with_equipment_data(Arc::clone(&base.equipment_data))
        .with_shop_data(Arc::clone(&base.shop_data))
        .with_reward_data(Arc::clone(&base.reward_data))
        .with_event_data(Arc::clone(&base.event_data))
        .with_pve_data(Arc::clone(&base.pve_data))
        .with_boss_omen_data(Arc::clone(&base.boss_omen_data))
        .with_buff_data(Arc::clone(&base.buff_data))
        .with_skill_data(Arc::clone(&base.skill_data))
        .with_skill_fragment_data(Arc::clone(&base.skill_fragment_data))
        .build_arc()
}

fn build_spawn_group(
    group_id: &str,
    side: Side,
    required_for_victory: bool,
    units: &[ResolvedUnit],
    owned_seed: u128,
) -> ScenarioSpawnGroup {
    let group_id = ScenarioGroupId::new(group_id);
    let spawns = units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            let owned_uuid = Uuid::from_u128(owned_seed + index as u128);
            ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side,
                draft: BattleUnitDraft {
                    owned_uuid,
                    source: BattleUnitSource::Abnormality {
                        base_uuid: unit.base_uuid,
                    },
                    threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
                    level: Tier::I,
                    stat_scale: Default::default(),
                    growth_stacks: GrowthStack::new(),
                    equipped_items: vec![],
                    equipped_item_enhancements: vec![],
                },
                position: unit.position,
                instance_salt: index as u32,
            }
        })
        .collect();

    ScenarioSpawnGroup {
        id: group_id,
        side,
        required_for_victory,
        enemy_movement_plan: None,
        spawns,
    }
}

fn build_battle_scenario(
    player_units: &[ResolvedUnit],
    opponent_units: &[ResolvedUnit],
) -> BattleScenario {
    let player_group_id = "player_initial";
    let enemy_group_id = "enemy_initial";
    let groups = vec![
        build_spawn_group(
            player_group_id,
            Side::Player,
            false,
            player_units,
            0xAAA0_0000_0000_0000_0000_0000_0000_0000,
        ),
        build_spawn_group(
            enemy_group_id,
            Side::Opponent,
            true,
            opponent_units,
            0xBBB0_0000_0000_0000_0000_0000_0000_0000,
        ),
    ];

    BattleScenario {
        battlefield: BattleFieldSpec {
            width: crate::common::BOARD_SIZE.0,
            height: crate::common::BOARD_SIZE.1,
            valid_tiles: Vec::new(),
            obstacles: Vec::new(),
        },
        artifacts: Vec::new(),
        groups,
        events: vec![
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_player_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(player_group_id),
                },
                once: true,
            },
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_enemy_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(enemy_group_id),
                },
                once: true,
            },
        ],
        win_condition: WinCondition::AllRequiredEnemyGroupsDefeated,
        tactical_plan: game_core::game::battle::scenario::TacticalPlan::default(),
    }
}

fn resolve_unit(
    game_data: &GameDataBase,
    unit: &UnitSeed,
    role: ScenarioUnitRole,
) -> (ResolvedUnit, Option<AbnormalityMetadata>) {
    let (metadata, should_append_metadata) = materialize_unit_metadata(game_data, unit, role);

    (
        ResolvedUnit {
            position: unit.position,
            base_uuid: metadata.uuid,
            runtime_patch: unit.patch.runtime_start.clone(),
        },
        should_append_metadata.then_some(metadata),
    )
}

pub(crate) fn resolve_abnormality_scenario(
    base_game_data: Arc<GameDataBase>,
    abnormality_id: &str,
    board: &BoardScenario,
) -> ResolvedScenario {
    let abnormality = base_game_data
        .abnormality_data
        .items
        .iter()
        .find(|abnormality| abnormality.id == abnormality_id)
        .cloned()
        .unwrap_or_else(|| panic!("missing abnormality data for {abnormality_id}"));
    assert_ne!(
        abnormality.uuid, SKILL_DUMMY_UUID,
        "test target cannot be the dummy abnormality"
    );

    let caster_seed = UnitSeed {
        kind: PlacedUnitKind::BaseUuid(abnormality.uuid),
        position: board.caster.position,
        raw_token: board.caster.raw_token.clone(),
        unit_symbol: board.caster.unit_symbol,
        patch: board.caster.patch.clone(),
    };

    let mut metadata_overrides = Vec::new();
    let (resolved_caster, caster_metadata) = resolve_unit(
        base_game_data.as_ref(),
        &caster_seed,
        ScenarioUnitRole::TargetCaster,
    );
    if let Some(metadata) = caster_metadata {
        metadata_overrides.push(metadata);
    }

    let mut resolved_player_units = vec![resolved_caster];
    for unit in &board.player_support_units {
        let (resolved, metadata) =
            resolve_unit(base_game_data.as_ref(), unit, ScenarioUnitRole::Other);
        resolved_player_units.push(resolved);
        if let Some(metadata) = metadata {
            metadata_overrides.push(metadata);
        }
    }

    let mut resolved_opponent_units = Vec::with_capacity(board.opponent_units.len());
    for unit in &board.opponent_units {
        let (resolved, metadata) =
            resolve_unit(base_game_data.as_ref(), unit, ScenarioUnitRole::Other);
        resolved_opponent_units.push(resolved);
        if let Some(metadata) = metadata {
            metadata_overrides.push(metadata);
        }
    }

    let game_data = build_game_data_with_units(base_game_data, &metadata_overrides);

    let mut runtime_patches = Vec::new();
    for unit in resolved_player_units
        .iter()
        .chain(resolved_opponent_units.iter())
    {
        if !unit.runtime_patch.is_empty() {
            runtime_patches.push((unit.position, unit.runtime_patch.clone()));
        }
    }

    let battle_scenario = build_battle_scenario(&resolved_player_units, &resolved_opponent_units);

    ResolvedScenario {
        abnormality,
        game_data,
        battle_scenario,
        runtime_patches,
    }
}
