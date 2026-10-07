use crate::authoring::control;
use crate::game::{
    ability::{
        DeliveryDef, ProjectileHitPolicy, SkillDef, SkillEffectDef, SkillHitTargetFilter,
        SkillStepDef, SkillTarget, StepTargetingMode,
    },
    battle::{
        cooldown::SourcedEffect,
        damage::{calculate_damage, DamageContext, DamageModifiers, DamageRequest, DamageSource},
        event_log::SkillCastTarget,
        ids::UnitInstanceId,
    },
    enums::Side,
    stats::{Effect, StatId, StatModifier, TriggerEffectTarget, TriggerType},
};

use super::BattleCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FutureDamageUsefulness {
    Known(u32),
    Indeterminate { expression_effects: usize },
}

impl BattleCore {
    pub(in crate::game::battle::core) fn is_basic_attack_useful_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> bool {
        if control::charge(1, "basic_target_usefulness").is_err() {
            return false;
        }
        self.preview_basic_attack_final_damage(attacker_instance_id, target_id)
            .is_some_and(|damage| damage > 0)
            || self.basic_attack_has_hostile_target_effect(attacker_instance_id, target_id)
    }

    pub(in crate::game::battle::core) fn is_skill_step_useful_ally_target(
        &self,
        caster: UnitInstanceId,
        step: &SkillStepDef,
        target: UnitInstanceId,
    ) -> bool {
        let Some(source) = self.units.get(&caster) else {
            return false;
        };
        let Some(unit) = self.units.get(&target) else {
            return false;
        };
        if !unit.is_active() || unit.owner != source.owner {
            return false;
        }
        step.effects.iter().any(|effect| match &effect.op {
            SkillEffectDef::Heal { amount } => {
                *amount > 0
                    && unit.role != crate::game::battle::types::BattleUnitRole::IndependentWall
                    && unit.stats.current_health < unit.stats.max_health
            }
            SkillEffectDef::HealExpr { .. } => {
                unit.role != crate::game::battle::types::BattleUnitRole::IndependentWall
                    && unit.stats.current_health < unit.stats.max_health
            }
            SkillEffectDef::Damage { .. }
            | SkillEffectDef::DamageExpr { .. }
            | SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Pull { .. }
            | SkillEffectDef::InterruptCast
            | SkillEffectDef::ExtraAttack { .. } => false,
            _ => true,
        })
    }

    pub(in crate::game::battle::core) fn is_skill_step_useful_hostile_target(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        if control::poll("skill_target_usefulness").is_err() {
            return false;
        }
        let Some((caster_owner, target_owner)) =
            self.caster_and_target_owner(caster_instance_id, target_id)
        else {
            return false;
        };
        if target_owner == caster_owner {
            return false;
        }
        self.preview_skill_step_future_damage(caster_instance_id, step, target_id)
            .is_some_and(|damage| match damage {
                FutureDamageUsefulness::Known(value) => value > 0,
                // A future occurrence has no selected release/consume frame.
                // Eligibility remains true without borrowing a past record or
                // folding only the legacy portion of a mixed step.
                FutureDamageUsefulness::Indeterminate { .. } => true,
            })
            || Self::skill_step_has_hostile_target_applied_effect(step)
            || self.skill_step_has_interruptible_cast_effect(step, target_id)
            || self.skill_step_schedules_useful_extra_attack(caster_instance_id, step, target_id)
    }

    pub(in crate::game::battle::core) fn skill_step_has_useful_hostile_target(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        targets: &[UnitInstanceId],
    ) -> bool {
        if control::charge(targets.len() as u64 + 1, "skill_useful_targets").is_err() {
            return false;
        }
        targets.iter().copied().any(|target_id| {
            self.is_skill_step_useful_hostile_target(caster_instance_id, step, target_id)
        })
    }

    pub(in crate::game::battle::core) fn skill_step_needs_hostile_usefulness_gate(
        &self,
        _caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        hit_targets: SkillHitTargetFilter,
        _targets: &[UnitInstanceId],
    ) -> bool {
        !matches!(hit_targets, SkillHitTargetFilter::Allies)
            && Self::skill_step_has_hostile_payload(step)
    }

