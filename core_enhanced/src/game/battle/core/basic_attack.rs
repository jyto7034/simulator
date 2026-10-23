use crate::game::{
    battle::{
        damage::{DamageModifiers, DamageSource},
        enums::BattleEvent,
        event_log::{AttackKind, BattleEventCause, BattleEventRootCause, BattleLogEvent},
        ids::UnitInstanceId,
    },
    data::equipment_data::WeaponRangeRole,
    enums::Side,
};

use super::BattleCore;

impl BattleCore {
    fn start_ranged_reposition_after_attack_release(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        time_ms: u64,
    ) {
        if self.blocked_by(attacker_instance_id).is_some() {
            return;
        }
        let Some(attacker) = self.units.get_mut(&attacker_instance_id) else {
            return;
        };
        if attacker.owner != Side::Opponent
            || attacker.basic_attack.range_role != WeaponRangeRole::Ranged
            || attacker.basic_attack.ranged_reposition_ms == 0
        {
            return;
        }
        attacker.ranged_reposition_until_ms = attacker
            .ranged_reposition_until_ms
            .max(time_ms.saturating_add(attacker.basic_attack.ranged_reposition_ms));
    }

    pub(in crate::game::battle::core) fn persisted_target_if_alive(
        &self,
        owner: Side,
        current_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        current_target.filter(|id| self.is_alive_enemy(*id, owner))
    }

