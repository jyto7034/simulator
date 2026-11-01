use crate::{
    authoring::control,
    game::{
        battle::{
            event_log::{AttackDelivery, AttackKind, BattleEventCause, BattleLogEvent},
            ids::UnitInstanceId,
            runtime_facts::{ActionFailureReason, ActionTerminalKind},
            tile_range::{TileRangePolicy, TileReachQuery},
        },
        data::abnormality_data::BasicAttackAction,
    },
};

use super::{BattleCore, RuntimeUnit};

fn effective_basic_heal_amount(unit: &RuntimeUnit) -> Option<i32> {
    let (flat, spell_power_percent) = match &unit.basic_attack.action {
        BasicAttackAction::Heal {
            flat,
            spell_power_percent,
            ..
        }
        | BasicAttackAction::HealLowestHealth {
            flat,
            spell_power_percent,
            ..
        } => (flat, spell_power_percent),
        _ => return None,
    };
    let amount = u64::from(*flat)
        .saturating_add(u64::from(unit.stats.spell_power) * u64::from(*spell_power_percent) / 100)
        .min(i32::MAX as u64) as i32;
    (amount > 0).then_some(amount)
}

impl BattleCore {
    pub(super) fn is_basic_heal_target(
        &self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
    ) -> bool {
        self.units
            .get(&actor)
            .zip(self.units.get(&target))
            .is_some_and(|(source, target)| {
                source.is_active()
                    && effective_basic_heal_amount(source).is_some()
                    && target.is_active()
                    && target.role != crate::game::battle::types::BattleUnitRole::IndependentWall
                    && !self.stress_protected_target(target.instance_id)
                    && source.owner == target.owner
                    && target.stats.current_health < target.stats.max_health
                    && if matches!(
                        source.basic_attack.action,
                        BasicAttackAction::HealLowestHealth { .. }
                    ) {
                        let policy = self.basic_attack_range_policy_for_unit(source);
                        target.is_combatant()
                            && !target.corroded
                            && target.stats.current_health > 0
                            && self.single_target_can_target_unit(
                                target.instance_id,
                                policy.air_capable,
                            )
                            && self.is_target_in_tile_range_policy(
                                actor,
                                target.instance_id,
                                policy.range_policy,
                                policy.tile_range.as_deref(),
                            )
                    } else {
                        self.is_basic_attack_target_in_range(actor, target.instance_id)
                    }
            })
    }

    pub(super) fn choose_basic_heal_target(&self, actor: UnitInstanceId) -> Option<UnitInstanceId> {
        control::charge(self.units.len() as u64 + 1, "basic_heal_selection").ok()?;
        self.authoring_target_candidates()
            .ok()?
            .filter(|target| self.is_basic_heal_target(actor, target.instance_id))
            .min_by(|a, b| self.compare_basic_heal_targets(actor, a, b))
            .map(|target| target.instance_id)
    }

    fn compare_basic_heal_targets(
        &self,
        actor: UnitInstanceId,
        a: &RuntimeUnit,
        b: &RuntimeUnit,
    ) -> std::cmp::Ordering {
        (u64::from(a.stats.current_health) * u64::from(b.stats.max_health.max(1)))
            .cmp(&(u64::from(b.stats.current_health) * u64::from(a.stats.max_health.max(1))))
            .then_with(|| {
                if self.units.get(&actor).is_some_and(|unit| {
                    matches!(
                        unit.basic_attack.action,
                        BasicAttackAction::HealLowestHealth { .. }
                    )
                }) {
                    b.stats
                        .max_health
                        .saturating_sub(b.stats.current_health)
                        .cmp(&a.stats.max_health.saturating_sub(a.stats.current_health))
                } else {
                    self.basic_attack_target_distance_sq_world(actor, a.instance_id)
                        .unwrap_or(f32::INFINITY)
                        .total_cmp(
                            &self
                                .basic_attack_target_distance_sq_world(actor, b.instance_id)
                                .unwrap_or(f32::INFINITY),
                        )
                }
            })
            .then_with(|| a.instance_id.cmp(&b.instance_id))
    }

    pub(super) fn handle_basic_heal_resolve_event(
        &mut self,
        time_ms: u64,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        attack_epoch: u64,
        kind: AttackKind,
        cause: BattleEventCause,
    ) {
        let source = self.runtime_fact_source.clone();
        let valid = self.use_attack_can_execute(&source)
            && !self.channel_pauses_basic(actor)
            && self
                .active_hard_cc_release_time_ms(actor, time_ms)
                .is_none()
            && self.units.get(&actor).is_some_and(|unit| {
                unit.is_active()
                    && unit.can_basic_attack()
                    && unit.basic_attack_epoch == attack_epoch
                    && unit.basic_attack.action.is_heal()
            });
        if kind == AttackKind::Auto {
            if let Some(unit) = self.units.get_mut(&actor) {
                if unit.basic_attack_epoch == attack_epoch
                    && unit.basic_attack_windup_until_ms == Some(time_ms)
                {
                    unit.basic_attack_windup_until_ms = None;
                }
            }
        }
        let released = valid
            && self.with_recording_context(cause, |core| {
                let seq = core.record_event_log(
                    time_ms,
                    BattleLogEvent::AttackResolve {
                        attacker_instance_id: actor,
                        target_instance_id: target,
                        kind: Some(kind),
                        delivery: Some(AttackDelivery::Instant),
                    },
                );
                let released = core.with_recording_cause(seq, |core| {
                    core.resolve_basic_heal_kind(actor, target, time_ms, kind == AttackKind::Auto)
                });
                if !released {
                    core.with_recording_cause(seq, |core| {
                        core.record_event_log(
                            time_ms,
                            BattleLogEvent::AttackMiss {
                                attacker_instance_id: actor,
                                target_instance_id: target,
                                kind: Some(kind),
                                delivery: Some(AttackDelivery::Instant),
                            },
                        )
                    });
                }
                released
            });
        self.finish_use_attack(&source);
        if let Some(action) = source.action {
            if self.runtime_facts.is_basic_action(&action) {
                if released {
                    let _ = self.runtime_facts.release(&action, 1, time_ms);
                    let _ = self.runtime_facts.dispatched(&action, 0, time_ms);
                } else {
                    let _ = self.runtime_facts.terminal(
                        &action,
                        ActionTerminalKind::Failed(ActionFailureReason::OriginUnavailable),
                        time_ms,
                    );
                }
            }
        }
        if let Some(ticket) = source.delivery {
            let _ = self.runtime_facts.close_delivery(&ticket, time_ms);
        }
    }

