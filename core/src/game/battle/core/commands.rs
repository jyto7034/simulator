use uuid::Uuid;

use crate::game::{
    ability::AbilityId,
    battle::{
        ability_executor::AbilityResult,
        timeline::{HpChangeReason, TimelineEvent},
    },
    stats::TriggerType,
};

use super::BattleCore;

use crate::game::battle::{
    ability_executor::{AbilityRequest, UnitSnapshot},
    damage::{
        apply_damage_to_unit, calculate_damage, BattleCommand, DamageContext, DamageRequest,
        DamageSource,
    },
    death::DeadUnit,
    enums::BattleEvent,
};

impl BattleCore {
    pub(super) fn apply_attack(&mut self, attacker_instance_id: Uuid, current_time_ms: u64) {
        let (attacker_owner, attacker_attack, target_id) = {
            let Some(attacker) = self.units.get(&attacker_instance_id) else {
                return;
            };
            if attacker.stats.current_health == 0 {
                return;
            }
            let Some(target_id) = attacker.current_target else {
                return;
            };
            (attacker.owner, attacker.stats.attack, target_id)
        };

        let (target_owner, target_defense, target_current_hp, target_max_hp) = {
            let Some(target) = self.units.get(&target_id) else {
                return;
            };
            if target.stats.current_health == 0 {
                return;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        if target_owner == attacker_owner {
            return;
        }

        // 공격 판정이 발생하면(명중 여부 무관) 즉시 +10 공명
        self.add_resonance(attacker_instance_id, 10, current_time_ms, true);

        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_id, TriggerType::OnHit);

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
            target_id,
            base_damage: attacker_attack,
            time_ms: current_time_ms,
        };

        let result = calculate_damage(&request, &ctx);

        let mut target_resonance_gain: Option<(u32, bool)> = None;
        if let Some(target) = self.units.get_mut(&target_id) {
            let hp_before = target.stats.current_health;
            apply_damage_to_unit(&mut target.stats, result.final_damage);
            let hp_after = target.stats.current_health;
            let delta = hp_after as i32 - hp_before as i32;

            // 피해로 HP가 실제 감소할 때 floor(실제 감소 HP * 0.1) 공명
            // 오버킬은 “실제 감소한 HP”만 인정하며, 만땅이 되는 순간 즉사면 자동 시전은 스킵.
            if hp_after < hp_before {
                let actual_decrease = hp_before - hp_after;
                let gained = actual_decrease / 10;
                if gained > 0 {
                    target_resonance_gain = Some((gained, hp_after > 0));
                }
            }

            self.record_timeline(
                current_time_ms,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(attacker_instance_id),
                    target_instance_id: target_id,
                    delta,
                    hp_before,
                    hp_after,
                    reason: HpChangeReason::BasicAttack,
                },
            );
        }

        if let Some((gained, allow_autocast)) = target_resonance_gain {
            self.add_resonance(target_id, gained, current_time_ms, allow_autocast);
        }

        self.process_commands(result.triggered_commands, current_time_ms);

