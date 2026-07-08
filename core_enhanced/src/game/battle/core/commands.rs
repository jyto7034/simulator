use crate::game::ability::{DeliveryDef, SkillHitTargetFilter};
use crate::game::battle::cooldown::{SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::core::BattleCore;
use crate::game::battle::damage::{
    calculate_damage, BattleCommand, DamageContext, DamageModifiers, DamageSource,
    DamageSourceSnapshot, ProcRollIdentity,
};
use crate::game::battle::enums::BattleEvent;
use crate::game::battle::event_log::{BattleEventCause, BattleLogEvent, HpChangeReason};
use crate::game::battle::ids::UnitInstanceId;
use crate::game::determinism;
use crate::game::enums::Side;
use crate::game::stats::{Effect, TriggerEffectTarget, TriggerType};
use uuid::Uuid;

use super::{
    basic_attack_projectile::ProjectileLaunch,
    damage_runtime::{AttackTargetSnapshot, BasicAttackDamageSnapshot},
    movement::types::{WorldVec2, DATA_UNITS_PER_WORLD},
    types::{CommandExecutionSummary, ProjectileGuidance},
};

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) struct TriggerEffectContext {
    pub(in crate::game::battle::core) trigger_unit_id: UnitInstanceId,
    pub(in crate::game::battle::core) counterpart_unit_id: Option<UnitInstanceId>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) struct TriggerAbilityContext {
    pub(in crate::game::battle::core) trigger_type: TriggerType,
    pub(in crate::game::battle::core) trigger_unit_id: UnitInstanceId,
    pub(in crate::game::battle::core) counterpart_unit_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) target_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) occurrence_id: Uuid,
    pub(in crate::game::battle::core) occurrence_index: u32,
}

impl BattleCore {
    pub(super) fn unit_world_position_or_tile_center(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> Option<WorldVec2> {
        if let Some(body) = self.unit_body_view(unit_instance_id) {
            return Some(body.position);
        }

        self.graveyard
            .get(&unit_instance_id)
            .map(|snapshot| snapshot.world_position)
    }

    pub(super) fn activation_commands_from_bindings(
        bindings: Vec<SourcedAbilityActivation>,
        caster_id: UnitInstanceId,
        context: TriggerAbilityContext,
    ) -> Vec<BattleCommand> {
        bindings
            .into_iter()
            .enumerate()
            .map(|(local_index, sourced)| match sourced.binding.activation {
                crate::game::ability::AbilityActivationDef::TriggerProc {
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    ..
                } => {
                    let skill_id = sourced.binding.ability_id;
                    let identity = ProcRollIdentity {
                        trigger_type: context.trigger_type,
                        activation_source: sourced.source,
                        ability_id: skill_id.clone(),
                        binding_index: sourced.binding_index,
                        caster_id,
                        trigger_unit_id: context.trigger_unit_id,
                        counterpart_unit_id: context.counterpart_unit_id,
                        target_id: context.target_id,
                        occurrence_id: context.occurrence_id,
                        occurrence_index: context
                            .occurrence_index
                            .saturating_add(local_index as u32),
                    };
                    BattleCommand::TriggerAbility {
                        skill_id,
                        caster_id,
                        target_id: context.target_id,
                        activation_source: sourced.source,
                        binding_index: sourced.binding_index,
                        proc_roll_identity: identity,
                        proc_chance_percent,
                        internal_cooldown_ms,
                        max_triggers_per_battle,
                        allow_dead_caster: false,
                    }
                }
            })
            .collect()
    }

    fn proc_occurrence_trigger_type_tag(trigger_type: TriggerType) -> u64 {
        match trigger_type {
            TriggerType::Permanent => 0x5045_524D_414E_454Eu64,
            TriggerType::OnAttack => 0x4F4E_4154_5441_434Bu64,
            TriggerType::OnHit => 0x4F4E_4849_545F_5F5Fu64,
            TriggerType::OnKill => 0x4F4E_4B49_4C4C_5F5Fu64,
            TriggerType::OnDeath => 0x4F4E_4445_4154_48u64,
            TriggerType::OnBattleStart => 0x4241_5454_4C45_5354u64,
            TriggerType::OnAllyDeath => 0x414C_4C59_4445_4144u64,
        }
    }

    pub(in crate::game::battle::core) fn proc_occurrence_id(
        &self,
        trigger_type: TriggerType,
        trigger_unit_id: UnitInstanceId,
        counterpart_unit_id: Option<UnitInstanceId>,
        time_ms: u64,
        salt: u64,
    ) -> Uuid {
        const PROC_OCCURRENCE_NS: u64 = 0x5052_4F43_4F43_4352u64; // "PROCOCCR"

        fn unit_tag(unit_id: UnitInstanceId) -> u64 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&unit_id.as_bytes()[..8]);
            u64::from_be_bytes(bytes)
        }

        let mut seed = self.seed
            ^ Self::proc_occurrence_trigger_type_tag(trigger_type).rotate_left(7)
            ^ unit_tag(trigger_unit_id).rotate_left(19)
            ^ time_ms.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ salt.rotate_left(41);
        if let Some(counterpart_unit_id) = counterpart_unit_id {
            seed ^= unit_tag(counterpart_unit_id).rotate_left(31);
        }

