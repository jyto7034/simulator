use std::collections::{HashMap, HashSet};

use crate::game::battle::{
    event_log::{BattleEventLog, BattleLogEvent, SkillCastTarget},
    ids::UnitInstanceId,
};

use super::{
    spawns::ExtractedSpawns,
    types::{EventLogValidatorConfig, EventLogViolation, EventLogViolationKind},
};

pub(super) fn validate_deaths(
    event_log: &BattleEventLog,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<EventLogViolation>,
    config: &EventLogValidatorConfig,
) {
    let mut died_units: HashSet<UnitInstanceId> = HashSet::new();
    let mut death_by_unit: HashMap<UnitInstanceId, (usize, u64)> = HashMap::new();

    for (index, entry) in event_log.entries.iter().enumerate() {
        let BattleLogEvent::UnitDied {
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
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::UnitDiedDuplicate,
                message: format!("unit {} has multiple UnitDied entries", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }

    for (index, entry) in event_log.entries.iter().enumerate() {
        if death_by_unit.is_empty() {
            break;
        }
        for unit_id in referenced_unit_ids(&entry.event) {
            if let Some(&(death_index, death_time_ms)) = death_by_unit.get(&unit_id) {
                if death_index < index
                    && death_time_ms < entry.time_ms
                    && is_dead_unit_operated_on(&entry.event, unit_id, config)
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::DeadUnitActsAfterDeath,
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

    for (index, entry) in event_log.entries.iter().enumerate() {
        let BattleLogEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };
        if !extracted
            .unit_owner_by_instance
            .contains_key(&unit_instance_id)
        {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::UnknownUnitReference,
                message: format!("UnitDied references unknown unit {}", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }
}

fn referenced_unit_ids(event: &BattleLogEvent) -> Vec<UnitInstanceId> {
    match event {
        BattleLogEvent::AttackStart {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::AttackResolve {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::AttackMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        BattleLogEvent::BasicAttackProjectileLaunched {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::BasicAttackProjectileImpacted {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        BattleLogEvent::AutoCastStart {
            caster_instance_id,
            target,
            ..
        }
        | BattleLogEvent::ManualCastStart {
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
        BattleLogEvent::AutoCastEnd { caster_instance_id }
        | BattleLogEvent::ManualCastEnd { caster_instance_id } => vec![*caster_instance_id],
        BattleLogEvent::SkillCastInterrupted {
            interrupter_instance_id,
            caster_instance_id,
            ..
        } => vec![*interrupter_instance_id, *caster_instance_id],
        BattleLogEvent::SkillCastCancelled {
            caster_instance_id, ..
        } => vec![*caster_instance_id],
        BattleLogEvent::TriggeredAbilityProc {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        BattleLogEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        BattleLogEvent::AbilityStepTriggered {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        BattleLogEvent::SkillAreaDeclared {
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
        BattleLogEvent::SkillProjectileLaunched {
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
        BattleLogEvent::SkillProjectileImpacted {
            caster_instance_id,
            first_hit_unit_id,
            ..
        } => first_hit_unit_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        BattleLogEvent::BuffApplied {
            caster_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::BuffTick {
            caster_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::BuffExpired {
            caster_instance_id,
            target_instance_id,
            ..
        } => vec![*caster_instance_id, *target_instance_id],
        BattleLogEvent::HpChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        BattleLogEvent::StatChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        BattleLogEvent::ResonanceChanged {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        BattleLogEvent::UnitDied {
            unit_instance_id,
            killer_instance_id,
            ..
        } => killer_instance_id
            .map(|killer| vec![*unit_instance_id, killer])
            .unwrap_or_else(|| vec![*unit_instance_id]),
        BattleLogEvent::MovementSegmentStarted {
            unit_instance_id, ..
        }
        | BattleLogEvent::MovementStopped {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        BattleLogEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => vec![*owner_unit_instance_id],
        BattleLogEvent::UnitSpawned {
            unit_instance_id, ..
        }
        | BattleLogEvent::UnitDeployed {
            unit_instance_id, ..
        }
        | BattleLogEvent::UnitWithdrawn {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        BattleLogEvent::BattleStart { .. }
        | BattleLogEvent::ArtifactSpawned { .. }
        | BattleLogEvent::BattleEnd { .. } => Vec::new(),
    }
}

fn is_dead_unit_operated_on(
    event: &BattleLogEvent,
    dead_unit_id: UnitInstanceId,
    config: &EventLogValidatorConfig,
) -> bool {
    match event {
        BattleLogEvent::AttackStart {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::AttackResolve {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::AttackMiss {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        BattleLogEvent::BasicAttackProjectileLaunched {
            attacker_instance_id,
            target_instance_id,
            ..
        }
        | BattleLogEvent::BasicAttackProjectileImpacted {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        BattleLogEvent::AutoCastStart {
            caster_instance_id,
            target,
            ..
        }
        | BattleLogEvent::ManualCastStart {
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
        BattleLogEvent::AutoCastEnd { caster_instance_id }
        | BattleLogEvent::ManualCastEnd { caster_instance_id } => {
            config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id
        }
        BattleLogEvent::SkillCastInterrupted {
            interrupter_instance_id,
            caster_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *interrupter_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *caster_instance_id == dead_unit_id)
        }
        BattleLogEvent::SkillCastCancelled {
            caster_instance_id, ..
        } => config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id,
        BattleLogEvent::TriggeredAbilityProc {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        BattleLogEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        BattleLogEvent::AbilityStepTriggered {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        BattleLogEvent::SkillAreaDeclared {
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
        BattleLogEvent::SkillProjectileLaunched {
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
        BattleLogEvent::SkillProjectileImpacted {
            caster_instance_id,
            first_hit_unit_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && first_hit_unit_id.is_some_and(|id| id == dead_unit_id))
        }
        BattleLogEvent::BuffApplied {
            target_instance_id, ..
        }
        | BattleLogEvent::BuffTick {
            target_instance_id, ..
        }
        | BattleLogEvent::BuffExpired {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        BattleLogEvent::HpChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        BattleLogEvent::StatChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        BattleLogEvent::ResonanceChanged {
            unit_instance_id, ..
        } => config.forbid_dead_units_as_targets && *unit_instance_id == dead_unit_id,
        BattleLogEvent::MovementSegmentStarted {
            unit_instance_id, ..
        }
        | BattleLogEvent::MovementStopped {
            unit_instance_id, ..
        } => config.forbid_dead_units_as_attackers && *unit_instance_id == dead_unit_id,
        BattleLogEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => *owner_unit_instance_id == dead_unit_id,
        BattleLogEvent::UnitSpawned {
            unit_instance_id, ..
        }
        | BattleLogEvent::UnitDeployed {
            unit_instance_id, ..
        }
        | BattleLogEvent::UnitWithdrawn {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        BattleLogEvent::UnitDied {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        BattleLogEvent::BattleStart { .. }
        | BattleLogEvent::ArtifactSpawned { .. }
        | BattleLogEvent::BattleEnd { .. } => false,
    }
}
