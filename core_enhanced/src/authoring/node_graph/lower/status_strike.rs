use super::*;
use crate::game::{
    ability::{mobile_effect::*, status_strike::*},
    battle::damage::DamageType,
};
impl<'a> Lower<'a> {
    fn reactive_status(&mut self, id: &'a str) -> Result<ReactiveStatusDef, Diagnostic> {
        self.used.insert(id);
        Ok(ReactiveStatusDef {
            definition: format!("{}:{id}", self.doc.document),
            kind: match self.text(id, "effect") {
                "slow" => ReactiveKind::Slow {
                    percent: self.number(id, "percent")?,
                },
                "freeze" => ReactiveKind::Freeze,
                _ => return Err(self.error(id, None, "Unresolved reactive status kind.")),
            },
            duration_ms: self.positive_duration(id, "duration")?,
            refresh: match self.text(id, "refresh") {
                "refresh" => true,
                "preserve" => false,
                _ => return Err(self.error(id, None, "Unresolved status refresh policy.")),
            },
            modifiers: Vec::new(),
        })
    }
    pub(super) fn status_strike_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let selection = self.input(request, "selection", "skill-target-selection")?;
        if self.text(selection, "mode") != "player-direction" {
            return Err(self.error(
                selection,
                None,
                "Status strike requires explicit direction selection.",
            ));
        }
        let convert = self.input(plan, "body", "skill-convert-status-plan")?;
        let spawn = self.input(plan, "after", "skill-spawn-mobile-plan")?;
        let strike = self.input(convert, "body", "skill-strike-plan")?;
        if self.text(strike, "target") != "selected-direction-area" {
            return Err(self.error(strike, None, "Expected a selected direction area strike."));
        }
        let from = self.input(convert, "from", "skill-reactive-status")?;
        let to = self.input(convert, "to", "skill-reactive-status")?;
        self.link(from, "blockedBy", to, "status")?;
        let mut from_def = self.reactive_status(from)?;
        let mut to_def = self.reactive_status(to)?;
        let subscriptions: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-owned-damage-status")
            .map(|(id, _)| id.as_str())
            .collect();
        if subscriptions.len() > 1 {
            return Err(self.error(
                plan,
                None,
                "Only one owned damage subscription is supported.",
            ));
        }
        for id in &subscriptions {
            self.used.insert(id);
            self.link(id, "status", from, "status")?;
        }
        let profile = self.input_any(spawn, "profile")?;
        let (mut mobile, _) = self.mobile_profile(profile, 0)?;
        let limit_id = self.input(spawn, "limit", "skill-count-positive")?;
        let limit = self.number(limit_id, "value")?;
        let deaths: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-status-death-mobile")
            .map(|(id, _)| id.as_str())
            .collect();
        if deaths.len() > 1 {
            return Err(self.error(plan, None, "Only one status death generator is supported."));
        }
        let death = if let Some(&id) = deaths.first() {
            self.used.insert(id);
            self.link(id, "profile", profile, "profile")?;
            self.link(id, "limit", limit_id, "count")?;
            let states = self.doc.nodes[id]
                .inputs
                .get("states")
                .ok_or_else(|| self.error(id, Some("states"), "Missing status sources."))?;
            let mut names = Vec::new();
            for s in states {
                if s.source.port != "status" || ![from, to].contains(&s.source.node.as_str()) {
                    return Err(self.error(
                        id,
                        Some("states"),
                        "Death sources must be emitted status definitions.",
                    ));
                }
                names.push(format!("{}:{}", self.doc.document, s.source.node));
            }
            Some(StatusDeathMobileDef {
                definition: format!("{}:{id}", self.doc.document),
                relic_definition: self.text(id, "relic").into(),
                states: names,
            })
        } else {
            None
        };
        let range_id = self.input_any(strike, "range")?;
        let mobile_range = self.input_any(profile, "range")?;
        let from_duration = self.input_any(from, "duration")?;
        let to_duration = self.input_any(to, "duration")?;
        let mut range_modifiers = Vec::new();
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.starts_with("skill-modifier-"))
            .map(|(id, _)| id.as_str())
            .collect();
        for id in modifiers {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let relic = self.text(condition, "definition").to_owned();
            let kind = self.doc.nodes[id].kind.as_str();
            if kind == "skill-modifier-control-vulnerability" {
                self.link(id, "reactive", to, "status")?;
                to_def.modifiers.push(ReactiveModifier {
                    id: id.into(),
                    relic_definition: relic,
                    change: ReactiveChange::Vulnerability {
                        percent: self.counter_ratio(id, "percent")?,
                        duration_ms: self.positive_duration(id, "duration")?,
                    },
                });
                continue;
            }
            let target = self.input_any(id, "target")?;
            if kind == "skill-modifier-duration" {
                let (negative, percent) = self.signed_period_ratio(id)?;
                let d = if target == from_duration {
                    &mut from_def
                } else if target == to_duration {
                    &mut to_def
                } else {
                    return Err(self.error(
                        id,
                        None,
                        "Duration modifiers bind to emitted statuses, not mobile delay.",
                    ));
                };
                d.modifiers.push(ReactiveModifier {
                    id: id.into(),
                    relic_definition: relic,
                    change: ReactiveChange::Duration { negative, percent },
                });
                continue;
            }
            let change = match kind {
                "skill-modifier-mobile-power" if target == profile => MobileChange::Power {
                    percent: self.counter_ratio(id, "percent")?,
                },
                "skill-modifier-mobile-vulnerability" if target == profile => {
                    MobileChange::Vulnerability {
                        percent: self.counter_ratio(id, "percent")?,
                        duration_ms: self.positive_duration(id, "duration")?,
                    }
                }
                "skill-modifier-range" if target == range_id || target == mobile_range => {
                    if self.text(id, "mode") != "replace" {
                        return Err(self.error(
                            id,
                            None,
                            "Use an explicit replacement mask for directional and chase areas.",
                        ));
                    }
                    MobileChange::Range {
                        pattern: self.mobile_pattern(id, "replacement", 0)?,
                    }
                }
                _ => return Err(self.error(
                    id,
                    None,
                    "Modifier is not bound to an emitted status, strike range or mobile effect.",
                )),
            };
            let m = MobileModifier {
                id: id.into(),
                relic_definition: relic,
                change,
            };
            if target == range_id {
                range_modifiers.push(m);
            } else {
                mobile.modifiers.push(m);
            }
        }
        let power = self.input_any(strike, "power")?;
        let program = StatusStrikeDef {
            definition: format!("{}:{plan}", self.doc.document),
            admission,
            range: self.tile_pattern(strike, "range")?,
            range_modifiers,
            power: self.mobile_power(power, 0)?,
            damage_type: match self.text(strike, "damageType") {
                "physical" => DamageType::Physical,
                "magic" => DamageType::Magic,
                _ => return Err(self.error(strike, None, "Unresolved strike damage type.")),
            },
            from: from_def,
            to: to_def,
            owned_damage: !subscriptions.is_empty(),
            mobile,
            limit,
            death,
        };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceStatusStrike { program })
    }
}
