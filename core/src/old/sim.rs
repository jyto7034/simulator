use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        battle::{
            core::BattleCore,
            damage::BattleCommand,
            enums::BattleEvent,
            timeline::{AttackKind, Timeline, TimelineCause, TimelineEvent, TimelineRootCause},
            types::{BattleResult, BattleWinner},
        },
        behavior::GameError,
        enums::Side,
        stats::UnitStats,
    },
};

impl BattleCore {
    fn compute_winner(&self) -> Option<BattleWinner> {
        let mut player_alive = false;
        let mut opponent_alive = false;

        for unit in self.units.values() {
            if unit.stats.current_health == 0 {
                continue;
            }

            match unit.owner {
                Side::Player => player_alive = true,
                Side::Opponent => opponent_alive = true,
            }

            if player_alive && opponent_alive {
                return None;
            }
        }

        Some(match (player_alive, opponent_alive) {
            (true, false) => BattleWinner::Player,
            (false, true) => BattleWinner::Opponent,
            (false, false) => BattleWinner::Draw,
            (true, true) => unreachable!(),
        })
    }

    fn finish_battle(&mut self, time_ms: u64, winner: BattleWinner) -> BattleResult {
        self.record_timeline(time_ms, TimelineEvent::BattleEnd { winner });
        BattleResult {
            winner,
            timeline: self.timeline.clone(),
        }
    }

