use super::*;
use crate::game::ability::{action::*, ground::*};
use crate::game::relic::RelicVariant;
impl<'a> Lower<'a> {
    pub(super) fn channel_ground_action(
        &mut self,
        request: &str,
        admit: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let channel = self.only("skill-channel-begin")?;
        let result = self.input(channel, "in", "skill-resonance-result")?;
        self.link(channel, "in", result, "out")?;
        self.link(result, "in", admit, "out")?;
        self.link(result, "receipt", admit, "receipt")?;
        self.link(channel, "use", admit, "use")?;
        let duration = self.input(channel, "duration", "skill-seconds")?;
        let duration_ms = self.positive_duration(channel, "duration")?;
        let pause_basic = match self.text(channel, "basicAttack") {
            "pause" => true,
            "allow" => false,
            _ => {
                return Err(self.error(
                    channel,
                    Some("basicAttack"),
                    "Resolve the channel basic-attack policy.",
                ))
            }
        };
        let mut policy = ActionPolicy {
            external_interrupt_protected: match self.text(channel, "interruption") {
                "standard" => false,
                "protected" => true,
                _ => {
                    return Err(self.error(
                        channel,
                        Some("interruption"),
                        "Resolve the channel interruption policy.",
                    ))
                }
            },
            ..Default::default()
        };
        let area = self.only("skill-channel-area")?;
        self.link(area, "channel", channel, "channel")?;
        self.link(area, "in", channel, "out")?;
        let placement = self.input(area, "area", "skill-range-placement")?;
        if self.text(placement, "origin") != "caster"
            || self.text(placement, "positionTime") != "accepted"
            || self.text(placement, "facingTime") != "accepted"
        {
            return Err(self.error(
                placement,
                None,
                "Channel area captures caster position and facing at admission.",
            ));
        }
        let range_id = self.input(placement, "range", "skill-range")?;
        let range = self.tile_pattern(placement, "range")?;
        let ended = self.only("skill-channel-ended")?;
        self.link(ended, "channel", channel, "channel")?;
        let request_cancel = self.only("skill-channel-cancel-request")?;
        self.link(request_cancel, "channel", channel, "channel")?;
        let cancel = self.only("skill-channel-cancel")?;
        self.link(cancel, "channel", channel, "channel")?;
        self.link(cancel, "in", request_cancel, "out")?;
        let (ground, event_rules) =
            self.ground_status_plan(area, range.clone(), range.clone(), duration_ms)?;
        let cues: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(id, n)| n.kind == "skill-cue" && !self.used.contains(id.as_str()))
            .map(|(id, _)| id.as_str())
            .collect();
        for cue in cues {
            self.used.insert(cue);
            if !self.doc.nodes[cue].inputs.get("event").is_some_and(|v| {
                v.len() == 1
                    && [channel, ended].contains(&v[0].source.node.as_str())
                    && v[0].source.port == "cue"
            }) {
                return Err(self.error(
                    cue,
                    Some("event"),
                    "Channel presentation observes start or end.",
                ));
            }
        }
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-modifier-range"
                        | "skill-modifier-duration"
                        | "skill-modifier-interruption"
                        | "skill-modifier-ramp"
                        | "skill-modifier-control-vulnerability"
                )
            })
            .map(|(id, _)| id.as_str())
            .collect();
        for id in modifiers {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-range" => {
                    self.link(id, "target", range_id, "range")?;
                    if self.text(id, "mode") != "replace" {
                        return Err(self.error(id, None, "Range modifier requires replacement."));
                    }
                    ActionChange::Range {
                        steps: vec!["channel".into()],
                        pattern: self.tile_pattern(id, "replacement")?,
                    }
                }
                "skill-modifier-duration" => {
                    self.link(id, "target", duration, "value")?;
                    ActionChange::Area {
                        step: "channel".into(),
                        change: AreaChange::Duration {
                            percent: self.number(id, "percent")?,
                        },
                    }
                }
                "skill-modifier-interruption" => {
                    self.link(id, "channel", channel, "channel")?;
                    ActionChange::ProtectInterruption
                }
                "skill-modifier-ramp" => {
                    let ramp = self.input(id, "target", "skill-ground-ramping-slow")?;
                    self.link(id, "target", ramp, "member")?;
                    let status = ground
                        .status
                        .statuses
                        .iter()
                        .position(|s| s.definition == ramp)
                        .ok_or_else(|| {
                            self.error(
                                id,
                                None,
                                "Ramp modifier must target this channel's progress.",
                            )
                        })?;
                    let (numerator, denominator) = self.exact_ratio(id, "factor")?;
                    ActionChange::Area {
                        step: "channel".into(),
                        change: AreaChange::RampRate {
                            status,
                            reached_rules: event_rules.get(ramp).cloned().unwrap_or_default(),
                            numerator,
                            denominator,
                        },
                    }
                }
                _ => {
                    let definition = self.input(id, "ground", "skill-ground-status")?;
                    self.link(id, "ground", definition, "status")?;
                    let status = ground
                        .status
                        .statuses
                        .iter()
                        .position(|s| {
                            s.definition == definition && s.kind == GroundStatusKind::Stun
                        })
                        .ok_or_else(|| {
                            self.error(
                                id,
                                None,
                                "Vulnerability requires this channel's actual stun.",
                            )
                        })?;
                    ActionChange::Area {
                        step: "channel".into(),
                        change: AreaChange::ControlVulnerability {
                            status,
                            percent: self.number(id, "percent")?,
                            duration_ms: self.positive_duration(id, "duration")?,
                        },
                    }
                }
            };
            policy.modifiers.push(ActionModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                variant: RelicVariant::Normal,
                change,
            });
        }
        let program = ChannelGroundProgram {
            range,
            duration_ms,
            pause_basic,
            hit_targets: ground.hit_targets,
            include_caster: ground.include_caster,
            status: ground.status,
            policy,
        };
        program
            .validate()
            .map_err(|e| self.error(channel, None, &e))?;
        Ok(GraphOperation::ResonanceChannelGround { admission, program })
    }
}
