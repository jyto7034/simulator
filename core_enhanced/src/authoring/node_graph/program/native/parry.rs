use super::*;
use crate::game::ability::*;
pub(super) fn bind(
    parent: &SkillDef,
    plan: &super::super::ParryProgram,
) -> Result<(SustainedEffectDef, SkillDef), String> {
    plan.validate()?;
    let mut counter = parent.clone();
    counter.id = format!("{}__counter", parent.id).into();
    counter.resonance_admission = None;
    counter.kind = SkillKind::Targeted;
    counter.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::AutomaticUnit,
        target: SkillTarget::EnemySingle {
            rule: UnitTargetRule::RouteProgress,
        },
        reach: SkillTargetReach::Global,
        target_range: None,
        air_capable: true,
    };
    counter.steps[0].target = SkillTarget::CastTarget;
    counter.steps[0].effects.push(AuthoredSkillEffect::new(
        "counter",
        if let Some(charge) = &plan.charge {
            SkillEffectDef::GuardCounter {
                program: std::sync::Arc::new(crate::game::ability::guard_charge::GuardCounterDef {
                    charge: std::sync::Arc::new(charge.clone()),
                    weapon: crate::game::ability::marks::MarkRatio {
                        numerator: plan.coefficient_numerator,
                        denominator: plan.coefficient_denominator,
                    },
                    damage_type: plan.damage_type,
                }),
            }
        } else {
            SkillEffectDef::DamageExpr {
                amount: delayed_area::weapon_amount(
                    plan.coefficient_numerator,
                    plan.coefficient_denominator,
                ),
                damage_type: plan.damage_type,
            }
        },
    ));
    Ok((plan.definition(counter.id.clone())?, counter))
}
