use super::*;
use crate::game::{
    ability::{ground::*, SkillHitTargetFilter},
    battle::tile_range::TileRangePattern,
};
impl<'a> Lower<'a> {
    pub(super) fn ground_status_action(
        &mut self,
        area: &'a str,
        admission: ResonanceAdmission,
        selection_range: TileRangePattern,
        footprint: TileRangePattern,
        duration_ms: u32,
    ) -> Result<GraphOperation, Diagnostic> {
        let (program, _) =
            self.ground_status_plan(area, selection_range, footprint, duration_ms)?;
        Ok(GraphOperation::ResonanceGroundStatus { admission, program })
    }
    pub(super) fn ground_status_plan(
        &mut self,
        area: &'a str,
        selection_range: TileRangePattern,
        footprint: TileRangePattern,
        duration_ms: u32,
    ) -> Result<(GroundStatusAreaProgram, BTreeMap<&'a str, Vec<usize>>), Diagnostic> {
        let filter = self.only("skill-ground-filter")?;
        let hit_targets = match self.text(filter, "side") {
            "enemies" => SkillHitTargetFilter::Enemies,
            "allies" => SkillHitTargetFilter::Allies,
            "all" => SkillHitTargetFilter::Any,
            _ => return Err(self.error(filter, Some("side"), "Choose ground recipients.")),
        };
        let include_caster = match self.text(filter, "caster") {
            "include" => true,
            "exclude" => false,
            _ => return Err(self.error(filter, Some("caster"), "Choose caster membership.")),
        };
        let subscriptions: Vec<_> = self
            .doc
            .nodes
            .iter()
            .filter(|(_, n)| {
                matches!(
                    n.kind.as_str(),
                    "skill-ground-enter"
                        | "skill-ground-ramping-slow"
                        | "skill-ground-dwell"
                        | "skill-ground-exit"
                        | "skill-ground-ended"
                )
            })
            .map(|(id, _)| id.as_str())
            .collect();
        let mut statuses = vec![];
        let mut status_indices = BTreeMap::new();
        let mut rules = vec![];
        let mut event_rules: BTreeMap<&'a str, Vec<usize>> = BTreeMap::new();
        for event in subscriptions {
            self.used.insert(event);
            self.link(event, "area", area, "area")?;
            let trigger = match self.doc.nodes[event].kind.as_str() {
                "skill-ground-enter" => Some(GroundTrigger::Enter),
                "skill-ground-dwell" | "skill-ground-ramping-slow" => Some(GroundTrigger::Dwell {
                    duration_ms: self.positive_duration(event, "duration")?,
                }),
                "skill-ground-exit" => Some(GroundTrigger::Exit),
                _ => None,
            };
            if trigger.is_some() {
                self.link(event, "filter", filter, "filter")?;
            }
            let cues: Vec<_> = self
                .doc
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.kind == "skill-cue"
                        && n.inputs
                            .get("event")
                            .is_some_and(|origins| origins.iter().any(|o| o.source.node == event))
                })
                .map(|(id, _)| id.as_str())
                .collect();
            for cue in cues {
                self.used.insert(cue);
                self.link(cue, "event", event, "cue")?;
            }
            let Some(trigger) = trigger else {
                continue;
            };
            if self.doc.nodes[event].kind == "skill-ground-ramping-slow" {
                let status = statuses.len();
                statuses.push(GroundStatusDef {
                    definition: event.into(),
                    kind: GroundStatusKind::RampingSlow {
                        duration_ms: self.positive_duration(event, "duration")?,
                        max_percent: self.number(event, "maxSlow")?,
                        rate_numerator: 1,
                        rate_denominator: 1,
                    },
                    overlap: GroundOverlap::Strongest,
                    lifetime: GroundStatusLifetime::UntilExit,
                    vulnerability: None,
                });
                status_indices.insert(event, status);
                rules.push(GroundStatusRule {
                    trigger: GroundTrigger::Enter,
                    status,
                    action: GroundStatusAction::Apply,
                });
            }
            let mut pending = vec![event];
            while let Some(parent) = pending.pop() {
                let actions: Vec<_> =
                    self.doc
                        .nodes
                        .iter()
                        .filter(|(_, n)| {
                            matches!(
                                n.kind.as_str(),
                                "skill-ground-apply-status" | "skill-ground-clear-status"
                            ) && n.inputs.get("in").is_some_and(|origins| {
                                origins.iter().any(|o| o.source.node == parent)
                            })
                        })
                        .map(|(id, _)| id.as_str())
                        .collect();
                for action in actions {
                    if !self.used.insert(action) {
                        return Err(self.error(
                            action,
                            None,
                            "Ground status action has multiple event owners.",
                        ));
                    }
                    self.link(action, "in", parent, "out")?;
                    self.link(action, "member", event, "member")?;
                    let definition = self.input(action, "status", "skill-ground-status")?;
                    let status = if let Some(index) = status_indices.get(definition) {
                        *index
                    } else {
                        let kind = match self.text(definition, "effect") {
                            "slow" => GroundStatusKind::Slow {
                                percent: self.number(definition, "percent")?,
                            },
                            "damage-taken" => GroundStatusKind::DamageTaken {
                                percent: self.number(definition, "percent")?,
                            },
                            "stun" => GroundStatusKind::Stun,
                            _ => {
                                return Err(self.error(
                                    definition,
                                    Some("effect"),
                                    "Choose the ground status kind.",
                                ))
                            }
                        };
                        let overlap = if kind == GroundStatusKind::Stun {
                            GroundOverlap::Strongest
                        } else {
                            match self.text(definition, "overlap") {
                                "sum" => GroundOverlap::Sum,
                                "strongest" => GroundOverlap::Strongest,
                                _ => {
                                    return Err(self.error(
                                        definition,
                                        Some("overlap"),
                                        "Choose ground contribution overlap.",
                                    ))
                                }
                            }
                        };
                        let lifetime = match self.text(definition, "lifetime") {
                            "membership" => GroundStatusLifetime::UntilExit,
                            "area" => GroundStatusLifetime::UntilAreaEnd,
                            _ => {
                                return Err(self.error(
                                    definition,
                                    Some("lifetime"),
                                    "Choose ground contribution lifetime.",
                                ))
                            }
                        };
                        let index = statuses.len();
                        statuses.push(GroundStatusDef {
                            vulnerability: None,
                            definition: definition.into(),
                            kind,
                            overlap,
                            lifetime,
                        });
                        status_indices.insert(definition, index);
                        index
                    };
                    let action_kind = if self.doc.nodes[action].kind == "skill-ground-apply-status"
                    {
                        GroundStatusAction::Apply
                    } else {
                        GroundStatusAction::Clear
                    };
                    if trigger == GroundTrigger::Exit && action_kind == GroundStatusAction::Apply {
                        return Err(self.error(
                            action,
                            None,
                            "Cannot apply status to an exited relation.",
                        ));
                    }
                    event_rules.entry(event).or_default().push(rules.len());
                    rules.push(GroundStatusRule {
                        trigger,
                        status,
                        action: action_kind,
                    });
                    pending.push(action);
                }
            }
        }
        let program = GroundStatusAreaProgram {
            selection_range,
            footprint,
            duration_ms,
            hit_targets,
            include_caster,
            status: GroundStatusProgram { statuses, rules },
        };
        program.validate().map_err(|e| self.error(area, None, &e))?;
        Ok((program, event_rules))
    }
}
