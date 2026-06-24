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

use super::{movement::ActionState, BattleCore};

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
        let in_range = |id: UnitInstanceId| {
            self.is_alive_enemy(id, attacker.owner)
                && self.is_basic_attack_target_in_range(attacker_instance_id, id)
                && self.is_basic_attack_useful_target(attacker_instance_id, id)
        };

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
                return;
            };
            (
                !attacker.is_active(),
                attacker.can_basic_attack(),
                attacker.next_basic_attack_ms,
                attacker.action_locks.can_basic_attack(current_time_ms),
                attacker.action_locks.basic_attack_until_ms,
                attacker.current_target,
                attacker.stats.attack_interval_ms.max(1),
            )
        };
        if is_dead || !can_basic_attack {
            return;
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
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: reposition_until,
                attacker_instance_id,
                target_instance_id: None,
                schedule_next,
                cause,
            });
            return;
        }

        if schedule_next && current_time_ms < next_ready_ms {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: next_ready_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            });
            return;
        }

        if let Some(hard_cc_release_time_ms) =
            self.active_hard_cc_release_time_ms(attacker_instance_id, current_time_ms)
        {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: hard_cc_release_time_ms.max(next_ready_ms),
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            });
            return;
        }

        if !can_attack {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: if schedule_next {
                    lock_until.max(next_ready_ms)
                } else {
                    lock_until
                },
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            });
            return;
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
            return;
        }

        let target_id = target.unwrap();
        let stopped_movement = self.interrupt_movement(
            time_ms,
            attacker_instance_id,
            crate::game::battle::event_log::MovementStopReason::AttackStarted,
            None,
            ActionState::Idle,
        );
        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
            attacker.current_target = Some(target_id);
            attacker.pending_basic_attack = false;
        }

        let _ = stopped_movement;

        let windup_ms = self
            .units
            .get(&attacker_instance_id)
            .map(|unit| unit.basic_attack.effective_windup_ms() as u64)
            .unwrap_or(0);
        let resolve_time = time_ms.saturating_add(windup_ms);

        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
            attacker.action_locks.lock_basic_attack_until(resolve_time);
            attacker.action_locks.lock_movement_until(resolve_time);
        }

        let attack_kind = if schedule_next {
            AttackKind::Auto
        } else {
            AttackKind::Triggered
        };
        let attack_delivery = self.basic_attack_delivery_for_unit(attacker_instance_id);
        let Some((source_attack, damage_type)) = self
            .units
            .get(&attacker_instance_id)
            .map(|unit| (unit.stats.attack, unit.basic_attack.damage_type))
        else {
            return;
        };
        let Some(source_snapshot) = self.damage_source_snapshot_for_unit(
            attacker_instance_id,
            target_id,
            DamageSource::BasicAttack,
            damage_type,
            source_attack,
            DamageModifiers::default(),
            1,
            time_ms,
            true,
        ) else {
            return;
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

        self.event_queue.push(BattleEvent::AttackResolve {
            time_ms: resolve_time,
            attacker_instance_id,
            target_instance_id: target_id,
            source_snapshot,
            kind: attack_kind,
            delivery: attack_delivery,
            cause: BattleEventCause::Parent { seq: start_seq },
        });

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
    }

    pub(super) fn handle_basic_attack_resolve_event(
        &mut self,
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        source_snapshot: crate::game::battle::damage::DamageSourceSnapshot,
        kind: AttackKind,
        delivery: crate::game::battle::event_log::AttackDelivery,
        cause: BattleEventCause,
    ) {
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
    }
}
