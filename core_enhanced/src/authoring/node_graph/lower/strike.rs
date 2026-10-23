use super::*;
use crate::game::{
    ability::action::*,
    battle::{damage::DamageType, tile_range::TileRangePattern},
    relic::RelicVariant,
};
#[derive(Default)]
struct Sites<'a> {
    ranges: BTreeMap<&'a str, Vec<String>>,
    plans: BTreeMap<&'a str, Vec<String>>,
    statuses: BTreeMap<&'a str, Vec<(String, String)>>,
    wait_durations: BTreeSet<&'a str>,
    durations: BTreeMap<&'a str, Vec<(String, String)>>,
}
impl<'a> Lower<'a> {
    fn strike_range(&mut self, id: &'a str) -> Result<(TileRangePattern, &'a str), Diagnostic> {
        let range = self.input_any(id, "range")?;
        let pattern = if self.doc.nodes[range].kind == "skill-limited-range" {
            let pattern = self.tile_pattern(range, "base")?;
            let limit = self.number(range, "tiles")?;
            let rows = &pattern.rows;
            let mut anchor = None;
            for (y, row) in rows.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c == '@' {
                        anchor = Some((x, y));
                    }
                }
            }
            let Some((ax, ay)) = anchor else {
                return Err(self.error(range, None, "Limited range requires an anchor."));
            };
            for (y, row) in rows.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if c == 'X' && x.abs_diff(ax).max(y.abs_diff(ay)) > limit as usize {
                        return Err(self.error(
                            range,
                            None,
                            "Base pattern exceeds its authored Chebyshev limit.",
                        ));
                    }
                }
            }
            pattern
        } else if self.doc.nodes[range].kind == "skill-range" {
            self.tile_pattern(id, "range")?
        } else {
            return Err(self.error(range, None, "Strike requires an explicit tile pattern."));
        };
        Ok((pattern, range))
    }
    fn strike_parts(
        &mut self,
        id: &'a str,
        parts: &mut Vec<StrikePart>,
        sites: &mut Sites<'a>,
        depth: usize,
    ) -> Result<Vec<String>, Diagnostic> {
        if depth > 64 || parts.len() >= 128 {
            return Err(self.error(id, None, "Finite action plan exceeds its bound."));
        }
        self.used.insert(id);
        let steps = match self.doc.nodes[id].kind.as_str() {
            "skill-sequence-plan" => {
                let a = self.input_any(id, "first")?;
                let b = self.input_any(id, "next")?;
                let mut steps = self.strike_parts(a, parts, sites, depth + 1)?;
                steps.extend(self.strike_parts(b, parts, sites, depth + 1)?);
                steps
            }
            "skill-wait-plan" => {
                sites
                    .wait_durations
                    .insert(self.input(id, "duration", "skill-seconds")?);
                parts.push(StrikePart::Wait {
                    duration_ms: self.duration_allow_zero(id, "duration")?,
                });
                Vec::new()
            }
            "skill-strike-plan" => {
                if self.text(id, "target") != "forward-area" {
                    return Err(self.error(
                        id,
                        None,
                        "This strike lowering requires current forward area recipients.",
                    ));
                }
                let (range, range_id) = self.strike_range(id)?;
                let step = format!("strike-{}", parts.len());
                sites.ranges.entry(range_id).or_default().push(step.clone());
                let power = self.input_any(id, "power")?;
                let (power, missing_hp) = if self.doc.nodes[power].kind == "skill-missing-hp-power"
                {
                    let percent = self.number(power, "percent")?;
                    let cap = self.number(power, "cap")?;
                    (
                        self.input(power, "base", "skill-weapon-power")?,
                        Some((percent, cap)),
                    )
                } else {
                    (power, None)
                };
                if self.doc.nodes[power].kind != "skill-weapon-power" {
                    return Err(self.error(power, None, "Strike requires weapon power."));
                }
                let (numerator, denominator) = self.coefficient(power)?;
                let damage_type = match self.text(id, "damageType") {
                    "physical" => DamageType::Physical,
                    "magic" => DamageType::Magic,
                    "true" => DamageType::True,
                    _ => return Err(self.error(id, None, "Unresolved damage type.")),
                };
                let mut after = Vec::new();
                for input in self.doc.nodes[id].inputs.get("after").into_iter().flatten() {
                    let status = input.source.node.as_str();
                    self.used.insert(status);
                    let effect = format!("after-{}", after.len());
                    match self.doc.nodes[status].kind.as_str() {
                        "skill-hit-pull" => after.push(StrikeStatus::Pull {
                            distance: self.number(status, "tiles")?,
                        }),
                        "skill-hit-status" if self.text(status, "effect") == "stun" => {
                            let time = self.input(status, "duration", "skill-seconds")?;
                            let duration_ms = self.positive_duration(status, "duration")?;
                            sites
                                .statuses
                                .entry(status)
                                .or_default()
                                .push((step.clone(), effect.clone()));
                            sites
                                .durations
                                .entry(time)
                                .or_default()
                                .push((step.clone(), effect));
                            after.push(StrikeStatus::Stun {
                                definition: format!("{}:{status}", self.doc.document),
                                duration_ms,
                            });
                        }
                        _ => return Err(self.error(status, None, "Unsupported strike status.")),
                    }
                }
                parts.push(StrikePart::Strike {
                    id: step.clone(),
                    range,
                    damage_type,
                    numerator,
                    denominator,
                    missing_hp,
                    after,
                });
                vec![step]
            }
            _ => {
                return Err(self.error(
                    id,
                    None,
                    "This finite action lowering supports strikes, waits and sequences.",
                ))
            }
        };
        sites.plans.entry(id).or_default().extend(steps.clone());
        Ok(steps)
    }
    pub(super) fn strike_action(
        &mut self,
        request: &str,
        entry: &'a str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let selector = self.input(entry, "selector", "skill-enemy-selector")?;
        if self.text(selector, "order") != "route" {
            return Err(self.error(selector, None, "Strike admission requires route ordering."));
        }
        let include_air = match self.text(selector, "air") {
            "include" => true,
            "exclude" => false,
            _ => return Err(self.error(selector, None, "Unresolved air policy.")),
        };
        let acquisition = self.tile_pattern(selector, "range")?;
        let mut parts = Vec::new();
        let mut sites = Sites::default();
        self.strike_parts(plan, &mut parts, &mut sites, 0)?;
        let mut policy = ActionPolicy::default();
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.starts_with("skill-modifier-"))
            .map(|(id, _)| id.as_str())
            .collect();
        for id in modifiers {
            self.used.insert(id);
            let condition = self.input(id, "condition", "skill-equipment-condition")?;
            if self.text(condition, "slot") != "relic"
                || self.text(condition, "variant") != "normal"
                || self.text(condition, "copies") != "once"
                || self.doc.nodes[condition].inputs.contains_key("requires")
            {
                return Err(self.error(
                    condition,
                    None,
                    "Action modifier requires one normal equipped relic without prerequisites.",
                ));
            }
            let changes = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-interruption" => {
                    self.link(id, "program", entry, "program")?;
                    vec![ActionChange::ProtectInterruption]
                }
                "skill-modifier-range" => {
                    if self.text(id, "mode") != "replace" {
                        return Err(self.error(id, None, "Range requires explicit replacement."));
                    }
                    let target = self.input_any(id, "target")?;
                    let steps = sites
                        .ranges
                        .get(target)
                        .ok_or_else(|| {
                            self.error(id, None, "Range modifier is not bound to a strike range.")
                        })?
                        .clone();
                    vec![ActionChange::Range {
                        steps,
                        pattern: self.tile_pattern(id, "replacement")?,
                    }]
                }
                "skill-modifier-multihit" => {
                    let target = self.input_any(id, "target")?;
                    let steps = sites
                        .plans
                        .get(target)
                        .ok_or_else(|| {
                            self.error(
                                id,
                                None,
                                "Multihit modifier is not bound to an executed plan.",
                            )
                        })?
                        .clone();
                    vec![ActionChange::Multihit {
                        steps,
                        increment_percent: self.number(id, "step")?,
                        cap_percent: self.number(id, "cap")?,
                    }]
                }
                "skill-modifier-duration" => {
                    let target = self.input_any(id, "target")?;
                    if sites.wait_durations.contains(target) {
                        return Err(self.error(id,None,"This native duration modifier cannot change a shared wait duration; use a separate status duration value."));
                    }
                    let bindings = sites.durations.get(target).ok_or_else(|| {
                        self.error(
                            id,
                            None,
                            "Duration modifier is not bound to a hit status duration.",
                        )
                    })?;
                    let percent = self.number(id, "percent")?;
                    bindings
                        .iter()
                        .map(|(step, effect)| ActionChange::Duration {
                            step: step.clone(),
                            effect: effect.clone(),
                            percent,
                        })
                        .collect()
                }
                "skill-modifier-control-vulnerability" => {
                    let target = self.input_any(id, "hit")?;
                    let bindings = sites.statuses.get(target).ok_or_else(|| {
                        self.error(id, None, "Vulnerability requires an executed stun.")
                    })?;
                    let percent = self.number(id, "percent")?;
                    let duration_ms = self.positive_duration(id, "duration")?;
                    bindings
                        .iter()
                        .map(|(step, effect)| ActionChange::ControlVulnerability {
                            step: step.clone(),
                            effect: effect.clone(),
                            percent,
                            duration_ms,
                        })
                        .collect()
                }
                _ => return Err(self.error(id, None, "Unsupported finite action modifier.")),
            };
            for (index, change) in changes.into_iter().enumerate() {
                policy.modifiers.push(ActionModifier {
                    id: format!("{id}-{index}"),
                    relic_definition: self.text(condition, "definition").into(),
                    variant: RelicVariant::Normal,
                    change,
                });
            }
        }
        let program = StrikeProgram {
            acquisition,
            include_air,
            parts,
            policy,
        };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceStrike { admission, program })
    }
}
