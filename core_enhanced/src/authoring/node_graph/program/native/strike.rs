use super::super::strike::*;
use super::*;
use crate::game::{ability::*, value_expr::*};
fn scalar(n: u32) -> ValueExpr {
    ValueExpr::Const(TypedInteger::new(n.into(), ValueUnit::Scalar))
}
fn hp_scalar(field: HitPointField) -> ValueExpr {
    ValueExpr::Ratio {
        numerator: Box::new(ValueExpr::Read(ValueRef::HitPoints {
            subject: ValueSubject::Target,
            field,
        })),
        denominator: Box::new(ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude))),
        rounding: Rounding::Floor,
    }
}
fn times(value: ValueExpr, scalar: ValueExpr) -> ValueExpr {
    ValueExpr::Mul {
        value: Box::new(value),
        scalar: Box::new(scalar),
    }
}
fn missing_amount(n: u64, d: u64, percent: u32, cap: u32) -> AmountExpr {
    // Keep the HP ratio exact until the final floor, including fractional
    // percentage points. Integer division of the ratio would lose authored power.
    let max = hp_scalar(HitPointField::Max);
    let bonus = ValueExpr::Clamp {
        value: Box::new(times(hp_scalar(HitPointField::Missing), scalar(percent))),
        min: Box::new(scalar(0)),
        max: Box::new(times(max.clone(), scalar(cap))),
    };
    AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Ratio {
            numerator: Box::new(times(
                delayed_area::weapon_amount(n, d).root,
                ValueExpr::Add(vec![times(max.clone(), scalar(100)), bonus]),
            )),
            denominator: Box::new(times(max, scalar(100))),
            rounding: Rounding::Floor,
        },
    )
}
pub(super) fn bind(skill: &mut SkillDef, program: &StrikeProgram) -> Result<(), String> {
    program.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::AutomaticUnit,
        target: SkillTarget::EnemySingle {
            rule: UnitTargetRule::RouteProgress,
        },
        reach: SkillTargetReach::Pattern,
        target_range: Some(program.acquisition.clone()),
        air_capable: program.include_air,
    };
    let template = skill.steps[0].clone();
    skill.steps.clear();
    let mut children = Vec::new();
    for (i, part) in program.parts.iter().enumerate() {
        match part {
            StrikePart::Wait { duration_ms } => children.push(SkillFlowDef::Wait {
                id: format!("wait-{i}"),
                duration_ms: u64::from(*duration_ms),
            }),
            StrikePart::Strike {
                id,
                range,
                damage_type,
                numerator,
                denominator,
                missing_hp,
                after,
            } => {
                let mut step = template.clone();
                step.id = id.clone();
                step.target = SkillTarget::CastTarget;
                step.delivery = DeliveryDef::TileArea {
                    footprint: range.clone(),
                    area: SkillTileAreaDeliveryDef {
                        anchor: SkillAreaAnchorSource::Caster,
                        tile_origin: SkillTileAreaOrigin::Caster,
                        hit_targets: SkillHitTargetFilter::Enemies,
                        ..Default::default()
                    },
                };
                step.effects.push(AuthoredSkillEffect::new(
                    "damage",
                    SkillEffectDef::DamageExpr {
                        amount: missing_hp.map_or_else(
                            || delayed_area::weapon_amount(*numerator, *denominator),
                            |(p, c)| missing_amount(*numerator, *denominator, p, c),
                        ),
                        damage_type: *damage_type,
                    },
                ));
                for (j, status) in after.iter().enumerate() {
                    let op = match status {
                        StrikeStatus::Pull { distance } => SkillEffectDef::Pull {
                            max_distance_tiles: *distance,
                        },
                        StrikeStatus::Stun {
                            definition,
                            duration_ms,
                        } => SkillEffectDef::Sustained {
                            definition: sustained::SustainedEffectDef::HitStatus {
                                duration_ms: *duration_ms,
                                profile: sustained::HitStatusProfile {
                                    definition: definition.clone(),
                                    kind: sustained::HitStatusKind::Stun,
                                    modifier_source: None,
                                    vulnerability: None,
                                },
                            },
                        },
                    };
                    let mut effect = AuthoredSkillEffect::new(format!("after-{j}"), op);
                    effect.attached_to = Some("damage".into());
                    step.effects.push(effect);
                }
                skill.steps.push(step);
                // A status has its own deadline; it does not delay the next strike.
                children.push(SkillFlowDef::Action {
                    id: format!("release-{i}"),
                    step_id: id.clone(),
                    wait: FlowMilestone::Released,
                });
            }
        }
    }
    skill.flow = Some(
        SkillFlowDef::Sequence {
            id: "sequence".into(),
            children,
        }
        .into(),
    );
    skill.action_policy = Some(program.policy.clone().into());
    program.policy.validate(skill)?;
    Ok(())
}