        // 해당 공격/피격 처리(데미지/사망 처리 포함)가 끝난 직후 자동 시전 시작.
        self.schedule_pending_autocasts(current_time_ms);
    }

    pub(super) fn process_commands(&mut self, commands: Vec<BattleCommand>, current_time_ms: u64) {
        for command in commands {
            match command {
                BattleCommand::UnitDied { unit_id, killer_id } => {
                    if let Some(unit) = self.units.get(&unit_id) {
                        self.death_handler.enqueue_death(DeadUnit {
                            unit_id,
                            killer_id,
                            owner: unit.owner,
                        });
                    }
                }
                BattleCommand::ExecuteAbility {
                    ability_id,
                    caster_id,
                    target_id,
                } => {
                    let result = self.execute_ability_via_executor(
                        ability_id,
                        caster_id,
                        target_id,
                        current_time_ms,
                    );

                    let cast_seq = result.executed.then(|| {
                        self.record_timeline(
                            current_time_ms,
                            TimelineEvent::AbilityCast {
                                ability_id,
                                caster_instance_id: caster_id,
                                target_instance_id: target_id,
                            },
                        )
                    });

                    if !result.commands.is_empty() {
                        if let Some(cast_seq) = cast_seq {
                            self.with_recording_cause(cast_seq, |core| {
                                core.process_commands(result.commands, current_time_ms);
                            });
                        } else {
                            self.process_commands(result.commands, current_time_ms);
                        }
                    }
                }
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } => {
                    if let Some(unit) = self.units.get_mut(&target_id) {
                        if unit.stats.current_health == 0 {
                            continue;
                        }
                        let stats_before = unit.stats;
                        unit.stats.apply_modifier(modifier);
                        let stats_after = unit.stats;
                        self.record_timeline(
                            current_time_ms,
                            TimelineEvent::StatChanged {
                                source_instance_id: None,
                                target_instance_id: target_id,
                                modifier,
                                stats_before,
                                stats_after,
                            },
                        );
                    }
                }
                BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent,
                    source_id,
                } => {
                    let mut target_resonance_gain: Option<(u32, bool)> = None;
                    if let Some(unit) = self.units.get_mut(&target_id) {
                        if unit.stats.current_health == 0 {
                            continue;
                        }
                        let owner = unit.owner;
                        let hp_before = unit.stats.current_health;
                        let percent_delta = (unit.stats.max_health as i128)
                            .saturating_mul(i128::from(percent))
                            / 100;
                        let delta = i128::from(flat) + percent_delta;
                        unit.stats.current_health = (i128::from(hp_before) + delta)
                            .clamp(0, unit.stats.max_health as i128)
                            as u32;

                        if unit.stats.current_health == 0 {
                            self.death_handler.enqueue_death(DeadUnit {
                                unit_id: target_id,
                                killer_id: source_id,
                                owner,
                            });
                        }

                        let hp_after = unit.stats.current_health;
                        let delta = hp_after as i32 - hp_before as i32;
                        if delta != 0 {
                            self.record_timeline(
                                current_time_ms,
                                TimelineEvent::HpChanged {
                                    source_instance_id: source_id,
                                    target_instance_id: target_id,
                                    delta,
                                    hp_before,
                                    hp_after,
                                    reason: HpChangeReason::Command,
                                },
                            );
                        }

                        // 피해로 HP가 실제 감소할 때 floor(실제 감소 HP * 0.1) 공명
                        if hp_after < hp_before {
                            let actual_decrease = hp_before - hp_after;
                            let gained = actual_decrease / 10;
                            if gained > 0 {
                                target_resonance_gain = Some((gained, hp_after > 0));
                            }
                        }
                    }
                    if let Some((gained, allow_autocast)) = target_resonance_gain {
                        self.add_resonance(target_id, gained, current_time_ms, allow_autocast);
                    }
                }
                BattleCommand::ScheduleAttack {
                    attacker_id,
                    target_id,
                    time_ms,
                } => {
                    self.event_queue.push(BattleEvent::Attack {
                        time_ms: current_time_ms.saturating_add(time_ms),
                        attacker_instance_id: attacker_id,
                        target_instance_id: target_id,
                        schedule_next: false,
                        cause_seq: self.recording_cause_seq(),
                    });
                }
                BattleCommand::ApplyBuff {
                    caster_id,
                    target_id,
                    buff_id,
                    duration_ms,
                } => {
                    let caster_alive = matches!(
                        self.units.get(&caster_id),
                        Some(unit) if unit.stats.current_health > 0
                    );
                    if !caster_alive {
                        continue;
                    }
                    self.event_queue.push(BattleEvent::ApplyBuff {
                        time_ms: current_time_ms,
                        caster_instance_id: caster_id,
                        target_instance_id: target_id,
                        buff_id,
                        duration_ms,
                        cause_seq: self.recording_cause_seq(),
                    });
                }
            }
        }

        self.process_pending_deaths(current_time_ms);
    }

    pub(super) fn execute_ability_via_executor(
        &mut self,
        ability_id: AbilityId,
        caster_id: Uuid,
        target_id: Option<Uuid>,
        current_time_ms: u64,
    ) -> AbilityResult {
        let Some(caster) = self.units.get(&caster_id) else {
            return AbilityResult {
                executed: false,
                commands: Vec::new(),
            };
        };
        if caster.stats.current_health == 0 {
            return AbilityResult {
                executed: false,
                commands: Vec::new(),
            };
        }
        let caster_snapshot = caster.to_snapshot();

        let mut unit_snapshots: Vec<UnitSnapshot> = self
            .units
            .values()
            .filter(|u| u.stats.current_health > 0)
            .map(|u| u.to_snapshot())
            .collect();
        unit_snapshots.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));

        let request = AbilityRequest {
            ability_id,
            caster_id,
            target_id,
            time_ms: current_time_ms,
        };

        self.ability_executor
            .execute(&request, &caster_snapshot, &unit_snapshots)
    }
}
