use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::{DeliveryDef, SkillArea, SkillDef, SkillEffectDef, SkillTarget},
        battle::{
            damage::BattleCommand,
            enums::{BattleEvent, ProjectilePayload},
            ids::UnitInstanceId,
            timeline::{
                AttackKind, SkillCastTarget, Timeline, TimelineCause, TimelineEvent,
                TimelineRootCause,
            },
            types::{BattleResult, BattleWinner},
        },
        behavior::GameError,
        enums::Side,
        stats::UnitStats,
    },
};

use super::{
    movement::ActionState, types::PendingAutoCast, ActiveBuff, BattleCore, BuffInstanceKey,
};

impl BattleCore {
    fn resolve_skill_targets_at_start(
        &self,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        caster_pos: Position,
    ) -> Option<SkillCastTarget> {
        match &skill.target {
            SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                unit_instance_id: caster_instance_id,
            }),
            SkillTarget::EnemySingle { .. } => {
                // note: UnitTargetRule 에 따라 처리되어야함. 지금은 가장 가까운 적 하나.
                let mut best: Option<(i32, UnitInstanceId)> = None;
                for unit in self.units.values() {
                    if unit.is_dead() || unit.owner == caster_owner {
                        continue;
                    }
                    let Some(unit_pos) = self.battlefield.position_of(unit.instance_id) else {
                        continue;
                    };
                    let d = caster_pos.chebyshev(&unit_pos);
                    if d > skill.range_tiles as i32 {
                        continue;
                    }
                    match best {
                        None => best = Some((d, unit.instance_id)),
                        Some((best_d, _)) if d < best_d => best = Some((d, unit.instance_id)),
                        Some((best_d, best_id))
                            if d == best_d && unit.instance_id.as_bytes() < best_id.as_bytes() =>
                        {
                            best = Some((d, unit.instance_id))
                        }
                        _ => {}
                    }
                }
                best.map(|(_, id)| SkillCastTarget::Unit {
                    unit_instance_id: id,
                })
            }
            SkillTarget::Allies { area } | SkillTarget::Enemies { area } => match area {
                SkillArea::All | SkillArea::RadiusChebyshev { .. } => Some(SkillCastTarget::Tile {
                    position: caster_pos,
                }),
            },
        }
    }
    fn compute_winner(&self) -> Option<BattleWinner> {
        let mut player_alive = false;
        let mut opponent_alive = false;

        for unit in self.units.values() {
            if unit.is_dead() {
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
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for unit_id in unit_ids {
            let Some(unit) = self.units.get_mut(&unit_id) else {
                continue;
            };
            let interval_ms = unit.stats.attack_interval_ms.max(1);
            unit.next_basic_attack_ms = interval_ms;
            unit.pending_basic_attack = false;
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: interval_ms,
                attacker_instance_id: unit_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::Period,
                },
            });
        }
    }

    pub(super) fn schedule_movement_intent(&mut self, time_ms: u64) {
        self.event_queue
            .push(BattleEvent::MovementIntent { time_ms });
    }

    fn split_bucket(
        &self,
        input: Vec<BattleEvent>,
        movement_intent: &mut bool,
        move_steps: &mut Vec<(UnitInstanceId, u32)>,
        combat: &mut Vec<BattleEvent>,
    ) {
        for event in input {
            match event {
                BattleEvent::MovementIntent { .. } => *movement_intent = true,
                BattleEvent::MoveStep {
                    unit_instance_id,
                    expected_move_epoch,
                    ..
                } => {
                    move_steps.push((unit_instance_id, expected_move_epoch));
                }
                other => combat.push(other),
            }
        }
    }

    pub fn run_battle(&mut self, _world: &mut World) -> Result<BattleResult, GameError> {
        self.units.clear();
        self.artifacts.clear();
        self.items.clear();
        self.graveyard.clear();
        self.buffs.clear();
        self.projectiles.clear();
        self.battlefield.clear();
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
                    width: core.battlefield.width(),
                    height: core.battlefield.height(),
                },
            );

            let mut unit_records: Vec<(UnitInstanceId, Side, Uuid, Position, UnitStats)> = core
                .units
                .values()
                .map(|u| {
                    let position = core
                        .battlefield
                        .position_of(u.instance_id)
                        .unwrap_or(Position::new(0, 0));
                    (u.instance_id, u.owner, u.base_uuid, position, u.stats)
                })
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

            let mut item_records: Vec<(Uuid, Side, UnitInstanceId, Uuid)> = core
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
        self.schedule_movement_intent(0);

        const MAX_BATTLE_TIME_MS: u64 = 60_000;

        let mut last_event_time_ms = 0;

        while let Some(next_time_ms) = self.event_queue.peek().map(|e| e.time_ms()) {
            let current_time_ms = next_time_ms;
            last_event_time_ms = current_time_ms;

            if current_time_ms > MAX_BATTLE_TIME_MS {
                return Ok(self.finish_battle(MAX_BATTLE_TIME_MS, BattleWinner::Draw));
            }

            let mut bucket: Vec<BattleEvent> = Vec::new();
            while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                bucket.push(self.event_queue.pop().unwrap());
            }

            let mut movement_intent = false;
            let mut move_steps: Vec<(UnitInstanceId, u32)> = Vec::new();
            let mut combat: Vec<BattleEvent> = Vec::new();

            self.split_bucket(bucket, &mut movement_intent, &mut move_steps, &mut combat);

            loop {
                combat.sort_by(|a, b| b.cmp(a));
                for event in combat.drain(..) {
                    self.process_event(event, current_time_ms)?;
                }

                let mut new_bucket: Vec<BattleEvent> = Vec::new();
                while matches!(self.event_queue.peek(), Some(e) if e.time_ms() == current_time_ms) {
                    new_bucket.push(self.event_queue.pop().unwrap());
                }

                if new_bucket.is_empty() {
                    break;
                }

                self.split_bucket(
                    new_bucket,
                    &mut movement_intent,
                    &mut move_steps,
                    &mut combat,
                );
            }

            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }

            if movement_intent {
                self.compute_movement_intents(current_time_ms);
                self.try_start_pending_basic_attacks(current_time_ms);
            }

            if !move_steps.is_empty() {
                self.handle_move_steps_at(current_time_ms, move_steps);
                self.post_move_retarget_at(current_time_ms);
                self.try_start_pending_basic_attacks(current_time_ms);
            }

            if let Some(winner) = self.compute_winner() {
                return Ok(self.finish_battle(current_time_ms, winner));
            }
        }

        let end_time_ms = last_event_time_ms.min(MAX_BATTLE_TIME_MS);
        let winner = self.compute_winner().unwrap_or(BattleWinner::Draw);
        Ok(self.finish_battle(end_time_ms, winner))
    }

    fn is_alive_enemy(&self, unit_id: UnitInstanceId, owner: Side) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && !unit.is_dead(),
            None => false,
        }
    }

    fn find_nearest_alive_enemy(
        &self,
        from_uuid: UnitInstanceId,
        from_side: Side,
    ) -> Option<UnitInstanceId> {
        let from_pos = self.battlefield.position_of(from_uuid)?;
        let mut nearest: Option<(UnitInstanceId, i32)> = None;

        for unit in self.units.values() {
            if unit.is_dead() {
                continue;
            }
            if unit.owner == from_side {
                continue;
            }

            let Some(unit_pos) = self.battlefield.position_of(unit.instance_id) else {
                continue;
            };
            let distance = from_pos.chebyshev(&unit_pos);
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
            let owner = unit.owner;
            let base_uuid = unit.base_uuid;
            if unit.is_dead() || !pending || now_ms < next_ready_ms {
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
                    cause: TimelineCause::Root {
                        kind: TimelineRootCause::Period,
                    },
                });
                continue;
            }

            let Some(attacker_pos) = self.battlefield.position_of(unit_id) else {
                continue;
            };
            let range_tiles = self.basic_attack_range_tiles(base_uuid);
            let target = self.choose_attack_target_in_range(owner, attacker_pos, range_tiles);
            let Some(target_id) = target else {
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
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::Period,
                },
            });
        }
    }

    pub(super) fn resolve_skill_targets_at_execute(
        &self,
        caster_instance_id: UnitInstanceId,
        skill: &SkillDef,
        start_target: Option<SkillCastTarget>,
    ) -> Vec<UnitInstanceId> {
        let Some(caster) = self.units.get(&caster_instance_id) else {
            return Vec::new();
        };
        if caster.is_dead() {
            return Vec::new();
        }

        let caster_owner = caster.owner;
        let caster_pos = match self.battlefield.position_of(caster_instance_id) {
            Some(pos) => pos,
            None => return Vec::new(),
        };

        let mut targets: Vec<UnitInstanceId> = Vec::new();

        match &skill.target {
            SkillTarget::SelfUnit => targets.push(caster_instance_id),
            SkillTarget::EnemySingle { .. } => {
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = start_target {
                    if self.is_alive_enemy(unit_instance_id, caster_owner)
                        && self
                            .battlefield
                            .position_of(unit_instance_id)
                            .is_some_and(|p| caster_pos.chebyshev(&p) <= skill.range_tiles as i32)
                    {
                        targets.push(unit_instance_id);
                    }
                }

                if targets.is_empty() {
                    if let Some(SkillCastTarget::Unit { unit_instance_id }) = self
                        .resolve_skill_targets_at_start(
                            skill,
                            caster_instance_id,
                            caster_owner,
                            caster_pos,
                        )
                    {
                        targets.push(unit_instance_id);
                    }
                }
            }
            SkillTarget::Allies { area } | SkillTarget::Enemies { area } => {
                let wants_allies = matches!(skill.target, SkillTarget::Allies { .. });
                let anchor = match start_target {
                    Some(SkillCastTarget::Tile { position }) => position,
                    _ => caster_pos,
                };
                let radius_tiles = match area {
                    SkillArea::All => None,
                    SkillArea::RadiusChebyshev { radius_tiles } => Some(*radius_tiles as i32),
                };

                for unit in self.units.values() {
                    if unit.is_dead() {
                        continue;
                    }

                    let is_ally = unit.owner == caster_owner;
                    if wants_allies != is_ally {
                        continue;
                    }

                    let Some(pos) = self.battlefield.position_of(unit.instance_id) else {
                        continue;
                    };

                    if let Some(r) = radius_tiles {
                        if anchor.chebyshev(&pos) > r {
                            continue;
                        }
                    }

                    targets.push(unit.instance_id);
                }
            }
        }

        targets.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        targets
    }

    pub(super) fn build_skill_effect_commands(
        caster_instance_id: UnitInstanceId,
        skill: &SkillDef,
        targets: &[UnitInstanceId],
    ) -> Vec<BattleCommand> {
        let mut commands: Vec<BattleCommand> = Vec::new();

        let hinted_target_id = if targets.len() == 1 {
            Some(targets[0])
        } else {
            None
        };

        for effect in &skill.effects {
            match effect {
                SkillEffectDef::Damage { amount } => {
                    let flat = amount.saturating_mul(-1);
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyHeal {
                            target_id: *target_id,
                            flat,
                            percent: 0,
                            source_id: Some(caster_instance_id),
                        });
                    }
                }
                SkillEffectDef::Heal { amount } => {
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyHeal {
                            target_id: *target_id,
                            flat: *amount,
                            percent: 0,
                            source_id: Some(caster_instance_id),
                        });
                    }
                }
                SkillEffectDef::ApplyBuff {
                    buff_id,
                    duration_ms,
                } => {
                    let buff_id = crate::game::battle::buffs::BuffId::from_name(buff_id);
                    for target_id in targets {
                        commands.push(BattleCommand::ApplyBuff {
                            caster_id: caster_instance_id,
                            target_id: *target_id,
                            buff_id,
                            duration_ms: u64::from(*duration_ms),
                        });
                    }
                }
                SkillEffectDef::ExtraAttack { count } => {
                    for _ in 0..(*count as usize) {
                        commands.push(BattleCommand::ScheduleAttack {
                            attacker_id: caster_instance_id,
                            target_id: hinted_target_id,
                            time_ms: 0,
                        });
                    }
                }
            }
        }

        commands
    }

    pub(super) fn process_event(
        &mut self,
        event: BattleEvent,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => {
                let (
                    is_dead,
                    can_attack,
                    lock_until,
                    owner,
                    base_uuid,
                    current_target,
                    interval_ms,
                ) = {
                    let Some(attacker) = self.units.get(&attacker_instance_id) else {
                        return Ok(());
                    };
                    (
                        attacker.is_dead(),
                        attacker.action_locks.can_basic_attack(current_time_ms),
                        attacker.action_locks.basic_attack_until_ms,
                        attacker.owner,
                        attacker.base_uuid,
                        attacker.current_target,
                        attacker.stats.attack_interval_ms.max(1),
                    )
                };
                if is_dead {
                    return Ok(());
                }

                // 행동 락(하드 CC/집중 등)으로 공격이 지연됐을 때 재스케줄링
                if !can_attack {
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: lock_until,
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause,
                    });
                    return Ok(());
                }

                let Some(attacker_pos) = self.battlefield.position_of(attacker_instance_id) else {
                    return Ok(());
                };
                let range_tiles = self.basic_attack_range_tiles(base_uuid);
                let in_range = |id: UnitInstanceId| {
                    self.is_alive_enemy(id, owner)
                        && self
                            .battlefield
                            .position_of(id)
                            .is_some_and(|pos| attacker_pos.chebyshev(&pos) <= range_tiles as i32)
                };

                let hinted_target = target_instance_id.filter(|id| in_range(*id));
                let persisted_target = current_target.filter(|id| in_range(*id));
                let target = hinted_target.or(persisted_target).or_else(|| {
                    self.choose_attack_target_in_range(owner, attacker_pos, range_tiles)
                });

                if target.is_none() {
                    if schedule_next {
                        if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                            attacker.pending_basic_attack = true;
                            attacker.next_basic_attack_ms = time_ms;
                        }
                    }
                    self.event_queue
                        .push(BattleEvent::MovementIntent { time_ms });
                    return Ok(());
                }

                let target_id = target.unwrap();
                let mut stopped_movement = false;
                if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                    if matches!(attacker.action_state, ActionState::Moving(_))
                        || matches!(attacker.action_state, ActionState::WaitRepath { .. })
                    {
                        attacker.move_epoch = attacker.move_epoch.wrapping_add(1);
                        attacker.action_state = ActionState::Idle;
                        stopped_movement = true;
                    }

                    attacker.current_target = Some(target_id);
                    attacker.pending_basic_attack = false;
                }

                if stopped_movement {
                    self.battlefield.cancel_reservation(attacker_instance_id);
                    self.record_movement_stopped(
                        time_ms,
                        attacker_instance_id,
                        crate::game::battle::timeline::MovementStopReason::TargetAcquired,
                        None,
                    );
                    self.event_queue
                        .push(BattleEvent::MovementIntent { time_ms });
                }

                let windup_ms = self
                    .game_data
                    .abnormality_data
                    .get_by_uuid(&base_uuid)
                    .map(|m| m.basic_attack.windup_ms as u64)
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

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AttackStart {
                            attacker_instance_id,
                            target_instance_id: target_id,
                            kind: Some(attack_kind),
                        },
                    )
                });

                self.event_queue.push(BattleEvent::AttackResolve {
                    time_ms: resolve_time,
                    attacker_instance_id,
                    target_instance_id: target_id,
                    kind: attack_kind,
                    cause: TimelineCause::Parent { seq: start_seq },
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
                        cause: TimelineCause::Root {
                            kind: TimelineRootCause::Period,
                        },
                    });
                }

                Ok(())
            }
            BattleEvent::AttackResolve {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                kind,
                cause,
            } => {
                let resolve_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AttackResolve {
                            attacker_instance_id,
                            target_instance_id,
                            kind: Some(kind),
                        },
                    )
                });

                let hit = self.with_recording_cause(resolve_seq, |core| {
                    core.resolve_basic_attack(attacker_instance_id, target_instance_id, time_ms)
                });

                if !hit {
                    self.with_recording_cause(resolve_seq, |core| {
                        core.record_timeline(
                            time_ms,
                            TimelineEvent::AttackMiss {
                                attacker_instance_id,
                                target_instance_id,
                                kind: Some(kind),
                            },
                        )
                    });
                }

                Ok(())
            }
            BattleEvent::ProjectileHit {
                time_ms,
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                payload,
                cause,
            } => {
                self.with_recording_context(cause, |core| {
                    core.apply_projectile_hit(
                        time_ms,
                        projectile_id,
                        attacker_instance_id,
                        target_instance_id,
                        payload,
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
                if caster.is_dead() {
                    return Ok(());
                }
                let blocked_until = caster.next_action_time;

                // CC 등 행동이 막힌 상태라면 이벤트 연기
                if time_ms < blocked_until {
                    if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                        unit.pending_cast = true;
                        unit.pending_cast_cause.get_or_insert(cause);
                    }
                    self.event_queue.push(BattleEvent::AutoCastStart {
                        time_ms: blocked_until,
                        caster_instance_id,
                        cause,
                    });
                    return Ok(());
                }

                let (caster_owner, caster_base_uuid) = {
                    let Some(caster) = self.units.get(&caster_instance_id) else {
                        return Ok(());
                    };
                    (caster.owner, caster.base_uuid)
                };

                let skill_id = self
                    .game_data
                    .abnormality_data
                    .get_by_uuid(&caster_base_uuid)
                    .and_then(|m| m.skill_id.as_deref())
                    .filter(|id| self.game_data.skill_data.get_by_id(id).is_some())
                    .map(str::to_string);

                if skill_id.is_none() || skill_id.as_deref().unwrap().len() == 0 {
                    // TODO: 기록
                    return Ok(());
                }

                let caster_pos = match self.battlefield.position_of(caster_instance_id) {
                    Some(pos) => pos,
                    None => return Ok(()),
                };

                let skill = match skill_id
                    .as_deref()
                    .and_then(|id| self.game_data.skill_data.get_by_id(id))
                {
                    Some(skill) => skill.clone(),
                    None => return Ok(()),
                };

                // "캐스트/집중"이 진행되는 동안 새로운 AutoCastStart를 막기 위한 캐스트 락.
                // (공격/이동은 ActionLocks로 개별 게이트)
                let cast_end_ms = if skill.focus_time_ms == 0 {
                    time_ms.saturating_add(1)
                } else {
                    time_ms.saturating_add(skill.focus_time_ms as u64)
                };
                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.next_action_time = cast_end_ms;

                    // 집중/시전 중 공명 획득은 항상 금지
                    caster.action_locks.lock_resonance_gain_until(cast_end_ms);

                    if !skill.focus_permissions.allows_basic_attack {
                        caster.action_locks.lock_basic_attack_until(cast_end_ms);
                    }

                    if !skill.focus_permissions.allows_move {
                        caster.action_locks.lock_movement_until(cast_end_ms);
                    }
                }

                // 이동 금지 스킬이면, 스케줄된 MoveStep을 epoch로 무효화하고 즉시 정지.
                if !skill.focus_permissions.allows_move {
                    let was_moving = self
                        .units
                        .get(&caster_instance_id)
                        .is_some_and(|u| matches!(u.action_state, ActionState::Moving(_)));
                    if was_moving {
                        self.update_move_position_to(caster_instance_id, time_ms);
                        self.battlefield.cancel_reservation(caster_instance_id);
                        if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                            caster.move_epoch = caster.move_epoch.wrapping_add(1);
                            caster.action_state = ActionState::Idle;
                        }
                        self.schedule_movement_intent(cast_end_ms);
                    }
                }

                let target = self.resolve_skill_targets_at_start(
                    &skill,
                    caster_instance_id,
                    caster_owner,
                    caster_pos,
                );

                let start_seq = self.with_recording_context(cause, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AutoCastStart {
                            skill_id,
                            caster_instance_id,
                            target,
                        },
                    )
                });

                // note: 즉발 스킬에도 cast_delay_ms 적용 하려면 아래 코드 블럭을 AutoCastEnd 로 옮겨야함.
                // 즉발인 경우 즉시 처리.
                if skill.focus_time_ms == 0 {
                    let target_instance_id = match target {
                        Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
                        _ => None,
                    };

                    let ability_seq = self.with_recording_cause(start_seq, |core| {
                        core.record_timeline(
                            time_ms,
                            TimelineEvent::AbilityCast {
                                skill_id: skill.id.clone(),
                                caster_instance_id,
                                target_instance_id,
                            },
                        )
                    });

                    self.with_recording_cause(ability_seq, |core| {
                        match &skill.delivery {
                            DeliveryDef::Instant => {
                                let targets = core.resolve_skill_targets_at_execute(
                                    caster_instance_id,
                                    &skill,
                                    target,
                                );
                                let commands = Self::build_skill_effect_commands(
                                    caster_instance_id,
                                    &skill,
                                    &targets,
                                );
                                if !commands.is_empty() {
                                    core.process_commands(commands, time_ms);
                                }
                            }
                            DeliveryDef::Projectile { speed_units_per_ms } => {
                                let (event_target_id, target_pos) = match target {
                                    Some(SkillCastTarget::Unit { unit_instance_id }) => (
                                        unit_instance_id,
                                        core.battlefield.position_of(unit_instance_id),
                                    ),
                                    Some(SkillCastTarget::Tile { position }) => {
                                        (caster_instance_id, Some(position))
                                    }
                                    None => (caster_instance_id, None),
                                };
                                let Some(target_pos) = target_pos else {
                                    return;
                                };
                                core.schedule_projectile_hit_event(
                                    time_ms,
                                    caster_instance_id,
                                    event_target_id,
                                    caster_pos,
                                    target_pos,
                                    *speed_units_per_ms,
                                    ProjectilePayload::Skill {
                                        skill_id: skill.id.clone(),
                                        caster_owner,
                                        cast_target: target,
                                    },
                                );
                            }
                        }
                        core.schedule_pending_autocasts(time_ms);
                    });
                } else {
                    if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                        caster.pending_autocast = Some(PendingAutoCast {
                            skill_id: skill.id.clone(),
                            start_target: target,
                        });
                    }
                }

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
                let pending = self
                    .units
                    .get_mut(&caster_instance_id)
                    .and_then(|unit| unit.pending_autocast.take());
                self.with_recording_context(cause, |core| {
                    if let Some(pending) = pending {
                        let Some(skill) = core.game_data.skill_data.get_by_id(&pending.skill_id)
                        else {
                            core.record_timeline(
                                time_ms,
                                TimelineEvent::AutoCastEnd { caster_instance_id },
                            );
                            return;
                        };
                        let skill = skill.clone();

                        let target_instance_id = match pending.start_target {
                            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                                Some(unit_instance_id)
                            }
                            _ => None,
                        };

                        let ability_seq = core.record_timeline(
                            time_ms,
                            TimelineEvent::AbilityCast {
                                skill_id: skill.id.clone(),
                                caster_instance_id,
                                target_instance_id,
                            },
                        );

                        core.with_recording_cause(ability_seq, |core| {
                            match &skill.delivery {
                                DeliveryDef::Instant => {
                                    let targets = core.resolve_skill_targets_at_execute(
                                        caster_instance_id,
                                        &skill,
                                        pending.start_target,
                                    );
                                    let commands = Self::build_skill_effect_commands(
                                        caster_instance_id,
                                        &skill,
                                        &targets,
                                    );
                                    if !commands.is_empty() {
                                        core.process_commands(commands, time_ms);
                                    }
                                }
                                DeliveryDef::Projectile { speed_units_per_ms } => {
                                    let caster_pos =
                                        match core.battlefield.position_of(caster_instance_id) {
                                            Some(pos) => pos,
                                            None => return,
                                        };
                                    let (event_target_id, target_pos) = match pending.start_target {
                                        Some(SkillCastTarget::Unit { unit_instance_id }) => (
                                            unit_instance_id,
                                            core.battlefield.position_of(unit_instance_id),
                                        ),
                                        Some(SkillCastTarget::Tile { position }) => {
                                            (caster_instance_id, Some(position))
                                        }
                                        None => (caster_instance_id, None),
                                    };
                                    let Some(target_pos) = target_pos else {
                                        return;
                                    };
                                    core.schedule_projectile_hit_event(
                                        time_ms,
                                        caster_instance_id,
                                        event_target_id,
                                        caster_pos,
                                        target_pos,
                                        *speed_units_per_ms,
                                        ProjectilePayload::Skill {
                                            skill_id: skill.id.clone(),
                                            caster_owner: core
                                                .units
                                                .get(&caster_instance_id)
                                                .map(|u| u.owner)
                                                .unwrap_or(Side::Player),
                                            cast_target: pending.start_target,
                                        },
                                    );
                                }
                            }
                            core.schedule_pending_autocasts(time_ms);
                        });
                    }

                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AutoCastEnd { caster_instance_id },
                    );
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };
                caster.resonance_current = 0;
                caster
                    .action_locks
                    .lock_resonance_gain_until(time_ms.saturating_add(caster.resonance_lock_ms));
                if caster.next_action_time <= time_ms {
                    caster.next_action_time = 0;
                }
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
                    Some(unit) if !unit.is_dead()
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

                let key = BuffInstanceKey {
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

                // TODO: Define re-apply semantics for same (caster,target,buff_id).
                // Current: stacks++ (clamped), expires_at=max, tick schedule is NOT reset once next_tick_ms is set.
                // If policy changes (e.g., refresh duration/tick, replace stacks), adjust here and update tests/timeline.
                let entry = self.buffs.entry(key).or_insert(ActiveBuff {
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

                if is_hard_cc {
                    let lock_until = effective_expires_at_ms.saturating_add(1);
                    if let Some(unit) = self.units.get_mut(&target_instance_id) {
                        unit.next_action_time = unit.next_action_time.max(lock_until);
                        unit.action_locks.lock_movement_until(lock_until);
                        unit.action_locks.lock_basic_attack_until(lock_until);
                        unit.action_locks.lock_resonance_gain_until(lock_until);
                    }

                    let was_moving = self
                        .units
                        .get(&target_instance_id)
                        .is_some_and(|u| matches!(u.action_state, ActionState::Moving(_)));
                    if was_moving {
                        self.update_move_position_to(target_instance_id, time_ms);
                        self.record_movement_stopped(
                            time_ms,
                            target_instance_id,
                            crate::game::battle::timeline::MovementStopReason::HardCC,
                            Some(lock_until),
                        );
                        self.battlefield.cancel_reservation(target_instance_id);
                        if let Some(unit) = self.units.get_mut(&target_instance_id) {
                            unit.move_epoch = unit.move_epoch.wrapping_add(1);
                            unit.action_state = ActionState::Idle;
                        }
                        self.schedule_movement_intent(lock_until);
                    }
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

                let key = BuffInstanceKey {
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

                let key = BuffInstanceKey {
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

                if matches!(
                    def.kind,
                    crate::game::battle::buffs::BuffKind::Stun
                        | crate::game::battle::buffs::BuffKind::Freeze
                ) {
                    self.schedule_pending_autocasts(time_ms.saturating_add(1));
                }
                Ok(())
            }
            BattleEvent::MoveStep { .. } | BattleEvent::MovementIntent { .. } => Ok(()),
        }
    }
}
