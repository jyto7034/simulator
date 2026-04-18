use uuid::Uuid;

use crate::ecs::resources::Position;
use crate::game::ability::{DeliveryDef, SkillHitTargetFilter};
use crate::game::battle::cooldown::{SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::core::BattleCore;
use crate::game::battle::damage::{
    apply_damage_to_unit, calculate_damage, BattleCommand, DamageContext, DamageRequest,
    DamageResult, DamageSource,
};
use crate::game::battle::enums::BattleEvent;
use crate::game::battle::enums::ProjectilePayload;
use crate::game::battle::ids::UnitInstanceId;
use crate::game::battle::timeline::{
    HpChangeReason, MovementStopReason, TimelineCause, TimelineEvent,
};
use crate::game::determinism;
use crate::game::enums::Side;
use crate::game::stats::{Effect, TriggerEffectTarget, TriggerType};

use super::{
    movement::{ActionState, ContinuousPosition, TILE_UNITS_PER_TILE},
    types::{CommandExecutionSummary, ProjectileGuidance},
};

#[derive(Debug, Clone)]
pub(super) struct ProjectileLaunch {
    pub(super) fired_at_ms: u64,
    pub(super) attacker_instance_id: UnitInstanceId,
    pub(super) target_instance_id: UnitInstanceId,
    pub(super) attacker_pos: Position,
    pub(super) target_pos: Position,
    pub(super) attacker_origin: ContinuousPosition,
    pub(super) target_aim: ContinuousPosition,
    pub(super) speed_units_per_ms: u32,
    pub(super) guidance: ProjectileGuidance,
    pub(super) payload: ProjectilePayload,
}

#[derive(Debug, Clone, Copy)]
struct AttackSourceSnapshot {
    instance_id: UnitInstanceId,
    owner: Side,
    attack: u32,
}

#[derive(Debug, Clone, Copy)]
struct AttackTargetSnapshot {
    instance_id: UnitInstanceId,
    owner: Side,
    defense: u32,
    current_hp: u32,
    max_hp: u32,
}

#[derive(Debug, Clone, Copy)]
struct BasicAttackDamageSnapshot {
    attacker: AttackSourceSnapshot,
    target: AttackTargetSnapshot,
    time_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct TriggerEffectContext {
    trigger_unit_id: UnitInstanceId,
    counterpart_unit_id: Option<UnitInstanceId>,
}

fn projectile_flight_ms(distance_units: u64, speed_units_per_ms: u32) -> u64 {
    if distance_units == 0 {
        return 0;
    }
    let speed_units_per_ms = speed_units_per_ms as u64;
    if speed_units_per_ms == 0 {
        // Defensive: treat invalid projectile speed as "instant" rather than panicking/overflowing.
        return 0;
    }
    distance_units.saturating_add(speed_units_per_ms.saturating_sub(1)) / speed_units_per_ms
}

pub(super) fn projectile_flight_ms_for_delivery(
    distance_units: u64,
    speed_units_per_ms: u32,
) -> u64 {
    projectile_flight_ms(distance_units, speed_units_per_ms)
}

impl BattleCore {
    pub(super) fn unit_continuous_position_or_tile_center(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> Option<ContinuousPosition> {
        if let Some(unit) = self.units.get(&unit_instance_id) {
            return Some(ContinuousPosition::new(unit.pos_x_units, unit.pos_y_units));
        }

        self.graveyard
            .get(&unit_instance_id)
            .map(|snapshot| ContinuousPosition::tile_center(snapshot.position))
    }

    pub(super) fn activation_commands_from_bindings(
        bindings: Vec<SourcedAbilityActivation>,
        caster_id: UnitInstanceId,
        target_id: Option<UnitInstanceId>,
    ) -> Vec<BattleCommand> {
        bindings
            .into_iter()
            .map(|sourced| match sourced.binding.activation {
                crate::game::ability::AbilityActivationDef::TriggerProc {
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    ..
                } => BattleCommand::TriggerAbility {
                    skill_id: sourced.binding.ability_id,
                    caster_id,
                    target_id,
                    activation_source: sourced.source,
                    binding_index: sourced.binding_index,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster: false,
                },
            })
            .collect()
    }

    fn trigger_commands_from_effects(
        &self,
        effects: Vec<SourcedEffect>,
        context: TriggerEffectContext,
        _target_id: Option<UnitInstanceId>,
        _allow_dead_caster: bool,
    ) -> Vec<BattleCommand> {
        let mut commands = Vec::new();

        for sourced in effects {
            let targets = self.resolve_trigger_effect_targets(context, sourced.target);
            match sourced.effect {
                Effect::Modifier(modifier) => {
                    commands.extend(targets.iter().copied().map(|target_id| {
                        BattleCommand::ApplyModifier {
                            target_id,
                            modifier,
                        }
                    }));
                }
                Effect::Heal { flat, percent } => {
                    commands.extend(targets.iter().copied().map(|target_id| {
                        BattleCommand::ApplyHeal {
                            target_id,
                            flat,
                            percent,
                            source_id: Some(context.trigger_unit_id),
                        }
                    }));
                }
                Effect::ApplyBuff {
                    buff_id,
                    duration_ms,
                } => {
                    commands.extend(targets.iter().copied().map(|target_id| {
                        BattleCommand::ApplyBuff {
                            caster_id: context.trigger_unit_id,
                            target_id,
                            buff_id: crate::game::battle::buffs::BuffId::from_name(&buff_id),
                            duration_ms,
                        }
                    }));
                }
                Effect::BonusDamage { .. } => {}
            }
        }

        commands
    }

    fn resolve_trigger_effect_targets(
        &self,
        context: TriggerEffectContext,
        target: TriggerEffectTarget,
    ) -> Vec<UnitInstanceId> {
        let owner = match self.units.get(&context.trigger_unit_id) {
            Some(unit) => unit.owner,
            None => return Vec::new(),
        };

        let mut resolved = match target {
            TriggerEffectTarget::SelfUnit => vec![context.trigger_unit_id],
            TriggerEffectTarget::CounterpartUnit => context
                .counterpart_unit_id
                .map(|unit_id| vec![unit_id])
                .unwrap_or_default(),
            TriggerEffectTarget::AllAllies => self
                .units
                .values()
                .filter(|unit| unit.owner == owner && !unit.is_dead())
                .map(|unit| unit.instance_id)
                .collect(),
            TriggerEffectTarget::AllEnemies => self
                .units
                .values()
                .filter(|unit| unit.owner != owner && !unit.is_dead())
                .map(|unit| unit.instance_id)
                .collect(),
        };

        resolved.retain(|unit_id| self.units.get(unit_id).is_some_and(|unit| !unit.is_dead()));
        resolved.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        resolved.dedup();
        resolved
    }

    fn death_trigger_commands(
        &self,
        killer_id: Option<UnitInstanceId>,
        dead_unit_id: UnitInstanceId,
        dead_owner: Side,
    ) -> Vec<BattleCommand> {
        let mut commands = Vec::new();

        commands.extend(self.trigger_commands_from_effects(
            self.collect_all_triggers(dead_unit_id, TriggerType::OnDeath),
            TriggerEffectContext {
                trigger_unit_id: dead_unit_id,
                counterpart_unit_id: killer_id,
            },
            None,
            true,
        ));
        commands.extend(
            Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(dead_unit_id, TriggerType::OnDeath),
                dead_unit_id,
                None,
            )
            .into_iter()
            .map(|command| match command {
                BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    binding_index,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    ..
                } => BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    binding_index,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster: true,
                },
                other => other,
            }),
        );

        if let Some(killer_id) = killer_id {
            commands.extend(self.trigger_commands_from_effects(
                self.collect_all_triggers(killer_id, TriggerType::OnKill),
                TriggerEffectContext {
                    trigger_unit_id: killer_id,
                    counterpart_unit_id: Some(dead_unit_id),
                },
                Some(dead_unit_id),
                false,
            ));
            commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(killer_id, TriggerType::OnKill),
                killer_id,
                Some(dead_unit_id),
            ));
        }

        let mut ally_ids: Vec<_> = self
            .units
            .values()
            .filter(|unit| {
                unit.instance_id != dead_unit_id && unit.owner == dead_owner && !unit.is_dead()
            })
            .map(|unit| unit.instance_id)
            .collect();
        ally_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for ally_id in ally_ids {
            commands.extend(self.trigger_commands_from_effects(
                self.collect_all_triggers(ally_id, TriggerType::OnAllyDeath),
                TriggerEffectContext {
                    trigger_unit_id: ally_id,
                    counterpart_unit_id: Some(dead_unit_id),
                },
                None,
                false,
            ));
            commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(ally_id, TriggerType::OnAllyDeath),
                ally_id,
                None,
            ));
        }

        commands
    }

    fn finalize_unit_death(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        target_owner: Side,
        time_ms: u64,
    ) {
        let interrupted = self.interrupt_movement(
            time_ms,
            target_instance_id,
            MovementStopReason::Died,
            None,
            ActionState::Dead,
        );
        if !interrupted {
            if let Some(target) = self.units.get_mut(&target_instance_id) {
                target.move_epoch = target.move_epoch.wrapping_add(1);
                target.action_state = ActionState::Dead;
            }
        }

        if let Some(position) = self.battlefield.remove(target_instance_id) {
            if let Some(target) = self.units.get(&target_instance_id) {
                self.graveyard
                    .insert(target_instance_id, target.to_snapshot(position));
            }
        }
        let death_seq = self.record_timeline(
            time_ms,
            TimelineEvent::UnitDied {
                unit_instance_id: target_instance_id,
                owner: target_owner,
                killer_instance_id: source_instance_id,
            },
        );

        let death_commands =
            self.death_trigger_commands(source_instance_id, target_instance_id, target_owner);
        if !death_commands.is_empty() {
            self.with_recording_cause(death_seq, |core| {
                core.process_commands(death_commands, time_ms);
            });
        }

        // Death changes both occupancy and target validity globally.
        // Re-run movement selection in the same tick so melee units do not
        // wait for their next attack cadence before advancing again.
        self.schedule_movement_intent(time_ms);
    }

    pub(super) fn schedule_projectile_hit_event(&mut self, launch: ProjectileLaunch) {
        // TODO: 추후 config 로 빼야함.
        const PROJECTILE_NS: u64 = 0x5052_4F4A_4543_544Cu64; // "PROJECTL"

        let dist_tiles = launch.attacker_pos.chebyshev(&launch.target_pos).max(0) as u64;
        let distance_units = dist_tiles.saturating_mul(TILE_UNITS_PER_TILE);
        let flight_ms = projectile_flight_ms(distance_units, launch.speed_units_per_ms);
        let impact_ms = launch.fired_at_ms.saturating_add(flight_ms);

        let seed = self
            .recording_cause()
            .and_then(|cause| cause.parent_seq())
            .unwrap_or_else(|| {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&launch.attacker_instance_id.as_bytes()[..8]);
                launch.fired_at_ms ^ u64::from_be_bytes(bytes)
            });
        let projectile_seq = self.projectile_seq;
        self.projectile_seq = self.projectile_seq.wrapping_add(1);
        let projectile_id = determinism::uuid_v4_from_seed(seed, PROJECTILE_NS, projectile_seq);

        self.projectiles.insert(
            projectile_id,
            super::ProjectileRecord {
                fired_at_ms: launch.fired_at_ms,
                attacker_instance_id: launch.attacker_instance_id,
                target_instance_id: launch.target_instance_id,
                start: launch.attacker_origin,
                aim: launch.target_aim,
                speed_units_per_ms: launch.speed_units_per_ms,
                guidance: launch.guidance,
            },
        );

        self.event_queue.push(BattleEvent::ProjectileHit {
            time_ms: impact_ms,
            projectile_id,
            attacker_instance_id: launch.attacker_instance_id,
            target_instance_id: launch.target_instance_id,
            payload: launch.payload,
            cause: self.recording_cause().unwrap_or_default(),
        });
    }

    pub(super) fn skill_delivery_accepts_unit(
        &self,
        caster_owner: Side,
        caster_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    ) -> bool {
        if target_id == caster_instance_id && !include_caster {
            return false;
        }

        let Some(target) = self.units.get(&target_id) else {
            return false;
        };
        if target.is_dead() {
            return false;
        }

        match hit_targets {
            SkillHitTargetFilter::Allies => target.owner == caster_owner,
            SkillHitTargetFilter::Enemies => target.owner != caster_owner,
            SkillHitTargetFilter::Any => true,
        }
    }

    pub(super) fn sample_unit_position_at(
        &self,
        unit_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Option<ContinuousPosition> {
        self.sample_motion_segment_at(unit_instance_id, time_ms)
            .map(|sample| sample.position_at(time_ms))
            .or_else(|| self.unit_continuous_position_or_tile_center(unit_instance_id))
    }

    fn calculate_basic_attack_damage_snapshot(
        &mut self,
        snapshot: BasicAttackDamageSnapshot,
    ) -> DamageResult {
        let on_attack_effects =
            self.collect_all_triggers(snapshot.attacker.instance_id, TriggerType::OnAttack);
        let on_hit_effects =
            self.collect_all_triggers(snapshot.target.instance_id, TriggerType::OnHit);
        let mut trigger_effect_commands = self.trigger_commands_from_effects(
            on_attack_effects.clone(),
            TriggerEffectContext {
                trigger_unit_id: snapshot.attacker.instance_id,
                counterpart_unit_id: Some(snapshot.target.instance_id),
            },
            Some(snapshot.target.instance_id),
            false,
        );
        trigger_effect_commands.extend(self.trigger_commands_from_effects(
            on_hit_effects.clone(),
            TriggerEffectContext {
                trigger_unit_id: snapshot.target.instance_id,
                counterpart_unit_id: Some(snapshot.attacker.instance_id),
            },
            Some(snapshot.attacker.instance_id),
            false,
        ));
        let mut trigger_ability_commands = Self::activation_commands_from_bindings(
            self.collect_all_trigger_activations(
                snapshot.attacker.instance_id,
                TriggerType::OnAttack,
            ),
            snapshot.attacker.instance_id,
            Some(snapshot.target.instance_id),
        );
        trigger_ability_commands.extend(Self::activation_commands_from_bindings(
            self.collect_all_trigger_activations(snapshot.target.instance_id, TriggerType::OnHit),
            snapshot.target.instance_id,
            Some(snapshot.attacker.instance_id),
        ));

        let ctx = DamageContext {
            attacker_side: snapshot.attacker.owner,
            target_side: snapshot.target.owner,
            attacker_attack: snapshot.attacker.attack,
            target_defense: snapshot.target.defense,
            target_current_hp: snapshot.target.current_hp,
            target_max_hp: snapshot.target.max_hp,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            attacker_id: snapshot.attacker.instance_id,
            target_id: snapshot.target.instance_id,
            base_damage: snapshot.attacker.attack,
            time_ms: snapshot.time_ms,
        };

        let mut result = calculate_damage(&request, &ctx);
        result
            .triggered_commands
            .splice(0..0, trigger_effect_commands);
        result
    }

    fn apply_damage_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        damage: u32,
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        let (target_owner, hp_before) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (target.owner, target.stats.current_health)
        };

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        apply_damage_to_unit(&mut target.stats, damage);
        let hp_after = target.stats.current_health;
        let delta = hp_after as i32 - hp_before as i32;

        self.record_timeline(
            time_ms,
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            let gained = (hp_before - hp_after) / 10;
            if gained > 0 {
                self.add_resonance(target_instance_id, gained, time_ms, hp_after > 0);
            }
        }

        if hp_after == 0 {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
        }
    }

    fn apply_hp_delta_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        if delta == 0 {
            return;
        }

        let (target_owner, hp_before, max_health) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (
                target.owner,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        let hp_after = if delta > 0 {
            hp_before
                .saturating_add(delta as u32)
                .min(max_health.max(1))
        } else {
            hp_before.saturating_sub(delta.unsigned_abs())
        };
        target.stats.current_health = hp_after;

        let applied_delta = hp_after as i32 - hp_before as i32;
        self.record_timeline(
            time_ms,
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta: applied_delta,
                hp_before,
                hp_after,
                reason,
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            let gained = (hp_before - hp_after) / 10;
            if gained > 0 {
                self.add_resonance(target_instance_id, gained, time_ms, hp_after > 0);
            }
        }

        if hp_after == 0 {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
        }
    }

    pub(super) fn apply_projectile_hit(
        &mut self,
        time_ms: u64,
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        payload: ProjectilePayload,
    ) {
        if self.projectiles.remove(&projectile_id).is_none() {
            return;
        };

        match payload {
            ProjectilePayload::BasicAttack => {
                let target_alive = self
                    .units
                    .get(&target_instance_id)
                    .is_some_and(|t| !t.is_dead());
                if !target_alive {
                    self.record_timeline(
                        time_ms,
                        TimelineEvent::ProjectileMiss {
                            projectile_id,
                            attacker_instance_id,
                            target_instance_id,
                        },
                    );
                    return;
                }
            }
        }

        let (target_owner, target_defense, target_current_hp, target_max_hp) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if target.is_dead() {
                return;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let attacker_live = matches!(
            self.units.get(&attacker_instance_id),
            Some(unit) if !unit.is_dead()
        );

        let attacker_attack = if let Some(unit) = self.units.get(&attacker_instance_id) {
            unit.stats.attack
        } else {
            self.graveyard
                .get(&attacker_instance_id)
                .map(|s| s.stats.attack)
                .unwrap_or(0)
        };

        if !attacker_live {
            // 피격 시점에 공격자가 죽었다면, 기본 공격 데미지만 입힘.
            let base_damage = attacker_attack.saturating_sub(target_defense).max(1);
            self.apply_damage_and_record(
                Some(attacker_instance_id),
                target_instance_id,
                base_damage,
                time_ms,
                HpChangeReason::BasicAttack,
            );
            self.schedule_pending_autocasts(time_ms);
            return;
        }

        let attacker_owner = match self.units.get(&attacker_instance_id) {
            Some(unit) => unit.owner,
            None => target_owner,
        };

        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_instance_id, TriggerType::OnHit);
        let mut trigger_effect_commands = self.trigger_commands_from_effects(
            on_attack_effects.clone(),
            TriggerEffectContext {
                trigger_unit_id: attacker_instance_id,
                counterpart_unit_id: Some(target_instance_id),
            },
            Some(target_instance_id),
            false,
        );
        trigger_effect_commands.extend(self.trigger_commands_from_effects(
            on_hit_effects.clone(),
            TriggerEffectContext {
                trigger_unit_id: target_instance_id,
                counterpart_unit_id: Some(attacker_instance_id),
            },
            Some(attacker_instance_id),
            false,
        ));
        let mut trigger_ability_commands = Self::activation_commands_from_bindings(
            self.collect_all_trigger_activations(attacker_instance_id, TriggerType::OnAttack),
            attacker_instance_id,
            Some(target_instance_id),
        );
        trigger_ability_commands.extend(Self::activation_commands_from_bindings(
            self.collect_all_trigger_activations(target_instance_id, TriggerType::OnHit),
            target_instance_id,
            Some(attacker_instance_id),
        ));

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

        let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
        let gained = dealt / 10;
        if gained > 0 {
            self.add_resonance(attacker_instance_id, gained, time_ms, true);
        }

        self.apply_damage_and_record(
            Some(attacker_instance_id),
            target_instance_id,
            result.final_damage,
            time_ms,
            HpChangeReason::BasicAttack,
        );

        let mut triggered_commands = result.triggered_commands;
        triggered_commands.splice(0..0, trigger_effect_commands);
        if !triggered_commands.is_empty() {
            self.process_commands(triggered_commands, time_ms);
        }
        if !trigger_ability_commands.is_empty() {
            self.process_commands(trigger_ability_commands, time_ms);
        }

        self.schedule_pending_autocasts(time_ms);
    }

    pub(super) fn resolve_basic_attack(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        current_time_ms: u64,
    ) -> bool {
        let (attacker_owner, attacker_base_uuid, attacker_attack) = {
            let Some(attacker) = self.units.get(&attacker_instance_id) else {
                return false;
            };
            if attacker.is_dead() {
                return false;
            }
            (attacker.owner, attacker.base_uuid, attacker.stats.attack)
        };

        let (target_owner, target_defense, target_current_hp, target_max_hp) = {
            let Some(target) = self.units.get(&target_id) else {
                return false;
            };
            if target.is_dead() {
                return false;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        if target_owner == attacker_owner {
            return false;
        }

        let basic = self
            .game_data
            .abnormality_data
            .get_by_uuid(&attacker_base_uuid)
            .map(|m| m.basic_attack.clone())
            .unwrap_or_default();

        if !self.is_basic_attack_target_in_range(attacker_instance_id, target_id) {
            return false;
        }

        match basic.delivery {
            DeliveryDef::Instant => {
                // Immediate resonance gain on attack release.
                self.add_resonance(attacker_instance_id, 10, current_time_ms, true);

                let result =
                    self.calculate_basic_attack_damage_snapshot(BasicAttackDamageSnapshot {
                        attacker: AttackSourceSnapshot {
                            instance_id: attacker_instance_id,
                            owner: attacker_owner,
                            attack: attacker_attack,
                        },
                        target: AttackTargetSnapshot {
                            instance_id: target_id,
                            owner: target_owner,
                            defense: target_defense,
                            current_hp: target_current_hp,
                            max_hp: target_max_hp,
                        },
                        time_ms: current_time_ms,
                    });

                // Resonance gain: 10% of actual HP decrease dealt.
                let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
                let gained = dealt / 10;
                if gained > 0 {
                    self.add_resonance(attacker_instance_id, gained, current_time_ms, true);
                }

                self.apply_damage_and_record(
                    Some(attacker_instance_id),
                    target_id,
                    result.final_damage,
                    current_time_ms,
                    HpChangeReason::BasicAttack,
                );

                if !result.triggered_commands.is_empty() {
                    self.process_commands(result.triggered_commands, current_time_ms);
                }
                let mut trigger_ability_commands = Self::activation_commands_from_bindings(
                    self.collect_all_trigger_activations(
                        attacker_instance_id,
                        TriggerType::OnAttack,
                    ),
                    attacker_instance_id,
                    Some(target_id),
                );
                trigger_ability_commands.extend(Self::activation_commands_from_bindings(
                    self.collect_all_trigger_activations(target_id, TriggerType::OnHit),
                    target_id,
                    Some(attacker_instance_id),
                ));
                if !trigger_ability_commands.is_empty() {
                    self.process_commands(trigger_ability_commands, current_time_ms);
                }

                self.schedule_pending_autocasts(current_time_ms);
                true
            }

            DeliveryDef::Projectile {
                speed_units_per_ms, ..
            } => {
                let Some(attacker_pos) = self.battlefield.position_of(attacker_instance_id) else {
                    return false;
                };
                let Some(target_pos) = self.battlefield.position_of(target_id) else {
                    return false;
                };
                self.add_resonance(attacker_instance_id, 10, current_time_ms, true);
                self.schedule_projectile_hit_event(ProjectileLaunch {
                    fired_at_ms: current_time_ms,
                    attacker_instance_id,
                    target_instance_id: target_id,
                    attacker_pos,
                    target_pos,
                    attacker_origin: self
                        .unit_continuous_position_or_tile_center(attacker_instance_id)
                        .unwrap_or_else(|| ContinuousPosition::tile_center(attacker_pos)),
                    target_aim: self
                        .unit_continuous_position_or_tile_center(target_id)
                        .unwrap_or_else(|| ContinuousPosition::tile_center(target_pos)),
                    speed_units_per_ms,
                    guidance: ProjectileGuidance::Homing,
                    payload: ProjectilePayload::BasicAttack,
                });
                true
            }
            DeliveryDef::Area { .. } => {
                // Basic attacks never use area delivery. Treat unexpected data
                // defensively as an instant hit path rather than introducing a
                // separate basic-attack spatial model.
                self.add_resonance(attacker_instance_id, 10, current_time_ms, true);

                let result =
                    self.calculate_basic_attack_damage_snapshot(BasicAttackDamageSnapshot {
                        attacker: AttackSourceSnapshot {
                            instance_id: attacker_instance_id,
                            owner: attacker_owner,
                            attack: attacker_attack,
                        },
                        target: AttackTargetSnapshot {
                            instance_id: target_id,
                            owner: target_owner,
                            defense: target_defense,
                            current_hp: target_current_hp,
                            max_hp: target_max_hp,
                        },
                        time_ms: current_time_ms,
                    });

                let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
                let gained = dealt / 10;
                if gained > 0 {
                    self.add_resonance(attacker_instance_id, gained, current_time_ms, true);
                }

                self.apply_damage_and_record(
                    Some(attacker_instance_id),
                    target_id,
                    result.final_damage,
                    current_time_ms,
                    HpChangeReason::BasicAttack,
                );

                if !result.triggered_commands.is_empty() {
                    self.process_commands(result.triggered_commands, current_time_ms);
                }
                let mut trigger_ability_commands = Self::activation_commands_from_bindings(
                    self.collect_all_trigger_activations(
                        attacker_instance_id,
                        TriggerType::OnAttack,
                    ),
                    attacker_instance_id,
                    Some(target_id),
                );
                trigger_ability_commands.extend(Self::activation_commands_from_bindings(
                    self.collect_all_trigger_activations(target_id, TriggerType::OnHit),
                    target_id,
                    Some(attacker_instance_id),
                ));
                if !trigger_ability_commands.is_empty() {
                    self.process_commands(trigger_ability_commands, current_time_ms);
                }

                self.schedule_pending_autocasts(current_time_ms);
                true
            }
        }
    }

    pub(super) fn process_commands(
        &mut self,
        commands: Vec<BattleCommand>,
        current_time_ms: u64,
    ) -> CommandExecutionSummary {
        let mut summary = CommandExecutionSummary::default();
        for command in commands {
            match command {
                BattleCommand::UnitDied { .. } => {
                    // Death is derived from HP reaching 0 and recorded in HpChanged handling.
                }
                BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    binding_index,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster,
                } => {
                    if !self.should_fire_triggered_ability(
                        super::sim::TriggeredAbilityProcContext {
                            source: activation_source,
                            ability_id: &skill_id,
                            binding_index,
                            current_time_ms,
                            proc_chance_percent,
                            internal_cooldown_ms,
                            max_triggers_per_battle,
                        },
                    ) {
                        continue;
                    }

                    let explicit_target = target_id.map(|unit_instance_id| {
                        crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id }
                    });
                    let proc_seq = self.record_timeline(
                        current_time_ms,
                        TimelineEvent::TriggeredAbilityProc {
                            skill_id: skill_id.clone(),
                            caster_instance_id: caster_id,
                            target_instance_id: target_id,
                            activation_source,
                            binding_index,
                        },
                    );
                    self.invoke_ability(
                        current_time_ms,
                        caster_id,
                        &skill_id,
                        explicit_target,
                        TimelineCause::Parent { seq: proc_seq },
                        allow_dead_caster,
                    );
                }
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } => {
                    let Some(target) = self.units.get_mut(&target_id) else {
                        continue;
                    };
                    if target.is_dead() {
                        continue;
                    }

                    let before = target.stats;
                    target.stats.apply_modifier(modifier);
                    let after = target.stats;

                    self.record_timeline(
                        current_time_ms,
                        TimelineEvent::StatChanged {
                            source_instance_id: None,
                            target_instance_id: target_id,
                            modifier,
                            stats_before: before,
                            stats_after: after,
                        },
                    );
                }
                BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent,
                    source_id,
                } => {
                    let max_health = match self.units.get(&target_id) {
                        Some(unit) if !unit.is_dead() => unit.stats.max_health.max(1),
                        _ => continue,
                    };

                    let percent_delta = (i64::from(max_health) * i64::from(percent)) / 100;
                    let delta_i64 = i64::from(flat).saturating_add(percent_delta);
                    let delta = delta_i64.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    let hp_before = self
                        .units
                        .get(&target_id)
                        .map(|unit| unit.stats.current_health)
                        .unwrap_or(0);

                    self.apply_hp_delta_and_record(
                        source_id,
                        target_id,
                        delta,
                        current_time_ms,
                        HpChangeReason::Command,
                    );

                    let hp_after = self
                        .units
                        .get(&target_id)
                        .map(|unit| unit.stats.current_health)
                        .unwrap_or(hp_before);
                    if hp_after < hp_before {
                        summary.actual_damage_target_count =
                            summary.actual_damage_target_count.saturating_add(1);
                    }
                }
                BattleCommand::ModifyResonance {
                    target_id,
                    amount,
                    allow_autocast_when_full,
                } => {
                    self.modify_resonance(
                        target_id,
                        amount,
                        current_time_ms,
                        allow_autocast_when_full,
                    );
                }
                BattleCommand::ScheduleAttack {
                    attacker_id,
                    target_id,
                    time_ms,
                } => {
                    self.event_queue.push(BattleEvent::AttackStart {
                        time_ms: current_time_ms.saturating_add(time_ms),
                        attacker_instance_id: attacker_id,
                        target_instance_id: target_id,
                        schedule_next: false,
                        cause: self.recording_cause().unwrap_or_default(),
                    });
                }
                BattleCommand::ApplyBuff {
                    caster_id,
                    target_id,
                    buff_id,
                    duration_ms,
                } => {
                    self.event_queue.push(BattleEvent::ApplyBuff {
                        time_ms: current_time_ms,
                        caster_instance_id: caster_id,
                        target_instance_id: target_id,
                        buff_id,
                        duration_ms,
                        cause: self.recording_cause().unwrap_or_default(),
                    });
                }
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::projectile_flight_ms;
    use crate::ecs::resources::Position;
    use crate::game::ability::{
        AbilityActivationBinding, AbilityActivationDef, DeliveryDef, SkillCastTargetingDef,
        SkillDef, SkillStepDef, SkillTarget, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::core::movement::ContinuousPosition;
    use crate::game::battle::core::movement::{ActionState, MovementState};
    use crate::game::battle::core::types::{ProjectileGuidance, RuntimeUnit};
    use crate::game::battle::core::ProjectileRecord;
    use crate::game::battle::enums::ProjectilePayload;
    use crate::game::battle::timeline::{
        HpChangeReason, MovementStopReason, Timeline, TimelineEvent,
    };
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::{
        abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
        artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase,
        equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        GameDataBase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::{TriggerEffectTarget, TriggerType, TriggeredEffect, UnitStats};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn projectile_flight_ms_is_zero_when_speed_is_zero() {
        assert_eq!(projectile_flight_ms(1, 0), 0);
        assert_eq!(projectile_flight_ms(1_000_000, 0), 0);
    }

    #[test]
    fn projectile_flight_ms_is_zero_when_distance_is_zero() {
        assert_eq!(projectile_flight_ms(0, 0), 0);
        assert_eq!(projectile_flight_ms(0, 1), 0);
        assert_eq!(projectile_flight_ms(0, u32::MAX), 0);
    }

    #[test]
    fn projectile_flight_ms_uses_ceil_division() {
        assert_eq!(projectile_flight_ms(1_000_000, 3_000), 334);
        assert_eq!(projectile_flight_ms(1_000_000, 2_000_000), 1);
        assert_eq!(projectile_flight_ms(2_000_000, 2_000_000), 1);
    }

    #[test]
    fn schedule_projectile_hit_event_stores_homing_launch_metadata() {
        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core =
            super::BattleCore::new(&empty_deck, &empty_deck, empty_game_data(), (4, 4), 1);

        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(41));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(42));

        core.schedule_projectile_hit_event(super::ProjectileLaunch {
            fired_at_ms: 100,
            attacker_instance_id: attacker_id,
            target_instance_id: target_id,
            attacker_pos: Position::new(0, 0),
            target_pos: Position::new(2, 0),
            attacker_origin: ContinuousPosition::new(100, 200),
            target_aim: ContinuousPosition::new(900, 200),
            speed_units_per_ms: 1000,
            guidance: ProjectileGuidance::Homing,
            payload: ProjectilePayload::BasicAttack,
        });

        let stored = core
            .projectiles
            .values()
            .next()
            .copied()
            .expect("missing projectile record");
        assert_eq!(stored.attacker_instance_id, attacker_id);
        assert_eq!(stored.target_instance_id, target_id);
        assert_eq!(stored.start, ContinuousPosition::new(100, 200));
        assert_eq!(stored.aim, ContinuousPosition::new(900, 200));
        assert_eq!(stored.guidance, ProjectileGuidance::Homing);
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: crate::game::data::event_pools::EventPoolConfig {
                dawn: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                noon: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                dusk: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                midnight: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                white: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
            },
        }))
    }

    fn battle_test_game_data(
        attacker_base_uuid: Uuid,
        target_base_uuid: Uuid,
        item_base_uuid: Uuid,
        item_effects: HashMap<TriggerType, Vec<TriggeredEffect>>,
        item_activations: Vec<AbilityActivationBinding>,
        skills: Vec<SkillDef>,
    ) -> Arc<GameDataBase> {
        let attacker = AbnormalityMetadata {
            id: "attacker".to_string(),
            uuid: attacker_base_uuid,
            name: "Attacker".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        let target = AbnormalityMetadata {
            id: "target".to_string(),
            uuid: target_base_uuid,
            name: "Target".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        let item = EquipmentMetadata {
            id: "item".to_string(),
            uuid: item_base_uuid,
            name: "Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            triggered_effects: item_effects,
            ability_activations: item_activations,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![attacker, target])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![item])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(skills)),
            event_pools: crate::game::data::event_pools::EventPoolConfig {
                dawn: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                noon: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                dusk: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                midnight: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                white: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
            },
        }))
    }

    fn battle_test_game_data_with_artifact(
        attacker_base_uuid: Uuid,
        target_base_uuid: Uuid,
        artifact_base_uuid: Uuid,
        artifact_effects: HashMap<TriggerType, Vec<TriggeredEffect>>,
    ) -> Arc<GameDataBase> {
        let attacker = AbnormalityMetadata {
            id: "attacker".to_string(),
            uuid: attacker_base_uuid,
            name: "Attacker".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        let target = AbnormalityMetadata {
            id: "target".to_string(),
            uuid: target_base_uuid,
            name: "Target".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        let artifact = crate::game::data::artifact_data::ArtifactMetadata {
            id: "artifact".to_string(),
            uuid: artifact_base_uuid,
            name: "Artifact".to_string(),
            description: "".to_string(),
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            triggered_effects: artifact_effects,
            ability_activations: vec![],
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![attacker, target])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![artifact])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: crate::game::data::event_pools::EventPoolConfig {
                dawn: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                noon: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                dusk: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                midnight: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                white: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
            },
        }))
    }

    fn damage_proc_skill(id: &str, amount: i32) -> SkillDef {
        SkillDef {
            id: id.to_string(),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::FirstStepTarget,
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![crate::game::ability::SkillEffectDef::Damage { amount }],
                presentation: Default::default(),
            }],
        }
    }

    fn test_runtime_unit(
        instance_id: crate::game::battle::ids::UnitInstanceId,
        owner: Side,
        base_uuid: Uuid,
        stats: UnitStats,
    ) -> RuntimeUnit {
        RuntimeUnit {
            instance_id,
            owner,
            base_uuid,
            stats,
            pos_x_units: 0,
            pos_y_units: 0,
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn write_timeline_export(name: &str, timeline: &Timeline) -> PathBuf {
        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("timeline_exports");
        std::fs::create_dir_all(&out_dir).expect("create timeline_exports directory");
        let out_path = out_dir.join(format!("{name}.json"));
        timeline
            .write_pretty_json(&out_path)
            .expect("write timeline json");
        out_path
    }

    fn drain_event_queue(core: &mut super::BattleCore) {
        while let Some(event) = core.event_queue.pop() {
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).expect("process event");
        }
    }

    #[test]
    fn apply_projectile_hit_is_idempotent_for_same_projectile_id() {
        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core =
            super::BattleCore::new(&empty_deck, &empty_deck, empty_game_data(), (4, 4), 1);

        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(1));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(2));

        let mut attacker_stats = UnitStats::with_values(100, 100, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 0, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            RuntimeUnit {
                instance_id: attacker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: attacker_stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );

        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: target_stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );

        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        let projectile_id = Uuid::from_u128(0xAAAA);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: target_id,
                start: ContinuousPosition::new(0, 0),
                aim: ContinuousPosition::new(1_000_000, 0),
                speed_units_per_ms: 1_000,
                guidance: ProjectileGuidance::Homing,
            },
        );

        core.apply_projectile_hit(
            10,
            projectile_id,
            attacker_id,
            target_id,
            ProjectilePayload::BasicAttack,
        );

        let hp_after_first = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        core.apply_projectile_hit(
            10,
            projectile_id,
            attacker_id,
            target_id,
            ProjectilePayload::BasicAttack,
        );

        let hp_after_second = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        assert_eq!(hp_after_first, hp_after_second);

        let hits = core
            .timeline
            .entries
            .iter()
            .filter(|entry| match &entry.event {
                crate::game::battle::timeline::TimelineEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == Some(attacker_id)
                    && *target_instance_id == target_id
                    && *reason == HpChangeReason::BasicAttack =>
                {
                    true
                }
                _ => false,
            })
            .count();
        assert_eq!(hits, 1);

        write_timeline_export(
            "apply_projectile_hit_is_idempotent_for_same_projectile_id",
            &core.timeline,
        );
    }

    #[test]
    fn apply_hp_delta_records_died_stop_with_latest_continuous_position() {
        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core =
            super::BattleCore::new(&empty_deck, &empty_deck, empty_game_data(), (4, 4), 1);

        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(3));
        let mut target_stats = UnitStats::with_values(100, 100, 0, 0, 1000);
        target_stats.move_speed_units_per_ms = 10;

        let mut movement = MovementState::new_at(Position::new(0, 0), 100);
        movement.target_x_units = 100;
        movement.target_y_units = 0;
        movement.last_update_ms = 100;

        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: target_stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.battlefield
            .place(target_id, Position::new(0, 0))
            .unwrap();

        core.apply_hp_delta_and_record(None, target_id, -999, 105, HpChangeReason::Command);

        let target = core
            .units
            .get(&target_id)
            .expect("target should remain in units map");
        assert_eq!(target.stats.current_health, 0);
        assert_eq!(target.pos_x_units, 50);
        assert!(matches!(target.action_state, ActionState::Dead));

        let stop_entry = core
            .timeline
            .entries
            .iter()
            .find(|entry| matches!(entry.event, TimelineEvent::MovementStopped { .. }))
            .expect("missing MovementStopped");

        match &stop_entry.event {
            TimelineEvent::MovementStopped {
                reason,
                pos_x_units,
                pos_y_units,
                ..
            } => {
                assert_eq!(*reason, MovementStopReason::Died);
                assert_eq!(*pos_x_units, 50);
                assert_eq!(*pos_y_units, 0);
            }
            _ => unreachable!("expected MovementStopped"),
        }
    }

    #[test]
    fn item_activation_invokes_ability_immediately() {
        let attacker_base_uuid = Uuid::from_u128(0x10);
        let target_base_uuid = Uuid::from_u128(0x20);
        let item_base_uuid = Uuid::from_u128(0x30);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(1));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(2));
        let item_instance_id = Uuid::from_u128(3);

        let activation = AbilityActivationBinding {
            ability_id: "item_proc".to_string(),
            activation: AbilityActivationDef::TriggerProc {
                trigger: TriggerType::OnAttack,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: None,
            },
        };
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            target_base_uuid,
            item_base_uuid,
            HashMap::new(),
            vec![activation],
            vec![damage_proc_skill("item_proc", 5)],
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core = super::BattleCore::new(&empty_deck, &empty_deck, game_data, (4, 4), 1);

        let mut attacker_stats = UnitStats::with_values(100, 100, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 1, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                attacker_stats,
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(target_id, Side::Opponent, target_base_uuid, target_stats),
        );
        core.items.insert(
            item_instance_id,
            crate::game::battle::core::RuntimeItem {
                instance_id: item_instance_id,
                owner: Side::Player,
                owner_unit_instance: attacker_id,
                base_uuid: item_base_uuid,
            },
        );
        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        assert_eq!(target_hp, 85);
        assert!(core.timeline.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityCast { skill_id, caster_instance_id, target_instance_id }
                if skill_id == "item_proc"
                    && *caster_instance_id == attacker_id
                    && *target_instance_id == Some(target_id)
            )
        }));
    }

    #[test]
    fn item_on_attack_counterpart_buff_targets_enemy_not_attacker() {
        let attacker_base_uuid = Uuid::from_u128(0x41);
        let target_base_uuid = Uuid::from_u128(0x42);
        let item_base_uuid = Uuid::from_u128(0x43);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(41));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(42));
        let item_instance_id = Uuid::from_u128(43);

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::ApplyBuff {
                    buff_id: "poison".to_string(),
                    duration_ms: 250,
                },
            )],
        );
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            target_base_uuid,
            item_base_uuid,
            item_effects,
            vec![],
            vec![],
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core = super::BattleCore::new(&empty_deck, &empty_deck, game_data, (4, 4), 1);

        let mut attacker_stats = UnitStats::with_values(100, 100, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 1, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                attacker_stats,
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(target_id, Side::Opponent, target_base_uuid, target_stats),
        );
        core.items.insert(
            item_instance_id,
            crate::game::battle::core::RuntimeItem {
                instance_id: item_instance_id,
                owner: Side::Player,
                owner_unit_instance: attacker_id,
                base_uuid: item_base_uuid,
            },
        );
        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);

        let poison = crate::game::battle::buffs::BuffId::from_name("poison");
        assert!(core.timeline.entries.iter().any(|entry| matches!(
            &entry.event,
            TimelineEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                ..
            } if *caster_instance_id == attacker_id
                && *target_instance_id == target_id
                && *buff_id == poison
        )));
        assert!(!core.timeline.entries.iter().any(|entry| matches!(
            &entry.event,
            TimelineEvent::BuffApplied {
                target_instance_id,
                buff_id,
                ..
            } if *target_instance_id == attacker_id && *buff_id == poison
        )));
    }

    #[test]
    fn artifact_on_attack_all_allies_heals_entire_team() {
        let attacker_base_uuid = Uuid::from_u128(0x51);
        let target_base_uuid = Uuid::from_u128(0x52);
        let artifact_base_uuid = Uuid::from_u128(0x53);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(51));
        let ally_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(54));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(52));
        let artifact_instance_id = Uuid::from_u128(53);

        let mut artifact_effects = HashMap::new();
        artifact_effects.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::AllAllies,
                crate::game::stats::Effect::Heal {
                    flat: 7,
                    percent: 0,
                },
            )],
        );
        let game_data = battle_test_game_data_with_artifact(
            attacker_base_uuid,
            target_base_uuid,
            artifact_base_uuid,
            artifact_effects,
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core = super::BattleCore::new(&empty_deck, &empty_deck, game_data, (4, 4), 1);

        let mut attacker_stats = UnitStats::with_values(100, 60, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut ally_stats = UnitStats::with_values(100, 40, 5, 0, 1000);
        ally_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 1, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                attacker_stats,
            ),
        );
        core.units.insert(
            ally_id,
            test_runtime_unit(ally_id, Side::Player, attacker_base_uuid, ally_stats),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(target_id, Side::Opponent, target_base_uuid, target_stats),
        );
        core.artifacts.insert(
            artifact_instance_id,
            crate::game::battle::core::RuntimeArtifact {
                instance_id: artifact_instance_id,
                owner: Side::Player,
                base_uuid: artifact_base_uuid,
            },
        );
        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(ally_id, Position::new(0, 1))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);

        assert_eq!(
            core.units.get(&attacker_id).unwrap().stats.current_health,
            67
        );
        assert_eq!(core.units.get(&ally_id).unwrap().stats.current_health, 47);
        assert_eq!(core.units.get(&target_id).unwrap().stats.current_health, 90);
    }

    #[test]
    fn item_activation_proc_respects_internal_cooldown() {
        let attacker_base_uuid = Uuid::from_u128(0x11);
        let target_base_uuid = Uuid::from_u128(0x21);
        let item_base_uuid = Uuid::from_u128(0x31);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(11));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(12));
        let item_instance_id = Uuid::from_u128(13);

        let activation = AbilityActivationBinding {
            ability_id: "proc_icd".to_string(),
            activation: AbilityActivationDef::TriggerProc {
                trigger: TriggerType::OnAttack,
                proc_chance_percent: 100,
                internal_cooldown_ms: 1_000,
                max_triggers_per_battle: None,
            },
        };
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            target_base_uuid,
            item_base_uuid,
            HashMap::new(),
            vec![activation],
            vec![damage_proc_skill("proc_icd", 5)],
        );

        let empty_deck = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };
        let mut core = super::BattleCore::new(&empty_deck, &empty_deck, game_data, (4, 4), 7);

        let mut attacker_stats = UnitStats::with_values(100, 100, 10, 0, 1000);
        attacker_stats.move_speed_units_per_ms = 1;
        let mut target_stats = UnitStats::with_values(100, 100, 1, 0, 1000);
        target_stats.move_speed_units_per_ms = 1;

        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                attacker_stats,
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(target_id, Side::Opponent, target_base_uuid, target_stats),
        );
        core.items.insert(
            item_instance_id,
            crate::game::battle::core::RuntimeItem {
                instance_id: item_instance_id,
                owner: Side::Player,
                owner_unit_instance: attacker_id,
                base_uuid: item_base_uuid,
            },
        );
        core.battlefield
            .place(attacker_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        assert!(core.resolve_basic_attack(attacker_id, target_id, 10));
        drain_event_queue(&mut core);

        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        assert_eq!(target_hp, 75);
        let proc_casts = core
            .timeline
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    &entry.event,
                    TimelineEvent::AbilityCast { skill_id, .. } if skill_id == "proc_icd"
                )
            })
            .count();
        assert_eq!(proc_casts, 1);
    }
}
