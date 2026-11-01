use super::*;
use crate::game::ability::paired_areas::PairedAreasDef;
pub(super) fn bind(skill: &mut SkillDef, program: &PairedAreasDef) -> Result<(), String> {
    program.validate()?;
    skill.kind = SkillKind::Targeted;
    skill.cast_targeting = SkillCastTargetingDef::Explicit {
        selection: SkillCastSelection::PlayerTilePair,
        target: SkillTarget::CastTarget,
        reach: SkillTargetReach::Pattern,
        target_range: Some(program.selection_range.clone()),
        air_capable: true,
    };
    skill.steps[0].effects.push(AuthoredSkillEffect::new(
        "pair",
        SkillEffectDef::PairedAreas {
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
