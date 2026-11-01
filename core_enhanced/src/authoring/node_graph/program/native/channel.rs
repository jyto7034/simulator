use super::*;
use crate::game::ability::*;
pub(super) fn bind(skill: &mut SkillDef, p: &ChannelGroundProgram) -> Result<(), String> {
    p.validate()?;
    let step = &mut skill.steps[0];
    step.id = "channel".into();
    step.delivery = DeliveryDef::TileArea {
        footprint: p.range.clone(),
        area: SkillTileAreaDeliveryDef {
            mode: SkillTileAreaMode::ChannelGround {
                pause_basic: p.pause_basic,
            },
            anchor: SkillAreaAnchorSource::AcceptedCastTarget,
            tile_origin: SkillTileAreaOrigin::Anchor,
            duration_ms: p.duration_ms,
            hit_targets: p.hit_targets,
            include_caster: p.include_caster,
            ground_status: Some(p.status.clone()),
            ..Default::default()
        },
    };
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "concentrate".into(),
            step_id: "channel".into(),
            wait: FlowMilestone::EffectsSettled,
        }
        .into(),
    );
    skill.action_policy = Some(p.policy.clone().into());
    p.policy.validate(skill)
}
