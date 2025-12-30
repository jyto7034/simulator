use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::SkillId,
        battle::{
            buffs::BuffId,
            cooldown::CooldownSource,
            timeline::{AttackKind, HpChangeReason, TimelineEvent},
        },
        data::GameDataBase,
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

// NOTE: Movement refactor in progress. Replay execution/ability plumbing is temporarily stubbed
// to allow compiling and iterating on movement logic.
#[derive(Debug, Clone)]
struct AbilityExecutor;

#[derive(Debug, Clone, Copy)]
struct PendingScheduledAttack {
    attacker_instance_id: Uuid,
    earliest_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerifiedCauseKind {
    Attack,
    Ability,
    BuffTick,
}

#[derive(Debug, Clone)]
enum ExpectedDecision {
    AbilityCast {
        time_ms: u64,
        ability_id: SkillId,
        caster_instance_id: Uuid,
        target_instance_id: Option<Uuid>,
        cooldown_source: CooldownSource,
    },
    Attack {
        time_ms: u64,
        attacker_instance_id: Uuid,
        target_instance_id: Uuid,
        kind: AttackKind,
    },
    BuffApplied {
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        duration_ms: u64,
    },
}

impl ExpectedDecision {
    fn matches(&self, time_ms: u64, event: &TimelineEvent) -> bool {
        match (self, event) {
            (
                ExpectedDecision::AbilityCast {
                    time_ms: expected_time,
                    ability_id,
                    caster_instance_id,
                    target_instance_id,
                    ..
                },
                TimelineEvent::AbilityCast {
                    skill_id: actual_ability,
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                },
            ) => {
                *expected_time == time_ms
                    && *ability_id == *actual_ability
                    && *caster_instance_id == *actual_caster
                    && *target_instance_id == *actual_target
            }
            (
                ExpectedDecision::Attack {
                    time_ms: expected_time,
                    attacker_instance_id,
                    target_instance_id,
                    kind,
                },
                TimelineEvent::Attack {
                    attacker_instance_id: actual_attacker,
                    target_instance_id: actual_target,
                    kind: actual_kind,
                },
            ) => {
                *expected_time == time_ms
                    && *attacker_instance_id == *actual_attacker
                    && *target_instance_id == *actual_target
                    && Some(*kind) == *actual_kind
            }
            (
                ExpectedDecision::BuffApplied {
                    time_ms: expected_time,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    duration_ms,
                },
                TimelineEvent::BuffApplied {
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                    buff_id: actual_buff,
                    duration_ms: actual_duration,
                },
            ) => {
                *expected_time == time_ms
                    && *caster_instance_id == *actual_caster
                    && *target_instance_id == *actual_target
                    && *buff_id == *actual_buff
                    && *duration_ms == *actual_duration
            }
            _ => false,
        }
    }

