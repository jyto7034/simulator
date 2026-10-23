use super::*;
use crate::game::ability::marks::*;
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
impl<'a> Lower<'a> {
    pub(super) fn signed_period_ratio(&self, id: &str) -> Result<(bool, MarkRatio), Diagnostic> {
        let Some(FieldValue::Number(value)) = self.doc.nodes[id].values.get("percent") else {
            return Err(self.error(id, Some("percent"), "Resolve the signed period change."));
        };
        let (numerator, denominator) = if *value == 0.0 { (0, 1) }
            else { self.exact_numeric_ratio(value.abs(), id, "percent")? };
        Ok((*value < 0.0, MarkRatio { numerator, denominator }))
    }
    pub(super) fn mark_ratio(&self, id: &str, field: &str, zero: bool) -> Result<MarkRatio, Diagnostic> {
        if zero && self.doc.nodes[id].values.get(field) == Some(&FieldValue::Number(0.0)) {
            return Ok(MarkRatio {
                numerator: 0,
                denominator: 1,
            });
        }
        let (numerator, denominator) = self.exact_ratio(id, field)?;
        Ok(MarkRatio {
            numerator,
            denominator,
        })
    }
    pub(super) fn mark_damage_type(&self, id: &str) -> Result<DamageType, Diagnostic> {
        match self.text(id, "damageType") {
            "physical" => Ok(DamageType::Physical),
            "magic" => Ok(DamageType::Magic),
            "true" => Ok(DamageType::True),
            _ => Err(self.error(id, Some("damageType"), "Resolve the mark damage type.")),
        }
    }
    pub(super) fn presence_burst_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let marks = self.input(plan, "marks", "skill-target-marks")?;
        if self.text(marks, "mode") != "presence"
            || self.text(marks, "lifetime") != "deployment"
            || self.text(marks, "death") != "remove"
        {
            return Err(self.error(
                marks,
                None,
                "Presence burst requires deployment-owned hosts removed on death.",
            ));
        }
        let power = self.input(plan, "power", "skill-population-power")?;
        self.link(power, "marks", marks, "marks")?;
        let hit = self.only("skill-mark-weapon-hit")?;
        self.link(hit, "marks", marks, "marks")?;
        if self.text(hit, "binding") != "fragment"
            || !self.text(hit, "weapon").is_empty()
            || self.number(hit, "amount")? != 1
            || self.number(hit, "grantMultiplier")? != 1
        {
            return Err(self.error(hit,None,"Presence host subscription belongs to the equipped fragment with one mark per host."));
        }
        let filter = self.input(hit, "filter", "skill-mark-filter")?;
        let flag = |field: &str| {
            self.doc.nodes[filter].values.get(field) == Some(&FieldValue::Boolean(true))
        };
        let filter_def = MarkFilter {
            air: flag("air"),
            boss: flag("boss"),
            abnormality: flag("abnormality"),
            summon: flag("summon"),
            temporary: flag("temporary"),
        };
        let spread = self.only("skill-mark-spread")?;
        self.link(spread, "marks", marks, "marks")?;
        self.link(spread, "source", hit, "source")?;
        self.link(spread, "filter", filter, "filter")?;
        let radius = self.mark_ratio(spread, "radius", true)?;
        let units = u128::from(radius.numerator) * 1_000_000;
        if units % u128::from(radius.denominator) != 0 {
            return Err(self.error(
                spread,
                Some("radius"),
                "Spread radius requires exact native spatial units.",
            ));
        }
        let spread_radius_units =
            u32::try_from(units / u128::from(radius.denominator)).map_err(|_| {
                self.error(
                    spread,
                    Some("radius"),
                    "Spread radius exceeds native spatial units.",
                )
            })?;
        let periodic = self.only("skill-mark-periodic-damage")?;
        self.link(periodic, "marks", marks, "marks")?;
        self.link(periodic, "power", power, "power")?;
        let interval = self.input(periodic, "interval", "skill-seconds")?;
        let explosion = self.input(plan, "explosion", "skill-explosion-rule")?;
        let range_id = self.input(plan, "range", "skill-range")?;
        let range = self.tile_pattern(plan, "range")?;
        let mut program = PresenceBurstProgram {
            marks: PresenceMarksDef {
                definition: format!("{}:{marks}", self.doc.document),
                filter: filter_def,
                chance_percent: self.mark_ratio(hit, "chance", true)?,
                spread_radius_units,
                spread_limit: self.number(spread, "limit")?,
                power: PopulationPowerDef {
                    growth_per_second: self.mark_ratio(power, "growth", true)?,
                    cap: self.mark_ratio(power, "cap", false)?,
                    decay_per_second: self.mark_ratio(power, "decay", true)?,
                    grace_ms: self.duration_allow_zero(power, "grace")?,
                    prevent_decay: self.doc.nodes[power].values.get("preventDecay")
                        == Some(&FieldValue::Boolean(true)),
                },
                periodic_interval_ms: self.positive_duration(periodic, "interval")?,
                periodic_coefficient: self.mark_ratio(periodic, "coefficient", false)?,
                periodic_damage_type: self.mark_damage_type(periodic)?,
            },
            explosion: MarkExplosionDef {
                range: range.clone(),
                coefficient: self.mark_ratio(plan, "coefficient", false)?,
                host_multiplier: self.mark_ratio(plan, "hostMultiplier", false)?,
                damage_type: self.mark_damage_type(explosion)?,
                bonus_percent: self.mark_ratio(explosion, "bonus", true)?,
                dedup: match self.text(explosion, "dedup") {
                    "wave" => ExplosionDedup::Wave,
                    "blast" => ExplosionDedup::Explosion,
                    _ => {
                        return Err(self.error(
                            explosion,
                            None,
                            "Resolve explosion deduplication scope.",
                        ))
                    }
                },
            },
            modifiers: vec![],
        };
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-modifier-duration"
                        | "skill-modifier-population"
                        | "skill-modifier-range"
                        | "skill-modifier-explosion-power"
                        | "skill-modifier-explosion-overlap"
                )
            })
            .map(|(id, _)| id.as_str())
            .collect();
        for id in modifiers {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-duration" => {
                    self.link(id, "target", interval, "value")?;
                    let (decrease, percent) = self.signed_period_ratio(id)?;
                    MarkChange::Period {
                        decrease, percent,
                    }
                }
                "skill-modifier-population" => {
                    self.link(id, "target", power, "power")?;
                    match self.text(id, "mode") {
                        "growth" => MarkChange::Growth {
                            percent: self.mark_ratio(id, "percent", true)?,
                        },
                        "preserve" => MarkChange::PreservePower,
                        _ => return Err(self.error(id, None, "Resolve population modifier.")),
                    }
                }
                "skill-modifier-range" => {
                    self.link(id, "target", range_id, "range")?;
                    let pattern = match self.text(id, "mode") {
                        "replace" => self.tile_pattern(id, "replacement")?,
                        "square-step" if range.include_anchor_tile && range.rows == ["@"] => {
                            TileRangePattern {
                                include_anchor_tile: true,
                                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                            }
                        }
                        _ => {
                            return Err(self.error(
                                id,
                                None,
                                "Author a replacement for this explosion shape.",
                            ))
                        }
                    };
                    MarkChange::Range { pattern }
                }
                "skill-modifier-explosion-power" => {
                    self.link(id, "target", explosion, "explosion")?;
                    MarkChange::ExplosionPower {
                        percent: self.mark_ratio(id, "percent", true)?,
                    }
                }
                _ => {
                    self.link(id, "target", explosion, "explosion")?;
                    MarkChange::ExplosionOverlap
                }
            };
            program.modifiers.push(MarkModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                change,
            });
        }
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonancePresenceBurst { admission, program })
    }
}
