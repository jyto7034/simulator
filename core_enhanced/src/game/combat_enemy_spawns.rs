use std::collections::HashSet;

use tracing::warn;

use crate::game::{
    battle::{
        scenario::{
            EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{BattleUnitDraft, BattleUnitSource},
    },
    behavior::GameError,
    combat_mission_policy::defense_route_tactical_point_id,
    combat_preview::{CombatPreview, EnemyKind, SpawnWave, SpawnWaveEnemyEntry},
    data::GameDataBase,
    determinism,
    enums::Side,
    growth::GrowthStack,
    resources::Position,
};

const PVE_OWNED_ABNORMALITY_NS: u64 = 0x0050_5645_4f57_4e44_u64; // "PVEOWND"

pub fn enemy_spawn_groups_from_preview(
    game_data: &GameDataBase,
    encounter_id: &str,
    combat_preview: &CombatPreview,
) -> Result<Vec<(ScenarioSpawnGroup, u32)>, GameError> {
    let mut groups = Vec::new();
    let mut global_spawn_index = 0usize;
    let mut waves = combat_preview.spawn_waves.clone();
    waves.sort_by_key(|wave| wave.time_ms);
    if waves.is_empty() {
        return Err(GameError::InvalidStaticData(format!(
            "combat preview for encounter '{}' has no spawn waves",
            encounter_id
        )));
    }

    for (wave_index, wave) in waves.iter().enumerate() {
        let group_id = ScenarioGroupId::new(format!("enemy_{}", wave.id));
        let positions = collect_wave_spawn_positions(combat_preview, wave)?;
        let drafts = enemy_drafts_for_wave(
            game_data,
            &wave.enemy_entries,
            wave_index,
            &mut global_spawn_index,
        )?;
        if drafts.is_empty() {
            return Err(GameError::InvalidStaticData(format!(
                "combat preview wave '{}' has no enemy entries",
                wave.id
            )));
        }

        if positions.len() < drafts.len() {
            return Err(GameError::InvalidStaticData(format!(
                "combat preview wave '{}' has {} unique spawn cells but requires {}",
                wave.id,
                positions.len(),
                drafts.len()
            )));
        }

        let spawns = drafts
            .into_iter()
            .enumerate()
            .map(|(index, (draft, instance_salt))| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side: Side::Opponent,
                draft,
                position: positions[index],
                instance_salt,
            })
            .collect();

        groups.push((
            ScenarioSpawnGroup {
                id: group_id,
                side: Side::Opponent,
                required_for_victory: wave.required_for_victory,
                enemy_movement_plan: enemy_movement_plan_for_wave(combat_preview, wave),
                spawns,
            },
            wave.time_ms,
        ));
    }

    Ok(groups)
}

fn enemy_movement_plan_for_wave(
    combat_preview: &CombatPreview,
    wave: &SpawnWave,
) -> Option<EnemyMovementPlan> {
    let route_id = wave.route_id.as_ref()?;
    let route = combat_preview
        .routes
        .iter()
        .find(|route| &route.id == route_id)?;
    let route_cells = if route.cells.is_empty() {
        vec![route.end]
    } else {
        route.cells.clone()
    };
    let point_ids = route_cells
        .iter()
        .enumerate()
        .map(|(index, _)| {
            defense_route_tactical_point_id(route_id, index, index + 1 == route_cells.len())
        })
        .collect::<Vec<_>>();
    Some(EnemyMovementPlan::PathAlongPath { point_ids })
}

fn enemy_drafts_for_wave(
    game_data: &GameDataBase,
    enemy_entries: &[SpawnWaveEnemyEntry],
    wave_index: usize,
    global_spawn_index: &mut usize,
) -> Result<Vec<(BattleUnitDraft, u32)>, GameError> {
    let mut drafts = Vec::new();
    for entry in enemy_entries {
        let count = entry.count.max(1);
        for _ in 0..count {
            let (base_uuid, source) = match entry.kind {
                EnemyKind::Abnormality => {
                    let abnormality_id = entry.abnormality_id.as_str();
                    let abnormality_meta = game_data
                        .abnormality_data
                        .get_by_id(abnormality_id)
                        .ok_or_else(|| {
                            warn!("Abnormality metadata not found: {}", abnormality_id);
                            GameError::MissingResource("AbnormalityMetadata")
                        })?;
                    (
                        abnormality_meta.uuid,
                        BattleUnitSource::Abnormality {
                            base_uuid: abnormality_meta.uuid,
                        },
                    )
                }
                EnemyKind::CorrodedEmployee => {
                    let profile_id = entry.profile_id.as_deref().ok_or_else(|| {
                        warn!(
                            "Corroded employee enemy entry missing profile_id: {}",
                            entry.abnormality_id
                        );
                        GameError::MissingResource("CorrodedEmployeeProfile")
                    })?;
                    let profile = game_data
                        .corroded_employee_data
                        .get_by_id(profile_id)
                        .ok_or_else(|| {
                            warn!("Corroded employee profile not found: {}", profile_id);
                            GameError::MissingResource("CorrodedEmployeeProfile")
                        })?;
                    (
                        profile.uuid,
                        BattleUnitSource::CorrodedEmployee {
                            profile_id: profile.id.clone(),
                            base_uuid: profile.uuid,
                        },
                    )
                }
                EnemyKind::FacilityEntity => {
                    return Err(GameError::MissingResource("FacilityEntityProfile"));
                }
            };

            let spawn_index = *global_spawn_index;
            *global_spawn_index = global_spawn_index.saturating_add(1);

            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&base_uuid.as_bytes()[..8]);
            let seed = u64::from_be_bytes(bytes) ^ ((wave_index as u64) << 32);
            let owned_uuid =
                determinism::uuid_v4_from_seed(seed, PVE_OWNED_ABNORMALITY_NS, spawn_index as u64);

            drafts.push((
                BattleUnitDraft {
                    owned_uuid,
                    source,
                    level: entry.tier,
                    growth_stacks: GrowthStack::new(),
                    equipped_items: vec![],
                    equipped_item_enhancements: vec![],
                },
                spawn_index as u32,
            ));
        }
    }

    Ok(drafts)
}

fn collect_wave_spawn_positions(
    combat_preview: &CombatPreview,
    wave: &SpawnWave,
) -> Result<Vec<Position>, GameError> {
    let mut cells = Vec::new();
    let mut seen = HashSet::new();

    for zone_id in &wave.spawn_zone_ids {
        if let Some(zone) = combat_preview
            .spawn_zones
            .iter()
            .find(|zone| &zone.id == zone_id)
        {
            push_unique_cells(&mut cells, &mut seen, &zone.cells);
        }
    }

    if cells.is_empty() {
        for zone in &combat_preview.spawn_zones {
            push_unique_cells(&mut cells, &mut seen, &zone.cells);
        }
    }

    if cells.is_empty() {
        return Err(GameError::InvalidStaticData(format!(
            "combat preview '{}' wave '{}' has no valid spawn cells",
            combat_preview.battlefield_template_id, wave.id
        )));
    }

    Ok(cells)
}

fn push_unique_cells(cells: &mut Vec<Position>, seen: &mut HashSet<Position>, source: &[Position]) {
    for cell in source {
        if seen.insert(*cell) {
            cells.push(*cell);
        }
    }
}
