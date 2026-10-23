use super::*;
use crate::game::ability::*;

pub(super) fn bind(skill: &mut SkillDef, plan: &GroundProgram) -> Result<(), String> {
    plan.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::PlayerTile,
        target: SkillTarget::CastTarget,
        reach: SkillTargetReach::Pattern,
        target_range: Some(plan.selection_range.clone()),
        air_capable: true,
    };
    let step = &mut skill.steps[0];
    step.id = "ground".into();
    step.target = SkillTarget::CastTarget;
    step.delivery = DeliveryDef::TileArea {
        footprint: plan.footprint.clone(),
        area: SkillTileAreaDeliveryDef {
            mode: SkillTileAreaMode::SelectedGround,
            anchor: SkillAreaAnchorSource::AcceptedCastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: plan.hit_targets,
            include_caster: plan.include_caster,
            duration_ms: plan.duration_ms,
            tick_interval_ms: Some(plan.interval_ms),
            first_tick_delay_ms: match plan.first {
                GroundFirstTick::Immediate => 0,
                GroundFirstTick::AfterInterval => plan.interval_ms,
            },
            ..Default::default()
        },
    };
    step.effects.push(AuthoredSkillEffect::new(
        "pulse",
        match plan.effect {
            GroundEffect::Damage {
                amount,
                damage_type,
            } => SkillEffectDef::Damage {
                amount: amount as i32,
                damage_type,
            },
            GroundEffect::Heal { amount } => SkillEffectDef::Heal {
                amount: amount as i32,
            },
        },
    ));
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "create".into(),
            step_id: "ground".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}

pub(super) fn bind_status(
    skill: &mut SkillDef,
    plan: &GroundStatusAreaProgram,
) -> Result<(), String> {
    plan.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::PlayerTile,
        target: SkillTarget::CastTarget,
        reach: SkillTargetReach::Pattern,
        target_range: Some(plan.selection_range.clone()),
        air_capable: true,
    };
    let step = &mut skill.steps[0];
    step.id = "ground".into();
    step.target = SkillTarget::CastTarget;
    step.delivery = DeliveryDef::TileArea {
        footprint: plan.footprint.clone(),
        area: SkillTileAreaDeliveryDef {
            mode: SkillTileAreaMode::SelectedGround,
            ground_status: Some(plan.status.clone()),
            anchor: SkillAreaAnchorSource::AcceptedCastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            tracking: SkillAreaTracking::GroundFixed,
            hit_targets: plan.hit_targets,
            include_caster: plan.include_caster,
            duration_ms: plan.duration_ms,
            ..Default::default()
        },
    };
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "create".into(),
            step_id: "ground".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}