    #[cfg(test)]
    pub(super) fn resolve_basic_heal(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        time_ms: u64,
    ) -> bool {
        self.resolve_basic_heal_kind(actor, target, time_ms, false)
    }

    fn resolve_basic_heal_kind(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        time_ms: u64,
        normal: bool,
    ) -> bool {
        let Some(unit) = self.units.get(&actor) else {
            return false;
        };
        let Some(amount) = effective_basic_heal_amount(unit) else {
            return false;
        };
        if control::charge_serialized(&unit.basic_attack, "basic_heal_release_profile_copy")
            .is_err()
        {
            return false;
        }
        let action = unit.basic_attack.action.clone();
        let owner = unit.owner;
        let facing = unit.facing_direction;
        let air_capable = unit.basic_attack.air_capable;
        if control::charge(self.units.len() as u64 + 1, "basic_heal_candidates").is_err()
            || crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.units
                    .len()
                    .checked_mul(std::mem::size_of::<UnitInstanceId>())
                    .and_then(|n| n.checked_add(64)),
                "basic_heal_targets",
            )
            .is_err()
        {
            return false;
        }
        let targets = if let BasicAttackAction::HealLowestHealth { max_targets, .. } = action {
            let count = self.units.len();
            if control::charge(
                (count as u64).saturating_mul(u64::from(count.max(1).ilog2()) + 1),
                "basic_heal_target_sort",
            )
            .is_err()
            {
                return false;
            }
            let Ok(candidates) = self.authoring_target_candidates() else {
                return false;
            };
            let mut targets = Vec::with_capacity(count);
            targets.extend(
                candidates
                    .filter(|unit| self.is_basic_heal_target(actor, unit.instance_id))
                    .map(|unit| unit.instance_id),
            );
            targets.sort_unstable_by(|a, b| {
                self.compare_basic_heal_targets(actor, &self.units[a], &self.units[b])
            });
            targets.truncate(max_targets as usize);
            targets
        } else {
            if !self.is_basic_heal_target(actor, target) {
                return false;
            }
            let BasicAttackAction::Heal { footprint, .. } = action else {
                return false;
            };
            let Some(anchor) = self.live_unit_projected_tile(target) else {
                return false;
            };
            if let Some(footprint) = &footprint {
                if Self::charge_authoring_tile_pattern(footprint, 2).is_err() {
                    return false;
                }
                let Ok(query) = TileReachQuery::try_new(
                    &self.battlefield,
                    TileRangePolicy::Pattern,
                    Some(footprint),
                    Some(anchor),
                    facing,
                ) else {
                    return false;
                };
                let Ok(candidates) = self.authoring_target_candidates() else {
                    return false;
                };
                let mut targets = Vec::with_capacity(self.units.len());
                targets.extend(
                    candidates
                        .filter(|unit| {
                            unit.owner == owner
                                && unit.role
                                    != crate::game::battle::types::BattleUnitRole::IndependentWall
                                && self.single_target_can_target_unit(unit.instance_id, air_capable)
                                && unit.stats.current_health < unit.stats.max_health
                                && self.unit_body_view(unit.instance_id).is_some_and(|body| {
                                    query.contains(body.position.project_to_tile())
                                })
                        })
                        .map(|unit| unit.instance_id),
                );
                targets.sort_unstable();
                targets
            } else {
                vec![target]
            }
        };
        if targets.is_empty() {
            return false;
        }
        let echo_eligible = normal && self.basic_heal_echo_action_eligible(actor);
        let mut actual_other_heal = 0u64;
        for target in targets {
            let before = if echo_eligible && self.equipment_support_pair_eligible(actor, target) {
                self.units.get(&target).map(|u| u.stats.current_health)
            } else {
                None
            };
            if self
                .apply_direct_heal_and_record(Some(actor), target, amount, time_ms)
                .is_err()
            {
                return false;
            }
            if let Some(before) = before {
                let actual = self
                    .units
                    .get(&target)
                    .map_or(0, |u| u.stats.current_health.saturating_sub(before));
                let Some(total) = actual_other_heal.checked_add(u64::from(actual)) else {
                    let _ = crate::game::battle::runtime_control::invariant("basic_heal_echo_sum");
                    return false;
                };
                actual_other_heal = total;
            }
            if control::poll("basic_heal_target_applied").is_err() {
                return false;
            }
        }
        if echo_eligible
            && self
                .basic_heal_echo_after_action(actor, actual_other_heal, time_ms)
                .is_err()
        {
            return false;
        }
        self.consume_basic_overlay(actor, time_ms).is_ok()
    }
}
