use super::*;
use crate::game::{ability::sustained::BasicLineProfile, battle::tile_range::TileRangePattern};

impl<'a> Lower<'a> {
    pub(super) fn attack_overlay_action(&mut self) -> Result<GraphOperation, Diagnostic> {
        let request = self.only("skill-request")?;
        self.self_selection(request)?;
        let state = self.only("skill-state")?;
        let lookup = self.only("skill-lookup")?;
        self.link(lookup, "in", request, "out")?;
        self.link(lookup, "request", request, "request")?;
        self.link(lookup, "state", state, "state")?;
        let qualify = self.only("skill-resonance-qualify")?;
        self.link(qualify, "in", lookup, "absent")?;
        self.link(qualify, "vacancy", lookup, "vacancy")?;
        self.link(qualify, "request", request, "request")?;
        let admission = self.request_admission(request, qualify)?;
        let admit = self.only("skill-resonance-admit")?;
        self.link(admit, "in", qualify, "out")?;
        self.link(admit, "permit", qualify, "permit")?;
        let install = self.only("skill-resonance-install")?;
        self.link(install, "in", admit, "out")?;
        self.link(install, "use", admit, "use")?;
        self.link(install, "state", state, "state")?;
        let duration_ms = self.positive_duration(install, "duration")?;
        let profile = self.input(install, "profile", "skill-attack-profile")?;
        self.link(install, "profile", profile, "profile")?;
        let dimension = |this: &Self, field: &str| -> Result<u32, Diagnostic> {
            let (n, d) = this.exact_ratio(profile, field)?;
            let scaled = u128::from(n) * 1_000_000;
            if scaled % u128::from(d) != 0 {
                return Err(this.error(
                    profile,
                    Some(field),
                    "Line dimensions require exact native world units.",
                ));
            }
            u32::try_from(scaled / u128::from(d)).map_err(|_| {
                this.error(profile, Some(field), "Line dimension exceeds native range.")
            })
        };
        let mut line = BasicLineProfile {
            length_units: dimension(self, "length")?,
            width_units: dimension(self, "width")?,
            power: self.number(profile, "power")?,
            weapon_effects: self.doc.nodes[profile].values.get("weaponEffects")
                == Some(&FieldValue::Boolean(true)),
            piercing_relics: self.doc.nodes[profile].values.get("piercingRelics")
                == Some(&FieldValue::Boolean(true)),
            piercing: None,
        };
        if self
            .doc
            .nodes
            .values()
            .any(|n| n.kind == "skill-modifier-piercing")
        {
            let modifier = self.only("skill-modifier-piercing")?;
            self.link(modifier, "target", profile, "profile")?;
            if self.text(modifier, "curve") != "capped-linear" {
                return Err(self.error(modifier,Some("curve"),"Resolve the piercing curve; this target supports authored capped-linear growth."));
            }
            let condition = self.taunt_condition(modifier)?;
            line.piercing = Some(crate::game::ability::sustained::BasicPiercingRule {
                relic_definition: self.text(condition, "definition").into(),
                increment_percent: self.number(modifier, "step")?,
                cap_percent: self.number(modifier, "cap")?,
            });
        }
        let permission = self.only("skill-permission")?;
        self.link(permission, "in", lookup, "present")?;
        self.link(permission, "request", request, "request")?;
        self.link(permission, "activation", lookup, "activation")?;
        if self.text(permission, "permission") != "manual-followup" {
            return Err(self.error(
                permission,
                Some("permission"),
                "Use the common manual followup policy.",
            ));
        }
        let claim = self.only("skill-claim")?;
        self.link(claim, "in", permission, "out")?;
        self.link(claim, "permit", permission, "permit")?;
        if self.text(claim, "expiryOrder") != "expiry-first" {
            return Err(self.error(
                claim,
                Some("expiryOrder"),
                "Expiration takes precedence over reinput.",
            ));
        }
        let capture = self.only("skill-capture-time")?;
        self.link(capture, "in", claim, "out")?;
        self.link(capture, "activation", claim, "activation")?;
        if self.text(capture, "moment") != "accepted" {
            return Err(self.error(
                capture,
                Some("moment"),
                "This immediate finish captures time at accepted reinput.",
            ));
        }
        let manual = self
            .doc
            .nodes
            .iter()
            .find(|(_, n)| {
                n.kind == "skill-finish"
                    && n.values.get("reason") == Some(&FieldValue::Text("manual".into()))
            })
            .map(|(id, _)| id.as_str())
            .ok_or_else(|| self.error(capture, None, "Manual finish is required."))?;
        self.used.insert(manual);
        self.link(manual, "in", capture, "out")?;
        self.link(manual, "activation", claim, "activation")?;
        let damage = self.only("skill-area-damage")?;
        self.link(damage, "in", manual, "out")?;
        let power = self.input(damage, "power", "skill-time-power")?;
        self.link(power, "seconds", capture, "seconds")?;
        let cap = match self.text(power, "curve") {
            "linear" => None,
            "capped-linear" => Some(self.number(power, "cap")?),
            _ => return Err(self.error(power, Some("curve"), "Resolve the finish power curve.")),
        };
        let area = self.input(damage, "area", "skill-area")?;
        if self.text(area, "facing") != "accepted" {
            return Err(self.error(
                area,
                Some("facing"),
                "This immediate finish locks the accepted reinput facing.",
            ));
        }
        let finish_range = TileRangePattern {
            include_anchor_tile: self.doc.nodes[area].values.get("includeAnchor")
                == Some(&FieldValue::Boolean(true)),
            rows: self
                .text(area, "pattern")
                .split('\n')
                .map(str::to_owned)
                .collect(),
        };
        let mut cue_sources = vec![(install, "cue"), (damage, "cue"), (damage, "completed")];
        for (kind, reason) in [
            ("skill-expiry", "expiry"),
            ("skill-owner-exit", "owner-exit"),
        ] {
            let event = self.only(kind)?;
            self.link(event, "state", state, "state")?;
            if reason == "owner-exit" && self.text(event, "policy") != "end-active" {
                return Err(self.error(
                    event,
                    Some("policy"),
                    "Owner exit must end this activation.",
                ));
            }
            let finish = self
                .doc
                .nodes
                .iter()
                .find(|(_, n)| {
                    n.kind == "skill-finish"
                        && n.values.get("reason") == Some(&FieldValue::Text(reason.into()))
                })
                .map(|(id, _)| id.as_str())
                .ok_or_else(|| {
                    self.error(event, None, "Lifecycle event requires an explicit finish.")
                })?;
            self.used.insert(finish);
            self.link(finish, "in", event, "out")?;
            self.link(finish, "activation", event, "activation")?;
            cue_sources.push((finish, "cue"));
        }
        let basic = self.only("skill-basic-event")?;
        self.link(basic, "state", state, "state")?;
        cue_sources.push((basic, "cue"));
        for (id, node) in &self.doc.nodes {
            if node.kind == "skill-cue" {
                if !node.inputs.get("event").is_some_and(|inputs| {
                    !inputs.is_empty()
                        && inputs.iter().all(|edge| {
                            cue_sources
                                .contains(&(edge.source.node.as_str(), edge.source.port.as_str()))
                        })
                }) {
                    return Err(self.error(
                        id,
                        Some("event"),
                        "Presentation must observe a bound attack or lifecycle event.",
                    ));
                }
                self.used.insert(id);
            }
        }
        let damage_type = match self.text(damage, "damageType") {
            "physical" => crate::game::battle::damage::DamageType::Physical,
            "magic" => crate::game::battle::damage::DamageType::Magic,
            "true" => crate::game::battle::damage::DamageType::True,
            _ => {
                return Err(self.error(
                    damage,
                    Some("damageType"),
                    "Resolve the finish damage type.",
                ))
            }
        };
        let program = AttackOverlayProgram {
            duration_ms,
            line,
            damage_type,
            finish_range,
            base: self.number(power, "base")?,
            per_second: self.number(power, "rate")?,
            cap,
        };
        program
            .validate()
            .map_err(|e| self.error(install, None, &e))?;
        Ok(GraphOperation::ResonanceAttackOverlay { admission, program })
    }
}
