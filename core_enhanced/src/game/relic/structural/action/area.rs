use super::*;
use crate::game::ability::{action::AreaChange, ground::*};
pub(super) fn apply(
    skill: &mut SkillDef,
    provider: usize,
    modifiers: &[EffectiveActionModifier],
    current: &EffectiveActionModifier,
    step: &str,
    change: &AreaChange,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let error = |message: &str| RelicPlanError::new(FaultCode::InvalidReference, message);
    let target = skill
        .steps
        .iter_mut()
        .find(|s| s.id == step)
        .ok_or_else(|| error("Area step unavailable"))?;
    let DeliveryDef::TileArea { area, .. } = &mut target.delivery else {
        return Err(error("Area delivery unavailable"));
    };
    match change {
        AreaChange::Duration { .. } => {
            let first = modifiers.iter().filter(|m| m.provider == provider).find(|m| matches!(&m.change, ActionChange::Area {step:s,change:AreaChange::Duration {..}} if s == step)).unwrap();
            if !std::ptr::eq(first, current) {
                return Ok(());
            }
            let sum = modifiers
                .iter()
                .filter(|m| m.provider == provider)
                .try_fold(0u64, |sum, m| match &m.change {
                    ActionChange::Area {
                        step: s,
                        change: AreaChange::Duration { percent },
                    } if s == step => sum.checked_add(u64::from(*percent)),
                    _ => Some(sum),
                })
                .ok_or_else(|| error("Area duration modifier overflow"))?;
            area.duration_ms = u32::try_from(
                (u128::from(area.duration_ms) * (100 + u128::from(sum))).div_ceil(100),
            )
            .map_err(|_| error("Area duration exceeds native milliseconds"))?;
        }
        AreaChange::RampRate {
            status,
            reached_rules,
            numerator,
            denominator,
        } => {
            // Multiple edits of one progress clock are an explicit conflict.
            if modifiers.iter().filter(|m| m.provider == provider && matches!(
                &m.change,
                ActionChange::Area { step: s, change: AreaChange::RampRate { status: i, reached_rules: rules, .. } }
                    if s == step && (i == status || rules.iter().any(|rule| reached_rules.contains(rule)))
            )).count() != 1 { return Err(error("Multiple ramp-rate writers")); }
            let p = area
                .ground_status
                .as_mut()
                .ok_or_else(|| error("Ground program unavailable"))?;
            let GroundStatusKind::RampingSlow {
                duration_ms,
                rate_numerator,
                rate_denominator,
                ..
            } = &mut p.statuses[*status].kind
            else {
                return Err(error("Ramp status unavailable"));
            };
            let modified = u32::try_from(
                (u128::from(*duration_ms) * u128::from(*denominator))
                    .div_ceil(u128::from(*numerator)),
            )
            .ok()
            .filter(|v| *v > 0)
            .ok_or_else(|| error("Ramp duration exceeds native milliseconds"))?;
            *rate_numerator = *numerator;
            *rate_denominator = *denominator;
            for index in reached_rules {
                p.rules[*index].trigger = GroundTrigger::Dwell {
                    duration_ms: modified,
                };
            }
        }
        AreaChange::ControlVulnerability {
            status,
            percent,
            duration_ms,
        } => {
            let p = area
                .ground_status
                .as_mut()
                .ok_or_else(|| error("Ground program unavailable"))?;
            let target = &mut p.statuses[*status];
            if target.vulnerability.is_some() {
                return Err(error("Multiple ground vulnerability writers"));
            }
            let v = HitVulnerability {
                definition: format!(
                    "{}:{}",
                    current.contribution.definition_id, current.contribution.rule_id
                ),
                percent: *percent,
                duration_ms: *duration_ms,
                modifier_source: Some(crate::game::ability::sustained::HitModifierSource {
                    copy_uuid: current.contribution.copy_uuid,
                    definition: current.contribution.definition_id.clone(),
                    rule: current.contribution.rule_id.clone(),
                }),
            };
            compiler::admit_owned_copy(&v, work)?;
            target.vulnerability = Some(v);
        }
    }
    area.try_validate_runtime_contract().map_err(|e| error(&e))
}
