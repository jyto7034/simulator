use super::*;
use crate::game::{
    ability::{paired_areas::*, sustained::TargetStateProfile},
    battle::tile_range::TileRangePattern,
};
impl<'a> Lower<'a> {
    pub(super) fn paired_areas(
        &mut self,
        request: &str,
        admit: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let selection = self.input(request, "selection", "skill-two-tile-selection")?;
        let selection_range = self.tile_pattern(selection, "range")?;
        let pair = self.only("skill-paired-areas")?;
        self.link(pair, "in", admit, "out")?;
        self.link(pair, "use", admit, "use")?;
        let first_id = self.input(pair, "first", "skill-range")?;
        let second_id = self.input(pair, "second", "skill-range")?;
        let first = self.tile_pattern(pair, "first")?;
        let second = self.tile_pattern(pair, "second")?;
        let duration = self.input(pair, "duration", "skill-seconds")?;
        let duration_ms = self.positive_duration(pair, "duration")?;
        let pulse = self.consumer(pair, "pair", "skill-pair-pulse", "pair")?;
        let interval_ms = self.positive_duration(pulse, "interval")?;
        let heal = self.consumer(pulse, "out", "skill-pair-base-heal", "in")?;
        self.link(heal, "pulse", pulse, "pulse")?;
        let power = self.input(heal, "power", "skill-healing-amount")?;
        let healing = self.number(power, "value")?;
        let transfer = self.consumer(heal, "out", "skill-pair-transfer", "in")?;
        self.link(transfer, "result", heal, "result")?;
        self.link(transfer, "pulse", pulse, "pulse")?;
        let intersection = self.consumer(pair, "pair", "skill-pair-intersection", "pair")?;
        let buff = self.input(intersection, "buff", "skill-attack-speed-buff")?;
        let profile = TargetStateProfile {
        damage_invulnerable: false,
        basic_attack_locked: false,
        active_skill_locked: false,
        received_heal_percent: 0,
            definition: format!("{}:{buff}", self.doc.document),
            attack_percent: 0,
            defense_percent: 0,
            attack_speed_points: self.number(buff, "points")?,
            hp_floor_one: false,
            end_on_source_exit: true,
        };
        let cues: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-cue")
            .map(|(id, _)| id.as_str())
            .collect();
        for id in cues {
            let events = self.doc.nodes[id]
                .inputs
                .get("event")
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    self.error(
                        id,
                        Some("event"),
                        "Cue requires a compiled pair occurrence.",
                    )
                })?;
            if events.iter().any(|e| {
                e.source.port != "cue"
                    || ![pair, heal, transfer, intersection].contains(&e.source.node.as_str())
            }) {
                return Err(self.error(
                    id,
                    Some("event"),
                    "Cue must observe this pair's declared effects.",
                ));
            }
            self.used.insert(id);
        }
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
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-duration" => {
                    self.link(id, "target", duration, "value")?;
                    let (negative, percent) = self.signed_period_ratio(id)?;
                    PairChange::Duration { negative, percent }
                }
                "skill-modifier-healing" => {
                    self.link(id, "target", power, "power")?;
                    PairChange::Healing {
                        percent: self.counter_ratio(id, "percent")?,
                    }
                }
                "skill-modifier-buff" => {
                    self.link(id, "target", buff, "buff")?;
                    PairChange::Buff {
                        percent: self.counter_ratio(id, "percent")?,
                    }
                }
                "skill-modifier-heal-buff" => {
                    self.link(id, "target", power, "power")?;
                    let state = self.input(id, "state", "skill-target-state")?;
                    let (duration_ms, profile) = self.state_profile(state)?;
                    PairChange::HealBuff {
                        duration_ms,
                        profile,
                    }
                }
                "skill-modifier-range" => {
                    let target = self.input(id, "target", "skill-range")?;
                    self.link(id, "target", target, "range")?;
                    let a = target == first_id;
                    let b = target == second_id;
                    if !a && !b {
                        return Err(self.error(
                            id,
                            Some("target"),
                            "Range changes must reference an effect mask, never selection.",
                        ));
                    }
                    let base = if a { &first } else { &second };
                    let pattern = match self.text(id, "mode") {
                        "replace" => self.tile_pattern(id, "replacement")?,
                        "square-step" if base.include_anchor_tile && base.rows == ["@"] => {
                            TileRangePattern {
                                include_anchor_tile: true,
                                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                            }
                        }
                        "square-step"
                            if base.include_anchor_tile && base.rows == ["XXX", "X@X", "XXX"] =>
                        {
                            TileRangePattern {
                                include_anchor_tile: true,
                                rows: vec![
                                    "XXXX".into(),
                                    "X@XX".into(),
                                    "XXXX".into(),
                                    "XXXX".into(),
                                ],
                            }
                        }
                        _ => {
                            return Err(self.error(
                                id,
                                None,
                                "Declare an explicit replacement mask for this shape.",
                            ))
                        }
                    };
                    PairChange::Range {
                        first: a,
                        second: b,
                        pattern,
                    }
                }
                "skill-modifier-coincident-area-burst" => {
                    self.link(id, "target", pair, "pair")?;
                    PairChange::CoincidentBurst {
                        delay_ms: self.duration_allow_zero(id, "delay")?,
                        pattern: self.tile_pattern(id, "range")?,
                        max_hp_percent: self.number(id, "percent")?,
                    }
                }
                _ => return Err(self.error(id, None, "Unsupported paired-area modifier.")),
            };
            modifiers.push(PairModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                change,
            });
        }
        let program = PairedAreasDef {
            definition: format!("{}:{pair}", self.doc.document),
            admission,
            selection_range,
            first,
            second,
            duration_ms,
            interval_ms,
            healing,
            transfer_percent: self.counter_ratio(transfer, "percent")?,
            intersection: profile,
            modifiers,
        };
        program.validate().map_err(|e| self.error(pair, None, &e))?;
        Ok(GraphOperation::ResonancePairedAreas { program })
    }
}
