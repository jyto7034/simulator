use uuid::Uuid;

use crate::game::{
    ability::AbilityId,
    battle::timeline::{HpChangeReason, TimelineEvent},
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
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return;
        };
        if attacker.stats.current_health == 0 {
            return;
        }

        let Some(target_id) = attacker.current_target else {
            return;
        };

        let Some(target) = self.units.get(&target_id) else {
            return;
        };
        if target.stats.current_health == 0 {
            return;
        }
        if target.owner == attacker.owner {
            return;
        }

        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_id, TriggerType::OnHit);

        let ctx = DamageContext {
            attacker_side: attacker.owner,
            target_side: target.owner,
            attacker_attack: attacker.stats.attack,
            target_defense: target.stats.defense,
            target_current_hp: target.stats.current_health,
            target_max_hp: target.stats.max_health,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: attacker_instance_id,
            target_id,
            base_damage: attacker.stats.attack,
            time_ms: current_time_ms,
        };

        let result = calculate_damage(&request, &ctx);

        if let Some(target) = self.units.get_mut(&target_id) {
            let hp_before = target.stats.current_health;
            apply_damage_to_unit(&mut target.stats, result.final_damage);
            let hp_after = target.stats.current_health;
            let delta = -(result.final_damage as i32);

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

        self.process_commands(result.triggered_commands, current_time_ms);
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

                    if result.executed {
                        self.record_timeline(
                            current_time_ms,
                            TimelineEvent::AbilityCast {
                                ability_id,
                                caster_instance_id: caster_id,
                                target_instance_id: target_id,
                            },
                        );
                    }

                    if !result.commands.is_empty() {
                        self.process_commands(result.commands, current_time_ms);
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
                    if let Some(unit) = self.units.get_mut(&target_id) {
                        if unit.stats.current_health == 0 {
                            continue;
                        }
                        let owner = unit.owner;
                        let hp_before = unit.stats.current_health;
                        let percent_delta = (unit.stats.max_health as i64) * (percent as i64) / 100;
                        let delta = (flat as i64) + percent_delta;
                        unit.stats.current_health = (hp_before as i64 + delta)
                            .clamp(0, unit.stats.max_health as i64)
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
    ) -> crate::game::battle::ability_executor::AbilityResult {
        let Some(caster) = self.units.get(&caster_id) else {
            return crate::game::battle::ability_executor::AbilityResult {
                executed: false,
                commands: Vec::new(),
            };
        };
        if caster.stats.current_health == 0 {
            return crate::game::battle::ability_executor::AbilityResult {
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
