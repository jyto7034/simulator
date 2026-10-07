use crate::authoring::control;
use crate::game::ability::{DeliveryDef, SkillHitTargetFilter};
use crate::game::battle::cooldown::{SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::core::BattleCore;
use crate::game::battle::damage::{
    BattleCommand, DamageModifiers, DamageSource, DamageSourceSnapshot, ProcRollIdentity,
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
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
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

#[derive(Debug)]
pub(in crate::game::battle::core) struct RuntimeCommandBatch {
    pub(in crate::game::battle::core) fact_source:
        crate::game::battle::runtime_facts::RuntimeFactSource,
    pub(in crate::game::battle::core) modifier_source_instance_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) commands: Vec<BattleCommand>,
}

impl RuntimeCommandBatch {
    pub(in crate::game::battle::core) fn unsourced(commands: Vec<BattleCommand>) -> Self {
        Self {
            fact_source: Default::default(),
            modifier_source_instance_id: None,
            commands,
        }
    }

    pub(in crate::game::battle::core) fn with_modifier_source(
        source_instance_id: UnitInstanceId,
        commands: Vec<BattleCommand>,
    ) -> Self {
        Self {
            fact_source: crate::game::battle::runtime_facts::RuntimeFactSource {
                actor: Some(source_instance_id),
                ..Default::default()
            },
            modifier_source_instance_id: Some(source_instance_id),
            commands,
        }
    }
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
                        equipment_source: sourced.equipment_source,
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
                Effect::BasicLifesteal { .. } => {}
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
        if self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|unit| unit.basic_attack.action.is_heal())
        {
            return self.resolve_basic_heal(attacker_instance_id, target_id, current_time_ms);
        }
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
        let target_owner = {
            let Some(target) = self.units.get(&target_id) else {
                return false;
            };
            if !target.is_active() {
                return false;
            }
            target.owner
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

        if let Some(unit) = self.units.get(&attacker_instance_id) {
            if control::charge_serialized(&unit.basic_attack, "basic_release_profile_copy").is_err()
            {
                return false;
            }
        }
        let area = self.units.get(&attacker_instance_id).and_then(|u| {
            if let DeliveryDef::TileArea { footprint, area } = &u.basic_attack.delivery {
                Some((footprint.clone(), area.clone(), u.basic_attack.air_capable))
            } else {
                None
            }
        });
        if let Some((footprint, area, air_capable)) = area {
            let anchor = match area.tile_origin {
                crate::game::ability::SkillTileAreaOrigin::Caster => {
                    self.live_unit_projected_tile(attacker_instance_id)
                }
                crate::game::ability::SkillTileAreaOrigin::Anchor => self.live_unit_projected_tile(
                    if area.anchor == crate::game::ability::SkillAreaAnchorSource::Caster {
                        attacker_instance_id
                    } else {
                        target_id
                    },
                ),
            };
            let Some(anchor) = anchor else {
                return false;
            };
            let Some(facing) = self.tile_area_projection_facing(attacker_instance_id, &footprint)
            else {
                return false;
            };
            let Some(tiles) = self.valid_tile_area_tiles(&footprint, anchor, facing) else {
                return false;
            };
            let targets = self.collect_tile_area_targets_at(
                current_time_ms,
                source_snapshot.source_side,
                attacker_instance_id,
                &tiles,
                SkillHitTargetFilter::Enemies,
                false,
            );
            self.grant_basic_attack_release_resonance(attacker_instance_id, current_time_ms);
            if self
                .consume_basic_overlay(attacker_instance_id, current_time_ms)
                .is_err()
            {
                return false;
            }
            let occurrence_id = source_snapshot.crit_roll_identity.source_instance_id;
            let mut actual_hp_loss = 0;
            for target in targets {
                if !self.single_target_can_target_unit(target, air_capable) {
                    continue;
                }
                let Ok(snapshot) =
                    crate::authoring::clone_shape::controlled_clone(&source_snapshot)
                else {
                    return false;
                };
                let Some(summary) = self.execute_committed_damage(CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: current_time_ms,
                    policy: CommittedDamagePolicy::BasicAttack {
                        occurrence_id,
                        occurrence_time_ms: current_time_ms,
                    },
                }) else {
                    return false;
                };
                if Self::add_basic_hp_loss(&mut actual_hp_loss, summary.actual_hp_loss).is_err() {
                    return false;
                }
            }
            if self
                .finish_basic_lifesteal(&source_snapshot, actual_hp_loss, current_time_ms)
                .is_err()
            {
                return false;
            }
            self.schedule_pending_autocasts(current_time_ms);
            return true;
        }
        match delivery {
            crate::game::battle::event_log::AttackDelivery::Instant => {
                self.grant_basic_attack_release_resonance(attacker_instance_id, current_time_ms);
                if self
                    .consume_basic_overlay(attacker_instance_id, current_time_ms)
                    .is_err()
                {
                    return false;
                }
                let occurrence_id = source_snapshot.crit_roll_identity.source_instance_id;
                let occurrence_time_ms = source_snapshot.committed_at_ms;
                let Ok(snapshot) =
                    crate::authoring::clone_shape::controlled_clone(&source_snapshot)
                else {
                    return false;
                };
                let Some(summary) = self.execute_committed_damage(CommittedDamageInput {
                    target_id,
                    source_snapshot: snapshot,
                    committed_at_ms: current_time_ms,
                    policy: CommittedDamagePolicy::BasicAttack {
                        occurrence_id,
                        occurrence_time_ms,
                    },
                }) else {
                    return false;
                };
                if self
                    .finish_basic_lifesteal(
                        &source_snapshot,
                        summary.actual_hp_loss,
                        current_time_ms,
                    )
                    .is_err()
                {
                    return false;
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
                    speed_units_per_ms,
                    hit_policy,
                    max_range_tiles,
                    max_lifetime_ms,
                    max_kills,
                    max_pierces,
                    collision,
                    ..
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
                let Some(mut launch_source_snapshot) = self.damage_source_snapshot_for_unit(
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
                launch_source_snapshot.fact_source = source_snapshot.fact_source;
                let spatial = (hit_policy
                    == crate::game::ability::ProjectileHitPolicy::DirectionalCollision)
                    .then_some(super::types::BasicSpatialProjectile {
                        actual_hp_loss: 0,
                        range_tiles: max_range_tiles.unwrap_or(1),
                        lifetime_ms: max_lifetime_ms.unwrap_or(1),
                        collision,
                        max_kills,
                        max_pierces,
                        kills: 0,
                        hit_unit_ids: Vec::new(),
                    });
                if self
                    .consume_basic_overlay(attacker_instance_id, current_time_ms)
                    .is_err()
                {
                    return false;
                }
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
                    guidance: if spatial.is_some() {
                        ProjectileGuidance::Fixed
                    } else {
                        ProjectileGuidance::Homing
                    },
                    spatial,
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
        let prior = self.authoring_direct_source.take();
        let result = self.process_commands_with_modifier_source(commands, current_time_ms, None);
        self.authoring_direct_source = prior;
        result
    }

    pub(in crate::game::battle::core) fn process_command_batches(
        &mut self,
        batches: Vec<RuntimeCommandBatch>,
        current_time_ms: u64,
    ) -> CommandExecutionSummary {
        let prior = self.authoring_direct_source.take();
        let mut total = CommandExecutionSummary::default();
        for mut batch in batches {
            if control::charge(1, "command_batch").is_err() {
                break;
            }
            // Replacing attribution for a synchronous trigger batch must keep
            // its causal resource authority. The surrounding finite delivery
            // remains live while this batch admits any child action.
            if batch.fact_source.resource_context().is_none() {
                batch.fact_source.resource_capabilities = match crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(
                    self.resource_context_for_source(&self.runtime_fact_source), batch.fact_source.resource_delivery().cloned(),
                ) { Ok(value) => value, Err(_) => break };
            }
            let prior_fact_source =
                std::mem::replace(&mut self.runtime_fact_source, batch.fact_source);
            let summary = self.process_commands_with_modifier_source(
                batch.commands,
                current_time_ms,
                batch.modifier_source_instance_id,
            );
            self.runtime_fact_source = prior_fact_source;
            total.actual_hp_loss = total.actual_hp_loss.saturating_add(summary.actual_hp_loss);
            total.actual_damage_target_count = total
                .actual_damage_target_count
                .saturating_add(summary.actual_damage_target_count);
            total.killed_target_count = total
                .killed_target_count
                .saturating_add(summary.killed_target_count);
        }
        self.authoring_direct_source = prior;
        total
    }

    #[cfg(test)]
    pub(in crate::game::battle::core) fn process_skill_commands(
        &mut self,
        commands: Vec<super::types::SkillCommand>,
        current_time_ms: u64,
        caster: UnitInstanceId,
        origin: Option<std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    ) -> CommandExecutionSummary {
        let prior_strike_scope = self.begin_strike_scope();
        let summary =
            self.process_skill_commands_in_scope(commands, current_time_ms, caster, origin);
        self.end_strike_scope(prior_strike_scope);
        summary
    }

    pub(in crate::game::battle::core) fn process_skill_commands_in_scope(
        &mut self,
        commands: Vec<super::types::SkillCommand>,
        current_time_ms: u64,
        caster: UnitInstanceId,
        origin: Option<std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    ) -> CommandExecutionSummary {
        let mut summary = CommandExecutionSummary::default();
        for (mut command, source) in commands {
            if control::charge(1, "skill_command").is_err() {
                break;
            }
            let prior = self.recording_skill_effect_source.take();
            let delivery = self.runtime_fact_source.delivery.clone();
            let action = self.runtime_fact_source.action.clone();
            let equipment = self.runtime_fact_source.equipment;
            let fragment = self.runtime_fact_source.fragment.clone();
            let resource_context = self.resource_context_for_source(&self.runtime_fact_source);
            let resource_delivery = self.runtime_fact_source.resource_delivery().cloned();
            let known_cast = origin
                .clone()
                .or_else(|| self.runtime_fact_source.cast.clone());
            let flow_occurrence = self.runtime_fact_source.flow.clone();
            let extended_observation = self.runtime_fact_source.extended_observation.clone();
            let reaction = self.runtime_fact_source.reaction;
            let prior_fact_source = std::mem::replace(
                &mut self.runtime_fact_source,
                crate::game::battle::runtime_facts::RuntimeFactSource {
                    extended_observation,
                    use_control: false,
                    flow: flow_occurrence,
                    reaction,
                    actor: Some(caster),
                    equipment,
                    fragment,
                    cast: known_cast,
                    authored: source.clone(),
                    action,
                    delivery,
                    allows_finite_children: true,
                    resource_capabilities: match crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(resource_context, resource_delivery) {
                        Ok(value) => value, Err(_) => { self.recording_skill_effect_source = prior; break; }
                    },
                },
            );
            if self
                .runtime_fact_source
                .admit_owner("skill_command_source")
                .is_err()
            {
                self.recording_skill_effect_source = prior;
                self.runtime_fact_source = prior_fact_source;
                break;
            }
            self.recording_skill_effect_source =
                source.zip(origin.as_ref()).map(|(authored, cast)| {
                    std::sync::Arc::new(
                        crate::game::battle::skill_source::SkillEffectExecutionSource {
                            authored,
                            cast: cast.clone(),
                        },
                    )
                });
            let prior_observation = self.authoring_direct_source.take();
            if let BattleCommand::ApplyDamage {
                source_snapshot, ..
            } = &mut command
            {
                let numeric_extension = source_snapshot
                    .fact_source
                    .relic_numeric()
                    .map(|_| source_snapshot.fact_source.extended_observation.clone());
                source_snapshot.fact_source = self.runtime_fact_source.clone();
                if let Some(extension) = numeric_extension {
                    source_snapshot.fact_source.extended_observation = extension;
                }
            }
            if self.authoring_observation.is_some() {
                if control::charge(1, "authoring_observation_source").is_err() {
                    self.recording_skill_effect_source = prior;
                    self.authoring_direct_source = prior_observation;
                    self.runtime_fact_source = prior_fact_source;
                    break;
                }
                self.authoring_direct_source = self.recording_skill_effect_source.clone();
            }
            #[cfg(test)]
            amount_command_observation::record(&command);
            let result = self.process_commands_with_modifier_source(
                std::iter::once(command),
                current_time_ms,
                Some(caster),
            );
            self.recording_skill_effect_source = prior;
            self.authoring_direct_source = prior_observation;
            self.runtime_fact_source = prior_fact_source;
            if control::poll("skill_command_result").is_err() {
                break;
            }
            summary.actual_hp_loss = summary.actual_hp_loss.saturating_add(result.actual_hp_loss);
            summary.actual_damage_target_count = summary
                .actual_damage_target_count
                .saturating_add(result.actual_damage_target_count);
            summary.killed_target_count = summary
                .killed_target_count
                .saturating_add(result.killed_target_count);
        }
        summary
    }

    pub(in crate::game::battle::core) fn process_commands_with_modifier_source(
        &mut self,
        commands: impl IntoIterator<Item = BattleCommand>,
        current_time_ms: u64,
        modifier_source_instance_id: Option<UnitInstanceId>,
    ) -> CommandExecutionSummary {
        let mut summary = CommandExecutionSummary::default();
        for command in commands {
            if control::charge(1, "battle_command").is_err() {
                break;
            }
            match command {
                BattleCommand::Resources {
                    operations,
                    targets,
                } => {
                    if self
                        .apply_typed_resource_operations(&operations, &targets, current_time_ms)
                        .is_err()
                    {
                        break;
                    }
                }
                BattleCommand::UnitDied { .. } => {
                    // Death is derived from HP reaching 0 and recorded in HpChanged handling.
                }
                BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    equipment_source,
                    binding_index,
                    proc_roll_identity,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster,
                } => {
                    if self.channel_active(caster_id) {
                        continue;
                    }
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
                            activation_source: activation_source.clone(),
                            binding_index,
                        },
                    );
                    let prior_proc = self.recording_skill_proc_origin.replace(
                        crate::game::battle::skill_source::SkillProcOrigin {
                            event_seq: proc_seq,
                            activation_source,
                            binding_index,
                        },
                    );
                    let resource_context =
                        self.resource_context_for_source(&self.runtime_fact_source);
                    let flow_occurrence = self.runtime_fact_source.flow.clone();
                    let prior_fact_source = std::mem::replace(
                        &mut self.runtime_fact_source,
                        crate::game::battle::runtime_facts::RuntimeFactSource {
                    flow: flow_occurrence,
                            actor: Some(caster_id),
                            equipment: equipment_source,
                            resource_capabilities: match crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(resource_context, None) {
                                Ok(value) => value, Err(_) => { self.recording_skill_proc_origin = prior_proc; break; }
                            },
                            ..Default::default()
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
                    self.recording_skill_proc_origin = prior_proc;
                    self.runtime_fact_source = prior_fact_source;
                }
                BattleCommand::Pull {
                    caster_id,
                    target_id,
                    max_distance_tiles,
                } => {
                    if self
                        .pull_enemy_toward_source(
                            caster_id,
                            target_id,
                            max_distance_tiles,
                            current_time_ms,
                        )
                        .is_err()
                    {
                        break;
                    }
                }
                BattleCommand::PayHealthCost {
                    caster_id,
                    amount,
                    allow_lethal,
                } => {
                    if self
                        .pay_health_cost(caster_id, amount, allow_lethal, current_time_ms)
                        .is_err()
                    {
                        break;
                    }
                }
                BattleCommand::RechargeShield {
                    caster_id,
                    source_skill,
                    amount,
                } => {
                    if self
                        .recharge_sustained_shield(
                            caster_id,
                            &source_skill,
                            amount,
                            current_time_ms,
                        )
                        .is_err()
                    {
                        break;
                    }
                }
                BattleCommand::BeginSustained {
                    caster_id,
                    target_id,
                    definition,
                } => {
                    if self
                        .begin_sustained(caster_id, target_id, definition, current_time_ms)
                        .is_err()
                    {
                        break;
                    }
                }
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier,
                } => {
                    let Some(target) = self.units.get(&target_id) else {
                        continue;
                    };
                    if !target.is_active() {
                        continue;
                    }

                    let before = target.stats;
                    let after = self.apply_permanent_with_sustained(target_id, before, modifier);
                    let prepared_death = if after.current_health == 0
                        && after.current_health != before.current_health
                    {
                        match self.prepare_death_fact(
                            modifier_source_instance_id,
                            target_id,
                            current_time_ms,
                        ) {
                            Ok(value) => value,
                            Err(_) => break,
                        }
                    } else {
                        None
                    };
                    let reservation = match self.reserve_authoring_observation() {
                        Ok(reservation) => reservation,
                        Err(_) => break,
                    };
                    let target = self
                        .units
                        .get_mut(&target_id)
                        .expect("checked modifier target");
                    target.stats = after;
                    target.body.move_speed = target.stats.move_speed_units_per_ms as f32 * 1_000.0
                        / DATA_UNITS_PER_WORLD;
                    let after = target.stats;
                    self.commit_authoring_observation(
                        reservation,
                        current_time_ms,
                        modifier_source_instance_id,
                        target_id,
                        super::authoring_observation::AppliedChange::Stats {
                            modifier,
                            before,
                            after,
                        },
                    );

                    self.record_event_log(
                        current_time_ms,
                        BattleLogEvent::StatChanged {
                            source_instance_id: modifier_source_instance_id,
                            target_instance_id: target_id,
                            modifier,
                            stats_before: before,
                            stats_after: after,
                        },
                    );

                    if self.record_runtime_stat_hp_change_and_finalize(
                        modifier_source_instance_id,
                        target_id,
                        before.current_health,
                        after.current_health,
                        current_time_ms,
                        prepared_death,
                    ) {
                        summary.killed_target_count = summary.killed_target_count.saturating_add(1);
                    }
                }
                BattleCommand::ApplyDamage {
                    target_id,
                    source_snapshot,
                } => {
                    if let Some(execution) = self.execute_committed_damage(CommittedDamageInput {
                        target_id,
                        source_snapshot,
                        committed_at_ms: current_time_ms,
                        policy: CommittedDamagePolicy::GeneralCommand,
                    }) {
                        summary.actual_hp_loss = summary
                            .actual_hp_loss
                            .saturating_add(execution.actual_hp_loss);
                        summary.actual_damage_target_count = summary
                            .actual_damage_target_count
                            .saturating_add(execution.actual_damage_target_count);
                        summary.killed_target_count = summary
                            .killed_target_count
                            .saturating_add(execution.killed_target_count);
                    }
                }
                BattleCommand::InterruptCast {
                    source_id,
                    target_id,
                } => {
                    self.cancel_resonance_use_control(target_id);
                    if self
                        .stop_channel_with_reason(
                            target_id,
                            current_time_ms,
                            crate::game::ability::sustained::SustainedEndReason::Interrupted,
                        )
                        .is_err()
                    {
                        break;
                    }
                    self.cancel_structured_flows(
                        target_id,
                        current_time_ms,
                        crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
                    );
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

                    self.cancel_pending_action(
                        pending.action.as_ref(),
                        current_time_ms,
                        crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
                    );

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
                        summary.actual_hp_loss = summary
                            .actual_hp_loss
                            .saturating_add(u64::from(hp_before - hp_after));
                        summary.actual_damage_target_count =
                            summary.actual_damage_target_count.saturating_add(1);
                    }
                }
                BattleCommand::ModifyResonance {
                    target_id,
                    amount,
                    allow_autocast_when_full,
                } => {
                    // Observe the command's actual mutation, not passive gain or
                    // cast reset. Ordinary gameplay takes the existing path.
                    let observation = if self.authoring_observation.is_some() {
                        let Some(target) = self.units.get(&target_id) else {
                            continue;
                        };
                        let before = target.resonance_current;
                        let reservation = match self.reserve_authoring_observation() {
                            Ok(reservation) => reservation,
                            Err(_) => break,
                        };
                        Some((before, reservation))
                    } else {
                        None
                    };
                    self.modify_resonance(
                        target_id,
                        amount,
                        current_time_ms,
                        allow_autocast_when_full,
                    );
                    if let Some((before, reservation)) = observation {
                        let target = self
                            .units
                            .get(&target_id)
                            .expect("retained resonance target");
                        let after = target.resonance_current;
                        let max = target.resonance_max.max(1);
                        if after != before {
                            self.commit_authoring_observation(
                                reservation,
                                current_time_ms,
                                modifier_source_instance_id,
                                target_id,
                                super::authoring_observation::AppliedChange::Resonance {
                                    before,
                                    after,
                                    max,
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
                    let Ok(mut finite_source) =
                        self.reserve_finite_source(self.runtime_fact_source.clone())
                    else {
                        break;
                    };
                    finite_source.use_control = match self.reserve_use_attack(&finite_source) {
                        Ok(owned) => owned,
                        Err(_) => break,
                    };
                    self.event_queue.push_with_context(
                        BattleEvent::AttackStart {
                            time_ms: current_time_ms.saturating_add(time_ms),
                            attacker_instance_id: attacker_id,
                            target_instance_id: target_id,
                            schedule_next: false,
                            cause: self.recording_cause().unwrap_or_default(),
                        },
                        None,
                        finite_source,
                    );
                }
                BattleCommand::ApplyBuff {
                    caster_id,
                    target_id,
                    buff_id,
                    duration_ms,
                } => {
                    let source = if self.authoring_observation.is_some() {
                        if control::charge(1, "authoring_buff_source").is_err() {
                            break;
                        }
                        self.authoring_direct_source.clone()
                    } else {
                        None
                    };
                    if self
                        .enqueue_finite_effect(
                            BattleEvent::ApplyBuff {
                                time_ms: current_time_ms,
                                caster_instance_id: caster_id,
                                target_instance_id: target_id,
                                buff_id,
                                duration_ms,
                                cause: self.recording_cause().unwrap_or_default(),
                            },
                            source,
                            self.runtime_fact_source.clone(),
                        )
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
        summary
    }
}

/// Test-only tap at the actual skill command consumer; never changes the command.
#[cfg(test)]
pub(super) mod amount_command_observation {
    use super::*;
    use crate::game::battle::core::value_expr_runtime::RuntimeAmount;
    use std::cell::RefCell;

    thread_local! {
        static INPUTS: RefCell<Option<Vec<(UnitInstanceId, RuntimeAmount)>>> = const { RefCell::new(None) };
    }

    pub(super) fn record(command: &BattleCommand) {
        INPUTS.with(|slot| {
            let mut slot = slot.borrow_mut();
            let Some(inputs) = slot.as_mut() else { return };
            match command {
                BattleCommand::BeginSustained {
                    target_id,
                    definition:
                        crate::game::ability::sustained::SustainedEffectDef::Shield { amount, .. },
                    ..
                } => {
                    inputs.push((*target_id, RuntimeAmount::Shield(*amount)));
                }
                BattleCommand::ApplyDamage {
                    target_id,
                    source_snapshot,
                } => {
                    inputs.push((
                        *target_id,
                        RuntimeAmount::Damage(source_snapshot.base_damage),
                    ));
                }
                BattleCommand::ApplyHeal {
                    target_id,
                    flat,
                    percent,
                    ..
                } => {
                    assert_eq!(*percent, 0, "authored amount heal has no percent term");
                    inputs.push((*target_id, RuntimeAmount::Heal(*flat)));
                }
                _ => {}
            }
        });
    }

    pub(in crate::game::battle::core) fn observe<T>(
        operation: impl FnOnce() -> T,
    ) -> (T, Vec<(UnitInstanceId, RuntimeAmount)>) {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                INPUTS.with(|slot| *slot.borrow_mut() = None);
            }
        }
        INPUTS.with(|slot| {
            assert!(slot.borrow().is_none(), "nested amount command observation");
            *slot.borrow_mut() = Some(Vec::new());
        });
        let _reset = Reset;
        let result = operation();
        let inputs = INPUTS.with(|slot| slot.borrow_mut().take().unwrap());
        (result, inputs)
    }
}

#[cfg(test)]
mod tests {
    use crate::game::ability::{
        AbilityActivationBinding, AbilityActivationDef, DeliveryDef, SkillCastTargetingDef,
        SkillDef, SkillId, SkillProjectileCollisionDef, SkillStepDef, SkillTarget,
        StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::cooldown::{CooldownSource, SourcedAbilityActivation};
    use crate::game::battle::core::movement::{
        types::{ActiveMovementSegment, UnitBody, WorldVec2, DATA_UNITS_PER_WORLD},
        ActionState, MovementSegmentEndKind,
    };
    use crate::game::battle::core::projectile_math::projectile_flight_ms;
    use crate::game::battle::core::types::{
        ActiveProjectileRuntime, ProjectileGuidance, RuntimeUnit, RuntimeUnitLifecycle,
    };
    use crate::game::battle::core::ProjectileRecord;
    use crate::game::battle::damage::{
        BattleCommand, CombatRollIdentity, CombatRollKind, DamageBreakdown, DamageFeedbackTag,
        DamageModifiers, DamageSource, DamageSourceSnapshot, DamageType, ProcRollIdentity,
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
    use crate::game::stats::{
        StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggerType, TriggeredEffect,
        UnitStats,
    };
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
            fact_source: Default::default(),
            skill_cast_source: None,
            value_expr_capture: None,
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
            on_attack_modifier_sources: Vec::new(),
            basic_lifesteal: Vec::new(),
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
            spatial: None,
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
                attack_epoch: 0,
                source_snapshot: player_snapshot,
                kind: AttackKind::Auto,
                delivery: AttackDelivery::Instant,
                cause: BattleEventCause::default(),
            },
            BattleEvent::AttackResolve {
                time_ms: 10,
                attacker_instance_id: opponent_id,
                target_instance_id: player_id,
                attack_epoch: 0,
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
            equipment_source: None,
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
            equipment_source,
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
        assert_eq!(*equipment_source, None);
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
    fn executed_proc_effect_retains_item_cast_and_authored_origin() {
        let skill = damage_proc_skill("source_proc", 5);
        let mut core = new_test_core(
            GameDataBuilder::empty()
                .with_skills(SkillDatabase::new(vec![skill]))
                .build_arc(),
            42,
        );
        let caster_id = Uuid::from_u128(0xCAE11).into();
        let target_id = Uuid::from_u128(0xCAE12).into();
        let item_instance_id = Uuid::from_u128(0xCAE13);
        core.units.insert(
            caster_id,
            test_runtime_unit(
                caster_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                Uuid::nil(),
                UnitStats::with_values(100, 100, 1, 0, 1),
            ),
        );
        let commands = super::BattleCore::activation_commands_from_bindings(
            vec![SourcedAbilityActivation {
                equipment_source: None,
                source: CooldownSource::Item { item_instance_id },
                binding: AbilityActivationBinding {
                    ability_id: "source_proc".into(),
                    activation: AbilityActivationDef::TriggerProc {
                        trigger: TriggerType::OnAttack,
                        proc_chance_percent: 100,
                        internal_cooldown_ms: 0,
                        max_triggers_per_battle: None,
                    },
                },
                binding_index: 4,
            }],
            caster_id,
            super::TriggerAbilityContext {
                trigger_type: TriggerType::OnAttack,
                trigger_unit_id: caster_id,
                counterpart_unit_id: Some(target_id),
                target_id: Some(target_id),
                occurrence_id: Uuid::from_u128(0xCAE14),
                occurrence_index: 0,
            },
        );
        core.process_commands(commands, 0);
        let step = core.event_queue.pop().expect("proc queues its skill step");
        let time_ms = step.time_ms();
        core.process_event(step, time_ms).unwrap();
        let source = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| {
                matches!(entry.event, BattleLogEvent::HpChanged { .. })
                    .then_some(entry.skill_effect_source.as_ref())
                    .flatten()
            })
            .expect("executed proc damage has an authored source");
        assert_eq!(
            source.authored.site.action.definition_id.as_str(),
            "source_proc"
        );
        assert_eq!(source.cast.caster_instance_id, caster_id);
        let proc_origin = source.cast.proc_origin.as_ref().unwrap();
        assert_eq!(
            proc_origin.activation_source,
            CooldownSource::Item { item_instance_id }
        );
        assert_eq!(proc_origin.binding_index, 4);
        assert_eq!(
            source.cast.cause,
            BattleEventCause::Parent {
                seq: proc_origin.event_seq
            }
        );
        assert!(core.recording_skill_effect_source.is_none());
        assert!(core.recording_skill_proc_origin.is_none());
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
                spatial: None,
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
                spatial: None,
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
    fn basic_and_homing_skill_projectiles_share_observable_motion_characterization_table() {
        #[derive(Clone, Copy)]
        struct MotionRow {
            name: &'static str,
            start: WorldVec2,
            target_start: WorldVec2,
            target_end: WorldVec2,
            target_radius: f32,
            duration_ms: u64,
            speed_units_per_ms: u32,
            expected_impact_time_ms: u64,
            expected_impact_position: WorldVec2,
        }

        let rows = [
            MotionRow {
                name: "zero-distance",
                start: WorldVec2::new(1.0, 1.0),
                target_start: WorldVec2::new(1.0, 1.0),
                target_end: WorldVec2::new(1.0, 1.0),
                target_radius: 0.0,
                duration_ms: 0,
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                expected_impact_time_ms: 0,
                expected_impact_position: WorldVec2::new(1.0, 1.0),
            },
            MotionRow {
                name: "sub-millisecond-ceil",
                start: WorldVec2::ZERO,
                target_start: WorldVec2::new(0.75, 0.0),
                target_end: WorldVec2::new(0.75, 0.0),
                target_radius: 0.0,
                duration_ms: 1,
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                expected_impact_time_ms: 1,
                expected_impact_position: WorldVec2::new(0.75, 0.0),
            },
            MotionRow {
                name: "exact-millisecond",
                start: WorldVec2::ZERO,
                target_start: WorldVec2::new(2.0, 0.0),
                target_end: WorldVec2::new(2.0, 0.0),
                target_radius: 0.0,
                duration_ms: 2,
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                expected_impact_time_ms: 2,
                expected_impact_position: WorldVec2::new(2.0, 0.0),
            },
            MotionRow {
                name: "moving-target-continuous-sweep",
                start: WorldVec2::ZERO,
                target_start: WorldVec2::new(5.0, 2.0),
                target_end: WorldVec2::new(5.0, 0.0),
                target_radius: 0.15,
                duration_ms: 6,
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                expected_impact_time_ms: 6,
                expected_impact_position: WorldVec2::new(4.861, 0.0),
            },
        ];

        for (row_index, row) in rows.into_iter().enumerate() {
            let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                0xC050_0000 + row_index as u128 * 0x10 + 1,
            ));
            let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                0xC050_0000 + row_index as u128 * 0x10 + 2,
            ));
            let projectile_id = Uuid::from_u128(0xC050_1000 + row_index as u128);
            let delivery_id = Uuid::from_u128(0xC050_2000 + row_index as u128);

            let make_core = || {
                let mut core = new_test_core(empty_game_data(), 0xC05);
                let mut attacker = test_runtime_unit(
                    attacker_id,
                    Side::Player,
                    Uuid::nil(),
                    UnitStats::with_values(100, 100, 10, 0, 1),
                );
                attacker.body = UnitBody::new_at(row.start, 0.0, 1.0);
                let mut target = test_runtime_unit(
                    target_id,
                    Side::Opponent,
                    Uuid::nil(),
                    UnitStats::with_values(100, 100, 0, 0, 1),
                );
                target.body = UnitBody::new_at(row.target_end, row.target_radius, 1.0);
                core.units.insert(attacker_id, attacker);
                core.units.insert(target_id, target);
                if row.target_start != row.target_end {
                    core.active_movement_segments.insert(
                        target_id,
                        ActiveMovementSegment {
                            start: row.target_start,
                            target: row.target_end,
                            started_at_ms: 0,
                            ends_at_ms: row.duration_ms,
                        },
                    );
                }
                core
            };

            let basic_snapshot = test_damage_source_snapshot(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                10,
                0,
            );
            let mut basic = make_core();
            basic.projectiles.insert(
                projectile_id,
                ProjectileRecord {
                    spatial: None,
                    fired_at_ms: 0,
                    last_reevaluation_ms: 0,
                    attacker_instance_id: attacker_id,
                    attacker_owner_at_launch: Side::Player,
                    air_capable_at_launch: false,
                    target_instance_id: target_id,
                    start: row.start,
                    current_position: row.start,
                    aim: row.target_start,
                    speed_units_per_ms: row.speed_units_per_ms,
                    guidance: ProjectileGuidance::Homing,
                    damage_type: DamageType::Physical,
                    source_snapshot: basic_snapshot,
                    max_travel_ms: row.duration_ms,
                    cause: BattleEventCause::default(),
                },
            );
            basic.advance_basic_attack_projectile(row.duration_ms, projectile_id);
            let (basic_impact_time_ms, basic_impact_position) = basic
                .event_log
                .entries
                .iter()
                .find_map(|entry| match &entry.event {
                    BattleLogEvent::BasicAttackProjectileImpacted {
                        projectile_id: id,
                        impact_position,
                        hit: true,
                        ..
                    } if *id == projectile_id => Some((entry.time_ms, *impact_position)),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{}: basic impact", row.name));

            let mut skill = make_core();
            skill.active_projectiles.insert(
                delivery_id,
                ActiveProjectileRuntime {
                    delivery_id,
                    cast_seq: 1,
                    step_index: 0,
                    skill_id: SkillId::from("motion_characterization"),
                    step_id: "projectile".to_string(),
                    caster_instance_id: attacker_id,
                    caster_owner: Side::Player,
                    spawned_at_ms: 0,
                    start: row.start,
                    current_position: row.start,
                    aim: row.target_start,
                    speed_units_per_ms: row.speed_units_per_ms,
                    guidance: ProjectileGuidance::Homing,
                    target_unit_id: Some(target_id),
                    source_snapshot: test_damage_source_snapshot(
                        attacker_id,
                        target_id,
                        DamageSource::Ability,
                        DamageType::Magic,
                        10,
                        0,
                    ),
                    collision: SkillProjectileCollisionDef {
                        radius_units: 0,
                        ..Default::default()
                    },
                    hit_unit_ids: Vec::new(),
                    killed_unit_count: 0,
                    max_kills: None,
                    impact_vfx_id: None,
                    last_reevaluation_ms: None,
                    next_reevaluation_ms: 0,
                    max_travel_ms: row.duration_ms,
                    cause: BattleEventCause::default(),
                },
            );
            skill
                .advance_skill_projectile(row.duration_ms, delivery_id)
                .unwrap();
            let skill_event = skill
                .event_queue
                .pop()
                .unwrap_or_else(|| panic!("{}: skill impact", row.name));
            let (skill_impact_time_ms, skill_impact_position) =
                match skill_event.event.view().into_owned() {
                    BattleEvent::SkillProjectileImpact {
                        time_ms,
                        delivery_id: id,
                        first_hit_unit_id: Some(hit_unit_id),
                        impact_position,
                        terminal: true,
                        ..
                    } if id == delivery_id && hit_unit_id == target_id => {
                        (time_ms, impact_position)
                    }
                    other => panic!("{}: unexpected skill event: {other:?}", row.name),
                };

            assert_eq!(
                basic_impact_time_ms, row.expected_impact_time_ms,
                "{}: basic time",
                row.name
            );
            assert_eq!(
                skill_impact_time_ms, row.expected_impact_time_ms,
                "{}: skill time",
                row.name
            );
            assert_eq!(
                basic_impact_position,
                row.expected_impact_position.quantized_milli(),
                "{}: expected basic position",
                row.name
            );
            assert_eq!(
                skill_impact_position.quantized_milli(),
                basic_impact_position,
                "{}: adapter position parity",
                row.name
            );
        }
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
                spatial: None,
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
                spatial: None,
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
                spatial: None,
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
                spatial: None,
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
                spatial: None,
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
                spatial: None,
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
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };

        GameDataBuilder::live_defaults()
            .with_abnormalities(vec![attacker, target])
            .with_equipment(vec![item])
            .with_skills(SkillDatabase::new(skills))
            .build_arc()
    }

    fn damage_proc_skill(id: &str, amount: i32) -> SkillDef {
        SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from(id),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                crate::game::ability::SkillTargetReach::Global,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
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
                }
                .authored("effect_01")],
                presentation: Default::default(),
            }],
        }
    }

    fn self_heal_proc_skill(id: &str) -> SkillDef {
        SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from(id),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::SelfUnit,
                crate::game::ability::SkillTargetReach::Pattern,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "heal".to_string(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![
                    crate::game::ability::SkillEffectDef::Heal { amount: 1 }.authored("effect_01")
                ],
                presentation: Default::default(),
            }],
        }
    }

    fn stat_modifier_skill(id: &str, modifier: StatModifier) -> SkillDef {
        SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from(id),
            name: id.to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                crate::game::ability::SkillTargetReach::Global,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "modify".to_string(),
                delay_ms: 0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![
                    crate::game::ability::SkillEffectDef::ModifyStats { modifier }
                        .authored("effect_01"),
                ],
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
            employee_trauma: None,
            corroded: false,
            fragment_source: None,
            basic_attack_speed_percent: 100,
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
            equipment_slots: None,
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
            basic_attack_epoch: 0,
            basic_attack_windup_until_ms: None,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            resonance_use: None,
            next_resonance_pulse_ms: None,
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
        let file = std::fs::File::create(&out_path).expect("create debug event log json");
        serde_json::to_writer_pretty(std::io::BufWriter::new(file), event_log)
            .expect("write debug event log json");
        out_path
    }

    fn drain_event_queue(core: &mut super::BattleCore) {
        while let Some(event) = core.event_queue.pop() {
            // Command tests drain finite ability causality, not the self-rescheduling movement clock.
            if matches!(
                event.event.view().into_owned(),
                BattleEvent::ContinuousMovementTick { .. }
            ) {
                continue;
            }
            let time_ms = event.time_ms();
            core.process_event(event, time_ms).expect("process event");
        }
    }

    fn new_test_core(game_data: Arc<GameDataBase>, seed: u64) -> super::BattleCore {
        super::BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), game_data, seed)
    }

    fn authoring_control_core(
        skill: SkillDef,
    ) -> (
        super::BattleCore,
        crate::game::battle::ids::UnitInstanceId,
        crate::game::battle::ids::UnitInstanceId,
    ) {
        let mut core = new_test_core(
            GameDataBuilder::empty()
                .with_skills(SkillDatabase::new(vec![skill]))
                .build_arc(),
            0xCA731,
        );
        let caster = Uuid::from_u128(0xCA732).into();
        let target = Uuid::from_u128(0xCA733).into();
        for (id, side, position) in [
            (caster, Side::Player, Position::new(0, 0)),
            (target, Side::Opponent, Position::new(1, 0)),
        ] {
            core.units.insert(
                id,
                test_runtime_unit(
                    id,
                    side,
                    Uuid::nil(),
                    UnitStats::with_values(1000, 1000, 1, 0, 1000),
                ),
            );
            place_test_unit(&mut core, id, position);
        }
        (core, caster, target)
    }

    fn authoring_buff_core() -> (
        super::BattleCore,
        super::UnitInstanceId,
        super::UnitInstanceId,
        SkillId,
    ) {
        let mut skill = damage_proc_skill("observed_buff", 1);
        skill.steps[0].delay_ms = 50;
        skill.steps[0].effects = vec![crate::game::ability::SkillEffectDef::ApplyBuff {
            buff_id: "silence".into(),
            duration_ms: 700,
        }
        .authored("buff_effect")];
        let skill_id = skill.id.clone();
        let data = GameDataBuilder::live_defaults()
            .with_skills(SkillDatabase::new(vec![skill]))
            .build_arc();
        let mut core = new_test_core(data, 0xB001);
        let caster = Uuid::from_u128(0xB002).into();
        let target = Uuid::from_u128(0xB003).into();
        for (id, side, position) in [
            (caster, Side::Player, Position::new(0, 0)),
            (target, Side::Opponent, Position::new(1, 0)),
        ] {
            core.units.insert(
                id,
                test_runtime_unit(
                    id,
                    side,
                    Uuid::nil(),
                    UnitStats::with_values(100, 100, 1, 0, 1000),
                ),
            );
            place_test_unit(&mut core, id, position);
        }
        (core, caster, target, skill_id)
    }

    #[test]
    fn authoring_buff_actual_delayed_apply_keeps_source_clone_and_optional_trace_isolated() {
        use crate::authoring::{
            clone_shape::controlled_clone,
            control,
            trace::{TraceAttempt, TraceLimits, TraceOptions},
            CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        use crate::game::battle::{
            buffs::BuffId,
            core::authoring_observation::{AppliedChange, BuffApplicationState},
        };
        let (base, caster, target, skill_id) = authoring_buff_core();
        let mut gameplay = Vec::new();
        let mut mandatory = Vec::new();
        for (enabled, limits) in [
            (false, None),
            (true, None),
            (true, Some(TraceLimits::default())),
            (
                true,
                Some(TraceLimits {
                    max_bytes: 1,
                    max_row_bytes: 1,
                    ..Default::default()
                }),
            ),
        ] {
            let mut core = base.clone();
            let options = TraceOptions { limits };
            if enabled {
                core.enable_authoring_observation_with_trace(options);
            }
            let _trace = TraceAttempt::enter(options).unwrap();
            let (result, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    core.invoke_ability(
                        10,
                        caster,
                        &skill_id,
                        Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                            unit_instance_id: target,
                        }),
                        BattleEventCause::default(),
                        false,
                    );
                    let step = core.event_queue.pop().unwrap();
                    assert!(matches!(
                        step.event.view().into_owned(),
                        BattleEvent::SkillStep { time_ms: 60, .. }
                    ));
                    core.process_event(step, 60)?;
                    assert!(core.buffs.is_empty(), "enqueue is not an application");
                    if enabled {
                        assert_eq!(core.authoring_observation()?.unwrap().applications, 0);
                    }
                    let mut cloned = controlled_clone(&core)?;
                    let queued = core.event_queue.pop().unwrap();
                    assert!(matches!(
                        queued.event.view().into_owned(),
                        BattleEvent::ApplyBuff { time_ms: 60, .. }
                    ));
                    assert_eq!(queued.source.is_some(), enabled);
                    let weak = queued.source.as_ref().map(Arc::downgrade);
                    core.process_event(queued, 60)?;
                    let queued_clone = cloned.event_queue.pop().unwrap();
                    cloned.process_event(queued_clone, 60)?;
                    assert_eq!(
                        serde_json::to_value(core.authoring_observation()?).unwrap(),
                        serde_json::to_value(cloned.authoring_observation()?).unwrap()
                    );
                    let key = super::super::types::BuffInstanceKey {
                        caster_instance_id: caster,
                        target_instance_id: target,
                        buff_id: BuffId::from_name("silence"),
                    };
                    assert_eq!(core.buffs[&key].expires_at_ms, 760);
                    if enabled {
                        let observed = core.authoring_observation()?.unwrap();
                        assert_eq!(observed.applications, 1);
                        assert_eq!(observed.sourced_applications, 1);
                        assert_eq!(observed.nonzero_hp_applications, 0);
                        assert!(observed.latest_nonzero_sourced_hp.is_none());
                        let witness = observed.latest.as_ref().unwrap();
                        assert_eq!(witness.time_ms, 60);
                        assert!(matches!(
                            witness.change,
                            AppliedChange::Buff {
                                before: None,
                                after: BuffApplicationState {
                                    stacks: 1,
                                    expires_at_ms: 760,
                                    next_tick_ms: None
                                },
                                ..
                            }
                        ));
                        let source = witness.source.as_ref().unwrap();
                        assert_eq!(source.cast.skill_id, skill_id);
                        assert_eq!(source.authored.site.effect_id.as_str(), "buff_effect");
                        // Unrelated ambient observation context cannot source a raw event.
                        core.authoring_direct_source = Some(source.clone());
                    }
                    let apply = |time_ms, duration_ms, buff_id| BattleEvent::ApplyBuff {
                        time_ms,
                        caster_instance_id: caster,
                        target_instance_id: target,
                        buff_id,
                        duration_ms,
                        cause: BattleEventCause::default(),
                    };
                    // Same-state accepted reapply still performs the existing log/expiry work.
                    core.process_event(apply(60, 700, key.buff_id), 60)?;
                    core.authoring_direct_source = None;
                    core.process_event(apply(70, 900, key.buff_id), 70)?;
                    assert_eq!(core.buffs[&key].expires_at_ms, 970);
                    assert_eq!(cloned.buffs[&key].expires_at_ms, 760);
                    core.process_event(apply(80, 0, key.buff_id), 80)?;
                    core.process_event(apply(80, 1, BuffId::from_name("not_a_buff")), 80)?;
                    core.units.get_mut(&target).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;
                    core.process_event(apply(80, 100, key.buff_id), 80)?;
                    if enabled {
                        let observed = core.authoring_observation()?.unwrap();
                        assert_eq!(observed.applications, 3);
                        assert_eq!(observed.sourced_applications, 1);
                        assert!(observed.latest.as_ref().unwrap().source.is_none());
                        assert_eq!(observed.latest.as_ref().unwrap().change.hp_delta(), 0);
                    }
                    Ok::<_, crate::game::behavior::GameError>((
                        weak,
                        BuffApplicationState::from(&core.buffs[&key]),
                    ))
                },
            )
            .unwrap();
            let (weak, buff) = result.unwrap();
            let view = core.authoring_trace.view();
            if let Some(limits) = limits {
                if limits.max_bytes == 1 {
                    assert!(view.ndjson.is_empty());
                    assert_eq!(view.gap.unwrap().dropped_rows, 3);
                } else {
                    let rows: Vec<serde_json::Value> =
                        serde_json::Deserializer::from_slice(view.ndjson)
                            .into_iter()
                            .map(Result::unwrap)
                            .collect();
                    assert_eq!(rows.len(), 3);
                    assert_eq!(rows[0]["change"]["channel"], "buff");
                    assert_eq!(rows[0]["change"]["after"]["expires_at_ms"], 760);
                    assert_eq!(rows[1]["change"]["before"], rows[1]["change"]["after"]);
                    assert!(view.gap.is_none());
                }
            }
            gameplay.push((serde_json::to_value(&core.event_log).unwrap(), buff));
            if enabled {
                mandatory.push((
                    usage,
                    serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                ));
                // The first sourced slot was overwritten by unsourced reapplications;
                // queued and cloned owners were dropped, so the combined source is reclaimed.
                assert!(weak.unwrap().upgrade().is_none());
            }
        }
        assert!(gameplay.windows(2).all(|pair| pair[0] == pair[1]));
        assert!(mandatory.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn authoring_buff_admission_failure_preserves_candidate_baseline_and_recovers() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut base, caster, target, skill_id) = authoring_buff_core();
        base.enable_authoring_observation();
        base.invoke_ability(
            10,
            caster,
            &skill_id,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
            BattleEventCause::default(),
            false,
        );
        let step = base.event_queue.pop().unwrap();
        base.process_event(step, 60).unwrap();
        let accepted_queue = base.event_queue_debug_snapshot();
        let accepted_log = serde_json::to_value(&base.event_log).unwrap();
        let accepted_observation =
            serde_json::to_value(base.authoring_observation().unwrap()).unwrap();
        let execute =
            |core: &mut super::BattleCore, limits: &ExecutionLimits, token: &CancellationToken| {
                let queued = core.event_queue.pop().unwrap();
                control::run(limits, token, ExecutionPhase::TrialExecution, || {
                    core.process_event(queued, 60)
                })
            };
        let mut complete = base.clone();
        let (ok, usage) = execute(
            &mut complete,
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap();
        ok.unwrap();
        assert_eq!(complete.buffs.len(), 1);
        let mut reservation_boundary_found = false;
        for work in 1..usage.work {
            let mut candidate = base.clone();
            if let Err(error) = execute(
                &mut candidate,
                &ExecutionLimits {
                    max_work: work,
                    ..Default::default()
                },
                &CancellationToken::default(),
            ) {
                if error.site == "authoring_observation_reserve" {
                    assert!(candidate.buffs.is_empty());
                    assert_eq!(
                        serde_json::to_value(&candidate.event_log).unwrap(),
                        accepted_log
                    );
                    assert_eq!(
                        serde_json::to_value(candidate.authoring_observation().unwrap()).unwrap(),
                        accepted_observation
                    );
                    reservation_boundary_found = true;
                    break;
                }
            }
        }
        assert!(
            reservation_boundary_found,
            "exercise the actual pre-apply reservation boundary"
        );
        let mut candidate = base.clone();
        let error = execute(
            &mut candidate,
            &ExecutionLimits {
                max_retained_entities: 1,
                ..Default::default()
            },
            &CancellationToken::default(),
        )
        .unwrap_err();
        assert_eq!(error.site, "buff_apply");
        assert!(candidate.buffs.is_empty());
        assert_eq!(
            serde_json::to_value(&candidate.event_log).unwrap(),
            accepted_log
        );
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        let mut candidate = base.clone();
        assert_eq!(
            execute(&mut candidate, &ExecutionLimits::default(), &cancelled)
                .unwrap_err()
                .code,
            "authoring_cancelled"
        );
        assert!(candidate.buffs.is_empty());
        // A cap at the post-mutation expiry enqueue rejects the entire candidate.
        let mut candidate = base.clone();
        candidate
            .event_queue
            .push(BattleEvent::ContinuousMovementTick { time_ms: 9999 });
        let error = execute(
            &mut candidate,
            &ExecutionLimits {
                max_pending_events: 1,
                ..Default::default()
            },
            &CancellationToken::default(),
        )
        .unwrap_err();
        assert_eq!(error.site, "event_enqueue");
        assert_eq!(
            candidate.buffs.len(),
            1,
            "failure occurs after the private leaf mutation"
        );
        assert_eq!(base.event_queue_debug_snapshot(), accepted_queue);
        assert_eq!(serde_json::to_value(&base.event_log).unwrap(), accepted_log);
        assert_eq!(
            serde_json::to_value(base.authoring_observation().unwrap()).unwrap(),
            accepted_observation
        );
        assert!(base.buffs.is_empty());
        let (ok, _) = execute(
            &mut base,
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap();
        ok.unwrap();
        assert_eq!(base.buffs.len(), 1);
        assert_eq!(
            base.authoring_observation().unwrap().unwrap().applications,
            1
        );
    }

    #[test]
    fn authoring_resonance_observes_real_skill_source_and_only_changed_commands() {
        use crate::authoring::{
            clone_shape::controlled_clone,
            control,
            trace::{TraceAttempt, TraceLimits, TraceOptions},
            CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        use crate::game::battle::core::authoring_observation::AppliedChange;
        let mut skill = self_heal_proc_skill("resource_observed");
        skill.steps[0].effects =
            vec![
                crate::game::ability::SkillEffectDef::ModifyResonance { amount: 70 }
                    .authored("resource_gain"),
            ];
        let skill_id = skill.id.clone();
        let (mut base, caster, _) = authoring_control_core(skill);
        let target = caster;
        base.units.get_mut(&target).unwrap().resonance_current = 40;
        base.units.get_mut(&target).unwrap().skill_activation_mode =
            crate::game::ability::SkillActivationMode::Manual;
        let mut results = Vec::new();
        for limits in [
            None,
            Some(TraceLimits::default()),
            Some(TraceLimits {
                max_bytes: 1,
                max_row_bytes: 1,
                ..Default::default()
            }),
        ] {
            let options = TraceOptions { limits };
            let mut core = base.clone();
            core.enable_authoring_observation_with_trace(options);
            let _attempt = TraceAttempt::enter(options).unwrap();
            let (result, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    // Real ability invocation obtains its cast/source from production compilation.
                    core.invoke_ability(
                        10,
                        caster,
                        &skill_id,
                        Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                            unit_instance_id: target,
                        }),
                        BattleEventCause::default(),
                        false,
                    );
                    drain_event_queue(&mut core);
                    assert_eq!(core.units[&target].resonance_current, 100);
                    let state = core.authoring_observation()?.unwrap();
                    assert_eq!(state.applications, 1);
                    assert_eq!(state.sourced_applications, 1);
                    assert_eq!(state.nonzero_hp_applications, 0);
                    assert!(state.latest_nonzero_sourced_hp.is_none());
                    let witness = state.latest.as_ref().unwrap();
                    assert_eq!(witness.source_unit, Some(caster));
                    assert_eq!(witness.target_unit, target);
                    assert_eq!(witness.time_ms, 10);
                    assert_eq!(witness.change.hp_delta(), 0);
                    assert!(matches!(
                        witness.change,
                        AppliedChange::Resonance {
                            before: 40,
                            after: 100,
                            max: 100
                        }
                    ));
                    let source = witness.source.as_ref().unwrap();
                    assert_eq!(source.cast.skill_id, skill_id);
                    assert_eq!(source.cast.caster_instance_id, caster);
                    assert_eq!(source.authored.site.effect_id.as_str(), "resource_gain");
                    assert!(core.authoring_direct_source.is_none());
                    let accepted = serde_json::to_value(state).unwrap();
                    let cloned = controlled_clone(&core)?;
                    assert_eq!(
                        serde_json::to_value(cloned.authoring_observation()?).unwrap(),
                        accepted
                    );
                    let command = |amount| BattleCommand::ModifyResonance {
                        target_id: target,
                        amount,
                        allow_autocast_when_full: false,
                    };
                    // At cap/zero delta do not fabricate occurrences. Underflow clamps to zero.
                    core.process_commands(vec![command(70), command(0), command(i32::MIN)], 20);
                    assert_eq!(core.units[&target].resonance_current, 0);
                    assert_eq!(core.authoring_observation()?.unwrap().applications, 2);
                    core.units
                        .get_mut(&target)
                        .unwrap()
                        .action_locks
                        .lock_resonance_gain_until(40);
                    core.process_commands(vec![command(20)], 30);
                    assert_eq!(core.authoring_observation()?.unwrap().applications, 2);
                    core.process_commands(vec![command(20)], 40);
                    assert_eq!(core.units[&target].resonance_current, 20);
                    assert_eq!(core.authoring_observation()?.unwrap().applications, 3);
                    assert!(core
                        .authoring_observation()?
                        .unwrap()
                        .latest
                        .as_ref()
                        .unwrap()
                        .source
                        .is_none());
                    core.units.get_mut(&target).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;
                    core.process_commands(vec![command(20)], 50);
                    assert_eq!(core.units[&target].resonance_current, 20);
                    assert_eq!(core.authoring_observation()?.unwrap().applications, 3);
                    assert_eq!(
                        serde_json::to_value(cloned.authoring_observation()?).unwrap(),
                        accepted
                    );
                    Ok::<(), crate::game::behavior::GameError>(())
                },
            )
            .unwrap();
            result.unwrap();
            let view = core.authoring_trace.view();
            match limits {
                None => assert!(!view.enabled),
                Some(limits) if limits.max_bytes == 1 => {
                    assert!(view.ndjson.is_empty());
                    assert_eq!(view.gap.unwrap().dropped_rows, 3);
                }
                Some(_) => {
                    let rows: Vec<serde_json::Value> =
                        serde_json::Deserializer::from_slice(view.ndjson)
                            .into_iter()
                            .map(Result::unwrap)
                            .collect();
                    assert_eq!(rows.len(), 3);
                    assert_eq!(
                        rows[0]["change"],
                        serde_json::json!({
                            "channel":"resonance", "before":40, "after":100, "max":100,
                        })
                    );
                    assert_eq!(
                        rows[0]["source"]["authored"]["site"]["effect_id"],
                        "resource_gain"
                    );
                    assert_eq!(rows[2]["change"]["after"], 20);
                    assert!(view.gap.is_none());
                }
            }
            results.push((
                usage,
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                serde_json::to_value(&core.event_log).unwrap(),
                core.units[&target].resonance_current,
            ));
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], results[2]);
    }

    #[test]
    fn authoring_resonance_reservation_failure_precedes_actual_mutation() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, _, target) = authoring_control_core(damage_proc_skill("resource_limit", 1));
        core.enable_authoring_observation();
        let before = serde_json::to_value(core.authoring_observation().unwrap()).unwrap();
        let events = serde_json::to_value(&core.event_log).unwrap();
        for cancelled in [false, true] {
            let token = CancellationToken::default();
            if cancelled {
                token.cancel();
            }
            let error = control::run(
                &ExecutionLimits {
                    max_work: 1,
                    ..Default::default()
                },
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    core.process_commands(
                        vec![BattleCommand::ModifyResonance {
                            target_id: target,
                            amount: 17,
                            allow_autocast_when_full: true,
                        }],
                        10,
                    )
                },
            )
            .unwrap_err();
            assert_eq!(
                error.code,
                if cancelled {
                    "authoring_cancelled"
                } else {
                    "authoring_work_limit"
                }
            );
            if !cancelled {
                assert_eq!(error.site, "authoring_observation_reserve");
            }
            assert_eq!(core.units[&target].resonance_current, 0);
            assert!(!core.units[&target].pending_cast);
            assert_eq!(
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                before
            );
            assert_eq!(serde_json::to_value(&core.event_log).unwrap(), events);
        }
        // A fresh attempt can use the same actual command after a rejected reservation.
        let (_, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                core.process_commands(
                    vec![BattleCommand::ModifyResonance {
                        target_id: target,
                        amount: 17,
                        allow_autocast_when_full: false,
                    }],
                    10,
                )
            },
        )
        .unwrap();
        assert_eq!(core.units[&target].resonance_current, 17);
        assert_eq!(
            core.authoring_observation().unwrap().unwrap().applications,
            1
        );
    }

    #[test]
    fn authoring_observation_tracks_real_applications_without_log_or_ambient_source_identity() {
        use crate::game::battle::skill_source::SkillCastExecutionSource;
        let (mut core, caster, target) = authoring_control_core(damage_proc_skill("observed", 17));
        core.enable_authoring_observation();
        let skill = core
            .game_data
            .skill_data
            .get_by_id("observed")
            .unwrap()
            .clone();
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: skill.id.clone(),
            caster_instance_id: caster,
            cast_seq: 17,
            ability_event_seq: 19,
            cause: BattleEventCause::default(),
            proc_origin: None,
        });
        for ordinal in 1..=2 {
            let (commands, _) = core
                .build_skill_step_commands(caster, &skill.steps[0], &[target], 0, None)
                .unwrap();
            core.process_skill_commands(commands, 0, caster, Some(cast.clone()));
            let observed = core.authoring_observation().unwrap().unwrap();
            assert_eq!(observed.applications, ordinal);
            assert_eq!(observed.nonzero_hp_applications, ordinal);
            assert_eq!(observed.sourced_applications, ordinal);
            let hit = observed.latest.as_ref().unwrap();
            assert_eq!(hit.ordinal, ordinal);
            assert_eq!(hit.change.hp_delta(), -17);
            assert!(Arc::ptr_eq(&hit.source.as_ref().unwrap().cast, &cast));
            assert_eq!(
                hit.source.as_ref().unwrap().authored.site.effect_id,
                "effect_01"
            );
            let before = serde_json::to_value(observed).unwrap();
            // A fixture log clear removes rows, without resetting semantic state.
            core.event_log.entries.clear();
            assert_eq!(
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                before
            );
        }
        let prior = core
            .authoring_observation()
            .unwrap()
            .unwrap()
            .latest
            .as_ref()
            .unwrap()
            .source
            .clone()
            .unwrap();
        core.authoring_direct_source = Some(prior.clone());
        core.process_command_batches(
            vec![super::RuntimeCommandBatch {
                fact_source: Default::default(),
                commands: vec![BattleCommand::ApplyHeal {
                    target_id: target,
                    flat: 3,
                    percent: 0,
                    source_id: Some(caster),
                }],
                modifier_source_instance_id: Some(caster),
            }],
            1,
        );
        assert!(Arc::ptr_eq(
            core.authoring_direct_source.as_ref().unwrap(),
            &prior
        ));
        let observed = core.authoring_observation().unwrap().unwrap();
        assert_eq!(observed.applications, 3);
        assert_eq!(observed.sourced_applications, 2);
        assert_eq!(observed.latest.as_ref().unwrap().change.hp_delta(), 3);
        assert!(observed.latest.as_ref().unwrap().source.is_none());
        assert_eq!(
            observed.latest_nonzero_sourced_hp.as_ref().unwrap().ordinal,
            2
        );
        assert_eq!(core.units[&target].stats.current_health, 969);
        core.process_commands(
            vec![BattleCommand::ApplyModifier {
                target_id: target,
                modifier: crate::game::stats::StatModifier {
                    stat: crate::game::stats::StatId::Attack,
                    kind: crate::game::stats::StatModifierKind::Flat,
                    value: 2,
                },
            }],
            2,
        );
        let observed = core.authoring_observation().unwrap().unwrap();
        assert_eq!(observed.applications, 4);
        assert_eq!(observed.nonzero_hp_applications, 3);
        assert!(matches!(
            observed.latest.as_ref().unwrap().change,
            super::super::authoring_observation::AppliedChange::Stats { .. }
        ));
        assert!(observed.latest.as_ref().unwrap().source.is_none());
        let cloned = core.clone();
        assert_eq!(
            serde_json::to_value(cloned.authoring_observation().unwrap()).unwrap(),
            serde_json::to_value(observed).unwrap()
        );
    }

    #[test]
    fn authoring_delivery_source_survives_real_log_eviction_and_correlation_shift() {
        use super::super::authoring_observation::ObservationKey;
        use crate::authoring::{
            control,
            trace::{TraceAttempt, TraceLimits, TraceOptions},
            CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        use crate::game::battle::event_log::{SkillCastTarget, BATTLE_HISTORY_MAX_EVENTS};

        fn semantic(mut value: serde_json::Value) -> serde_json::Value {
            fn strip(value: &mut serde_json::Value) {
                match value {
                    serde_json::Value::Object(fields) => {
                        // These fields are explicitly log correlation. Keep cast,
                        // authored site, target, time and application ordinal.
                        if fields.contains_key("authored") && fields.contains_key("cast_seq") {
                            fields.remove("ability_event_seq");
                            fields.remove("cause");
                            if let Some(proc) = fields
                                .get_mut("proc_origin")
                                .and_then(|v| v.as_object_mut())
                            {
                                proc.remove("event_seq");
                            }
                        }
                        for value in fields.values_mut() {
                            strip(value);
                        }
                    }
                    serde_json::Value::Array(values) => {
                        for value in values {
                            strip(value);
                        }
                    }
                    _ => {}
                }
            }
            strip(&mut value);
            value
        }

        for (delivery, expected_per_cast) in [
            ("Instant", 2),
            ("Projectile(speed_units_per_ms:500)", 1),
            ("TileArea(footprint:(include_anchor_tile:false,rows:[\"@X\"]),area:(anchor:Caster,tick_policy:EveryTick,duration_ms:300,tick_interval_ms:Some(100)))", 3),
        ] {
            let mut skill = damage_proc_skill("eviction_delivery", 7);
            skill.steps[0].delay_ms = 100;
            skill.steps[0].delivery = ron::de::from_str(delivery).unwrap();
            if delivery == "Instant" {
                skill.steps[0].repeat = crate::game::ability::SkillStepRepeat::Times { count: 2 };
            }
            let (mut base, caster, target) = authoring_control_core(skill);
            base.units.get_mut(&caster).unwrap().facing_direction =
                Some(crate::game::battle::tile_range::FacingDirection::Up);
            let mut baseline = None;
            let mut baseline_trace = None;
            let mut native_baseline = None;
            let mut correlations = Vec::new();
            for noisy in [false, true] {
                for limits in [None, Some(TraceLimits::default()),
                    Some(TraceLimits { max_bytes: 1, max_row_bytes: 1, ..Default::default() })] {
                    let mut core = base.clone();
                    let options = TraceOptions { limits };
                    core.enable_authoring_observation_with_trace(options);
                    core.bind_authoring_observation(ObservationKey {
                        entry_ordinal: 1, battle_uuid: Uuid::from_u128(0xE71C7),
                    }).unwrap();
                    let _trace = TraceAttempt::enter(options).unwrap();
                    if noisy {
                        for _ in 0..3 {
                            core.record_event_log(0, BattleLogEvent::BattleStart { width: 4, height: 4 });
                        }
                    }
                    core.invoke_ability(10, caster, "eviction_delivery",
                        Some(SkillCastTarget::Unit { unit_instance_id: target }),
                        BattleEventCause::default(), false);
                    let cast_log_seq = core.active_skill_casts[&0].execution_source.as_ref().unwrap().ability_event_seq;
                    correlations.push(cast_log_seq);
                    assert_eq!(core.authoring_observation().unwrap().unwrap().applications, 0);
                    if noisy {
                        // Actual bounded append/eviction while the delayed cast owns
                        // its source. This is test-only observation input, not gameplay.
                        for _ in 0..=BATTLE_HISTORY_MAX_EVENTS {
                            core.record_event_log(10, BattleLogEvent::BattleStart { width: 4, height: 4 });
                        }
                        assert_eq!(core.event_log.entries.len(), BATTLE_HISTORY_MAX_EVENTS);
                        assert!(core.event_log.floor_seq() > cast_log_seq);
                        assert!(core.event_log.entries.iter().all(|entry| entry.seq != cast_log_seq));
                    }
                    control::run(&ExecutionLimits::default(), &CancellationToken::default(),
                        ExecutionPhase::TrialExecution, || drain_event_queue(&mut core)).unwrap();
                    let first = core.authoring_observation().unwrap().unwrap();
                    assert_eq!(first.applications, expected_per_cast);
                    assert_eq!(first.sourced_applications, expected_per_cast);
                    let witness = first.latest_nonzero_sourced_hp.as_ref().unwrap();
                    assert_eq!(witness.source.as_ref().unwrap().cast.ability_event_seq, cast_log_seq);
                    assert_eq!(witness.source.as_ref().unwrap().cast.cast_seq, 0);
                    assert!(witness.time_ms >= 110);
                    assert_eq!(witness.target_unit, target);
                    let first = semantic(serde_json::to_value(first).unwrap());
                    // A following real cast and next damage identity check that
                    // trace/noise did not merely preserve an already captured result.
                    control::run(&ExecutionLimits::default(), &CancellationToken::default(),
                        ExecutionPhase::TrialExecution, || {
                            core.invoke_ability(1000, caster, "eviction_delivery",
                                Some(SkillCastTarget::Unit { unit_instance_id: target }),
                                BattleEventCause::default(), false);
                            drain_event_queue(&mut core);
                        }).unwrap();
                    let observed = core.authoring_observation().unwrap().unwrap();
                    assert_eq!(observed.applications, expected_per_cast * 2);
                    assert_eq!(observed.latest.as_ref().unwrap().source.as_ref().unwrap().cast.cast_seq, 1);
                    let native = serde_json::to_value(core.runtime_fact_view()).unwrap();
                    assert_eq!(native["summary"]["damage_mutations"], expected_per_cast * 2);
                    if let Some(expected) = &native_baseline { assert_eq!(&native, expected); }
                    else { native_baseline = Some(native); }
                    let result = (
                        first,
                        semantic(serde_json::to_value(observed).unwrap()),
                        serde_json::to_value(core.units[&caster].stats).unwrap(),
                        serde_json::to_value(core.units[&target].stats).unwrap(),
                        (core.skill_cast_seq, core.projectile_seq, core.area_seq, core.damage_source_seq),
                        format!("{:?}", core.damage_source_snapshot_for_unit(caster, target,
                            DamageSource::Ability, DamageType::Magic, 7, DamageModifiers::default(), 0, 2000, false)),
                    );
                    if let Some(baseline) = &baseline { assert_eq!(&result, baseline, "{delivery}/{noisy}/{limits:?}"); }
                    else { baseline = Some(result); }
                    let trace = core.authoring_trace.view();
                    match limits {
                        None => assert!(!trace.enabled),
                        Some(limits) if limits.max_bytes == 1 => {
                            assert!(trace.ndjson.is_empty());
                            assert_eq!(trace.gap.unwrap().dropped_rows, expected_per_cast * 2);
                        }
                        Some(_) => {
                            assert!(trace.gap.is_none());
                            let rows: Vec<serde_json::Value> = serde_json::Deserializer::from_slice(trace.ndjson)
                                .into_iter().map(|row| semantic(row.unwrap())).collect();
                            assert_eq!(rows.len() as u64, expected_per_cast * 2);
                            if let Some(baseline) = &baseline_trace { assert_eq!(&rows, baseline); }
                            else { baseline_trace = Some(rows); }
                        }
                    }
                    assert!(core.active_skill_casts.is_empty());
                    assert!(core.active_projectiles.is_empty());
                    assert!(core.active_areas.is_empty());
                }
            }
            assert_eq!(correlations[..3], [correlations[0]; 3]);
            assert_eq!(correlations[3..], [correlations[3]; 3]);
            assert_eq!(correlations[3] - correlations[0], 3);
            // Ordinary native execution has no R03 observer or trace owner.
            let mut bare = base.clone();
            assert!(bare.authoring_observation().unwrap().is_none());
            for time in [10, 1000] {
                bare.invoke_ability(time, caster, "eviction_delivery",
                    Some(SkillCastTarget::Unit { unit_instance_id: target }),
                    BattleEventCause::default(), false);
                drain_event_queue(&mut bare);
            }
            assert_eq!(serde_json::to_value(bare.runtime_fact_view()).unwrap(), native_baseline.unwrap());
        }
    }

    #[test]
    fn optional_trace_actual_sourced_damage_keeps_mandatory_work_and_clones_identical() {
        use crate::authoring::{
            clone_shape::controlled_clone,
            control,
            trace::{TraceAttempt, TraceLimits, TraceOptions},
            CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        use crate::game::battle::skill_source::SkillCastExecutionSource;
        let (base, caster, target) = authoring_control_core(damage_proc_skill("trace_actual", 17));
        let skill = base
            .game_data
            .skill_data
            .get_by_id("trace_actual")
            .unwrap()
            .clone();
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: skill.id.clone(),
            caster_instance_id: caster,
            cast_seq: 17,
            ability_event_seq: 19,
            cause: BattleEventCause::default(),
            proc_origin: None,
        });
        let mut results = Vec::new();
        for limits in [
            None,
            Some(TraceLimits::default()),
            Some(TraceLimits {
                max_attempt_work: 1,
                ..Default::default()
            }),
        ] {
            let options = TraceOptions { limits };
            let mut core = base.clone();
            core.enable_authoring_observation_with_trace(options);
            let attempt = TraceAttempt::enter(options).unwrap();
            let (result, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    for _ in 0..2 {
                        let (commands, _) = core.build_skill_step_commands(
                            caster,
                            &skill.steps[0],
                            &[target],
                            0,
                            None,
                        )?;
                        core.process_skill_commands(commands, 0, caster, Some(cast.clone()));
                    }
                    // Populated optional bytes must not change mandatory clone cost.
                    controlled_clone(&core)
                },
            )
            .unwrap();
            let cloned = result.unwrap();
            assert_eq!(core.units[&target].stats.current_health, 966);
            assert_eq!(
                core.authoring_trace.view().ndjson,
                cloned.authoring_trace.view().ndjson
            );
            match limits {
                None => assert!(!core.authoring_trace.view().enabled),
                Some(limits) if limits.max_attempt_work == 1 => {
                    assert_eq!(core.authoring_trace.view().gap.unwrap().dropped_rows, 2);
                    assert!(core.authoring_trace.view().ndjson.is_empty());
                }
                Some(_) => {
                    let rows: Vec<serde_json::Value> =
                        serde_json::Deserializer::from_slice(core.authoring_trace.view().ndjson)
                            .into_iter()
                            .map(Result::unwrap)
                            .collect();
                    assert_eq!(rows.len(), 2);
                    assert_eq!(rows[0]["ordinal"], 1);
                    assert_eq!(rows[1]["ordinal"], 2);
                    assert_eq!(
                        rows[1]["source"]["authored"]["site"]["effect_id"],
                        "effect_01"
                    );
                    assert_eq!(rows[1]["change"]["after"], 966);
                    assert!(attempt.usage().work > 0);
                }
            }
            results.push((
                usage,
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                serde_json::to_value(&core.event_log).unwrap(),
            ));
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], results[2]);
    }

    #[test]
    fn authoring_observation_reserves_before_hp_mutation_and_rejects_counter_overflow() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, _, target) = authoring_control_core(damage_proc_skill("observed_limit", 17));
        core.enable_authoring_observation();
        let mut candidate = core.clone();
        let limits = ExecutionLimits {
            max_work: 2,
            ..ExecutionLimits::default()
        };
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                candidate.apply_hp_delta_and_record(None, target, -5, 1, HpChangeReason::Command);
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.site, "authoring_observation_reserve");
        assert_eq!(candidate.units[&target].stats.current_health, 1000);
        assert_eq!(
            candidate
                .authoring_observation()
                .unwrap()
                .unwrap()
                .applications,
            0
        );
        control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                candidate.apply_hp_delta_and_record(None, target, -5, 1, HpChangeReason::Command);
            },
        )
        .unwrap();
        assert_eq!(candidate.units[&target].stats.current_health, 995);
        assert_eq!(
            candidate
                .authoring_observation()
                .unwrap()
                .unwrap()
                .applications,
            1
        );
        assert_eq!(core.units[&target].stats.current_health, 1000);
        assert_eq!(
            core.authoring_observation().unwrap().unwrap().applications,
            0
        );
        candidate
            .authoring_observation
            .as_mut()
            .unwrap()
            .applications = u64::MAX;
        let failure = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                candidate.apply_hp_delta_and_record(None, target, -5, 2, HpChangeReason::Command);
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_observation_invalid");
        assert_eq!(failure.site, "authoring_observation_counter");
        assert_eq!(candidate.units[&target].stats.current_health, 995);
        assert!(candidate.authoring_observation().is_err());
    }

    #[test]
    fn authoring_control_effect_buffer_rejects_before_any_command_is_applied() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut skill = damage_proc_skill("buffer_limit", 1);
        skill.steps[0].effects =
            vec![crate::game::ability::SkillEffectDef::ExtraAttack { count: 3 }.authored("extra")];
        let (mut core, caster, target) = authoring_control_core(skill.clone());
        let limits = ExecutionLimits {
            max_effect_commands: 2,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                assert!(core
                    .build_skill_step_commands(caster, &skill.steps[0], &[target], 0, None)
                    .is_err());
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_effect_command_limit");
        assert_eq!(error.requested, Some(3));
        assert!(core.event_queue.is_empty());
        assert_eq!(core.units[&target].stats.current_health, 1000);
        let limits = ExecutionLimits {
            max_effect_commands: 3,
            ..Default::default()
        };
        let (commands, usage) = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                core.build_skill_step_commands(caster, &skill.steps[0], &[target], 0, None)
                    .unwrap()
                    .0
            },
        )
        .unwrap();
        assert_eq!(commands.len(), 3);
        assert_eq!(usage.effect_commands_peak, 3);
    }

    #[test]
    fn authoring_control_geometry_stops_before_validated_projection_without_panicking() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::tile_range::{FacingDirection, TileRangePattern};
        let (core, _, _) = authoring_control_core(damage_proc_skill("geometry_limit", 1));
        let limits = ExecutionLimits {
            max_geometry_cells: 4,
            ..Default::default()
        };
        let exact = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["@XXX".into()],
        };
        let (tiles, usage) = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                core.valid_tile_area_tiles(&exact, Position::new(0, 0), FacingDirection::Up)
                    .unwrap()
            },
        )
        .unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(usage.geometry_cells_peak, 4);
        let oversized = TileRangePattern {
            rows: vec!["@XXXX".into()],
            ..exact
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                assert!(core
                    .valid_tile_area_tiles(&oversized, Position::new(0, 0), FacingDirection::Up)
                    .is_none());
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_geometry_limit");
        assert_eq!(error.requested, Some(5));
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn authoring_control_cast_admission_precedes_identity_allocation_and_context_replacement_is_bounded(
    ) {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::{core::types::SkillImpactContext, event_log::SkillCastTarget};
        let (mut core, caster, target) = authoring_control_core(damage_proc_skill("cast_limit", 1));
        let invoke = |core: &mut super::BattleCore| {
            core.invoke_ability(
                0,
                caster,
                "cast_limit",
                Some(SkillCastTarget::Unit {
                    unit_instance_id: target,
                }),
                BattleEventCause::default(),
                false,
            )
        };
        let limits = ExecutionLimits {
            max_retained_entities: 2,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || invoke(&mut core),
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_entity_limit");
        assert_eq!(core.skill_cast_seq, 0);
        assert!(core.event_queue.is_empty());
        assert!(core.active_skill_casts.is_empty());
        let copy_limits = ExecutionLimits {
            max_work: 1,
            ..Default::default()
        };
        let error = control::run(
            &copy_limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || invoke(&mut core),
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_work_limit");
        assert_eq!(error.site, "skill_definition_copy");
        assert_eq!(core.skill_cast_seq, 0);
        assert!(core.event_queue.is_empty());
        invoke(&mut core);
        assert_eq!(core.skill_cast_seq, 1);
        let context = SkillImpactContext {
            delivery_id: Uuid::from_u128(0xCA734),
            impact_time_ms: 0,
            impact_position: WorldVec2::from_tile_center(Position::new(1, 0)),
            direction_hint: None,
            first_hit_unit_id: Some(target),
            hit_unit_ids: vec![target],
            spawned_area_id: None,
        };
        let (_, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || core.update_skill_cast_impact_context(0, 0, context.clone()),
        )
        .unwrap();
        let limits = ExecutionLimits {
            max_retained_entities: usage.retained_entities_peak,
            ..Default::default()
        };
        for _ in 0..3 {
            control::run(
                &limits,
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || core.update_skill_cast_impact_context(0, 0, context.clone()),
            )
            .unwrap();
        }
        let cast = &core.active_skill_casts[&0];
        assert_eq!(cast.impact_contexts_by_step.len(), 1);
        assert_eq!(cast.last_impact_context.as_ref().unwrap(), &context);
    }

    #[test]
    fn authoring_control_queue_failure_restores_prior_effect_context() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::skill_source::{
            SkillCastExecutionSource, SkillEffectExecutionSource,
        };
        let mut skill = damage_proc_skill("source_limit", 1);
        skill.steps[0].effects =
            vec![crate::game::ability::SkillEffectDef::ExtraAttack { count: 3 }.authored("extra")];
        let (baseline, caster, target) = authoring_control_core(skill);
        let skill = baseline
            .game_data
            .skill_data
            .get_by_id("source_limit")
            .unwrap()
            .clone();
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: skill.id.clone(),
            caster_instance_id: caster,
            cast_seq: 17,
            ability_event_seq: 19,
            cause: BattleEventCause::default(),
            proc_origin: None,
        });
        let prior = Arc::new(SkillEffectExecutionSource {
            authored: skill.steps[0].effects[0].source().unwrap().clone(),
            cast: cast.clone(),
        });
        let mut candidate = baseline.clone();
        candidate.recording_skill_effect_source = Some(prior.clone());
        let (commands, _) = candidate
            .build_skill_step_commands(caster, &skill.steps[0], &[target], 0, None)
            .unwrap();
        let limits = ExecutionLimits {
            max_pending_events: 1,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                candidate.process_skill_commands(commands, 0, caster, Some(cast));
                assert!(Arc::ptr_eq(
                    candidate.recording_skill_effect_source.as_ref().unwrap(),
                    &prior
                ));
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_queue_limit");
        assert_eq!(candidate.event_queue.iter().count(), 1);
        assert!(baseline.event_queue.is_empty());
        assert!(baseline.recording_skill_effect_source.is_none());
    }

    #[test]
    fn authoring_control_area_tick_policies_match_ordinary_execution() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::ability::{
            SkillAreaAnchorSource, SkillAreaTickPolicy, SkillTileAreaDeliveryDef,
        };
        use crate::game::battle::{
            event_log::SkillCastTarget,
            tile_range::{FacingDirection, TileRangePattern},
        };
        for (policy, expected_hp) in [
            (SkillAreaTickPolicy::EveryTick, 997),
            (SkillAreaTickPolicy::OncePerArea, 999),
            (SkillAreaTickPolicy::OnEnter, 999),
        ] {
            let mut skill = damage_proc_skill("area_control", 1);
            skill.steps[0].delivery = DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec!["@X".into()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor: SkillAreaAnchorSource::Caster,
                    tick_policy: policy,
                    duration_ms: 300,
                    tick_interval_ms: Some(100),
                    ..Default::default()
                },
            };
            let (mut baseline, caster, target) = authoring_control_core(skill);
            baseline.units.get_mut(&caster).unwrap().facing_direction = Some(FacingDirection::Up);
            let execute = |core: &mut super::BattleCore| {
                core.invoke_ability(
                    0,
                    caster,
                    "area_control",
                    Some(SkillCastTarget::Unit {
                        unit_instance_id: target,
                    }),
                    BattleEventCause::default(),
                    false,
                );
                drain_event_queue(core);
            };
            let mut ordinary = baseline.clone();
            execute(&mut ordinary);
            assert_eq!(
                ordinary.units[&target].stats.current_health, expected_hp,
                "{policy:?}"
            );
            let mut controlled = baseline.clone();
            let (_, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || execute(&mut controlled),
            )
            .unwrap();
            assert_eq!(
                controlled.units[&target].stats.current_health, expected_hp,
                "{policy:?}"
            );
            assert_eq!(
                serde_json::to_value(&controlled.event_log).unwrap(),
                serde_json::to_value(&ordinary.event_log).unwrap(),
                "{policy:?}"
            );
            assert!(controlled.active_areas.is_empty());
            assert!(controlled.active_skill_casts.is_empty());
            assert!(usage.geometry_cells_peak >= 2);
            assert!(usage.retained_entities_peak >= 4);
        }
    }

    #[test]
    fn authoring_control_fixed_projectile_preserves_impacts_and_counts_detached_runtime() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut baseline, caster, target) =
            authoring_control_core(damage_proc_skill("fixed_control", 1));
        let delivery = Uuid::from_u128(0xCA735);
        let start = WorldVec2::from_tile_center(Position::new(0, 0));
        let aim = WorldVec2::from_tile_center(Position::new(3, 0));
        baseline.active_projectiles.insert(
            delivery,
            ActiveProjectileRuntime {
                delivery_id: delivery,
                cast_seq: 0,
                step_index: 0,
                skill_id: SkillId::from("fixed_control"),
                step_id: "hit".into(),
                caster_instance_id: caster,
                caster_owner: Side::Player,
                spawned_at_ms: 0,
                start,
                current_position: start,
                aim,
                speed_units_per_ms: 30,
                guidance: ProjectileGuidance::Fixed,
                target_unit_id: None,
                source_snapshot: test_damage_source_snapshot(
                    caster,
                    target,
                    DamageSource::Ability,
                    DamageType::Magic,
                    1,
                    0,
                ),
                collision: SkillProjectileCollisionDef {
                    radius_units: 0,
                    ..Default::default()
                },
                hit_unit_ids: Vec::new(),
                killed_unit_count: 0,
                max_kills: None,
                impact_vfx_id: None,
                last_reevaluation_ms: None,
                next_reevaluation_ms: 0,
                max_travel_ms: 100,
                cause: BattleEventCause::default(),
            },
        );
        let mut ordinary = baseline.clone();
        ordinary.advance_skill_projectile(100, delivery).unwrap();
        assert!(ordinary.event_queue.iter().any(|event| matches!(event.as_ref(), BattleEvent::SkillProjectileImpact { first_hit_unit_id: Some(id), .. } if *id == target)));
        let mut controlled = baseline.clone();
        let (advance, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || controlled.advance_skill_projectile(100, delivery),
        )
        .unwrap();
        advance.unwrap();
        assert_eq!(
            controlled.event_queue_debug_snapshot(),
            ordinary.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(&controlled.event_log).unwrap(),
            serde_json::to_value(&ordinary.event_log).unwrap()
        );
        let mut rejected = baseline.clone();
        let limits = ExecutionLimits {
            max_retained_entities: 2,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || rejected.advance_skill_projectile(100, delivery),
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_entity_limit");
        assert_eq!(error.site, "detached_projectile_motion");
        assert_eq!(error.requested, Some(3));
        assert!(rejected.event_queue.is_empty());
        let mut rejected = baseline.clone();
        let limits = ExecutionLimits {
            max_work: usage.work / 2,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || rejected.advance_skill_projectile(100, delivery),
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_work_limit");
        assert_eq!(baseline.active_projectiles.len(), 1);
        assert!(baseline.event_queue.is_empty());
    }

    #[test]
    fn authoring_control_nested_target_scan_is_repeatable_across_map_rebuilds() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::core::skill_runtime::cast::SkillTargetUsefulness;
        use crate::game::battle::tile_range::TileRangePolicy;
        let mut skill = damage_proc_skill("nested_targets", 1);
        skill.steps[0].targeting = StepTargetingMode::RetargetOnStep;
        skill.steps[0].effects = (0..32)
            .map(|index| {
                crate::game::ability::SkillEffectDef::Damage {
                    amount: 1,
                    damage_type: DamageType::Magic,
                }
                .authored(format!("damage_{index}"))
            })
            .collect();
        let (mut baseline, caster, target) = authoring_control_core(skill.clone());
        let alternate = Uuid::from_u128(0xCA734).into();
        let mut other = baseline.units[&target].clone();
        other.instance_id = alternate;
        other.stats.current_health = 100;
        baseline.units.insert(alternate, other);
        place_test_unit(&mut baseline, alternate, Position::new(2, 0));
        baseline.units.get_mut(&caster).unwrap().current_target = Some(target);
        let select = |core: &super::BattleCore, rule| {
            core.choose_skill_target_by_rule(
                caster,
                Side::Player,
                Position::new(0, 0),
                TileRangePolicy::WholeFieldValidTiles,
                None,
                rule,
                false,
                SkillTargetUsefulness::Cast {
                    time_ms: 0,
                    skill: &skill,
                },
            )
        };
        for rule in [
            UnitTargetRule::CurrentTarget,
            UnitTargetRule::Nearest,
            UnitTargetRule::LowestHealthEnemy,
        ] {
            let expected = select(&baseline, rule);
            assert_eq!(
                expected,
                Some(if matches!(rule, UnitTargetRule::LowestHealthEnemy) {
                    alternate
                } else {
                    target
                })
            );
            let (accepted, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || select(&baseline, rule),
            )
            .unwrap();
            assert_eq!(accepted, expected);
            let tight = ExecutionLimits {
                max_work: usage.work / 2,
                ..Default::default()
            };
            let failure = control::run(
                &tight,
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || select(&baseline, rule),
            )
            .unwrap_err();
            assert_eq!(failure.code, "authoring_work_limit");
            for reverse in [false, true, false, true] {
                let mut rebuilt = baseline.clone();
                let mut units: Vec<_> = rebuilt.units.drain().collect();
                units.sort_by_key(|(id, _)| *id);
                if reverse {
                    units.reverse();
                }
                rebuilt.units = std::collections::HashMap::new();
                rebuilt.units.extend(units);
                let actual = control::run(
                    &ExecutionLimits::default(),
                    &CancellationToken::default(),
                    ExecutionPhase::TrialExecution,
                    || select(&rebuilt, rule),
                )
                .unwrap();
                assert_eq!(actual, (accepted, usage.clone()));
                let actual_failure = control::run(
                    &tight,
                    &CancellationToken::default(),
                    ExecutionPhase::TrialExecution,
                    || select(&rebuilt, rule),
                )
                .unwrap_err();
                assert_eq!(actual_failure, failure);
                assert_eq!(rebuilt.skill_cast_seq, baseline.skill_cast_seq);
                assert!(rebuilt.event_queue.is_empty());
            }
        }
    }

    #[test]
    fn authoring_control_bounds_same_event_repeats_and_retry_matches_uncontrolled_execution() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut skill = damage_proc_skill("bounded_repeat", 1);
        skill.steps[0].repeat = crate::game::ability::SkillStepRepeat::Times { count: 255 };
        let (mut baseline, caster, target) = authoring_control_core(skill);
        baseline.invoke_ability(
            0,
            caster,
            "bounded_repeat",
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
            BattleEventCause::default(),
            false,
        );
        let event = baseline
            .event_queue
            .pop()
            .expect("real cast queues its step");
        let mut ordinary = baseline.clone();
        ordinary.process_event(event.clone(), 0).unwrap();
        let mut controlled = baseline.clone();
        let (_, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || controlled.process_event(event.clone(), 0).unwrap(),
        )
        .unwrap();
        assert_eq!(ordinary.units[&target].stats.current_health, 745);
        assert_eq!(controlled.units[&target].stats.current_health, 745);
        let observe = |core: &super::BattleCore| serde_json::to_value(&core.event_log).unwrap();
        assert_eq!(observe(&controlled), observe(&ordinary));
        assert_eq!(
            controlled.event_queue.iter().count(),
            ordinary.event_queue.iter().count()
        );

        let mut rejected = baseline.clone();
        let limits = ExecutionLimits {
            max_work: usage.work / 2,
            ..Default::default()
        };
        let error = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                assert!(rejected.process_event(event.clone(), 0).is_err());
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_work_limit");
        assert!(rejected.units[&target].stats.current_health > 745);
        assert!(rejected.units[&target].stats.current_health < 1000);
        assert_eq!(baseline.units[&target].stats.current_health, 1000);
        assert!(rejected.recording_skill_effect_source.is_none());
        let mut retry = baseline.clone();
        control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || retry.process_event(event, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(observe(&retry), observe(&ordinary));
    }

    fn committed_damage_test_core(
        attacker_id: crate::game::battle::ids::UnitInstanceId,
        target_id: crate::game::battle::ids::UnitInstanceId,
        delivery: AttackDelivery,
        target_hp: u32,
    ) -> super::BattleCore {
        let mut core = new_test_core(empty_game_data(), 0xC028);
        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            Uuid::nil(),
            UnitStats::with_values(200, 200, 100, 0, 1000),
        );
        attacker.basic_attack.damage_type = DamageType::Physical;
        if matches!(delivery, AttackDelivery::Projectile) {
            attacker.basic_attack.delivery = DeliveryDef::Projectile {
                speed_units_per_ms: DATA_UNITS_PER_WORLD as u32,
                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            };
        }
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            Uuid::nil(),
            UnitStats::with_values(target_hp, target_hp, 1, 0, 1000),
        );
        target.incoming_damage_modifiers = DamageModifiers {
            crit_chance_percent: 100,
            crit_damage_percent: 50,
            ..DamageModifiers::default()
        };
        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));
        core
    }

    fn damage_observation(
        core: &super::BattleCore,
        target_id: crate::game::battle::ids::UnitInstanceId,
    ) -> (
        u32,
        u32,
        i32,
        u32,
        u32,
        bool,
        Vec<DamageBreakdown>,
        Vec<DamageFeedbackTag>,
    ) {
        core.event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::HpChanged {
                    target_instance_id,
                    hp_before,
                    hp_after,
                    delta,
                    raw_damage: Some(raw_damage),
                    final_damage: Some(final_damage),
                    damage_breakdown: Some(damage_breakdown),
                    critical: Some(critical),
                    feedback_tags,
                    ..
                } if *target_instance_id == target_id => Some((
                    *hp_before,
                    *hp_after,
                    *delta,
                    *raw_damage,
                    *final_damage,
                    *critical,
                    damage_breakdown.clone(),
                    feedback_tags.clone(),
                )),
                _ => None,
            })
            .expect("missing committed damage HpChanged")
    }

    #[test]
    fn instant_projectile_and_general_damage_share_snapshot_live_target_and_crit_contract() {
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC281));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC282));

        let mut instant =
            committed_damage_test_core(attacker_id, target_id, AttackDelivery::Instant, 200);
        let instant_snapshot = instant
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                0,
                true,
            )
            .expect("instant source snapshot");
        instant.units.get_mut(&attacker_id).unwrap().stats.attack = 999;
        instant.units.get_mut(&target_id).unwrap().stats.defense = 100;
        assert!(instant.resolve_committed_basic_attack(
            attacker_id,
            target_id,
            instant_snapshot,
            AttackDelivery::Instant,
            10,
        ));

        let mut projectile =
            committed_damage_test_core(attacker_id, target_id, AttackDelivery::Projectile, 200);
        let projectile_request_snapshot = projectile
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                1,
                0,
                true,
            )
            .expect("projectile request snapshot");
        assert!(projectile.resolve_committed_basic_attack(
            attacker_id,
            target_id,
            projectile_request_snapshot,
            AttackDelivery::Projectile,
            0,
        ));
        projectile.units.get_mut(&attacker_id).unwrap().stats.attack = 999;
        projectile.units.get_mut(&target_id).unwrap().stats.defense = 100;
        let projectile_id = *projectile
            .projectiles
            .keys()
            .next()
            .expect("launched basic projectile");
        projectile.advance_basic_attack_projectile(1, projectile_id);

        let mut general =
            committed_damage_test_core(attacker_id, target_id, AttackDelivery::Instant, 200);
        let general_snapshot = general
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::Ability,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                0,
                0,
                false,
            )
            .expect("general source snapshot");
        general.units.get_mut(&attacker_id).unwrap().stats.attack = 999;
        general.units.get_mut(&target_id).unwrap().stats.defense = 100;
        let general_summary = general.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id,
                source_snapshot: general_snapshot,
            }],
            1,
        );

        let instant_damage = damage_observation(&instant, target_id);
        let projectile_damage = damage_observation(&projectile, target_id);
        let general_damage = damage_observation(&general, target_id);
        assert_eq!(instant_damage, projectile_damage);
        assert_eq!(instant_damage, general_damage);
        assert_eq!(instant_damage.0, 200);
        assert_eq!(instant_damage.1, 125);
        assert_eq!(instant_damage.2, -75);
        assert_eq!(instant_damage.3, 150);
        assert_eq!(instant_damage.4, 75);
        assert!(instant_damage.5);
        assert_eq!(instant_damage.6.len(), 1);
        assert_eq!(instant_damage.6[0].damage_type, DamageType::Physical);
        assert_eq!(
            instant_damage.7,
            vec![DamageFeedbackTag::Mitigated, DamageFeedbackTag::Critical]
        );

        assert_eq!(instant.units[&attacker_id].resonance_current, 17);
        assert_eq!(instant.units[&target_id].resonance_current, 7);
        assert_eq!(projectile.units[&attacker_id].resonance_current, 17);
        assert_eq!(projectile.units[&target_id].resonance_current, 7);
        assert_eq!(general.units[&attacker_id].resonance_current, 0);
        assert_eq!(general.units[&target_id].resonance_current, 0);
        assert_eq!(general_summary.actual_damage_target_count, 1);
        assert_eq!(general_summary.killed_target_count, 0);
    }

    #[test]
    fn committed_damage_uses_actual_overkill_delta_and_finalizes_one_causal_death() {
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC283));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC284));

        for delivery in [AttackDelivery::Instant, AttackDelivery::Projectile] {
            let mut core = committed_damage_test_core(attacker_id, target_id, delivery, 5);
            let snapshot = core
                .damage_source_snapshot_for_unit(
                    attacker_id,
                    target_id,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    100,
                    DamageModifiers::default(),
                    1,
                    0,
                    true,
                )
                .expect("basic source snapshot");
            assert!(core.resolve_committed_basic_attack(
                attacker_id,
                target_id,
                snapshot,
                delivery,
                0,
            ));
            if matches!(delivery, AttackDelivery::Projectile) {
                let projectile_id = *core
                    .projectiles
                    .keys()
                    .next()
                    .expect("launched basic projectile");
                core.advance_basic_attack_projectile(1, projectile_id);
            }

            assert_eq!(damage_observation(&core, target_id).2, -5);
            assert_eq!(core.units[&attacker_id].resonance_current, 10);
            assert!(core.graveyard.contains_key(&target_id));
            assert_eq!(
                core.event_log
                    .entries
                    .iter()
                    .filter(|entry| matches!(
                        entry.event,
                        BattleLogEvent::UnitDied { unit_instance_id, .. }
                            if unit_instance_id == target_id
                    ))
                    .count(),
                1
            );
        }

        let mut general =
            committed_damage_test_core(attacker_id, target_id, AttackDelivery::Instant, 5);
        let snapshot = general
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::Ability,
                DamageType::Physical,
                100,
                DamageModifiers::default(),
                0,
                0,
                false,
            )
            .expect("general source snapshot");
        let parent_seq = general.record_event_log(
            0,
            BattleLogEvent::BattleStart {
                width: 4,
                height: 4,
            },
        );
        let summary = general.with_recording_cause(parent_seq, |core| {
            core.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id,
                    source_snapshot: snapshot,
                }],
                1,
            )
        });
        assert_eq!(summary.actual_damage_target_count, 1);
        assert_eq!(summary.killed_target_count, 1);

        let damage_entries: Vec<_> = general
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.event,
                    BattleLogEvent::HpChanged { target_instance_id, .. }
                        | BattleLogEvent::UnitDied {
                            unit_instance_id: target_instance_id,
                            ..
                        } if target_instance_id == target_id
                )
            })
            .collect();
        assert_eq!(damage_entries.len(), 2);
        assert!(matches!(
            damage_entries[0].event,
            BattleLogEvent::HpChanged { delta: -5, .. }
        ));
        assert!(matches!(
            damage_entries[1].event,
            BattleLogEvent::UnitDied {
                killer_instance_id: Some(killer_id),
                ..
            } if killer_id == attacker_id
        ));
        assert!(damage_entries
            .iter()
            .all(|entry| entry.cause == BattleEventCause::Parent { seq: parent_seq }));
    }

    #[test]
    fn general_damage_leaf_does_not_collect_basic_attack_or_hit_occurrences() {
        let attacker_base_uuid = Uuid::from_u128(0xC285);
        let target_base_uuid = Uuid::from_u128(0xC286);
        let item_base_uuid = Uuid::from_u128(0xC287);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC288));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC289));
        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::BonusDamage {
                    flat: 10,
                    percent: 0,
                    damage_type: DamageType::True,
                },
            )],
        );
        item_effects.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::BonusDamage {
                    flat: 8,
                    percent: 0,
                    damage_type: DamageType::True,
                },
            )],
        );
        let activation = AbilityActivationBinding {
            ability_id: SkillId::from("general_leaf_must_not_proc"),
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
            item_effects,
            vec![activation],
            vec![damage_proc_skill("general_leaf_must_not_proc", 5)],
        );
        let mut core = new_test_core(game_data, 0xC028);
        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                UnitStats::with_values(100, 100, 20, 0, 1000),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                target_base_uuid,
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        for (instance_id, owner, owner_unit_instance) in [
            (Uuid::from_u128(0xC28A), Side::Player, attacker_id),
            (Uuid::from_u128(0xC28B), Side::Opponent, target_id),
        ] {
            core.items.insert(
                instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id,
                    owner,
                    owner_unit_instance,
                    base_uuid: item_base_uuid,
                },
            );
        }

        let source_snapshot = core
            .damage_source_snapshot_for_unit(
                attacker_id,
                target_id,
                DamageSource::Ability,
                DamageType::True,
                20,
                DamageModifiers::default(),
                0,
                5,
                false,
            )
            .expect("general source snapshot");
        assert!(source_snapshot.on_attack_bonus_damage.is_empty());
        let summary = core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id,
                source_snapshot,
            }],
            5,
        );

        assert_eq!(core.units[&attacker_id].stats.current_health, 100);
        assert_eq!(core.units[&target_id].stats.current_health, 80);
        assert_eq!(summary.actual_damage_target_count, 1);
        assert_eq!(summary.killed_target_count, 0);
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::TriggeredAbilityProc { .. } | BattleLogEvent::AbilityCast { .. }
        )));
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

        let (commands, _) = core
            .build_skill_step_commands(
                caster_id,
                &skill.steps[0],
                &[target_id],
                launch_snapshot.committed_at_ms,
                Some(&launch_snapshot),
            )
            .unwrap();
        core.process_skill_commands(
            commands,
            20,
            caster_id,
            launch_snapshot.skill_cast_source.clone(),
        );

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
        )
        .unwrap();

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
    fn runtime_max_health_modifier_decision_table_preserves_hp_death_and_provenance() {
        for (
            case_index,
            sourced,
            kind,
            value,
            expected_max_health,
            expected_current_health,
            lethal,
        ) in [
            (0_u128, false, StatModifierKind::Flat, -40, 60, 60, false),
            (1, true, StatModifierKind::Flat, -40, 60, 60, false),
            (2, false, StatModifierKind::Percent, -50, 50, 50, false),
            (3, true, StatModifierKind::Percent, -50, 50, 50, false),
            (4, false, StatModifierKind::Flat, -100, 0, 0, true),
            (5, true, StatModifierKind::Flat, -100, 0, 0, true),
            (6, false, StatModifierKind::Percent, -100, 0, 0, true),
            (7, true, StatModifierKind::Percent, -100, 0, 0, true),
        ] {
            let source_base_uuid = Uuid::from_u128(0xA10 + case_index);
            let target_base_uuid = Uuid::from_u128(0xA20 + case_index);
            let item_base_uuid = Uuid::from_u128(0xA30 + case_index);
            let source_id =
                crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xB10 + case_index));
            let target_id =
                crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xB20 + case_index));
            let modifier = StatModifier {
                stat: StatId::MaxHealth,
                kind,
                value,
            };
            let skill = stat_modifier_skill(&format!("max_health_case_{case_index}"), modifier);
            let skill_id = skill.id.clone();
            let game_data = battle_test_game_data(
                source_base_uuid,
                target_base_uuid,
                item_base_uuid,
                HashMap::new(),
                vec![],
                vec![skill],
            );
            let mut core = new_test_core(game_data, 20 + case_index as u64);
            core.units.insert(
                source_id,
                test_runtime_unit(
                    source_id,
                    Side::Player,
                    source_base_uuid,
                    UnitStats::with_values(100, 100, 10, 0, 1000),
                ),
            );
            core.units.insert(
                target_id,
                test_runtime_unit(
                    target_id,
                    Side::Opponent,
                    target_base_uuid,
                    UnitStats::with_values(100, 80, 1, 0, 1000),
                ),
            );
            place_test_unit(&mut core, source_id, Position::new(0, 0));
            place_test_unit(&mut core, target_id, Position::new(1, 0));

            let execution_summary = if sourced {
                core.invoke_ability(
                    7,
                    source_id,
                    &skill_id,
                    Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                        unit_instance_id: target_id,
                    }),
                    BattleEventCause::default(),
                    false,
                );
                drain_event_queue(&mut core);
                None
            } else {
                Some(core.process_commands(
                    vec![BattleCommand::ApplyModifier {
                        target_id,
                        modifier,
                    }],
                    7,
                ))
            };

            let expected_source = sourced.then_some(source_id);
            assert_eq!(core.runtime_fact_summary().deaths, u64::from(lethal));
            if lethal {
                let death = core.runtime_fact_summary().latest_death.as_ref().unwrap();
                assert_eq!(death.predeath.hp_before_lethal, 80);
                assert_eq!(death.predeath.victim, target_id);
                assert_eq!(death.source.actor, expected_source);
                assert!(death.lethal_damage.is_none());
            }
            let stat_events = core
                .event_log
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.event,
                        BattleLogEvent::StatChanged {
                            target_instance_id,
                            modifier: event_modifier,
                            ..
                        } if target_instance_id == target_id && event_modifier == modifier
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(stat_events.len(), 1, "case {case_index}");
            let BattleLogEvent::StatChanged {
                source_instance_id,
                stats_before,
                stats_after,
                ..
            } = stat_events[0].event
            else {
                unreachable!();
            };
            assert_eq!(source_instance_id, expected_source, "case {case_index}");
            assert_eq!(stats_before.max_health, 100, "case {case_index}");
            assert_eq!(stats_before.current_health, 80, "case {case_index}");
            assert_eq!(
                stats_after.max_health, expected_max_health,
                "case {case_index}"
            );
            assert_eq!(
                stats_after.current_health, expected_current_health,
                "case {case_index}"
            );

            let hp_events = core
                .event_log
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.event,
                        BattleLogEvent::HpChanged { target_instance_id, .. }
                            if target_instance_id == target_id
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(hp_events.len(), 1, "case {case_index}");
            let BattleLogEvent::HpChanged {
                source_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
                damage_source,
                damage_type,
                raw_damage,
                final_damage,
                ..
            } = hp_events[0].event
            else {
                unreachable!();
            };
            assert_eq!(source_instance_id, expected_source, "case {case_index}");
            assert_eq!(hp_before, 80, "case {case_index}");
            assert_eq!(hp_after, expected_current_health, "case {case_index}");
            assert_eq!(
                delta,
                expected_current_health as i32 - 80,
                "case {case_index}"
            );
            assert_eq!(reason, HpChangeReason::Command, "case {case_index}");
            assert_eq!(damage_source, None, "case {case_index}");
            assert_eq!(damage_type, None, "case {case_index}");
            assert_eq!(raw_damage, None, "case {case_index}");
            assert_eq!(final_damage, None, "case {case_index}");
            assert!(stat_events[0].seq < hp_events[0].seq, "case {case_index}");

            let death_events = core
                .event_log
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.event,
                        BattleLogEvent::UnitDied { unit_instance_id, .. }
                            if unit_instance_id == target_id
                    )
                })
                .collect::<Vec<_>>();
            if lethal {
                assert_eq!(death_events.len(), 1, "case {case_index}");
                assert!(hp_events[0].seq < death_events[0].seq, "case {case_index}");
                assert!(matches!(
                    death_events[0].event,
                    BattleLogEvent::UnitDied {
                        killer_instance_id,
                        ..
                    } if killer_instance_id == expected_source
                ));
                assert!(matches!(
                    core.units[&target_id].lifecycle,
                    RuntimeUnitLifecycle::Dead
                ));
                assert!(core.graveyard.contains_key(&target_id));
            } else {
                assert!(death_events.is_empty(), "case {case_index}");
                assert!(core.units[&target_id].is_active(), "case {case_index}");
                assert!(!core.graveyard.contains_key(&target_id));
            }
            if let Some(execution_summary) = execution_summary {
                assert_eq!(
                    execution_summary.killed_target_count,
                    usize::from(lethal),
                    "case {case_index}"
                );
            }
            assert!(
                !core
                    .units
                    .values()
                    .any(|unit| unit.is_active() && unit.stats.current_health == 0),
                "case {case_index}"
            );
        }
    }

    #[test]
    fn runtime_max_health_modifier_without_a_clamp_emits_no_hp_event_or_heal() {
        let mut core = new_test_core(empty_game_data(), 30);
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xBC01));
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Player,
                Uuid::nil(),
                UnitStats::with_values(100, 40, 10, 0, 1000),
            ),
        );
        place_test_unit(&mut core, target_id, Position::new(0, 0));

        core.process_commands(
            vec![
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier: StatModifier {
                        stat: StatId::MaxHealth,
                        kind: StatModifierKind::Flat,
                        value: 50,
                    },
                },
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier: StatModifier {
                        stat: StatId::MaxHealth,
                        kind: StatModifierKind::Flat,
                        value: -20,
                    },
                },
            ],
            3,
        );

        assert_eq!(core.units[&target_id].stats.max_health, 130);
        assert_eq!(core.units[&target_id].stats.current_health, 40);
        assert!(core.units[&target_id].is_active());
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::StatChanged { target_instance_id, .. }
                        if target_instance_id == target_id
                ))
                .count(),
            2
        );
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::HpChanged { target_instance_id, .. }
                if target_instance_id == target_id
        )));
    }

    #[test]
    fn sourced_lethal_max_health_modifier_runs_cleanup_and_nested_death_heals_once() {
        let source_base_uuid = Uuid::from_u128(0xC10);
        let target_base_uuid = Uuid::from_u128(0xC20);
        let item_base_uuid = Uuid::from_u128(0xC30);
        let source_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC11));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC21));
        let collapse_skill = stat_modifier_skill(
            "max_health_collapse",
            StatModifier {
                stat: StatId::MaxHealth,
                kind: StatModifierKind::Flat,
                value: -10,
            },
        );
        let collapse_skill_id = collapse_skill.id.clone();

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnKill,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::Modifier(StatModifier {
                    stat: StatId::Attack,
                    kind: StatModifierKind::Flat,
                    value: 2,
                }),
            )],
        );
        let activations = [
            (TriggerType::OnDeath, "death_heal"),
            (TriggerType::OnKill, "kill_heal"),
        ]
        .into_iter()
        .map(|(trigger, ability_id)| AbilityActivationBinding {
            ability_id: SkillId::from(ability_id),
            activation: AbilityActivationDef::TriggerProc {
                trigger,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: None,
            },
        })
        .collect::<Vec<_>>();
        let game_data = battle_test_game_data(
            source_base_uuid,
            target_base_uuid,
            item_base_uuid,
            item_effects,
            activations,
            vec![
                collapse_skill,
                self_heal_proc_skill("death_heal"),
                self_heal_proc_skill("kill_heal"),
            ],
        );
        let mut core = new_test_core(game_data, 31);
        core.units.insert(
            source_id,
            test_runtime_unit(
                source_id,
                Side::Player,
                source_base_uuid,
                UnitStats::with_values(100, 90, 10, 0, 1000),
            ),
        );
        core.units.insert(
            target_id,
            test_runtime_unit(
                target_id,
                Side::Opponent,
                target_base_uuid,
                UnitStats::with_values(10, 10, 1, 0, 1000),
            ),
        );
        for (index, (owner, owner_unit_instance)) in
            [(Side::Player, source_id), (Side::Opponent, target_id)]
                .into_iter()
                .enumerate()
        {
            let instance_id = Uuid::from_u128(0xC31 + index as u128);
            core.items.insert(
                instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id,
                    owner,
                    owner_unit_instance,
                    base_uuid: item_base_uuid,
                },
            );
        }
        place_test_unit(&mut core, source_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));
        let poison = crate::game::battle::buffs::BuffId::from_name("poison");
        core.process_commands(
            vec![
                BattleCommand::ApplyBuff {
                    caster_id: source_id,
                    target_id,
                    buff_id: poison,
                    duration_ms: 500,
                },
                BattleCommand::ApplyBuff {
                    caster_id: target_id,
                    target_id: source_id,
                    buff_id: poison,
                    duration_ms: 500,
                },
            ],
            0,
        );

        core.enable_authoring_observation();
        core.invoke_ability(
            10,
            source_id,
            &collapse_skill_id,
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: target_id,
            }),
            BattleEventCause::default(),
            false,
        );
        drain_event_queue(&mut core);

        assert!(matches!(
            core.units[&target_id].lifecycle,
            RuntimeUnitLifecycle::Dead
        ));
        assert_eq!(core.units[&target_id].stats.current_health, 0);
        assert_eq!(core.units[&source_id].stats.current_health, 91);
        assert_eq!(core.units[&source_id].stats.attack, 12);
        assert!(core.graveyard.contains_key(&target_id));
        assert!(core.buffs.is_empty());

        let observed = core.authoring_observation().unwrap().unwrap();
        // Two earlier unsourced Buff applications, lethal Stats, unsourced
        // OnKill modifier, and actual kill-heal skill. The dead target's heal
        // never mutates HP. Buff applications do not inflate HP/source counters.
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(entry.event, BattleLogEvent::BuffApplied { .. }))
                .count(),
            2
        );
        assert_eq!(observed.applications, 5);
        assert_eq!(observed.sourced_applications, 2);
        assert_eq!(observed.nonzero_hp_applications, 2);
        let latest = observed.latest.as_ref().unwrap();
        assert_eq!(latest.ordinal, 5);
        assert_eq!(latest.change.hp_delta(), 1);
        assert_eq!(
            latest.source.as_ref().unwrap().cast.skill_id.as_str(),
            "kill_heal"
        );
        assert!(core.authoring_direct_source.is_none());
        let preserved = serde_json::to_value(observed).unwrap();
        let mut clone = core.clone();
        clone.process_commands(
            vec![BattleCommand::ApplyHeal {
                target_id: source_id,
                flat: 1,
                percent: 0,
                source_id: None,
            }],
            11,
        );
        assert_eq!(
            clone.authoring_observation().unwrap().unwrap().applications,
            6
        );
        assert_eq!(clone.units[&source_id].stats.current_health, 92);
        assert_eq!(core.units[&source_id].stats.current_health, 91);
        assert_eq!(
            serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
            preserved
        );
        assert!(Arc::ptr_eq(
            clone
                .authoring_observation()
                .unwrap()
                .unwrap()
                .latest_nonzero_sourced_hp
                .as_ref()
                .unwrap()
                .source
                .as_ref()
                .unwrap(),
            latest.source.as_ref().unwrap()
        ));

        let target_hp_events = core
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.event,
                    BattleLogEvent::HpChanged { target_instance_id, .. }
                        if target_instance_id == target_id
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(target_hp_events.len(), 1);
        assert!(matches!(
            target_hp_events[0].event,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source),
                delta: -10,
                hp_before: 10,
                hp_after: 0,
                reason: HpChangeReason::Command,
                ..
            } if source == source_id
        ));
        let target_deaths = core
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.event,
                    BattleLogEvent::UnitDied { unit_instance_id, .. }
                        if unit_instance_id == target_id
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(target_deaths.len(), 1);
        assert!(matches!(
            target_deaths[0].event,
            BattleLogEvent::UnitDied {
                killer_instance_id: Some(killer),
                ..
            } if killer == source_id
        ));
        assert!(target_hp_events[0].seq < target_deaths[0].seq);
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::AbilityCast {
                skill_id,
                caster_instance_id,
                ..
            } if skill_id.as_str() == "death_heal" && *caster_instance_id == target_id
        )));
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::AbilityCast {
                skill_id,
                caster_instance_id,
                ..
            } if skill_id.as_str() == "kill_heal" && *caster_instance_id == source_id
        )));
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
                employee_trauma: None,
                corroded: false,
                fragment_source: None,
                basic_attack_speed_percent: 100,
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
                equipment_slots: None,
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
                basic_attack_epoch: 0,
                basic_attack_windup_until_ms: None,
                ranged_reposition_until_ms: 0,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                resonance_use: None,
                next_resonance_pulse_ms: None,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );

        core.units.insert(
            target_id,
            RuntimeUnit {
                employee_trauma: None,
                corroded: false,
                fragment_source: None,
                basic_attack_speed_percent: 100,
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
                equipment_slots: None,
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
                basic_attack_epoch: 0,
                basic_attack_windup_until_ms: None,
                ranged_reposition_until_ms: 0,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                resonance_use: None,
                next_resonance_pulse_ms: None,
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
                spatial: None,
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
                employee_trauma: None,
                corroded: false,
                fragment_source: None,
                basic_attack_speed_percent: 100,
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
                equipment_slots: None,
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
                basic_attack_epoch: 0,
                basic_attack_windup_until_ms: None,
                ranged_reposition_until_ms: 0,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                resonance_use: None,
                next_resonance_pulse_ms: None,
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
                persistent_copy_uuid: None,
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
    fn item_occurrence_executes_ordinary_effect_before_ability_activation() {
        let attacker_base_uuid = Uuid::from_u128(0x110);
        let target_base_uuid = Uuid::from_u128(0x120);
        let item_base_uuid = Uuid::from_u128(0x130);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x111));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x121));
        let item_instance_id = Uuid::from_u128(0x131);

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::Modifier(crate::game::stats::StatModifier {
                    stat: crate::game::stats::StatId::Attack,
                    kind: crate::game::stats::StatModifierKind::Flat,
                    value: 5,
                }),
            )],
        );
        let activation = AbilityActivationBinding {
            ability_id: SkillId::from("ordered_item_proc"),
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
            item_effects,
            vec![activation],
            vec![damage_proc_skill("ordered_item_proc", 5)],
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
                persistent_copy_uuid: None,
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

        assert_eq!(core.units[&attacker_id].stats.attack, 15);
        let stat_seq = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match entry.event {
                BattleLogEvent::StatChanged {
                    target_instance_id, ..
                } if target_instance_id == attacker_id => Some(entry.seq),
                _ => None,
            })
            .expect("missing item StatChanged");
        let proc_seq = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::TriggeredAbilityProc {
                    skill_id,
                    caster_instance_id,
                    ..
                } if skill_id.as_str() == "ordered_item_proc"
                    && *caster_instance_id == attacker_id =>
                {
                    Some(entry.seq)
                }
                _ => None,
            })
            .expect("missing item proc event");
        let cast_seq = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::AbilityCast {
                    skill_id,
                    caster_instance_id,
                    ..
                } if skill_id.as_str() == "ordered_item_proc"
                    && *caster_instance_id == attacker_id =>
                {
                    Some(entry.seq)
                }
                _ => None,
            })
            .expect("missing item ability cast");
        assert!(stat_seq < proc_seq && proc_seq < cast_seq);
    }

    #[test]
    fn item_occurrence_all_enemies_targets_are_complete_and_sorted() {
        let source_base_uuid = Uuid::from_u128(0x810);
        let enemy_base_uuid = Uuid::from_u128(0x820);
        let item_base_uuid = Uuid::from_u128(0x830);
        let source_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x811));
        let ally_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x812));
        let first_enemy_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x821));
        let second_enemy_id =
            crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x822));
        let item_instance_id = Uuid::from_u128(0x831);

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnBattleStart,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::AllEnemies,
                crate::game::stats::Effect::Modifier(crate::game::stats::StatModifier {
                    stat: crate::game::stats::StatId::Attack,
                    kind: crate::game::stats::StatModifierKind::Flat,
                    value: 3,
                }),
            )],
        );
        let game_data = battle_test_game_data(
            source_base_uuid,
            enemy_base_uuid,
            item_base_uuid,
            item_effects,
            vec![],
            vec![],
        );
        let mut core = new_test_core(game_data, 13);
        for (unit_id, owner, base_uuid) in [
            (source_id, Side::Player, source_base_uuid),
            (ally_id, Side::Player, source_base_uuid),
            (second_enemy_id, Side::Opponent, enemy_base_uuid),
            (first_enemy_id, Side::Opponent, enemy_base_uuid),
        ] {
            core.units.insert(
                unit_id,
                test_runtime_unit(
                    unit_id,
                    owner,
                    base_uuid,
                    UnitStats::with_values(100, 100, 10, 0, 1000),
                ),
            );
        }
        core.items.insert(
            item_instance_id,
            crate::game::battle::core::RuntimeItem {
                persistent_copy_uuid: None,
                instance_id: item_instance_id,
                owner: Side::Player,
                owner_unit_instance: source_id,
                base_uuid: item_base_uuid,
            },
        );

        let plan = core.dispatch_item_trigger_occurrence(
            crate::game::battle::core::triggers::TriggerOccurrenceContext {
                trigger_type: TriggerType::OnBattleStart,
                trigger_unit_id: source_id,
                counterpart_unit_id: None,
                target_id: None,
                occurrence_id: Uuid::from_u128(0x832),
                occurrence_index: 0,
                allow_dead_caster: false,
                committed_hit: None,
                time_ms: 0,
            },
        );

        let target_ids = plan
            .effect_batches
            .iter()
            .flat_map(|batch| batch.commands.iter())
            .map(|command| match command {
                BattleCommand::ApplyModifier { target_id, .. } => *target_id,
                other => panic!("unexpected command: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(target_ids, vec![first_enemy_id, second_enemy_id]);
        assert!(plan.current_hit_effects.is_empty());
        assert!(plan.activation_commands.is_empty());
    }

    #[test]
    fn typed_item_occurrence_plan_preserves_attack_hit_and_death_context_rules() {
        for (case_index, trigger_type, target_id_present, allow_dead_caster) in [
            (0_u128, TriggerType::OnAttack, true, false),
            (1, TriggerType::OnHit, true, false),
            (2, TriggerType::OnKill, true, false),
            (3, TriggerType::OnDeath, false, true),
            (4, TriggerType::OnAllyDeath, false, false),
        ] {
            let trigger_base_uuid = Uuid::from_u128(0x210 + case_index);
            let counterpart_base_uuid = Uuid::from_u128(0x220 + case_index);
            let item_base_uuid = Uuid::from_u128(0x230 + case_index);
            let trigger_unit_id =
                crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x310 + case_index));
            let counterpart_unit_id =
                crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x320 + case_index));
            let item_instance_id = Uuid::from_u128(0x330 + case_index);
            let occurrence_id = Uuid::from_u128(0x340 + case_index);

            let mut item_effects = HashMap::new();
            let effect_target =
                if matches!(trigger_type, TriggerType::OnKill | TriggerType::OnAllyDeath) {
                    TriggerEffectTarget::SelfUnit
                } else {
                    TriggerEffectTarget::CounterpartUnit
                };
            item_effects.insert(
                trigger_type,
                vec![TriggeredEffect::targeted(
                    effect_target,
                    crate::game::stats::Effect::ApplyBuff {
                        buff_id: "poison".to_string(),
                        duration_ms: 250,
                    },
                )],
            );
            let activation = AbilityActivationBinding {
                ability_id: SkillId::from("matrix_proc"),
                activation: AbilityActivationDef::TriggerProc {
                    trigger: trigger_type,
                    proc_chance_percent: 100,
                    internal_cooldown_ms: 0,
                    max_triggers_per_battle: None,
                },
            };
            let game_data = battle_test_game_data(
                trigger_base_uuid,
                counterpart_base_uuid,
                item_base_uuid,
                item_effects,
                vec![activation],
                vec![self_heal_proc_skill("matrix_proc")],
            );
            let mut core = new_test_core(game_data, 9);
            core.units.insert(
                trigger_unit_id,
                test_runtime_unit(
                    trigger_unit_id,
                    Side::Player,
                    trigger_base_uuid,
                    UnitStats::with_values(100, 100, 10, 0, 1000),
                ),
            );
            core.units.insert(
                counterpart_unit_id,
                test_runtime_unit(
                    counterpart_unit_id,
                    Side::Opponent,
                    counterpart_base_uuid,
                    UnitStats::with_values(100, 100, 1, 0, 1000),
                ),
            );
            if trigger_type == TriggerType::OnDeath {
                let trigger_unit = core.units.get_mut(&trigger_unit_id).unwrap();
                trigger_unit.lifecycle = RuntimeUnitLifecycle::Dead;
                trigger_unit.stats.current_health = 0;
            }
            core.items.insert(
                item_instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id: item_instance_id,
                    owner: Side::Player,
                    owner_unit_instance: trigger_unit_id,
                    base_uuid: item_base_uuid,
                },
            );

            let target_id = target_id_present.then_some(counterpart_unit_id);
            let plan = core.dispatch_item_trigger_occurrence(
                crate::game::battle::core::triggers::TriggerOccurrenceContext {
                    trigger_type,
                    trigger_unit_id,
                    counterpart_unit_id: Some(counterpart_unit_id),
                    target_id,
                    occurrence_id,
                    occurrence_index: 7,
                    allow_dead_caster,
                    committed_hit: None,
                    time_ms: 99,
                },
            );
            assert!(plan.current_hit_effects.is_empty());
            let commands = plan.into_commands();
            assert_eq!(commands.len(), 2, "case {trigger_type:?}");
            let expected_effect_target = if effect_target == TriggerEffectTarget::CounterpartUnit {
                counterpart_unit_id
            } else {
                trigger_unit_id
            };
            assert!(matches!(
                &commands[0],
                BattleCommand::ApplyBuff {
                    caster_id,
                    target_id,
                    ..
                } if *caster_id == trigger_unit_id && *target_id == expected_effect_target
            ));
            assert!(matches!(
                &commands[1],
                BattleCommand::TriggerAbility {
                    caster_id,
                    target_id: command_target_id,
                    proc_roll_identity,
                    allow_dead_caster: command_allow_dead_caster,
                    ..
                } if *caster_id == trigger_unit_id
                    && *command_target_id == target_id
                    && proc_roll_identity.trigger_type == trigger_type
                    && proc_roll_identity.occurrence_id == occurrence_id
                    && *command_allow_dead_caster == allow_dead_caster
            ));
        }
    }

    #[test]
    fn on_hit_retaliation_plan_skips_dead_or_missing_attacker_without_amplification() {
        let attacker_base_uuid = Uuid::from_u128(0x710);
        let defender_base_uuid = Uuid::from_u128(0x720);
        let item_base_uuid = Uuid::from_u128(0x730);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x711));
        let defender_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x721));
        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::BonusDamage {
                    flat: 8,
                    percent: 0,
                    damage_type: DamageType::Physical,
                },
            )],
        );
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            defender_base_uuid,
            item_base_uuid,
            item_effects,
            vec![],
            vec![],
        );
        let mut core = new_test_core(game_data, 12);
        core.units.insert(
            attacker_id,
            test_runtime_unit(
                attacker_id,
                Side::Player,
                attacker_base_uuid,
                UnitStats::with_values(100, 100, 10, 0, 1000),
            ),
        );
        core.units.insert(
            defender_id,
            test_runtime_unit(
                defender_id,
                Side::Opponent,
                defender_base_uuid,
                UnitStats::with_values(100, 100, 1, 0, 1000),
            ),
        );
        core.items.insert(
            Uuid::from_u128(0x731),
            crate::game::battle::core::RuntimeItem {
                persistent_copy_uuid: None,
                instance_id: Uuid::from_u128(0x731),
                owner: Side::Opponent,
                owner_unit_instance: defender_id,
                base_uuid: item_base_uuid,
            },
        );
        let committed_hit = test_damage_source_snapshot(
            attacker_id,
            defender_id,
            DamageSource::BasicAttack,
            DamageType::Physical,
            10,
            0,
        );

        core.units.get_mut(&attacker_id).unwrap().lifecycle = RuntimeUnitLifecycle::Dead;
        core.units
            .get_mut(&attacker_id)
            .unwrap()
            .stats
            .current_health = 0;
        for attacker_presence in ["dead", "missing"] {
            let plan = core.dispatch_item_trigger_occurrence(
                crate::game::battle::core::triggers::TriggerOccurrenceContext {
                    trigger_type: TriggerType::OnHit,
                    trigger_unit_id: defender_id,
                    counterpart_unit_id: Some(attacker_id),
                    target_id: Some(attacker_id),
                    occurrence_id: Uuid::from_u128(0x732),
                    occurrence_index: 0,
                    allow_dead_caster: false,
                    committed_hit: Some(&committed_hit),
                    time_ms: 1,
                },
            );
            assert!(plan.current_hit_effects.is_empty(), "{attacker_presence}");
            assert!(plan.effect_batches.is_empty(), "{attacker_presence}");
            assert!(plan.activation_commands.is_empty(), "{attacker_presence}");
            core.units.remove(&attacker_id);
        }
    }

    #[test]
    fn death_item_occurrences_execute_dead_caster_and_sorted_ally_plans() {
        let killer_base_uuid = Uuid::from_u128(0x410);
        let victim_base_uuid = Uuid::from_u128(0x420);
        let item_base_uuid = Uuid::from_u128(0x430);
        let killer_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x510));
        let victim_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x520));
        let first_ally_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x521));
        let second_ally_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x522));

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnDeath,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::Heal {
                    flat: 1,
                    percent: 0,
                },
            )],
        );
        item_effects.insert(
            TriggerType::OnKill,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::Modifier(crate::game::stats::StatModifier {
                    stat: crate::game::stats::StatId::Attack,
                    kind: crate::game::stats::StatModifierKind::Flat,
                    value: 1,
                }),
            )],
        );
        item_effects.insert(
            TriggerType::OnAllyDeath,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::Modifier(crate::game::stats::StatModifier {
                    stat: crate::game::stats::StatId::Attack,
                    kind: crate::game::stats::StatModifierKind::Flat,
                    value: 2,
                }),
            )],
        );
        let activations = [
            (TriggerType::OnDeath, "death_proc"),
            (TriggerType::OnKill, "kill_proc"),
            (TriggerType::OnAllyDeath, "ally_death_proc"),
        ]
        .into_iter()
        .map(|(trigger, ability_id)| AbilityActivationBinding {
            ability_id: SkillId::from(ability_id),
            activation: AbilityActivationDef::TriggerProc {
                trigger,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: None,
            },
        })
        .collect::<Vec<_>>();
        let game_data = battle_test_game_data(
            killer_base_uuid,
            victim_base_uuid,
            item_base_uuid,
            item_effects,
            activations,
            vec![
                self_heal_proc_skill("death_proc"),
                self_heal_proc_skill("kill_proc"),
                self_heal_proc_skill("ally_death_proc"),
            ],
        );
        let mut core = new_test_core(game_data, 11);
        for (unit_id, owner, base_uuid, attack) in [
            (killer_id, Side::Player, killer_base_uuid, 10),
            (victim_id, Side::Opponent, victim_base_uuid, 1),
            (first_ally_id, Side::Opponent, victim_base_uuid, 10),
            (second_ally_id, Side::Opponent, victim_base_uuid, 20),
        ] {
            core.units.insert(
                unit_id,
                test_runtime_unit(
                    unit_id,
                    owner,
                    base_uuid,
                    UnitStats::with_values(100, 100, attack, 0, 1000),
                ),
            );
        }
        for (index, (owner, owner_unit_instance)) in [
            (Side::Player, killer_id),
            (Side::Opponent, victim_id),
            (Side::Opponent, first_ally_id),
            (Side::Opponent, second_ally_id),
        ]
        .into_iter()
        .enumerate()
        {
            let instance_id = Uuid::from_u128(0x600 + index as u128);
            core.items.insert(
                instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id,
                    owner,
                    owner_unit_instance,
                    base_uuid: item_base_uuid,
                },
            );
        }

        let summary = core.process_commands(
            vec![
                BattleCommand::ApplyDamage {
                    target_id: victim_id,
                    source_snapshot: test_damage_source_snapshot(
                        killer_id,
                        victim_id,
                        DamageSource::Ability,
                        DamageType::True,
                        100,
                        10,
                    ),
                },
                // A second lethal input for the now-dead target must not repeat finalization
                // or any OnDeath/OnKill/OnAllyDeath occurrence.
                BattleCommand::ApplyDamage {
                    target_id: victim_id,
                    source_snapshot: test_damage_source_snapshot(
                        killer_id,
                        victim_id,
                        DamageSource::Ability,
                        DamageType::True,
                        100,
                        10,
                    ),
                },
            ],
            10,
        );

        assert_eq!(summary.actual_damage_target_count, 1);
        assert_eq!(summary.killed_target_count, 1);
        assert!(matches!(
            core.units[&victim_id].lifecycle,
            RuntimeUnitLifecycle::Dead
        ));
        assert!(core.graveyard.contains_key(&victim_id));
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::UnitDied {
                        unit_instance_id,
                        ..
                    } if unit_instance_id == victim_id
                ))
                .count(),
            1
        );
        assert_eq!(core.units[&killer_id].stats.attack, 11);
        assert_eq!(core.units[&first_ally_id].stats.attack, 12);
        assert_eq!(core.units[&second_ally_id].stats.attack, 22);

        let casts = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match &entry.event {
                BattleLogEvent::AbilityCast {
                    skill_id,
                    caster_instance_id,
                    ..
                } if matches!(
                    skill_id.as_str(),
                    "death_proc" | "kill_proc" | "ally_death_proc"
                ) =>
                {
                    Some((skill_id.as_str().to_string(), *caster_instance_id))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            casts,
            vec![
                ("death_proc".to_string(), victim_id),
                ("kill_proc".to_string(), killer_id),
                ("ally_death_proc".to_string(), first_ally_id),
                ("ally_death_proc".to_string(), second_ally_id),
            ]
        );

        let ally_effect_targets = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::StatChanged {
                    target_instance_id,
                    modifier,
                    ..
                } if modifier.value == 2 => Some(target_instance_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ally_effect_targets, vec![first_ally_id, second_ally_id]);
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
                persistent_copy_uuid: None,
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
    fn item_on_hit_counterpart_bonus_damage_retaliates_once_without_reflection_recursion() {
        let attacker_base_uuid = Uuid::from_u128(0x51);
        let target_base_uuid = Uuid::from_u128(0x52);
        let item_base_uuid = Uuid::from_u128(0x53);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(51));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(52));

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnHit,
            vec![
                TriggeredEffect::targeted(
                    TriggerEffectTarget::CounterpartUnit,
                    crate::game::stats::Effect::BonusDamage {
                        flat: 8,
                        percent: 0,
                        damage_type: DamageType::Physical,
                    },
                ),
                TriggeredEffect::targeted(
                    TriggerEffectTarget::CounterpartUnit,
                    crate::game::stats::Effect::BonusDamage {
                        flat: 0,
                        percent: 50,
                        damage_type: DamageType::Magic,
                    },
                ),
            ],
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
        for (item_instance_id, owner, owner_unit_instance) in [
            (Uuid::from_u128(0x531), Side::Player, attacker_id),
            (Uuid::from_u128(0x532), Side::Opponent, target_id),
        ] {
            core.items.insert(
                item_instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id: item_instance_id,
                    owner,
                    owner_unit_instance,
                    base_uuid: item_base_uuid,
                },
            );
        }
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, target_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
        drain_event_queue(&mut core);

        assert_eq!(core.units[&target_id].stats.current_health, 90);
        assert_eq!(core.units[&attacker_id].stats.current_health, 87);
        let damage_events = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    delta,
                    ..
                } if delta < 0 => Some((source_instance_id, target_instance_id, delta)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            damage_events,
            vec![
                (Some(attacker_id), target_id, -10),
                (Some(target_id), attacker_id, -8),
                (Some(target_id), attacker_id, -5),
            ],
            "retaliation must be attributed to the effect owner and must not trigger a return reflection"
        );
    }

    #[test]
    fn fatal_hit_releases_owner_attributed_retaliation_before_the_defender_dies() {
        let attacker_base_uuid = Uuid::from_u128(0x91);
        let defender_base_uuid = Uuid::from_u128(0x92);
        let item_base_uuid = Uuid::from_u128(0x93);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x911));
        let defender_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0x921));

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::BonusDamage {
                    flat: 8,
                    percent: 0,
                    damage_type: DamageType::Physical,
                },
            )],
        );
        let game_data = battle_test_game_data(
            attacker_base_uuid,
            defender_base_uuid,
            item_base_uuid,
            item_effects,
            vec![],
            vec![],
        );
        let mut core = new_test_core(game_data, 14);
        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            attacker_base_uuid,
            UnitStats::with_values(100, 100, 10, 0, 1000),
        );
        attacker.stats.move_speed_units_per_ms = 1;
        let mut defender = test_runtime_unit(
            defender_id,
            Side::Opponent,
            defender_base_uuid,
            UnitStats::with_values(10, 10, 1, 0, 1000),
        );
        defender.stats.move_speed_units_per_ms = 1;
        core.units.insert(attacker_id, attacker);
        core.units.insert(defender_id, defender);
        core.items.insert(
            Uuid::from_u128(0x931),
            crate::game::battle::core::RuntimeItem {
                persistent_copy_uuid: None,
                instance_id: Uuid::from_u128(0x931),
                owner: Side::Opponent,
                owner_unit_instance: defender_id,
                base_uuid: item_base_uuid,
            },
        );
        place_test_unit(&mut core, attacker_id, Position::new(0, 0));
        place_test_unit(&mut core, defender_id, Position::new(1, 0));

        assert!(core.resolve_basic_attack(attacker_id, defender_id, 0));
        drain_event_queue(&mut core);

        assert!(matches!(
            core.units[&defender_id].lifecycle,
            RuntimeUnitLifecycle::Dead
        ));
        assert_eq!(core.units[&attacker_id].stats.current_health, 92);
        let damage_events = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    delta,
                    ..
                } if delta < 0 => Some((source_instance_id, target_instance_id, delta)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            damage_events,
            vec![
                (Some(attacker_id), defender_id, -10),
                (Some(defender_id), attacker_id, -8),
            ]
        );
    }

    #[test]
    fn projectile_on_hit_counterpart_bonus_damage_uses_the_same_non_recursive_retaliation() {
        let attacker_base_uuid = Uuid::from_u128(0x61);
        let target_base_uuid = Uuid::from_u128(0x62);
        let item_base_uuid = Uuid::from_u128(0x63);
        let attacker_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(61));
        let target_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(62));
        let projectile_id = Uuid::from_u128(0x630);

        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                crate::game::stats::Effect::BonusDamage {
                    flat: 8,
                    percent: 0,
                    damage_type: DamageType::Physical,
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
        let mut attacker = test_runtime_unit(
            attacker_id,
            Side::Player,
            attacker_base_uuid,
            UnitStats::with_values(100, 100, 10, 0, 1000),
        );
        attacker.body = UnitBody::new_at(WorldVec2::ZERO, 0.10, 1.0);
        let mut target = test_runtime_unit(
            target_id,
            Side::Opponent,
            target_base_uuid,
            UnitStats::with_values(100, 100, 1, 0, 1000),
        );
        target.body = UnitBody::new_at(WorldVec2::new(1.0, 0.0), 0.10, 1.0);
        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);
        for (item_instance_id, owner, owner_unit_instance) in [
            (Uuid::from_u128(0x631), Side::Player, attacker_id),
            (Uuid::from_u128(0x632), Side::Opponent, target_id),
        ] {
            core.items.insert(
                item_instance_id,
                crate::game::battle::core::RuntimeItem {
                    persistent_copy_uuid: None,
                    instance_id: item_instance_id,
                    owner,
                    owner_unit_instance,
                    base_uuid: item_base_uuid,
                },
            );
        }
        core.projectiles.insert(
            projectile_id,
            ProjectileRecord {
                spatial: None,
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

        assert_eq!(core.units[&target_id].stats.current_health, 90);
        assert_eq!(core.units[&attacker_id].stats.current_health, 92);
        let damage_events = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    delta,
                    ..
                } if delta < 0 => Some((source_instance_id, target_instance_id, delta)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            damage_events,
            vec![
                (Some(attacker_id), target_id, -10),
                (Some(target_id), attacker_id, -8),
            ]
        );
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
                persistent_copy_uuid: None,
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
                persistent_copy_uuid: None,
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

    #[test]
    fn runtime_facts_group_effect_major_damage_by_actual_iteration_and_target() {
        let mut skill = damage_proc_skill("fact_strike", 17);
        skill.steps[0].effects.push(
            crate::game::ability::SkillEffectDef::Damage {
                amount: 23,
                damage_type: crate::game::battle::damage::DamageType::True,
            }
            .authored("second"),
        );
        let (mut core, caster, target) = authoring_control_core(skill.clone());
        assert!(core.authoring_observation.is_none());
        for repeat in 0..2 {
            let (commands, _) = core
                .build_skill_step_commands(caster, &skill.steps[0], &[target], repeat, None)
                .unwrap();
            let old = core.process_skill_commands(commands, repeat, caster, None);
            assert_eq!(
                old.actual_damage_target_count, 2,
                "legacy gate counts remain unchanged"
            );
            let facts = core.runtime_fact_summary();
            assert_eq!(facts.damage_mutations, (repeat + 1) * 2);
            assert_eq!(facts.damage_strikes, repeat + 1);
            assert_eq!(facts.damaging_strikes, repeat + 1);
            assert_eq!(facts.damage_hp_lost, (repeat + 1) * 40);
            let last = facts.latest_damage.as_ref().unwrap();
            assert_eq!(last.segments.len(), 1);
            assert_eq!(last.segments[0].ordinal, 1);
            assert_eq!(last.strike.0, repeat);
            assert_eq!(last.actual_hp_loss(), 23);
            assert_eq!(last.source.actor, Some(caster));
            assert!(last.source.cast.is_none());
        }
        assert!(core.runtime_strike_scope.is_none());
        let before = serde_json::to_value(core.runtime_fact_summary()).unwrap();
        core.event_log.entries.clear();
        assert_eq!(
            serde_json::to_value(core.runtime_fact_summary()).unwrap(),
            before
        );
        assert_eq!(
            core.actor_fact_summary(caster)
                .unwrap()
                .caused
                .damage_strikes,
            2
        );
        assert_eq!(
            core.actor_fact_summary(target)
                .unwrap()
                .received
                .damage_hp_lost,
            80
        );
    }

    #[test]
    fn runtime_facts_keep_overheal_zero_signed_heal_and_predeath_sources_separate() {
        use crate::game::battle::core::types::{ActiveBuff, BuffInstanceKey};
        use crate::game::battle::runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource};
        let (mut core, caster, target) = authoring_control_core(damage_proc_skill("fact_heal", 1));
        core.apply_hp_delta_and_record(Some(caster), target, 500, 0, HpChangeReason::Command);
        core.apply_hp_delta_and_record(Some(caster), target, 0, 1, HpChangeReason::Command);
        assert_eq!(core.runtime_fact_summary().heal_mutations, 2);
        assert_eq!(core.runtime_fact_summary().healing_delta, 0);
        assert!(core.runtime_fact_summary().latest_nonzero_heal.is_none());
        assert_eq!(
            core.runtime_fact_summary()
                .latest_heal
                .as_ref()
                .unwrap()
                .requested_delta,
            0
        );
        let provider = |value| EquipmentSourceSnapshot {
            definition_uuid: Uuid::from_u128(91),
            runtime_item_instance_id: Uuid::from_u128(value),
            persistent_copy_uuid: Some(Uuid::from_u128(value + 100)),
        };
        let buff_provider = provider(10);
        let killer_provider = provider(20);
        core.buffs.insert(
            BuffInstanceKey {
                caster_instance_id: target,
                target_instance_id: target,
                buff_id: crate::game::battle::buffs::BuffId::from_name("silence"),
            },
            ActiveBuff {
                latest_application_source: RuntimeFactSource {
                    actor: Some(target),
                    equipment: Some(buff_provider),
                    ..Default::default()
                },
                stacks: 1,
                expires_at_ms: 10,
                next_tick_ms: None,
            },
        );
        core.runtime_fact_source = RuntimeFactSource {
            actor: Some(caster),
            equipment: Some(killer_provider),
            ..Default::default()
        };
        core.apply_hp_delta_and_record(Some(caster), target, -2000, 10, HpChangeReason::Command);
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.heal_mutations, 3);
        assert_eq!(facts.healing_delta, -1000);
        assert_eq!(facts.damage_mutations, 0);
        assert_eq!(facts.cost_mutations, 0);
        let death = facts.latest_death.as_ref().unwrap();
        assert_eq!(death.predeath.hp_before_lethal, 1000);
        assert_eq!(
            death.predeath.buffs.len(),
            1,
            "expiry equality remains active until expiration dispatch"
        );
        assert_eq!(
            death.predeath.buffs[0].latest_application_source.equipment,
            Some(buff_provider)
        );
        assert_eq!(death.source.equipment, Some(killer_provider));
        assert!(death.lethal_damage.is_none());
        assert!(core.buffs.is_empty());
        core.finalize_unit_death(Some(caster), target, Side::Opponent, 11);
        assert_eq!(core.runtime_fact_summary().deaths, 1);
    }

    #[test]
    fn runtime_facts_resource_modifiers_do_not_become_payments() {
        let (mut core, caster, target) =
            authoring_control_core(damage_proc_skill("fact_resource", 1));
        core.runtime_fact_source.actor = Some(caster);
        core.units.get_mut(&target).unwrap().resonance_current = 30;
        core.process_commands(
            vec![BattleCommand::ModifyResonance {
                target_id: target,
                amount: -100,
                allow_autocast_when_full: false,
            }],
            1,
        );
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.cost_mutations, 0);
        assert_eq!(facts.resource_spent, 0);
        assert_eq!(facts.resource_change_mutations, 1);
        assert_eq!(facts.resource_change_delta, -30);
        let change = facts.latest_resource_change.as_ref().unwrap();
        assert_eq!(
            (change.requested_delta, change.before, change.after),
            (-100, 30, 0)
        );
        assert_eq!(change.source.actor, Some(caster));
        assert!(facts.latest_cost.is_none());
    }

    #[test]
    fn runtime_facts_lethal_damage_retains_combined_hp_transition() {
        let skill = damage_proc_skill("fact_overkill", 2000);
        let (mut core, caster, target) = authoring_control_core(skill.clone());
        let (commands, _) = core
            .build_skill_step_commands(caster, &skill.steps[0], &[target], 2, None)
            .unwrap();
        core.process_skill_commands(commands, 2, caster, None);
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.damage_mutations, 1);
        assert_eq!(facts.damage_hp_lost, 1000);
        assert_eq!(facts.deaths, 1);
        let damage = facts.latest_damage.as_ref().unwrap();
        assert_eq!(damage.requested_damage, 2000);
        assert_eq!(damage.actual_hp_loss(), 1000);
        assert_eq!(damage.segments[0].calculated_damage, 2000);
        let death = facts.latest_death.as_ref().unwrap();
        assert_eq!(death.predeath.hp_before_lethal, 1000);
        assert!(Arc::ptr_eq(damage, death.lethal_damage.as_ref().unwrap()));
    }
}
