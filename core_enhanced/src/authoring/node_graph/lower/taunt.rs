use super::*;
use crate::game::{
    ability::{action::*, sustained::ApproachTauntProfile},
    relic::RelicVariant,
    resource::{ResourceId, WeaponGrowthMultiplier, WeaponKillGrowthDef},
};

impl<'a> Lower<'a> {
    pub(super) fn taunt_condition(&mut self, modifier: &str) -> Result<&'a str, Diagnostic> {
        let condition = self.input(modifier, "condition", "skill-equipment-condition")?;
        if self.text(condition, "slot") != "relic"
            || self.text(condition, "variant") != "normal"
            || self.text(condition, "copies") != "once"
            || self.doc.nodes[condition].inputs.contains_key("requires")
        {
            return Err(self.error(
                condition,
                None,
                "This binding requires a normal equipped relic, once, without prerequisites.",
            ));
        }
        Ok(condition)
    }
    fn weapon_growth(&mut self) -> Result<Option<WeaponGrowthBinding>, Diagnostic> {
        if !self
            .doc
            .nodes
            .values()
            .any(|n| n.kind == "skill-weapon-kill-growth")
        {
            return Ok(None);
        }
        let id = self.only("skill-weapon-kill-growth")?;
        let mut amounts = Vec::new();
        let mut multiplier: Option<Option<WeaponGrowthMultiplier>> = None;
        for port in ["normal", "elite", "abnormality"] {
            let value = self.input(id, port, "skill-integer-amount")?;
            amounts.push(self.number(value, "value")?);
            let modifiers: Vec<_> = self
                .doc
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.kind == "skill-modifier-number"
                        && n.inputs
                            .get("target")
                            .is_some_and(|v| v.iter().any(|o| o.source.node == value))
                })
                .map(|(id, _)| id.as_str())
                .collect();
            if modifiers.len() > 1 {
                return Err(self.error(
                    value,
                    None,
                    "One growth acquisition multiplier is supported per quantity.",
                ));
            }
            let next = if let Some(modifier) = modifiers.first().copied() {
                self.used.insert(modifier);
                self.link(modifier, "target", value, "value")?;
                let condition = self.taunt_condition(modifier)?;
                Some(WeaponGrowthMultiplier {
                    relic_definition: self.text(condition, "definition").into(),
                    factor: self.number(modifier, "factor")?,
                })
            } else {
                None
            };
            if multiplier.as_ref().is_some_and(|old| old != &next) {
                return Err(self.error(
                    id,
                    None,
                    "Growth tiers must share the same conditional acquisition multiplier.",
                ));
            }
            multiplier = Some(next);
        }
        let weapon = self.text(id, "weapon").to_owned();
        let key = self.text(id, "key");
        if key.is_empty() || key.trim() != key {
            return Err(self.error(
                id,
                Some("key"),
                "Growth requires a resolved state identity.",
            ));
        }
        let resource = ResourceId::new(format!("weapon:{}:{weapon}:{key}", weapon.len()))
            .map_err(|e| self.error(id, Some("key"), &e.to_string()))?;
        let growth = WeaponKillGrowthDef {
            resource,
            normal: amounts[0],
            elite: amounts[1],
            abnormality: amounts[2],
            damage_per_stack: self.number(id, "damage")?,
            secondary_percent: self.number(id, "secondaryPercent")?,
            multiplier: multiplier.flatten(),
        };
        growth.validate().map_err(|e| self.error(id, None, &e))?;
        Ok(Some(WeaponGrowthBinding { weapon, growth }))
    }
    pub(super) fn taunt_action(
        &mut self,
        request: &str,
        admit: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let query = self.only("skill-range-targets")?;
        let result = self.input(query, "in", "skill-resonance-result")?;
        self.link(query, "in", result, "out")?;
        self.link(result, "in", admit, "out")?;
        self.link(result, "receipt", admit, "receipt")?;
        self.link(query, "use", admit, "use")?;
        let end = self.only("skill-resonance-end")?;
        self.link(end, "in", query, "out")?;
        self.link(end, "use", admit, "use")?;
        let placement = self.input(query, "area", "skill-range-placement")?;
        if self.text(placement, "origin") != "caster"
            || self.text(placement, "positionTime") != "release"
            || self.text(placement, "facingTime") != "release"
        {
            return Err(self.error(
                placement,
                None,
                "Approach taunt queries the current caster position and facing at release.",
            ));
        }
        let range_id = self.input(placement, "range", "skill-range")?;
        let range = self.tile_pattern(placement, "range")?;
        let filter = self.input(query, "filter", "skill-ground-filter")?;
        if self.text(filter, "side") != "enemies" || self.text(filter, "caster") != "exclude" {
            return Err(self.error(
                filter,
                None,
                "Approach taunt requires enemy recipients excluding caster.",
            ));
        }
        let apply = self.only("skill-target-apply")?;
        self.link(apply, "in", query, "each")?;
        self.link(apply, "target", query, "target")?;
        let state = self.input(apply, "state", "skill-target-state")?;
        if self.text(state, "sourceExit") != "end" {
            return Err(self.error(state, None, "Approach taunt ends when its source exits."));
        }
        let duration = self.input(state, "duration", "skill-seconds")?;
        let duration_ms = self.positive_duration(state, "duration")?;
        let mut speed = None;
        let mut limit = None;
        let mut taunt = false;
        for origin in self.doc.nodes[state]
            .inputs
            .get("effects")
            .into_iter()
            .flatten()
        {
            let id = origin.source.node.as_str();
            self.used.insert(id);
            if origin.source.port != "effect" {
                return Err(self.error(state, Some("effects"), "Invalid target effect output."));
            }
            match self.doc.nodes[id].kind.as_str() {
                "skill-target-taunt" if !taunt => taunt=true,
                "skill-target-speed" if speed.is_none() => speed=Some(self.number(id,"percent")?),
                "skill-target-attack-limit" if limit.is_none() => {
                    let (n,d)=self.exact_ratio(id,"tiles")?;
                    let scaled=u128::from(n)*1_000_000;
                    if scaled%u128::from(d)!=0 { return Err(self.error(id,None,"Attack distance must be exact in native world units.")); }
                    limit=Some(u32::try_from(scaled/u128::from(d)).map_err(|_|self.error(id,None,"Attack distance exceeds native range."))?);
                }
                _=>return Err(self.error(id,None,"Approach taunt requires one taunt, speed increase and attack limit without duplicate effects.")),
            }
        }
        let (true, Some(speed_percent), Some(attack_range_units)) = (taunt, speed, limit) else {
            return Err(self.error(
                state,
                None,
                "Approach taunt requires the complete shared-lifetime effect bundle.",
            ));
        };
        let ended = self.only("skill-target-ended")?;
        self.link(ended, "instance", apply, "instance")?;
        let cues: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-cue")
            .map(|(id, _)| id.as_str())
            .collect();
        for cue in cues {
            self.used.insert(cue);
            let origins = self.doc.nodes[cue].inputs.get("event");
            if !origins.is_some_and(|v| {
                v.len() == 1
                    && [apply, ended].contains(&v[0].source.node.as_str())
                    && v[0].source.port == "cue"
            }) {
                return Err(self.error(
                    cue,
                    Some("event"),
                    "Taunt presentation must observe its apply or end event.",
                ));
            }
        }
        let mut policy = ActionPolicy::default();
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-modifier-range"
                        | "skill-modifier-duration"
                        | "skill-modifier-taunt-damage"
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
                        return Err(self.error(
                            id,
                            None,
                            "Range modifier requires an explicit replacement.",
                        ));
                    }
                    ActionChange::Range {
                        steps: vec!["taunt".into()],
                        pattern: self.tile_pattern(id, "replacement")?,
                    }
                }
                "skill-modifier-duration" => {
                    self.link(id, "target", duration, "value")?;
                    ActionChange::Duration {
                        step: "taunt".into(),
                        effect: "approach".into(),
                        percent: self.number(id, "percent")?,
                    }
                }
                _ => {
                    self.link(id, "target", state, "state")?;
                    let percent = self.number(id, "percent")?;
                    let (taken_percent, basic_percent) = match self.text(id, "mode") {
                        "taken" => (percent, 0),
                        "dealt-basic" => (0, percent),
                        _ => {
                            return Err(self.error(
                                id,
                                None,
                                "Choose the taunt damage relationship.",
                            ))
                        }
                    };
                    ActionChange::TauntDamage {
                        step: "taunt".into(),
                        effect: "approach".into(),
                        taken_percent,
                        basic_percent,
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
        let program = TauntProgram {
            range,
            duration_ms,
            profile: ApproachTauntProfile {
                speed_percent,
                attack_range_units,
                damage_reduction_percent: 0,
                basic_damage_bonus_percent: 0,
            },
            policy,
            weapon_growth: self.weapon_growth()?,
        };
        program
            .validate()
            .map_err(|e| self.error(state, None, &e))?;
        Ok(GraphOperation::ResonanceTaunt { admission, program })
    }
}
