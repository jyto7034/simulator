use super::*;

impl<'a> Lower<'a> {
    fn support_attributes(&mut self, plan: &'a str) -> Result<(u32, u32), Diagnostic> {
        let mut damage = None;
        let mut speed = None;
        for origin in self.doc.nodes[plan]
            .inputs
            .get("effects")
            .into_iter()
            .flatten()
        {
            let id = origin.source.node.as_str();
            self.used.insert(id);
            if self.doc.nodes[id].kind != "skill-attribute-effect" {
                return Err(self.error(
                    id,
                    None,
                    "Support requires explicit regular basic coefficients.",
                ));
            }
            let amount = self.number(id, "amount")?;
            let previous = match (self.text(id, "attribute"), self.text(id, "operation")) {
                ("basic-power", "percent") => damage.replace(amount),
                ("basic-speed", "points") => speed.replace(amount),
                _ => {
                    return Err(self.error(
                        id,
                        None,
                        "Only regular basic power and speed are supported here.",
                    ))
                }
            };
            if previous.is_some() {
                return Err(self.error(id, None, "Duplicate support property."));
            }
        }
        match (damage, speed) {
            (Some(damage), Some(speed)) => Ok((damage, speed)),
            _ => Err(self.error(
                plan,
                None,
                "Both regular basic coefficients must be explicit.",
            )),
        }
    }
    pub(super) fn support_action(
        &mut self,
        request: &str,
        run: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let range = self.tile_pattern(plan, "range")?;
        let (duration_ms, effect) = if self.doc.nodes[plan].kind == "skill-aura-buff-plan" {
            let stop = self.only("skill-action-stop-input")?;
            self.link(stop, "run", run, "run")?;
            let (damage_percent, speed_points) = self.support_attributes(plan)?;
            (
                self.positive_duration(plan, "duration")?,
                SupportEffectProgram::Channel {
                    damage_percent,
                    speed_points,
                },
            )
        } else {
            if self.text(plan, "recipient") != "allies" {
                return Err(self.error(
                    plan,
                    None,
                    "Personal shielding requires allied recipients.",
                ));
            }
            let state = self.input(plan, "state", "skill-target-state")?;
            if self.text(state, "sourceExit") != "end" {
                return Err(self.error(state, None, "Personal shielding ends on provider exit."));
            }
            let shield = self.input(state, "effects", "skill-personal-shield-effect")?;
            (
                self.positive_duration(state, "duration")?,
                SupportEffectProgram::PersonalShield {
                    max_health_percent: self.number(shield, "percent")?,
                },
            )
        };
        let mut conversion = None;
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-modifier-single-support")
            .map(|(id, _)| id.as_str())
            .collect();
        for modifier in modifiers {
            self.used.insert(modifier);
            self.link(modifier, "target", plan, "plan")?;
            if conversion.is_some() {
                return Err(self.error(
                    modifier,
                    None,
                    "One support conversion is allowed per effect.",
                ));
            }
            let condition = self.input(modifier, "condition", "skill-equipment-condition")?;
            if self.text(condition, "slot") != "relic"
                || self.text(condition, "variant") != "normal"
                || self.text(condition, "copies") != "once"
                || self.doc.nodes[condition].inputs.contains_key("requires")
            {
                return Err(self.error(
                    condition,
                    None,
                    "This conversion requires one normal equipped relic without prerequisites.",
                ));
            }
            let replacement = match (&effect, self.text(modifier, "mode")) {
                (SupportEffectProgram::PersonalShield { .. }, "shield") => {
                    if self.doc.nodes[modifier].inputs.contains_key("effects") {
                        return Err(self.error(
                            modifier,
                            None,
                            "Personal shield conversion cannot replace attributes.",
                        ));
                    }
                    let (numerator, denominator) = self.exact_ratio(modifier, "factor")?;
                    SupportReplacementProgram::PersonalShield {
                        numerator,
                        denominator,
                    }
                }
                (SupportEffectProgram::Channel { .. }, "attributes") => {
                    let (damage_percent, speed_points) = self.support_attributes(modifier)?;
                    SupportReplacementProgram::Channel {
                        damage_percent,
                        speed_points,
                    }
                }
                _ => {
                    return Err(self.error(
                        modifier,
                        None,
                        "Support conversion mode does not match its effect.",
                    ))
                }
            };
            conversion = Some(SupportConversionProgram {
                relic_definition: self.text(condition, "definition").into(),
                selection_range: self.tile_pattern(modifier, "range")?,
                replacement,
            });
        }
        let mut policy = crate::game::ability::action::ActionPolicy::default();
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-modifier-duration" | "skill-modifier-range"
                )
            })
            .map(|(id, _)| id.as_str())
            .collect();
        for id in modifiers {
            if !matches!(effect, SupportEffectProgram::Channel { .. }) {
                return Err(self.error(
                    id,
                    None,
                    "Support modifiers currently require a channel aura.",
                ));
            }
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            use crate::game::ability::action::*;
            let change = if self.doc.nodes[id].kind == "skill-modifier-duration" {
                let duration = self.input(plan, "duration", "skill-seconds")?;
                self.link(id, "target", duration, "value")?;
                ActionChange::Duration {
                    step: "finish".into(),
                    effect: "support".into(),
                    percent: self.number(id, "percent")?,
                }
            } else {
                let range = self.input(plan, "range", "skill-range")?;
                self.link(id, "target", range, "range")?;
                if self.text(id, "mode") != "replace" {
                    return Err(self.error(
                        id,
                        None,
                        "Support footprint expansion requires explicit replacement tiles.",
                    ));
                }
                ActionChange::SupportRange {
                    step: "finish".into(),
                    effect: "support".into(),
                    pattern: self.tile_pattern(id, "replacement")?,
                }
            };
            policy.modifiers.push(ActionModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                variant: crate::game::relic::RelicVariant::Normal,
                change,
            });
        }
        let program = SupportProgram {
            family: format!("{}:{plan}", self.doc.document),
            duration_ms,
            range,
            effect,
            conversion,
            policy,
        };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceSupport { admission, program })
    }
}
