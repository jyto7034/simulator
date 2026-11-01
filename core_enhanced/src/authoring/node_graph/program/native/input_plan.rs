use super::*;
use crate::game::ability::input_plan::InputPlanDef;
pub(super) fn bind(skill: &mut SkillDef, program: &InputPlanDef) -> Result<(), String> {
    program.validate()?;
    skill.steps[0].effects.push(AuthoredSkillEffect::new(
        "volley",
        SkillEffectDef::InputPlan {
            program: program.clone().into(),
        },
    ));
    skill.use_lifetime = SkillUseLifetime::ActionAndEffects {
        effects: vec![SkillUseEffectRef {
            step: "finish".into(),
            effect: "volley".into(),
        }],
    };
    skill.flow = Some(
        SkillFlowDef::Action {
            id: "volley".into(),
            step_id: "finish".into(),
            wait: FlowMilestone::Released,
        }
        .into(),
    );
    Ok(())
}
