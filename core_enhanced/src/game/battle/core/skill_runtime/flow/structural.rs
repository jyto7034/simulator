//! Finite grafts are cold children of an original occurrence, never parent use
//! control. A branch owns a preleased chain hold; each child Action still opens
//! its own ordinary Delivery resource scope.
use super::*;

#[derive(Debug, Clone)]
pub(super) struct SideOccurrence {
    anchor: usize,
    pub(super) anchor_occurrence: usize,
    pub(super) flow: Box<FlowState>,
    source: RuntimeFactSource,
    started: bool,
    // Cancellation closes the branch reservation, but an already released
    // child still owns its exact result frontier until its delivery settles.
    settlement_only: bool,
}
crate::authoring::clone_shape::struct_shape!(SideOccurrence {
    anchor,
    anchor_occurrence,
    flow,
    source,
    started,
    settlement_only
});

impl FlowState {
    pub(in crate::game::battle::core) fn has_pending_sides(&self) -> bool {
        self.side_occurrences
            .as_ref()
            .is_some_and(|sides| !sides.is_empty())
    }
    pub(super) fn prepare_sides(
        &mut self,
        anchor: usize,
        occurrence: usize,
        core: &mut BattleCore,
        cast: u64,
    ) -> Result<(), GameError> {
        if self.side_occurrences.is_none() {
            return Ok(());
        }
        control::charge(
            self.plan.side_branches.len() as u64,
            "flow_side_anchor_selection",
        )?;
        let plan = self.plan.clone();
        for branch in &plan.side_branches {
            if branch.anchor != anchor {
                continue;
            }
            let mut source = core
                .active_skill_casts
                .get(&cast)
                .ok_or(GameError::InvalidAction)?
                .fact_source
                .clone();
            source.admit_owner("flow_side_source")?;
            core.ensure_value_expr_action_witness(
                &core
                    .active_skill_casts
                    .get(&cast)
                    .ok_or(GameError::InvalidAction)?
                    .skill_id
                    .clone(),
                &source,
            )?;
            source.use_control = false;
            // This actual lease is captured while the original occurrence is
            // authorized. Retaining a plan by itself never mints authority.
            source = core.reserve_finite_source(source)?;
            let mut child = Self::new_at(plan.clone(), self.target, branch.root, false)?;
            control::charge(
                plan.nodes[anchor].result_scope.len() as u64 + 1,
                "flow_side_prior_result_copy",
            )?;
            for &index in &plan.nodes[anchor].result_scope {
                child.nodes[index] = self.nodes[index].clone();
                if let Some(occurrence) = self.nodes[index].occurrence {
                    self.copy_scope_result(occurrence, &mut child)?;
                }
            }
            child.nodes[anchor] = self.nodes[anchor].clone();
            // Prior results are snapshotted now, before a parent Repeat can
            // reset/reuse them. The anchor itself settles independently later.
            if let Some(results) = &self.effect_results {
                control::charge(
                    (results.len() as u64)
                        .checked_mul(plan.nodes[anchor].result_scope.len() as u64 + 1)
                        .ok_or(GameError::InvalidAction)?,
                    "flow_side_prior_effect_copy",
                )?;
                for ((key, site), result) in results.iter() {
                    if plan.nodes[anchor]
                        .result_scope
                        .iter()
                        .any(|index| self.nodes[*index].occurrence == Some(*key))
                    {
                        child.record_effect_result(*key, site, result)?;
                    }
                }
            }
            let sides = self.side_occurrences.as_mut().expect("original side owner");
            core.admit_authoring_runtime_growth(1, "flow_side_occurrence")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<SideOccurrence>() + std::mem::size_of::<FlowState>()),
                "flow_side_occurrence_storage",
            )?;
            if sides.len() == sides.capacity() {
                sides.reserve_exact(1);
            }
            sides.push(SideOccurrence {
                anchor,
                anchor_occurrence: occurrence,
                flow: Box::new(child),
                source,
                started: false,
                settlement_only: false,
            });
        }
        Ok(())
    }
    pub(super) fn start_settled_sides(
        &mut self,
        core: &BattleCore,
        cast: u64,
    ) -> Result<(), GameError> {
        let Some(sides) = &mut self.side_occurrences else {
            return Ok(());
        };
        control::charge(sides.len() as u64, "flow_side_settlement_scan")?;
        for side in sides.iter_mut() {
            if side.started || side.settlement_only {
                continue;
            }
            let Some(progress) = core
                .active_skill_casts
                .get(&cast)
                .and_then(|cast| cast.step_progress.get(&side.anchor_occurrence))
                .filter(|progress| progress.is_terminal())
            else {
                continue;
            };
            let state = &mut side.flow.nodes[side.anchor];
            state.phase = 2;
            state.dispatched = true;
            state.occurrence = Some(side.anchor_occurrence);
            state.result = progress.result.clone();
            if let Some(results) = &self.effect_results {
                control::charge(results.len() as u64, "flow_side_anchor_effect_copy")?;
                for ((occurrence, site), result) in results.iter() {
                    if *occurrence == side.anchor_occurrence {
                        side.flow.record_effect_result(*occurrence, site, result)?;
                    }
                }
            }
            Self::copy_anchor_result(&self.scope_results, side.anchor_occurrence, &mut side.flow)?;
            side.started = true;
        }
        Ok(())
    }
    pub(super) fn drive_sides(
        &mut self,
        core: &mut BattleCore,
        cast: u64,
        skill: &SkillDef,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(mut sides) = self.side_occurrences.take() else {
            return Ok(());
        };
        let operation = (|| {
            control::charge(sides.len() as u64, "flow_side_drive")?;
            let mut index = 0;
            while index < sides.len() {
                let side = &mut sides[index];
                if !side.started {
                    index += 1;
                    continue;
                }
                side.flow.next_occurrence = self.next_occurrence;
                side.flow.wake = None;
                if !side.flow.complete {
                    side.flow.complete = side.flow.drive(side.flow.root, core, cast, skill, now)?;
                }
                self.next_occurrence = side.flow.next_occurrence;
                side.flow.prune_result_scopes(core, cast)?;
                if let Some(wake) = side.flow.wake {
                    self.wake = Some(self.wake.map_or(wake, |old| old.min(wake)));
                }
                let first_generated = self
                    .plan
                    .side_branches
                    .iter()
                    .map(|b| b.root)
                    .min()
                    .expect("side root");
                control::charge(
                    side.flow.nodes.len() as u64,
                    "flow_side_delivery_settlement",
                )?;
                let released_pending = if side.settlement_only {
                    side.flow
                        .released_child_pending(core, cast, first_generated)?
                } else {
                    side.flow.pending_child_delivery(core, cast)
                        || side
                            .flow
                            .nodes
                            .iter()
                            .enumerate()
                            .filter(|(node, _)| *node >= first_generated)
                            .filter_map(|(_, node)| node.occurrence)
                            .any(|occurrence| {
                                core.active_skill_casts
                                    .get(&cast)
                                    .and_then(|cast| cast.step_progress.get(&occurrence))
                                    .is_none_or(|progress| !progress.is_terminal())
                            })
                };
                if side.flow.complete && !released_pending {
                    if !side.settlement_only {
                        Self::close_side(core, &side.source, now)?;
                    }
                    let anchor_occurrence = side.anchor_occurrence;
                    sides.remove(index);
                    if !sides
                        .iter()
                        .any(|side| side.anchor_occurrence == anchor_occurrence)
                        && !self
                            .nodes
                            .iter()
                            .any(|node| node.occurrence == Some(anchor_occurrence))
                    {
                        if let Some(results) = &mut self.effect_results {
                            control::charge(results.len() as u64, "flow_side_scope_release")?;
                            results.retain(|(occurrence, _), _| *occurrence != anchor_occurrence);
                        }
                    }
                } else {
                    index += 1;
                }
            }
            Ok(())
        })();
        self.side_occurrences = Some(sides);
        operation
    }
    fn close_side(
        core: &mut BattleCore,
        source: &RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        core.close_source_resource_delivery(source.resource_delivery().map(Arc::as_ref))?;
        if let Some(ticket) = &source.delivery {
            core.runtime_facts.close_delivery(ticket, now)?;
        }
        Ok(())
    }
    pub(super) fn cancel_sides(
        &mut self,
        core: &mut BattleCore,
        cast: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(mut sides) = self.side_occurrences.take() else {
            return Ok(());
        };
        let operation = (|| {
            control::charge(sides.len() as u64, "flow_side_cancel")?;
            crate::game::battle::runtime_control::charge(sides.len() as u64, "flow_side_cancel")?;
            control::charge(
                self.plan.side_branches.len() as u64,
                "flow_cancel_child_roots",
            )?;
            crate::game::battle::runtime_control::charge(
                self.plan.side_branches.len() as u64,
                "flow_cancel_child_roots",
            )?;
            let first_generated = self
                .plan
                .side_branches
                .iter()
                .map(|branch| branch.root)
                .min()
                .unwrap_or(usize::MAX);
            let mut closed = Ok(());
            sides.retain_mut(|side| {
                if closed.is_err() {
                    return true;
                }
                let retained = (|| {
                    if !side.settlement_only {
                        Self::close_side(core, &side.source, now)?;
                        side.settlement_only = true;
                        // The child interpreter and cancelled reservation cannot
                        // authorize any further Action or retain a closed ticket.
                        side.flow.complete = true;
                        side.flow.wake = None;
                        side.source = RuntimeFactSource::default();
                    }
                    side.flow
                        .released_child_pending(core, cast, first_generated)
                })();
                match retained {
                    Ok(pending) => pending,
                    Err(error) => {
                        closed = Err(error);
                        true
                    }
                }
            });
            closed
        })();
        // Keep the temporary owner in the failed candidate as well; the
        // surrounding transaction decides whether any change can commit.
        self.side_occurrences = Some(sides);
        operation
    }
    fn released_child_pending(
        &self,
        core: &BattleCore,
        cast: u64,
        first_generated: usize,
    ) -> Result<bool, GameError> {
        let scopes = self.scope_results.as_ref().map_or(0, |rows| rows.len());
        for count in [scopes, self.nodes.len()] {
            control::charge(count as u64, "flow_cancelled_child_settlement")?;
            crate::game::battle::runtime_control::charge(
                count as u64,
                "flow_cancelled_child_settlement",
            )?;
        }
        let released = |occurrence| {
            core.active_skill_casts
                .get(&cast)
                .and_then(|cast| cast.step_progress.get(&occurrence))
                .is_some_and(|progress| progress.started && !progress.is_terminal())
        };
        // A Repeat may have reset the node of an earlier released occurrence.
        // Its prefix row remains the bounded owner of that exact occurrence.
        Ok(self.scope_results.as_ref().is_some_and(|rows| {
            rows.keys()
                .any(|occurrence| released(*occurrence) && self.scope_prefix(*occurrence).is_some())
        }) || self
            .nodes
            .iter()
            .enumerate()
            .filter(|(node, _)| *node >= first_generated)
            .filter_map(|(_, node)| node.occurrence)
            .any(released))
    }
    pub(super) fn occurrence_node(&self, occurrence: usize) -> Option<usize> {
        control::charge(self.nodes.len() as u64, "flow_source_lookup").ok()?;
        if let Some(index) = self
            .nodes
            .iter()
            .position(|state| state.occurrence == Some(occurrence))
        {
            return Some(index);
        }
        if let Some(sides) = &self.side_occurrences {
            for side in sides.iter() {
                if let Some(index) = side.flow.occurrence_node(occurrence) {
                    return Some(index);
                }
            }
        }
        None
    }
    pub(super) fn mark_dispatched(&mut self, occurrence: usize) -> Result<bool, GameError> {
        control::charge(self.nodes.len() as u64, "flow_dispatch_owner")?;
        if let Some(state) = self
            .nodes
            .iter_mut()
            .find(|state| state.occurrence == Some(occurrence))
        {
            state.dispatched = true;
            return Ok(true);
        }
        if let Some(sides) = &mut self.side_occurrences {
            for side in sides.iter_mut() {
                if side.flow.mark_dispatched(occurrence)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    pub(super) fn source_for_occurrence(
        &self,
        occurrence: usize,
    ) -> Result<Option<RuntimeFactSource>, GameError> {
        let Some(sides) = &self.side_occurrences else {
            return Ok(None);
        };
        let first_generated = self
            .plan
            .side_branches
            .iter()
            .map(|b| b.root)
            .min()
            .expect("side root");
        for side in sides.iter() {
            control::charge(side.flow.nodes.len() as u64, "flow_side_source_lookup")?;
            if side.flow.owns_child_occurrence(occurrence, first_generated) {
                if side.settlement_only {
                    return Err(GameError::InvalidAction);
                }
                side.source.admit_owner("flow_side_dispatch_source")?;
                return Ok(Some(side.source.clone()));
            }
        }
        Ok(None)
    }
    pub(super) fn side_extent(&self) -> Option<usize> {
        let Some(sides) = &self.side_occurrences else {
            return Some(0);
        };
        control::charge(sides.len() as u64, "flow_side_extent").ok()?;
        let mut bytes = std::mem::size_of::<Vec<SideOccurrence>>().checked_add(
            sides
                .capacity()
                .checked_mul(std::mem::size_of::<SideOccurrence>())?,
        )?;
        for side in sides.iter() {
            bytes = bytes
                .checked_add(std::mem::size_of::<FlowState>())?
                .checked_add(
                    side.flow
                        .nodes
                        .capacity()
                        .checked_mul(std::mem::size_of::<NodeState>())?,
                )?
                .checked_add(side.source.retained_extent()?)?
                .checked_add(side.flow.effect_scope_extent()?)?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn skill_flow_source(
        &self,
        cast: u64,
        occurrence: usize,
    ) -> Result<Option<RuntimeFactSource>, GameError> {
        let Some(cast) = self.active_skill_casts.get(&cast) else {
            return Ok(None);
        };
        if let Some(flow) = &cast.flow {
            if let Some(source) = flow.source_for_occurrence(occurrence)? {
                return Ok(Some(source));
            }
        }
        cast.fact_source
            .admit_owner("flow_original_dispatch_source")?;
        Ok(Some(cast.fact_source.clone()))
    }
    pub(in crate::game::battle::core) fn is_side_occurrence(
        &self,
        cast: u64,
        occurrence: usize,
    ) -> bool {
        let Some(flow) = self
            .active_skill_casts
            .get(&cast)
            .and_then(|cast| cast.flow.as_ref())
        else {
            return false;
        };
        let Some(sides) = &flow.side_occurrences else {
            return false;
        };
        let first_generated = flow
            .plan
            .side_branches
            .iter()
            .map(|b| b.root)
            .min()
            .expect("side root");
        for side in sides.iter() {
            if control::charge(side.flow.nodes.len() as u64, "flow_side_occurrence_lookup").is_err()
            {
                return false;
            }
            if side.flow.owns_child_occurrence(occurrence, first_generated) {
                return true;
            }
        }
        false
    }
}
