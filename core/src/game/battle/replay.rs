use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::ecs::resources::Position;
use crate::game::{
    battle::{
        ability_executor::{AbilityExecutor, AbilityRequest, UnitSnapshot},
        buffs::{self, BuffId},
        damage::{calculate_damage, DamageContext, DamageRequest, DamageSource},
        timeline::{HpChangeReason, Timeline, TimelineEvent},
    },
    data::GameDataBase,
    enums::Side,
    stats::{Effect, StatModifier, TriggerType, UnitStats},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineReplayViolationKind {
    UnknownUnitReference,
    UnknownItemReference,
    UnknownArtifactReference,
    AttackDuringCast,
    AutoCastWithoutFullResonance,
    ExpectedOutcomeMissing,
    UnexpectedOutcome,
    OutcomeMismatch,
}

#[derive(Debug, Clone)]
pub struct TimelineReplayViolation {
    pub kind: TimelineReplayViolationKind,
    pub message: String,
    pub entry_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TimelineReplayerConfig {
    pub validate_basic_attack_outcomes: bool,
    pub validate_ability_outcomes: bool,
    pub validate_buff_tick_outcomes: bool,
    pub validate_autocast_gating: bool,
    pub forbid_unexpected_outcomes_for_verified_causes: bool,
}

impl Default for TimelineReplayerConfig {
    fn default() -> Self {
        Self {
            validate_basic_attack_outcomes: true,
            validate_ability_outcomes: true,
            validate_buff_tick_outcomes: true,
            validate_autocast_gating: true,
            forbid_unexpected_outcomes_for_verified_causes: false,
        }
    }
}

pub struct TimelineReplayer {
    game_data: Arc<GameDataBase>,
    config: TimelineReplayerConfig,
}

impl TimelineReplayer {
    pub fn new(game_data: Arc<GameDataBase>, config: TimelineReplayerConfig) -> Self {
        Self { game_data, config }
    }

    pub fn replay(&self, timeline: &Timeline) -> Result<(), Vec<TimelineReplayViolation>> {
        let mut violations = Vec::new();
        let mut state = ReplayState::new(self.game_data.clone());
        let mut expectations: HashMap<u64, Vec<ExpectedOutcome>> = HashMap::new();
        let mut verified_causes: HashMap<u64, VerifiedCauseKind> = HashMap::new();

        for (index, entry) in timeline.entries.iter().enumerate() {
            match &entry.event {
                TimelineEvent::BattleStart { .. } => {}
                TimelineEvent::UnitSpawned {
                    unit_instance_id,
                    owner,
                    base_uuid,
                    position,
                    stats,
                } => {
                    state.spawn_unit(*unit_instance_id, *owner, *base_uuid, *position, *stats);
                }
                TimelineEvent::ItemSpawned {
                    item_instance_id,
                    owner,
                    owner_unit_instance_id,
                    base_uuid,
                } => {
                    state.spawn_item(
                        *item_instance_id,
                        *owner,
                        *owner_unit_instance_id,
                        *base_uuid,
                        &mut violations,
                        index,
                    );
                }
                TimelineEvent::ArtifactSpawned {
                    artifact_instance_id,
                    owner,
                    base_uuid,
                } => {
                    state.spawn_artifact(*artifact_instance_id, *owner, *base_uuid);
                }
                TimelineEvent::Attack {
                    attacker_instance_id,
                    target_instance_id,
                    kind: _,
                } => {
                    if self.config.validate_basic_attack_outcomes {
                        verified_causes.insert(entry.seq, VerifiedCauseKind::Attack);
                        state.predict_attack_outcomes(
                            entry.seq,
                            entry.time_ms,
                            *attacker_instance_id,
                            *target_instance_id,
                            &mut expectations,
                            &mut violations,
                            index,
                        );
                    }

                    state.on_attack_decision(
                        entry.time_ms,
                        *attacker_instance_id,
                        *target_instance_id,
                        &mut violations,
                        index,
                    );
                }
                TimelineEvent::AutoCastStart {
                    caster_instance_id,
                    ability_id,
                    target_instance_id,
                } => {
                    if self.config.validate_autocast_gating {
                        state.on_autocast_start(
                            entry.time_ms,
                            *caster_instance_id,
                            *ability_id,
                            *target_instance_id,
                            &mut violations,
                            index,
                        );
                    }
                }
                TimelineEvent::AutoCastEnd { caster_instance_id } => {
                    if self.config.validate_autocast_gating {
                        state.on_autocast_end(
                            entry.time_ms,
                            *caster_instance_id,
                            &mut violations,
                            index,
                        );
                    }
                }
                TimelineEvent::AbilityCast {
                    ability_id,
                    caster_instance_id,
                    target_instance_id,
                } => {
                    if self.config.validate_ability_outcomes {
                        verified_causes.insert(entry.seq, VerifiedCauseKind::Ability);
                        state.predict_ability_outcomes(
                            entry.seq,
                            entry.time_ms,
                            *ability_id,
                            *caster_instance_id,
                            *target_instance_id,
                            &mut expectations,
                            &mut violations,
                            index,
                        );
                    }
                }
                TimelineEvent::BuffApplied {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    duration_ms,
                } => {
                    state.on_buff_applied(
                        entry.time_ms,
                        *caster_instance_id,
                        *target_instance_id,
                        *buff_id,
                        *duration_ms,
                    );
                }
                TimelineEvent::BuffTick {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                } => {
                    if self.config.validate_buff_tick_outcomes {
                        verified_causes.insert(entry.seq, VerifiedCauseKind::BuffTick);
                        state.predict_buff_tick_outcomes(
                            entry.seq,
                            entry.time_ms,
                            *caster_instance_id,
                            *target_instance_id,
                            *buff_id,
                            &mut expectations,
                            &mut violations,
                            index,
                        );
                    }
                    state.on_buff_tick_decision(
                        entry.time_ms,
                        *caster_instance_id,
                        *target_instance_id,
                        *buff_id,
                        &mut violations,
                        index,
                    );
                }
                TimelineEvent::BuffExpired {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                } => {
                    state.on_buff_expired(
                        entry.time_ms,
                        *caster_instance_id,
                        *target_instance_id,
                        *buff_id,
                    );
                }
                TimelineEvent::HpChanged { .. }
                | TimelineEvent::StatChanged { .. }
                | TimelineEvent::UnitDied { .. } => {
                    let cause = entry.cause_seq;
                    if let Some(cause_seq) = cause {
                        if let Some(expected) = expectations.get_mut(&cause_seq) {
                            if let Some(pos) = expected
                                .iter()
                                .position(|e| e.matches(&entry.event))
                            {
                                expected.swap_remove(pos);
                            } else if self
                                .config
                                .forbid_unexpected_outcomes_for_verified_causes
                                && verified_causes.contains_key(&cause_seq)
                            {
                                violations.push(TimelineReplayViolation {
                                    kind: TimelineReplayViolationKind::UnexpectedOutcome,
                                    message: format!(
                                        "unexpected outcome for cause_seq {}: {:?}",
                                        cause_seq, entry.event
                                    ),
                                    entry_index: Some(index),
                                });
                            }
                        } else if self.config.forbid_unexpected_outcomes_for_verified_causes
                            && verified_causes.contains_key(&cause_seq)
                        {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::UnexpectedOutcome,
                                message: format!(
                                    "unexpected outcome for cause_seq {}: {:?}",
                                    cause_seq, entry.event
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }

                    state.apply_outcome_entry(entry.time_ms, &entry.event, &mut violations, index);
                }
                TimelineEvent::BattleEnd { .. } => {}
            }
        }

        for (cause_seq, remaining) in expectations {
            if remaining.is_empty() {
                continue;
            }
            if !verified_causes.contains_key(&cause_seq) {
                continue;
            }
            for expected in remaining {
                violations.push(TimelineReplayViolation {
                    kind: TimelineReplayViolationKind::ExpectedOutcomeMissing,
                    message: format!(
                        "missing expected outcome for cause_seq {}: {}",
                        cause_seq,
                        expected.describe()
                    ),
                    entry_index: None,
                });
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerifiedCauseKind {
    Attack,
    Ability,
    BuffTick,
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
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    buffs: HashMap<BuffInstanceKey, ActiveBuff>,
}

impl ReplayState {
    fn new(game_data: Arc<GameDataBase>) -> Self {
        Self {
            game_data,
            ability_executor: AbilityExecutor::new(),
            units: HashMap::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            buffs: HashMap::new(),
        }
    }

    fn spawn_unit(
        &mut self,
        unit_instance_id: Uuid,
        owner: Side,
        base_uuid: Uuid,
        position: Position,
        stats: UnitStats,
    ) {
        let (resonance_start, resonance_max, resonance_lock_ms) = self
            .game_data
            .abnormality_data
            .get_by_uuid(&base_uuid)
            .map(|meta| (meta.resonance_start, meta.resonance_max.max(1), meta.resonance_lock_ms))
            .unwrap_or((0, 100, 1000));
        let resonance_current = resonance_start.min(resonance_max);

        self.units.insert(
            unit_instance_id,
            RuntimeUnit {
                instance_id: unit_instance_id,
                owner,
                base_uuid,
                stats,
                position,
                current_target: None,
                resonance_current,
                resonance_max,
                resonance_lock_ms,
                resonance_gain_locked_until_ms: 0,
                casting_until_ms: 0,
                pending_cast: false,
            },
        );
    }

    fn spawn_item(
        &mut self,
        item_instance_id: Uuid,
        _owner: Side,
        owner_unit_instance_id: Uuid,
        base_uuid: Uuid,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        if !self.units.contains_key(&owner_unit_instance_id) {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::UnknownUnitReference,
                message: format!(
                    "item {} references unknown owner unit {}",
                    item_instance_id, owner_unit_instance_id
                ),
                entry_index: Some(entry_index),
            });
        }

        self.items.insert(
            item_instance_id,
            RuntimeItem {
                instance_id: item_instance_id,
                owner_unit_instance: owner_unit_instance_id,
                base_uuid,
            },
        );
    }

    fn spawn_artifact(&mut self, artifact_instance_id: Uuid, owner: Side, base_uuid: Uuid) {
        self.artifacts.insert(
            artifact_instance_id,
            RuntimeArtifact {
                instance_id: artifact_instance_id,
                owner,
                base_uuid,
            },
        );
    }

    fn on_attack_decision(
        &mut self,
        time_ms: u64,
        attacker_instance_id: Uuid,
        target_instance_id: Uuid,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let (attacker_owner, casting_until_ms) = match self.units.get(&attacker_instance_id) {
            Some(attacker) => (attacker.owner, attacker.casting_until_ms),
            None => {
                violations.push(TimelineReplayViolation {
                    kind: TimelineReplayViolationKind::UnknownUnitReference,
                    message: format!("attack references unknown attacker {}", attacker_instance_id),
                    entry_index: Some(entry_index),
                });
                return;
            }
        };

        if time_ms < casting_until_ms {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::AttackDuringCast,
                message: format!(
                    "attack at {}ms occurs before casting_until_ms {}",
                    time_ms, casting_until_ms
                ),
                entry_index: Some(entry_index),
            });
        }

        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
            attacker.current_target = Some(target_instance_id);
            self.add_resonance(attacker_instance_id, 10, time_ms, true);
        }

        if !self.is_alive_enemy(target_instance_id, attacker_owner) {
            return;
        }
    }

    fn on_autocast_start(
        &mut self,
        time_ms: u64,
        caster_instance_id: Uuid,
        ability_id: Option<crate::game::ability::AbilityId>,
        target_instance_id: Option<Uuid>,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let Some((base_uuid, owner, current_target, resonance_current, resonance_max)) =
            self.units.get(&caster_instance_id).map(|caster| {
                (
                    caster.base_uuid,
                    caster.owner,
                    caster.current_target,
                    caster.resonance_current,
                    caster.resonance_max,
                )
            })
        else {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::UnknownUnitReference,
                message: format!("AutoCastStart references unknown caster {}", caster_instance_id),
                entry_index: Some(entry_index),
            });
            return;
        };

        let pending_cast = self
            .units
            .get(&caster_instance_id)
            .is_some_and(|caster| caster.pending_cast);
        let max = resonance_max.max(1);
        if resonance_current < max && !pending_cast {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::AutoCastWithoutFullResonance,
                message: format!(
                    "AutoCastStart for {} without full resonance (current={}, max={})",
                    caster_instance_id, resonance_current, max
                ),
                entry_index: Some(entry_index),
            });
        }

        let expected_ability = self
            .game_data
            .abnormality_data
            .get_by_uuid(&base_uuid)
            .and_then(|meta| meta.abilities.first().copied());
        if expected_ability != ability_id {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::OutcomeMismatch,
                message: format!(
                    "AutoCastStart ability_id mismatch for {}: expected {:?}, got {:?}",
                    caster_instance_id, expected_ability, ability_id
                ),
                entry_index: Some(entry_index),
            });
        }

        let expected_target = current_target.filter(|id| self.is_alive_enemy(*id, owner));
        if expected_target != target_instance_id {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::OutcomeMismatch,
                message: format!(
                    "AutoCastStart target mismatch for {}: expected {:?}, got {:?}",
                    caster_instance_id, expected_target, target_instance_id
                ),
                entry_index: Some(entry_index),
            });
        }

        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
            caster.pending_cast = false;
            caster.casting_until_ms = time_ms.saturating_add(1);
        }
    }

    fn on_autocast_end(
        &mut self,
        time_ms: u64,
        caster_instance_id: Uuid,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let Some(caster) = self.units.get_mut(&caster_instance_id) else {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::UnknownUnitReference,
                message: format!("AutoCastEnd references unknown caster {}", caster_instance_id),
                entry_index: Some(entry_index),
            });
            return;
        };

        caster.resonance_current = 0;
        caster.resonance_gain_locked_until_ms = time_ms.saturating_add(caster.resonance_lock_ms);
        caster.casting_until_ms = 0;
        caster.pending_cast = false;
    }

    fn on_buff_applied(
        &mut self,
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        duration_ms: u64,
    ) {
        let Some(def) = buffs::get(buff_id) else {
            return;
        };
        if duration_ms == 0 {
            return;
        }
        if !self.units.contains_key(&target_instance_id) {
            return;
        }

        let key = BuffInstanceKey {
            caster_instance_id,
            target_instance_id,
            buff_id,
        };

        let expires_at_ms = time_ms.saturating_add(duration_ms);
        let max_stacks = def.max_stacks.max(1);

        let entry = self.buffs.entry(key).or_insert(ActiveBuff {
            stacks: 0,
            expires_at_ms,
            next_tick_ms: None,
        });

        entry.expires_at_ms = entry.expires_at_ms.max(expires_at_ms);
        entry.stacks = entry.stacks.saturating_add(1).min(max_stacks);

        if def.tick_interval_ms > 0 && entry.next_tick_ms.is_none() {
            let tick_time_ms = time_ms.saturating_add(def.tick_interval_ms);
            if tick_time_ms < entry.expires_at_ms {
                entry.next_tick_ms = Some(tick_time_ms);
            }
        }
    }

    fn on_buff_tick_decision(
        &mut self,
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let Some(def) = buffs::get(buff_id) else {
            return;
        };
        if def.tick_interval_ms == 0 {
            return;
        }

        let key = BuffInstanceKey {
            caster_instance_id,
            target_instance_id,
            buff_id,
        };
        let Some(active) = self.buffs.get_mut(&key) else {
            return;
        };

        if time_ms >= active.expires_at_ms {
            return;
        }

        if active.next_tick_ms != Some(time_ms) {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::OutcomeMismatch,
                message: format!(
                    "BuffTick at {}ms does not match expected next_tick_ms {:?}",
                    time_ms, active.next_tick_ms
                ),
                entry_index: Some(entry_index),
            });
        }

        let next_tick_ms = time_ms.saturating_add(def.tick_interval_ms);
        if next_tick_ms < active.expires_at_ms {
            active.next_tick_ms = Some(next_tick_ms);
        } else {
            active.next_tick_ms = None;
        }
    }

    fn on_buff_expired(
        &mut self,
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
    ) {
        let key = BuffInstanceKey {
            caster_instance_id,
            target_instance_id,
            buff_id,
        };
        let Some(active) = self.buffs.get(&key) else {
            return;
        };
        if time_ms < active.expires_at_ms {
            return;
        }
        self.buffs.remove(&key);
    }

    fn predict_attack_outcomes(
        &self,
        cause_seq: u64,
        time_ms: u64,
        attacker_instance_id: Uuid,
        target_instance_id: Uuid,
        expectations: &mut HashMap<u64, Vec<ExpectedOutcome>>,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let (attacker_owner, attacker_attack) = match self.units.get(&attacker_instance_id) {
            Some(attacker) if attacker.stats.current_health > 0 => (attacker.owner, attacker.stats.attack),
            Some(_) => return,
            None => {
                violations.push(TimelineReplayViolation {
                    kind: TimelineReplayViolationKind::UnknownUnitReference,
                    message: format!("attack references unknown attacker {}", attacker_instance_id),
                    entry_index: Some(entry_index),
                });
                return;
            }
        };

        let (target_owner, target_defense, target_current_hp, target_max_hp) =
            match self.units.get(&target_instance_id) {
                Some(target) if target.stats.current_health > 0 => (
                    target.owner,
                    target.stats.defense,
                    target.stats.current_health,
                    target.stats.max_health,
                ),
                Some(_) => return,
                None => {
                    violations.push(TimelineReplayViolation {
                        kind: TimelineReplayViolationKind::UnknownUnitReference,
                        message: format!("attack references unknown target {}", target_instance_id),
                        entry_index: Some(entry_index),
                    });
                    return;
                }
            };

        if target_owner == attacker_owner {
            return;
        }

        let on_attack_effects = self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_instance_id, TriggerType::OnHit);

        let ctx = DamageContext {
            attacker_side: attacker_owner,
            target_side: target_owner,
            attacker_attack,
            target_defense,
            target_current_hp,
            target_max_hp,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: attacker_instance_id,
            target_id: target_instance_id,
            base_damage: attacker_attack,
            time_ms,
        };

        let result = calculate_damage(&request, &ctx);
        let hp_before = target_current_hp;
        let hp_after = target_current_hp.saturating_sub(result.final_damage);
        let delta = hp_after as i32 - hp_before as i32;

        expectations
            .entry(cause_seq)
            .or_default()
            .push(ExpectedOutcome::HpChanged {
                source_instance_id: Some(attacker_instance_id),
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason: HpChangeReason::BasicAttack,
            });

        if result.target_killed {
            expectations
                .entry(cause_seq)
                .or_default()
                .push(ExpectedOutcome::UnitDied {
                    unit_instance_id: target_instance_id,
                    owner: target_owner,
                    killer_instance_id: Some(attacker_instance_id),
                });
        }
    }

    fn predict_ability_outcomes(
        &mut self,
        cause_seq: u64,
        time_ms: u64,
        ability_id: crate::game::ability::AbilityId,
        caster_instance_id: Uuid,
        target_instance_id: Option<Uuid>,
        expectations: &mut HashMap<u64, Vec<ExpectedOutcome>>,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        let Some(caster) = self.units.get(&caster_instance_id) else {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::UnknownUnitReference,
                message: format!("AbilityCast references unknown caster {}", caster_instance_id),
                entry_index: Some(entry_index),
            });
            return;
        };
        if caster.stats.current_health == 0 {
            return;
        }

        let caster_snapshot = caster.to_snapshot();
        let unit_snapshots = self.alive_unit_snapshots();
        let request = AbilityRequest {
            ability_id,
            caster_id: caster_instance_id,
            target_id: target_instance_id,
            time_ms,
        };

        let result = self
            .ability_executor
            .execute(&request, &caster_snapshot, &unit_snapshots);
        if !result.executed {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::OutcomeMismatch,
                message: format!(
                    "AbilityCast {:?} for {} was recorded but executor returned executed=false",
                    ability_id, caster_instance_id
                ),
                entry_index: Some(entry_index),
            });
            return;
        }

        let mut shadow = self.clone_for_shadow();
        for command in result.commands {
            match command {
                crate::game::battle::damage::BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent,
                    source_id,
                } => {
                    if let Some(expected) =
                        shadow.apply_heal_expected(time_ms, target_id, flat, percent, source_id)
                    {
                        expectations.entry(cause_seq).or_default().push(expected);
                    }
                }
                crate::game::battle::damage::BattleCommand::ApplyModifier { target_id, modifier } => {
                    if let Some(expected) = shadow.apply_modifier_expected(time_ms, target_id, modifier) {
                        expectations.entry(cause_seq).or_default().push(expected);
                    }
                }
                _ => {}
            }
        }

        for (unit_id, dead) in shadow.killed_units {
            if dead {
                let owner = self
                    .units
                    .get(&unit_id)
                    .map(|u| u.owner)
                    .unwrap_or(Side::Opponent);
                expectations.entry(cause_seq).or_default().push(ExpectedOutcome::UnitDied {
                    unit_instance_id: unit_id,
                    owner,
                    killer_instance_id: shadow.killers.get(&unit_id).copied().flatten(),
                });
            }
        }
    }

    fn predict_buff_tick_outcomes(
        &mut self,
        cause_seq: u64,
        time_ms: u64,
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        expectations: &mut HashMap<u64, Vec<ExpectedOutcome>>,
        _violations: &mut Vec<TimelineReplayViolation>,
        _entry_index: usize,
    ) {
        let Some(def) = buffs::get(buff_id) else {
            return;
        };

        let key = BuffInstanceKey {
            caster_instance_id,
            target_instance_id,
            buff_id,
        };
        let Some(active) = self.buffs.get(&key) else {
            return;
        };

        let stacks = active.stacks.max(1) as i32;
        let dmg_per_tick = match def.kind {
            buffs::BuffKind::PeriodicDamage { damage_per_tick } => damage_per_tick as i32,
        };
        let dmg = dmg_per_tick.saturating_mul(stacks);
        if dmg <= 0 {
            return;
        }

        let mut shadow = self.clone_for_shadow();
        if let Some(expected) =
            shadow.apply_heal_expected(time_ms, target_instance_id, -dmg, 0, Some(caster_instance_id))
        {
            expectations.entry(cause_seq).or_default().push(expected);
        }

        for (unit_id, dead) in shadow.killed_units {
            if dead {
                let owner = self
                    .units
                    .get(&unit_id)
                    .map(|u| u.owner)
                    .unwrap_or(Side::Opponent);
                expectations.entry(cause_seq).or_default().push(ExpectedOutcome::UnitDied {
                    unit_instance_id: unit_id,
                    owner,
                    killer_instance_id: shadow.killers.get(&unit_id).copied().flatten(),
                });
            }
        }
    }

    fn apply_outcome_entry(
        &mut self,
        time_ms: u64,
        event: &TimelineEvent,
        violations: &mut Vec<TimelineReplayViolation>,
        entry_index: usize,
    ) {
        match event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                hp_before,
                hp_after,
                delta: _,
                reason: _,
            } => {
                let Some(target) = self.units.get_mut(target_instance_id) else {
                    violations.push(TimelineReplayViolation {
                        kind: TimelineReplayViolationKind::UnknownUnitReference,
                        message: format!("HpChanged references unknown target {}", target_instance_id),
                        entry_index: Some(entry_index),
                    });
                    return;
                };
                if target.stats.current_health != *hp_before {
                    violations.push(TimelineReplayViolation {
                        kind: TimelineReplayViolationKind::OutcomeMismatch,
                        message: format!(
                            "HpChanged hp_before mismatch for {}: state={}, event={}",
                            target_instance_id, target.stats.current_health, hp_before
                        ),
                        entry_index: Some(entry_index),
                    });
                }
                target.stats.current_health = *hp_after;

                if *hp_after < *hp_before {
                    let actual_decrease = hp_before.saturating_sub(*hp_after);
                    let gained = actual_decrease / 10;
                    if gained > 0 {
                        let allow_autocast = *hp_after > 0;
                        self.add_resonance(*target_instance_id, gained, time_ms, allow_autocast);
                    }
                }

                if let Some(source_id) = source_instance_id {
                    if *source_id != *target_instance_id {
                        let _ = self.units.get(source_id);
                    }
                }
            }
            TimelineEvent::StatChanged {
                target_instance_id,
                stats_before,
                stats_after,
                ..
            } => {
                let Some(target) = self.units.get_mut(target_instance_id) else {
                    violations.push(TimelineReplayViolation {
                        kind: TimelineReplayViolationKind::UnknownUnitReference,
                        message: format!(
                            "StatChanged references unknown target {}",
                            target_instance_id
                        ),
                        entry_index: Some(entry_index),
                    });
                    return;
                };
                if target.stats.current_health != stats_before.current_health
                    || target.stats.max_health != stats_before.max_health
                    || target.stats.attack != stats_before.attack
                    || target.stats.defense != stats_before.defense
                    || target.stats.attack_interval_ms != stats_before.attack_interval_ms
                {
                    violations.push(TimelineReplayViolation {
                        kind: TimelineReplayViolationKind::OutcomeMismatch,
                        message: format!("StatChanged stats_before mismatch for {}", target_instance_id),
                        entry_index: Some(entry_index),
                    });
                }
                target.stats = *stats_after;
            }
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } => {
                self.units.remove(unit_instance_id);
                self.buffs
                    .retain(|key, _| key.target_instance_id != *unit_instance_id);
                for unit in self.units.values_mut() {
                    if unit.current_target == Some(*unit_instance_id) {
                        unit.current_target = None;
                    }
                }
            }
            _ => {}
        }
    }

    fn is_alive_enemy(&self, unit_id: Uuid, owner: Side) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && unit.stats.current_health > 0,
            None => false,
        }
    }

    fn add_resonance(
        &mut self,
        unit_instance_id: Uuid,
        amount: u32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if amount == 0 {
            return;
        }
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };
        if unit.stats.current_health == 0 {
            return;
        }
        if now_ms < unit.resonance_gain_locked_until_ms || now_ms < unit.casting_until_ms {
            return;
        }

        let max = unit.resonance_max.max(1);
        let before = unit.resonance_current.min(max);
        let after = before.saturating_add(amount).min(max);
        unit.resonance_current = after;

        if allow_autocast_when_full && before < max && after == max {
            unit.pending_cast = true;
        }
    }

    fn alive_unit_snapshots(&self) -> Vec<UnitSnapshot> {
        let mut unit_snapshots: Vec<UnitSnapshot> = self
            .units
            .values()
            .filter(|u| u.stats.current_health > 0)
            .map(|u| u.to_snapshot())
            .collect();
        unit_snapshots.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
        unit_snapshots
    }

    fn collect_all_triggers(&self, unit_instance_id: Uuid, trigger: TriggerType) -> Vec<Effect> {
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Vec::new();
        };

        let mut effects = self.collect_triggers(TriggerSource::Artifact { side: unit.owner }, trigger);
        effects.extend(self.collect_triggers(TriggerSource::Item { unit_instance_id }, trigger));
        effects
    }

    fn collect_triggers(&self, source: TriggerSource, trigger: TriggerType) -> Vec<Effect> {
        let mut effects = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                let mut artifacts: Vec<&RuntimeArtifact> =
                    self.artifacts.values().filter(|a| a.owner == side).collect();
                artifacts.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for artifact in artifacts {
                    if let Some(metadata) = self.game_data.artifact_data.get_by_uuid(&artifact.base_uuid) {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned());
                        }
                    }
                }
            }
            TriggerSource::Item { unit_instance_id } => {
                let mut items: Vec<&RuntimeItem> = self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                    .collect();
                items.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for item in items {
                    if let Some(metadata) = self.game_data.equipment_data.get_by_uuid(&item.base_uuid) {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned());
                        }
                    }
                }
            }
        }

        effects
    }

    fn clone_for_shadow(&self) -> ShadowState {
        ShadowState {
            units: self.units.clone(),
            killed_units: HashMap::new(),
            killers: HashMap::new(),
        }
    }
}

