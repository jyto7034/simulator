use crate::{
    authoring::control,
    game::{
        battle::{
            event_log::{
                AttackDelivery, AttackKind, BattleEventCause, BattleLogEvent, HpChangeReason,
            },
            ids::UnitInstanceId,
            runtime_facts::{ActionFailureReason, ActionTerminalKind},
            tile_range::{TileRangePolicy, TileReachQuery},
        },
        data::abnormality_data::BasicAttackAction,
    },
};

use super::{BattleCore, RuntimeUnit};

fn effective_basic_heal_amount(unit: &RuntimeUnit) -> Option<i32> {
    let BasicAttackAction::Heal {
        flat,
        spell_power_percent,
        ..
    } = &unit.basic_attack.action
    else {
        return None;
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
                    && self.is_basic_attack_target_in_range(actor, target.instance_id)
            })
    }

    pub(super) fn choose_basic_heal_target(&self, actor: UnitInstanceId) -> Option<UnitInstanceId> {
        control::charge(self.units.len() as u64 + 1, "basic_heal_selection").ok()?;
        self.authoring_target_candidates()
            .ok()?
            .filter(|target| self.is_basic_heal_target(actor, target.instance_id))
            .min_by(|a, b| {
                (u64::from(a.stats.current_health) * u64::from(b.stats.max_health.max(1)))
                    .cmp(
                        &(u64::from(b.stats.current_health) * u64::from(a.stats.max_health.max(1))),
                    )
                    .then_with(|| {
                        self.basic_attack_target_distance_sq_world(actor, a.instance_id)
                            .unwrap_or(f32::INFINITY)
                            .total_cmp(
                                &self
                                    .basic_attack_target_distance_sq_world(actor, b.instance_id)
                                    .unwrap_or(f32::INFINITY),
                            )
                    })
                    .then_with(|| a.instance_id.cmp(&b.instance_id))
            })
            .map(|target| target.instance_id)
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
                    core.resolve_basic_heal(actor, target, time_ms)
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

    pub(super) fn resolve_basic_heal(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        time_ms: u64,
    ) -> bool {
        if !self.is_basic_heal_target(actor, target) {
            return false;
        }
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
        let BasicAttackAction::Heal { footprint, .. } = unit.basic_attack.action.clone() else {
            return false;
        };
        let owner = unit.owner;
        let facing = unit.facing_direction;
        let air_capable = unit.basic_attack.air_capable;
        let Some(anchor) = self.live_unit_projected_tile(target) else {
            return false;
        };
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
        let targets = if let Some(footprint) = &footprint {
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
            let mut targets: Vec<_> = candidates
                .filter(|unit| {
                    unit.owner == owner
                        && unit.role != crate::game::battle::types::BattleUnitRole::IndependentWall
                        && self.single_target_can_target_unit(unit.instance_id, air_capable)
                        && unit.stats.current_health < unit.stats.max_health
                        && self
                            .unit_body_view(unit.instance_id)
                            .is_some_and(|body| query.contains(body.position.project_to_tile()))
                })
                .map(|unit| unit.instance_id)
                .collect();
            targets.sort_unstable();
            targets
        } else {
            vec![target]
        };
        for target in targets {
            self.apply_hp_delta_and_record(
                Some(actor),
                target,
                amount,
                time_ms,
                HpChangeReason::Command,
            );
        }
        self.consume_basic_overlay(actor, time_ms).is_ok()
    }
}
