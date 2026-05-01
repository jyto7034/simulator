use std::collections::{HashMap, HashSet};

use crate::game::battle::{
    ids::UnitInstanceId,
    timeline::{SkillCastTarget, Timeline, TimelineEvent},
};

use super::{
    spawns::ExtractedSpawns,
    types::{TimelineValidatorConfig, TimelineViolation, TimelineViolationKind},
};

pub(super) fn validate_deaths(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
    let mut died_units: HashSet<UnitInstanceId> = HashSet::new();
    let mut death_by_unit: HashMap<UnitInstanceId, (usize, u64)> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };

        death_by_unit
            .entry(unit_instance_id)
            .and_modify(|existing| {
                if index < existing.0 {
                    *existing = (index, entry.time_ms);
                }
            })
            .or_insert((index, entry.time_ms));

        if !died_units.insert(unit_instance_id) {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnitDiedDuplicate,
                message: format!("unit {} has multiple UnitDied entries", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }

    for (index, entry) in timeline.entries.iter().enumerate() {
        if death_by_unit.is_empty() {
            break;
        }
        for unit_id in referenced_unit_ids(&entry.event) {
            if let Some(&(death_index, death_time_ms)) = death_by_unit.get(&unit_id) {
                if death_index < index
                    && death_time_ms < entry.time_ms
                    && is_dead_unit_operated_on(&entry.event, unit_id, config)
                {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::DeadUnitActsAfterDeath,
                        message: format!(
                            "unit {} is referenced after death at time_ms {}",
                            unit_id, entry.time_ms
                        ),
                        entry_index: Some(index),
                    });
                }
            }
        }
    }

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };
        if !extracted
            .unit_owner_by_instance
            .contains_key(&unit_instance_id)
        {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnknownUnitReference,
                message: format!("UnitDied references unknown unit {}", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }
}

fn referenced_unit_ids(event: &TimelineEvent) -> Vec<UnitInstanceId> {
    match event {
        TimelineEvent::AttackStart {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::AttackResolve {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::AttackMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        TimelineEvent::ProjectileMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        TimelineEvent::BasicAttackProjectileLaunched {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::BasicAttackProjectileImpacted {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        TimelineEvent::AutoCastStart {
            caster_instance_id,
            target,
            ..
        } => {
            let mut ids = vec![*caster_instance_id];
            if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                ids.push(*unit_instance_id);
            }
            ids
        }
        TimelineEvent::AutoCastEnd { caster_instance_id } => vec![*caster_instance_id],
        TimelineEvent::TriggeredAbilityProc {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        TimelineEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        TimelineEvent::AbilityStepTriggered {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        TimelineEvent::SkillAreaDeclared {
            caster_instance_id,
            target,
            ..
        } => {
            let mut ids = vec![*caster_instance_id];
            if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                ids.push(*unit_instance_id);
            }
            ids
        }
        TimelineEvent::SkillProjectileLaunched {
            caster_instance_id,
            target,
            ..
        } => {
            let mut ids = vec![*caster_instance_id];
            if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                ids.push(*unit_instance_id);
            }
            ids
        }
        TimelineEvent::SkillProjectileImpacted {
            caster_instance_id,
            first_hit_unit_id,
            ..
        } => first_hit_unit_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        TimelineEvent::BuffApplied {
            caster_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::BuffTick {
            caster_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::BuffExpired {
            caster_instance_id,
            target_instance_id,
            ..
        } => vec![*caster_instance_id, *target_instance_id],
        TimelineEvent::HpChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        TimelineEvent::StatChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        TimelineEvent::ResonanceChanged {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        TimelineEvent::UnitDied {
            unit_instance_id,
            killer_instance_id,
            ..
        } => killer_instance_id
            .map(|killer| vec![*unit_instance_id, killer])
            .unwrap_or_else(|| vec![*unit_instance_id]),
        TimelineEvent::MovementSegmentStarted {
            unit_instance_id, ..
        }
        | TimelineEvent::MovementStopped {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        TimelineEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => vec![*owner_unit_instance_id],
        TimelineEvent::UnitSpawned {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        TimelineEvent::BattleStart { .. }
        | TimelineEvent::ArtifactSpawned { .. }
        | TimelineEvent::BattleEnd { .. } => Vec::new(),
    }
}

fn is_dead_unit_operated_on(
    event: &TimelineEvent,
    dead_unit_id: UnitInstanceId,
    config: &TimelineValidatorConfig,
) -> bool {
    match event {
        TimelineEvent::AttackStart {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::AttackResolve {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::AttackMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        TimelineEvent::ProjectileMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        TimelineEvent::BasicAttackProjectileLaunched {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | TimelineEvent::BasicAttackProjectileImpacted {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        TimelineEvent::AutoCastStart {
            caster_instance_id,
            target,
            ..
        } => {
            let is_target_dead = target.is_some_and(|t| match t {
                SkillCastTarget::Unit { unit_instance_id } => unit_instance_id == dead_unit_id,
                SkillCastTarget::Tile { .. } => false,
            });
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && is_target_dead)
        }
        TimelineEvent::AutoCastEnd { caster_instance_id } => {
            config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id
        }
        TimelineEvent::TriggeredAbilityProc {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        TimelineEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        TimelineEvent::AbilityStepTriggered {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        TimelineEvent::SkillAreaDeclared {
            caster_instance_id,
            target,
            ..
        } => {
            let is_target_dead = target.is_some_and(|t| match t {
                SkillCastTarget::Unit { unit_instance_id } => unit_instance_id == dead_unit_id,
                SkillCastTarget::Tile { .. } => false,
            });
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && is_target_dead)
        }
        TimelineEvent::SkillProjectileLaunched {
            caster_instance_id,
            target,
            ..
        } => {
            let is_target_dead = target.is_some_and(|t| match t {
                SkillCastTarget::Unit { unit_instance_id } => unit_instance_id == dead_unit_id,
                SkillCastTarget::Tile { .. } => false,
            });
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && is_target_dead)
        }
        TimelineEvent::SkillProjectileImpacted {
            caster_instance_id,
            first_hit_unit_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && first_hit_unit_id.is_some_and(|id| id == dead_unit_id))
        }
        TimelineEvent::BuffApplied {
            target_instance_id, ..
        }
        | TimelineEvent::BuffTick {
            target_instance_id, ..
        }
        | TimelineEvent::BuffExpired {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        TimelineEvent::HpChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        TimelineEvent::StatChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        TimelineEvent::ResonanceChanged {
            unit_instance_id, ..
        } => config.forbid_dead_units_as_targets && *unit_instance_id == dead_unit_id,
        TimelineEvent::MovementSegmentStarted {
            unit_instance_id, ..
        }
        | TimelineEvent::MovementStopped {
            unit_instance_id, ..
        } => config.forbid_dead_units_as_attackers && *unit_instance_id == dead_unit_id,
        TimelineEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => *owner_unit_instance_id == dead_unit_id,
        TimelineEvent::UnitSpawned {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        TimelineEvent::UnitDied {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        TimelineEvent::BattleStart { .. }
        | TimelineEvent::ArtifactSpawned { .. }
        | TimelineEvent::BattleEnd { .. } => false,
    }
}
