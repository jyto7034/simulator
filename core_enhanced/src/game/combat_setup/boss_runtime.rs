use crate::game::{
    battle::scenario::{EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup},
    behavior::GameError,
    data::{
        boss_runtime_data::{
            BossSummonPlacementData, CompiledBossControlNode, CompiledBossTask,
            ResolvedBossActorSetup, ResolvedBossEncounterSetup, ResolvedBossSummonRoute,
            ResolvedBossSummonSpawnZone,
        },
        pve_data::PveEncounter,
        GameDataBase,
    },
    enums::Side,
};

pub fn resolve_boss_runtime_setup<T>(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    enemy_groups: &[(ScenarioSpawnGroup, T)],
) -> Result<Option<ResolvedBossEncounterSetup>, GameError> {
    resolve_boss_runtime_setup_inner(game_data, encounter, enemy_groups, None)
}

pub fn resolve_boss_runtime_setup_with_preview<T>(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    enemy_groups: &[(ScenarioSpawnGroup, T)],
    combat_preview: &crate::game::combat_preview::CombatPreview,
) -> Result<Option<ResolvedBossEncounterSetup>, GameError> {
    resolve_boss_runtime_setup_inner(game_data, encounter, enemy_groups, Some(combat_preview))
}

fn resolve_boss_runtime_setup_inner<T>(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    enemy_groups: &[(ScenarioSpawnGroup, T)],
    combat_preview: Option<&crate::game::combat_preview::CombatPreview>,
) -> Result<Option<ResolvedBossEncounterSetup>, GameError> {
    let Some(binding) = &encounter.boss_runtime else {
        return Ok(None);
    };
    let definition = game_data
        .boss_runtime_data
        .get_encounter(&binding.encounter_id)
        .cloned()
        .ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "encounter '{}' references missing Boss Runtime definition '{}'",
                encounter.id,
                binding.encounter_id.as_str()
            ))
        })?;

    let mut actors = Vec::with_capacity(binding.actors.len());
    let mut bound_unit_refs = std::collections::HashSet::new();
    for actor in &binding.actors {
        let actor_slot_index = definition
            .actor_slots
            .iter()
            .position(|slot| slot == &actor.actor_slot_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "encounter '{}' binds undeclared Boss actor slot '{}'",
                    encounter.id,
                    actor.actor_slot_id.as_str()
                ))
            })?;
        let group_id = ScenarioGroupId::new(format!("enemy_{}", actor.wave_id));
        let group = enemy_groups
            .iter()
            .map(|(group, _)| group)
            .find(|group| group.id == group_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "Boss actor slot '{}' references unresolved wave '{}'",
                    actor.actor_slot_id.as_str(),
                    actor.wave_id
                ))
            })?;
        let spawn = group
            .spawns
            .get(actor.spawn_index as usize)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "Boss actor slot '{}' spawn_index {} is outside resolved wave '{}'",
                    actor.actor_slot_id.as_str(),
                    actor.spawn_index,
                    actor.wave_id
                ))
            })?;
        if group.side != Side::Opponent || spawn.side != Side::Opponent {
            return Err(GameError::InvalidStaticData(format!(
                "Boss actor slot '{}' must bind an opponent spawn",
                actor.actor_slot_id.as_str()
            )));
        }
        if definition.actor_slot_requires_authored_route(actor_slot_index)
            && !matches!(
                group.enemy_movement_plan,
                Some(EnemyMovementPlan::PathAlongCells { ref cells }) if !cells.is_empty()
            )
        {
            return Err(GameError::InvalidStaticData(format!(
                "Boss actor slot '{}' requires a non-empty authored route",
                actor.actor_slot_id.as_str()
            )));
        }
        if !bound_unit_refs.insert(spawn.unit_ref.clone()) {
            return Err(GameError::InvalidStaticData(format!(
                "Boss Runtime setup binds unit_ref '{}' more than once",
                spawn.unit_ref.0
            )));
        }
        actors.push(ResolvedBossActorSetup {
            actor_slot_index,
            unit_ref: spawn.unit_ref.clone(),
        });
    }
    actors.sort_by_key(|actor| actor.actor_slot_index);
    if actors.len() != definition.actor_slots.len()
        || actors
            .iter()
            .enumerate()
            .any(|(index, actor)| actor.actor_slot_index != index)
    {
        return Err(GameError::InvalidStaticData(format!(
            "encounter '{}' must resolve every Boss actor slot exactly once",
            encounter.id
        )));
    }

    let enemy_stat_scale = combat_preview
        .map(|preview| preview.enemy_stat_scale)
        .unwrap_or_default();
    let summon_spawn_zones = combat_preview
        .map(|preview| {
            preview
                .spawn_zones
                .iter()
                .map(|zone| ResolvedBossSummonSpawnZone {
                    id: zone.id.clone(),
                    cells: zone.cells.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let summon_routes = combat_preview
        .map(|preview| {
            preview
                .routes
                .iter()
                .map(|route| ResolvedBossSummonRoute {
                    id: route.id.clone(),
                    cells: if route.cells.is_empty() {
                        vec![route.end]
                    } else {
                        route.cells.clone()
                    },
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    validate_summon_setup_requirements(
        encounter.id.as_str(),
        &definition,
        &summon_spawn_zones,
        &summon_routes,
    )?;

    Ok(Some(ResolvedBossEncounterSetup {
        definition,
        actors,
        enemy_stat_scale,
        summon_spawn_zones,
        summon_routes,
    }))
}

fn validate_summon_setup_requirements(
    encounter_id: &str,
    definition: &crate::game::data::boss_runtime_data::CompiledBossEncounterDefinition,
    zones: &[ResolvedBossSummonSpawnZone],
    routes: &[ResolvedBossSummonRoute],
) -> Result<(), GameError> {
    let mut checked_patterns = std::collections::HashSet::new();
    for phase in &definition.phases {
        for binding in &phase.actor_behaviors {
            for node in &binding.behavior.nodes {
                let CompiledBossControlNode::PatternCall { candidates, .. } = node else {
                    continue;
                };
                for candidate in candidates {
                    if !checked_patterns.insert(candidate.pattern.id.clone()) {
                        continue;
                    }
                    for state in &candidate.pattern.states {
                        for task in &state.tasks {
                            let CompiledBossTask::Summon {
                                count,
                                placement,
                                route_id,
                                ..
                            } = task
                            else {
                                continue;
                            };
                            if let BossSummonPlacementData::SpawnZone { zone_id } = placement {
                                let zone = zones.iter().find(|zone| &zone.id == zone_id).ok_or_else(|| {
                                    GameError::InvalidStaticData(format!(
                                        "Boss Runtime encounter '{}' references missing summon spawn zone '{}'",
                                        encounter_id, zone_id
                                    ))
                                })?;
                                let unique = zone
                                    .cells
                                    .iter()
                                    .map(|cell| (cell.x, cell.y))
                                    .collect::<std::collections::HashSet<_>>();
                                if zone.cells.is_empty() || unique.len() != zone.cells.len() {
                                    return Err(GameError::InvalidStaticData(format!(
                                        "Boss Runtime encounter '{}' summon spawn zone '{}' must contain unique authored cells",
                                        encounter_id, zone_id
                                    )));
                                }
                                if zone.cells.len() < *count as usize {
                                    return Err(GameError::InvalidStaticData(format!(
                                        "Boss Runtime encounter '{}' summon spawn zone '{}' has fewer cells than count {}",
                                        encounter_id, zone_id, count
                                    )));
                                }
                            }
                            if let Some(route_id) = route_id {
                                let route = routes.iter().find(|route| &route.id == route_id).ok_or_else(|| {
                                    GameError::InvalidStaticData(format!(
                                        "Boss Runtime encounter '{}' references missing summon route '{}'",
                                        encounter_id, route_id
                                    ))
                                })?;
                                if route.cells.is_empty() {
                                    return Err(GameError::InvalidStaticData(format!(
                                        "Boss Runtime encounter '{}' summon route '{}' must not be empty",
                                        encounter_id, route_id
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::{
            scenario::{EnemyMovementPlan, ScenarioUnitRef, ScenarioUnitSpawn},
            types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass},
        },
        data::{
            boss_runtime_data::{
                tests::valid_catalog, BossActorSlotId, BossEncounterRuntimeId,
                CompiledBossControlNode, ResolvedBossParameterValue,
            },
            pve_data::{
                PveBossActorBindingData, PveBossRuntimeBindingData, PveEncounter, PveEncounterClass,
            },
            GameDataBuilder,
        },
        enums::{RewardMode, RiskLevel, Tier},
        growth::GrowthStack,
        resources::Position,
    };

    fn game_data() -> GameDataBase {
        let database = valid_catalog()
            .compile()
            .expect("Boss fixture catalog should compile");
        GameDataBuilder::empty()
            .with_boss_runtime_data(Arc::new(database))
            .build()
    }

    fn encounter(binding: Option<PveBossRuntimeBindingData>) -> PveEncounter {
        PveEncounter {
            id: "fixture_encounter".to_string(),
            encounter_class: PveEncounterClass::NormalBoss,
            primary_abnormality_id: None,
            boss_runtime: binding,
            risk_level: RiskLevel::ALEPH,
            node_type: None,
            mission_variant: None,
            terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
            outcome_policies: PveEncounter::fixture_outcome_policies(RewardMode::ClaimAll, vec![]),
            battlefield: None,
            tactical_plan: None,
            waves: vec![],
            static_obstacles: vec![],
        }
    }

    fn binding(spawn_index: u32) -> PveBossRuntimeBindingData {
        PveBossRuntimeBindingData {
            encounter_id: BossEncounterRuntimeId::new("tracking_encounter"),
            actors: vec![PveBossActorBindingData {
                actor_slot_id: BossActorSlotId::new("primary"),
                wave_id: "wave_0".to_string(),
                spawn_index,
            }],
        }
    }

    fn enemy_group(with_route: bool) -> Vec<(ScenarioSpawnGroup, u32)> {
        vec![(
            ScenarioSpawnGroup {
                id: ScenarioGroupId::new("enemy_wave_0"),
                side: Side::Opponent,
                required_for_victory: true,
                enemy_movement_plan: with_route.then(|| EnemyMovementPlan::PathAlongCells {
                    cells: vec![Position::new(3, 1), Position::new(2, 1)],
                }),
                spawns: vec![ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("enemy_wave_0_0"),
                    side: Side::Opponent,
                    draft: BattleUnitDraft {
                        owned_uuid: Uuid::from_u128(1),
                        source: BattleUnitSource::Abnormality {
                            base_uuid: Uuid::from_u128(2),
                        },
                        threat_class: BattleUnitThreatClass::Boss,
                        level: Tier::I,
                        stat_scale: Default::default(),
                        growth_stacks: GrowthStack::new(),
                        equipment_layout: None,
                        equipped_items: vec![],
                        equipped_item_enhancements: vec![],
                        employee_start_condition: None,
                    },
                    position: Position::new(3, 1),
                    instance_salt: 0,
                }],
            },
            0,
        )]
    }

    #[test]
    fn no_binding_resolves_to_none_without_consulting_boss_catalog() {
        let game_data = GameDataBuilder::empty().build();
        assert!(
            resolve_boss_runtime_setup::<u32>(&game_data, &encounter(None), &[])
                .expect("no-binding setup should preserve the ordinary path")
                .is_none()
        );
    }

    #[test]
    fn bound_actor_resolves_to_immutable_definition_and_argument_frame() {
        let game_data = game_data();
        let setup = resolve_boss_runtime_setup(
            &game_data,
            &encounter(Some(binding(0))),
            &enemy_group(true),
        )
        .expect("valid binding should resolve")
        .expect("binding should produce Boss setup");

        assert_eq!(setup.actors[0].actor_slot_index, 0);
        assert_eq!(setup.actors[0].unit_ref.0, "enemy_wave_0_0");
        let behavior = &setup.definition.phases[0].actor_behaviors[0].behavior;
        let CompiledBossControlNode::PatternCall { candidates, .. } = &behavior.nodes[0] else {
            panic!("fixture entry should be a PatternCall")
        };
        assert_eq!(
            candidates[0].argument_frame.values[1],
            ResolvedBossParameterValue::DurationMs(5_000)
        );

        drop(game_data);
        assert_eq!(setup.definition.id.as_str(), "tracking_encounter");
        assert_eq!(candidates[0].pattern.id.as_str(), "track_and_rejoin");
    }

    #[test]
    fn invalid_route_or_spawn_binding_rejects_before_scenario_construction() {
        let game_data = game_data();
        let route_error = resolve_boss_runtime_setup(
            &game_data,
            &encounter(Some(binding(0))),
            &enemy_group(false),
        )
        .unwrap_err();
        assert!(
            matches!(route_error, GameError::InvalidStaticData(message) if message.contains("authored route"))
        );

        let spawn_error = resolve_boss_runtime_setup(
            &game_data,
            &encounter(Some(binding(1))),
            &enemy_group(true),
        )
        .unwrap_err();
        assert!(
            matches!(spawn_error, GameError::InvalidStaticData(message) if message.contains("spawn_index"))
        );
    }
}