        determinism::uuid_v4_from_seed(seed, PROC_OCCURRENCE_NS, salt)
    }

    pub(in crate::game::battle::core) fn trigger_commands_from_effects(
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
                Effect::BonusDamage { .. } | Effect::ModifyDamage(_) => {}
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
                .filter(|unit| unit.owner == owner && unit.is_active())
                .map(|unit| unit.instance_id)
                .collect(),
            TriggerEffectTarget::AllEnemies => self
                .units
                .values()
                .filter(|unit| unit.owner != owner && unit.is_active())
                .map(|unit| unit.instance_id)
                .collect(),
        };

        resolved.retain(|unit_id| self.units.get(unit_id).is_some_and(|unit| unit.is_active()));
        resolved.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        resolved.dedup();
        resolved
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
        if !target.is_active() {
            return false;
        }

        match hit_targets {
            SkillHitTargetFilter::Allies => target.owner == caster_owner,
            SkillHitTargetFilter::Enemies => target.owner != caster_owner,
            SkillHitTargetFilter::Any => true,
        }
    }

    pub(super) fn sample_unit_world_position_at(
        &self,
        unit_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Option<crate::game::battle::core::movement::types::WorldVec2> {
        if let Some(segment) = self.active_movement_segments.get(&unit_instance_id) {
            return Some(segment.sample_position_at(time_ms));
        }

        self.unit_world_position_or_tile_center(unit_instance_id)
    }

    pub(in crate::game::battle::core) fn sample_unit_body_at(
        &self,
        unit_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Option<crate::game::battle::core::movement::types::UnitBody> {
        let mut body = self.unit_body_view(unit_instance_id)?;
        if let Some(position) = self.sample_unit_world_position_at(unit_instance_id, time_ms) {
            body.position = position;
        }
        Some(body)
    }

    #[cfg(test)]
    pub(super) fn resolve_basic_attack(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        current_time_ms: u64,
    ) -> bool {
        let delivery = self.basic_attack_delivery_for_unit(attacker_instance_id);
        let Some((source_attack, damage_type)) = self
            .units
            .get(&attacker_instance_id)
            .filter(|unit| unit.is_active())
            .map(|unit| (unit.stats.attack, unit.basic_attack.damage_type))
        else {
            return false;
        };
        let Some(source_snapshot) = self.damage_source_snapshot_for_unit(
            attacker_instance_id,
            target_id,
            DamageSource::BasicAttack,
            damage_type,
            source_attack,
            DamageModifiers::default(),
            1,
            current_time_ms,
            true,
        ) else {
            return false;
        };
        self.resolve_committed_basic_attack(
            attacker_instance_id,
            target_id,
            source_snapshot,
            delivery,
            current_time_ms,
        )
    }

    pub(super) fn resolve_committed_basic_attack(
        &mut self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        source_snapshot: DamageSourceSnapshot,
        delivery: crate::game::battle::event_log::AttackDelivery,
        current_time_ms: u64,
    ) -> bool {
        let (
            target_owner,
            target_defense,
            target_magic_resist,
            target_incoming_modifiers,
            target_current_hp,
            target_max_hp,
        ) = {
            let Some(target) = self.units.get(&target_id) else {
                return false;
            };
            if !target.is_active() {
                return false;
            }
            (
                target.owner,
                target.stats.defense,
                target.stats.magic_resist,
                target.incoming_damage_modifiers,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        if target_owner == source_snapshot.source_side {
            return false;
        }

        let attacker_live = self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|unit| unit.is_active());
        if attacker_live && !self.is_basic_attack_target_in_range(attacker_instance_id, target_id) {
            return false;
        }

        match delivery {
            crate::game::battle::event_log::AttackDelivery::Instant => {
                self.grant_basic_attack_release_resonance(attacker_instance_id, current_time_ms);
                let attack_occurrence_id = source_snapshot.crit_roll_identity.source_instance_id;

                let result =
                    self.calculate_basic_attack_damage_snapshot(BasicAttackDamageSnapshot {
                        source: source_snapshot,
                        target: AttackTargetSnapshot {
                            instance_id: target_id,
                            owner: target_owner,
                            defense: target_defense,
                            magic_resist: target_magic_resist,
                            incoming_damage_modifiers: target_incoming_modifiers,
                            current_hp: target_current_hp,
                            max_hp: target_max_hp,
                        },
                    });

                let dealt = target_current_hp.saturating_sub(result.target_remaining_hp);
                self.grant_basic_attack_damage_dealt_resonance(
                    attacker_instance_id,
                    dealt,
                    current_time_ms,
                );

                self.apply_damage_result_and_record(
                    Some(attacker_instance_id),
                    &result,
                    current_time_ms,
                    HpChangeReason::BasicAttack,
                );

                if !result.triggered_commands.is_empty() {
                    self.process_commands(result.triggered_commands, current_time_ms);
                }
                let mut trigger_ability_commands = if attacker_live {
                    Self::activation_commands_from_bindings(
                        self.collect_all_trigger_activations(
                            attacker_instance_id,
                            TriggerType::OnAttack,
                        ),
                        attacker_instance_id,
                        TriggerAbilityContext {
                            trigger_type: TriggerType::OnAttack,
                            trigger_unit_id: attacker_instance_id,
                            counterpart_unit_id: Some(target_id),
                            target_id: Some(target_id),
                            occurrence_id: attack_occurrence_id,
                            occurrence_index: 0,
                        },
                    )
                } else {
                    Vec::new()
                };
                trigger_ability_commands.extend(Self::activation_commands_from_bindings(
                    self.collect_all_trigger_activations(target_id, TriggerType::OnHit),
                    target_id,
                    TriggerAbilityContext {
                        trigger_type: TriggerType::OnHit,
                        trigger_unit_id: target_id,
                        counterpart_unit_id: Some(attacker_instance_id),
                        target_id: Some(attacker_instance_id),
                        occurrence_id: attack_occurrence_id,
                        occurrence_index: 0,
                    },
                ));
                if !trigger_ability_commands.is_empty() {
                    self.process_commands(trigger_ability_commands, current_time_ms);
                }

                self.schedule_pending_autocasts(current_time_ms);
                true
            }

            crate::game::battle::event_log::AttackDelivery::Projectile => {
                let Some((basic, source_attack)) = self
                    .units
                    .get(&attacker_instance_id)
                    .filter(|unit| unit.is_active())
                    .map(|unit| (unit.basic_attack.clone(), unit.stats.attack))
                else {
                    return false;
                };
                let DeliveryDef::Projectile {
                    speed_units_per_ms, ..
                } = basic.delivery
                else {
                    return false;
                };
                let Some(attacker_pos) = self.live_unit_projected_tile(attacker_instance_id) else {
                    return false;
                };
                let Some(target_pos) = self.live_unit_projected_tile(target_id) else {
                    return false;
                };
                self.grant_basic_attack_release_resonance(attacker_instance_id, current_time_ms);
                let Some(launch_source_snapshot) = self.damage_source_snapshot_for_unit(
                    attacker_instance_id,
                    target_id,
                    DamageSource::BasicAttack,
                    basic.damage_type,
                    source_attack,
                    DamageModifiers::default(),
                    1,
                    current_time_ms,
                    true,
                ) else {
                    return false;
                };
                self.spawn_basic_attack_projectile(ProjectileLaunch {
                    fired_at_ms: current_time_ms,
                    attacker_instance_id,
                    attacker_owner_at_launch: launch_source_snapshot.source_side,
                    air_capable_at_launch: basic.air_capable,
                    target_instance_id: target_id,
                    attacker_origin: self
                        .unit_world_position_or_tile_center(attacker_instance_id)
                        .unwrap_or_else(|| WorldVec2::from_tile_center(attacker_pos)),
                    target_aim: self
                        .unit_world_position_or_tile_center(target_id)
                        .unwrap_or_else(|| WorldVec2::from_tile_center(target_pos)),
                    speed_units_per_ms,
                    guidance: ProjectileGuidance::Homing,
                    damage_type: basic.damage_type,
                    source_snapshot: launch_source_snapshot,
                });
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
                    proc_roll_identity,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster,
                } => {
                    if !self.should_fire_triggered_ability(
                        super::sim::TriggeredAbilityProcContext {
                            proc_roll_identity: &proc_roll_identity,
                            current_time_ms,
                            proc_chance_percent,
                            internal_cooldown_ms,
                            max_triggers_per_battle,
                        },
                    ) {
                        continue;
                    }

                    let explicit_target = target_id.map(|unit_instance_id| {
                        crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id }
                    });
                    let proc_seq = self.record_event_log(
                        current_time_ms,
                        BattleLogEvent::TriggeredAbilityProc {
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
                        BattleEventCause::Parent { seq: proc_seq },
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
                    if !target.is_active() {
                        continue;
                    }

                    let before = target.stats;
                    target.stats.apply_modifier(modifier);
                    target.body.move_speed = target.stats.move_speed_units_per_ms as f32 * 1_000.0
                        / DATA_UNITS_PER_WORLD;
                    let after = target.stats;

                    self.record_event_log(
                        current_time_ms,
                        BattleLogEvent::StatChanged {
                            source_instance_id: None,
                            target_instance_id: target_id,
                            modifier,
                            stats_before: before,
                            stats_after: after,
                        },
                    );
                }
                BattleCommand::ApplyDamage {
                    target_id,
                    source_snapshot,
                } => {
                    let Some(target) = self.units.get(&target_id) else {
                        continue;
                    };
                    if !target.is_active() {
                        continue;
                    }

                    let snapshot_on_attack_effects =
                        Self::damage_effects_from_source_snapshot(&source_snapshot);
                    let ctx = DamageContext {
                        attacker_side: source_snapshot.source_side,
                        target_side: target.owner,
                        attacker_attack: source_snapshot.source_attack,
                        target_armor: target.stats.defense,
                        target_magic_resist: target.stats.magic_resist,
                        target_incoming_modifiers: target.incoming_damage_modifiers,
                        target_current_hp: target.stats.current_health,
                        target_max_hp: target.stats.max_health,
                        on_attack_effects: &snapshot_on_attack_effects,
                        on_hit_effects: &[],
                    };
                    let request = source_snapshot.request(target_id);
                    let result = calculate_damage(&request, &ctx);
                    let killed_target =
                        ctx.target_current_hp > 0 && result.target_remaining_hp == 0;
                    self.apply_damage_result_and_record(
                        Some(source_snapshot.source_id),
                        &result,
                        current_time_ms,
                        HpChangeReason::Command,
                    );
                    let hp_after = self
                        .units
                        .get(&target_id)
                        .map(|unit| unit.stats.current_health)
                        .unwrap_or(ctx.target_current_hp);
                    if hp_after < ctx.target_current_hp {
                        summary.actual_damage_target_count =
                            summary.actual_damage_target_count.saturating_add(1);
                    }
                    if killed_target {
                        summary.killed_target_count = summary.killed_target_count.saturating_add(1);
                    }

                    if !result.triggered_commands.is_empty() {
                        self.process_commands(result.triggered_commands, current_time_ms);
                    }
                }
                BattleCommand::InterruptCast {
                    source_id,
                    target_id,
                } => {
                    let Some(target) = self.units.get(&target_id) else {
                        continue;
                    };
                    if !target.is_active() {
                        continue;
                    }

                    let Some(pending) = self
                        .units
                        .get_mut(&target_id)
                        .and_then(|unit| unit.pending_skill_cast.take())
                    else {
                        continue;
                    };

                    if let Some(unit) = self.units.get_mut(&target_id) {
                        unit.pending_cast = false;
                        unit.pending_cast_cause = None;
                    }

                    self.record_event_log(
                        current_time_ms,
                        BattleLogEvent::SkillCastInterrupted {
                            interrupter_instance_id: source_id,
                            caster_instance_id: target_id,
                            interrupted_skill_id: pending.skill_id,
                            interrupted_cast_seq: pending.start_seq,
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
                        Some(unit) if unit.is_active() => unit.stats.max_health.max(1),
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
    use crate::game::ability::{
        AbilityActivationBinding, AbilityActivationDef, DeliveryDef, SkillCastTargetingDef,
        SkillDef, SkillId, SkillStepDef, SkillTarget, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::cooldown::{CooldownSource, SourcedAbilityActivation};
    use crate::game::battle::core::movement::{
        types::{ActiveMovementSegment, UnitBody, WorldVec2, DATA_UNITS_PER_WORLD},
        ActionState, MovementSegmentEndKind,
    };
    use crate::game::battle::core::projectile_math::projectile_flight_ms;
    use crate::game::battle::core::types::{ProjectileGuidance, RuntimeUnit, RuntimeUnitLifecycle};
    use crate::game::battle::core::ProjectileRecord;
    use crate::game::battle::damage::{
        BattleCommand, CombatRollIdentity, CombatRollKind, DamageFeedbackTag, DamageModifiers,
        DamageSource, DamageSourceSnapshot, DamageType, ProcRollIdentity,
    };
    use crate::game::battle::enums::BattleEvent;
    use crate::game::battle::event_log::{
        AttackDelivery, AttackKind, BattleEventCause, BattleEventLog, BattleLogEvent,
        HpChangeReason, MovementStopReason,
    };
    use crate::game::battle::scenario::BattleScenario;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        equipment_data::{EquipmentMetadata, EquipmentType},
        skill_data::SkillDatabase,
        GameDataBase, GameDataBuilder,
    };
    use crate::game::enums::Side;
    use crate::game::resources::Position;
    use crate::game::stats::{TriggerEffectTarget, TriggerType, TriggeredEffect, UnitStats};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_damage_source_snapshot(
        source_id: crate::game::battle::ids::UnitInstanceId,
        _target_id: crate::game::battle::ids::UnitInstanceId,
        damage_source: DamageSource,
        damage_type: DamageType,
        base_damage: u32,
        time_ms: u64,
    ) -> DamageSourceSnapshot {
        DamageSourceSnapshot {
            source_id,
            source_side: Side::Player,
            source_attack: base_damage,
            source: damage_source,
            damage_type,
            base_damage,
            modifiers: DamageModifiers::default(),
            crit_roll_percent: None,
            crit_roll_identity: CombatRollIdentity::new(
                CombatRollKind::for_damage_source(damage_source),
                Some(source_id),
                Uuid::from_u128(
                    0xC011_0000_0000_0000_0000_0000_0000_0000u128
                        ^ source_id.as_uuid().as_u128()
                        ^ (time_ms as u128),
                ),
            )
            .with_target(_target_id),
            minimum_damage: 0,
            committed_at_ms: time_ms,
            on_attack_modifiers: DamageModifiers::default(),
            on_attack_bonus_damage: Vec::new(),
        }
    }

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
    fn spawn_basic_attack_projectile_stores_homing_launch_metadata() {
        let mut core = new_test_core(empty_game_data(), 1);

        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(41));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(42));

        core.spawn_basic_attack_projectile(super::ProjectileLaunch {
            fired_at_ms: 100,
            attacker_instance_id: attacker_id,
            attacker_owner_at_launch: Side::Player,
            air_capable_at_launch: false,
            target_instance_id: target_id,
            attacker_origin: WorldVec2::new(0.0001, 0.0002),
            target_aim: WorldVec2::new(0.0009, 0.0002),
            speed_units_per_ms: 1000,
            guidance: ProjectileGuidance::Homing,
            damage_type: DamageType::Physical,
            source_snapshot: test_damage_source_snapshot(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                10,
                100,
            ),
        });

        let stored = core
            .projectiles
            .values()
            .next()
            .cloned()
            .expect("missing projectile record");
        assert_eq!(stored.attacker_instance_id, attacker_id);
        assert_eq!(stored.attacker_owner_at_launch, Side::Player);
        assert!(!stored.air_capable_at_launch);
        assert_eq!(stored.target_instance_id, target_id);
        assert_eq!(stored.start, WorldVec2::new(0.0001, 0.0002));
        assert_eq!(stored.aim, WorldVec2::new(0.0009, 0.0002));
        assert_eq!(stored.guidance, ProjectileGuidance::Homing);
        assert_eq!(stored.max_travel_ms, 1);
    }

    #[test]
    fn committed_instant_basic_attack_snapshot_survives_same_timestamp_source_death() {
        let mut core = new_test_core(empty_game_data(), 1);
        let player_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD01));
        let opponent_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD02));

        let mut player = test_runtime_unit(
            player_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 100, 0, 1),
        );
        player.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut opponent = test_runtime_unit(
            opponent_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 100, 0, 1),
        );
        opponent.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.10, 1.0);
        core.units.insert(player_id, player);
        core.units.insert(opponent_id, opponent);
        place_test_unit(&mut core, player_id, Position::new(0, 0));
        place_test_unit(&mut core, opponent_id, Position::new(1, 0));

        let player_snapshot = core
            .damage_source_snapshot_for_unit(
                player_id,
                opponent_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                10,
                true,
            )
            .expect("player snapshot");
        let opponent_snapshot = core
            .damage_source_snapshot_for_unit(
                opponent_id,
                player_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                10,
                true,
            )
            .expect("opponent snapshot");

        for event in [
            BattleEvent::AttackResolve {
                time_ms: 10,
                attacker_instance_id: player_id,
                target_instance_id: opponent_id,
                source_snapshot: player_snapshot,
                kind: AttackKind::Auto,
                delivery: AttackDelivery::Instant,
                cause: BattleEventCause::default(),
            },
            BattleEvent::AttackResolve {
                time_ms: 10,
                attacker_instance_id: opponent_id,
                target_instance_id: player_id,
                source_snapshot: opponent_snapshot,
                kind: AttackKind::Auto,
                delivery: AttackDelivery::Instant,
                cause: BattleEventCause::default(),
            },
        ] {
            core.process_event(event, 10)
                .expect("process attack resolve");
        }

        assert!(core.graveyard.contains_key(&player_id));
        assert!(core.graveyard.contains_key(&opponent_id));
    }

    #[test]
    fn damage_crit_roll_is_independent_from_event_log_sequence() {
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD11));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD12));

        fn core_with_attacker_and_target(
            attacker_id: crate::game::battle::ids::UnitInstanceId,
            target_id: crate::game::battle::ids::UnitInstanceId,
        ) -> super::BattleCore {
            let mut core = new_test_core(empty_game_data(), 7);
            core.units.insert(
                attacker_id,
                test_runtime_unit(
                    attacker_id,
                    Side::Player,
                    Uuid::nil(),
                    UnitStats::with_values(100, 100, 100, 0, 1),
                ),
            );
            core.units.insert(
                target_id,
                test_runtime_unit(
                    target_id,
                    Side::Opponent,
                    Uuid::nil(),
                    UnitStats::with_values(100, 100, 100, 0, 1),
                ),
            );
            core
        }

        let mut control = core_with_attacker_and_target(attacker_id, target_id);
        let mut with_extra_log = core_with_attacker_and_target(attacker_id, target_id);

        let control_snapshot = control
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                10,
                true,
            )
            .expect("control snapshot");

        with_extra_log.record_event_log(
            5,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );

        let after_extra_log = with_extra_log
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                10,
                true,
            )
            .expect("snapshot after extra log");

        assert_ne!(with_extra_log.event_log_seq, control.event_log_seq);
        assert_eq!(
            control_snapshot.crit_roll_identity,
            after_extra_log.crit_roll_identity
        );
        assert_eq!(
            control_snapshot.crit_roll_percent,
            after_extra_log.crit_roll_percent
        );
    }

    #[test]
    fn delayed_damage_materialization_uses_hit_index_identity_not_event_log_sequence() {
        let mut core = new_test_core(empty_game_data(), 9);
        let caster_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD21));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD22));

        core.units.insert(
            caster_id,
            test_runtime_unit(
                caster_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 100, 0, 1),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 100, 0, 1),
            ),
        );

        let template = core
            .damage_source_snapshot_template_for_unit(
                caster_id,
                DamageSource::Ability,
                DamageType::Magic,
                33,
                DamageModifiers::default(),
                0,
                20,
                false,
            )
            .expect("source template");

        core.record_event_log(
            15,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );

        let first_hit =
            core.materialize_damage_source_snapshot_for_target_hit(template.clone(), target_id, 0);
        let second_hit =
            core.materialize_damage_source_snapshot_for_target_hit(template, target_id, 1);

        assert_eq!(first_hit.crit_roll_identity.target_unit_id, Some(target_id));
        assert_eq!(first_hit.crit_roll_identity.hit_index, 0);
        assert_eq!(
            second_hit.crit_roll_identity.target_unit_id,
            Some(target_id)
        );
        assert_eq!(second_hit.crit_roll_identity.hit_index, 1);
        assert_ne!(first_hit.crit_roll_identity, second_hit.crit_roll_identity);
    }

    fn test_proc_roll_identity(
        ability_id: &str,
        occurrence_id: Uuid,
        occurrence_index: u32,
    ) -> ProcRollIdentity {
        let caster_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xCA57));
        let trigger_unit_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xAA));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xBB));
        let item_instance_id = Uuid::from_u128(0x1234);
        ProcRollIdentity {
            trigger_type: TriggerType::OnAttack,
            activation_source: CooldownSource::Item { item_instance_id },
            ability_id: SkillId::from(ability_id),
            binding_index: 0,
            caster_id,
            trigger_unit_id,
            counterpart_unit_id: Some(target_id),
            target_id: Some(target_id),
            occurrence_id,
            occurrence_index,
        }
    }

    #[test]
    fn proc_roll_percent_is_independent_from_event_log_sequence() {
        let identity = test_proc_roll_identity("proc_roll_log_test", Uuid::from_u128(0x9001), 0);
        let control = new_test_core(empty_game_data(), 99);
        let mut with_extra_log = new_test_core(empty_game_data(), 99);

        with_extra_log.record_event_log(
            5,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );

        assert_ne!(control.event_log_seq, with_extra_log.event_log_seq);
        assert_eq!(
            control.proc_roll_percent(&identity),
            with_extra_log.proc_roll_percent(&identity)
        );
    }

    #[test]
    fn proc_roll_percent_uses_occurrence_identity_not_success_count() {
        let core = new_test_core(empty_game_data(), 123);
        let first =
            test_proc_roll_identity("proc_roll_occurrence_test", Uuid::from_u128(0x9002), 0);
        let same_again =
            test_proc_roll_identity("proc_roll_occurrence_test", Uuid::from_u128(0x9002), 0);
        assert_eq!(
            core.proc_roll_percent(&first),
            core.proc_roll_percent(&same_again)
        );

        let first_roll = core.proc_roll_percent(&first);
        let distinct_occurrence = (1..64)
            .map(|index| {
                test_proc_roll_identity("proc_roll_occurrence_test", Uuid::from_u128(0x9002), index)
            })
            .find(|identity| core.proc_roll_percent(identity) != first_roll)
            .expect("occurrence identity should be able to produce an independent roll");

        assert_ne!(first, distinct_occurrence);
        assert_ne!(
            first_roll,
            core.proc_roll_percent(&distinct_occurrence),
            "different occurrence identity should not be collapsed to successful count"
        );
    }

    #[test]
    fn activation_commands_carry_trigger_proc_roll_identity() {
        let item_instance_id = Uuid::from_u128(0x5678);
        let caster_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC001));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC002));
        let occurrence_id = Uuid::from_u128(0xBEEF);
        let binding = SourcedAbilityActivation {
            source: CooldownSource::Item { item_instance_id },
            binding: AbilityActivationBinding {
                ability_id: SkillId::from("identity_proc"),
                activation: AbilityActivationDef::TriggerProc {
                    trigger: TriggerType::OnAttack,
                    proc_chance_percent: 75,
                    internal_cooldown_ms: 250,
                    max_triggers_per_battle: Some(3),
                },
            },
            binding_index: 4,
        };

        let commands = super::BattleCore::activation_commands_from_bindings(
            vec![binding],
            caster_id,
            super::TriggerAbilityContext {
                trigger_type: TriggerType::OnAttack,
                trigger_unit_id: caster_id,
                counterpart_unit_id: Some(target_id),
                target_id: Some(target_id),
                occurrence_id,
                occurrence_index: 9,
            },
        );

        assert_eq!(commands.len(), 1);
        let BattleCommand::TriggerAbility {
            skill_id,
            caster_id: command_caster_id,
            target_id: command_target_id,
            activation_source,
            binding_index,
            proc_roll_identity,
            proc_chance_percent,
            internal_cooldown_ms,
            max_triggers_per_battle,
            allow_dead_caster,
        } = &commands[0]
        else {
            panic!("expected TriggerAbility command");
        };

        assert_eq!(skill_id.as_str(), "identity_proc");
        assert_eq!(*command_caster_id, caster_id);
        assert_eq!(*command_target_id, Some(target_id));
        assert_eq!(
            *activation_source,
            CooldownSource::Item { item_instance_id }
        );
        assert_eq!(*binding_index, 4);
        assert_eq!(*proc_chance_percent, 75);
        assert_eq!(*internal_cooldown_ms, 250);
        assert_eq!(*max_triggers_per_battle, Some(3));
        assert!(!allow_dead_caster);

        assert_eq!(proc_roll_identity.trigger_type, TriggerType::OnAttack);
        assert_eq!(
            proc_roll_identity.activation_source,
            CooldownSource::Item { item_instance_id }
        );
        assert_eq!(proc_roll_identity.ability_id.as_str(), "identity_proc");
        assert_eq!(proc_roll_identity.binding_index, 4);
        assert_eq!(proc_roll_identity.caster_id, caster_id);
        assert_eq!(proc_roll_identity.trigger_unit_id, caster_id);
        assert_eq!(proc_roll_identity.counterpart_unit_id, Some(target_id));
        assert_eq!(proc_roll_identity.target_id, Some(target_id));
        assert_eq!(proc_roll_identity.occurrence_id, occurrence_id);
        assert_eq!(proc_roll_identity.occurrence_index, 9);
    }

    #[test]
    fn unit_world_position_sampling_interpolates_active_movement_segment() {
        let mut core = new_test_core(empty_game_data(), 1);
        let unit_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(43));
        let mut unit = test_runtime_unit(
            unit_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 1, 0, 1),
        );
        unit.body = UnitBody::new_at(WorldVec2::new(10.0, 0.0), 0.35, 1.0);
        core.units.insert(unit_id, unit);

        core.record_event_log(
            0,
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id: unit_id,
                start: WorldVec2::ZERO.quantized_milli(),
                target: WorldVec2::new(10.0, 0.0).quantized_milli(),
                started_at_ms: 0,
                ends_at_ms: 100,
                end_kind: MovementSegmentEndKind::Boundary,
            },
        );
        core.active_movement_segments.insert(
            unit_id,
            ActiveMovementSegment {
                start: WorldVec2::ZERO,
                target: WorldVec2::new(10.0, 0.0),
                started_at_ms: 0,
                ends_at_ms: 100,
            },
        );

        assert_eq!(
            core.sample_unit_world_position_at(unit_id, 25),
            Some(WorldVec2::new(2.5, 0.0))
        );
        assert_eq!(
            core.sample_unit_world_position_at(unit_id, 75),
            Some(WorldVec2::new(7.5, 0.0))
        );
    }

    #[test]
    fn advance_basic_attack_projectile_is_target_locked_not_path_collision() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(46));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(47));
        let bystander_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(48));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(10.0, 0.0), 0.10, 1.0);
        let mut bystander = test_runtime_unit(
            bystander_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        bystander.body = UnitBody::new_at(WorldVec2::new(5.0, 0.0), 0.10, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        core.units.insert(bystander_id, bystander);

        let projectile_id = Uuid::from_u128(0xBEEF);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(10.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 10,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(10, projectile_id);

        assert!(!core.projectiles.contains_key(&projectile_id));
        assert!(core
            .units
            .get(&target_id)
            .is_some_and(|target| target.stats.current_health < 100));
        assert!(core
            .units
            .get(&bystander_id)
            .is_some_and(|bystander| bystander.stats.current_health == 100));
    }

    #[test]
    fn advance_basic_attack_projectile_hits_moving_locked_target_by_sweep() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(61));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(62));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(5.0, 2.0), 0.15, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        core.active_movement_segments.insert(
            target_id,
            ActiveMovementSegment {
                start: WorldVec2::new(5.0, 2.0),
                target: WorldVec2::new(5.0, 0.0),
                started_at_ms: 0,
                ends_at_ms: 6,
            },
        );

        let projectile_id = Uuid::from_u128(0xCAFE);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(5.0, 2.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 6,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(6, projectile_id);

        assert!(!core.projectiles.contains_key(&projectile_id));
        assert!(core
            .units
            .get(&target_id)
            .is_some_and(|target| target.stats.current_health < 100));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id: id,
                target_instance_id,
                hit: true,
                ..
            } if *id == projectile_id && *target_instance_id == target_id
        )));
    }

    #[test]
    fn advance_basic_attack_projectile_misses_when_locked_target_dies_before_contact() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(63));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(64));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(5.0, 0.0), 0.15, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        core.apply_hp_delta_and_record(None, target_id, -999, 0, HpChangeReason::Command);

        let projectile_id = Uuid::from_u128(0xDEAD);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(5.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 5,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(5, projectile_id);

        assert!(!core.projectiles.contains_key(&projectile_id));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id: id,
                target_instance_id,
                hit: false,
                ..
            } if *id == projectile_id && *target_instance_id == target_id
        )));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source),
                target_instance_id,
                reason: HpChangeReason::BasicAttack,
                ..
            } if *source == attacker_id && *target_instance_id == target_id
        )));
    }

    #[test]
    fn advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x6401));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x6402));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(5.0, 0.0), 0.15, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);

        let projectile_id = Uuid::from_u128(0x6403);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Opponent,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(5.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 5,
                cause: BattleEventCause::default(),
            },
        );

        core.units.get_mut(&target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;

        core.advance_basic_attack_projectile(5, projectile_id);

        let target = core.units.get(&target_id).unwrap();
        assert_eq!(target.lifecycle, RuntimeUnitLifecycle::Withdrawn);
        assert_eq!(target.stats.current_health, 100);
        assert!(!core.projectiles.contains_key(&projectile_id));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id: id,
                target_instance_id,
                hit: false,
                ..
            } if *id == projectile_id && *target_instance_id == target_id
        )));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source),
                target_instance_id,
                reason: HpChangeReason::BasicAttack,
                ..
            } if *source == attacker_id && *target_instance_id == target_id
        )));
    }

    #[test]
    fn advance_basic_attack_projectile_does_not_recheck_tile_range_at_arrival() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(65));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(66));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.15, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));
        core.units.get_mut(&attacker_id).unwrap().body =
            UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        core.units.get_mut(&target_id).unwrap().body =
            UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.15, 1.0);

        let projectile_id = Uuid::from_u128(0xFACE);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(1.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 1,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(1, projectile_id);

        assert!(core
            .units
            .get(&target_id)
            .is_some_and(|target| target.stats.current_health < 100));
    }

    #[test]
    fn basic_attack_projectile_uses_launch_source_attack_after_stat_drift() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD11));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD12));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.10, 1.0);
        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);

        let projectile_id = Uuid::from_u128(0xD13);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(1.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 1,
                cause: BattleEventCause::default(),
            },
        );

        core.units.get_mut(&attacker_id).unwrap().stats.attack = 100;
        core.advance_basic_attack_projectile(1, projectile_id);

        assert_eq!(core.units.get(&target_id).unwrap().stats.current_health, 90);
    }

    #[test]
    fn basic_attack_projectile_uses_live_target_defense_at_impact() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD21));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD22));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 100, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.10, 1.0);
        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);

        let projectile_id = Uuid::from_u128(0xD23);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(1.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    100,
                    0,
                ),
                max_travel_ms: 1,
                cause: BattleEventCause::default(),
            },
        );

        core.units.get_mut(&target_id).unwrap().stats.defense = 100;
        core.advance_basic_attack_projectile(1, projectile_id);

        assert_eq!(core.units.get(&target_id).unwrap().stats.current_health, 50);
    }

    #[test]
    fn advance_basic_attack_projectile_uses_launch_side_after_attacker_death() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(67));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(68));

        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 10, 0, 1),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(100, 100, 0, 0, 1),
        );
        target.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.15, 1.0);

        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        core.apply_hp_delta_and_record(None, attacker_id, -999, 0, HpChangeReason::Command);

        let projectile_id = Uuid::from_u128(0xBADA);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::ZERO,
                current_position: WorldVec2::ZERO,
                aim: WorldVec2::new(1.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 1,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(1, projectile_id);

        assert!(core
            .units
            .get(&target_id)
            .is_some_and(|target| target.stats.current_health < 100));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id: id,
                target_instance_id,
                hit: true,
                ..
            } if *id == projectile_id && *target_instance_id == target_id
        )));
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
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
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let item = EquipmentMetadata {
            id: "item".to_string(),
            uuid: item_base_uuid,
            name: "Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: item_effects,
            ability_activations: item_activations,
            weapon_profile: Some(Default::default()),
        };

        GameDataBuilder::live_defaults()
            .with_abnormalities(vec![attacker, target])
            .with_equipment(vec![item])
            .with_skills(SkillDatabase::new(skills))
            .build_arc()
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
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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

        GameDataBuilder::empty()
            .with_abnormalities(vec![attacker, target])
            .with_artifacts(vec![artifact])
            .build_arc()
    }

    fn damage_proc_skill(id: &str, amount: i32) -> SkillDef {
        SkillDef {
            id: SkillId::from(id),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                range_policy:
                    crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles,
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![crate::game::ability::SkillEffectDef::Damage {
                    amount,
                    damage_type: crate::game::battle::damage::DamageType::Magic,
                }],
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
        let basic_attack = crate::game::data::abnormality_data::BasicAttackDef {
            defense_tile_range: Some(crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: false,
                rows: vec![
                    "XXXXX".to_string(),
                    "XXXXX".to_string(),
                    "XX@XX".to_string(),
                    "XXXXX".to_string(),
                    "XXXXX".to_string(),
                ],
            }),
            ..Default::default()
        };
        RuntimeUnit {
            instance_id,
            lifecycle: RuntimeUnitLifecycle::Active,
            spawn_order: u64::from(instance_id.as_bytes()[15]),
            source_owned_uuid: instance_id.as_uuid(),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            base_uuid,
            source_identity: crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                base_uuid,
            },
            stats,
            incoming_damage_modifiers: Default::default(),
            basic_attack,
            skill_id: None,
            skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
            body: Default::default(),
            tactical_anchor: None,
            enemy_movement_plan: None,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
            facing_direction: Some(crate::game::battle::tile_range::FacingDirection::Right),
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn write_debug_event_log_export(name: &str, event_log: &BattleEventLog) -> PathBuf {
        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug_event_log_exports");
        std::fs::create_dir_all(&out_dir).expect("create debug_event_log_exports directory");
        let out_path = out_dir.join(format!("{name}.json"));
        event_log
            .write_pretty_json(&out_path)
            .expect("write debug event log json");
        out_path
    }

    fn drain_event_queue(core: &mut super::BattleCore) {
        while let Some(event) = core.event_queue.pop() {
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).expect("process event");
        }
    }

    fn new_test_core(game_data: Arc<GameDataBase>, seed: u64) -> super::BattleCore {
        super::BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), game_data, seed)
    }

    #[test]
    fn apply_damage_command_records_feedback_tags_on_hp_changed() {
        let mut core = new_test_core(empty_game_data(), 1);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xF01));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xF02));

        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 10, 0, 1000),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                Uuid::nil(),
                UnitStats::with_values(200, 200, 1, 0, 1000),
            ),
        );

        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::Ability,
                    DamageType::True,
                    100,
                    50,
                ),
            }],
            50,
        );

        let feedback_tags = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    reason,
                    damage_type,
                    feedback_tags,
                    ..
                } if *source_instance_id == Some(attacker_id)
                    && *target_instance_id == target_id
                    && *reason == HpChangeReason::Command =>
                {
                    assert_eq!(*damage_type, Some(DamageType::True));
                    Some(feedback_tags.clone())
                }
                _ => None,
            })
            .expect("missing command damage HpChanged");

        assert_eq!(feedback_tags, vec![DamageFeedbackTag::FixedDamage]);
    }

    #[test]
    fn skill_projectile_impact_uses_launch_source_snapshot_after_caster_death() {
        let mut core = new_test_core(empty_game_data(), 1);
        let caster_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD31));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD32));
        let skill = damage_proc_skill("projectile_snapshot_test", 25);

        core.units.insert(
            caster_id,
            test_runtime_unit(
                caster_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        let launch_snapshot = core
            .damage_source_snapshot_template_for_unit(
                caster_id,
                DamageSource::Ability,
                DamageType::Magic,
                0,
                DamageModifiers::default(),
                0,
                10,
                false,
            )
            .expect("launch snapshot");
        core.units.remove(&caster_id);

        let (commands, _) = core.build_skill_step_commands(
            caster_id,
            &skill.steps[0],
            &[target_id],
            launch_snapshot.committed_at_ms,
            Some(&launch_snapshot),
        );
        core.process_commands(commands, 20);

        assert_eq!(core.units.get(&target_id).unwrap().stats.current_health, 75);
    }

    #[test]
    fn skill_projectile_impact_does_not_hit_withdrawn_target() {
        let caster_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD41));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xD42));
        let delivery_id = Uuid::from_u128(0xD43);
        let skill = damage_proc_skill("withdrawn_projectile_target_test", 25);
        let skill_id = skill.id.clone();
        let mut core = new_test_core(
            GameDataBuilder::empty()
                .with_skills(SkillDatabase::new(vec![skill]))
                .build_arc(),
            1,
        );

        core.units.insert(
            caster_id,
            test_runtime_unit(
                caster_id,
                Side::Opponent,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        core.units.get_mut(&target_id).unwrap().lifecycle = RuntimeUnitLifecycle::Withdrawn;

        let launch_snapshot = core
            .damage_source_snapshot_template_for_unit(
                caster_id,
                DamageSource::Ability,
                DamageType::Magic,
                0,
                DamageModifiers::default(),
                0,
                10,
                false,
            )
            .expect("launch snapshot");

        core.apply_skill_projectile_impact(
            20,
            delivery_id,
            1,
            0,
            skill_id.clone(),
            "hit".to_string(),
            caster_id,
            launch_snapshot,
            WorldVec2::new(1.0, 0.0),
            Some(target_id),
            None,
            true,
        );

        assert_eq!(
            core.units.get(&target_id).unwrap().stats.current_health,
            100
        );
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::SkillProjectileImpacted {
                delivery_id: id,
                skill_id: event_skill_id,
                first_hit_unit_id: None,
                terminal: true,
                ..
            } if *id == delivery_id && *event_skill_id == skill_id
        )));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::HpChanged {
                target_instance_id,
                ..
            } if *target_instance_id == target_id
        )));
    }

    fn place_test_unit(
        core: &mut super::BattleCore,
        unit_id: crate::game::battle::ids::UnitInstanceId,
        position: Position,
    ) {
        core.battlefield.ensure_walkable_tile(position).unwrap();
        if let Some(unit) = core.units.get_mut(&unit_id) {
            unit.set_world_position(WorldVec2::from_tile_center(position));
        }
    }

    #[test]
    fn advance_basic_attack_projectile_is_idempotent_for_same_projectile_id() {
        let mut core = new_test_core(empty_game_data(), 1);

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
                lifecycle: RuntimeUnitLifecycle::Active,
                spawn_order: 0,
                source_owned_uuid: attacker_id.as_uuid(),
                owner: Side::Player,
                role: crate::game::battle::types::BattleUnitRole::Combatant,
                threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
                base_uuid: Uuid::nil(),
                source_identity:
                    crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                        base_uuid: Uuid::nil(),
                    },
                stats: attacker_stats,
                incoming_damage_modifiers: Default::default(),
                basic_attack: Default::default(),
                skill_id: None,
                skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
                body: Default::default(),
                tactical_anchor: None,
                enemy_movement_plan: None,
                block_capacity: 0,
                block_radius_units: 0.0,
                blockable: true,
                mobility_kind: Default::default(),
                target_traits: Vec::new(),
                facing_direction: None,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                ranged_reposition_until_ms: 0,
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
                lifecycle: RuntimeUnitLifecycle::Active,
                spawn_order: 1,
                source_owned_uuid: target_id.as_uuid(),
                owner: Side::Opponent,
                role: crate::game::battle::types::BattleUnitRole::Combatant,
                threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
                base_uuid: Uuid::nil(),
                source_identity:
                    crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                        base_uuid: Uuid::nil(),
                    },
                stats: target_stats,
                incoming_damage_modifiers: Default::default(),
                basic_attack: Default::default(),
                skill_id: None,
                skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
                body: Default::default(),
                tactical_anchor: None,
                enemy_movement_plan: None,
                block_capacity: 0,
                block_radius_units: 0.0,
                blockable: true,
                mobility_kind: Default::default(),
                target_traits: Vec::new(),
                facing_direction: None,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                ranged_reposition_until_ms: 0,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );

        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));
        core.units.get_mut(&attacker_id).unwrap().body =
            UnitBody::new_at(WorldVec2::new(0.0, 0.0), 0.10, 1.0);
        core.units.get_mut(&target_id).unwrap().body =
            UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.10, 1.0);

        let projectile_id = Uuid::from_u128(0xAAAA);
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                fired_at_ms: 0,
                last_reevaluation_ms: 0,
                attacker_instance_id: attacker_id,
                attacker_owner_at_launch: Side::Player,
                air_capable_at_launch: false,
                target_instance_id: target_id,
                start: WorldVec2::new(0.0, 0.0),
                current_position: WorldVec2::new(0.0, 0.0),
                aim: WorldVec2::new(1.0, 0.0),
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                guidance: ProjectileGuidance::Homing,
                damage_type: DamageType::Physical,
                source_snapshot: test_damage_source_snapshot(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    10,
                    0,
                ),
                max_travel_ms: 1,
                cause: BattleEventCause::default(),
            },
        );

        core.advance_basic_attack_projectile(10, projectile_id);

        let hp_after_first = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        core.advance_basic_attack_projectile(10, projectile_id);

        let hp_after_second = core
            .units
            .get(&target_id)
            .map(|u| u.stats.current_health)
            .unwrap();

        assert_eq!(hp_after_first, hp_after_second);

        let hits = core
            .event_log
            .entries
            .iter()
            .filter(|entry| match &entry.event {
                crate::game::battle::event_log::BattleLogEvent::HpChanged {
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

        let hp_changed = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                crate::game::battle::event_log::BattleLogEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    reason,
                    damage_source,
                    damage_type,
                    raw_damage,
                    final_damage,
                    damage_breakdown,
                    critical,
                    feedback_tags,
                    ..
                } if *source_instance_id == Some(attacker_id)
                    && *target_instance_id == target_id
                    && *reason == HpChangeReason::BasicAttack =>
                {
                    Some((
                        *damage_source,
                        *damage_type,
                        *raw_damage,
                        *final_damage,
                        damage_breakdown.clone(),
                        *critical,
                        feedback_tags.clone(),
                    ))
                }
                _ => None,
            })
            .expect("missing basic attack HpChanged metadata");
        assert_eq!(hp_changed.0, Some(DamageSource::BasicAttack));
        assert_eq!(hp_changed.1, Some(DamageType::Physical));
        assert!(hp_changed.2.is_some());
        assert!(hp_changed.3.is_some());
        assert_eq!(hp_changed.5, Some(false));
        assert_eq!(hp_changed.6, Vec::<DamageFeedbackTag>::new());
        let breakdown = hp_changed.4.expect("missing damage breakdown");
        assert_eq!(breakdown[0].damage_type, DamageType::Physical);

        write_debug_event_log_export(
            "advance_basic_attack_projectile_is_idempotent_for_same_projectile_id",
            &core.event_log,
        );
    }

    #[test]
    fn apply_hp_delta_records_died_stop_with_latest_continuous_position() {
        let mut core = new_test_core(empty_game_data(), 1);

        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(3));
        let mut target_stats = UnitStats::with_values(100, 100, 0, 0, 1000);
        target_stats.move_speed_units_per_ms = 10;

        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                lifecycle: RuntimeUnitLifecycle::Active,
                spawn_order: 0,
                source_owned_uuid: target_id.as_uuid(),
                owner: Side::Opponent,
                role: crate::game::battle::types::BattleUnitRole::Combatant,
                threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
                base_uuid: Uuid::nil(),
                source_identity:
                    crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                        base_uuid: Uuid::nil(),
                    },
                stats: target_stats,
                incoming_damage_modifiers: Default::default(),
                basic_attack: Default::default(),
                skill_id: None,
                skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
                body: Default::default(),
                tactical_anchor: None,
                enemy_movement_plan: None,
                block_capacity: 0,
                block_radius_units: 0.0,
                blockable: true,
                mobility_kind: Default::default(),
                target_traits: Vec::new(),
                facing_direction: None,
                move_epoch: 0,
                action_state: ActionState::Idle,
                action_locks: Default::default(),
                current_target: None,
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                ranged_reposition_until_ms: 0,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        place_test_unit(&mut core, target_id, Position::new(0, 0));
        core.units.get_mut(&target_id).unwrap().set_world_position(
            crate::game::battle::core::movement::types::WorldVec2::new(0.05, 0.0),
        );

        core.apply_hp_delta_and_record(None, target_id, -999, 105, HpChangeReason::Command);

        let target = core
            .units
            .get(&target_id)
            .expect("target should remain in units map");
        assert_eq!(target.stats.current_health, 0);
        assert_eq!(target.body.position.x, 0.05);
        assert!(matches!(target.action_state, ActionState::Dead));
        assert_eq!(
            core.graveyard
                .get(&target_id)
                .map(|snapshot| snapshot.world_position),
            Some(crate::game::battle::core::movement::types::WorldVec2::new(
                0.05, 0.0
            ))
        );

        let stop_entry = core
            .event_log
            .entries
            .iter()
            .find(|entry| matches!(entry.event, BattleLogEvent::MovementStopped { .. }))
            .expect("missing MovementStopped");

        match &stop_entry.event {
            BattleLogEvent::MovementStopped {
                reason,
                world_position,
                stopped_at_ms,
                ..
            } => {
                assert_eq!(*reason, MovementStopReason::Died);
                assert_eq!(world_position.x_milli, 50);
                assert_eq!(world_position.y_milli, 0);
                assert_eq!(*stopped_at_ms, stop_entry.time_ms);
            }
            _ => unreachable!("expected MovementStopped"),
        }

        let death_entry = core
            .event_log
            .entries
            .iter()
            .find(|entry| matches!(entry.event, BattleLogEvent::UnitDied { .. }))
            .expect("missing UnitDied");

        match &death_entry.event {
            BattleLogEvent::UnitDied {
                unit_instance_id,
                world_position,
                position,
                ..
            } => {
                assert_eq!(*unit_instance_id, target_id);
                assert_eq!(world_position.x_milli, 50);
                assert_eq!(world_position.y_milli, 0);
                assert_eq!(*position, Position::new(0, 0));
            }
            _ => unreachable!("expected UnitDied"),
        }

        let hp_feedback_tags = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::HpChanged {
                    target_instance_id,
                    feedback_tags,
                    ..
                } if *target_instance_id == target_id => Some(feedback_tags.clone()),
                _ => None,
            })
            .expect("missing HpChanged");
        assert_eq!(hp_feedback_tags, Vec::<DamageFeedbackTag>::new());
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
            ability_id: SkillId::from("item_proc"),
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

        let mut core = new_test_core(game_data, 1);

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
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);
        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        assert_eq!(target_hp, 85);
        assert!(core.event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast { skill_id, caster_instance_id, target_instance_id }
                if skill_id.as_str() == "item_proc"
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

        let mut core = new_test_core(game_data, 1);

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
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);

        let poison = crate::game::battle::buffs::BuffId::from_name("poison");
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                ..
            } if *caster_instance_id == attacker_id
                && *target_instance_id == target_id
                && *buff_id == poison
        )));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::BuffApplied {
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

        let mut core = new_test_core(game_data, 1);

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
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, ally_id, Position::new(0, 1));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

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
            ability_id: SkillId::from("proc_icd"),
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

        let mut core = new_test_core(game_data, 7);

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
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        assert!(core.resolve_basic_attack(attacker_id, target_id, 10));
        drain_event_queue(&mut core);

        let target_hp = core.units.get(&target_id).unwrap().stats.current_health;
        assert_eq!(target_hp, 75);
        let proc_casts = core
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    &entry.event,
                    BattleLogEvent::AbilityCast { skill_id, .. } if skill_id.as_str() == "proc_icd"
                )
            })
            .count();
        assert_eq!(proc_casts, 1);
    }

    #[test]
    fn item_activation_proc_respects_max_triggers_per_battle() {
        let attacker_base_uuid = Uuid::from_u128(0x12);
        let target_base_uuid = Uuid::from_u128(0x22);
        let item_base_uuid = Uuid::from_u128(0x32);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(21));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(22));
        let item_instance_id = Uuid::from_u128(23);

        let activation = AbilityActivationBinding {
            ability_id: SkillId::from("proc_max_once"),
            activation: AbilityActivationDef::TriggerProc {
                trigger: TriggerType::OnAttack,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: Some(1),
            },
        };
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            target_base_uuid,
            item_base_uuid,
            HashMap::new(),
            vec![activation],
            vec![damage_proc_skill("proc_max_once", 5)],
        );

        let mut core = new_test_core(game_data, 8);

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
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        assert!(core.resolve_basic_attack(attacker_id, target_id, 10));
        drain_event_queue(&mut core);

        let proc_casts = core
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    &entry.event,
                    BattleLogEvent::AbilityCast { skill_id, .. } if skill_id.as_str() == "proc_max_once"
                )
            })
            .count();
        assert_eq!(proc_casts, 1);
    }
}
