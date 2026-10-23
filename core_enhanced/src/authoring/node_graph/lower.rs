mod status_strike;
mod mobile_effect;
mod paired_areas;
mod prepared_volley;
mod input_plan;
mod relation_action;
mod attack_overlay;
mod counter_channel;
mod channel;
mod marks;
mod tile_burst;
mod delayed_area;
mod strike;
mod taunt;
mod ground;
mod ground_status;
mod parry;
mod prepared_basic;
mod reactive_heal;
mod scheduled_area;
mod support;
mod target_state;
use super::{program::*, *};
use crate::game::ability::{resonance::*, SkillId};
use std::collections::{BTreeMap, BTreeSet};

/// The only graph compilation entry accepts source bytes, never an unchecked
/// deserialized normalization or caller-supplied claim that validation succeeded.
pub fn compile(source: &str, skill_id: SkillId) -> Result<PortableGraph, Vec<Diagnostic>> {
    let doc = read_and_normalize(source)?;
    let mut errors = authoring_gaps(&doc);
    errors.extend(resonance_scope_diagnostics(&doc));
    let supported = [
        "skill-reactive-status", "skill-owned-damage-status", "skill-convert-status-plan", "skill-spawn-mobile-plan", "skill-status-death-mobile",
        "skill-mobile-effect", "skill-mobile-variant", "skill-scaled-range", "skill-fusion-count-power", "skill-fusion-tier", "skill-fusion-family", "skill-place-fusion-plan", "skill-fusion-death-seed", "skill-modifier-mobile-power", "skill-modifier-mobile-vulnerability",
        "skill-two-tile-selection", "skill-paired-areas", "skill-pair-pulse", "skill-pair-base-heal", "skill-pair-transfer", "skill-pair-intersection", "skill-attack-speed-buff", "skill-modifier-buff", "skill-modifier-coincident-area-burst",
        "skill-credit-ledger", "skill-damage-mark", "skill-damage-recovery", "skill-portal-policy", "skill-portal-program", "skill-run-portals", "skill-portal-release-request", "skill-portal-freeze", "skill-serial-projectiles", "skill-portal-stop-input", "skill-modifier-projectile-multihit", "skill-modifier-continuous-volley",
        "skill-plan-prepare", "skill-layout-memory", "skill-stock-periodic", "skill-hostile-profile", "skill-hostile-family-weakness", "skill-hostile-spawner", "skill-cohort-killed", "skill-plan-layout", "skill-plan-aim-mode", "skill-plan-precise-aim", "skill-plan-focused-aim", "skill-plan-merge", "skill-plan-commit", "skill-projectile-profile", "skill-projectile-results", "skill-damage-bonus", "skill-modifier-projectile-wound", "skill-modifier-wound-grant", "skill-modifier-projectile-status", "skill-modifier-projectile-push", "skill-modifier-projectile-explosion",
        "skill-stock", "skill-battle-mode", "skill-ally-relation", "skill-relation-reduction", "skill-relation-emergency-shield", "skill-relation-periodic", "skill-resource-credit", "skill-relation-hp-zero", "skill-defer-mode", "skill-mode-entered", "skill-relation-clear", "skill-self-stock-damage", "skill-count-stock", "skill-count-paid", "skill-count-constant", "skill-count-quotient", "skill-count-sum", "skill-count-duration", "skill-repeat-plan", "skill-spend-plan", "skill-stance-plan", "skill-timed-plan", "skill-hit-ramp", "skill-mode-program", "skill-action-periodic-self-damage", "skill-modifier-grant", "skill-modifier-hit-count", "skill-modifier-capacity", "skill-modifier-primary-area", "skill-modifier-retarget", "skill-modifier-relation-shield",
        "skill-modifier-piercing", "skill-state", "skill-lookup", "skill-permission", "skill-claim", "skill-resonance-install", "skill-attack-profile", "skill-basic-event", "skill-capture-time", "skill-time-power", "skill-area", "skill-area-damage", "skill-finish", "skill-expiry", "skill-owner-exit",
        "number", "skill-menu-request", "skill-battle-counter", "skill-counter-periodic", "skill-counter-gain", "skill-counter-terms", "skill-counter-receipt", "skill-aim-memory", "skill-remember-aim", "skill-counter-auto-request", "skill-sequence-channel", "skill-channel-phase", "skill-phase-area", "skill-corrupted-filter", "skill-ground-pull", "skill-count-range", "skill-count-range-tier", "skill-count-damage", "skill-count-healing", "skill-cast-outcomes", "skill-completion-award", "skill-modifier-healing", "skill-modifier-heal-buff",
        "skill-path-burst-plan", "skill-stack-burst-plan", "skill-stack-grade", "skill-burst-conversion", "skill-weapon-evolution", "skill-weapon-mark-owner", "skill-modifier-mark-grant",
        "skill-target-marks", "skill-mark-filter", "skill-mark-weapon-hit", "skill-mark-spread", "skill-population-power", "skill-mark-periodic-damage", "skill-mark-burst-plan", "skill-explosion-rule", "skill-modifier-population", "skill-modifier-explosion-power", "skill-modifier-explosion-overlap",
        "skill-channel-begin", "skill-channel-cancel-request", "skill-channel-cancel", "skill-channel-ended", "skill-channel-area", "skill-ground-ramping-slow", "skill-modifier-ramp",
        "skill-range-targets", "skill-target-taunt", "skill-target-speed", "skill-target-attack-limit", "skill-target-apply", "skill-target-ended", "skill-modifier-taunt-damage", "skill-weapon-kill-growth", "skill-integer-amount", "skill-modifier-number",
        "skill-strike-plan", "skill-hit-pull", "skill-hit-status", "skill-limited-range", "skill-missing-hp-power",
        "skill-modifier-range", "skill-modifier-duration", "skill-modifier-control-vulnerability", "skill-modifier-multihit", "skill-modifier-interruption",
        "skill-ground-enter", "skill-ground-exit", "skill-ground-dwell", "skill-ground-status", "skill-ground-apply-status", "skill-ground-clear-status",
        "skill-ground-area", "skill-ground-periodic", "skill-ground-targets", "skill-ground-filter",
        "skill-ground-damage", "skill-ground-heal", "skill-ground-ended", "skill-cue",
        "skill-damage-amount", "skill-healing-amount", "skill-range-placement",
        "skill-aura-buff-plan",
        "skill-action-stop-input",
        "skill-equipment-condition",
        "skill-personal-shield-effect",
        "skill-modifier-single-support",
        "skill-parry-plan",
        "skill-passive-guard",
        "skill-direct-reflection", "skill-guard-charge",
        "skill-next-basic-plan",
        "skill-scheduled-plan",
        "skill-count-positive",
        "skill-sequence-plan",
        "skill-wait-plan",
        "skill-on-hit-plan",
        "skill-self-heal-plan",
        "skill-delayed-area-plan",
        "skill-enemy-selector",
        "skill-weapon-power",
        "skill-reactive-heal-plan",
        "skill-range",
        "skill-target-selection",
        "skill-run-action",
        "skill-action-entry",
        "skill-recipient-buff-plan",
        "skill-target-state",
        "skill-seconds",
        "skill-attribute-effect",
        "skill-hp-floor-effect",
        "skill-request",
        "skill-resonance-amount",
        "skill-resonance-settings",
        "skill-resonance-read",
        "skill-resonance-requirement",
        "skill-resonance-cost",
        "skill-resonance-qualify",
        "skill-resonance-admit",
        "skill-resonance-result",
        "skill-resonance-end",
    ];
    for (id, node) in &doc.nodes {
        if !supported.contains(&node.kind.as_str()) {
            errors.push(issue(
                &doc,
                Some(id),
                None,
                "graph.target_unsupported",
                "This operation has no lowering in the selected native target.",
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut lower = Lower {
        doc: &doc,
        used: BTreeSet::new(),
    };
    let operation = if doc.nodes.values().any(|n| n.kind == "skill-portal-program") { lower.prepared_volley() } else if doc.nodes.values().any(|n| n.kind == "skill-plan-prepare") { lower.input_plan() } else if doc.nodes.values().any(|n| n.kind == "skill-mode-program") { lower.relation_action() } else if doc.nodes.values().any(|n| n.kind == "skill-resonance-install") { lower.attack_overlay_action() } else if doc.nodes.values().any(|n| n.kind == "skill-battle-counter") { lower.counter_channel_action() } else { lower.resonance() }.map_err(|e| vec![e])?;
    for id in doc.nodes.keys() {
        if !lower.used.contains(id.as_str()) {
            errors.push(issue(
                &doc,
                Some(id),
                None,
                "graph.entry_unbound",
                "This node is not part of the executable request.",
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut source_map = BTreeMap::new();
    for (path, kind) in [
        (
            "operation.ResonanceUse.admission.capacity",
            "skill-resonance-settings",
        ),
        (
            "operation.ResonanceUse.admission.requirement",
            "skill-resonance-requirement",
        ),
        (
            "operation.ResonanceUse.admission.cost",
            "skill-resonance-cost",
        ),
        ("native.admission", "skill-resonance-admit"),
        ("native.steps.finish", "skill-resonance-end"),
    ] {
        let Some(root) = doc
            .nodes
            .iter()
            .find(|(_, node)| node.kind == kind)
            .map(|(id, _)| id)
        else {
            continue;
        };
        let operation_name = match operation {
            GraphOperation::ResonanceStatusStrike{..} => "ResonanceStatusStrike",
            GraphOperation::ResonanceFusionPlacement{..} => "ResonanceFusionPlacement",
            GraphOperation::ResonancePairedAreas{..} => "ResonancePairedAreas",
            GraphOperation::ResonancePreparedVolley{..} => "ResonancePreparedVolley",
            GraphOperation::ResonanceInputPlan{..} => "ResonanceInputPlan",
            GraphOperation::ResonanceRelationAction{..} => "ResonanceRelationAction",
            GraphOperation::ResonanceAttackOverlay { .. } => "ResonanceAttackOverlay",
            GraphOperation::ResonanceCounterChannel { .. } => "ResonanceCounterChannel",
            GraphOperation::ResonanceTileBurst { .. } => "ResonanceTileBurst",
            GraphOperation::ResonancePresenceBurst { .. } => "ResonancePresenceBurst",
            GraphOperation::ResonanceChannelGround { .. } => "ResonanceChannelGround",
            GraphOperation::ResonanceTaunt { .. } => "ResonanceTaunt",
            GraphOperation::ResonanceUse { .. } => "ResonanceUse",
            GraphOperation::ResonanceStrike { .. } => "ResonanceStrike",
            GraphOperation::ResonanceGround { .. } => "ResonanceGround",
            GraphOperation::ResonanceGroundStatus { .. } => "ResonanceGroundStatus",
            GraphOperation::ResonanceScheduledArea { .. } => "ResonanceScheduledArea",
            GraphOperation::ResonanceDelayedArea { .. } => "ResonanceDelayedArea",
            GraphOperation::ResonanceState { .. } => "ResonanceState",
            GraphOperation::ResonanceParry { .. } => "ResonanceParry",
            GraphOperation::ResonanceSupport { .. } => "ResonanceSupport",
            GraphOperation::ResonancePreparedBasic { .. } => "ResonancePreparedBasic",
            GraphOperation::ResonanceReactiveHeal { .. } => "ResonanceReactiveHeal",
        };
        let path = path.replace("ResonanceUse", operation_name);
        source_map.insert(path, source_locations(&doc, root));
    }
    for id in doc.nodes.keys() {
        source_map.insert(format!("node.{id}"), source_locations(&doc, id));
    }
    let artifact = PortableGraph {
        format: "skill-graph-ir".into(),
        version: IR_VERSION,
        semantic_version: SEMANTIC_VERSION.into(),
        target: NATIVE_TARGET.into(),
        skill_id,
        operation,
        source_document: doc.document.clone(),
        source_digest: doc.digest.clone(),
        semantic_fingerprint: doc.semantic_fingerprint.clone(),
        source_map,
    };
    // Publication checks exactly the same loader used after RON transport.
    artifact
        .load_content()
        .map_err(|reason| vec![issue(&doc, None, None, "graph.native_contract", &reason)])?;
    Ok(artifact)
}
fn source_locations(doc: &NormalizedDocument, root: &str) -> Vec<Location> {
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    let mut locations = vec![];
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let node = &doc.nodes[id];
        locations.push(Location {
            node: Some(id.into()),
            ..Default::default()
        });
        locations.extend(node.values.keys().map(|field| Location {
            node: Some(id.into()),
            field: Some(field.clone()),
            ..Default::default()
        }));
        for (port, origins) in &node.inputs {
            for origin in origins {
                pending.push(&origin.source.node);
                locations.push(Location {
                    node: Some(id.into()),
                    port: Some(port.clone()),
                    ..Default::default()
                });
                locations.push(Location {
                    node: Some(origin.source.node.clone()),
                    port: Some(origin.source.port.clone()),
                    ..Default::default()
                });
                locations.extend(origin.path.iter().map(|edge| Location {
                    edge: Some(edge.clone()),
                    ..Default::default()
                }));
                locations.extend(origin.reroutes.iter().map(|id| Location {
                    node: Some(id.clone()),
                    ..Default::default()
                }));
            }
        }
    }
    locations.sort_by_key(|l| {
        (
            l.node.clone(),
            l.port.clone(),
            l.edge.clone(),
            l.field.clone(),
        )
    });
    locations.dedup();
    locations
}

struct Lower<'a> {
    doc: &'a NormalizedDocument,
    used: BTreeSet<&'a str>,
}
impl<'a> Lower<'a> {
    fn error(&self, id: &str, port: Option<&str>, reason: &str) -> Diagnostic {
        issue(self.doc, Some(id), port, "graph.lowering_contract", reason)
    }
    fn only(&mut self, kind: &str) -> Result<&'a str, Diagnostic> {
        let ids: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == kind)
            .map(|(id, _)| id.as_str())
            .collect();
        if ids.len() != 1 {
            return Err(issue(
                self.doc,
                ids.first().copied(),
                None,
                "graph.entry_ambiguous",
                &format!("Expected exactly one {kind}."),
            ));
        }
        self.used.insert(ids[0]);
        Ok(ids[0])
    }
    fn input(&mut self, id: &str, port: &str, kind: &str) -> Result<&'a str, Diagnostic> {
        let Some(origins) = self.doc.nodes[id].inputs.get(port).filter(|o| o.len() == 1) else {
            return Err(self.error(id, Some(port), "Exactly one producer is required."));
        };
        let producer = origins[0].source.node.as_str();
        if self.doc.nodes[producer].kind != kind {
            return Err(self.error(id, Some(port), "Producer is not supported for this input."));
        }
        self.used.insert(producer);
        Ok(producer)
    }
    fn link(&mut self, id: &str, port: &str, from: &str, output: &str) -> Result<(), Diagnostic> {
        let origins = self.doc.nodes[id].inputs.get(port);
        if !origins
            .is_some_and(|o| o.len() == 1 && o[0].source.node == from && o[0].source.port == output)
        {
            return Err(self.error(
                id,
                Some(port),
                "This input must originate from the same request's required output.",
            ));
        }
        Ok(())
    }
    fn number(&self, id: &str, field: &str) -> Result<u32, Diagnostic> {
        match self.doc.nodes[id].values.get(field) {
            Some(FieldValue::Number(n))
                if n.is_finite() && *n >= 0.0 && *n <= u32::MAX as f64 && n.fract() == 0.0 =>
            {
                Ok(*n as u32)
            }
            _ => Err(issue(
                self.doc,
                Some(id),
                None,
                "graph.numeric_target_range",
                "This native field requires an exact unsigned 32-bit integer.",
            )),
        }
    }
    fn amount(&mut self, id: &str, port: &str) -> Result<u32, Diagnostic> {
        let producer = self.input(id, port, "skill-resonance-amount")?;
        self.number(producer, "value")
    }
    fn mode(&self, id: &str) -> &str {
        match self.doc.nodes[id].values.get("mode") {
            Some(FieldValue::Text(s)) => s,
            _ => "",
        }
    }
    fn request_admission(&mut self, request: &str, qualify: &str) -> Result<ResonanceAdmission, Diagnostic> {
        let read = self.input(qualify, "snapshot", "skill-resonance-read")?;
        self.link(read, "request", request, "request")?;
        let settings = self.input(read, "settings", "skill-resonance-settings")?;
        let capacity = self.amount(settings, "capacity")?;
        let requirement = self.input(qualify, "requirement", "skill-resonance-requirement")?;
        let requirement = match self.mode(requirement) {
            "full" => ResonanceRequirement::Full,
            "payable" => ResonanceRequirement::Payable,
            "none" => ResonanceRequirement::None,
            "minimum" => ResonanceRequirement::Minimum(self.amount(requirement, "minimum")?),
            _ => return Err(self.error(requirement, None, "Unresolved requirement.")),
        };
        let cost_id = self.input(qualify, "cost", "skill-resonance-cost")?;
        self.link(cost_id, "snapshot", read, "snapshot")?;
        let cost = match self.mode(cost_id) {
            "all" => ResonanceCost::All,
            "zero" => ResonanceCost::Zero,
            "fixed" => ResonanceCost::Fixed(self.amount(cost_id, "amount")?),
            "max-percent" | "current-percent" => {
                let percent = self.number(cost_id, "percent")? as u8;
                let minimum = self.number(cost_id, "minimumCost")?;
                if self.mode(cost_id) == "max-percent" {
                    ResonanceCost::MaximumPercent { percent, minimum }
                } else {
                    ResonanceCost::CurrentPercent { percent, minimum }
                }
            }
            _ => {
                return Err(issue(
                    self.doc,
                    Some(cost_id),
                    None,
                    "graph.target_unsupported",
                    "Dynamic selected/count costs require their own native request binding.",
                ))
            }
        };
        let admission = ResonanceAdmission {
            capacity,
            requirement,
            cost,
        };
        admission
            .validate()
            .map_err(|r| self.error(qualify, None, r))?;
        Ok(admission)
    }
    fn resonance(&mut self) -> Result<GraphOperation, Diagnostic> {
        let request = self.only("skill-request")?;
        let qualify = self.only("skill-resonance-qualify")?;
        let admit = self.only("skill-resonance-admit")?;
        self.link(qualify, "request", request, "request")?;
        self.link(qualify, "in", request, "out")?;
        self.link(admit, "permit", qualify, "permit")?;
        self.link(admit, "in", qualify, "out")?;
        let admission = self.request_admission(request, qualify)?;
        if self
            .doc
            .nodes
            .values()
            .any(|n| n.kind == "skill-run-action")
        {
            self.target_state_action(request, qualify, admit, admission)
        } else if self.doc.nodes.values().any(|n| n.kind == "skill-channel-begin") {
            self.channel_ground_action(request, admit, admission)
        } else if self.doc.nodes.values().any(|n| n.kind == "skill-range-targets") {
            self.taunt_action(request, admit, admission)
        } else if self.doc.nodes.values().any(|n| n.kind == "skill-paired-areas") {
            self.paired_areas(request, admit, admission)
        } else if self.doc.nodes.values().any(|n| n.kind == "skill-ground-area") {
            self.ground_action(request, admit, admission)
        } else {
            self.self_selection(request)?;
            let end = self.only("skill-resonance-end")?;
            self.link(end, "use", admit, "use")?;
            let prev = self.doc.nodes[end]
                .inputs
                .get("in")
                .and_then(|o| o.first())
                .map(|o| o.source.node.as_str())
                .ok_or_else(|| {
                    self.error(end, Some("in"), "End requires the admitted success path.")
                })?;
            if prev != admit {
                let result = self.input(end, "in", "skill-resonance-result")?;
                self.link(end, "in", result, "out")?;
                self.link(result, "receipt", admit, "receipt")?;
                self.link(result, "in", admit, "out")?;
            } else {
                self.link(end, "in", admit, "out")?;
            }
            Ok(GraphOperation::ResonanceUse { admission })
        }
    }
}
