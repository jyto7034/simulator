use super::*;
use crate::game::{
    ability::{marks::MarkRatio, relation_action::*},
    battle::damage::DamageType,
};
const ZERO: MarkRatio = MarkRatio {
    numerator: 0,
    denominator: 1,
};
impl<'a> Lower<'a> {
    fn relation_count(
        &mut self,
        id: &'a str,
        stock: &str,
        depth: usize,
    ) -> Result<ActionCount, Diagnostic> {
        if depth > 32 {
            return Err(self.error(id, None, "Count expression depth exceeded."));
        }
        self.used.insert(id);
        Ok(match self.doc.nodes[id].kind.as_str() {
            "skill-count-constant" | "skill-count-positive" => {
                let count = self.number(id, "value")?;
                if count == 0 && self.doc.nodes[id].kind == "skill-count-positive" {
                    return Err(self.error(
                        id,
                        Some("value"),
                        "A positive count must be at least one.",
                    ));
                }
                ActionCount::Constant(count)
            }
            "skill-count-stock" => {
                self.link(id, "bank", stock, "bank")?;
                ActionCount::StartStock
            }
            "skill-count-paid" => ActionCount::Paid,
            "skill-count-quotient" => {
                let child = self.input_any(id, "count")?;
                ActionCount::Quotient {
                    count: Box::new(self.relation_count(child, stock, depth + 1)?),
                    divisor: self.number(id, "divisor")?,
                    minimum: self.number(id, "minimum")?,
                }
            }
            "skill-count-sum" => {
                let first = self.input_any(id, "first")?;
                let second = self.input_any(id, "second")?;
                ActionCount::Sum {
                    first: Box::new(self.relation_count(first, stock, depth + 1)?),
                    second: Box::new(self.relation_count(second, stock, depth + 1)?),
                }
            }
            _ => return Err(self.error(id, None, "Unsupported action count expression.")),
        })
    }
    fn relation_status(&mut self, id: &'a str, stock: &str) -> Result<PlanStatus, Diagnostic> {
        self.used.insert(id);
        if self.doc.nodes[id].kind != "skill-hit-status" {
            return Err(self.error(id, None, "Expected a hit status definition."));
        }
        let kind = match self.text(id, "effect") {
            "stun" => PlanStatusKind::Stun,
            "root" => PlanStatusKind::Root,
            "basic-lock" => PlanStatusKind::BasicLock,
            "vulnerability" => PlanStatusKind::Vulnerability,
            _ => return Err(self.error(id, None, "Resolve the hit status.")),
        };
        let count = if self.doc.nodes[id].inputs.contains_key("count") {
            let c = self.input_any(id, "count")?;
            self.relation_count(c, stock, 0)?
        } else {
            ActionCount::Constant(1)
        };
        Ok(PlanStatus {
            definition: id.into(),
            kind,
            duration_ms: self.positive_duration(id, "duration")?,
            count,
            percent: if kind == PlanStatusKind::Vulnerability {
                self.counter_ratio(id, "percent")?
            } else {
                ZERO
            },
        })
    }
    fn relation_range(&mut self, id: &str, port: &str) -> Result<PlanRange, Diagnostic> {
        let definition = self.input(id, port, "skill-range")?.into();
        Ok(PlanRange {
            definition,
            pattern: self.tile_pattern(id, port)?,
        })
    }
    fn relation_damage_type(&self, id: &str) -> Result<DamageType, Diagnostic> {
        match self.text(id, "damageType") {
            "physical" => Ok(DamageType::Physical),
            "magic" => Ok(DamageType::Magic),
            "true" => Ok(DamageType::True),
            _ => Err(self.error(id, Some("damageType"), "Choose an explicit damage type.")),
        }
    }
    fn relation_plan(
        &mut self,
        id: &'a str,
        stock: &str,
        plans: &mut Vec<NamedActionPlan>,
        path: &mut Vec<&'a str>,
    ) -> Result<usize, Diagnostic> {
        if path.len() >= 64 || path.contains(&id) {
            return Err(self.error(id, None, "Cyclic or overly deep action plan."));
        }
        if let Some(i) = plans.iter().position(|p| p.definition == id) {
            return Ok(i);
        }
        self.used.insert(id);
        path.push(id);
        let op = match self.doc.nodes[id].kind.as_str() {
            "skill-strike-plan" => {
                let target = match self.text(id, "target") {
                    "primary" => PlanTarget::Primary,
                    "selected-area" => PlanTarget::SelectedArea,
                    "forward-area" => PlanTarget::ForwardArea,
                    "selected-direction-area" => PlanTarget::SelectedDirectionArea,
                    _ => return Err(self.error(id, None, "Resolve strike recipients.")),
                };
                let range = if target == PlanTarget::Primary {
                    if self.doc.nodes[id].inputs.contains_key("range") {
                        return Err(self.error(
                            id,
                            Some("range"),
                            "A primary strike cannot have an area range input.",
                        ));
                    }
                    None
                } else {
                    Some(self.relation_range(id, "range")?)
                };
                let p = self.input(id, "power", "skill-damage-amount")?;
                let status_ids: Vec<_> = self.doc.nodes[id]
                    .inputs
                    .get("after")
                    .into_iter()
                    .flatten()
                    .map(|o| o.source.node.as_str())
                    .collect();
                let mut after = Vec::new();
                for s in status_ids {
                    after.push(self.relation_status(s, stock)?);
                }
                let ramp = if self.doc.nodes[id].inputs.contains_key("ramp") {
                    let r = self.input(id, "ramp", "skill-hit-ramp")?;
                    self.counter_ratio(r, "percent")?
                } else {
                    ZERO
                };
                ActionPlan::Strike {
                    target,
                    power: self.number(p, "value")?,
                    damage_type: self.relation_damage_type(id)?,
                    range,
                    after,
                    ramp,
                }
            }
            "skill-sequence-plan" => {
                let a = self.input_any(id, "first")?;
                let b = self.input_any(id, "next")?;
                ActionPlan::Sequence {
                    first: self.relation_plan(a, stock, plans, path)?,
                    next: self.relation_plan(b, stock, plans, path)?,
                }
            }
            "skill-wait-plan" => ActionPlan::Wait {
                duration_ms: self.positive_duration(id, "duration")?,
            },
            "skill-stance-plan" => ActionPlan::Stance {
                duration_ms: self.positive_duration(id, "duration")?,
                reduction: self.counter_ratio(id, "percent")?,
            },
            "skill-repeat-plan" => {
                let b = self.input(id, "body", "skill-strike-plan")?;
                let c = self.input_any(id, "count")?;
                let timing = match self.text(id, "timing") {
                    "fixed-total" => PlanRepeatTiming::FixedTotal,
                    "interval" => PlanRepeatTiming::Interval {
                        delayed_first: match self.text(id, "first") {
                            "immediate" => false,
                            "after-interval" => true,
                            _ => {
                                return Err(self.error(
                                    id,
                                    None,
                                    "Resolve the first repetition time.",
                                ))
                            }
                        },
                    },
                    _ => return Err(self.error(id, None, "Resolve repetition timing.")),
                };
                ActionPlan::Repeat {
                    body: self.relation_plan(b, stock, plans, path)?,
                    count: self.relation_count(c, stock, 0)?,
                    time_ms: self.positive_duration(id, "time")?,
                    timing,
                }
            }
            "skill-spend-plan" => {
                self.link(id, "bank", stock, "bank")?;
                let amount = match self.text(id, "spend") {
                    "all" => None,
                    "amount" => {
                        let c = self.input_any(id, "count")?;
                        Some(self.relation_count(c, stock, 0)?)
                    }
                    _ => return Err(self.error(id, None, "Resolve consumption boundary.")),
                };
                let b = self.input_any(id, "body")?;
                let after = if self.doc.nodes[id].inputs.contains_key("after") {
                    let s = self.input(id, "after", "skill-hit-status")?;
                    Some(self.relation_status(s, stock)?)
                } else {
                    None
                };
                ActionPlan::Spend {
                    amount,
                    body: self.relation_plan(b, stock, plans, path)?,
                    after,
                }
            }
            "skill-timed-plan" => {
                let b = self.input(id, "body", "skill-strike-plan")?;
                let d = self.input(id, "duration", "skill-count-duration")?;
                let c = self.input_any(d, "count")?;
                ActionPlan::Timed {
                    body: self.relation_plan(b, stock, plans, path)?,
                    base_ms: self.positive_duration(d, "base")?,
                    per_count_ms: self.positive_duration(d, "perCount")?,
                    count: self.relation_count(c, stock, 0)?,
                    interval_ms: self.positive_duration(id, "interval")?,
                    delayed_first: match self.text(id, "first") {
                        "immediate" => false,
                        "after-interval" => true,
                        _ => return Err(self.error(id, None, "Resolve the first timed hit.")),
                    },
                }
            }
            _ => return Err(self.error(id, None, "Unsupported relation action plan.")),
        };
        path.pop();
        let i = plans.len();
        plans.push(NamedActionPlan {
            definition: id.into(),
            op,
        });
        Ok(i)
    }
    fn relation_entry(
        &mut self,
        id: &'a str,
        stock: &str,
        plans: &mut Vec<NamedActionPlan>,
    ) -> Result<RelationActionEntry, Diagnostic> {
        self.used.insert(id);
        let plan = self.input_any(id, "plan")?;
        let next_basic = match self.text(id, "start") {
            "next-basic" => true,
            "immediate" => false,
            _ => return Err(self.error(id, None, "Resolve action start.")),
        };
        let (interrupt_on_silence, protected) = match self.text(id, "interruption") {
            "standard" => (false, false),
            "silence" => (true, false),
            "protected" => (false, true),
            _ => return Err(self.error(id, None, "Resolve action interruption.")),
        };
        let selector = if self.doc.nodes[id].inputs.contains_key("selector") {
            let s = self.input(id, "selector", "skill-enemy-selector")?;
            if self.text(s, "order") != "block-route" || self.text(s, "air") != "exclude" {
                return Err(self.error(
                    s,
                    None,
                    "This primary selector requires block/route order and ground enemies.",
                ));
            }
            Some(self.relation_range(s, "range")?)
        } else {
            None
        };
        Ok(RelationActionEntry {
            definition: id.into(),
            plan: self.relation_plan(plan, stock, plans, &mut Vec::new())?,
            next_basic,
            interrupt_on_silence,
            protected,
            selector,
            self_damage: None,
            manual_stop: false,
        })
    }
    pub(super) fn relation_action(&mut self) -> Result<GraphOperation, Diagnostic> {
        let stock = self.only("skill-stock")?;
        if self.text(stock, "lifetime") != "battle" {
            return Err(self.error(
                stock,
                None,
                "Relationship stock must belong to the battle copy.",
            ));
        }
        let stock_capacity = match self.text(stock, "limit") {
            "bounded" => Some(self.number(stock, "capacity")?),
            "unbounded" => None,
            _ => return Err(self.error(stock, None, "Resolve stock limit.")),
        };
        let mode = self.only("skill-battle-mode")?;
        let relation = self.only("skill-ally-relation")?;
        self.link(relation, "blocked", mode, "mode")?;
        let reduction =
            self.consumer(relation, "relation", "skill-relation-reduction", "relation")?;
        let shield = self.consumer(
            relation,
            "relation",
            "skill-relation-emergency-shield",
            "relation",
        )?;
        let periodic =
            self.consumer(relation, "relation", "skill-relation-periodic", "relation")?;
        self.link(periodic, "bank", stock, "bank")?;
        let period_id = self.input(periodic, "interval", "skill-seconds")?;
        let gain = self.consumer(periodic, "grant", "skill-resource-credit", "grant")?;
        self.link(gain, "in", periodic, "out")?;
        self.link(gain, "bank", stock, "bank")?;
        let death = self.consumer(relation, "relation", "skill-relation-hp-zero", "relation")?;
        let defer = self.consumer(death, "out", "skill-defer-mode", "in")?;
        self.link(defer, "event", death, "event")?;
        self.link(defer, "mode", mode, "mode")?;
        let entered = self.consumer(mode, "mode", "skill-mode-entered", "mode")?;
        let clear = self.consumer(entered, "out", "skill-relation-clear", "in")?;
        self.link(clear, "relation", relation, "relation")?;
        let self_damage = self.consumer(clear, "out", "skill-self-stock-damage", "in")?;
        let count = self.input(self_damage, "count", "skill-count-stock")?;
        self.link(count, "bank", stock, "bank")?;
        if self.text(self_damage, "damage") != "fixed-bypass" {
            return Err(self.error(
                self_damage,
                None,
                "Mode entry requires explicit bypass damage.",
            ));
        }
        let request = self.only("skill-request")?;
        let qualify = self.only("skill-resonance-qualify")?;
        let admit = self.only("skill-resonance-admit")?;
        self.link(qualify, "in", request, "out")?;
        self.link(qualify, "request", request, "request")?;
        self.link(admit, "in", qualify, "out")?;
        self.link(admit, "permit", qualify, "permit")?;
        let admission = self.request_admission(request, qualify)?;
        let program = self.input(qualify, "program", "skill-mode-program")?;
        self.link(program, "mode", mode, "mode")?;
        let run = self.only("skill-run-action")?;
        self.link(run, "in", admit, "out")?;
        self.link(run, "use", admit, "use")?;
        self.link(run, "program", program, "program")?;
        let selection = self.input(request, "selection", "skill-target-selection")?;
        let selection_range = match self.text(selection, "mode") {
            "self" => None,
            "player-tile" => Some(self.tile_pattern(selection, "range")?),
            _ => {
                return Err(self.error(selection, None, "Resolve self or tile request selection."))
            }
        };
        let normal_id = self.input(program, "normal", "skill-action-entry")?;
        let active_id = self.input(program, "active", "skill-action-entry")?;
        let mut plans = Vec::new();
        let mut normal = self.relation_entry(normal_id, stock, &mut plans)?;
        let mut active = self.relation_entry(active_id, stock, &mut plans)?;
        let self_ticks: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-action-periodic-self-damage")
            .map(|(id, _)| id.as_str())
            .collect();
        for id in self_ticks {
            self.used.insert(id);
            self.link(id, "run", run, "run")?;
            let e = self.input(id, "entry", "skill-action-entry")?;
            let entry = if e == normal_id {
                &mut normal
            } else if e == active_id {
                &mut active
            } else {
                return Err(self.error(id, None, "Self damage must belong to this mode entry."));
            };
            if entry.self_damage.is_some() {
                return Err(self.error(id, None, "Duplicate periodic self damage."));
            }
            entry.self_damage = Some((
                self.positive_duration(id, "interval")?,
                self.counter_ratio(id, "percent")?,
                self.relation_damage_type(id)?,
            ));
        }
        let stops: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-action-stop-input")
            .map(|(id, _)| id.as_str())
            .collect();
        if stops.len() > 1 {
            return Err(self.error(run, None, "Duplicate stop subscription."));
        }
        for id in stops {
            self.used.insert(id);
            self.link(id, "run", run, "run")?;
            for e in [&mut normal, &mut active] {
                e.manual_stop = matches!(plans[e.plan].op, ActionPlan::Timed { .. });
            }
            if !normal.manual_stop && !active.manual_stop {
                return Err(self.error(id, None, "Stop input requires a timed action."));
            }
        }
        let mut modifiers = Vec::new();
        let ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.starts_with("skill-modifier-"))
            .map(|(id, _)| id.as_str())
            .collect();
        for id in ids {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let relic_definition = self.text(condition, "definition").into();
            let target = if self.doc.nodes[id].kind == "skill-modifier-control-vulnerability" {
                self.input(id, "hit", "skill-hit-status")?
            } else if self.doc.nodes[id].kind == "skill-modifier-interruption" {
                self.input(id, "program", "skill-mode-program")?
            } else {
                self.input_any(id, "target")?
            };
            let wrong = || {
                self.error(
                    id,
                    None,
                    "Modifier target is outside this declared relationship/action.",
                )
            };
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-duration" => {
                    if target != period_id {
                        return Err(wrong());
                    }
                    let (negative, percent) = self.signed_period_ratio(id)?;
                    RelationActionChange::Period {
                        definition: target.into(),
                        negative,
                        percent,
                    }
                }
                "skill-modifier-capacity" => {
                    if target != stock {
                        return Err(wrong());
                    }
                    RelationActionChange::UnlimitedStock
                }
                "skill-modifier-grant" => {
                    if target != periodic {
                        return Err(wrong());
                    }
                    RelationActionChange::Gain {
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-relation-shield" => {
                    if target != relation {
                        return Err(wrong());
                    }
                    RelationActionChange::Shield {
                        percent: self.counter_ratio(id, "percent")?,
                    }
                }
                "skill-modifier-hit-count" => {
                    if !plans.iter().any(|p| match &p.op {
                        ActionPlan::Strike { after, .. } => {
                            after.iter().any(|s| s.definition == target)
                        }
                        ActionPlan::Spend { after, .. } => {
                            after.as_ref().is_some_and(|s| s.definition == target)
                        }
                        _ => false,
                    }) {
                        return Err(wrong());
                    }
                    RelationActionChange::StatusCount {
                        definition: target.into(),
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-interruption" => {
                    if target != program {
                        return Err(wrong());
                    }
                    RelationActionChange::Protection
                }
                "skill-modifier-primary-area" => {
                    if !plans.iter().any(|p| {
                        p.definition == target
                            && matches!(
                                p.op,
                                ActionPlan::Strike {
                                    target: PlanTarget::Primary,
                                    ..
                                }
                            )
                    }) {
                        return Err(wrong());
                    }
                    RelationActionChange::PrimaryArea {
                        definition: target.into(),
                    }
                }
                "skill-modifier-range" => {
                    if !plans.iter().any(|p|matches!(&p.op,ActionPlan::Strike{range:Some(r),..} if r.definition==target))||self.text(id,"mode")!="replace"{return Err(wrong());}
                    RelationActionChange::Range {
                        definition: target.into(),
                        pattern: self.tile_pattern(id, "replacement")?,
                    }
                }
                "skill-modifier-multihit" => {
                    let e = if plans[normal.plan].definition == target {
                        &normal
                    } else if plans[active.plan].definition == target {
                        &active
                    } else {
                        return Err(wrong());
                    };
                    RelationActionChange::Multihit {
                        entry: e.definition.clone(),
                        step: self.counter_ratio(id, "step")?,
                        cap: self.counter_ratio(id, "cap")?,
                    }
                }
                "skill-modifier-retarget" => {
                    if target != normal_id && target != active_id {
                        return Err(wrong());
                    }
                    let (n, d) = self.exact_ratio(id, "tiles")?;
                    let scaled = u128::from(n) * 1_000_000;
                    if scaled % u128::from(d) != 0 {
                        return Err(wrong());
                    }
                    RelationActionChange::Retarget {
                        entry: target.into(),
                        distance_units: u32::try_from(scaled / u128::from(d))
                            .map_err(|_| wrong())?,
                    }
                }
                "skill-modifier-control-vulnerability" => {
                    if self.text(target, "effect") != "stun" {
                        return Err(wrong());
                    }
                    RelationActionChange::ControlVulnerability {
                        definition: target.into(),
                        percent: self.number(id, "percent")?,
                        duration_ms: self.positive_duration(id, "duration")?,
                    }
                }
                _ => return Err(self.error(id, None, "Unsupported relationship modifier.")),
            };
            modifiers.push(RelationActionModifier {
                id: id.into(),
                relic_definition,
                change,
            });
        }
        let value = RelationActionDef {
            definition: program.into(),
            stock_definition: stock.into(),
            relation_definition: relation.into(),
            mode_definition: mode.into(),
            menu_entry: self.text(relation, "entry").into(),
            admission,
            selection_range,
            initial_stock: self.number(stock, "initial")?,
            stock_capacity,
            period_ms: self.positive_duration(periodic, "interval")?,
            periodic_gain: self.number(gain, "amount")?,
            reduction: self.counter_ratio(reduction, "percent")?,
            shield_threshold: self.number(shield, "threshold")?,
            shield_percent: self.counter_ratio(shield, "shield")?,
            shield_duration_ms: self.positive_duration(shield, "duration")?,
            mode_damage_percent: self.counter_ratio(self_damage, "percent")?,
            normal,
            active,
            plans,
            modifiers,
        };
        value
            .validate()
            .map_err(|e| self.error(program, None, &e))?;
        Ok(GraphOperation::ResonanceRelationAction { program: value })
    }
}
