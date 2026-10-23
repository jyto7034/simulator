use super::*;
use crate::game::ability::{counter_channel::CounterChannelDef, *};
pub(super) fn bind(template: SkillDef, program: &CounterChannelDef) -> Result<String, String> {
    program.validate()?;
    let root = template.id.clone();
    let mut skills = Vec::new();
    for (entry, e) in program.entries.iter().enumerate() {
        let mut skill = template.clone();
        skill.id = program.entry_skill_id(&root, entry);
        skill.resonance_admission = Some(e.admission);
        skill.kind = SkillKind::Targeted;
        skill.cast_targeting = SkillCastTargetingDef::Explicit {
            selection: SkillCastSelection::PlayerTile,
            target: SkillTarget::CastTarget,
            reach: if e.automatic {
                SkillTargetReach::Global
            } else {
                SkillTargetReach::Pattern
            },
            target_range: e.selection_range.clone(),
            air_capable: true,
        };
        skill.steps[0].effects.push(AuthoredSkillEffect::new(
            "channel",
            SkillEffectDef::CounterChannel {
                program: program.clone().into(),
                root: root.clone(),
                entry,
            },
        ));
        skill.use_lifetime = SkillUseLifetime::ActionAndEffects {
            effects: vec![SkillUseEffectRef {
                step: "finish".into(),
                effect: "channel".into(),
            }],
        };
        skill.flow = Some(
            SkillFlowDef::Action {
                id: "channel".into(),
                step_id: "finish".into(),
                wait: FlowMilestone::Released,
            }
            .into(),
        );
        skills.push(skill);
    }
    #[derive(Serialize)]
    struct NativeSkills {
        skills: Vec<SkillDef>,
    }
    ron::ser::to_string(&NativeSkills { skills }).map_err(|e| e.to_string())
}
