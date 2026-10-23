use super::*;
use crate::game::ability::{
    input_plan::*,
    relation_action::{ActionCount, PlanStatus, PlanStatusKind},
};
use crate::game::battle::damage::DamageType;
impl<'a> Lower<'a> {
    fn shot_condition(
        &mut self,
        id: &'a str,
        path: &mut Vec<&'a str>,
        result: &mut Vec<String>,
    ) -> Result<(), Diagnostic> {
        if path.len() >= 32 || path.contains(&id) {
            return Err(self.error(id, None, "Cyclic or deep equipment prerequisites."));
        }
        if self.doc.nodes[id].kind != "skill-equipment-condition"
            || self.text(id, "slot") != "relic"
            || self.text(id, "variant") != "normal"
            || self.text(id, "copies") != "once"
        {
            return Err(self.error(
                id,
                None,
                "Expected a normal equipped relic condition, applied once.",
            ));
        }
        self.used.insert(id);
        path.push(id);
        let children: Vec<_> = self.doc.nodes[id]
            .inputs
            .get("requires")
            .into_iter()
            .flatten()
            .map(|o| o.source.node.as_str())
            .collect();
        for child in children {
            self.shot_condition(child, path, result)?;
            let name = self.text(child, "definition").to_owned();
            if !result.contains(&name) {
                result.push(name);
            }
        }
        path.pop();
        Ok(())
    }
    pub(super) fn shot_damage_type(&self, id: &str) -> Result<DamageType, Diagnostic> {
        match self.text(id, "damageType") {
            "physical" => Ok(DamageType::Physical),
            "magic" => Ok(DamageType::Magic),
            "true" => Ok(DamageType::True),
            _ => Err(self.error(id, Some("damageType"), "Resolve damage type.")),
        }
    }
    pub(super) fn shot_choice(&self, id: &str, field: &str, yes: &str, no: &str) -> Result<bool, Diagnostic> {
        match self.text(id, field) {
            s if s == yes => Ok(true),
            s if s == no => Ok(false),
            _ => Err(self.error(id, Some(field), "Resolve this policy.")),
        }
    }
    pub(super) fn shot_status(&mut self, id: &'a str) -> Result<PlanStatus, Diagnostic> {
        self.used.insert(id);
        let kind = match self.text(id, "effect") {
            "stun" => PlanStatusKind::Stun,
            "root" => PlanStatusKind::Root,
            "basic-lock" => PlanStatusKind::BasicLock,
            "vulnerability" => PlanStatusKind::Vulnerability,
            _ => return Err(self.error(id, None, "Resolve hit status.")),
        };
        let count = if self.doc.nodes[id].inputs.contains_key("count") {
            let c = self.input_any(id, "count")?;
            if !matches!(
                self.doc.nodes[c].kind.as_str(),
                "skill-count-constant" | "skill-count-positive"
            ) {
                return Err(self.error(c, None, "Projectile status requires a constant count."));
            }
            self.number(c, "value")?
        } else {
            1
        };
        Ok(PlanStatus {
            definition: id.into(),
            kind,
            duration_ms: self.positive_duration(id, "duration")?,
            count: ActionCount::Constant(count),
            percent: if kind == PlanStatusKind::Vulnerability {
                self.counter_ratio(id, "percent")?
            } else {
                crate::game::ability::marks::MarkRatio {
                    numerator: 0,
                    denominator: 1,
                }
            },
        })
    }
    pub(super) fn shot_power(&mut self, id: &'a str, depth: usize) -> Result<u32, Diagnostic> {
        if depth > 32 {
            return Err(self.error(id, None, "Damage expression depth exceeded."));
        }
        self.used.insert(id);
        match self.doc.nodes[id].kind.as_str() {
            "skill-damage-amount" => self.number(id, "value"),
            "skill-damage-bonus" => {
                let base = self.input_any(id, "base")?;
                let value = self.shot_power(base, depth + 1)?;
                let bonus = self.number(id, "bonus")?;
                if bonus == 0 {
                    return Err(self.error(id, None, "Additional power must be positive."));
                }
                value
                    .checked_add(bonus)
                    .ok_or_else(|| self.error(id, None, "Power overflow."))
            }
            _ => Err(self.error(id, None, "Unsupported projectile power.")),
        }
    }
    fn shot_projectile(&mut self, id: &'a str) -> Result<ShotProjectileDef, Diagnostic> {
        if self.text(id, "delivery") != "directional"
            || self.text(id, "tracking") != "fixed"
            || self.doc.nodes[id].inputs.contains_key("area")
        {
            return Err(self.error(
                id,
                None,
                "An input volley requires fixed directional projectiles.",
            ));
        }
        let power = self.input_any(id, "power")?;
        let after_ids: Vec<_> = self.doc.nodes[id]
            .inputs
            .get("after")
            .into_iter()
            .flatten()
            .map(|o| o.source.node.as_str())
            .collect();
        let mut after = Vec::new();
        for s in after_ids {
            after.push(self.shot_status(s)?);
        }
        Ok(ShotProjectileDef {
            definition: id.into(),
            power: self.shot_power(power, 0)?,
            damage_type: self.shot_damage_type(id)?,
            speed: self.counter_ratio(id, "speed")?,
            range: self.counter_ratio(id, "range")?,
            width: self.counter_ratio(id, "width")?,
            lifetime_ms: self.positive_duration(id, "lifetime")?,
            max_hits: match self.text(id, "pierce") {
                "count" => Some(self.number(id, "hits")?),
                "unbounded" => None,
                _ => return Err(self.error(id, None, "Resolve projectile hit limit.")),
            },
            air: self.shot_choice(id, "air", "include", "exclude")?,
            obstacle_stop: self.shot_choice(id, "obstacle", "stop", "pass")?,
            after,
        })
    }
    fn shot_stock(&mut self, id: &'a str) -> Result<ShotStockDef, Diagnostic> {
        if self.text(id, "lifetime") != "battle" {
            return Err(self.error(id, None, "Input plan stocks require battle-copy lifetime."));
        }
        Ok(ShotStockDef {
            definition: id.into(),
            initial: self.number(id, "initial")?,
            capacity: match self.text(id, "limit") {
                "bounded" => Some(self.number(id, "capacity")?),
                "unbounded" => None,
                _ => return Err(self.error(id, None, "Resolve stock capacity.")),
            },
        })
    }
    pub(super) fn input_plan(&mut self) -> Result<GraphOperation, Diagnostic> {
        let prepare = self.only("skill-plan-prepare")?;
        let memory = self.input(prepare, "memory", "skill-layout-memory")?;
        let normal_id = self.input(memory, "normal", "skill-stock")?;
        let enhanced_id = self.input(memory, "enhanced", "skill-stock")?;
        let normal = self.shot_stock(normal_id)?;
        let enhanced = self.shot_stock(enhanced_id)?;
        let periodic = self.consumer(normal_id, "bank", "skill-stock-periodic", "bank")?;
        if self.text(periodic, "scope") != "equipped" {
            return Err(self.error(
                periodic,
                None,
                "Input plan periodic stock uses the equipped battle copy.",
            ));
        }
        let gain = self.consumer(periodic, "grant", "skill-resource-credit", "grant")?;
        self.link(gain, "in", periodic, "out")?;
        self.link(gain, "bank", normal_id, "bank")?;
        let spawner = self.only("skill-hostile-spawner")?;
        let weakness = self.input(spawner, "profile", "skill-hostile-family-weakness")?;
        let hostile = self.input(weakness, "base", "skill-hostile-profile")?;
        let killed = self.consumer(spawner, "cohort", "skill-cohort-killed", "cohort")?;
        self.link(killed, "bank", enhanced_id, "bank")?;
        let reward = self.consumer(killed, "grant", "skill-resource-credit", "grant")?;
        self.link(reward, "in", killed, "out")?;
        self.link(reward, "bank", enhanced_id, "bank")?;
        let cohort = HostileCohortDef {
            definition: spawner.into(),
            interval_ms: self.positive_duration(spawner, "interval")?,
            hp: self.number(hostile, "hp")?,
            power: self.number(hostile, "power")?,
            speed: self.counter_ratio(hostile, "speed")?,
            physical_defense: self.number(hostile, "physicalDefense")?,
            magic_defense: self.number(hostile, "magicDefense")?,
            block: self.number(hostile, "block")?,
            leak: self.number(hostile, "leak")?,
            damage_type: self.shot_damage_type(hostile)?,
            attack_range: self.tile_pattern(hostile, "range")?,
            attack_interval_ms: self.positive_duration(hostile, "interval")?,
            weakness_family: self.text(weakness, "family").into(),
            weakness_percent: self.counter_ratio(weakness, "percent")?,
            kill_gain: self.number(reward, "amount")?,
        };
        let layout = self.consumer(prepare, "out", "skill-plan-layout", "in")?;
        self.link(layout, "preparation", prepare, "preparation")?;
        let mode = self.consumer(layout, "out", "skill-plan-aim-mode", "in")?;
        self.link(mode, "layout", layout, "layout")?;
        let precise = self.consumer(mode, "precise", "skill-plan-precise-aim", "in")?;
        let focused = self.consumer(mode, "focused", "skill-plan-focused-aim", "in")?;
        for p in [precise, focused] {
            self.link(p, "layout", layout, "layout")?;
        }
        let merge = self.only("skill-plan-merge")?;
        self.link(merge, "precise", precise, "plan")?;
        self.link(merge, "focused", focused, "plan")?;
        let edges = self.doc.nodes[merge].inputs.get("in");
        if !edges.is_some_and(|e| {
            e.len() == 2
                && [precise, focused].iter().all(|p| {
                    e.iter()
                        .any(|o| o.source.node == *p && o.source.port == "out")
                })
        }) {
            return Err(self.error(
                merge,
                Some("in"),
                "Merge must pair the precise and focused branch completions.",
            ));
        }
        let qualify = self.consumer(merge, "out", "skill-resonance-qualify", "in")?;
        self.link(qualify, "plan", merge, "plan")?;
        self.link(qualify, "request", prepare, "request")?;
        let read = self.input(qualify, "snapshot", "skill-resonance-read")?;
        let settings = self.input(read, "settings", "skill-resonance-settings")?;
        self.link(prepare, "settings", settings, "settings")?;
        let requirement = self.input(qualify, "requirement", "skill-resonance-requirement")?;
        self.link(prepare, "requirement", requirement, "requirement")?;
        let admission = self.request_admission(prepare, qualify)?;
        let commit = self.consumer(qualify, "out", "skill-plan-commit", "in")?;
        self.link(commit, "permit", qualify, "permit")?;
        self.link(commit, "plan", merge, "plan")?;
        let normal_p = self.input(commit, "normal", "skill-projectile-profile")?;
        let enhanced_p = self.input(commit, "enhanced", "skill-projectile-profile")?;
        let normal_projectile = self.shot_projectile(normal_p)?;
        let enhanced_projectile = self.shot_projectile(enhanced_p)?;
        self.consumer(commit, "batch", "skill-projectile-results", "batch")?;
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
            let condition = self.input(id, "condition", "skill-equipment-condition")?;
            let mut requires = Vec::new();
            self.shot_condition(condition, &mut Vec::new(), &mut requires)?;
            let relic_definition = self.text(condition, "definition").into();
            let profile = if matches!(
                self.doc.nodes[id].kind.as_str(),
                "skill-modifier-projectile-status"
                    | "skill-modifier-projectile-push"
                    | "skill-modifier-projectile-explosion"
            ) {
                let p = self.input(id, "target", "skill-projectile-profile")?;
                if p != normal_p && p != enhanced_p {
                    return Err(self.error(id, None, "Modifier references another volley."));
                }
                Some(p)
            } else {
                None
            };
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-grant" => {
                    self.link(id, "target", killed, "grant")?;
                    ShotChange::KillGain {
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-projectile-wound" => {
                    let profiles = self.doc.nodes[id]
                        .inputs
                        .get("profiles")
                        .into_iter()
                        .flatten()
                        .map(|o| o.source.node.clone())
                        .collect();
                    ShotChange::Wound {
                        profiles,
                        amount: self.number(id, "amount")?,
                        cap: self.number(id, "cap")?,
                        percent: self.counter_ratio(id, "percent")?,
                        duration_ms: self.positive_duration(id, "duration")?,
                    }
                }
                "skill-modifier-wound-grant" => {
                    let w = self.input(id, "target", "skill-modifier-projectile-wound")?;
                    ShotChange::WoundGrant {
                        definition: w.into(),
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-projectile-status" => {
                    let s = self.input(id, "status", "skill-hit-status")?;
                    ShotChange::Status {
                        profile: profile.unwrap().into(),
                        status: self.shot_status(s)?,
                    }
                }
                "skill-modifier-projectile-push" => ShotChange::Push {
                    profile: profile.unwrap().into(),
                    tiles: self.counter_ratio(id, "tiles")?,
                },
                "skill-modifier-projectile-explosion" => {
                    let e = self.input(id, "explosion", "skill-explosion-rule")?;
                    if self.text(e, "dedup") != "blast" {
                        return Err(self.error(
                            e,
                            None,
                            "Piercing explosions deduplicate per blast.",
                        ));
                    }
                    let power = self.input_any(id, "power")?;
                    ShotChange::Explosion {
                        profile: profile.unwrap().into(),
                        power: self.shot_power(power, 0)?,
                        damage_type: self.shot_damage_type(e)?,
                        range: self.tile_pattern(id, "range")?,
                        air: self.shot_choice(id, "air", "include", "exclude")?,
                        bonus: self.counter_ratio(e, "bonus")?,
                    }
                }
                "skill-modifier-control-vulnerability" => {
                    let status = self.input(id, "hit", "skill-hit-status")?;
                    ShotChange::ControlVulnerability {
                        definition: status.into(),
                        percent: self.number(id, "percent")?,
                        duration_ms: self.positive_duration(id, "duration")?,
                    }
                }
                _ => return Err(self.error(id, None, "Unsupported input plan modifier.")),
            };
            modifiers.push(ShotModifier {
                id: id.into(),
                relic_definition,
                requires,
                change,
            });
        }
        let program = InputPlanDef {
            definition: prepare.into(),
            admission,
            normal,
            enhanced,
            stock_interval_ms: self.positive_duration(periodic, "interval")?,
            stock_gain: self.number(gain, "amount")?,
            cohort,
            placement_range: self.tile_pattern(prepare, "range")?,
            allow_duplicates: self.shot_choice(layout, "duplicates", "allow", "reject")?,
            normal_projectile,
            enhanced_projectile,
            modifiers,
        };
        program
            .validate()
            .map_err(|e| self.error(prepare, None, &e))?;
        Ok(GraphOperation::ResonanceInputPlan { program })
    }
}