impl RuntimeUnit {
    fn to_snapshot(&self) -> UnitSnapshot {
        UnitSnapshot {
            id: self.instance_id,
            owner: self.owner,
            position: self.position,
            stats: self.stats,
        }
    }
}

struct ShadowState {
    units: HashMap<Uuid, RuntimeUnit>,
    killed_units: HashMap<Uuid, bool>,
    killers: HashMap<Uuid, Option<Uuid>>,
}

impl ShadowState {
    fn apply_heal_expected(
        &mut self,
        _time_ms: u64,
        target_id: Uuid,
        flat: i32,
        percent: i32,
        source_id: Option<Uuid>,
    ) -> Option<ExpectedOutcome> {
        let unit = self.units.get_mut(&target_id)?;
        if unit.stats.current_health == 0 {
            return None;
        }

        let hp_before = unit.stats.current_health;
        let percent_delta = (unit.stats.max_health as i64) * (percent as i64) / 100;
        let delta = (flat as i64) + percent_delta;
        let hp_after = (hp_before as i64 + delta)
            .clamp(0, unit.stats.max_health as i64) as u32;
        unit.stats.current_health = hp_after;

        if hp_after == 0 {
            self.killed_units.insert(target_id, true);
            self.killers.insert(target_id, source_id);
        }

        let delta = hp_after as i32 - hp_before as i32;
        if delta == 0 {
            return None;
        }

        Some(ExpectedOutcome::HpChanged {
            source_instance_id: source_id,
            target_instance_id: target_id,
            delta,
            hp_before,
            hp_after,
            reason: HpChangeReason::Command,
        })
    }

    fn apply_modifier_expected(
        &mut self,
        _time_ms: u64,
        target_id: Uuid,
        modifier: StatModifier,
    ) -> Option<ExpectedOutcome> {
        let unit = self.units.get_mut(&target_id)?;
        if unit.stats.current_health == 0 {
            return None;
        }
        let stats_before = unit.stats;
        unit.stats.apply_modifier(modifier);
        let stats_after = unit.stats;
        Some(ExpectedOutcome::StatChanged {
            source_instance_id: None,
            target_instance_id: target_id,
            modifier,
            stats_before,
            stats_after,
        })
    }
}
