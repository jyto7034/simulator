use super::*;
use crate::game::{ability::*, battle::tile_range::TileRangePolicy};

pub(super) fn bind(
    parent: &SkillDef,
    plan: &super::super::AttackOverlayProgram,
) -> Result<(SustainedEffectDef, SkillDef), String> {
    plan.validate()?;
    let mut child = parent.clone();
    child.id = format!("{}::finish", parent.id).into();
    child.resonance_admission = None;
    child.use_lifetime = SkillUseLifetime::Action;
    // All operands are validated integers. Time is captured by the committed
    // reinput, and rounded only after the millisecond-precision multiplication.
    use crate::game::value_expr::*;
    let constant = |value, unit| Box::new(ValueExpr::Const(TypedInteger::new(value, unit)));
    let increase = ValueExpr::Mul {
        value: constant(1, ValueUnit::Magnitude),
        scalar: Box::new(ValueExpr::Ratio {
            numerator: Box::new(ValueExpr::Mul {
                value: Box::new(ValueExpr::Read(ValueRef::Time(TimeValue::RemainingTime))),
                scalar: constant(plan.per_second.into(), ValueUnit::Scalar),
            }),
            denominator: constant(1000, ValueUnit::Milliseconds),
            rounding: Rounding::Ceil,
        }),
    };
    let mut root = ValueExpr::Add(vec![
        *constant(plan.base.into(), ValueUnit::Magnitude),
        increase,
    ]);
    if let Some(cap) = plan.cap {
        root = ValueExpr::Clamp {
            value: Box::new(root),
            min: constant(0, ValueUnit::Magnitude),
            max: constant(cap.into(), ValueUnit::Magnitude),
        };
    }
    let amount = AmountExpr::authored(EvalMoment::Snapshot, root);
    child.steps[0].delivery = DeliveryDef::TileArea {
        footprint: plan.finish_range.clone(),
        area: SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::Caster,
            tile_origin: SkillTileAreaOrigin::Caster,
            ..Default::default()
        },
    };
    child.steps[0].effects.push(AuthoredSkillEffect::new(
        "damage",
        SkillEffectDef::DamageExpr {
            amount,
            damage_type: plan.damage_type,
        },
    ));
    child.flow = None; // Immediate child is consumed atomically by the reinput transaction.
    Ok((
        SustainedEffectDef::BasicAttackOverlay {
            duration_ms: plan.duration_ms,
            next_attack_only: false,
            goodbye_skill: Some(child.id.clone()),
            profile: sustained::BasicAttackOverlayProfile {
                range_policy: TileRangePolicy::WholeFieldValidTiles,
                defense_tile_range: None,
                delivery: DeliveryDef::Instant,
                line: Some(plan.line.clone()),
                followup_on_input: true,
            },
        },
        child,
    ))
}
