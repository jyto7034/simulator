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

impl BattleCore {
    pub(in crate::game::battle::core) fn is_basic_attack_useful_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> bool {
        self.preview_basic_attack_final_damage(attacker_instance_id, target_id)
            .is_some_and(|damage| damage > 0)
            || self.basic_attack_has_hostile_target_effect(attacker_instance_id, target_id)
    }

    pub(in crate::game::battle::core) fn is_skill_step_useful_hostile_target(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        let Some((caster_owner, target_owner)) =
            self.caster_and_target_owner(caster_instance_id, target_id)
        else {
            return false;
        };
        if target_owner == caster_owner {
            return false;
        }

        self.preview_skill_step_final_damage(caster_instance_id, step, target_id)
            .is_some_and(|damage| damage > 0)
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
        targets.iter().copied().any(|target_id| {
            self.is_skill_step_useful_hostile_target(caster_instance_id, step, target_id)
        })
    }

    pub(in crate::game::battle::core) fn skill_step_needs_hostile_usefulness_gate(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        hit_targets: SkillHitTargetFilter,
        targets: &[UnitInstanceId],
    ) -> bool {
        if matches!(hit_targets, SkillHitTargetFilter::Allies) {
            return false;
        }
        if !Self::skill_step_has_hostile_payload(step) {
            return false;
        }
        let Some(caster_owner) = self.units.get(&caster_instance_id).map(|unit| unit.owner) else {
            return false;
        };
        targets.iter().copied().any(|target_id| {
            self.units
                .get(&target_id)
                .is_some_and(|target| target.owner != caster_owner)
        })
    }

    pub(in crate::game::battle::core) fn skill_has_useful_automatic_cast_target(
        &self,
        time_ms: u64,
        skill: &SkillDef,
        caster_instance_id: UnitInstanceId,
        cast_target: Option<SkillCastTarget>,
    ) -> bool {
        let mut has_hostile_payload = false;
        let mut has_hostile_targeting_step = false;

        for (step_index, step) in skill.steps.iter().enumerate() {
            if Self::skill_step_targets_enemies(step) {
                has_hostile_targeting_step = true;
            }
            if !Self::skill_step_has_hostile_payload(step) {
                continue;
            }
            has_hostile_payload = true;

            match &step.target {
                SkillTarget::EnemySingle { rule } => {
                    if let Some(SkillCastTarget::Unit { unit_instance_id }) = cast_target {
                        if self.is_skill_step_useful_hostile_target(
                            caster_instance_id,
                            step,
                            unit_instance_id,
                        ) {
                            return true;
                        }
                    }
                    if cast_target.is_none()
                        && matches!(
                            &step.delivery,
                            DeliveryDef::Projectile {
                                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                                allow_targetless_cast: true,
                                ..
                            }
                        )
                        && matches!(
                            &step.delivery,
                            DeliveryDef::Projectile {
                                max_range_tiles: Some(_),
                                ..
                            } | DeliveryDef::Projectile {
                                max_lifetime_ms: Some(_),
                                ..
                            }
                        )
                        && self
                            .units
                            .get(&caster_instance_id)
                            .and_then(|unit| unit.facing_direction)
                            .is_some()
                    {
                        return true;
                    }
                    if matches!(step.targeting, StepTargetingMode::RetargetOnStep)
                        && self
                            .resolve_cast_origin_context(None, caster_instance_id, false)
                            .and_then(|(caster_owner, caster_pos)| {
                                self.choose_skill_target_by_rule(
                                    caster_instance_id,
                                    caster_owner,
                                    caster_pos,
                                    step.range_policy,
                                    step.defense_tile_range.as_ref(),
                                    *rule,
                                    step.air_capable,
                                    Some(step),
                                )
                            })
                            .is_some()
                    {
                        return true;
                    }
                }
                SkillTarget::SelfUnit | SkillTarget::CastTarget => {
                    if let DeliveryDef::TileArea { area } = &step.delivery {
                        let Some(tile_range) = step.defense_tile_range.as_ref() else {
                            continue;
                        };
                        let Some((caster_owner, _, _, _, anchor_tile)) = self
                            .resolve_tile_area_geometry(
                                time_ms,
                                0,
                                step_index,
                                caster_instance_id,
                                cast_target,
                                area,
                            )
                        else {
                            continue;
                        };
                        let Some(facing) = self
                            .units
                            .get(&caster_instance_id)
                            .and_then(|unit| unit.facing_direction)
                        else {
                            continue;
                        };
                        let Ok(affected_tiles) = tile_range.affected_tiles(anchor_tile, facing)
                        else {
                            continue;
                        };
                        let targets = self.collect_tile_area_targets_at(
                            time_ms,
                            caster_owner,
                            caster_instance_id,
                            &affected_tiles,
                            area.hit_targets,
                            area.include_caster,
                        );
                        if !self.skill_step_needs_hostile_usefulness_gate(
                            caster_instance_id,
                            step,
                            area.hit_targets,
                            &targets,
                        ) || self.skill_step_has_useful_hostile_target(
                            caster_instance_id,
                            step,
                            &targets,
                        ) {
                            return true;
                        }
                    } else if let Some(SkillCastTarget::Unit { unit_instance_id }) = cast_target {
                        if self.is_skill_step_useful_hostile_target(
                            caster_instance_id,
                            step,
                            unit_instance_id,
                        ) {
                            return true;
                        }
                    }
                }
            }
        }

        !has_hostile_payload && !has_hostile_targeting_step
    }

