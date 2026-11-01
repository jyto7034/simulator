use super::*;
use crate::game::ability::{marks::*, tile_burst::*, weapon_marks::*};
use crate::game::battle::tile_range::TileRangePattern;

impl<'a> Lower<'a> {
    fn mark_weapon(&mut self) -> Result<(WeaponMarkBinding, &'a str), Diagnostic> {
        let owner = self.only("skill-weapon-mark-owner")?;
        let marks = self.input(owner, "marks", "skill-target-marks")?;
        if self.text(marks, "mode") != "stacks"
            || self.text(marks, "lifetime") != "battle"
            || self.text(marks, "death") != "tile"
        {
            return Err(self.error(
                marks,
                None,
                "Weapon marks require battle-owned stacks transferred to the death center tile.",
            ));
        }
        let evolution = self.input(owner, "evolution", "skill-weapon-evolution")?;
        let weapon = self.text(evolution, "weapon").to_string();
        let ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-mark-weapon-hit")
            .map(|(id, _)| id.as_str())
            .collect();
        if ids.len() != 2 {
            return Err(self.error(
                owner,
                None,
                "Declare exactly one base and one upgraded weapon hit subscription.",
            ));
        }
        let mut base = None;
        let mut upgraded = None;
        let mut filter = None;
        let mut multiplier_by_hit = Vec::new();
        for id in ids {
            self.used.insert(id);
            self.link(id, "marks", marks, "marks")?;
            self.link(id, "evolution", evolution, "evolution")?;
            if self.text(id, "weapon") != weapon {
                return Err(self.error(
                    id,
                    None,
                    "Weapon hit identity differs from its evolution.",
                ));
            }
            let f = self.input(id, "filter", "skill-mark-filter")?;
            if filter.replace(f).is_some_and(|old| old != f) {
                return Err(self.error(
                    id,
                    None,
                    "Both stages must share the explicit target filter.",
                ));
            }
            let hit = WeaponMarkHit {
                chance_percent: self.mark_ratio(id, "chance", true)?,
                amount: self.number(id, "amount")?,
                multiplier: self.mark_ratio(id, "grantMultiplier", false)?,
            };
            let slot = match self.text(id, "binding") {
                "weapon-base" => &mut base,
                "weapon-upgrade" => &mut upgraded,
                _ => {
                    return Err(self.error(
                        id,
                        None,
                        "Stack hit must select an explicit weapon stage.",
                    ))
                }
            };
            if slot.replace(hit).is_some() {
                return Err(self.error(id, None, "Duplicate weapon stage subscription."));
            }
            let mods: Vec<_> = self
                .doc
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.kind == "skill-modifier-mark-grant"
                        && n.inputs
                            .get("target")
                            .is_some_and(|edges| edges.iter().any(|e| e.source.node == id))
                })
                .map(|(id, _)| id.as_str())
                .collect();
            let mut rules = std::collections::BTreeMap::new();
            for m in mods {
                self.used.insert(m);
                self.link(m, "target", id, "source")?;
                let c = self.taunt_condition(m)?;
                if rules
                    .insert(
                        self.text(c, "definition").to_string(),
                        self.mark_ratio(m, "factor", false)?,
                    )
                    .is_some()
                {
                    return Err(self.error(
                        m,
                        None,
                        "Duplicate new-grant multiplier for the same relic.",
                    ));
                }
            }
            multiplier_by_hit.push(rules);
        }
        if multiplier_by_hit[0] != multiplier_by_hit[1] {
            return Err(self.error(
                owner,
                None,
                "Both weapon stages must declare the same new-grant relic multipliers.",
            ));
        }
        let filter = filter.unwrap();
        let flag = |field: &str| {
            self.doc.nodes[filter].values.get(field) == Some(&FieldValue::Boolean(true))
        };
        let definition = MarkWeaponDef {
            evolution: WeaponEvolutionDef {
                fragment: self.text(evolution, "fragment").into(),
                form: self.text(evolution, "form").into(),
                base: self.text(evolution, "base").into(),
                upgraded: self.text(evolution, "upgraded").into(),
            },
            key: self.text(owner, "key").into(),
            filter: MarkFilter {
                air: flag("air"),
                boss: flag("boss"),
                abnormality: flag("abnormality"),
                summon: flag("summon"),
                temporary: flag("temporary"),
            },
            base: base.ok_or_else(|| self.error(owner, None, "Missing base subscription."))?,
            upgraded: upgraded
                .ok_or_else(|| self.error(owner, None, "Missing upgraded subscription."))?,
            modifiers: multiplier_by_hit
                .remove(0)
                .into_iter()
                .map(|(relic_definition, multiplier)| WeaponMarkMultiplier {
                    relic_definition,
                    multiplier,
                })
                .collect(),
        };
        definition
            .validate()
            .map_err(|e| self.error(owner, None, &e))?;
        Ok((WeaponMarkBinding { weapon, definition }, marks))
    }
    pub(super) fn tile_burst_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let (weapon, marks) = self.mark_weapon()?;
        let stacked = self.doc.nodes[plan].kind == "skill-stack-burst-plan";
        let grade = if stacked {
            Some(self.input(plan, "grade", "skill-stack-grade")?)
        } else {
            None
        };
        let power = self.input(grade.unwrap_or(plan), "power", "skill-damage-amount")?;
        let source = if let Some(grade) = grade {
            self.link(plan, "marks", marks, "marks")?;
            let conversion = if self.doc.nodes[plan].inputs.contains_key("conversion") {
                let c = self.input(plan, "conversion", "skill-burst-conversion")?;
                let condition = self.taunt_condition(c)?;
                let orb_power = self.input(c, "power", "skill-damage-amount")?;
                Some(OrbConversion {
                    relic_definition: self.text(condition, "definition").into(),
                    threshold: u64::from(self.number(c, "threshold")?),
                    base_power: self.number(orb_power, "value")?,
                    power_per_stack: self.mark_ratio(c, "perStack", true)?,
                    range: self.tile_pattern(c, "range")?,
                    presentation_ms: self.positive_duration(c, "presentation")?,
                })
            } else {
                None
            };
            TileBurstSource::WeaponStacks {
                weapon: weapon.weapon.clone(),
                key: weapon.definition.key.clone(),
                divisor: self.number(grade, "divisor")?,
                rank_bonus_percent: self.mark_ratio(grade, "rankBonus", true)?,
                interval_ms: self.positive_duration(plan, "interval")?,
                conversion,
            }
        } else {
            TileBurstSource::WalkablePath
        };
        let explosion = self.input(plan, "explosion", "skill-explosion-rule")?;
        let range_id = self.input(plan, "range", "skill-range")?;
        let range = self.tile_pattern(plan, "range")?;
        let mut effect = TileBurstDef {
            source,
            power: self.number(power, "value")?,
            delay_ms: self.duration_allow_zero(plan, "delay")?,
            range: range.clone(),
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
            modifiers: vec![],
        };
        let modifiers: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-modifier-range"
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
            effect.modifiers.push(MarkModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                change,
            });
        }
        let program = TileBurstProgram { effect, weapon };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceTileBurst { admission, program })
    }
}
