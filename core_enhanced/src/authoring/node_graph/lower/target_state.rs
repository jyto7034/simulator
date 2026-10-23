use super::*;
use crate::game::ability::sustained::TargetStateProfile;

impl<'a> Lower<'a> {
    pub(super) fn text(&self, id: &str, field: &str) -> &str {
        match self.doc.nodes[id].values.get(field) {
            Some(FieldValue::Text(value)) => value,
            _ => "",
        }
    }
    pub(super) fn target_state_action(
        &mut self,
        request: &str,
        qualify: &str,
        admit: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let run = self.only("skill-run-action")?;
        self.link(run, "in", admit, "out")?;
        self.link(run, "use", admit, "use")?;
        let entry = self.input(run, "program", "skill-action-entry")?;
        self.link(qualify, "program", entry, "program")?;
        if self.text(entry, "start") != "immediate"
            || self.text(entry, "interruption") != "standard"
        {
            return Err(self.error(
                entry,
                None,
                "This state plan requires immediate start and standard interruption.",
            ));
        }
        let plan = self.input_any(entry, "plan")?;
        if self.doc.nodes.values().any(|node| node.kind == "skill-convert-status-plan") { return self.status_strike_action(request, plan, admission); }
        if self.doc.nodes.values().any(|node| node.kind == "skill-strike-plan") {
            return self.strike_action(request, entry, plan, admission);
        }
        if matches!(
            self.doc.nodes[plan].kind.as_str(),
            "skill-sequence-plan" | "skill-on-hit-plan" | "skill-delayed-area-plan"
        ) {
            return self.delayed_area_action(request, entry, plan, admission);
        }
        if self.doc.nodes[entry].inputs.contains_key("selector") {
            return Err(self.error(entry, Some("selector"), "This plan selects recipients inside its body and cannot ignore an admission selector."));
        }
        if matches!(self.doc.nodes[plan].kind.as_str(), "skill-path-burst-plan"|"skill-stack-burst-plan") { return self.tile_burst_action(request,plan,admission); }
        if self.doc.nodes[plan].kind == "skill-mark-burst-plan" { return self.presence_burst_action(request,plan,admission); }
        if self.doc.nodes[plan].kind == "skill-place-fusion-plan" { return self.fusion_action(request, plan, admission); }
        if self.doc.nodes[plan].kind == "skill-parry-plan" {
            return self.parry_action(request, plan, admission);
        }
        if self.doc.nodes[plan].kind == "skill-next-basic-plan" {
            return self.prepared_basic_action(request, plan, admission);
        }
        if self.doc.nodes[plan].kind == "skill-scheduled-plan" {
            return self.scheduled_area_action(request, plan, admission);
        }
        if self.doc.nodes[plan].kind == "skill-reactive-heal-plan" {
            return self.reactive_heal_action(request, plan, admission);
        }
        if self.doc.nodes[plan].kind == "skill-aura-buff-plan"
            || (self.doc.nodes[plan].kind == "skill-recipient-buff-plan"
                && self.text(plan, "recipient") == "allies")
        {
            return self.support_action(request, run, plan, admission);
        }
        if self.doc.nodes[plan].kind != "skill-recipient-buff-plan" {
            return Err(self.error(plan, None, "Unsupported native action plan."));
        }
        self.self_selection(request)?;
        if self.doc.nodes[plan].inputs.contains_key("range") {
            return Err(self.error(plan, Some("range"), "Self state has no range input."));
        }
        if self.text(plan, "recipient") != "self" {
            return Err(self.error(
                plan,
                None,
                "This native state plan currently supports self recipients.",
            ));
        }
        let state = self.input(plan, "state", "skill-target-state")?;
        let (duration_ms, profile) = self.state_profile(state)?;
        Ok(GraphOperation::ResonanceState {
            admission,
            duration_ms,
            profile,
        })
    }
    pub(super) fn state_profile(
        &mut self,
        state: &'a str,
    ) -> Result<(u32, TargetStateProfile), Diagnostic> {
        let duration_ms = self.positive_duration(state, "duration")?;
        let mut profile = TargetStateProfile {
            definition: format!("{}:{state}", self.doc.document),
            attack_percent: 0,
            defense_percent: 0,
            attack_speed_points: 0,
            hp_floor_one: false,
            end_on_source_exit: match self.text(state, "sourceExit") {
                "end" => true,
                "independent" => false,
                _ => {
                    return Err(self.error(state, None, "State source-exit policy is unresolved."))
                }
            },
        };
        let mut properties = BTreeSet::new();
        for origin in &self.doc.nodes[state].inputs["effects"] {
            let id = origin.source.node.as_str();
            self.used.insert(id);
            let property = match self.doc.nodes[id].kind.as_str() {
                "skill-attribute-effect" => {
                    let amount = self.number(id, "amount")?;
                    if amount == 0 {
                        return Err(self.error(id, None, "State increases must be positive."));
                    }
                    let property = self.text(id, "attribute");
                    match (property, self.text(id, "operation")) {
                        ("attack", "percent") => profile.attack_percent = amount,
                        ("defense", "percent") => profile.defense_percent = amount,
                        ("attack-speed", "points") => profile.attack_speed_points = amount,
                        _ => return Err(self.error(id, None, "This native state profile does not support this attribute/operation pair.")),
                    }
                    property
                }
                "skill-hp-floor-effect" => {
                    if self.number(id, "minimum")? != 1 {
                        return Err(self.error(
                            id,
                            None,
                            "Native survival currently supports HP floor 1.",
                        ));
                    }
                    profile.hp_floor_one = true;
                    "hp-floor"
                }
                _ => {
                    return Err(self.error(
                        id,
                        None,
                        "This native state profile does not support this effect.",
                    ))
                }
            };
            if !properties.insert(property.to_owned()) {
                return Err(self.error(
                    id,
                    None,
                    "Duplicate state property requires an explicit composition policy.",
                ));
            }
        }
        profile
            .validate()
            .map_err(|e| self.error(state, None, &e))?;
        Ok((duration_ms, profile))
    }
}