    pub(in crate::game::battle::core) fn skill_step_requires_hostile_usefulness(
        step: &SkillStepDef,
    ) -> bool {
        Self::skill_step_has_hostile_payload(step)
    }

    pub(in crate::game::battle::core) fn skill_step_has_useful_actual_delivery_target(
        &self,
        time_ms: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        step_target: Option<SkillCastTarget>,
    ) -> bool {
        if !Self::skill_step_has_hostile_payload(step) {
            return false;
        }
        if let Some(useful) = self
            .effective_footprint_usefulness(
                time_ms,
                step_index,
                caster_instance_id,
                step,
                step_target,
            )
            .unwrap_or(Some(false))
        {
            return useful;
        }

        if matches!(
            &step.delivery,
            DeliveryDef::Projectile {
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: true,
                max_range_tiles: Some(_),
                ..
            } | DeliveryDef::Projectile {
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: true,
                max_lifetime_ms: Some(_),
                ..
            }
        ) && self
            .units
            .get(&caster_instance_id)
            .and_then(|unit| unit.facing_direction)
            .is_some()
        {
            return true;
        }

        if let DeliveryDef::TileArea { footprint, area } = &step.delivery {
            let Some((caster_owner, _, _, _, anchor_tile)) = self.resolve_tile_area_geometry(
                time_ms,
                0,
                step_index,
                caster_instance_id,
                step_target,
                area,
            ) else {
                return false;
            };
            let Some(facing) = self.tile_area_projection_facing(caster_instance_id, footprint)
            else {
                return false;
            };
            let Some(affected_tiles) = self.valid_tile_area_tiles(footprint, anchor_tile, facing)
            else {
                return false;
            };
            let targets = self.collect_tile_area_targets_at(
                time_ms,
                caster_owner,
                caster_instance_id,
                &affected_tiles,
                area.hit_targets,
                area.include_caster,
            );
            if self.skill_step_needs_hostile_usefulness_gate(
                caster_instance_id,
                step,
                area.hit_targets,
                &targets,
            ) {
                return self.skill_step_has_useful_hostile_target(
                    caster_instance_id,
                    step,
                    &targets,
                );
            }
            return !targets.is_empty();
        }

        let Some(SkillCastTarget::Unit { unit_instance_id }) = step_target else {
            return false;
        };
        self.is_skill_step_useful_hostile_target(caster_instance_id, step, unit_instance_id)
    }

