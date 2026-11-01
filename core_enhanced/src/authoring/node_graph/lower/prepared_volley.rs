use super::*;
use crate::game::{ability::prepared_volley::*, battle::tile_range::TileRangePattern};
impl<'a> Lower<'a> {
    pub(super) fn prepared_volley(&mut self) -> Result<GraphOperation, Diagnostic> {
        let ledger = self.only("skill-credit-ledger")?;
        let recovery = self.only("skill-damage-recovery")?;
        self.link(recovery, "bank", ledger, "bank")?;
        let mark = self.input(recovery, "mark", "skill-damage-mark")?;
        let mark_duration = self.input(mark, "duration", "skill-seconds")?;
        let credit = self.consumer(recovery, "grant", "skill-resource-credit", "grant")?;
        self.link(credit, "in", recovery, "out")?;
        self.link(credit, "bank", ledger, "bank")?;
        let program = self.only("skill-portal-program")?;
        self.link(program, "bank", ledger, "bank")?;
        let policy = self.input(program, "policy", "skill-portal-policy")?;
        let request = self.only("skill-request")?;
        self.self_selection(request)?;
        let qualify = self.only("skill-resonance-qualify")?;
        self.link(qualify, "request", request, "request")?;
        self.link(qualify, "in", request, "out")?;
        let admit = self.only("skill-resonance-admit")?;
        self.link(admit, "permit", qualify, "permit")?;
        self.link(admit, "in", qualify, "out")?;
        self.link(qualify, "program", program, "program")?;
        let admission = self.request_admission(request, qualify)?;
        let run = self.consumer(admit, "out", "skill-run-portals", "in")?;
        self.link(run, "use", admit, "use")?;
        self.link(run, "program", program, "program")?;
        let release = self.consumer(run, "charge", "skill-portal-release-request", "charge")?;
        let freeze = self.consumer(release, "right", "skill-portal-freeze", "right")?;
        self.link(freeze, "in", release, "out")?;
        self.consumer(run, "charge", "skill-portal-stop-input", "charge")?;
        let serial = self.consumer(freeze, "volley", "skill-serial-projectiles", "volley")?;
        self.link(serial, "in", freeze, "out")?;
        let selector = self.input(serial, "selector", "skill-enemy-selector")?;
        if self.text(selector, "order") != "unmarked-route" {
            return Err(self.error(
                selector,
                None,
                "Serial volley selector requires unmarked-route ordering.",
            ));
        }
        self.link(selector, "mark", mark, "mark")?;
        let profile = self.input(serial, "profile", "skill-projectile-profile")?;
        if self.text(profile, "delivery") != "impact" || self.text(profile, "tracking") != "homing"
        {
            return Err(self.error(
                profile,
                None,
                "Serial volley requires a homing impact projectile.",
            ));
        }
        let power = self.input_any(profile, "power")?;
        let area = self.input(profile, "area", "skill-range")?;
        let after_ids: Vec<_> = self.doc.nodes[profile]
            .inputs
            .get("after")
            .into_iter()
            .flatten()
            .map(|o| o.source.node.as_str())
            .collect();
        let mut after = Vec::new();
        for id in after_ids {
            after.push(self.shot_status(id)?);
        }
        let projectile = HomingImpactDef {
            definition: profile.into(),
            power: self.shot_power(power, 0)?,
            damage_type: self.shot_damage_type(profile)?,
            speed: self.counter_ratio(profile, "speed")?,
            range: self.counter_ratio(profile, "range")?,
            width: self.counter_ratio(profile, "width")?,
            lifetime_ms: self.positive_duration(profile, "lifetime")?,
            area: self.tile_pattern(profile, "area")?,
            air: self.shot_choice(profile, "air", "include", "exclude")?,
            obstacle_stop: self.shot_choice(profile, "obstacle", "stop", "pass")?,
            after,
        };
        self.consumer(serial, "batch", "skill-projectile-results", "batch")?;
        let ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.starts_with("skill-modifier-"))
            .map(|(id, _)| id.as_str())
            .collect();
        let mut modifiers = Vec::new();
        for id in ids {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let relic_definition = self.text(condition, "definition").into();
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-duration" => {
                    self.link(id, "target", mark_duration, "value")?;
                    let (negative, percent) = self.signed_period_ratio(id)?;
                    VolleyChange::MarkDuration { negative, percent }
                }
                "skill-modifier-grant" => {
                    self.link(id, "target", recovery, "grant")?;
                    VolleyChange::Recovery {
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-interruption" => {
                    self.link(id, "program", program, "program")?;
                    VolleyChange::Protection
                }
                "skill-modifier-projectile-multihit" => {
                    self.link(id, "target", profile, "profile")?;
                    VolleyChange::Multihit {
                        step: self.counter_ratio(id, "step")?,
                        cap: self.counter_ratio(id, "cap")?,
                    }
                }
                "skill-modifier-projectile-status" => {
                    self.link(id, "target", profile, "profile")?;
                    let s = self.input(id, "status", "skill-hit-status")?;
                    VolleyChange::Status {
                        status: self.shot_status(s)?,
                    }
                }
                "skill-modifier-continuous-volley" => {
                    self.link(id, "target", program, "program")?;
                    VolleyChange::Continuous {
                        percent: self.counter_ratio(id, "percent")?,
                        cost_interval_ms: self.positive_duration(id, "costInterval")?,
                        interval_ms: self.positive_duration(id, "interval")?,
                    }
                }
                "skill-modifier-range" => {
                    self.link(id, "target", area, "range")?;
                    let range = &projectile.area;
                    let pattern = match self.text(id, "mode") {
                        "replace" => self.tile_pattern(id, "replacement")?,
                        "square-step" if range.include_anchor_tile && range.rows == ["@"] => {
                            TileRangePattern {
                                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                                include_anchor_tile: true,
                                ..range.clone()
                            }
                        }
                        "square-step"
                            if range.include_anchor_tile && range.rows == ["XXX", "X@X", "XXX"] =>
                        {
                            TileRangePattern {
                                rows: vec![
                                    "XXXX".into(),
                                    "X@XX".into(),
                                    "XXXX".into(),
                                    "XXXX".into(),
                                ],
                                include_anchor_tile: true,
                                ..range.clone()
                            }
                        }
                        _ => {
                            return Err(self.error(
                                id,
                                None,
                                "Declare a replacement mask for this area.",
                            ))
                        }
                    };
                    VolleyChange::Area { pattern }
                }
                _ => return Err(self.error(id, None, "Unsupported prepared volley modifier.")),
            };
            modifiers.push(VolleyModifier {
                id: id.into(),
                relic_definition,
                change,
            });
        }
        let program = PreparedVolleyDef {
            definition: program.into(),
            admission,
            ledger: CreditLedgerDef {
                definition: ledger.into(),
                grant: self.number(ledger, "grant")?,
                credit_limit: self.number(ledger, "creditLimit")?,
            },
            mark_definition: mark.into(),
            mark_duration_ms: self.positive_duration(mark, "duration")?,
            recovery: self.number(credit, "amount")?,
            policy: PreparationPolicy {
                maximum: self.number(policy, "maximum")?,
                first_ms: self.positive_duration(policy, "first")?,
                last_ms: self.positive_duration(policy, "last")?,
                hold_ms: self.positive_duration(policy, "hold")?,
                deadline_ms: self.positive_duration(policy, "deadline")?,
                decay_ms: self.positive_duration(policy, "decay")?,
            },
            selection_range: self.tile_pattern(selector, "range")?,
            selection_air: self.shot_choice(selector, "air", "include", "exclude")?,
            projectile,
            modifiers,
        };
        program.validate().map_err(|e| self.error(run, None, &e))?;
        Ok(GraphOperation::ResonancePreparedVolley { program })
    }
}
