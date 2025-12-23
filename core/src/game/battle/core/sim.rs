use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{battle::timeline::TimelineEvent, behavior::GameError, enums::Side, stats::UnitStats},
};

use super::super::{
    buffs,
    enums::BattleEvent,
    types::{BattleResult, BattleWinner},
};
use super::BattleCore;

impl BattleCore {
    pub fn run_battle(&mut self, _world: &mut World) -> Result<BattleResult, GameError> {
        self.units.clear();
        self.artifacts.clear();
        self.items.clear();
        self.graveyard.clear();
        self.buffs.clear();
        self.death_handler.reset();
        self.ability_executor.reset_cooldowns();
        self.timeline = crate::game::battle::timeline::Timeline::new();
        self.timeline_seq = 0;
        self.recording_cause_stack.clear();

        self.build_runtime_units_from_decks(Side::Player)?;
        self.build_runtime_units_from_decks(Side::Opponent)?;
        self.build_runtime_field()?;

        self.record_timeline(
            0,
            TimelineEvent::BattleStart {
                width: self.runtime_field.width,
                height: self.runtime_field.height,
            },
        );

        let mut unit_records: Vec<(Uuid, Side, Uuid, Position, UnitStats)> = self
            .units
            .values()
            .map(|u| (u.instance_id, u.owner, u.base_uuid, u.position, u.stats))
            .collect();
        unit_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (unit_instance_id, owner, base_uuid, position, stats) in unit_records {
            self.record_timeline(
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

        let mut artifact_records: Vec<(Uuid, Side, Uuid)> = self
            .artifacts
            .values()
            .map(|a| (a.instance_id, a.owner, a.base_uuid))
            .collect();
        artifact_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (artifact_instance_id, owner, base_uuid) in artifact_records {
            self.record_timeline(
                0,
                TimelineEvent::ArtifactSpawned {
                    artifact_instance_id,
                    owner,
                    base_uuid,
                },
            );
        }

        let mut item_records: Vec<(Uuid, Side, Uuid, Uuid)> = self
            .items
            .values()
            .map(|i| (i.instance_id, i.owner, i.owner_unit_instance, i.base_uuid))
            .collect();
        item_records.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (item_instance_id, owner, owner_unit_instance_id, base_uuid) in item_records {
            self.record_timeline(
                0,
                TimelineEvent::ItemSpawned {
                    item_instance_id,
                    owner,
                    owner_unit_instance_id,
                    base_uuid,
                },
            );
        }

        self.event_queue.clear();
        self.init_initial_events();

        const MAX_BATTLE_TIME_MS: u64 = 60_000;

        while let Some(event) = self.event_queue.pop() {
            let current_time_ms = event.time_ms();

            if current_time_ms > MAX_BATTLE_TIME_MS {
                self.record_timeline(
                    MAX_BATTLE_TIME_MS,
                    TimelineEvent::BattleEnd {
                        winner: BattleWinner::Draw,
                    },
                );
                return Ok(BattleResult {
                    winner: BattleWinner::Draw,
                    timeline: self.timeline.clone(),
                });
            }

            self.process_event(event, current_time_ms)?;

            let player_alive = self.units.values().any(|u| u.owner == Side::Player);
            let opponent_alive = self.units.values().any(|u| u.owner == Side::Opponent);

            let winner = match (player_alive, opponent_alive) {
                (true, true) => None,
                (true, false) => Some(BattleWinner::Player),
                (false, true) => Some(BattleWinner::Opponent),
                (false, false) => Some(BattleWinner::Draw),
            };

            if let Some(winner) = winner {
                self.record_timeline(current_time_ms, TimelineEvent::BattleEnd { winner });
                return Ok(BattleResult {
                    winner,
                    timeline: self.timeline.clone(),
                });
            }
        }

        self.record_timeline(
            MAX_BATTLE_TIME_MS,
            TimelineEvent::BattleEnd {
                winner: BattleWinner::Draw,
            },
        );
        Ok(BattleResult {
            winner: BattleWinner::Draw,
            timeline: self.timeline.clone(),
        })
    }

    fn init_initial_events(&mut self) {
        for unit in self.units.values() {
            self.event_queue.push(BattleEvent::Attack {
                time_ms: unit.stats.attack_interval_ms,
                attacker_instance_id: unit.instance_id,
                target_instance_id: None,
                schedule_next: true,
                cause_seq: None,
            });
        }
    }

    pub(super) fn process_event(
        &mut self,
        event: BattleEvent,
        current_time_ms: u64,
    ) -> Result<(), GameError> {
        match event {
            BattleEvent::Attack {
                attacker_instance_id,
                time_ms,
                target_instance_id,
                schedule_next,
                cause_seq,
            } => {
                let Some(attacker) = self.units.get(&attacker_instance_id) else {
                    return Ok(());
                };

                if attacker.stats.current_health == 0 {
                    return Ok(());
                }

                if current_time_ms < attacker.casting_until_ms {
                    self.event_queue.push(BattleEvent::Attack {
                        time_ms: attacker.casting_until_ms,
                        attacker_instance_id,
                        target_instance_id,
                        schedule_next,
                        cause_seq,
                    });
                    return Ok(());
                }

                let owner = attacker.owner;
                let current_target = attacker.current_target;

                let hinted_target = target_instance_id.filter(|id| self.is_alive_enemy(*id, owner));
                let persisted_target = current_target.filter(|id| self.is_alive_enemy(*id, owner));

                let target = hinted_target
                    .or(persisted_target)
                    .or_else(|| self.find_nearest_alive_enemy(attacker_instance_id, owner));

                let attack_seq = if let Some(target_id) = target {
                    if let Some(attacker) = self.units.get_mut(&attacker_instance_id) {
                        attacker.current_target = Some(target_id);
                    }

                    Some(self.with_recording_parent(cause_seq, |core| {
                        core.record_timeline(
                            time_ms,
                            TimelineEvent::Attack {
                                attacker_instance_id,
                                target_instance_id: target_id,
                                kind: Some(if schedule_next {
                                    crate::game::battle::timeline::AttackKind::Auto
                                } else {
                                    crate::game::battle::timeline::AttackKind::Triggered
                                }),
                            },
                        )
                    }))
                } else {
                    None
                };

                if let Some(attack_seq) = attack_seq {
                    self.with_recording_cause(attack_seq, |core| {
                        core.apply_attack(attacker_instance_id, time_ms);
                    });
                } else {
                    self.apply_attack(attacker_instance_id, time_ms);
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
                        cause_seq: None,
                    });
                }

                Ok(())
            }
            BattleEvent::AutoCastStart {
                time_ms,
                caster_instance_id,
                cause_seq,
            } => {
                let (caster_owner, caster_base_uuid, caster_current_target) = {
                    let Some(caster) = self.units.get(&caster_instance_id) else {
                        return Ok(());
                    };
                    if caster.stats.current_health == 0 {
                        return Ok(());
                    }
                    (caster.owner, caster.base_uuid, caster.current_target)
                };

                let ability_id = self
                    .game_data
                    .abnormality_data
                    .get_by_uuid(&caster_base_uuid)
                    .and_then(|meta| meta.abilities.first().copied());

                // Instant cast is modeled as [time_ms, time_ms + 1) to block resonance gain.
                let cast_end_ms = time_ms.saturating_add(1);
                if let Some(caster) = self.units.get_mut(&caster_instance_id) {
                    caster.casting_until_ms = cast_end_ms;
                }

                let target_hint =
                    caster_current_target.filter(|id| self.is_alive_enemy(*id, caster_owner));

                let start_seq = self.with_recording_parent(cause_seq, |core| {
                    core.record_timeline(
                        time_ms,
                        TimelineEvent::AutoCastStart {
                            caster_instance_id,
                            ability_id,
                            target_instance_id: target_hint,
                        },
                    )
                });

                self.with_recording_cause(start_seq, |core| {
                    if let Some(ability_id) = ability_id {
                        let result = core.execute_ability_via_executor(
                            ability_id,
                            caster_instance_id,
                            target_hint,
                            time_ms,
                        );

                        if result.executed {
                            let cast_seq = core.record_timeline(
                                time_ms,
                                TimelineEvent::AbilityCast {
                                    ability_id,
                                    caster_instance_id,
                                    target_instance_id: target_hint,
                                },
                            );

                            if !result.commands.is_empty() {
                                core.with_recording_cause(cast_seq, |core| {
                                    core.process_commands(result.commands, time_ms);
                                });
                            }
                        }

                        core.schedule_pending_autocasts(time_ms);
                    }

                    core.event_queue.push(BattleEvent::AutoCastEnd {
                        time_ms: cast_end_ms,
                        caster_instance_id,
                        cause_seq: Some(start_seq),
                    });
                });

                Ok(())
            }
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause_seq,
            } => {
                self.with_recording_parent(cause_seq, |core| {
                    core.record_timeline(time_ms, TimelineEvent::AutoCastEnd { caster_instance_id })
                });

                let Some(caster) = self.units.get_mut(&caster_instance_id) else {
                    return Ok(());
                };

                caster.resonance_current = 0;
                caster.resonance_gain_locked_until_ms =
                    time_ms.saturating_add(caster.resonance_lock_ms);
                caster.casting_until_ms = 0;
                caster.pending_cast = false;

                Ok(())
            }
            BattleEvent::ApplyBuff {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
                cause_seq,
            } => {
                let Some(def) = buffs::get(buff_id) else {
                    return Ok(());
                };

                if duration_ms == 0 {
                    return Ok(());
                }

                if !self.units.contains_key(&target_instance_id) {
                    return Ok(());
                }

                let applied_seq = self.with_recording_parent(cause_seq, |core| {
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

                let entry = self.buffs.entry(key).or_insert(super::ActiveBuff {
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
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: tick_time_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause_seq: Some(applied_seq),
                        });
                    }
                }

                if entry.expires_at_ms > current_time_ms {
                    self.event_queue.push(BattleEvent::BuffExpire {
                        time_ms: entry.expires_at_ms,
                        caster_instance_id,
                        target_instance_id,
                        buff_id,
                        cause_seq: Some(applied_seq),
                    });
                }

                Ok(())
            }
            BattleEvent::BuffTick {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                cause_seq,
            } => {
                let Some(def) = buffs::get(buff_id) else {
                    return Ok(());
                };

                if def.tick_interval_ms == 0 {
                    return Ok(());
                }

                let key = super::BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };
                let (stacks, _expires_at_ms) = {
                    let Some(active) = self.buffs.get(&key) else {
                        return Ok(());
                    };

                    if time_ms >= active.expires_at_ms {
                        return Ok(());
                    }

                    if active.next_tick_ms != Some(time_ms) {
                        return Ok(());
                    }

                    (active.stacks, active.expires_at_ms)
                };

                let tick_seq = self.with_recording_parent(cause_seq, |core| {
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
                    buffs::BuffKind::PeriodicDamage { damage_per_tick } => {
                        let stacks = stacks.max(1) as i32;
                        let dmg = (damage_per_tick as i32).saturating_mul(stacks);
                        if dmg > 0 {
                            self.with_recording_cause(tick_seq, |core| {
                                core.process_commands(
                                    vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
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
                }

                let next_tick_ms = time_ms.saturating_add(def.tick_interval_ms);
                if let Some(active) = self.buffs.get_mut(&key) {
                    if next_tick_ms < active.expires_at_ms {
                        active.next_tick_ms = Some(next_tick_ms);
                        self.event_queue.push(BattleEvent::BuffTick {
                            time_ms: next_tick_ms,
                            caster_instance_id,
                            target_instance_id,
                            buff_id,
                            cause_seq: Some(tick_seq),
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
                cause_seq,
            } => {
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

                self.with_recording_parent(cause_seq, |core| {
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
                Ok(())
            }
        }
    }

    fn is_alive_enemy(&self, unit_id: Uuid, owner: Side) -> bool {
        match self.units.get(&unit_id) {
            Some(unit) => unit.owner != owner && unit.stats.current_health > 0,
            None => false,
        }
    }

    fn find_nearest_alive_enemy(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.runtime_field.unit_positions.get(&from_uuid)?;
        let mut nearest: Option<(Uuid, i32)> = None;

        for (pos, placement) in &self.runtime_field.placements {
            if placement.side == from_side {
                continue;
            }
            if !matches!(
                self.units.get(&placement.uuid),
                Some(unit) if unit.stats.current_health > 0
            ) {
                continue;
            }

            let distance = from_pos.manhattan(pos);
            match nearest {
                None => nearest = Some((placement.uuid, distance)),
                Some((_best_uuid, best_dist)) if distance < best_dist => {
                    nearest = Some((placement.uuid, distance));
                }
                Some((best_uuid, best_dist))
                    if distance == best_dist
                        && placement.uuid.as_bytes() < best_uuid.as_bytes() =>
                {
                    nearest = Some((placement.uuid, distance));
                }
                _ => {}
            }
        }

        nearest.map(|(uuid, _)| uuid)
    }
}
