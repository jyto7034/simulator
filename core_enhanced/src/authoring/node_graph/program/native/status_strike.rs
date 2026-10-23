use super::*;
use crate::game::ability::status_strike::StatusStrikeDef;
pub(super) fn bind(skill: &mut SkillDef, program: &StatusStrikeDef) -> Result<(), String> {
    program.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::PlayerDirection,
        target: SkillTarget::CastTarget,
        reach: SkillTargetReach::Pattern,
        target_range: Some(program.range.clone()),
        air_capable: true,
    };
    skill.steps[0].effects.push(AuthoredSkillEffect::new(
        "status-strike",
        SkillEffectDef::StatusStrike {
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
