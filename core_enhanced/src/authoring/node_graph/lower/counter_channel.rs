use super::*;
use crate::game::{ability::counter_channel::*, battle::damage::DamageType};
impl<'a> Lower<'a> {
    pub(super) fn consumers(&self, from: &str, output: &str, kind: &str, input: &str) -> Vec<&'a str> {
        self.doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                n.kind == kind
                    && n.inputs.get(input).is_some_and(|v| {
                        v.len() == 1 && v[0].source.node == from && v[0].source.port == output
                    })
            })
            .map(|(id, _)| id.as_str())
            .collect()
    }
    pub(super) fn consumer(
        &mut self,
        from: &str,
        output: &str,
        kind: &str,
        input: &str,
    ) -> Result<&'a str, Diagnostic> {
        let found = self.consumers(from, output, kind, input);
        if found.len() != 1 {
            return Err(self.error(
                from,
                Some(output),
                &format!("Expected one {kind} consumer."),
            ));
        }
        self.used.insert(found[0]);
        Ok(found[0])
    }
    pub(super) fn counter_ratio(
        &self,
        id: &str,
        field: &str,
    ) -> Result<crate::game::ability::marks::MarkRatio, Diagnostic> {
        if matches!(self.doc.nodes[id].values.get(field),Some(FieldValue::Number(n)) if *n==0.0) {
            return Ok(crate::game::ability::marks::MarkRatio {
                numerator: 0,
                denominator: 1,
            });
        }
        let (numerator, denominator) = self.exact_ratio(id, field)?;
        Ok(crate::game::ability::marks::MarkRatio {
            numerator,
            denominator,
        })
    }
    fn counter_range(&mut self, id: &str, port: &str) -> Result<CounterRange, Diagnostic> {
        let definition = self.input(id, port, "skill-range")?.into();
        let pattern = self.tile_pattern(id, port)?;
        Ok(CounterRange {
            definition,
            pattern,
        })
    }
    fn counter_amount(
        &mut self,
        effect: &str,
        damage: bool,
        receipt: Option<&str>,
    ) -> Result<CounterAmount, Diagnostic> {
        let power = self.input_any(effect, "power")?;
        let direct = if damage {
            "skill-damage-amount"
        } else {
            "skill-healing-amount"
        };
        let count = if damage {
            "skill-count-damage"
        } else {
            "skill-count-healing"
        };
        let (base, per_count_percent) = if self.doc.nodes[power].kind == direct {
            (
                self.number(power, "value")?,
                crate::game::ability::marks::MarkRatio {
                    numerator: 0,
                    denominator: 1,
                },
            )
        } else if self.doc.nodes[power].kind == count {
            let receipt = receipt.ok_or_else(|| {
                self.error(
                    power,
                    None,
                    "Count amount requires this admission's consumed counter receipt.",
                )
            })?;
            self.link(power, "count", receipt, "count")?;
            let base = self.input(power, "base", direct)?;
            (
                self.number(base, "value")?,
                self.counter_ratio(power, "percent")?,
            )
        } else {
            return Err(self.error(power, None, "Unsupported phase amount producer."));
        };
        Ok(CounterAmount {
            definition: power.into(),
            base,
            per_count_percent,
        })
    }
    pub(super) fn counter_channel_action(&mut self) -> Result<GraphOperation, Diagnostic> {
        let counter = self.only("skill-battle-counter")?;
        let capacity = self.number(counter, "capacity")?;
        let memory = self.only("skill-aim-memory")?;
        self.link(memory, "counter", counter, "counter")?;
        let periodic = self.only("skill-counter-periodic")?;
        self.link(periodic, "counter", counter, "counter")?;
        let period = self.input(periodic, "interval", "skill-seconds")?;
        let interval_ms = self.positive_duration(periodic, "interval")?;
        let gain = self.consumer(periodic, "out", "skill-counter-gain", "in")?;
        self.link(gain, "counter", counter, "counter")?;
        let gain_value = self.input(gain, "amount", "number")?;
        let periodic_gain = self.number(gain_value, "value")?;
        let auto = self.only("skill-counter-auto-request")?;
        self.link(auto, "counter", counter, "counter")?;
        self.link(auto, "memory", memory, "memory")?;
        let cue = self.consumer(auto, "cue", "skill-cue", "event")?;
        self.used.insert(cue);
        let requests: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == "skill-menu-request")
            .map(|(id, _)| id.as_str())
            .chain(std::iter::once(auto))
            .collect();
        let mut entries = Vec::new();
        let mut channels = BTreeMap::new();
        let mut awards = BTreeSet::new();
        let mut default_entry = None;
        for request in requests {
            self.used.insert(request);
            let automatic = request == auto;
            let key = if automatic {
                request.to_owned()
            } else {
                self.text(request, "entry").to_owned()
            };
            let qualify =
                self.consumer(request, "request", "skill-resonance-qualify", "request")?;
            self.link(qualify, "in", request, "out")?;
            let admit = self.consumer(qualify, "permit", "skill-resonance-admit", "permit")?;
            self.link(admit, "in", qualify, "out")?;
            let admission = self.request_admission(request, qualify)?;
            let terms = self.input(qualify, "terms", "skill-counter-terms")?;
            self.link(terms, "counter", counter, "counter")?;
            let minimum = self.number(terms, "minimum")?;
            let maximum = self.number(terms, "maximum")?;
            let spend = match self.text(terms, "spend") {
                "none" => CounterSpend::None,
                "admission-all" => CounterSpend::AdmissionAll,
                "end-all" => CounterSpend::EndAll,
                _ => {
                    return Err(self.error(terms, None, "Choose the counter consumption boundary."))
                }
            };
            let selection_range = if automatic {
                None
            } else {
                let selection = self.input(request, "selection", "skill-target-selection")?;
                if self.text(selection, "mode") != "player-tile" {
                    return Err(self.error(
                        selection,
                        None,
                        "Counter menu channels require explicit tile selection.",
                    ));
                }
                Some(self.tile_pattern(selection, "range")?)
            };
            let channel = self.consumer(admit, "use", "skill-sequence-channel", "use")?;
            channels.insert(channel, key.clone());
            let mut predecessor = admit;
            let receipt = if spend == CounterSpend::AdmissionAll {
                let receipt =
                    self.consumer(admit, "counterReceipt", "skill-counter-receipt", "receipt")?;
                self.link(receipt, "in", admit, "out")?;
                predecessor = receipt;
                Some(receipt)
            } else {
                None
            };
            let remember = self.consumers(admit, "use", "skill-remember-aim", "use");
            if remember.len() > 1 {
                return Err(self.error(admit, None, "Duplicate aim-memory writes."));
            }
            if let Some(&remember) = remember.first() {
                self.used.insert(remember);
                self.link(remember, "memory", memory, "memory")?;
                self.link(remember, "in", predecessor, "out")?;
                predecessor = remember;
            }
            self.link(channel, "in", predecessor, "out")?;
            let pause_basic = match self.text(channel, "basicAttack") {
                "pause" => true,
                "allow" => false,
                _ => return Err(self.error(channel, None, "Resolve channel basic action policy.")),
            };
            let protected = match self.text(channel, "interruption") {
                "standard" => false,
                "protected" => true,
                _ => return Err(self.error(channel, None, "Resolve channel interruption policy.")),
            };
            let mut phases = Vec::new();
            let mut visited = BTreeSet::new();
            let mut phase = self.input(channel, "first", "skill-channel-phase")?;
            let mut damage_results = Vec::new();
            let mut healing_results = Vec::new();
            loop {
                if !visited.insert(phase) {
                    return Err(self.error(
                        phase,
                        None,
                        "Channel phases must form a finite acyclic sequence.",
                    ));
                }
                let duration_ms = self.positive_duration(phase, "duration")?;
                let placement = self.input(phase, "area", "skill-range-placement")?;
                let origin = match self.text(placement, "origin") {
                    "selected-tile" => PhaseOrigin::SelectedTile,
                    "selected-direction" => PhaseOrigin::SelectedDirection,
                    _ => {
                        return Err(self.error(
                            placement,
                            None,
                            "Phase placement requires accepted tile or accepted direction.",
                        ))
                    }
                };
                let range_id = self.input_any(placement, "range")?;
                let range = if self.doc.nodes[range_id].kind == "skill-range" {
                    PhaseRange::Fixed(self.counter_range(placement, "range")?)
                } else if self.doc.nodes[range_id].kind == "skill-count-range" {
                    let receipt = receipt.ok_or_else(|| {
                        self.error(
                            range_id,
                            None,
                            "Count range requires a consumed-counter receipt.",
                        )
                    })?;
                    self.link(range_id, "count", receipt, "count")?;
                    let mut tiers = Vec::new();
                    for tier in self.doc.nodes[range_id]
                        .inputs
                        .get("tiers")
                        .into_iter()
                        .flatten()
                    {
                        let id = tier.source.node.as_str();
                        self.used.insert(id);
                        if self.doc.nodes[id].kind != "skill-count-range-tier"
                            || tier.source.port != "tier"
                        {
                            return Err(self.error(id, None, "Invalid count range tier."));
                        }
                        tiers.push((self.number(id, "count")?, self.counter_range(id, "range")?));
                    }
                    tiers.sort_by_key(|t| t.0);
                    PhaseRange::Consumed(tiers)
                } else {
                    return Err(self.error(range_id, None, "Unsupported phase range."));
                };
                let area = self.consumer(phase, "phase", "skill-phase-area", "phase")?;
                self.link(area, "channel", channel, "channel")?;
                let tick = self.consumer(area, "area", "skill-ground-periodic", "area")?;
                if self.text(tick, "first") != "immediate" {
                    return Err(self.error(
                        tick,
                        None,
                        "Sequence channels use immediate half-open phase ticks.",
                    ));
                }
                let interval_ms = self.positive_duration(tick, "interval")?;
                let mut previous = tick;
                let mut output = "out";
                let mut queries = Vec::new();
                loop {
                    let target_nodes =
                        self.consumers(previous, output, "skill-ground-targets", "in");
                    if target_nodes.is_empty() {
                        break;
                    }
                    if target_nodes.len() != 1 {
                        return Err(self.error(
                            previous,
                            Some(output),
                            "Phase queries require an explicit sequential order.",
                        ));
                    }
                    let targets = target_nodes[0];
                    self.used.insert(targets);
                    self.link(targets, "area", area, "area")?;
                    let filter = self.input_any(targets, "filter")?;
                    let corrupted_only = self.doc.nodes[filter].kind == "skill-corrupted-filter";
                    let (side, include_caster) = if corrupted_only {
                        (crate::game::ability::SkillHitTargetFilter::Enemies, false)
                    } else {
                        if self.doc.nodes[filter].kind != "skill-ground-filter" {
                            return Err(self.error(
                                filter,
                                None,
                                "Unsupported phase target filter.",
                            ));
                        }
                        let side = match self.text(filter, "side") {
                            "enemies" => crate::game::ability::SkillHitTargetFilter::Enemies,
                            "allies" => crate::game::ability::SkillHitTargetFilter::Allies,
                            "all" => crate::game::ability::SkillHitTargetFilter::Any,
                            _ => {
                                return Err(self.error(filter, None, "Resolve phase target side."))
                            }
                        };
                        let include = match self.text(filter, "caster") {
                            "include" => true,
                            "exclude" => false,
                            _ => {
                                return Err(self.error(
                                    filter,
                                    None,
                                    "Resolve phase caster membership.",
                                ))
                            }
                        };
                        (side, include)
                    };
                    let effects: Vec<_> = [
                        "skill-ground-pull",
                        "skill-ground-damage",
                        "skill-ground-heal",
                    ]
                    .into_iter()
                    .flat_map(|k| self.consumers(targets, "member", k, "member"))
                    .collect();
                    if effects.len() != 1 {
                        return Err(self.error(
                            targets,
                            None,
                            "Each ordered query requires one explicit effect.",
                        ));
                    }
                    let id = effects[0];
                    self.used.insert(id);
                    self.link(id, "in", targets, "out")?;
                    let effect = match self.doc.nodes[id].kind.as_str() {
                        "skill-ground-pull" => {
                            let (n, d) = self.exact_ratio(id, "tiles")?;
                            let units = u128::from(n) * 1_000_000 / u128::from(d);
                            if u128::from(n) * 1_000_000 % u128::from(d) != 0 {
                                return Err(self.error(id,None,"Pull distance must be exactly representable in native world units."));
                            }
                            PhaseEffect::Pull {
                                distance_units: u32::try_from(units).map_err(|_| {
                                    self.error(id, None, "Pull exceeds native world distance.")
                                })?,
                            }
                        }
                        "skill-ground-damage" => {
                            let amount = self.counter_amount(id, true, receipt)?;
                            let damage_type = match self.text(id, "damageType") {
                                "physical" => DamageType::Physical,
                                "magic" => DamageType::Magic,
                                "true" => DamageType::True,
                                _ => {
                                    return Err(self.error(id, None, "Resolve phase damage type."))
                                }
                            };
                            damage_results.push(id);
                            PhaseEffect::Damage {
                                amount,
                                damage_type,
                            }
                        }
                        _ => {
                            let amount = self.counter_amount(id, false, receipt)?;
                            healing_results.push(id);
                            PhaseEffect::Heal { amount }
                        }
                    };
                    queries.push(PhaseQuery {
                        definition: id.into(),
                        targets: side,
                        include_caster,
                        corrupted_only,
                        effect,
                    });
                    previous = targets;
                    output = "completed";
                }
                phases.push(CounterPhase {
                    definition: phase.into(),
                    origin,
                    range,
                    duration_ms,
                    interval_ms,
                    queries,
                });
                if !self.doc.nodes[phase].inputs.contains_key("next") {
                    break;
                }
                phase = self.input(phase, "next", "skill-channel-phase")?;
            }
            let award = self.consumers(channel, "completed", "skill-completion-award", "in");
            if award.len() > 1 {
                return Err(self.error(channel, None, "Duplicate channel completion awards."));
            }
            let completion_award = if let Some(&award) = award.first() {
                self.used.insert(award);
                awards.insert(award);
                let summary = self.input(award, "summary", "skill-cast-outcomes")?;
                self.link(summary, "channel", channel, "channel")?;
                if damage_results.len() != 1 || healing_results.len() != 1 {
                    return Err(self.error(
                        summary,
                        None,
                        "Outcome award must identify one damage and one healing result stream.",
                    ));
                }
                self.link(summary, "damage", damage_results[0], "result")?;
                self.link(summary, "healing", healing_results[0], "result")?;
                let gain = self.consumer(award, "amount", "skill-counter-gain", "amount")?;
                self.link(gain, "in", award, "out")?;
                self.link(gain, "counter", counter, "counter")?;
                Some(CompletionAward {
                    base: self.number(award, "base")?,
                    damage_and_healing_bonus: self.number(award, "bonus")?,
                })
            } else {
                None
            };
            if !remember.is_empty() {
                if default_entry.replace(entries.len()).is_some() {
                    return Err(self.error(
                        channel,
                        None,
                        "Only one manual entry may establish the default aim memory.",
                    ));
                }
            }
            entries.push(CounterEntry {
                key,
                automatic,
                admission,
                selection_range,
                minimum,
                maximum,
                spend,
                remember_aim: !remember.is_empty(),
                pause_basic,
                protected,
                phases,
                completion_award,
            });
        }
        let modifier_ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.starts_with("skill-modifier-"))
            .map(|(id, _)| id.as_str())
            .collect();
        let mut modifiers = Vec::new();
        for id in modifier_ids {
            self.used.insert(id);
            let condition = self.taunt_condition(id)?;
            let change = match self.doc.nodes[id].kind.as_str() {
                "skill-modifier-duration" => {
                    self.link(id, "target", period, "value")?;
                    let (decrease, percent) = self.signed_period_ratio(id)?;
                    CounterChange::Period { decrease, percent }
                }
                "skill-modifier-number" => {
                    let target = self.input_any(id, "target")?;
                    let target = if target == gain_value {
                        self.link(id, "target", target, "value")?;
                        CounterGainTarget::Periodic
                    } else if awards.contains(target) {
                        self.link(id, "target", target, "amount")?;
                        CounterGainTarget::Completion
                    } else {
                        return Err(self.error(id,None,"Counter gain modifier must target this counter's award or periodic amount."));
                    };
                    CounterChange::Gain {
                        target,
                        factor: self.counter_ratio(id, "factor")?,
                    }
                }
                "skill-modifier-range" => {
                    let target = self.input(id, "target", "skill-range")?;
                    self.link(id, "target", target, "range")?;
                    if self.text(id, "mode") != "replace" {
                        return Err(self.error(
                            id,
                            None,
                            "Resolve the explicit phase range replacement.",
                        ));
                    }
                    CounterChange::Range {
                        definition: target.into(),
                        pattern: self.tile_pattern(id, "replacement")?,
                    }
                }
                "skill-modifier-healing" => {
                    let target = self.input_any(id, "target")?;
                    self.link(id, "target", target, "power")?;
                    CounterChange::Healing {
                        definition: target.into(),
                        percent: self.counter_ratio(id, "percent")?,
                    }
                }
                "skill-modifier-heal-buff" => {
                    let target = self.input_any(id, "target")?;
                    self.link(id, "target", target, "power")?;
                    let state = self.input(id, "state", "skill-target-state")?;
                    let (duration_ms, profile) = self.state_profile(state)?;
                    CounterChange::HealBuff {
                        definition: target.into(),
                        duration_ms,
                        profile,
                    }
                }
                "skill-modifier-interruption" => {
                    let channel = self.input(id, "channel", "skill-sequence-channel")?;
                    let entry = channels
                        .get(channel)
                        .ok_or_else(|| {
                            self.error(id, None, "Protection must reference a compiled channel.")
                        })?
                        .clone();
                    CounterChange::Protection { entry }
                }
                _ => return Err(self.error(id, None, "Unsupported counter channel modifier.")),
            };
            modifiers.push(CounterModifier {
                id: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                change,
            });
        }
        let program = CounterChannelDef {
            definition: counter.into(),
            capacity,
            interval_ms,
            periodic_gain,
            default_entry: default_entry.ok_or_else(|| {
                self.error(
                    counter,
                    None,
                    "Counter menu requires a default remembered-aim entry.",
                )
            })?,
            entries,
            modifiers,
        };
        program
            .validate()
            .map_err(|e| self.error(counter, None, &e))?;
        Ok(GraphOperation::ResonanceCounterChannel { program })
    }
}
