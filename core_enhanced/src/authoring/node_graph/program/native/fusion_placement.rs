use super::*;
use crate::game::ability::mobile_effect::FusionPlacementDef;
pub(super) fn bind(skill: &mut SkillDef, program: &FusionPlacementDef) -> Result<(), String> {
    program.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::PlayerTile,
        target: SkillTarget::CastTarget,
        reach: SkillTargetReach::Pattern,
        target_range: Some(program.selection_range.clone()),
        air_capable: true,
    };
    skill.steps[0].effects.push(AuthoredSkillEffect::new(
        "fusion",
        SkillEffectDef::FusionPlacement {
            program: program.clone().into(),
        },
    ));
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "install".into(),
            step_id: "finish".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}
