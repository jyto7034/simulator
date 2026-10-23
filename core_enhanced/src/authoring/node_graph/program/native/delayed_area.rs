use super::*;
use crate::game::{ability::*, value_expr::*};

pub(super) fn bind(
    skill: &mut SkillDef,
    plan: &super::super::DelayedAreaProgram,
) -> Result<(), String> {
    plan.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.focus_time_ms = plan.prepare_ms;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::AutomaticUnit,
        target: SkillTarget::EnemySingle {
            rule: UnitTargetRule::RouteProgress,
        },
        reach: SkillTargetReach::Pattern,
        target_range: Some(plan.acquisition.clone()),
        air_capable: plan.include_air,
    };
    let step = &mut skill.steps[0];
    step.id = "impact".into();
    step.target = SkillTarget::CastTarget;
    step.delivery = DeliveryDef::TileArea {
        footprint: plan.footprint.clone(),
        area: SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::AcceptedCastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: SkillHitTargetFilter::Enemies,
            first_tick_delay_ms: plan.delivery_delay_ms,
            on_hit_caster_heal_percent: plan.on_hit_heal_percent,
            ..Default::default()
        },
    };
    step.effects.push(AuthoredSkillEffect::new(
        "damage",
        SkillEffectDef::DamageExpr {
            amount: weapon_amount(plan.coefficient_numerator, plan.coefficient_denominator),
            damage_type: plan.damage_type,
        },
    ));
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "release".into(),
            step_id: "impact".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}

pub(super) fn weapon_amount(numerator: u64, denominator: u64) -> AmountExpr {
    AmountExpr::authored(
        EvalMoment::Snapshot,
        ValueExpr::Ratio {
            numerator: Box::new(ValueExpr::Mul {
                value: Box::new(ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::Source,
                    stat: StatSelector::Attack,
                })),
                scalar: Box::new(ValueExpr::Const(TypedInteger::new(
                    numerator.into(),
                    ValueUnit::Scalar,
                ))),
            }),
            denominator: Box::new(ValueExpr::Const(TypedInteger::new(
                denominator.into(),
                ValueUnit::Scalar,
            ))),
            rounding: Rounding::Ceil,
        },
    )
}
