use super::*;
use crate::game::{
    ability::{marks::MarkRatio, mobile_effect::*},
    battle::{
        damage::DamageType,
        tile_range::{FacingDirection, TileRangePattern},
    },
    resources::Position,
};
impl<'a> Lower<'a> {
    pub(super) fn mobile_pattern(
        &mut self,
        id: &str,
        port: &str,
        depth: usize,
    ) -> Result<TileRangePattern, Diagnostic> {
        if depth > 32 {
            return Err(self.error(id, Some(port), "Range expression is too deep."));
        }
        let origin = self.input_any(id, port)?;
        if self.doc.nodes[origin].kind == "skill-range" {
            return self.tile_pattern(id, port);
        }
        if self.doc.nodes[origin].kind != "skill-scaled-range" {
            return Err(self.error(
                origin,
                None,
                "Expected a tile pattern or an explicit scaled pattern.",
            ));
        }
        let base = self.mobile_pattern(origin, "base", depth + 1)?;
        let factor = self.number(origin, "factor")?;
        if factor == 0 || factor > 32 {
            return Err(self.error(origin, None, "Range factor must be in 1..32."));
        }
        let tiles = base
            .affected_tiles(Position::new(0, 0), FacingDirection::Up)
            .map_err(|e| self.error(origin, None, &e))?;
        let f = i64::from(factor);
        let mut set = BTreeSet::new();
        for tile in tiles {
            // A source cell spans [center-.5, center+.5], including its boundary.
            let lo_x = (2 * i64::from(tile.x) - 1) * f;
            let hi_x = (2 * i64::from(tile.x) + 1) * f;
            let lo_y = (2 * i64::from(tile.y) - 1) * f;
            let hi_y = (2 * i64::from(tile.y) + 1) * f;
            for y in -((-lo_y).div_euclid(2))..=hi_y.div_euclid(2) {
                for x in -((-lo_x).div_euclid(2))..=hi_x.div_euclid(2) {
                    set.insert((x, y));
                    if set.len() > 16384 {
                        return Err(self.error(
                            origin,
                            None,
                            "Scaled range exceeds the bounded mask size.",
                        ));
                    }
                }
            }
        }
        let min_x = set.iter().map(|v| v.0).min().unwrap_or(0).min(0);
        let max_x = set.iter().map(|v| v.0).max().unwrap_or(0).max(0);
        let min_y = set.iter().map(|v| v.1).min().unwrap_or(0).min(0);
        let max_y = set.iter().map(|v| v.1).max().unwrap_or(0).max(0);
        if (max_x - min_x + 1) * (max_y - min_y + 1) > 16384 {
            return Err(self.error(origin, None, "Scaled range bounds exceed the mask size."));
        }
        let pattern = TileRangePattern {
            include_anchor_tile: set.contains(&(0, 0)),
            rows: (min_y..=max_y)
                .map(|y| {
                    (min_x..=max_x)
                        .map(|x| {
                            if x == 0 && y == 0 {
                                '@'
                            } else if set.contains(&(x, y)) {
                                'X'
                            } else {
                                '.'
                            }
                        })
                        .collect()
                })
                .collect(),
        };
        pattern
            .validate()
            .map_err(|e| self.error(origin, None, &e))?;
        Ok(pattern)
    }
    pub(super) fn mobile_power(
        &mut self,
        id: &'a str,
        depth: usize,
    ) -> Result<MobilePower, Diagnostic> {
        if depth > 32 {
            return Err(self.error(id, None, "Mobile power is too deep."));
        }
        self.used.insert(id);
        match self.doc.nodes[id].kind.as_str() {
            "skill-weapon-power" => {
                let (numerator, denominator) = self.coefficient(id)?;
                Ok(MobilePower::Weapon {
                    coefficient: MarkRatio {
                        numerator,
                        denominator,
                    },
                })
            }
            "skill-damage-bonus" => {
                let base = self.input_any(id, "base")?;
                Ok(MobilePower::Add {
                    base: Box::new(self.mobile_power(base, depth + 1)?),
                    amount: self.number(id, "bonus")?,
                })
            }
            "skill-fusion-count-power" => {
                let base = self.input_any(id, "base")?;
                Ok(MobilePower::Fusion {
                    base: Box::new(self.mobile_power(base, depth + 1)?),
                    baseline: self.number(id, "baseline")?,
                    percent: self.counter_ratio(id, "percent")?,
                })
            }
            _ => Err(self.error(id, None, "Unsupported mobile power expression.")),
        }
    }
    pub(super) fn mobile_profile(
        &mut self,
        id: &'a str,
        depth: usize,
    ) -> Result<(MobileProfile, Vec<&'a str>), Diagnostic> {
        if depth > 32 {
            return Err(self.error(id, None, "Mobile profile inheritance is too deep."));
        }
        self.used.insert(id);
        let (mut p, mut ancestry) = if self.doc.nodes[id].kind == "skill-mobile-variant" {
            let base = self.input_any(id, "base")?;
            self.mobile_profile(base, depth + 1)?
        } else if self.doc.nodes[id].kind == "skill-mobile-effect" {
            let (n, d) = self.exact_ratio(id, "speed")?;
            let units = u128::from(n) * 1000;
            if units % u128::from(d) != 0 {
                return Err(self.error(
                    id,
                    Some("speed"),
                    "Speed requires exact millitiles per second.",
                ));
            }
            let speed = u32::try_from(units / u128::from(d))
                .map_err(|_| self.error(id, Some("speed"), "Speed exceeds u32."))?;
            (
                MobileProfile {
                    definition: String::new(),
                    motion: match self.text(id, "motion") {
                        "wait-leap" => MobileMotion::WaitLeap,
                        "chase-delay" => MobileMotion::ChaseDelay,
                        _ => return Err(self.error(id, None, "Unresolved mobile motion.")),
                    },
                    detection: self.tile_pattern(id, "detection")?,
                    range: TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["@".into()],
                    },
                    power: MobilePower::Weapon {
                        coefficient: MarkRatio {
                            numerator: 1,
                            denominator: 1,
                        },
                    },
                    damage_type: match self.text(id, "damageType") {
                        "physical" => DamageType::Physical,
                        "magic" => DamageType::Magic,
                        _ => return Err(self.error(id, None, "Unresolved mobile damage type.")),
                    },
                    speed_units_per_second: speed,
                    delay_ms: self.duration_allow_zero(id, "delay")?,
                    lifetime_ms: if self.doc.nodes[id].inputs.contains_key("lifetime") {
                        Some(self.positive_duration(id, "lifetime")?)
                    } else {
                        None
                    },
                    modifiers: Vec::new(),
                },
                Vec::new(),
            )
        } else {
            return Err(self.error(id, None, "Expected a mobile effect or variant."));
        };
        p.definition = format!("{}:{id}", self.doc.document);
        p.range = self.mobile_pattern(id, "range", 0)?;
        let power = self.input_any(id, "power")?;
        p.power = self.mobile_power(power, 0)?;
        // Concrete modifiers attach after inheritance: power never leaks into another variant.
        p.modifiers.clear();
        ancestry.push(id);
        Ok((p, ancestry))
    }
    pub(super) fn fusion_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let select = self.input(request, "selection", "skill-target-selection")?;
        if self.text(select, "mode") != "player-tile" {
            return Err(self.error(
                select,
                None,
                "Fusion placement requires player tile selection.",
            ));
        }
        let selection_range = self.tile_pattern(select, "range")?;
        let family = self.input(plan, "family", "skill-fusion-family")?;
        let origins = self.doc.nodes[family]
            .inputs
            .get("tiers")
            .ok_or_else(|| self.error(family, Some("tiers"), "Fusion needs tiers."))?
            .clone();
        let mut tiers = Vec::new();
        let mut refs = Vec::new();
        for origin in origins {
            let id = self
                .doc
                .nodes
                .get_key_value(&origin.source.node)
                .unwrap()
                .0
                .as_str();
            self.used.insert(id);
            if self.doc.nodes[id].kind != "skill-fusion-tier" || origin.source.port != "tier" {
                return Err(self.error(id, None, "Expected a fusion tier."));
            }
            let profile = self.input_any(id, "profile")?;
            let range = self.input_any(profile, "range")?;
            let (p, ancestry) = self.mobile_profile(profile, 0)?;
            tiers.push(FusionTier {
                minimum: self.number(id, "minimum")?,
                profile: p,
            });
            refs.push((profile, range, ancestry));
        }
        let seed_ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-fusion-death-seed")
            .map(|(id, _)| id.as_str())
            .collect();
        if seed_ids.len() > 1 {
            return Err(self.error(
                family,
                None,
                "A fusion family supports one death seed declaration.",
            ));
        }
        let mut seed = if let Some(&id) = seed_ids.first() {
            self.used.insert(id);
            self.link(id, "family", family, "family")?;
            Some(FusionSeed {
                definition: format!("{}:{id}", self.doc.document),
                relic_definition: self.text(id, "relic").into(),
                duration_ms: self.positive_duration(id, "duration")?,
                modifiers: Vec::new(),
            })
        } else {
            None
        };
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
            if self.doc.nodes[id].kind == "skill-modifier-duration" {
                let seed_id = *seed_ids.first().ok_or_else(|| {
                    self.error(id, None, "Duration modifier requires a death seed.")
                })?;
                let duration = self.input(seed_id, "duration", "skill-seconds")?;
                self.link(id, "target", duration, "value")?;
                let (negative, percent) = self.signed_period_ratio(id)?;
                seed.as_mut().unwrap().modifiers.push(SeedDurationModifier {
                    id: id.into(),
                    relic_definition: relic,
                    negative,
                    percent,
                });
                continue;
            }
            let target = self.input_any(id, "target")?;
            let mut matched = false;
            for (t, (profile, range, ancestry)) in tiers.iter_mut().zip(&refs) {
                let change = match self.doc.nodes[id].kind.as_str() {
                    "skill-modifier-mobile-power" if target == *profile => {
                        Some(MobileChange::Power {
                            percent: self.counter_ratio(id, "percent")?,
                        })
                    }
                    "skill-modifier-mobile-vulnerability" if ancestry.contains(&target) => {
                        Some(MobileChange::Vulnerability {
                            percent: self.counter_ratio(id, "percent")?,
                            duration_ms: self.positive_duration(id, "duration")?,
                        })
                    }
                    "skill-modifier-range" if target == *range => {
                        let pattern = match self.text(id, "mode") {
                            "replace" => self.mobile_pattern(id, "replacement", 0)?,
                            "square-step"
                                if t.profile.range.include_anchor_tile
                                    && t.profile.range.rows == ["@"] =>
                            {
                                TileRangePattern {
                                    include_anchor_tile: true,
                                    rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                                }
                            }
                            "square-step"
                                if t.profile.range.include_anchor_tile
                                    && t.profile.range.rows == ["XXX", "X@X", "XXX"] =>
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
                                    "Use an explicit replacement for this range.",
                                ))
                            }
                        };
                        Some(MobileChange::Range { pattern })
                    }
                    _ => None,
                };
                if let Some(change) = change {
                    matched = true;
                    t.profile.modifiers.push(MobileModifier {
                        id: id.into(),
                        relic_definition: relic.clone(),
                        change,
                    });
                }
            }
            if !matched {
                return Err(self.error(id,Some("target"),"Modifier must bind to an emitted profile, its range, or inherited vulnerability."));
            }
        }
        tiers.sort_by_key(|t| t.minimum);
        let program = FusionPlacementDef {
            definition: format!("{}:{family}", self.doc.document),
            admission,
            selection_range,
            primary: self.number(plan, "primary")?,
            adjacent: self.number(plan, "adjacent")?,
            max_units: self.number(family, "maxUnits")?,
            max_tiles: self.number(family, "maxTiles")?,
            tiers,
            seed,
        };
        program
            .validate()
            .map_err(|e| self.error(family, None, &e))?;
        Ok(GraphOperation::ResonanceFusionPlacement { program })
    }
}
