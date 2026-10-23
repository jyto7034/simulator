//! Graph-authored modifiers resolve against real equipped relic copies. Their
//! receipts and effect sites are part of the same immutable effective plan.
mod area;
use super::*;
use crate::game::{
    ability::{
        action::*,
        sustained::{HitVulnerability, SustainedEffectDef},
        DeliveryDef, SkillDef, SkillEffectDef,
    },
    data::GameDataBase,
    relic::RelicVariant,
};
#[derive(Clone, serde::Serialize)]
pub(crate) struct EffectiveActionModifier {
    pub provider: usize,
    pub contribution: PlanContribution,
    pub change: ActionChange,
}
pub(crate) fn declares(
    data: &GameDataBase,
    definition: &str,
    variant: RelicVariant,
) -> Result<bool, RelicPlanError> {
    for skill in &data.skill_data.skills {
        crate::authoring::control::charge(1, "action_modifier_skill_scan")
            .map_err(RelicPlanError::control)?;
        if let Some(policy) = &skill.action_policy {
            crate::authoring::control::charge(
                policy.modifiers.len() as u64,
                "action_modifier_scan",
            )
            .map_err(RelicPlanError::control)?;
            if policy
                .modifiers
                .iter()
                .any(|m| m.relic_definition == definition && m.variant == variant)
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
pub(crate) fn validate_references(data: &GameDataBase) -> Result<(), RelicPlanError> {
    for skill in &data.skill_data.skills {
        let Some(policy) = &skill.action_policy else {
            continue;
        };
        for m in &policy.modifiers {
            crate::authoring::control::charge(1, "action_modifier_reference")
                .map_err(RelicPlanError::control)?;
            let d = data
                .relic_data
                .get_by_id(&m.relic_definition)
                .ok_or_else(|| {
                    RelicPlanError::new(
                        FaultCode::InvalidReference,
                        "Action modifier references unknown relic",
                    )
                })?;
            if m.variant == RelicVariant::Corrupt && d.corruption.is_none() {
                return Err(RelicPlanError::new(
                    FaultCode::InvalidReference,
                    "Action modifier references unavailable variant",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn resolve(
    plan: &mut EffectivePlanComponents,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    use dto::*;
    for (provider, p) in plan.providers.iter().enumerate() {
        let skill = data.skill_data.get_by_id(p.skill_id()).ok_or_else(|| {
            RelicPlanError::new(FaultCode::InvalidReference, "Action provider unavailable")
        })?;
        let Some(policy) = &skill.action_policy else {
            continue;
        };
        for m in &policy.modifiers {
            work.charge(plan.copies.len(), "action_modifier_equipped_copy")?;
            let Some(copy) = plan
                .copies
                .iter()
                .find(|c| c.definition_id == m.relic_definition && c.variant == m.variant)
            else {
                continue;
            };
            if let ActionChange::Range { steps, pattern } = &m.change {
                if plan.action_modifiers.iter().any(|prior|prior.provider==provider&&matches!(&prior.change,ActionChange::Range{steps:old,pattern:p} if p!=pattern&&steps.iter().any(|s|old.contains(s)))) {
                    return Err(RelicPlanError::new(FaultCode::StructuralWriteConflict,"Different range replacements target the same action"));
                }
            }
            if let ActionChange::SupportRange { step, effect, pattern } = &m.change {
                if plan.action_modifiers.iter().any(|prior| prior.provider == provider && matches!(&prior.change, ActionChange::SupportRange {step:s,effect:e,pattern:p} if s==step && e==effect && p!=pattern)) {
                    return Err(RelicPlanError::new(FaultCode::StructuralWriteConflict,"Different support footprints target the same effect"));
                }
            }
            compiler::admit_serialized_copy(m, work)?;
            let contribution = PlanContribution {
                copy_uuid: copy.copy_uuid,
                definition_id: copy.definition_id.clone(),
                variant: copy.variant,
                rule_id: format!("action-{}-{}", provider, m.id),
            };
            compiler::admit_serialized_copy(&contribution, work)?;
            let mut targets = Vec::new();
            let mut sites = Vec::new();
            for e in plan
                .effects
                .iter_mut()
                .filter(|e| e.original && e.provider == provider && e.present)
            {
                work.charge(1, "action_modifier_target")?;
                let EffectiveEffectKey::Original { site } = &e.key else {
                    continue;
                };
                let selected = match &m.change {
                    ActionChange::Area { step, .. } => &site.action.action_id == step,
                    ActionChange::Range { steps, .. } | ActionChange::Multihit { steps, .. } => {
                        steps.contains(&site.action.action_id)
                    }
                    ActionChange::SupportRange { step, effect, .. }
                    | ActionChange::TauntDamage { step, effect, .. }
                    | ActionChange::Duration { step, effect, .. }
                    | ActionChange::ControlVulnerability { step, effect, .. } => {
                        &site.action.action_id == step && &site.effect_id == effect
                    }
                    ActionChange::ProtectInterruption => true,
                };
                if selected && matches!(m.change, ActionChange::SupportRange { .. }) && e.recipes.iter().any(|r| matches!(r, authoring::StructuralRecipeDef::FixedRecipientSupport)) {
                    continue;
                }
                if !selected {
                    continue;
                }
                if let ActionChange::Range { pattern, .. } = &m.change {
                    compiler::admit_serialized_copy(e.delivery.as_ref(), work)?;
                    let mut delivery = (*e.delivery).clone();
                    let DeliveryDef::TileArea { footprint, .. } = &mut delivery else {
                        return Err(RelicPlanError::new(
                            FaultCode::InvalidReference,
                            "Action range target is not a tile area",
                        ));
                    };
                    *footprint = pattern.clone();
                    e.delivery = std::sync::Arc::new(delivery);
                }
                if e.operation == authoring::EffectOperation::Damage {
                    sites.push(site.clone());
                }
                compiler::admit_serialized_copy(&e.key, work)?;
                targets.push(EffectiveEffectAddressDto {
                    provider_index: provider,
                    effect: e.key.clone(),
                });
            }
            // Delivery/lifetime modifiers have an action site even when that
            // action has no per-recipient effects. Their DTO lists the change.
            let action_only = match &m.change {
                ActionChange::Area { step, .. } => skill.steps.iter().any(|s| &s.id == step),
                ActionChange::Range { steps, .. } => steps.iter().all(|id| skill.steps.iter().any(|s| &s.id == id)),
                ActionChange::ProtectInterruption => !skill.steps.is_empty(),
                _ => false,
            };
            if targets.is_empty() && matches!(m.change, ActionChange::SupportRange { .. }) {
                work.reserve(Some(std::mem::size_of::<RelicRuleStateDto>()), "support_range_dormant_receipt")?;
                plan.rules.push(RelicRuleStateDto { contribution, state:RelicRuleState::Dormant { reason:RelicDormantReason::NoMatchingEffect }, targets });
                continue;
            }
            if targets.is_empty() && !action_only {
                return Err(RelicPlanError::new(
                    FaultCode::InvalidReference,
                    "Action modifier target unavailable",
                ));
            }
            if let ActionChange::Multihit {
                increment_percent,
                cap_percent,
                ..
            } = &m.change
            {
                work.reserve(
                    Some(std::mem::size_of::<causal::EffectiveWedgeRule>()),
                    "action_wedge_owner",
                )?;
                plan.wedge_rules.push(causal::EffectiveWedgeRule {
                    contribution: contribution.clone(),
                    selector: Default::default(),
                    condition: Default::default(),
                    increment_percent: *increment_percent,
                    cap_percent: Some(*cap_percent),
                    effect_sites: sites,
                });
            }
            work.reserve(
                Some(
                    std::mem::size_of::<EffectiveActionModifier>()
                        + std::mem::size_of::<RelicRuleStateDto>(),
                ),
                "action_modifier_owner",
            )?;
            plan.action_modifiers.push(EffectiveActionModifier {
                provider,
                contribution: contribution.clone(),
                change: m.change.clone(),
            });
            plan.rules.push(RelicRuleStateDto {
                contribution,
                state: RelicRuleState::Active,
                targets,
            });
        }
    }
    Ok(())
}
pub(super) fn apply(
    skill: &mut SkillDef,
    provider: usize,
    modifiers: &[EffectiveActionModifier],
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    for m in modifiers.iter().filter(|m| m.provider == provider) {
        work.charge(1, "action_modifier_lower")?;
        match &m.change {
            ActionChange::SupportRange { step, effect, pattern } => {
                let e = skill.steps.iter_mut().find(|s| &s.id==step).and_then(|s| s.effects.iter_mut().find(|e| &e.id==effect)).ok_or_else(|| RelicPlanError::new(FaultCode::InvalidReference,"Support range site unavailable"))?;
                let SkillEffectDef::Sustained { definition:SustainedEffectDef::ChannelAura { aura, .. } } = &mut e.op else { return Err(RelicPlanError::new(FaultCode::InvalidReference,"Support range requires channel aura")); };
                let support = aura.support.as_mut().ok_or_else(|| RelicPlanError::new(FaultCode::InvalidReference,"Support range policy unavailable"))?;
                compiler::admit_serialized_copy(pattern,work)?;
                support.range = std::sync::Arc::new(pattern.clone());
            }
            ActionChange::Area { step, change } => area::apply(skill, provider, modifiers, m, step, change, work)?,
            ActionChange::ProtectInterruption => {
                let p = skill.action_policy.as_mut().ok_or_else(|| {
                    RelicPlanError::new(FaultCode::InvalidReference, "Action policy unavailable")
                })?;
                compiler::admit_serialized_copy(p.as_ref(), work)?;
                std::sync::Arc::make_mut(p).external_interrupt_protected = true;
            }
            ActionChange::Range { steps, pattern } => {
                for s in skill.steps.iter_mut().filter(|s| steps.contains(&s.id)) {
                    let DeliveryDef::TileArea { footprint, .. } = &mut s.delivery else {
                        return Err(RelicPlanError::new(
                            FaultCode::InvalidReference,
                            "Action range requires tile area",
                        ));
                    };
                    compiler::admit_serialized_copy(pattern, work)?;
                    *footprint = pattern.clone();
                }
            }
            ActionChange::TauntDamage { step, effect, taken_percent, basic_percent } => {
                let e = skill.steps.iter_mut().find(|s| &s.id == step).and_then(|s| s.effects.iter_mut().find(|e| &e.id == effect))
                    .ok_or_else(|| RelicPlanError::new(FaultCode::InvalidReference, "Taunt modifier site unavailable"))?;
                let SkillEffectDef::Sustained { definition: SustainedEffectDef::Control { kind: crate::game::ability::sustained::ControlKind::ApproachTaunt { profile }, .. } } = &mut e.op else {
                    return Err(RelicPlanError::new(FaultCode::InvalidReference, "Taunt modifier requires approach taunt"));
                };
                profile.damage_reduction_percent = profile.damage_reduction_percent.checked_add(*taken_percent).filter(|v| *v <= 100)
                    .ok_or_else(|| RelicPlanError::new(FaultCode::OutOfRange, "Taunt reduction exceeds 100 percent"))?;
                profile.basic_damage_bonus_percent = profile.basic_damage_bonus_percent.checked_add(*basic_percent).filter(|v| *v <= i32::MAX as u32)
                    .ok_or_else(|| RelicPlanError::new(FaultCode::OutOfRange, "Taunt basic bonus overflow"))?;
            }
            ActionChange::Duration {
                step,
                effect,
                percent,
            }
            | ActionChange::ControlVulnerability {
                step,
                effect,
                percent,
                ..
            } => {
                let s = skill
                    .steps
                    .iter_mut()
                    .find(|s| &s.id == step)
                    .ok_or_else(|| {
                        RelicPlanError::new(FaultCode::InvalidReference, "Action step unavailable")
                    })?;
                let e = s
                    .effects
                    .iter_mut()
                    .find(|e| &e.id == effect)
                    .ok_or_else(|| {
                        RelicPlanError::new(
                            FaultCode::InvalidReference,
                            "Action effect unavailable",
                        )
                    })?;
                let (duration_ms, profile) = match &mut e.op {
                    SkillEffectDef::Sustained { definition: SustainedEffectDef::HitStatus { duration_ms, profile } } => (duration_ms, Some(profile)),
                    SkillEffectDef::Sustained { definition: SustainedEffectDef::Control { duration_ms, .. } | SustainedEffectDef::ChannelAura { duration_ms, .. } } if matches!(m.change, ActionChange::Duration { .. }) => (duration_ms, None),
                    _ => return Err(RelicPlanError::new(FaultCode::InvalidReference, "Action modifier requires timed status")),
                };
                match &m.change {
                    ActionChange::Duration { .. } => {
                        // Same-kind contributions add, then the original duration
                        // is converted once with an upward millisecond rounding.
                        let first=modifiers.iter().filter(|other|other.provider==provider).find(|other|matches!(&other.change,ActionChange::Duration{step:s,effect:e,..} if s==step&&e==effect)).unwrap();
                        if !std::ptr::eq(first, m) {
                            continue;
                        }
                        let sum = modifiers
                            .iter()
                            .filter(|other| other.provider == provider)
                            .try_fold(0u64, |sum, other| match &other.change {
                                ActionChange::Duration {
                                    step: s,
                                    effect: e,
                                    percent: p,
                                } if s == step && e == effect => sum.checked_add(u64::from(*p)),
                                _ => Some(sum),
                            })
                            .ok_or_else(|| {
                                RelicPlanError::new(
                                    FaultCode::OutOfRange,
                                    "Duration percentage overflow",
                                )
                            })?;
                        *duration_ms = u32::try_from(
                            u64::from(*duration_ms)
                                .checked_mul(100 + sum)
                                .ok_or_else(|| {
                                    RelicPlanError::new(
                                        FaultCode::OutOfRange,
                                        "Duration multiplication overflow",
                                    )
                                })?
                                .div_ceil(100),
                        )
                        .map_err(|_| {
                            RelicPlanError::new(
                                FaultCode::OutOfRange,
                                "Modified duration overflows milliseconds",
                            )
                        })?;
                    }
                    ActionChange::ControlVulnerability { duration_ms, .. } => {
                        let profile = profile.ok_or_else(|| RelicPlanError::new(FaultCode::InvalidReference, "Vulnerability requires hit status"))?;
                        if profile.vulnerability.is_some() {
                            return Err(RelicPlanError::new(
                                FaultCode::StructuralWriteConflict,
                                "Multiple vulnerability writes",
                            ));
                        }
                        profile.vulnerability = Some(HitVulnerability {
                            definition: format!(
                                "{}:{}",
                                m.contribution.definition_id, m.contribution.rule_id
                            ),
                            percent: *percent,
                            duration_ms: *duration_ms,
                            modifier_source: Some(
                                crate::game::ability::sustained::HitModifierSource {
                                    copy_uuid: m.contribution.copy_uuid,
                                    definition: m.contribution.definition_id.clone(),
                                    rule: m.contribution.rule_id.clone(),
                                },
                            ),
                        });
                    }
                    _ => unreachable!(),
                }
            }
            ActionChange::Multihit { .. } => (),
        }
    }
    Ok(())
}
