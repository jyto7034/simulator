//! Exact occurrence results for structural children. Parent legacy cumulative
//! results stay separate; a child's prefix is seeded only from Action producers
//! in its compiled scope, never from the parent's latest cumulative frontier.
use super::*;
use crate::game::ability::authoring::ActionSite;
use crate::game::battle::core::value_expr_runtime::{ActionResultWitness, EffectResultProjection};

#[derive(Debug, Clone)]
pub(super) struct ScopedResultWitness {
    own: ActionResultWitness,
    prefix: Option<ActionResultWitness>,
}
crate::authoring::clone_shape::struct_shape!(ScopedResultWitness { own, prefix });
impl ScopedResultWitness {
    fn new(
        id: &crate::game::ability::SkillId,
        ids: Arc<Vec<crate::game::resource::ResourceId>>,
        prefix: bool,
    ) -> Result<Self, GameError> {
        let bytes = ids
            .len()
            .checked_mul(std::mem::size_of::<i128>())
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<ActionResultWitness>() + id.as_str().len() + 128,
                )
            })
            .and_then(|bytes| bytes.checked_mul(if prefix { 2 } else { 1 }))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>() + 64));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            bytes,
            "flow_occurrence_result_owner",
        )?;
        control::charge(
            ids.len() as u64 * if prefix { 2 } else { 1 },
            "flow_occurrence_result_initialize",
        )?;
        Ok(Self {
            own: ActionResultWitness::new(id.clone(), ids.clone()),
            prefix: prefix.then(|| ActionResultWitness::new(id.clone(), ids)),
        })
    }
    fn extent(&self) -> Option<usize> {
        self.own
            .retained_extent()?
            .checked_add(
                self.prefix
                    .as_ref()
                    .map_or(Some(0), |value| value.retained_extent())?,
            )?
            .checked_add(std::mem::size_of::<Self>() + 64)
    }
}
impl FlowState {
    pub(super) fn seed_action_scope(
        &mut self,
        node: usize,
        occurrence: usize,
        core: &BattleCore,
        cast: u64,
    ) -> Result<(), GameError> {
        // Original Action prefixes retain their historic ActionHandle owner.
        if self.root == self.plan.root || self.scope_results.is_none() {
            return Ok(());
        }
        let owner = core
            .active_skill_casts
            .get(&cast)
            .ok_or(GameError::InvalidAction)?;
        let Some(action) = owner.fact_source.action.as_ref() else {
            return Ok(());
        };
        let ids = if let Some(global) = core
            .runtime_facts
            .action_result_witness(action)
            .map_err(|_| GameError::InvalidAction)?
        {
            global.resource_ids()
        } else {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<Vec<crate::game::resource::ResourceId>>() + 64),
                "flow_empty_resource_scope",
            )?;
            Arc::new(Vec::new())
        };
        let mut next = ScopedResultWitness::new(&owner.skill_id, ids, true)?;
        let results = self
            .scope_results
            .as_ref()
            .expect("structural result owner");
        control::charge(
            self.plan.nodes[node].result_scope.len() as u64,
            "flow_action_scope_seed",
        )?;
        for &producer in &self.plan.nodes[node].result_scope {
            let Some(key) = self.nodes[producer].occurrence else {
                continue;
            };
            if let Some(prior) = results.get(&key) {
                next.prefix
                    .as_mut()
                    .expect("child prefix")
                    .merge_completed(&prior.own)?;
            }
        }
        core.admit_authoring_runtime_growth(1, "flow_occurrence_result")?;
        self.scope_results
            .as_mut()
            .expect("structural result owner")
            .insert(occurrence, next);
        Ok(())
    }
    pub(super) fn copy_scope_result(
        &self,
        occurrence: usize,
        child: &mut FlowState,
    ) -> Result<(), GameError> {
        let Some(prior) = self
            .scope_results
            .as_ref()
            .and_then(|rows| rows.get(&occurrence))
        else {
            return Ok(());
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            prior.own.retained_extent().and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<ScopedResultWitness>() + 64)
            }),
            "flow_prior_result_snapshot",
        )?;
        control::charge(
            prior.own.resource_ids().len() as u64 + 1,
            "flow_prior_result_copy",
        )?;
        child
            .scope_results
            .as_mut()
            .expect("child result scope")
            .insert(
                occurrence,
                ScopedResultWitness {
                    own: prior.own.clone(),
                    prefix: None,
                },
            );
        Ok(())
    }
    pub(super) fn copy_anchor_result(
        results: &Option<Box<BTreeMap<usize, ScopedResultWitness>>>,
        occurrence: usize,
        child: &mut FlowState,
    ) -> Result<(), GameError> {
        let Some(prior) = results.as_ref().and_then(|rows| rows.get(&occurrence)) else {
            return Ok(());
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            prior.own.retained_extent().and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<ScopedResultWitness>() + 64)
            }),
            "flow_anchor_result_snapshot",
        )?;
        control::charge(
            prior.own.resource_ids().len() as u64 + 1,
            "flow_anchor_result_copy",
        )?;
        child
            .scope_results
            .as_mut()
            .expect("child result scope")
            .insert(
                occurrence,
                ScopedResultWitness {
                    own: prior.own.clone(),
                    prefix: None,
                },
            );
        Ok(())
    }
    pub(super) fn owns_child_occurrence(&self, occurrence: usize, first: usize) -> bool {
        self.scope_results
            .as_ref()
            .and_then(|rows| rows.get(&occurrence))
            .is_some_and(|row| row.prefix.is_some())
            || self
                .nodes
                .iter()
                .enumerate()
                .any(|(index, state)| index >= first && state.occurrence == Some(occurrence))
    }
    pub(super) fn pending_child_delivery(&self, core: &BattleCore, cast: u64) -> bool {
        self.scope_results.as_ref().is_some_and(|rows| {
            rows.iter().any(|(occurrence, row)| {
                row.prefix.is_some()
                    && core
                        .active_skill_casts
                        .get(&cast)
                        .and_then(|owner| owner.step_progress.get(occurrence))
                        .is_some_and(|progress| !progress.is_terminal())
            })
        })
    }
    pub(super) fn scope_prefix(&self, occurrence: usize) -> Option<&ActionResultWitness> {
        if let Some(value) = self
            .scope_results
            .as_ref()
            .and_then(|rows| rows.get(&occurrence))
            .and_then(|value| value.prefix.as_ref())
        {
            return Some(value);
        }
        if let Some(sides) = &self.side_occurrences {
            for side in sides.iter() {
                if let Some(value) = side.flow.scope_prefix(occurrence) {
                    return Some(value);
                }
            }
        }
        None
    }
    pub(super) fn settle_scope(
        &mut self,
        occurrence: usize,
        site: &ActionSite,
        projection: &EffectResultProjection,
        ids: Arc<Vec<crate::game::resource::ResourceId>>,
    ) -> Result<bool, GameError> {
        if let Some(sides) = &mut self.side_occurrences {
            let first = self
                .plan
                .side_branches
                .iter()
                .map(|branch| branch.root)
                .min()
                .expect("side root");
            for side in sides.iter_mut() {
                control::charge(
                    side.flow.nodes.len() as u64,
                    "flow_result_occurrence_lookup",
                )?;
                if side.flow.owns_child_occurrence(occurrence, first) {
                    // A Repeat may already have reset its node after Released.
                    // The occurrence owner lives until its own delivery settles.
                    side.flow.settle_scope(occurrence, site, projection, ids)?;
                    return Ok(true);
                }
            }
        }
        let Some(results) = &mut self.scope_results else {
            return Ok(false);
        };
        if !results.contains_key(&occurrence) {
            let value = ScopedResultWitness::new(&site.definition_id, ids, false)?;
            results.insert(occurrence, value);
        }
        let row = results
            .get_mut(&occurrence)
            .expect("admitted occurrence result");
        row.own.settle(site, projection)?;
        if let Some(prefix) = &mut row.prefix {
            prefix.settle(site, projection)?;
        }
        Ok(self.root != self.plan.root)
    }
    pub(super) fn scope_extent(&self) -> Option<usize> {
        let Some(results) = &self.scope_results else {
            return Some(0);
        };
        control::charge(results.len() as u64, "flow_result_scope_extent").ok()?;
        results
            .values()
            .try_fold(std::mem::size_of_val(results.as_ref()), |bytes, row| {
                bytes.checked_add(row.extent()?)
            })
    }
    pub(super) fn prune_result_scopes(
        &mut self,
        core: &BattleCore,
        cast: u64,
    ) -> Result<(), GameError> {
        let Some(results) = &mut self.scope_results else {
            return Ok(());
        };
        let work = results
            .len()
            .checked_mul(
                self.nodes.len()
                    + self
                        .side_occurrences
                        .as_ref()
                        .map_or(0, |sides| sides.len())
                    + 1,
            )
            .ok_or(GameError::InvalidAction)?;
        control::charge(work as u64, "flow_result_scope_prune")?;
        results.retain(|key, _| {
            self.nodes.iter().any(|node| node.occurrence == Some(*key))
                || self
                    .side_occurrences
                    .as_ref()
                    .is_some_and(|sides| sides.iter().any(|side| side.anchor_occurrence == *key))
                || core
                    .active_skill_casts
                    .get(&cast)
                    .and_then(|cast| cast.step_progress.get(key))
                    .is_some_and(|progress| !progress.is_terminal())
        });
        Ok(())
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn structural_result_witness(
        &self,
        source: &RuntimeFactSource,
    ) -> Option<&ActionResultWitness> {
        let cast = source.cast.as_ref()?;
        let occurrence = source.flow.as_ref()?.occurrence;
        self.active_skill_casts
            .get(&cast.cast_seq)?
            .flow
            .as_ref()?
            .scope_prefix(occurrence)
    }
    pub(in crate::game::battle::core) fn settle_skill_effect_result(
        &mut self,
        source: &RuntimeFactSource,
        site: &ActionSite,
        projection: &EffectResultProjection,
        ids: Arc<Vec<crate::game::resource::ResourceId>>,
    ) -> Result<(), GameError> {
        let action = source.action.as_ref().ok_or(GameError::InvalidAction)?;
        self.runtime_facts
            .validate_action_source(action, source)
            .map_err(|_| GameError::InvalidAction)?;
        if let Some(delivery) = &source.delivery {
            self.runtime_facts
                .validate_live_delivery(delivery)
                .map_err(|_| GameError::InvalidAction)?;
        }
        let released_child = source.relic_effective().is_some_and(|binding| {
            source.flow.as_ref().is_some_and(|flow| {
                binding
                    .execution()
                    .flow
                    .side_branches
                    .iter()
                    .any(|branch| flow.node_index >= branch.root)
            })
        });
        let scoped_child = if let (Some(cast), Some(occurrence)) = (&source.cast, &source.flow) {
            self.active_skill_casts
                .get_mut(&cast.cast_seq)
                .and_then(|cast| cast.flow.as_mut())
                .map(|flow| flow.settle_scope(occurrence.occurrence, site, projection, ids))
                .transpose()?
                .unwrap_or(false)
        } else {
            false
        };
        // Cancelled/finished branch owners cannot make late child deliveries
        // settle into the historic parent ActionHandle frontier.
        if !scoped_child && !released_child {
            self.runtime_facts
                .settle_action_result(source, site, projection)?;
        }
        Ok(())
    }
}
