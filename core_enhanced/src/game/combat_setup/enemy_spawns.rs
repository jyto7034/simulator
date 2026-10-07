//! Converts combat preview waves into deterministic enemy scenario spawn groups.
//!
//! Preview wave order, spawn-zone selection, enemy draft creation, and owned UUID
//! generation live here so battle startup remains reproducible.

use std::collections::HashSet;

use tracing::warn;

use crate::game::{
    battle::{
        scenario::{
            EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{BattleUnitDraft, BattleUnitExactStatOverride, BattleUnitSource},
    },
    behavior::GameError,
    combat_preview::{CombatPreview, EnemyKind, SpawnWave, SpawnWaveEnemyEntry},
    data::{pve_data::PveWaveActivationData, GameDataBase},
    determinism,
    enums::Side,
    growth::GrowthStack,
    map::AuthoredEncounterActorStatOverride,
    resources::Position,
};

const PVE_OWNED_ABNORMALITY_NS: u64 = 0x0050_5645_4f57_4e44_u64; // "PVEOWND"

pub fn enemy_spawn_groups_from_preview(
    game_data: &GameDataBase,
    encounter_id: &str,
    combat_preview: &CombatPreview,
) -> Result<Vec<(ScenarioSpawnGroup, PveWaveActivationData)>, GameError> {
    enemy_spawn_groups_from_preview_with_overrides(game_data, encounter_id, combat_preview, &[])
}

pub(crate) fn enemy_spawn_groups_from_preview_with_overrides(
    game_data: &GameDataBase,
    encounter_id: &str,
    combat_preview: &CombatPreview,
    actor_stat_overrides: &[AuthoredEncounterActorStatOverride],
) -> Result<Vec<(ScenarioSpawnGroup, PveWaveActivationData)>, GameError> {
    let mut groups = Vec::new();
    let mut global_spawn_index = 0usize;
    let waves = combat_preview.spawn_waves.clone();
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
            combat_preview.enemy_stat_scale,
            actor_stat_overrides,
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
            wave.activation.clone(),
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
    Some(EnemyMovementPlan::PathAlongCells { cells: route_cells })
}

fn enemy_drafts_for_wave(
    game_data: &GameDataBase,
    enemy_entries: &[SpawnWaveEnemyEntry],
    stat_scale: crate::game::battle::types::BattleUnitStatScale,
    actor_stat_overrides: &[AuthoredEncounterActorStatOverride],
    wave_index: usize,
    global_spawn_index: &mut usize,
) -> Result<Vec<(BattleUnitDraft, u32)>, GameError> {
    use crate::authoring::control::{self, Resource};
    let mut drafts = Vec::new();
    for entry in enemy_entries {
        let count = entry.count.max(1);
        control::admit(
            Resource::RetainedEntities,
            *global_spawn_index,
            count as usize,
            "enemy_draft_expansion",
        )?;
        control::charge(u64::from(count), "enemy_drafts")?;
        for _ in 0..count {
            control::poll("enemy_draft")?;
            let (base_uuid, source, threat_class, exact_override) = match entry.kind {
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
                        abnormality_meta.threat_class,
                        actor_stat_overrides
                            .iter()
                            .find(|override_data| override_data.abnormality_id == abnormality_id)
                            .map(|override_data| BattleUnitExactStatOverride {
                                max_health: override_data.max_health,
                                attack: override_data.attack,
                                defense: override_data.defense,
                                magic_resistance: override_data.magic_resistance,
                            }),
                    )
                }
                EnemyKind::CorrodedEmployee => {
                    let profile_id = entry.profile_id.as_deref().ok_or_else(|| {
                        warn!("Corroded employee enemy entry missing profile_id");
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
                        crate::game::battle::types::BattleUnitThreatClass::Normal,
                        None,
                    )
                }
                EnemyKind::FacilityEntity => {
                    return Err(GameError::MissingResource("FacilityEntityProfile"));
                }
            };
            let mut stat_scale = stat_scale;
            stat_scale.exact_override = exact_override;

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
                    threat_class,
                    level: entry.tier,
                    stat_scale,
                    growth_stacks: GrowthStack::new(),
                    equipment_layout: None,
                    equipped_items: vec![],
                    equipped_item_enhancements: vec![],
                    employee_start_condition: None,
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::types::BattleUnitSource,
        map::{GameMode, MapNodeCategory, MapNodeId},
    };

    #[test]
    fn standard_floor_three_overrides_only_nothing_there_to_exact_authored_stats() {
        let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("live data must load");
        let schedule = game_data
            .map_catalogs
            .node_assignment_profiles
            .resolve(
                &game_data.map_catalogs.facility_templates,
                GameMode::Standard,
                2,
            )
            .unwrap();
        let authored = schedule
            .profile
            .fixed_slot_assignments
            .iter()
            .find(|assignment| assignment.slot_id.as_str() == "final_chamber")
            .and_then(|assignment| assignment.authored_combat.as_ref())
            .expect("F3 final chamber must author combat policy");
        assert_eq!(
            authored
                .outcome_policies
                .iter()
                .find(|policy| policy.outcome_id == "victory")
                .expect("F3 final chamber Victory policy")
                .reward_ids,
            [
                "enkephalin_reward".to_string(),
                "field_equipment_salvage_reward".to_string(),
            ]
        );
        assert_eq!(authored.actor_stat_overrides.len(), 1);

        let preview = CombatPreview::try_generate_for_node(
            MapNodeId::new(Uuid::nil()),
            MapNodeCategory::Boss,
            Some("suppress_nothing_there"),
            &game_data,
            0xF3,
        )
        .expect("Nothing There preview must generate");
        let groups = enemy_spawn_groups_from_preview_with_overrides(
            &game_data,
            "suppress_nothing_there",
            &preview,
            &authored.actor_stat_overrides,
        )
        .expect("authored enemy spawns must resolve");
        let drafts = groups
            .iter()
            .flat_map(|(group, _)| group.spawns.iter().map(|spawn| &spawn.draft))
            .collect::<Vec<_>>();
        let nothing_there = drafts
            .iter()
            .find(|draft| {
                matches!(
                    draft.source,
                    BattleUnitSource::Abnormality { base_uuid }
                        if base_uuid
                            == game_data
                                .abnormality_data
                                .get_by_id("o-06-20_nothing_there")
                                .unwrap()
                                .uuid
                )
            })
            .expect("Nothing There draft");
        let stats = nothing_there.effective_stats(&game_data).unwrap();
        assert_eq!(stats.max_health, 720);
        assert_eq!(stats.current_health, 720);
        assert_eq!(stats.attack, 53);
        assert_eq!(stats.defense, 36);
        assert_eq!(stats.magic_resist, 33);
        assert!(drafts
            .iter()
            .any(|draft| matches!(draft.source, BattleUnitSource::CorrodedEmployee { .. })));
        assert!(drafts
            .iter()
            .filter(|draft| matches!(draft.source, BattleUnitSource::CorrodedEmployee { .. }))
            .all(|draft| draft.stat_scale.exact_override.is_none()));

        let unmodified_groups =
            enemy_spawn_groups_from_preview(&game_data, "suppress_nothing_there", &preview)
                .unwrap();
        let unmodified = unmodified_groups
            .iter()
            .flat_map(|(group, _)| group.spawns.iter().map(|spawn| &spawn.draft))
            .find(|draft| matches!(draft.source, BattleUnitSource::Abnormality { .. }))
            .expect("unmodified Nothing There draft")
            .effective_stats(&game_data)
            .unwrap();
        assert_eq!(unmodified.max_health, 1_200);
        assert_eq!(unmodified.attack, 88);
        assert_eq!(unmodified.defense, 60);
        assert_eq!(unmodified.magic_resist, 55);
    }
}
