mod relation_action;
mod input_plan;
mod status_strike;
mod fusion_placement;
mod paired_areas;
mod prepared_volley;
mod attack_overlay;
mod counter_channel;
mod channel;
mod delayed_area;
mod strike;
mod taunt;
mod ground;
mod parry;
mod scheduled_area;
mod support;
use super::*;
use crate::game::ability::{
    authoring::AuthoredSkillEffect,
    flow::{FlowMilestone, SkillFlowDef},
    sustained::SustainedEffectDef,
    SkillCastSelection, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillKind, SkillTarget,
    SkillTargetReach, SkillUseEffectRef, SkillUseLifetime,
};

pub(super) fn bind(operation: &GraphOperation, skill_id: &SkillId) -> Result<String, String> {
    let mut skill: SkillDef = ron::de::from_str(
        r#"(
        id:"placeholder", name:"", kind:Untargeted, use_lifetime:Action,
        cast_targeting:Explicit(selection:self, target:SelfUnit, reach:pattern, air_capable:false),
        steps:[(id:"finish", delay_ms:0, target:SelfUnit, delivery:Instant, effects:[])]
    )"#,
    )
    .map_err(|e| e.to_string())?;
    skill.id = skill_id.clone();
    if let GraphOperation::ResonanceCounterChannel { program } = operation { return counter_channel::bind(skill, program); }
    skill.resonance_admission = Some(operation.admission());
    let mut additional_skills = Vec::new();
    let definition = match operation {
        GraphOperation::ResonanceStatusStrike { program } => { status_strike::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceFusionPlacement { program } => { fusion_placement::bind(&mut skill, program)?; None }
        GraphOperation::ResonancePairedAreas { program } => { paired_areas::bind(&mut skill, program)?; None }
        GraphOperation::ResonancePreparedVolley { program } => { prepared_volley::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceInputPlan { program } => { input_plan::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceRelationAction { program } => { relation_action::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceAttackOverlay { program, .. } => {
            skill.focus_permissions.allows_basic_attack = true;
            skill.focus_permissions.allows_move = true;
            let (definition, child) = attack_overlay::bind(&skill, program)?;
            additional_skills.push(child);
            Some(definition)
        }
        GraphOperation::ResonanceCounterChannel { .. } => unreachable!("counter channels bind all menu entries above"),
        GraphOperation::ResonanceTileBurst { program, .. } => {
            program.validate()?;
            skill.steps[0].effects.push(AuthoredSkillEffect::new("burst", SkillEffectDef::TileBurst { program: program.effect.clone().into() }));
            skill.flow=Some(SkillFlowDef::Action{id:"burst".into(),step_id:"finish".into(),wait:FlowMilestone::Released}.into());
            None
        }
        GraphOperation::ResonancePresenceBurst { program, .. } => {
            program.validate()?;
            skill.steps[0].effects.push(AuthoredSkillEffect::new("burst", SkillEffectDef::PresenceBurst { program: program.clone().into() }));
            skill.flow=Some(SkillFlowDef::Action{id:"burst".into(),step_id:"finish".into(),wait:FlowMilestone::Released}.into());
            None
        }
        GraphOperation::ResonanceChannelGround { program, .. } => { channel::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceTaunt { program, .. } => { taunt::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceStrike { program, .. } => { strike::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceGroundStatus { program, .. } => { ground::bind_status(&mut skill, program)?; None }
        GraphOperation::ResonanceGround { program, .. } => { ground::bind(&mut skill, program)?; None }
        GraphOperation::ResonanceSupport { program, .. } => {
            support::bind(&mut skill, program)?;
            None
        }
        GraphOperation::ResonanceParry { program, .. } => {
            let (definition, child) = parry::bind(&skill, program)?;
            additional_skills.push(child);
            Some(definition)
        }
        GraphOperation::ResonanceUse { .. } => None,
        GraphOperation::ResonancePreparedBasic { profile, .. } => {
            Some(SustainedEffectDef::PreparedBasic {
                profile: profile.clone(),
            })
        }
        GraphOperation::ResonanceScheduledArea { program, .. } => {
            scheduled_area::bind(&mut skill, program)?;
            None
        }
        GraphOperation::ResonanceDelayedArea { program, .. } => {
            delayed_area::bind(&mut skill, program)?;
            None
        }
        GraphOperation::ResonanceState {
            duration_ms,
            profile,
            ..
        } => Some(SustainedEffectDef::TargetState {
            duration_ms: *duration_ms,
            profile: profile.clone(),
        }),
        GraphOperation::ResonanceReactiveHeal {
            duration_ms,
            profile,
            range,
            ..
        } => {
            skill.kind = SkillKind::Targeted;
            skill.cast_targeting = SkillCastTargetingDef::Explicit {
                selection: SkillCastSelection::PlayerUnit,
                target: SkillTarget::AllySingle,
                reach: SkillTargetReach::Pattern,
                target_range: Some(range.clone()),
                air_capable: true,
            };
            skill.steps[0].target = SkillTarget::AllySingle;
            Some(SustainedEffectDef::ReactiveHeal {
                duration_ms: *duration_ms,
                profile: profile.clone(),
            })
        }
    };
    if let Some(definition) = definition {
        if !matches!(&definition, SustainedEffectDef::PreparedBasic { .. }) {
            skill.use_lifetime = SkillUseLifetime::ActionAndEffects {
                effects: vec![SkillUseEffectRef {
                    step: "finish".into(),
                    effect: "state".into(),
                }],
            };
        }
        skill.steps[0].effects.push(AuthoredSkillEffect::new(
            "state",
            SkillEffectDef::Sustained { definition },
        ));
        skill.flow = Some(
            SkillFlowDef::Action {
                id: "apply".into(),
                step_id: "finish".into(),
                wait: FlowMilestone::Released,
            }
            .into(),
        );
    }
    #[derive(Serialize)]
    struct NativeSkills {
        skills: Vec<SkillDef>,
    }
    additional_skills.insert(0, skill);
    ron::ser::to_string(&NativeSkills {
        skills: additional_skills,
    })
    .map_err(|e| e.to_string())
}