    /// New footprint rows resolve independently. Untransformed sibling payloads
    /// continue through their original delivery/eligibility path.
    pub(in crate::game::battle::core) fn effective_footprint_usefulness(
        &self,
        time_ms: u64,
        step_index: usize,
        caster: UnitInstanceId,
        step: &SkillStepDef,
        target: Option<SkillCastTarget>,
    ) -> Result<Option<bool>, crate::game::behavior::GameError> {
        use crate::game::relic::structural::authoring::{RelicProperty, StructuralRecipeDef};
        let source = &self.runtime_fact_source;
        if source.relic_effective().is_none() && self.effective_owner.is_none() {
            return Ok(None);
        }
        let mut transformed = false;
        let mut other_hostile = false;
        for effect in &step.effects {
            control::charge(1, "effective_footprint_preview_row")?;
            let Some(site) = effect.source().map(|source| &source.site) else {
                continue;
            };
            let descriptor = self.effective_effect_view(site, caster, source)?;
            let pattern = descriptor.and_then(|descriptor| {
                descriptor.recipes.iter().find_map(|recipe| match recipe {
                    StructuralRecipeDef::DamageFootprint { footprint } => Some(footprint),
                    _ => None,
                })
            });
            let Some(pattern) = pattern else {
                other_hostile |= matches!(
                    effect.op,
                    SkillEffectDef::Damage { .. }
                        | SkillEffectDef::DamageExpr { .. }
                        | SkillEffectDef::Pull { .. }
                        | SkillEffectDef::InterruptCast
                        | SkillEffectDef::ExtraAttack { .. }
                );
                continue;
            };
            transformed = true;
            let unit = self
                .units
                .get(&caster)
                .ok_or(crate::game::behavior::GameError::UnitNotFound)?;
            let (anchor, filter, include) = match &step.delivery {
                DeliveryDef::Instant => (
                    self.sample_unit_world_position_at(caster, time_ms)
                        .map_or(unit.body.projected_tile(), |position| {
                            position.project_to_tile()
                        }),
                    SkillHitTargetFilter::Enemies,
                    false,
                ),
                DeliveryDef::TileArea { area, .. } => {
                    let Some((_, _, _, _, tile)) = self.resolve_tile_area_geometry(
                        time_ms,
                        source.cast.as_ref().map_or(0, |cast| cast.cast_seq),
                        step_index,
                        caster,
                        target,
                        area,
                    ) else {
                        continue;
                    };
                    (tile, area.hit_targets, area.include_caster)
                }
                DeliveryDef::Projectile { .. } => {
                    return Err(crate::game::behavior::GameError::InvalidAction)
                }
            };
            let Some(facing) = self.tile_area_projection_facing(caster, pattern) else {
                continue;
            };
            let Some(tiles) = self.valid_tile_area_tiles(pattern, anchor, facing) else {
                continue;
            };
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.units
                    .len()
                    .checked_mul(std::mem::size_of::<UnitInstanceId>()),
                "effective_footprint_preview_targets",
            )?;
            let targets = self
                .collect_tile_area_targets_at(time_ms, unit.owner, caster, &tiles, filter, include);
            control::poll("effective_footprint_preview")?;
            for target in targets {
                control::charge(1, "effective_footprint_preview_damage")?;
                let Some(victim) = self.units.get(&target) else {
                    continue;
                };
                if victim.owner == unit.owner || !victim.is_active() {
                    continue;
                }
                let (base_damage, damage_type) = match effect.op {
                    SkillEffectDef::DamageExpr { .. } => return Ok(Some(true)),
                    SkillEffectDef::Damage {
                        amount,
                        damage_type,
                    } => {
                        let base = descriptor
                            .and_then(|descriptor| {
                                descriptor.properties.iter().find(|property| {
                                    property.property == RelicProperty::DamageAmount
                                })
                            })
                            .and_then(|property| property.final_value)
                            .and_then(|value| u32::try_from(value.value()).ok())
                            .unwrap_or(amount.unsigned_abs());
                        (base, damage_type)
                    }
                    _ => continue,
                };
                let context = DamageContext {
                    attacker_side: unit.owner,
                    target_side: victim.owner,
                    attacker_attack: unit.stats.attack,
                    target_armor: victim.stats.defense,
                    target_magic_resist: victim.stats.magic_resist,
                    target_incoming_modifiers: victim.incoming_damage_modifiers,
                    target_current_hp: victim.stats.current_health,
                    target_max_hp: victim.stats.max_health,
                    on_attack_effects: &[],
                    on_hit_effects: &[],
                };
                let request = DamageRequest {
                    source: DamageSource::Ability,
                    damage_type,
                    modifiers: Self::skill_step_damage_modifiers(step),
                    crit_roll_percent: None,
                    attacker_id: caster,
                    target_id: target,
                    base_damage,
                    minimum_damage: 0,
                    time_ms,
                };
                if calculate_damage(&request, &context).final_damage > 0 {
                    return Ok(Some(true));
                }
            }
        }
        Ok(if transformed && !other_hostile {
            Some(false)
        } else {
            None
        })
    }

    pub(in crate::game::battle::core) fn skill_has_useful_automatic_cast_target(
        &self,
        time_ms: u64,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        cast_target: Option<SkillCastTarget>,
    ) -> bool {
        if control::charge(skill.steps.len() as u64 + 1, "skill_useful_steps").is_err() {
            return false;
        }
        if self
            .validate_skill_object_placements(time_ms, caster_instance_id, skill, cast_target)
            .is_err()
        {
            return false;
        }
        if self.channel_active(caster_instance_id)
            || self
                .validate_sustained_start(caster_instance_id, skill)
                .is_err()
        {
            return false;
        }
        if matches!(
            skill.cast_target_definition().map(|value| value.0),
            Some(SkillTarget::AllySingle)
        ) {
            let Some(SkillCastTarget::Unit { unit_instance_id }) = cast_target else {
                return false;
            };
            if !skill.steps.iter().any(|step| {
                matches!(step.target, SkillTarget::AllySingle)
                    && self.is_skill_step_useful_ally_target(
                        caster_instance_id,
                        step,
                        unit_instance_id,
                    )
            }) {
                return false;
            }
        }
        let mut has_hostile_payload = false;
        let mut has_hostile_targeting_step = false;

        for (step_index, step) in skill.steps.iter().enumerate() {
            if control::poll("skill_useful_step").is_err() {
                return false;
            }
            if Self::skill_step_targets_enemies(step) {
                has_hostile_targeting_step = true;
            }
            if !Self::skill_step_has_hostile_payload(step) {
                continue;
            }
            has_hostile_payload = true;

            let step_target = match step.targeting {
                StepTargetingMode::ReuseCastTarget => match step.target {
                    SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                        unit_instance_id: caster_instance_id,
                    }),
                    SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle | SkillTarget::CastTarget => cast_target,
                },
                StepTargetingMode::RetargetOnStep => match &step.target {
                    SkillTarget::SelfUnit => Some(SkillCastTarget::Unit {
                        unit_instance_id: caster_instance_id,
                    }),
                    SkillTarget::EnemySingle { rule } => self
                        .resolve_cast_origin_context(None, caster_instance_id, false)
                        .and_then(|(caster_owner, caster_pos)| {
                            let (_, reach, target_range, air_capable) =
                                skill.cast_target_definition()?;
                            self.choose_skill_target_by_rule(
                                caster_instance_id,
                                caster_owner,
                                caster_pos,
                                match reach {
                                    crate::game::ability::SkillTargetReach::Pattern => {
                                        crate::game::battle::tile_range::TileRangePolicy::Pattern
                                    }
                                    crate::game::ability::SkillTargetReach::Global => {
                                        crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles
                                    }
                                },
                                target_range,
                                *rule,
                                air_capable,
                                super::skill_runtime::cast::SkillTargetUsefulness::DeliveryStep {
                                    time_ms,
                                    step_index,
                                    step,
                                },
                            )
                            .map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id })
                        }),
                    SkillTarget::AllySingle => {
                        let (_, reach, range, air_capable) = skill.cast_target_definition().expect("validated skill targeting");
                        self.choose_ally_skill_target(caster_instance_id, reach, range, air_capable,
                            super::skill_runtime::cast::SkillTargetUsefulness::DeliveryStep { time_ms, step_index, step })
                            .map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id })
                    }
                    SkillTarget::CastTarget => None,
                },
            };

            if self.skill_step_has_useful_actual_delivery_target(
                time_ms,
                step_index,
                caster_instance_id,
                step,
                step_target,
            ) {
                return true;
            }
        }

        control::poll("skill_usefulness_result").is_ok()
            && !has_hostile_payload
            && !has_hostile_targeting_step
    }

    fn preview_basic_attack_final_damage(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> Option<u32> {
        let attacker = self.units.get(&attacker_instance_id)?;
        let target = self.units.get(&target_id)?;
        if !attacker.is_active() || !target.is_active() || attacker.owner == target.owner {
            return None;
        }
        // Basic attacks enforce one point of minimum damage whenever their
        // positive base component exists. Trigger collection and a complete
        // damage breakdown are only needed for a zero-base attack whose
        // usefulness may come from bonus effects.
        if attacker.stats.attack > 0 {
            return Some(1);
        }

        let on_attack_effects =
            self.collect_current_hit_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_current_hit_triggers(target_id, TriggerType::OnHit);
        control::poll("basic_usefulness_triggers").ok()?;
        let ctx = DamageContext {
            attacker_side: attacker.owner,
            target_side: target.owner,
            attacker_attack: attacker.stats.attack,
            target_armor: target.stats.defense,
            target_magic_resist: target.stats.magic_resist,
            target_incoming_modifiers: target.incoming_damage_modifiers,
            target_current_hp: target.stats.current_health,
            target_max_hp: target.stats.max_health,
            on_attack_effects: &on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: attacker.basic_attack.damage_type,
            modifiers: DamageModifiers::default(),
            crit_roll_percent: None,
            attacker_id: attacker_instance_id,
            target_id,
            base_damage: attacker.stats.attack,
            minimum_damage: 1,
            time_ms: 0,
        };

        Some(calculate_damage(&request, &ctx).final_damage)
    }

    fn preview_skill_step_future_damage(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> Option<FutureDamageUsefulness> {
        control::charge(
            (step.effects.len() as u64)
                .saturating_mul(2)
                .saturating_add(1),
            "skill_usefulness_damage",
        )
        .ok()?;
        let caster = self.units.get(&caster_instance_id)?;
        let target = self.units.get(&target_id)?;
        if !caster.is_active() || !target.is_active() || caster.owner == target.owner {
            return None;
        }

        let expression_effects = step
            .effects
            .iter()
            .filter(|effect| matches!(effect.op, SkillEffectDef::DamageExpr { .. }))
            .count();
        if expression_effects != 0 {
            return Some(FutureDamageUsefulness::Indeterminate { expression_effects });
        }

        let modifiers = Self::skill_step_damage_modifiers(step);
        let ctx = DamageContext {
            attacker_side: caster.owner,
            target_side: target.owner,
            attacker_attack: caster.stats.attack,
            target_armor: target.stats.defense,
            target_magic_resist: target.stats.magic_resist,
            target_incoming_modifiers: target.incoming_damage_modifiers,
            target_current_hp: target.stats.current_health,
            target_max_hp: target.stats.max_health,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };

        let final_damage = step.effects.iter().fold(0_u32, |acc, effect| {
            let SkillEffectDef::Damage {
                amount,
                damage_type,
            } = &effect.op
            else {
                return acc;
            };
            if *amount == 0 {
                return acc;
            }
            let request = DamageRequest {
                source: DamageSource::Ability,
                damage_type: *damage_type,
                modifiers,
                crit_roll_percent: None,
                attacker_id: caster_instance_id,
                target_id,
                base_damage: amount.unsigned_abs(),
                minimum_damage: 0,
                time_ms: 0,
            };
            acc.saturating_add(calculate_damage(&request, &ctx).final_damage)
        });

        Some(FutureDamageUsefulness::Known(final_damage))
    }

    fn caster_and_target_owner(
        &self,
        caster_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> Option<(Side, Side)> {
        Some((
            self.units.get(&caster_instance_id)?.owner,
            self.units.get(&target_id)?.owner,
        ))
    }

    fn basic_attack_has_hostile_target_effect(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> bool {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return false;
        };
        let Some(target) = self.units.get(&target_id) else {
            return false;
        };
        if !attacker.is_active() || !target.is_active() || attacker.owner == target.owner {
            return false;
        }

        self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack)
            .iter()
            .any(|effect| {
                Self::trigger_effect_can_apply_to_target(effect, target.owner, attacker.owner)
                    && Self::is_hostile_effect(&effect.effect)
            })
    }

    fn trigger_effect_can_apply_to_target(
        effect: &SourcedEffect,
        target_owner: Side,
        source_owner: Side,
    ) -> bool {
        match effect.target {
            TriggerEffectTarget::CounterpartUnit => target_owner != source_owner,
            TriggerEffectTarget::AllEnemies => target_owner != source_owner,
            TriggerEffectTarget::SelfUnit | TriggerEffectTarget::AllAllies => false,
        }
    }

    fn skill_step_damage_modifiers(step: &SkillStepDef) -> DamageModifiers {
        step.effects
            .iter()
            .filter_map(|effect| match &effect.op {
                SkillEffectDef::ModifyDamage { modifiers } => Some(*modifiers),
                _ => None,
            })
            .fold(DamageModifiers::default(), DamageModifiers::merge)
    }

    fn skill_step_has_hostile_payload(step: &SkillStepDef) -> bool {
        if control::charge(step.effects.len() as u64 + 1, "skill_hostile_payload").is_err() {
            return false;
        }
        step.effects.iter().any(|effect| match &effect.op {
            SkillEffectDef::Damage { amount, .. } => *amount != 0,
            SkillEffectDef::DamageExpr { .. } => true,
            SkillEffectDef::Sustained { definition } => definition.is_hostile(),
            SkillEffectDef::ModifyStats { modifier } => Self::is_hostile_stat_modifier(*modifier),
            SkillEffectDef::ApplyBuff { .. } | SkillEffectDef::Pull { .. } => true,
            SkillEffectDef::InterruptCast => true,
            SkillEffectDef::ExtraAttack { count } => *count > 0,
            SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Heal { .. }
            | SkillEffectDef::ModifyResonance { .. }
            | SkillEffectDef::Resources { .. }
            | SkillEffectDef::ModifyTacticalResource { .. } => false,
            SkillEffectDef::HealExpr { .. }
            | SkillEffectDef::PersonalShieldGrant { .. }
            | SkillEffectDef::ShieldExpr { .. }
            | SkillEffectDef::CreateObject { .. }
            | SkillEffectDef::HealthCost { .. }
            | SkillEffectDef::RechargeShield { .. } => false,
        })
    }

    fn skill_step_targets_enemies(step: &SkillStepDef) -> bool {
        matches!(step.target, SkillTarget::EnemySingle { .. })
            || matches!(
                &step.delivery,
                DeliveryDef::TileArea { area, .. }
                    if !matches!(area.hit_targets, SkillHitTargetFilter::Allies)
            )
    }

    fn skill_step_has_hostile_target_applied_effect(step: &SkillStepDef) -> bool {
        if control::charge(step.effects.len() as u64 + 1, "skill_hostile_effects").is_err() {
            return false;
        }
        step.effects.iter().any(|effect| match &effect.op {
            SkillEffectDef::Sustained { definition } => definition.is_hostile(),
            SkillEffectDef::ModifyStats { modifier } => Self::is_hostile_stat_modifier(*modifier),
            SkillEffectDef::ApplyBuff { .. } | SkillEffectDef::Pull { .. } => true,
            SkillEffectDef::Damage { .. }
            | SkillEffectDef::DamageExpr { .. }
            | SkillEffectDef::InterruptCast
            | SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Heal { .. }
            | SkillEffectDef::ModifyResonance { .. }
            | SkillEffectDef::ModifyTacticalResource { .. }
            | SkillEffectDef::Resources { .. }
            | SkillEffectDef::ExtraAttack { .. } => false,
            SkillEffectDef::HealExpr { .. }
            | SkillEffectDef::PersonalShieldGrant { .. }
            | SkillEffectDef::ShieldExpr { .. }
            | SkillEffectDef::CreateObject { .. }
            | SkillEffectDef::HealthCost { .. }
            | SkillEffectDef::RechargeShield { .. } => false,
        })
    }

    fn skill_step_schedules_useful_extra_attack(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        if control::charge(step.effects.len() as u64 + 1, "skill_useful_extra_attacks").is_err() {
            return false;
        }
        step.effects.iter().any(|effect| {
            matches!(&effect.op, SkillEffectDef::ExtraAttack { count } if *count > 0)
                && self.is_basic_attack_useful_target(caster_instance_id, target_id)
        })
    }

    fn skill_step_has_interruptible_cast_effect(
        &self,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        if control::charge(step.effects.len() as u64 + 1, "skill_useful_interrupt").is_err() {
            return false;
        }
        step.effects
            .iter()
            .any(|effect| matches!(&effect.op, SkillEffectDef::InterruptCast))
            && self
                .units
                .get(&target_id)
                .is_some_and(|unit| unit.pending_skill_cast.is_some())
    }

    fn is_hostile_effect(effect: &Effect) -> bool {
        match effect {
            Effect::Modifier(modifier) => Self::is_hostile_stat_modifier(*modifier),
            Effect::ApplyBuff { .. } => true,
            Effect::BasicLifesteal { .. }
            | Effect::BonusDamage { .. }
            | Effect::ModifyDamage(_)
            | Effect::Heal { .. } => false,
        }
    }

    fn is_hostile_stat_modifier(modifier: StatModifier) -> bool {
        match modifier.stat {
            StatId::MaxHealth
            | StatId::Attack
            | StatId::SpellPower
            | StatId::Defense
            | StatId::MagicResist => modifier.value < 0,
            StatId::AttackIntervalMs => modifier.value > 0,
            StatId::MoveSpeedUnitsPerMs => modifier.value < 0,
        }
    }
}