    pub fn init_intial_events(&mut self) {
        for unit in self.units.values() {
            self.event_queue.push(BattleEvent::Attack {
                time_ms: unit.stats.attack_interval_ms,
                attacker_instance_id: unit.instance_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::Period,
                },
            });
        }
    }

    pub fn run_battle(&mut self, _world: &mut World) -> Result<BattleResult, GameError> {
        self.units.clear();
        self.artifacts.clear();
        self.items.clear();
        self.graveyard.clear();
        self.buffs.clear();
        self.projectiles.clear();
        self.movement_field.clear();
        self.timeline = Timeline::new();
        self.timeline_seq = 0;
        self.projectile_seq = 0;
        self.recording_cause_stack.clear();

        self.build_runtime_units_from_decks(Side::Player)?;
        self.build_runtime_units_from_decks(Side::Opponent)?;
        self.build_runtime_field()?;

        self.with_recording_root(TimelineRootCause::Init, |core| {
            core.record_timeline(
                0,
                TimelineEvent::BattleStart {
                    width: core.movement_field.width,
                    height: core.movement_field.height,
                },
            );

            let mut unit_records: Vec<(Uuid, Side, Uuid, Position, UnitStats)> = core
                .units
                .values()
                .map(|u| (u.instance_id, u.owner, u.base_uuid, u.position, u.stats))
                .collect();
            unit_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (unit_instance_id, owner, base_uuid, position, stats) in unit_records {
                core.record_timeline(
                    0,
                    TimelineEvent::UnitSpawned {
                        unit_instance_id,
                        owner,
                        base_uuid,
                        position,
                        stats,
                    },
                );
            }

            let mut artifact_records: Vec<(Uuid, Side, Uuid)> = core
                .artifacts
                .values()
                .map(|a| (a.instance_id, a.owner, a.base_uuid))
                .collect();
            artifact_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (artifact_instance_id, owner, base_uuid) in artifact_records {
                core.record_timeline(
                    0,
                    TimelineEvent::ArtifactSpawned {
                        artifact_instance_id,
                        owner,
                        base_uuid,
                    },
                );
            }

            let mut item_records: Vec<(Uuid, Side, Uuid, Uuid)> = core
                .items
                .values()
                .map(|i| (i.instance_id, i.owner, i.owner_unit_instance, i.base_uuid))
                .collect();
            item_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            for (item_instance_id, owner, owner_unit_instance_id, base_uuid) in item_records {
                core.record_timeline(
                    0,
                    TimelineEvent::ItemSpawned {
                        item_instance_id,
                        owner,
                        owner_unit_instance_id,
                        base_uuid,
                    },
                );
            }
        });

        self.event_queue.clear();
        self.init_intial_events();
        // Initial movement planning.
        self.schedule_movement_intent(0);

        const MAX_BATTLE_TIME_MS: u64 = 60_000;

        let mut last_event_time_ms = 0;

        while let Some(next_time_ms) = self.event_queue.peek().map(|e| e.time_ms()) {
            let current_time_ms = next_time_ms;
            last_event_time_ms = current_time_ms;

            if current_time_ms > MAX_BATTLE_TIME_MS {
                return Ok(self.finish_battle(MAX_BATTLE_TIME_MS, BattleWinner::Draw));
            }

            // Expire per-unit soft occupancy at the start of this time slice.
            self.expire_soft_occupancy(current_time_ms);

            // Drain all events scheduled for this `current_time_ms` into a local bucket.
            let mut bucket: Vec<BattleEvent> = Vec::new();
            while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                bucket.push(self.event_queue.pop().unwrap());
            }

            // Process all combat events first (projectiles/buffs/attacks/etc). During combat,
            // more same-tick events may be scheduled; those must also be resolved before movement.
            let mut movement_intent = false;
            let mut move_steps: Vec<Uuid> = Vec::new();
            let mut combat: Vec<BattleEvent> = Vec::new();

            fn split_bucket(
                input: Vec<BattleEvent>,
                movement_intent: &mut bool,
                move_steps: &mut Vec<Uuid>,
                combat: &mut Vec<BattleEvent>,
            ) {
                for event in input {
                    match event {
                        BattleEvent::MovementIntent { .. } => *movement_intent = true,
                        BattleEvent::MoveStep {
                            unit_instance_id, ..
                        } => {
                            move_steps.push(unit_instance_id);
                        }
                        other => combat.push(other),
                    }
                }
            }

            split_bucket(bucket, &mut movement_intent, &mut move_steps, &mut combat);

            loop {
                combat.sort_by(|a, b| b.cmp(a)); // process order = heap pop order
                for event in combat.drain(..) {
                    self.process_event(event, current_time_ms)?;
                }

                // Pull newly scheduled same-tick events, if any.
                let mut new_bucket: Vec<BattleEvent> = Vec::new();
                while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                    new_bucket.push(self.event_queue.pop().unwrap());
                }

                if new_bucket.is_empty() {
                    break;
                }

                split_bucket(
                    new_bucket,
                    &mut movement_intent,
                    &mut move_steps,
                    &mut combat,
                );
            }

            // Battle may end during combat resolution.
            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }

            // Movement intent pass (BFS/reservations); no movement is applied here.
            if movement_intent {
                self.compute_movement_intents(current_time_ms);
            }

            // Apply move steps (position changes) after all combat events at this time.
            if !move_steps.is_empty() {
                self.handle_move_steps_at(current_time_ms, move_steps);

                // Movement may schedule a same-tick movement intent (to react to newly freed tiles).
                let mut followup_intent = false;
                while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                    match self.event_queue.pop().unwrap() {
                        BattleEvent::MovementIntent { .. } => followup_intent = true,
                        BattleEvent::MoveStep {
                            unit_instance_id, ..
                        } => {
                            // A MoveStep should never be scheduled for the same tick as a result
                            // of applying a step; ignore defensively.
                            let _ = unit_instance_id;
                        }
                        other => {
                            // Combat events are not expected during movement; re-queue them.
                            self.event_queue.push(other);
                            break;
                        }
                    }
                }
                if followup_intent {
                    self.compute_movement_intents(current_time_ms);
                }
            }

            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }
        }

        let end_time_ms = last_event_time_ms.min(MAX_BATTLE_TIME_MS);
        let winner = self.compute_winner().unwrap_or(BattleWinner::Draw);
        Ok(self.finish_battle(end_time_ms, winner))
    }

    fn is_alive_enemy(&self, unit_id: Uuid, owner: Side) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && unit.stats.current_health > 0,
            None => false,
        }
    }

    fn find_nearest_alive_enemy(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.units.get(&from_uuid)?.position;
        let mut nearest: Option<(Uuid, i32)> = None;

        for unit in self.units.values() {
            if unit.stats.current_health == 0 {
                continue;
            }
            if unit.owner == from_side {
                continue;
            }

            let distance = from_pos.chebyshev(&unit.position);
            match nearest {
                None => nearest = Some((unit.instance_id, distance)),
                Some((_best_uuid, best_dist)) if distance < best_dist => {
                    nearest = Some((unit.instance_id, distance));
                }
                Some((best_uuid, best_dist))
                    if distance == best_dist
                        && unit.instance_id.as_bytes() < best_uuid.as_bytes() =>
                {
                    nearest = Some((unit.instance_id, distance));
                }
                _ => {}
            }
        }

        nearest.map(|(uuid, _)| uuid)
    }

    pub(super) fn process_event(
        &mut self,
        event: BattleEvent,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::Attack {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => {
                let Some(attacker) = self.units.get(&attacker_instance_id) else {
                    return Ok(());
                };

                if attacker.stats.current_health == 0 {
                    return Ok(());
                }

                // Rule: while moving/ghosting, a unit cannot take actions. Attacks keep their
                // periodic schedule but do not execute until movement ends.
                if matches!(
                    attacker.move_state,
                    super::MoveState::Moving | super::MoveState::Ghost
                ) {
                    if schedule_next {
                        let interval_ms = attacker.stats.attack_interval_ms.max(1);
                        self.event_queue.push(BattleEvent::Attack {
                            time_ms: time_ms.saturating_add(interval_ms),
                            attacker_instance_id,
                            target_instance_id: None,
                            schedule_next: true,
                            cause: TimelineCause::Root {
                                kind: TimelineRootCause::Period,
                            },
                        });
                    }
                    return Ok(());
                }

                // 하드 CC 등, 공격이 지연됐을때, 재스케줄링
                if current_time_ms < attacker.next_action_time {
                    self.event_queue.push(BattleEvent::Attack {
                        time_ms: attacker.next_action_time,
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause,
                    });
                    return Ok(());
                }

                let owner = attacker.owner;
                let current_target = attacker.current_target;

                let hinted_target = target_instance_id.filter(|id| self.is_alive_enemy(*id, owner));
                let persisted_target = current_target.filter(|id| self.is_alive_enemy(*id, owner));

                // 지정된 타겟 ( target_instance_id ) 이 None 이면 persisted 타겟을. 하지만 그것도 None 이라면 새로 타겟을 찾음.
                let target = hinted_target
                    .or(persisted_target)
                    .or_else(|| self.find_nearest_alive_enemy(attacker_instance_id, owner));

                let attack_seq = if let Some(target_id) = target {
                    // target 업데이트
                    if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                        attacker.current_target = Some(target_id);
                    }

                    // 현재 Attack 을 Timeline 에 기록
                    Some(self.with_recording_context(cause, |core| {
                        core.record_timeline(
                            time_ms,
                            TimelineEvent::Attack {
                                attacker_instance_id,
                                target_instance_id: target_id,
                                kind: Some(if schedule_next {
                                    AttackKind::Auto
                                } else {
                                    AttackKind::Triggered
                                }),
                            },
                        )
                    }))
                } else {
                    None
                };

                if let Some(attack_seq) = attack_seq {
                    self.with_recording_cause(attack_seq, |core| {
                        core.apply_attack(attacker_instance_id, current_time_ms);
                    })
                } else {
                    self.apply_attack(attacker_instance_id, current_time_ms);
                }

                if schedule_next {
                    let Some(attacker) = self.units.get(&attacker_instance_id) else {
                        return Ok(());
                    };
                    if attacker.stats.current_health == 0 {
                        return Ok(());
                    }

                    let interval_ms = attacker.stats.attack_interval_ms.max(1);
                    self.event_queue.push(BattleEvent::Attack {
                        time_ms: time_ms.saturating_add(interval_ms),
                        attacker_instance_id,
                        target_instance_id: None,
                        schedule_next: true,
                        cause: TimelineCause::Root {
                            kind: TimelineRootCause::Period,
                        },
                    });
                }

                Ok(())
            }
            BattleEvent::ProjectileHit {
                time_ms,
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                cause,
            } => {
                self.with_recording_context(cause, |core| {
                    core.apply_projectile_hit(
                        time_ms,
                        projectile_id,
                        attacker_instance_id,
                        target_instance_id,
                    );
                });

                Ok(())
            }
            BattleEvent::AutoCastStart {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                let Some(caster) = self.units.get(&caster_instance_id) else {
                    return Ok(());
                };
                if caster.stats.current_health == 0 {
                    return Ok(());
                }
                if matches!(
                    caster.move_state,
                    super::MoveState::Moving | super::MoveState::Ghost
                ) {
                    // While moving, do not poll/reschedule per tick. Keep the cast pending and
                    // let movement completion schedule the actual AutoCastStart.
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }

                if let super::MoveState::CCLocked { until_ms } = caster.move_state {
                    // Hard CC: keep pending and let the CC release hook schedule the cast.
                    let _ = until_ms;
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    return Ok(());
                }

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };

                // Instant cast is modeled as [time_ms, time_ms + 1) to block resonance gain.
                let cast_end_ms = time_ms.saturating_add(1);
                caster.next_action_time = cast_end_ms;

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        // TODO: skill id 제대로 설정 해야함.
                        TimelineEvent::AutoCastStart {
                            skill_id: None,
                            caster_instance_id,
                            target_instance_id: None,
                        },
                    )
                });

                self.event_queue.push(BattleEvent::AutoCastEnd {
                    time_ms: cast_end_ms,
                    caster_instance_id,
                    cause: TimelineCause::Parent { seq: start_seq },
                });

                Ok(())
            }
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                self.with_recording_context(cause, |core| {
                    core.record_timeline(time_ms, TimelineEvent::AutoCastEnd { caster_instance_id })
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };
                caster.resonance_current = 0;
                caster.resonance_gain_locked_until_ms =
                    time_ms.saturating_add(caster.resonance_lock_ms);
                caster.next_action_time = 0;
                caster.pending_cast = false;

                Ok(())
            }
            BattleEvent::ApplyBuff {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
                cause,
            } => {
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
                    return Ok(());
                };

                if duration_ms == 0 {
                    return Ok(());
                }

                if !matches!(
                    self.units.get(&target_instance_id),
                    Some(unit) if unit.stats.current_health > 0
                ) {
                    return Ok(());
                }

                let applied_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffApplied {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            duration_ms,
                        },
                    )
                });

                let key = super::BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let expires_at_ms = time_ms.saturating_add(duration_ms);
                let max_stacks = def.max_stacks.max(1);
                let is_hard_cc = matches!(
                    def.kind,
                    crate::game::battle::buffs::BuffKind::Stun
                        | crate::game::battle::buffs::BuffKind::Freeze
                );

                // Hard CC is exclusive per target: any existing hard CC on this target is replaced.
                if is_hard_cc {
                    self.buffs.retain(|k, _| {
                        if k.target_instance_id != target_instance_id {
                            return true;
                        }
                        let Some(kdef) = crate::game::battle::buffs::get(k.buff_id) else {
                            return true;
                        };
                        !matches!(
                            kdef.kind,
                            crate::game::battle::buffs::BuffKind::Stun
                                | crate::game::battle::buffs::BuffKind::Freeze
                        )
                    });
                }

                let entry = self.buffs.entry(key).or_insert(super::ActiveBuff {
                    stacks: 0,
                    expires_at_ms,
                    next_tick_ms: None,
                });

                entry.expires_at_ms = entry.expires_at_ms.max(expires_at_ms);
                entry.stacks = entry.stacks.saturating_add(1).min(max_stacks);

                let effective_expires_at_ms = entry.expires_at_ms;

                if def.tick_interval_ms > 0 && entry.next_tick_ms.is_none() {
                    let tick_time_ms = time_ms.saturating_add(def.tick_interval_ms);
                    if tick_time_ms < entry.expires_at_ms {
                        entry.next_tick_ms = Some(tick_time_ms);
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: tick_time_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause: TimelineCause::Parent { seq: applied_seq },
                        });
                    }
                }

                self.event_queue.push(BattleEvent::BuffExpire {
                    time_ms: effective_expires_at_ms,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    cause: TimelineCause::Parent { seq: applied_seq },
                });

                // Hard CC: lock movement/actions until buff expiration.
                if is_hard_cc {
                    self.apply_hard_cc(target_instance_id, effective_expires_at_ms, time_ms);
                }

                Ok(())
            }
            BattleEvent::BuffTick {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                cause,
            } => {
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
                    return Ok(());
                };

                let key = super::BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = self.buffs.get(&key) else {
                    return Ok(());
                };

                if Some(time_ms) != active.next_tick_ms {
                    return Ok(());
                }

                let (stacks, expires_at_ms) = (active.stacks, active.expires_at_ms);

                let tick_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffTick {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                        },
                    )
                });

                match def.kind {
                    crate::game::battle::buffs::BuffKind::PeriodicDamage { damage_per_tick } => {
                        let stacks = stacks.max(1) as i32;
                        let dmg = (damage_per_tick as i32).saturating_mul(stacks);
                        if dmg > 0 {
                            self.with_recording_cause(tick_seq, |core| {
                                core.process_commands(
                                    vec![BattleCommand::ApplyHeal {
                                        target_id: target_instance_id,
                                        flat: -dmg,
                                        percent: 0,
                                        source_id: Some(caster_instance_id),
                                    }],
                                    time_ms,
                                );
                                core.schedule_pending_autocasts(time_ms);
                            });
                        }
                    }
                    _ => {}
                }

                let next_tick_ms = time_ms.saturating_add(def.tick_interval_ms);
                if let Some(active) = self.buffs.get_mut(&key) {
                    if next_tick_ms < expires_at_ms {
                        active.next_tick_ms = Some(next_tick_ms);
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: next_tick_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause: TimelineCause::Parent { seq: tick_seq },
                        });
                    } else {
                        active.next_tick_ms = None;
                    }
                }

                Ok(())
            }
            BattleEvent::BuffExpire {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                cause,
            } => {
                let Some(def) = crate::game::battle::buffs::get(buff_id) else {
                    return Ok(());
                };

                let key = super::BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = self.buffs.get(&key) else {
                    return Ok(());
                };

                if time_ms < active.expires_at_ms {
                    return Ok(());
                }

                self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::BuffExpired {
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                        },
                    )
                });

                self.buffs.remove(&key);

                // Hard CC ends with buff expiration.
                if matches!(
                    def.kind,
                    crate::game::battle::buffs::BuffKind::Stun
                        | crate::game::battle::buffs::BuffKind::Freeze
                ) {
                    self.release_hard_cc(target_instance_id, time_ms);
                }
                Ok(())
            }
            BattleEvent::MovementIntent { .. } | BattleEvent::MoveStep { .. } => Ok(()),
        }
    }
}
