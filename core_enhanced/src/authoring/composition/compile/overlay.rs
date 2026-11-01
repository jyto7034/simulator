use super::*;

impl Compiler<'_> {
    /// Reinput executes a finishing leaf inside the existing transaction. Only
    /// an explicit single released action can use that representation; never
    /// erase authored sequencing, waits, costs or effect ownership.
    pub(super) fn lower_overlay_children(&self, skills: &mut [SkillDef]) -> Result<(), Diagnostic> {
        let children: BTreeSet<_> = skills
            .iter()
            .flat_map(|skill| &skill.steps)
            .flat_map(|step| &step.effects)
            .filter_map(|effect| match &effect.op {
                SkillEffectDef::Sustained {
                    definition:
                        SustainedEffectDef::BasicAttackOverlay {
                            goodbye_skill: Some(child),
                            ..
                        },
                } => Some(child.clone()),
                _ => None,
            })
            .collect();
        for (index, skill) in skills.iter_mut().enumerate() {
            if !children.contains(&skill.id) {
                continue;
            }
            let single_action = matches!(skill.flow.as_deref(),
                Some(SkillFlowDef::Action { step_id, wait: FlowMilestone::Released, .. })
                if skill.steps.len() == 1 && step_id == &skill.steps[0].id);
            if !single_action
                || skill.focus_time_ms != 0
                || skill.resonance_admission.is_some()
                || !matches!(skill.use_lifetime, SkillUseLifetime::Action)
                || skill.action_policy.is_some()
            {
                let path = format!("skills[{index}].body");
                return Err(self.error(DiagnosticCode::Native, &path, &[path.clone()],
                    "overlay finishing skill requires one Released Action, no preparation, admission, owned effects or action modifiers"));
            }
            // Delivery and references are checked by the native validator.
            // Step/effect IDs and source addresses remain unchanged.
            skill.flow = None;
        }
        Ok(())
    }
}