    fn describe(&self) -> String {
        match self {
            ExpectedDecision::AbilityCast {
                time_ms,
                ability_id,
                caster_instance_id,
                target_instance_id,
                cooldown_source,
            } => format!(
                "AbilityCast(time_ms={}, ability_id={:?}, caster={}, target={:?}, cooldown_source={:?})",
                time_ms, ability_id, caster_instance_id, target_instance_id, cooldown_source
            ),
            ExpectedDecision::Attack {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                kind,
            } => format!(
                "Attack(time_ms={}, attacker={}, target={}, kind={:?})",
                time_ms, attacker_instance_id, target_instance_id, kind
            ),
            ExpectedDecision::BuffApplied {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } => format!(
                "BuffApplied(time_ms={}, caster={}, target={}, buff_id={}, duration_ms={})",
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id.as_u64(),
                duration_ms
            ),
        }
    }
}

#[derive(Debug, Clone)]
enum ExpectedOutcome {
    HpChanged {
        source_instance_id: Option<Uuid>,
        target_instance_id: Uuid,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
    },
    StatChanged {
        source_instance_id: Option<Uuid>,
        target_instance_id: Uuid,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    UnitDied {
        unit_instance_id: Uuid,
        owner: Side,
        killer_instance_id: Option<Uuid>,
    },
}

impl ExpectedOutcome {
    fn matches(&self, event: &TimelineEvent) -> bool {
        match (self, event) {
            (
                ExpectedOutcome::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    delta,
                    hp_before,
                    hp_after,
                    reason,
                },
                TimelineEvent::HpChanged {
                    source_instance_id: actual_source,
                    target_instance_id: actual_target,
                    delta: actual_delta,
                    hp_before: actual_before,
                    hp_after: actual_after,
                    reason: actual_reason,
                },
            ) => {
                *source_instance_id == *actual_source
                    && *target_instance_id == *actual_target
                    && *delta == *actual_delta
                    && *hp_before == *actual_before
                    && *hp_after == *actual_after
                    && *reason == *actual_reason
            }
            (
                ExpectedOutcome::StatChanged {
                    source_instance_id,
                    target_instance_id,
                    modifier,
                    stats_before,
                    stats_after,
                },
                TimelineEvent::StatChanged {
                    source_instance_id: actual_source,
                    target_instance_id: actual_target,
                    modifier: actual_modifier,
                    stats_before: actual_before,
                    stats_after: actual_after,
                },
            ) => {
                *source_instance_id == *actual_source
                    && *target_instance_id == *actual_target
                    && modifier.stat == actual_modifier.stat
                    && modifier.kind == actual_modifier.kind
                    && modifier.value == actual_modifier.value
                    && stats_before.max_health == actual_before.max_health
                    && stats_before.current_health == actual_before.current_health
                    && stats_before.attack == actual_before.attack
                    && stats_before.defense == actual_before.defense
                    && stats_before.attack_interval_ms == actual_before.attack_interval_ms
                    && stats_after.max_health == actual_after.max_health
                    && stats_after.current_health == actual_after.current_health
                    && stats_after.attack == actual_after.attack
                    && stats_after.defense == actual_after.defense
                    && stats_after.attack_interval_ms == actual_after.attack_interval_ms
            }
            (
                ExpectedOutcome::UnitDied {
                    unit_instance_id,
                    owner,
                    killer_instance_id,
                },
                TimelineEvent::UnitDied {
                    unit_instance_id: actual_unit,
                    owner: actual_owner,
                    killer_instance_id: actual_killer,
                },
            ) => {
                *unit_instance_id == *actual_unit
                    && *owner == *actual_owner
                    && *killer_instance_id == *actual_killer
            }
            _ => false,
        }
    }

    fn describe(&self) -> String {
        match self {
            ExpectedOutcome::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
            } => format!(
                "HpChanged(source={:?}, target={}, delta={}, before={}, after={}, reason={:?})",
                source_instance_id, target_instance_id, delta, hp_before, hp_after, reason
            ),
            ExpectedOutcome::StatChanged {
                target_instance_id,
                modifier,
                ..
            } => format!(
                "StatChanged(target={}, modifier={:?}:{:?} {})",
                target_instance_id, modifier.stat, modifier.kind, modifier.value
            ),
            ExpectedOutcome::UnitDied {
                unit_instance_id,
                owner,
                killer_instance_id,
            } => format!(
                "UnitDied(unit={}, owner={:?}, killer={:?})",
                unit_instance_id, owner, killer_instance_id
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeUnit {
    instance_id: Uuid,
    owner: Side,
    base_uuid: Uuid,
    stats: UnitStats,
    position: Position,
    current_target: Option<Uuid>,
    resonance_current: u32,
    resonance_max: u32,
    resonance_lock_ms: u64,
    resonance_gain_locked_until_ms: u64,
    casting_until_ms: u64,
    pending_cast: bool,
}

#[derive(Debug, Clone)]
struct RuntimeArtifact {
    instance_id: Uuid,
    owner: Side,
    base_uuid: Uuid,
}

#[derive(Debug, Clone)]
struct RuntimeItem {
    instance_id: Uuid,
    owner_unit_instance: Uuid,
    base_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BuffInstanceKey {
    caster_instance_id: Uuid,
    target_instance_id: Uuid,
    buff_id: BuffId,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    stacks: u8,
    expires_at_ms: u64,
    next_tick_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: Uuid },
}

struct ReplayState {
    game_data: Arc<GameDataBase>,
    ability_executor: AbilityExecutor,
    units: HashMap<Uuid, RuntimeUnit>,
    known_units: HashSet<Uuid>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    buffs: HashMap<BuffInstanceKey, ActiveBuff>,
}
