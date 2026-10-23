use super::*;
use crate::game::ability::relation_action::RelationActionDef;
pub(super) fn bind(skill: &mut SkillDef, program: &RelationActionDef) -> Result<(), String> {
    program.validate()?;
    if let Some(range) = &program.selection_range {
        skill.kind = SkillKind::Targeted;
        skill.cast_targeting = SkillCastTargetingDef::Explicit {
            selection: SkillCastSelection::PlayerTile,
            target: SkillTarget::CastTarget,
            reach: SkillTargetReach::Pattern,
            target_range: Some(range.clone()),
            air_capable: false,
        };
    }
    // A pending next-basic entry does not pause movement or the basic clock.
    // The admitted native action owns those restrictions only once it starts.
    skill.focus_permissions.allows_basic_attack = true;
    skill.focus_permissions.allows_move = true;
    skill.steps[0].effects.push(AuthoredSkillEffect::new(
        "action",
        SkillEffectDef::RelationAction {
            program: program.clone().into(),
        },
    ));
    skill.use_lifetime = SkillUseLifetime::ActionAndEffects {
        effects: vec![SkillUseEffectRef {
            step: "finish".into(),
            effect: "action".into(),
        }],
    };
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "action".into(),
            step_id: "finish".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}