    pub(in crate::game::battle::core) fn skill_cast_target_usefulness_step<'a>(
        &self,
        skill: &'a SkillDef,
        target: &SkillTarget,
    ) -> Option<&'a SkillStepDef> {
        if !matches!(target, SkillTarget::EnemySingle { .. }) {
            return None;
        }

        skill
            .steps
            .iter()
            .find(|step| {
                Self::skill_step_has_hostile_payload(step)
                    && matches!(
                        step.target,
                        SkillTarget::EnemySingle { .. } | SkillTarget::CastTarget
                    )
            })
            .or_else(|| skill.first_step())
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

        let on_attack_effects =
            self.collect_all_triggers(attacker_instance_id, TriggerType::OnAttack);
        let on_hit_effects = self.collect_all_triggers(target_id, TriggerType::OnHit);
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

    fn preview_skill_step_final_damage(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> Option<u32> {
        let caster = self.units.get(&caster_instance_id)?;
        let target = self.units.get(&target_id)?;
        if !caster.is_active() || !target.is_active() || caster.owner == target.owner {
            return None;
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
            } = effect
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

        Some(final_damage)
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
            .filter_map(|effect| match effect {
                SkillEffectDef::ModifyDamage { modifiers } => Some(*modifiers),
                _ => None,
            })
            .fold(DamageModifiers::default(), DamageModifiers::merge)
    }

    fn skill_step_has_hostile_payload(step: &SkillStepDef) -> bool {
        step.effects.iter().any(|effect| match effect {
            SkillEffectDef::Damage { amount, .. } => *amount != 0,
            SkillEffectDef::ModifyStats { modifier } => Self::is_hostile_stat_modifier(*modifier),
            SkillEffectDef::ApplyBuff { .. } => true,
            SkillEffectDef::InterruptCast => true,
            SkillEffectDef::ExtraAttack { count } => *count > 0,
            SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Heal { .. }
            | SkillEffectDef::ModifyResonance { .. }
            | SkillEffectDef::ModifyStabilization { .. } => false,
        })
    }

    fn skill_step_targets_enemies(step: &SkillStepDef) -> bool {
        matches!(step.target, SkillTarget::EnemySingle { .. })
            || matches!(
                &step.delivery,
                DeliveryDef::TileArea { area }
                    if !matches!(area.hit_targets, SkillHitTargetFilter::Allies)
            )
    }

    fn skill_step_has_hostile_target_applied_effect(step: &SkillStepDef) -> bool {
        step.effects.iter().any(|effect| match effect {
            SkillEffectDef::ModifyStats { modifier } => Self::is_hostile_stat_modifier(*modifier),
            SkillEffectDef::ApplyBuff { .. } => true,
            SkillEffectDef::Damage { .. }
            | SkillEffectDef::InterruptCast
            | SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Heal { .. }
            | SkillEffectDef::ModifyResonance { .. }
            | SkillEffectDef::ModifyStabilization { .. }
            | SkillEffectDef::ExtraAttack { .. } => false,
        })
    }

    fn skill_step_schedules_useful_extra_attack(
        &self,
        caster_instance_id: UnitInstanceId,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        step.effects.iter().any(|effect| {
            matches!(effect, SkillEffectDef::ExtraAttack { count } if *count > 0)
                && self.is_basic_attack_useful_target(caster_instance_id, target_id)
        })
    }

    fn skill_step_has_interruptible_cast_effect(
        &self,
        step: &SkillStepDef,
        target_id: UnitInstanceId,
    ) -> bool {
        step.effects
            .iter()
            .any(|effect| matches!(effect, SkillEffectDef::InterruptCast))
            && self
                .units
                .get(&target_id)
                .is_some_and(|unit| unit.pending_skill_cast.is_some())
    }

    fn is_hostile_effect(effect: &Effect) -> bool {
        match effect {
            Effect::Modifier(modifier) => Self::is_hostile_stat_modifier(*modifier),
            Effect::ApplyBuff { .. } => true,
            Effect::BonusDamage { .. } | Effect::ModifyDamage(_) | Effect::Heal { .. } => false,
        }
    }

    fn is_hostile_stat_modifier(modifier: StatModifier) -> bool {
        match modifier.stat {
            StatId::MaxHealth | StatId::Attack | StatId::Defense | StatId::MagicResist => {
                modifier.value < 0
            }
            StatId::AttackIntervalMs => modifier.value > 0,
            StatId::MoveSpeedUnitsPerMs => modifier.value < 0,
        }
    }
}
