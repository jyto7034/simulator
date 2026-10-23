use super::*;
use crate::game::{
    ability::sustained::{ChannelAuraDef, SupportCoefficientScope},
    relic::{
        structural::{
            authoring::{PersonalShieldGrantDef, PersonalShieldRecipientsDef},
            support::{SupportConversionDef, SupportPolicyDef, SupportReplacementDef},
        },
        RelicVariant,
    },
    value_expr::*,
};
use std::sync::Arc;

fn scaled(value: ValueExpr, numerator: u64, denominator: u64) -> ValueExpr {
    ValueExpr::Ratio {
        numerator: Box::new(ValueExpr::Mul {
            value: Box::new(value),
            scalar: Box::new(ValueExpr::Const(TypedInteger::new(
                numerator.into(),
                ValueUnit::Scalar,
            ))),
        }),
        denominator: Box::new(ValueExpr::Const(TypedInteger::new(
            denominator.into(),
            ValueUnit::Scalar,
        ))),
        rounding: Rounding::Floor,
    }
}
fn shield_value(percent: u32) -> ValueExpr {
    scaled(
        ValueExpr::Read(ValueRef::HitPoints {
            subject: ValueSubject::Source,
            field: HitPointField::Max,
        }),
        percent.into(),
        100,
    )
}
pub(super) fn bind(skill: &mut SkillDef, program: &SupportProgram) -> Result<(), String> {
    program.validate()?;
    let conversion = program.conversion.as_ref().map(|c| {
        let replacement = match c.replacement {
            SupportReplacementProgram::PersonalShield {
                numerator,
                denominator,
            } => {
                let SupportEffectProgram::PersonalShield { max_health_percent } = program.effect
                else {
                    unreachable!("validated replacement")
                };
                SupportReplacementDef::PersonalShield {
                    amount: Arc::new(AmountExpr::authored(
                        EvalMoment::Snapshot,
                        scaled(shield_value(max_health_percent), numerator, denominator),
                    )),
                }
            }
            SupportReplacementProgram::Channel {
                damage_percent,
                speed_points,
            } => SupportReplacementDef::RegularBasic {
                damage_percent,
                speed_points,
            },
        };
        SupportConversionDef {
            relic_definition: c.relic_definition.clone(),
            variant: RelicVariant::Normal,
            selection_range: Arc::new(c.selection_range.clone()),
            replacement,
        }
    });
    let support = Some(SupportPolicyDef {
        range: Arc::new(program.range.clone()),
        conversion,
    });
    let effect = match program.effect {
        SupportEffectProgram::PersonalShield { max_health_percent } => {
            SkillEffectDef::PersonalShieldGrant {
                definition: PersonalShieldGrantDef {
                    amount: Arc::new(AmountExpr::authored(
                        EvalMoment::Snapshot,
                        shield_value(max_health_percent),
                    )),
                    duration_ms: program.duration_ms,
                    priority: 0,
                    recipients: PersonalShieldRecipientsDef::PlayerEmployeesAtRelease {
                        radius_units: 0,
                    },
                    support,
                    replace_only_recipients: true,
                    source_creation: None,
                },
            }
        }
        SupportEffectProgram::Channel {
            damage_percent,
            speed_points,
        } => {
            skill.use_lifetime = SkillUseLifetime::ActionAndEffects {
                effects: vec![SkillUseEffectRef {
                    step: "finish".into(),
                    effect: "support".into(),
                }],
            };
            SkillEffectDef::Sustained {
                definition: SustainedEffectDef::ChannelAura {
                    duration_ms: program.duration_ms,
                    aura: ChannelAuraDef {
                        family: program.family.clone(),
                        priority: 0,
                        prefer_strongest: true,
                        radius_units: 0,
                        support,
                        basic_damage_percent: damage_percent,
                        basic_speed_points: speed_points,
                        basic_scope: SupportCoefficientScope::RegularOnly,
                    },
                },
            }
        }
    };
    skill.steps[0]
        .effects
        .push(AuthoredSkillEffect::new("support", effect));
    skill.flow = Some(Arc::new(SkillFlowDef::Action {
        id: "release".into(),
        step_id: "finish".into(),
        wait: if matches!(program.effect, SupportEffectProgram::Channel { .. }) {
            FlowMilestone::EffectsSettled
        } else {
            FlowMilestone::Released
        },
    }));
    skill.action_policy = Some(program.policy.clone().into());
    program.policy.validate(skill)
}
