use crate::game::ability::*;

pub(super) fn bind(
    skill: &mut SkillDef,
    plan: &super::super::ScheduledAreaProgram,
) -> Result<(), String> {
    plan.validate()?;
    super::delayed_area::bind(skill, &plan.impact)?;
    skill.kind = SkillKind::Untargeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::SelfUnit,
        target: SkillTarget::SelfUnit,
        reach: SkillTargetReach::Pattern,
        target_range: None,
        air_capable: true,
    };
    let step = &mut skill.steps[0];
    step.target = SkillTarget::SelfUnit;
    step.delivery = DeliveryDef::TileArea {
        footprint: plan.impact.acquisition.clone(),
        area: SkillTileAreaDeliveryDef {
            emission: Some(SkillAreaEmissionDef {
                footprint: plan.impact.footprint.clone(),
                delay_ms: plan.impact.delivery_delay_ms,
                include_air: plan.impact.include_air,
            }),
            anchor: SkillAreaAnchorSource::AcceptedCastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            hit_targets: SkillHitTargetFilter::Enemies,
            duration_ms: plan.duration_ms()?,
            tick_interval_ms: Some(plan.interval_ms),
            ..Default::default()
        },
    };
    Ok(())
}
