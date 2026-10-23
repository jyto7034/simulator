use super::*;
use crate::game::ability::*;
pub(super) fn bind(skill: &mut SkillDef, program: &TauntProgram) -> Result<(), String> {
    program.validate()?;
    let step = &mut skill.steps[0];
    step.id = "taunt".into();
    step.delivery = DeliveryDef::TileArea {
        footprint: program.range.clone(),
        area: SkillTileAreaDeliveryDef {
            mode: SkillTileAreaMode::ReleaseQuery,
            anchor: SkillAreaAnchorSource::Caster,
            tile_origin: SkillTileAreaOrigin::Caster,
            hit_targets: SkillHitTargetFilter::Enemies,
            include_caster: false,
            ..Default::default()
        },
    };
    step.effects.push(AuthoredSkillEffect::new(
        "approach",
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::Control {
                duration_ms: program.duration_ms,
                kind: sustained::ControlKind::ApproachTaunt {
                    profile: program.profile,
                },
            },
        },
    ));
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "apply".into(),
            step_id: "taunt".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    skill.action_policy = Some(program.policy.clone().into());
    program.policy.validate(skill)
}