    pub(in crate::game::battle::core) fn persisted_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
        current_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        self.persisted_target_if_alive(attacker.owner, current_target)
            .filter(|id| self.is_basic_attack_target_in_range(attacker_instance_id, *id))
    }

    pub(in crate::game::battle::core) fn select_basic_attack_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        current_target: Option<UnitInstanceId>,
        hinted_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        if attacker.basic_attack.action.is_heal() {
            return self.choose_basic_heal_target(attacker_instance_id);
        }
        let in_range = |id: UnitInstanceId| {
            self.is_alive_enemy(id, attacker.owner)
                && self.is_basic_attack_target_in_range(attacker_instance_id, id)
                && self.is_basic_attack_useful_target(attacker_instance_id, id)
        };

        if let Some(forced) = self.taunt_source(attacker_instance_id) {
            return in_range(forced).then_some(forced);
        }
        if let Some(wall) = self.blocked_by(attacker_instance_id).filter(|id| {
            self.units.get(id).is_some_and(|unit| {
                unit.role == crate::game::battle::types::BattleUnitRole::IndependentWall
            }) && in_range(*id)
        }) {
            return Some(wall);
        }
        if self.is_fixed_defense_route_enemy(attacker_instance_id) {
            if let Some(blocked_target) = self
                .blocked_by(attacker_instance_id)
                .filter(|id| in_range(*id))
            {
                return Some(blocked_target);
            }
            if attacker.is_airborne() {
                return self.airborne_enemy_basic_attack_target_in_range(attacker_instance_id);
            }
            if attacker.basic_attack.range_role == WeaponRangeRole::Ranged {
                return self
                    .persisted_target_in_range(attacker_instance_id, current_target)
                    .filter(|id| self.is_basic_attack_useful_target(attacker_instance_id, *id))
                    .or_else(|| self.choose_attack_target_in_range(attacker_instance_id));
            }
            return self
                .fixed_defense_route_end_target_for_enemy(attacker_instance_id)
                .filter(|id| in_range(*id));
        }

        if attacker.owner == Side::Player {
            if attacker.basic_attack.range_role == WeaponRangeRole::Melee {
                if let Some(blocked_target) = self
                    .first_blocked_enemy(attacker_instance_id)
                    .filter(|id| in_range(*id))
                {
                    return Some(blocked_target);
                }
            }

            return self
                .choose_attack_target_in_range(attacker_instance_id)
                .or_else(|| hinted_target.filter(|id| in_range(*id)))
                .or_else(|| self.persisted_target_in_range(attacker_instance_id, current_target));
        }

        self.blocked_by(attacker_instance_id)
            .filter(|id| in_range(*id))
            .or_else(|| hinted_target.filter(|id| in_range(*id)))
            .or_else(|| self.persisted_target_in_range(attacker_instance_id, current_target))
            .or_else(|| self.choose_attack_target_in_range(attacker_instance_id))
    }

    pub(super) fn try_start_pending_basic_attacks(&mut self, now_ms: u64) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for unit_id in unit_ids {
            if self.channel_pauses_basic(unit_id) || !self.boss_runtime_allows_combat_action(unit_id) {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.pending_basic_attack = false;
                }
                continue;
            }
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };
            let pending = unit.pending_basic_attack;
            let next_ready_ms = unit.next_basic_attack_ms;
            let can_attack = unit.action_locks.can_basic_attack(now_ms);
            let lock_until = unit.action_locks.basic_attack_until_ms;
            if !unit.is_active() || !unit.can_basic_attack() || !pending || now_ms < next_ready_ms {
                continue;
            }

            if !can_attack {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.pending_basic_attack = false;
                }
                self.event_queue.push(BattleEvent::AttackStart {
                    time_ms: lock_until,
                    attacker_instance_id: unit_id,
                    target_instance_id: None,
                    schedule_next: true,
                    cause: BattleEventCause::Root {
                        kind: BattleEventRootCause::Period,
                    },
                });
                continue;
            }

            let target = self.select_basic_attack_target(unit_id, unit.current_target, None);
            let Some(target_id) = target else {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = None;
                }
                continue;
            };

            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.pending_basic_attack = false;
            }

            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now_ms,
                attacker_instance_id: unit_id,
                target_instance_id: Some(target_id),
                schedule_next: true,
                cause: BattleEventCause::Root {
                    kind: BattleEventRootCause::Period,
                },
            });
        }
    }

    pub(super) fn handle_basic_attack_start_event(
        &mut self,
        time_ms: u64,
        current_time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        schedule_next: bool,
        cause: BattleEventCause,
    ) {
        let prior = self.runtime_fact_source.clone();
        let pending_ticket = prior.delivery.clone();
        let continued = self.handle_basic_attack_start_inner(
            time_ms,
            current_time_ms,
            attacker_instance_id,
            target_instance_id,
            schedule_next,
            cause,
        );
        self.runtime_fact_source = prior;
        if !continued {
            self.finish_use_attack(&self.runtime_fact_source.clone());
            if let Some(ticket) = pending_ticket {
                let _ = self.runtime_facts.close_delivery(&ticket, current_time_ms);
            }
        }
    }

    fn handle_basic_attack_start_inner(
        &mut self,
        time_ms: u64,
        current_time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        schedule_next: bool,
        cause: BattleEventCause,
    ) -> bool {
        if !self.use_attack_can_execute(&self.runtime_fact_source) {
            return false;
        }
        if self.channel_pauses_basic(attacker_instance_id)
            || !self.boss_runtime_allows_combat_action(attacker_instance_id)
        {
            if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                attacker.pending_basic_attack = false;
            }
            return false;
        }
        let (
            is_dead,
            can_basic_attack,
            next_ready_ms,
            can_attack,
            lock_until,
            current_target,
            interval_ms,
        ) = {
            let Some(attacker) = self.units.get(&attacker_instance_id) else {
                return false;
            };
            (
                !attacker.is_active(),
                attacker.can_basic_attack(),
                attacker.next_basic_attack_ms,
                attacker.action_locks.can_basic_attack(current_time_ms),
                attacker.action_locks.basic_attack_until_ms,
                attacker.current_target,
                self.sustained_basic_interval(
                    attacker_instance_id,
                    attacker.stats.attack_interval_ms.max(1),
                ),
            )
        };
        if is_dead || !can_basic_attack {
            return false;
        }

        if schedule_next && self.unit_is_in_defense_stance(attacker_instance_id) {
            if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                attacker.pending_basic_attack = true;
                attacker.next_basic_attack_ms = time_ms.max(next_ready_ms);
            }
            return false;
        }

        let reposition_until = self
            .units
            .get(&attacker_instance_id)
            .map(|unit| unit.ranged_reposition_until_ms)
            .unwrap_or(0);
        if schedule_next
            && current_time_ms < reposition_until
            && self.blocked_by(attacker_instance_id).is_none()
        {
            self.event_queue.push_with_context(
                BattleEvent::AttackStart {
                    time_ms: reposition_until,
                    attacker_instance_id,
                    target_instance_id: None,
                    schedule_next,
                    cause,
                },
                None,
                self.runtime_fact_source.clone(),
            );
            return true;
        }

        if schedule_next && current_time_ms < next_ready_ms {
            self.event_queue.push_with_context(
                BattleEvent::AttackStart {
                    time_ms: next_ready_ms,
                    attacker_instance_id,
                    target_instance_id,
                    schedule_next,
                    cause,
                },
                None,
                self.runtime_fact_source.clone(),
            );
            return true;
        }

        if schedule_next {
            if let Some(hard_cc_release_time_ms) =
                self.active_hard_cc_release_time_ms(attacker_instance_id, current_time_ms)
            {
                self.event_queue.push_with_context(
                    BattleEvent::AttackStart {
                        time_ms: hard_cc_release_time_ms.max(next_ready_ms),
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause,
                    },
                    None,
                    self.runtime_fact_source.clone(),
                );
                return true;
            }
        }

        if !can_attack {
            self.event_queue.push_with_context(
                BattleEvent::AttackStart {
                    time_ms: if schedule_next {
                        lock_until.max(next_ready_ms)
                    } else {
                        lock_until
                    },
                    attacker_instance_id,
                    target_instance_id,
                    schedule_next,
                    cause,
                },
                None,
                self.runtime_fact_source.clone(),
            );
            return true;
        }

        let hinted_target = if schedule_next {
            None
        } else {
            target_instance_id
        };
        let target =
            self.select_basic_attack_target(attacker_instance_id, current_target, hinted_target);

        if target.is_none() {
            if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                attacker.current_target = None;
            }
            if schedule_next {
                if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                    attacker.pending_basic_attack = true;
                    attacker.next_basic_attack_ms = time_ms;
                }
            }
            return false;
        }

        let target_id = target.unwrap();
        if self.runtime_fact_source.action.is_none()
            || !self.runtime_fact_source.allows_finite_children
        {
            let source = crate::game::battle::runtime_facts::RuntimeFactSource {
                extended_observation: match self
                    .independent_action_origin(attacker_instance_id)
                    .and_then(|origin| {
                        crate::game::battle::runtime_facts::RuntimeFactSource::extended_observation(
                            origin, None,
                        )
                    }) {
                    Ok(value) => value,
                    Err(_) => return false,
                },
                actor: Some(attacker_instance_id),
                ..Default::default()
            };
            let parent = self
                .independent_objects
                .objects
                .get(&attacker_instance_id)
                .map_or(&self.runtime_fact_source, |o| &o.source);
            let source = match source.inherit_causal_context(parent) {
                Ok(source) => source,
                Err(_) => return false,
            };
            let Ok(action) = self.runtime_facts.accept(
                attacker_instance_id,
                None,
                source.clone(),
                time_ms,
                false,
            ) else {
                return false;
            };
            let source = if let Some(causal) = self
                .runtime_facts
                .accepted_source(&action)
                .ok()
                .and_then(|s| s.causal().cloned())
            {
                match source.with_causal(Some(causal)) {
                    Ok(source) => source,
                    Err(_) => return false,
                }
            } else {
                source
            };
            self.runtime_fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
                actor: Some(attacker_instance_id),
                action: Some(action),
                allows_finite_children: true,
                ..source
            };
            self.runtime_fact_source =
                match self.reserve_finite_source(self.runtime_fact_source.clone()) {
                    Ok(source) => source,
                    Err(_) => return false,
                };
        }

        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
            attacker.current_target = Some(target_id);
            attacker.pending_basic_attack = false;
        }

        let windup_ms = self
            .units
            .get(&attacker_instance_id)
            .map(|unit| unit.basic_attack.effective_windup_ms() as u64)
            .unwrap_or(0);
        let resolve_time = time_ms.saturating_add(windup_ms);

        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
            attacker.action_locks.lock_basic_attack_until(resolve_time);
            attacker.action_locks.lock_movement_until(resolve_time);
            if schedule_next {
                attacker.basic_attack_windup_until_ms = Some(resolve_time);
            }
        }

        let attack_kind = if schedule_next {
            AttackKind::Auto
        } else {
            AttackKind::Triggered
        };
        let attack_delivery = self.basic_attack_delivery_for_unit(attacker_instance_id);
        let Some((source_attack, damage_type, attack_epoch)) =
            self.units.get(&attacker_instance_id).map(|unit| {
                (
                    unit.stats.attack,
                    unit.basic_attack.damage_type,
                    unit.basic_attack_epoch,
                )
            })
        else {
            return false;
        };

        let source_snapshot = if self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|unit| unit.basic_attack.action.is_heal())
        {
            None
        } else {
            let Some(mut source_snapshot) = self.damage_source_snapshot_for_regular_occurrence(
                attacker_instance_id,
                target_id,
                DamageSource::BasicAttack,
                damage_type,
                source_attack,
                DamageModifiers::default(),
                1,
                time_ms,
                true,
                schedule_next,
            ) else {
                return false;
            };
            if schedule_next && self.bind_relation_basic(&source_snapshot, attack_epoch).is_err() { return false; }
            if schedule_next && self.bind_prepared_basic(&mut source_snapshot, attack_epoch, time_ms).is_err() { return false; }
            if schedule_next && self.bind_weapon_marks(&mut source_snapshot).is_err() { return false; }
            if schedule_next && self.bind_presence(&mut source_snapshot).is_err() { return false; }
            if schedule_next && self.bind_weapon_growth(&mut source_snapshot, target_id).is_err() { return false; }
            Some(source_snapshot)
        };

        let start_seq = self.with_recording_context(cause, |core| {
            core.record_event_log(
                time_ms,
                BattleLogEvent::AttackStart {
                    attacker_instance_id,
                    target_instance_id: target_id,
                    kind: Some(attack_kind),
                    delivery: Some(attack_delivery),
                },
            )
        });

        let event = match source_snapshot {
            Some(source_snapshot) => BattleEvent::AttackResolve {
                time_ms: resolve_time,
                attacker_instance_id,
                target_instance_id: target_id,
                attack_epoch,
                source_snapshot,
                kind: attack_kind,
                delivery: attack_delivery,
                cause: BattleEventCause::Parent { seq: start_seq },
            },
            None => BattleEvent::BasicHealResolve {
                time_ms: resolve_time,
                attacker_instance_id,
                target_instance_id: target_id,
                attack_epoch,
                kind: attack_kind,
                cause: BattleEventCause::Parent { seq: start_seq },
            },
        };
        self.event_queue
            .push_with_context(event, None, self.runtime_fact_source.clone());

        if schedule_next {
            let next_time = time_ms.saturating_add(interval_ms);
            if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                attacker.next_basic_attack_ms = next_time;
            }
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: next_time,
                attacker_instance_id,
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::Root {
                    kind: BattleEventRootCause::Period,
                },
            });
        }
        true
    }

    pub(super) fn handle_basic_attack_resolve_event(
        &mut self,
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        attack_epoch: u64,
        source_snapshot: crate::game::battle::damage::DamageSourceSnapshot,
        kind: AttackKind,
        delivery: crate::game::battle::event_log::AttackDelivery,
        cause: BattleEventCause,
    ) {
        // Delivery is selected at release; expiry during windup restores the weapon.
        // Released projectiles retain their own immutable launch profile.
        let delivery = if self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|u| u.is_active())
        {
            self.basic_attack_delivery_for_unit(attacker_instance_id)
        } else {
            delivery
        };
        let source = source_snapshot.fact_source.clone();
        let action = source.action.clone();
        let ticket = source.delivery.clone();
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let released = self.handle_basic_attack_resolve_inner(
            time_ms,
            attacker_instance_id,
            target_instance_id,
            attack_epoch,
            source_snapshot,
            kind,
            delivery,
            cause,
        );
        self.finish_use_attack(&self.runtime_fact_source.clone());
        self.runtime_fact_source = prior;
        if let Some(action) = action {
            if self.runtime_facts.is_basic_action(&action) {
                if released {
                    let _ = self.runtime_facts.release(&action, 1, time_ms);
                    let _ = self.runtime_facts.dispatched(&action, 0, time_ms);
                } else {
                    let _ = self.runtime_facts.terminal(&action,
                        crate::game::battle::runtime_facts::ActionTerminalKind::Failed(crate::game::battle::runtime_facts::ActionFailureReason::OriginUnavailable), time_ms);
                }
            }
        }
        if !released || delivery == crate::game::battle::event_log::AttackDelivery::Instant {
            if let Some(ticket) = ticket {
                let _ = self.runtime_facts.close_delivery(&ticket, time_ms);
            }
        }
    }

    fn handle_basic_attack_resolve_inner(
        &mut self,
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        attack_epoch: u64,
        source_snapshot: crate::game::battle::damage::DamageSourceSnapshot,
        kind: AttackKind,
        delivery: crate::game::battle::event_log::AttackDelivery,
        cause: BattleEventCause,
    ) -> bool {
        if !self.use_attack_can_execute(&self.runtime_fact_source) {
            return false;
        }
        if self.channel_pauses_basic(attacker_instance_id) {
            return false;
        }
        if self.units.get(&attacker_instance_id).is_some_and(|unit| {
            matches!(
                unit.lifecycle,
                crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
            )
        }) {
            return false;
        }
        if kind == AttackKind::Auto
            && self
                .units
                .get(&attacker_instance_id)
                .is_none_or(|unit| unit.basic_attack_epoch != attack_epoch)
        {
            return false;
        }
        if kind == AttackKind::Auto {
            if let Some(unit) = self.units.get_mut(&attacker_instance_id) {
                if unit.basic_attack_windup_until_ms == Some(time_ms) {
                    unit.basic_attack_windup_until_ms = None;
                }
            }
        }

        let resolve_seq = self.with_recording_context(cause, |core| {
            core.record_event_log(
                time_ms,
                BattleLogEvent::AttackResolve {
                    attacker_instance_id,
                    target_instance_id,
                    kind: Some(kind),
                    delivery: Some(delivery),
                },
            )
        });

        let hit = self.with_recording_cause(resolve_seq, |core| {
            core.resolve_committed_basic_attack(
                attacker_instance_id,
                target_instance_id,
                source_snapshot,
                delivery,
                time_ms,
            )
        });

        self.start_ranged_reposition_after_attack_release(attacker_instance_id, time_ms);

        if !hit {
            self.with_recording_cause(resolve_seq, |core| {
                core.record_event_log(
                    time_ms,
                    BattleLogEvent::AttackMiss {
                        attacker_instance_id,
                        target_instance_id,
                        kind: Some(kind),
                        delivery: Some(delivery),
                    },
                )
            });
        }
        hit
    }
}
