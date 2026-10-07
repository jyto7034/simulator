use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::game::battle::{
    ids::UnitInstanceId,
    runtime_control::{
        self as control, BattleRuntimeFault, RuntimeFactLimits, RuntimeFaultCode as Code,
    },
    runtime_facts::*,
};
use crate::game::{ability::SkillId, behavior::GameError};

use super::BattleCore;

#[derive(Debug, Clone)]
struct ActiveAction {
    handle: ActionHandle,
    snapshot: ActionSnapshot,
    // Established by the action owner when the accepted skill starts its cast.
    // This remains with the action while its released deliveries are live.
    bound_cast: Option<Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    action_result: Option<Box<super::value_expr_runtime::ActionResultWitness>>,
    pending_steps: BTreeSet<usize>,
    deliveries: BTreeSet<DeliveryOrdinal>,
    public_reserved: bool,
    extent: usize,
}
crate::authoring::clone_shape::struct_shape!(ActiveAction {
    handle,
    snapshot,
    bound_cast,
    action_result,
    pending_steps,
    deliveries,
    public_reserved,
    extent
});

#[derive(Debug, Clone)]
struct RetainedAction {
    handle: ActionHandle,
    receipt: Arc<SettledActionReceipt>,
    extent: usize,
}
crate::authoring::clone_shape::struct_shape!(RetainedAction {
    handle,
    receipt,
    extent
});

/// Ordered actor rows retain bounded spare slots between insertions.
#[derive(Debug, Clone, Default)]
pub(super) struct ActorSummaries {
    rows: Vec<(UnitInstanceId, ActorFactSummary)>,
}
crate::authoring::clone_shape::struct_shape!(ActorSummaries { rows });
impl ActorSummaries {
    fn len(&self) -> usize {
        self.rows.len()
    }
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    fn position(&self, key: &UnitInstanceId) -> Result<usize, usize> {
        self.rows.binary_search_by_key(key, |(id, _)| *id)
    }
    fn contains_key(&self, key: &UnitInstanceId) -> bool {
        self.position(key).is_ok()
    }
    fn get(&self, key: &UnitInstanceId) -> Option<&ActorFactSummary> {
        self.position(key).ok().map(|index| &self.rows[index].1)
    }
    fn get_mut(&mut self, key: &UnitInstanceId) -> Option<&mut ActorFactSummary> {
        self.position(key).ok().map(|index| &mut self.rows[index].1)
    }
    fn iter(&self) -> impl Iterator<Item = (&UnitInstanceId, &ActorFactSummary)> {
        self.rows.iter().map(|(id, value)| (id, value))
    }
    fn values(&self) -> impl Iterator<Item = &ActorFactSummary> {
        self.rows.iter().map(|(_, value)| value)
    }
    #[cfg(test)]
    fn clear(&mut self) {
        self.rows = Vec::new();
    }
    fn insert(
        &mut self,
        key: UnitInstanceId,
        value: ActorFactSummary,
        max_actors: usize,
    ) -> Result<(), GameError> {
        match self.position(&key) {
            Ok(index) => self.rows[index].1 = value,
            Err(index) => {
                crate::authoring::control::charge(
                    (self.rows.len() - index) as u64,
                    "actor_row_shift",
                )?;
                if self.rows.len() == self.rows.capacity() {
                    let capacity = control::admitted_buffer_capacity(
                        self.rows.len(),
                        self.rows.capacity(),
                        max_actors,
                        std::mem::size_of::<(UnitInstanceId, ActorFactSummary)>(),
                        64,
                        "actor_row_reserve",
                    )?;
                    self.rows.reserve_exact(capacity - self.rows.len());
                }
                self.rows.insert(index, (key, value));
            }
        }
        Ok(())
    }
}
impl std::ops::Index<&UnitInstanceId> for ActorSummaries {
    type Output = ActorFactSummary;
    fn index(&self, key: &UnitInstanceId) -> &Self::Output {
        self.get(key).expect("admitted actor")
    }
}

fn writable_actor_summaries(
    map: &mut Arc<ActorSummaries>,
) -> Result<&mut ActorSummaries, GameError> {
    use crate::authoring::{
        clone_shape::{controlled_clone, CloneShape},
        control::CloneMeter,
    };
    if Arc::get_mut(map).is_none() {
        let bytes = map
            .len()
            .checked_mul(std::mem::size_of::<(UnitInstanceId, ActorFactSummary)>())
            .and_then(|bytes| bytes.checked_add(64));
        control::admit_retained_growth(0, bytes, "actor_summary_copy")?;
        if control::is_active() {
            let mut meter = CloneMeter::for_native_scan()?;
            map.as_ref().visit_clone_shape(&mut meter)?;
            control::charge(meter.native_scan_slots() as u64, "actor_summary_scan")?;
            control::charge_clone(
                meter
                    .native_copy_slots()
                    .ok_or_else(|| overflow("actor_summary_slots"))?,
                "actor_summary_copy",
            )?;
        }
        *map = Arc::new(controlled_clone(map.as_ref())?);
    }
    Ok(Arc::get_mut(map).expect("actor summaries have a unique writer"))
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFactState {
    pub(super) resources: Option<Box<crate::game::resource::runtime::RuntimeResources>>,
    attempt: Arc<AttemptToken>,
    next_action: u64,
    next_strike: u64,
    next_delivery: u64,
    next_fact: u64,
    active: Arc<BTreeMap<ActionOrdinal, ActiveAction>>,
    retained: BTreeMap<ActionOrdinal, RetainedAction>,
    pub(super) actors: Arc<ActorSummaries>,
    pub(super) providers: Arc<BTreeMap<EquipmentSourceSnapshot, ProviderFactSummary>>,
    latest_settled_action: Option<Arc<ActionSnapshot>>,
    pub(super) summary: FactSummary,
    pub(super) usage: RuntimeFactUsage,
    pub(super) limits: RuntimeFactLimits,
}
crate::authoring::clone_shape::struct_shape!(RuntimeFactState {
    resources,
    attempt,
    next_action,
    next_strike,
    next_delivery,
    next_fact,
    active,
    retained,
    actors,
    providers,
    summary,
    latest_settled_action,
    usage,
    limits
});

impl Default for RuntimeFactState {
    fn default() -> Self {
        Self::new(RuntimeFactLimits::default())
    }
}

fn overflow(site: &'static str) -> GameError {
    control::reject(BattleRuntimeFault {
        code: Code::CounterOverflow,
        site,
        limit: None,
        requested: None,
    })
}

fn invalid_cast_binding() -> GameError {
    control::reject(BattleRuntimeFault {
        code: Code::Invariant,
        site: "action_cast_binding",
        limit: None,
        requested: None,
    })
}

fn next(counter: &mut u64, site: &'static str) -> Result<u64, GameError> {
    let value = *counter;
    *counter = value.checked_add(1).ok_or_else(|| overflow(site))?;
    Ok(value)
}

/// Rust's B-tree nodes hold eleven key/value slots and up to twelve edges.
/// Every non-root node has at least five keys. Include one empty root even
/// after removal; inline spare slots are storage, not merely populated rows.
fn fact_tree_extent<K, V>(len: usize) -> Option<usize> {
    let nodes = 1_usize.checked_add(len.saturating_sub(1) / 5)?;
    let node_bytes = std::mem::size_of::<(K, V)>()
        .checked_mul(11)?
        .checked_add(12 * std::mem::size_of::<usize>())?
        .checked_add(128)?;
    nodes.checked_mul(node_bytes)
}

fn fact_tree_growth<K, V>(len: usize) -> Option<usize> {
    fact_tree_extent::<K, V>(len.checked_add(1)?)?.checked_sub(fact_tree_extent::<K, V>(len)?)
}

/// A summary map shares immutable witnesses between timestamp candidates. Only
/// an actual summary update detaches it, after admitting its nodes/inline data.
fn writable_fact_map<K, V>(
    map: &mut Arc<BTreeMap<K, V>>,
    nested_bytes: Option<usize>,
) -> Result<&mut BTreeMap<K, V>, GameError>
where
    K: Ord + Clone + crate::authoring::clone_shape::CloneShape,
    V: Clone + crate::authoring::clone_shape::CloneShape,
{
    use crate::authoring::{
        clone_shape::{controlled_clone, CloneShape},
        control::CloneMeter,
    };
    if Arc::get_mut(map).is_none() {
        let extent = fact_tree_extent::<K, V>(map.len())
            .and_then(|bytes| bytes.checked_add(64))
            .and_then(|bytes| nested_bytes.and_then(|nested| bytes.checked_add(nested)));
        control::admit_retained_growth(0, extent, "fact_summary_map_copy")?;
        if control::is_active() {
            let mut meter = CloneMeter::for_native_scan()?;
            map.as_ref().visit_clone_shape(&mut meter)?;
            control::charge(meter.native_scan_slots() as u64, "fact_summary_map_scan")?;
            control::charge_clone(
                meter
                    .native_copy_slots()
                    .ok_or_else(|| overflow("fact_summary_map_slots"))?,
                "fact_summary_map_copy",
            )?;
        }
        *map = Arc::new(controlled_clone(map.as_ref())?);
    }
    Ok(Arc::get_mut(map).expect("fact summary map has a unique writer"))
}

impl RuntimeFactState {
    pub(super) fn initial_retained_bytes_bound() -> Option<usize> {
        192_usize
            .checked_add(fact_tree_extent::<ActionOrdinal, ActiveAction>(0)?)?
            .checked_add(fact_tree_extent::<ActionOrdinal, RetainedAction>(0)?)?
            .checked_add(fact_tree_extent::<
                EquipmentSourceSnapshot,
                ProviderFactSummary,
            >(0)?)
    }
    pub(super) fn new(limits: RuntimeFactLimits) -> Self {
        Self {
            resources: None,
            attempt: Arc::new(AttemptToken),
            next_action: 0,
            next_strike: 0,
            next_delivery: 0,
            next_fact: 0,
            active: Arc::new(BTreeMap::new()),
            retained: BTreeMap::new(),
            actors: Arc::new(ActorSummaries::default()),
            providers: Arc::new(BTreeMap::new()),
            summary: FactSummary::default(),
            latest_settled_action: None,
            usage: RuntimeFactUsage {
                retained_bytes_bound: Self::initial_retained_bytes_bound().expect("fixed root"),
                ..Default::default()
            },
            limits,
        }
    }

    /// Keep an empty leaf available during a same-time action batch, then
    /// release its large inline value array before publishing the timestamp.
    /// This never detaches shared state or allocates a replacement owner.
    pub(super) fn release_empty_action_storage(&mut self) {
        if let Some(active) = Arc::get_mut(&mut self.active) {
            if active.is_empty() {
                *active = BTreeMap::new();
            }
        }
        if self.retained.is_empty() {
            self.retained = BTreeMap::new();
        }
    }

    fn active_for_write(
        &mut self,
    ) -> Result<&mut BTreeMap<ActionOrdinal, ActiveAction>, GameError> {
        if Arc::get_mut(&mut self.active).is_none() {
            crate::authoring::control::charge(self.active.len() as u64, "active_copy_extent_scan")?;
            let nested = self.active.values().try_fold(0_usize, |bytes, action| {
                action
                    .pending_steps
                    .len()
                    .checked_add(action.deliveries.len())
                    .and_then(|nodes| nodes.checked_mul(128))
                    .and_then(|extra| extra.checked_add(2 * fact_tree_extent::<usize, ()>(0)?))
                    .and_then(|extra| bytes.checked_add(extra))
            });
            writable_fact_map(&mut self.active, nested)?;
        }
        Ok(Arc::get_mut(&mut self.active).expect("active action map has a unique writer"))
    }

    fn replace_extent(
        &mut self,
        old: usize,
        new: Option<usize>,
        site: &'static str,
    ) -> Result<usize, GameError> {
        let new = new.ok_or_else(|| overflow(site))?;
        let requested = self
            .usage
            .retained_bytes_bound
            .checked_sub(old)
            .and_then(|n| n.checked_add(new));
        control::limit(
            Code::RetainedByteLimit,
            site,
            self.limits.max_retained_bytes,
            requested,
        )?;
        control::admit_retained_growth(old, Some(new), site)?;
        self.usage.retained_bytes_bound = requested.expect("checked retention");
        Ok(new)
    }

    pub(super) fn accept(
        &mut self,
        actor: UnitInstanceId,
        skill_id: Option<&SkillId>,
        source: RuntimeFactSource,
        time_ms: u64,
        public_reserved: bool,
    ) -> Result<ActionHandle, GameError> {
        crate::authoring::control::charge(1, "action_accept")?;
        control::limit(
            Code::ActiveActionLimit,
            "action_accept",
            self.limits.max_active_actions,
            self.active.len().checked_add(1),
        )?;
        if public_reserved {
            control::limit(
                Code::RetainedResultLimit,
                "action_result_reserve",
                self.limits.max_retained_results,
                self.usage.public_result_reservations.checked_add(1),
            )?;
        }
        // Admission includes the future compact receipt and allocation overhead;
        // settlement never needs to acquire another promised-result slot.
        let tree_growth = fact_tree_growth::<ActionOrdinal, ActiveAction>(self.active.len())
            .and_then(|active| {
                if public_reserved {
                    fact_tree_growth::<ActionOrdinal, RetainedAction>(
                        self.usage.public_result_reservations,
                    )
                    .and_then(|retained| active.checked_add(retained))
                } else {
                    Some(active)
                }
            });
        self.replace_extent(0, tree_growth, "action_tree_growth")?;
        let extent = std::mem::size_of::<ActiveAction>()
            .checked_add(256)
            .and_then(|n| n.checked_add(2 * fact_tree_extent::<usize, ()>(0)?))
            .and_then(|n| n.checked_add(skill_id.map_or(0, |id| id.as_str().len())))
            .and_then(|n| {
                source
                    .retained_extent()
                    .and_then(|source| n.checked_add(source))
            });
        let extent = self.replace_extent(0, extent, "action_source_reserve")?;
        let ordinal = ActionOrdinal(next(&mut self.next_action, "action_ordinal")?);
        let accepted_actions = self
            .usage
            .accepted_actions
            .checked_add(1)
            .ok_or_else(|| overflow("accepted_actions"))?;
        let handle = ActionHandle {
            validity: Arc::new(ActionToken {
                attempt: Arc::clone(&self.attempt),
                ordinal,
            }),
        };
        if let Some(resources) = self.resources.as_mut() {
            let parent = source.resource_context().cloned().or_else(|| {
                source
                    .action
                    .as_ref()
                    .and_then(|action| resources.context(action).cloned())
            });
            resources.accept(&handle, parent.as_deref())?;
        }
        let mut source = source.observation();
        source.action = Some(handle.clone());
        self.active_for_write()?.insert(
            ordinal,
            ActiveAction {
                handle: handle.clone(),
                snapshot: ActionSnapshot {
                    ordinal,
                    actor,
                    skill_id: skill_id.map(|id| Arc::new(id.clone())),
                    source: Arc::new(source),
                    accepted_at_ms: time_ms,
                    released_at_ms: None,
                    terminal: None,
                    effects_settled_at_ms: None,
                    pending_control_steps: 0,
                    outstanding_deliveries: 0,
                    facts: FactSummary::default(),
                },
                bound_cast: None,
                action_result: None,
                pending_steps: BTreeSet::new(),
                deliveries: BTreeSet::new(),
                public_reserved,
                extent,
            },
        );
        self.usage.active_actions += 1;
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::peak(
            crate::runtime_observation::ACTIONS,
            self.usage.active_actions as u64,
        );
        self.usage.public_result_reservations += usize::from(public_reserved);
        self.usage.accepted_actions = accepted_actions;
        Ok(handle)
    }

    pub(super) fn independent_origin_for_action(
        &self,
        handle: &ActionHandle,
    ) -> Option<Arc<crate::game::battle::runtime_facts::IndependentObjectOriginSnapshot>> {
        self.live(handle)
            .then(|| {
                self.active[&handle.ordinal()]
                    .snapshot
                    .source
                    .independent_object()
                    .cloned()
            })
            .flatten()
    }

    pub(super) fn is_basic_action(&self, handle: &ActionHandle) -> bool {
        self.live(handle) && self.active[&handle.ordinal()].snapshot.skill_id.is_none()
    }

    pub(super) fn cancel_unreleased_basic(
        &mut self,
        actor: UnitInstanceId,
        reason: ActionCancelReason,
        time_ms: u64,
    ) -> Result<(), GameError> {
        crate::authoring::control::charge(self.active.len() as u64, "cancel_basic_actions")?;
        let extent = self.reserve_fact_storage(
            self.active
                .len()
                .checked_mul(std::mem::size_of::<ActionHandle>())
                .and_then(|bytes| bytes.checked_add(64)),
        )?;
        let mut handles = Vec::with_capacity(self.active.len());
        handles.extend(
            self.active
                .values()
                .filter(|action| {
                    action.snapshot.actor == actor
                        && action.snapshot.skill_id.is_none()
                        && action.snapshot.released_at_ms.is_none()
                        && action.snapshot.terminal.is_none()
                })
                .map(|action| action.handle.clone()),
        );
        let result = (|| {
            for handle in handles {
                self.terminal(&handle, ActionTerminalKind::Cancelled(reason), time_ms)?;
            }
            Ok(())
        })();
        self.release_fact_storage(extent);
        result
    }

    fn live(&self, handle: &ActionHandle) -> bool {
        Arc::ptr_eq(&self.attempt, &handle.validity.attempt)
            && self
                .active
                .get(&handle.ordinal())
                .is_some_and(|action| action.handle == *handle)
    }

    pub(super) fn validate_live_action(
        &self,
        handle: &ActionHandle,
    ) -> Result<(), ActionQueryError> {
        if !Arc::ptr_eq(&self.attempt, &handle.validity.attempt) {
            return Err(ActionQueryError::DifferentAttempt);
        }
        match self.active.get(&handle.ordinal()) {
            Some(action) if action.handle == *handle => Ok(()),
            Some(_) => Err(ActionQueryError::StaleOccurrence),
            None => Err(ActionQueryError::Unavailable),
        }
    }

    pub(super) fn action_result_witness(
        &self,
        handle: &ActionHandle,
    ) -> Result<Option<&super::value_expr_runtime::ActionResultWitness>, ActionQueryError> {
        self.validate_live_action(handle)?;
        Ok(self.active[&handle.ordinal()].action_result.as_deref())
    }

    pub(super) fn install_action_result_witness(
        &mut self,
        handle: &ActionHandle,
        skill_id: &SkillId,
        witness: super::value_expr_runtime::ActionResultWitness,
    ) -> Result<(), GameError> {
        self.validate_live_action(handle).map_err(|_| {
            GameError::InvalidStaticData("action result has no live action".to_string())
        })?;
        let action = &self.active[&handle.ordinal()];
        if action.action_result.is_some()
            || action
                .snapshot
                .skill_id
                .as_deref()
                .is_some_and(|id| id != skill_id)
            || witness.definition_id() != skill_id
        {
            return Err(GameError::InvalidStaticData(
                "action result definition or installation frontier differs".to_string(),
            ));
        }
        let bytes = witness
            .retained_extent()
            .ok_or_else(|| overflow("action_result_extent"))?;
        // The caller admitted the complete witness before cloning its IDs or
        // allocating the cumulative slots. Record its active owner here
        // without charging the same native retained growth a second time.
        let extent = action
            .extent
            .checked_add(bytes)
            .ok_or_else(|| overflow("action_result_extent"))?;
        let requested = self
            .usage
            .retained_bytes_bound
            .checked_add(bytes)
            .ok_or_else(|| overflow("action_result_extent"))?;
        control::limit(
            Code::RetainedByteLimit,
            "action_result_witness",
            self.limits.max_retained_bytes,
            Some(requested),
        )?;
        let action = self
            .active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action");
        action.action_result = Some(Box::new(witness));
        action.extent = extent;
        self.usage.retained_bytes_bound = requested;
        Ok(())
    }

    pub(super) fn settle_action_result(
        &mut self,
        source: &RuntimeFactSource,
        site: &crate::game::ability::authoring::ActionSite,
        projection: &super::value_expr_runtime::EffectResultProjection,
    ) -> Result<(), GameError> {
        let Some(handle) = source.action.as_ref() else {
            return Ok(());
        };
        self.validate_action_source(handle, source).map_err(|_| {
            GameError::InvalidStaticData(
                "action result source is not the active action".to_string(),
            )
        })?;
        if let Some(delivery) = source.delivery.as_deref() {
            self.validate_live_delivery(delivery).map_err(|_| {
                GameError::InvalidStaticData("action result delivery is not live".to_string())
            })?;
        }
        if self.active[&handle.ordinal()].action_result.is_none() {
            return Ok(());
        }
        self.active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action")
            .action_result
            .as_mut()
            .expect("checked witness")
            .settle(site, projection)
    }

    /// The active action owns the accepted execution identity. Effect metadata
    /// may differ for a child or delivery, but its actor and exact provider
    /// copy cannot be supplied by another execution.
    pub(super) fn validate_action_source(
        &self,
        handle: &ActionHandle,
        source: &RuntimeFactSource,
    ) -> Result<Option<EquipmentSourceSnapshot>, ActionQueryError> {
        self.validate_live_action(handle)?;
        let snapshot = &self.active[&handle.ordinal()].snapshot;
        let bound_cast = self.active[&handle.ordinal()].bound_cast.as_ref();
        if match (
            snapshot.source.independent_object(),
            source.independent_object(),
        ) {
            (Some(bound), Some(claimed)) => !Arc::ptr_eq(bound, claimed),
            (None, None) => false,
            _ => true,
        } || match (snapshot.source.relic_effective(), source.relic_effective()) {
            (Some(bound), Some(claimed)) => !Arc::ptr_eq(bound, claimed),
            (None, None) => false,
            _ => true,
        } || match (&snapshot.source.fragment, &source.fragment) {
            (Some(bound), Some(claimed)) => !Arc::ptr_eq(bound, claimed),
            (None, None) => false,
            _ => true,
        } || source.actor.is_some_and(|actor| actor != snapshot.actor)
            || snapshot
                .source
                .actor
                .is_some_and(|actor| actor != snapshot.actor)
            || snapshot
                .source
                .equipment
                .is_some_and(|provider| source.equipment != Some(provider))
            || snapshot.source.cast.as_ref().is_some_and(|cast| {
                source
                    .cast
                    .as_ref()
                    .is_none_or(|claimed| !Arc::ptr_eq(cast, claimed))
            })
            || match (bound_cast, source.cast.as_ref()) {
                (Some(bound), Some(claimed)) => !Arc::ptr_eq(bound, claimed),
                (Some(_), None) | (None, Some(_)) if snapshot.source.cast.is_none() => true,
                _ => false,
            }
        {
            return Err(ActionQueryError::StaleOccurrence);
        }
        Ok(snapshot.source.equipment)
    }

    /// An accepted skill has no cast yet. Once the execution source exists,
    /// the action owner fixes that one relationship before any child source is
    /// published. The private Arc identity distinguishes even equal cast data.
    pub(super) fn bind_cast(
        &mut self,
        handle: &ActionHandle,
        cast: &Arc<crate::game::battle::skill_source::SkillCastExecutionSource>,
    ) -> Result<(), GameError> {
        self.validate_live_action(handle)
            .map_err(|_| invalid_cast_binding())?;
        let action = &self.active[&handle.ordinal()];
        if action.bound_cast.is_some()
            || action.snapshot.actor != cast.caster_instance_id
            || action.snapshot.skill_id.as_deref() != Some(&cast.skill_id)
            || action.snapshot.source.cast.is_some()
        {
            return Err(invalid_cast_binding());
        }
        let extent = self.replace_extent(
            action.extent,
            action
                .extent
                .checked_add(std::mem::size_of_val(cast.as_ref()))
                .and_then(|bytes| bytes.checked_add(cast.skill_id.as_str().len())),
            "action_cast_binding",
        )?;
        let action = self
            .active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action");
        action.bound_cast = Some(Arc::clone(cast));
        action.extent = extent;
        Ok(())
    }

    pub(super) fn validate_live_delivery(
        &self,
        ticket: &DeliveryTicket,
    ) -> Result<(), ActionQueryError> {
        self.validate_live_action(&ticket.action)?;
        self.active
            .get(&ticket.action.ordinal())
            .filter(|action| action.deliveries.contains(&ticket.ordinal))
            .map(|_| ())
            .ok_or(ActionQueryError::StaleOccurrence)
    }

    pub(super) fn release(
        &mut self,
        handle: &ActionHandle,
        steps: usize,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if !self.live(handle) {
            return Ok(());
        }
        let action = &self.active[&handle.ordinal()];
        if action.snapshot.released_at_ms.is_some() || action.snapshot.terminal.is_some() {
            return Ok(());
        }
        control::limit(
            Code::ControlStepLimit,
            "action_release_steps",
            self.limits.max_control_steps,
            Some(steps),
        )?;
        crate::authoring::control::charge(steps as u64, "action_release_steps")?;
        let old = action.extent;
        let extent = self.replace_extent(
            old,
            steps
                .checked_mul(128)
                .and_then(|bytes| old.checked_add(bytes)),
            "action_step_storage",
        )?;
        let action = self
            .active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action");
        action.pending_steps.extend(0..steps);
        action.snapshot.pending_control_steps = steps;
        action.snapshot.released_at_ms = Some(time_ms);
        action.extent = extent;
        if steps == 0 {
            self.terminal(handle, ActionTerminalKind::Finished, time_ms)?;
        }
        Ok(())
    }

    pub(super) fn dispatched(
        &mut self,
        handle: &ActionHandle,
        step: usize,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if !self.live(handle) {
            return Ok(());
        }
        if self.active[&handle.ordinal()].snapshot.terminal.is_some() {
            return Ok(());
        }
        let action = self
            .active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action");
        let removed = action.pending_steps.remove(&step);
        if removed {
            action.snapshot.pending_control_steps = action.pending_steps.len();
            action.extent -= 128;
        }
        let finished = action.pending_steps.is_empty() && action.snapshot.released_at_ms.is_some();
        if removed {
            self.usage.retained_bytes_bound -= 128;
        }
        if finished {
            self.terminal(handle, ActionTerminalKind::Finished, time_ms)?;
        }
        Ok(())
    }

    pub(super) fn terminal(
        &mut self,
        handle: &ActionHandle,
        kind: ActionTerminalKind,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if !self.live(handle) {
            return Ok(());
        }
        if self.active[&handle.ordinal()].snapshot.terminal.is_none() {
            if let Some(resources) = self.resources.as_mut() {
                resources.terminal(handle)?;
            }
            let action = self
                .active_for_write()?
                .get_mut(&handle.ordinal())
                .expect("live action");
            action.snapshot.terminal = Some(ActionTerminal { time_ms, kind });
            let released = action.pending_steps.len() * 128;
            action.pending_steps.clear();
            action.snapshot.pending_control_steps = 0;
            action.extent -= released;
            self.usage.retained_bytes_bound -= released;
        }
        self.settle_if_ready(handle, time_ms)
    }

    pub(super) fn delivery(
        &mut self,
        handle: &ActionHandle,
    ) -> Result<Option<Arc<DeliveryTicket>>, GameError> {
        self.delivery_with_resource_parent(handle, None)
    }

    fn delivery_with_resource_parent(
        &mut self,
        handle: &ActionHandle,
        parent: Option<&crate::game::resource::runtime::ResourceDelivery>,
    ) -> Result<Option<Arc<DeliveryTicket>>, GameError> {
        if !self.live(handle) {
            return Ok(None);
        }
        control::limit(
            Code::DeliveryLimit,
            "finite_delivery",
            self.limits.max_deliveries,
            self.usage.outstanding_deliveries.checked_add(1),
        )?;
        let old = self.active[&handle.ordinal()].extent;
        let extent = self.replace_extent(old, old.checked_add(128), "finite_delivery_storage")?;
        let ordinal = DeliveryOrdinal(next(&mut self.next_delivery, "delivery_ordinal")?);
        let resource = if let Some(resources) = self.resources.as_mut() {
            let context = resources.context(handle).cloned().ok_or_else(|| {
                crate::game::resource::store::ResourceError::from(
                    crate::game::resource::store::ResourceFaultCode::StaleScope,
                )
                .into_game_error()
            })?;
            Some(match parent {
                Some(parent) => resources.finite_completion(Some(&context), parent)?.1,
                None => resources.finite(Some(&context))?.1,
            })
        } else {
            None
        };
        let action = self
            .active_for_write()?
            .get_mut(&handle.ordinal())
            .expect("live action");
        action.deliveries.insert(ordinal);
        action.snapshot.outstanding_deliveries = action.deliveries.len();
        action.extent = extent;
        self.usage.outstanding_deliveries += 1;
        control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<DeliveryTicket>() + 64),
            "delivery_ticket_owner",
        )?;
        Ok(Some(Arc::new(DeliveryTicket {
            action: handle.clone(),
            ordinal,
            resource,
        })))
    }

    pub(super) fn close_delivery(
        &mut self,
        ticket: &DeliveryTicket,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if !self.live(&ticket.action) {
            return Ok(());
        }
        if !self.active[&ticket.action.ordinal()]
            .deliveries
            .contains(&ticket.ordinal)
        {
            return self.settle_if_ready(&ticket.action, time_ms);
        }
        if let (Some(resources), Some(delivery)) =
            (self.resources.as_mut(), ticket.resource.as_ref())
        {
            resources.close_finite(delivery)?;
        }
        let action = self
            .active_for_write()?
            .get_mut(&ticket.action.ordinal())
            .expect("live action");
        if action.deliveries.remove(&ticket.ordinal) {
            action.snapshot.outstanding_deliveries = action.deliveries.len();
            action.extent -= 128;
            self.usage.outstanding_deliveries -= 1;
            self.usage.retained_bytes_bound -= 128;
        }
        self.settle_if_ready(&ticket.action, time_ms)
    }

    fn settle_if_ready(&mut self, handle: &ActionHandle, time_ms: u64) -> Result<(), GameError> {
        let Some(action) = self.active.get(&handle.ordinal()) else {
            return Ok(());
        };
        if action.handle != *handle
            || action.snapshot.terminal.is_none()
            || !action.deliveries.is_empty()
        {
            return Ok(());
        }
        let settled_actions = self
            .usage
            .settled_actions
            .checked_add(1)
            .ok_or_else(|| overflow("settled_actions"))?;
        let mut snapshot = action.snapshot.clone();
        snapshot.effects_settled_at_ms = Some(time_ms);
        let witness_extent =
            action_snapshot_extent(&snapshot).ok_or_else(|| overflow("settled_witness_extent"))?;
        let old = self
            .latest_settled_action
            .as_ref()
            .map_or(Some(0), |value| action_snapshot_extent(value))
            .ok_or_else(|| overflow("settled_witness_extent"))?;
        self.replace_extent(old, Some(witness_extent), "latest_settled_action")?;
        self.actor(snapshot.actor)?;
        let old = self.actors[&snapshot.actor]
            .latest_settled_action
            .as_ref()
            .map_or(Some(0), |value| action_snapshot_extent(value))
            .ok_or_else(|| overflow("settled_witness_extent"))?;
        self.replace_extent(old, Some(witness_extent), "actor_settled_action")?;
        crate::authoring::control::charge(1, "settled_witness_owner")?;
        let snapshot = Arc::new(snapshot);
        self.latest_settled_action = Some(snapshot.clone());
        writable_actor_summaries(&mut self.actors)?
            .get_mut(&snapshot.actor)
            .expect("admitted actor")
            .latest_settled_action = Some(snapshot.clone());
        let action = self
            .active_for_write()?
            .remove(&handle.ordinal())
            .expect("checked action");
        if let Some(resources) = self.resources.as_mut() {
            resources.settled(handle);
        }
        self.usage.retained_bytes_bound -=
            fact_tree_growth::<ActionOrdinal, ActiveAction>(self.active.len())
                .expect("admitted tree");
        self.usage.active_actions -= 1;
        self.usage.settled_actions = settled_actions;
        let action_result_bytes = match &action.action_result {
            Some(witness) => witness
                .retained_extent()
                .ok_or_else(|| overflow("action_result_extent"))?,
            None => 0,
        };
        let retained_action_extent = action
            .extent
            .checked_sub(action_result_bytes)
            .ok_or_else(|| overflow("action_result_extent"))?;
        self.usage.retained_bytes_bound -= action_result_bytes;
        if action.public_reserved {
            self.retained.insert(
                handle.ordinal(),
                RetainedAction {
                    handle: action.handle,
                    receipt: Arc::new(SettledActionReceipt { action: snapshot }),
                    extent: retained_action_extent,
                },
            );
            self.usage.completed_results += 1;
        } else {
            self.usage.retained_bytes_bound -= retained_action_extent;
        }
        Ok(())
    }

    fn observed_source(
        &self,
        source: &RuntimeFactSource,
    ) -> Result<Arc<RuntimeFactSource>, GameError> {
        if let Some(action) = source
            .action
            .as_ref()
            .and_then(|handle| self.active.get(&handle.ordinal()))
        {
            if action.snapshot.source.as_ref() == source {
                return Ok(Arc::clone(&action.snapshot.source));
            }
        }
        control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|bytes| bytes.checked_add(64)),
            "observed_fact_source_owner",
        )?;
        Ok(Arc::new(crate::authoring::clone_shape::controlled_clone(
            source,
        )?))
    }

    pub(super) fn accepted_source(
        &self,
        handle: &ActionHandle,
    ) -> Result<&RuntimeFactSource, ActionQueryError> {
        self.validate_live_action(handle)?;
        Ok(self.active[&handle.ordinal()].snapshot.source.as_ref())
    }
    pub(super) fn query(&self, handle: &ActionHandle) -> Result<ActionSnapshot, ActionQueryError> {
        if !Arc::ptr_eq(&self.attempt, &handle.validity.attempt) {
            return Err(ActionQueryError::DifferentAttempt);
        }
        if let Some(action) = self.active.get(&handle.ordinal()) {
            if action.handle != *handle {
                return Err(ActionQueryError::StaleOccurrence);
            }
            return Ok(action.snapshot.clone());
        }
        if let Some(action) = self.retained.get(&handle.ordinal()) {
            if action.handle != *handle {
                return Err(ActionQueryError::StaleOccurrence);
            }
            return Ok(action.receipt.action.as_ref().clone());
        }
        Err(ActionQueryError::Unavailable)
    }

    pub(super) fn detach_result(&mut self, handle: &ActionHandle) -> Result<(), GameError> {
        if !Arc::ptr_eq(&self.attempt, &handle.validity.attempt) {
            return Err(GameError::InvalidAction);
        }
        if let Some(active) = self.active.get(&handle.ordinal()) {
            if active.handle != *handle {
                return Err(GameError::InvalidAction);
            }
            if active.public_reserved {
                self.active_for_write()?
                    .get_mut(&handle.ordinal())
                    .expect("validated action")
                    .public_reserved = false;
                self.usage.public_result_reservations -= 1;
                self.usage.retained_bytes_bound -=
                    fact_tree_growth::<ActionOrdinal, RetainedAction>(
                        self.usage.public_result_reservations,
                    )
                    .expect("admitted result tree");
            }
        } else if self.retained.contains_key(&handle.ordinal()) {
            self.take(handle).map_err(|_| GameError::InvalidAction)?;
        }
        Ok(())
    }

    pub(super) fn take(
        &mut self,
        handle: &ActionHandle,
    ) -> Result<Arc<SettledActionReceipt>, ActionQueryError> {
        if !Arc::ptr_eq(&self.attempt, &handle.validity.attempt) {
            return Err(ActionQueryError::DifferentAttempt);
        }
        if let Some(action) = self.active.get(&handle.ordinal()) {
            return Err(if action.handle == *handle {
                ActionQueryError::NotSettled
            } else {
                ActionQueryError::StaleOccurrence
            });
        }
        let action = self
            .retained
            .get(&handle.ordinal())
            .ok_or(ActionQueryError::Unavailable)?;
        if action.handle != *handle {
            return Err(ActionQueryError::StaleOccurrence);
        }
        let receipt = Arc::clone(&action.receipt);
        let action = self
            .retained
            .remove(&handle.ordinal())
            .expect("validated result");
        self.usage.public_result_reservations -= 1;
        self.usage.retained_bytes_bound -= fact_tree_growth::<ActionOrdinal, RetainedAction>(
            self.usage.public_result_reservations,
        )
        .expect("admitted result tree");
        self.usage.completed_results -= 1;
        self.usage.retained_bytes_bound -= action.extent;
        if self.retained.is_empty() {
            self.retained = BTreeMap::new();
        }
        Ok(receipt)
    }

    pub(super) fn strike(&mut self) -> Result<StrikeOrdinal, GameError> {
        next(&mut self.next_strike, "strike_ordinal").map(StrikeOrdinal)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StrikeProgress {
    deflected: bool,
    ordinal: StrikeOrdinal,
    next_segment: usize,
    mutations: u64,
    positive: bool,
}
crate::authoring::clone_shape::copy_shape!(StrikeProgress);

/// Admitted before current-hit planning and formula evaluation. Fact preparation
/// consumes this identity; it cannot create an identity from a completed result.
pub(super) struct AdmittedStrike {
    target: UnitInstanceId,
    progress: StrikeProgress,
}

impl AdmittedStrike {
    pub(super) fn first_positive_roll_key(&self) -> Option<u64> {
        (!self.progress.positive && !self.progress.deflected).then_some(self.progress.ordinal.0)
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct StrikeScope {
    targets: BTreeMap<UnitInstanceId, StrikeProgress>,
}
crate::authoring::clone_shape::struct_shape!(StrikeScope { targets });
impl BattleCore {
    pub(crate) fn mount_run_resources(
        &mut self,
        state: &crate::game::resource::durable::RunResourceState,
        owners: &std::collections::BTreeSet<crate::game::resource::store::ResourceOwner>,
    ) -> Result<(), GameError> {
        self.prepare_runtime_resources()?;
        if let Some(resources) = self.runtime_facts.resources.as_mut() {
            resources.mount_run(state, owners)?;
        }
        Ok(())
    }

    pub(crate) fn capture_run_resources(
        &self,
    ) -> Result<Option<crate::game::resource::durable::RunResourceResult>, GameError> {
        let mut result = self
            .runtime_facts
            .resources
            .as_ref()
            .map(|resources| resources.capture_run())
            .transpose()?
            .flatten();
        if let Some(result) = &mut result {
            result.limits = self.runtime_fact_limits();
        }
        Ok(result)
    }

    pub(super) fn begin_strike_scope(&mut self) -> Option<StrikeScope> {
        self.runtime_strike_scope.replace(StrikeScope::default())
    }
    pub(super) fn end_strike_scope(&mut self, prior: Option<StrikeScope>) {
        if let Some(scope) = std::mem::replace(&mut self.runtime_strike_scope, prior) {
            self.runtime_facts.release_fact_storage(
                scope.targets.len() * (std::mem::size_of::<StrikeProgress>() + 96),
            );
        }
    }
    pub(super) fn admit_damage_strike(
        &mut self,
        target: UnitInstanceId,
    ) -> Result<AdmittedStrike, GameError> {
        let existing = self
            .runtime_strike_scope
            .as_ref()
            .and_then(|scope| scope.targets.get(&target))
            .copied();
        let progress = match existing {
            Some(value) => value,
            None => {
                if self.runtime_strike_scope.is_some() {
                    self.runtime_facts
                        .reserve_fact_storage(Some(std::mem::size_of::<StrikeProgress>() + 96))?;
                }
                let progress = StrikeProgress {
                    deflected: false,
                    ordinal: self.runtime_facts.strike()?,
                    next_segment: 0,
                    mutations: 0,
                    positive: false,
                };
                if let Some(scope) = &mut self.runtime_strike_scope {
                    scope.targets.insert(target, progress);
                }
                progress
            }
        };
        Ok(AdmittedStrike { target, progress })
    }

    pub(super) fn mark_strike_deflected(&mut self, strike: &mut AdmittedStrike) {
        strike.progress.deflected = true;
        if let Some(scope) = &mut self.runtime_strike_scope {
            scope.targets.insert(strike.target, strike.progress);
        }
    }
    pub(super) fn strike_deflected(&self, target: UnitInstanceId) -> bool {
        self.runtime_strike_scope
            .as_ref()
            .and_then(|s| s.targets.get(&target))
            .is_some_and(|p| p.deflected)
    }
    fn fact_strike_segments(
        &mut self,
        admitted: AdmittedStrike,
        count: usize,
        positive: bool,
    ) -> Result<(StrikeOrdinal, u32, bool, bool), GameError> {
        crate::authoring::control::charge(count as u64, "strike_segments")?;
        let AdmittedStrike {
            target,
            mut progress,
        } = admitted;
        let next = progress.next_segment.checked_add(count);
        control::limit(
            Code::StrikeSegmentLimit,
            "strike_segments",
            self.runtime_facts.limits.max_strike_segments,
            next,
        )?;
        let result = (
            progress.ordinal,
            progress.next_segment as u32,
            progress.mutations == 0,
            positive && !progress.positive,
        );
        progress.next_segment = next.expect("admitted segments");
        progress.mutations = progress
            .mutations
            .checked_add(1)
            .ok_or_else(|| overflow("strike_mutations"))?;
        progress.positive |= positive;
        if let Some(scope) = &mut self.runtime_strike_scope {
            scope.targets.insert(target, progress);
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub(super) enum FactUpdate {
    Damage(Arc<DamageFact>, bool, bool),
    Heal(Arc<HealFact>),
    Cost(Arc<CostFact>),
    ResourceChange(Arc<ResourceChangeFact>),
    Death(Arc<DeathFact>),
}

fn add_u64(slot: &mut u64, delta: u64) -> Result<(), GameError> {
    *slot = slot
        .checked_add(delta)
        .ok_or_else(|| overflow("fact_total"))?;
    Ok(())
}
fn add_i64(slot: &mut i64, delta: i64) -> Result<(), GameError> {
    *slot = slot
        .checked_add(delta)
        .ok_or_else(|| overflow("fact_signed_total"))?;
    Ok(())
}
fn sum_extents(values: &[usize]) -> Option<usize> {
    values
        .iter()
        .try_fold(0usize, |sum, value| sum.checked_add(*value))
}
fn summary_extent(summary: &FactSummary) -> Option<usize> {
    sum_extents(&[
        summary
            .latest_damage
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_nonzero_damage
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_heal
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_nonzero_heal
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_resource_change
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_cost
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_nonzero_cost
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_death
            .as_ref()
            .map_or(0, |v| v.retained_extent),
    ])
}
fn action_snapshot_extent(value: &ActionSnapshot) -> Option<usize> {
    std::mem::size_of::<ActionSnapshot>()
        .checked_add(64)?
        .checked_add(value.source.retained_extent()?)?
        .checked_add(value.skill_id.as_ref().map_or(0, |id| id.as_str().len()))?
        .checked_add(summary_extent(&value.facts)?)
}

fn provider_extent(summary: &ProviderFactSummary) -> Option<usize> {
    sum_extents(&[
        summary
            .latest_damage
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_heal
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_resource_change
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_cost
            .as_ref()
            .map_or(0, |v| v.retained_extent),
        summary
            .latest_death
            .as_ref()
            .map_or(0, |v| v.retained_extent),
    ])
}
impl FactUpdate {
    fn source(&self) -> &RuntimeFactSource {
        match self {
            Self::Damage(v, ..) => &v.source,
            Self::Heal(v) => &v.source,
            Self::Cost(v) => &v.source,
            Self::ResourceChange(v) => &v.source,
            Self::Death(v) => &v.source,
        }
    }
    fn recipient(&self) -> Option<UnitInstanceId> {
        match self {
            Self::Damage(v, ..) => Some(v.target),
            Self::Heal(v) => Some(v.target),
            Self::Cost(v) => match v.owner {
                ResourceOwner::Unit(id) => Some(id),
                _ => None,
            },
            Self::ResourceChange(v) => match v.owner {
                ResourceOwner::Unit(id) => Some(id),
                _ => None,
            },
            Self::Death(v) => Some(v.predeath.victim),
        }
    }
    fn apply(&self, summary: &mut FactSummary) -> Result<(), GameError> {
        match self {
            Self::Damage(value, first, first_positive) => {
                add_u64(&mut summary.damage_mutations, 1)?;
                add_u64(&mut summary.damage_strikes, u64::from(*first))?;
                add_u64(&mut summary.damaging_strikes, u64::from(*first_positive))?;
                add_u64(
                    &mut summary.damage_hp_lost,
                    u64::from(value.actual_hp_loss()),
                )?;
                summary.latest_damage = Some(value.clone());
                if value.actual_hp_loss() > 0 {
                    summary.latest_nonzero_damage = Some(value.clone());
                }
            }
            Self::Heal(value) => {
                add_u64(&mut summary.heal_mutations, 1)?;
                add_i64(&mut summary.healing_delta, value.actual_delta())?;
                summary.latest_heal = Some(value.clone());
                if value.actual_delta() != 0 {
                    summary.latest_nonzero_heal = Some(value.clone());
                }
            }
            Self::ResourceChange(value) => {
                add_u64(&mut summary.resource_change_mutations, 1)?;
                add_i64(&mut summary.resource_change_delta, value.actual_delta())?;
                summary.latest_resource_change = Some(value.clone());
            }
            Self::Cost(value) => {
                add_u64(&mut summary.cost_mutations, 1)?;
                add_i64(&mut summary.resource_spent, value.actual_spent())?;
                summary.latest_cost = Some(value.clone());
                if value.actual_spent() != 0 {
                    summary.latest_nonzero_cost = Some(value.clone());
                }
            }
            Self::Death(value) => {
                add_u64(&mut summary.deaths, 1)?;
                summary.latest_death = Some(value.clone());
            }
        }
        Ok(())
    }
}

impl RuntimeFactState {
    pub(super) fn reserve_fact_storage(
        &mut self,
        bytes: Option<usize>,
    ) -> Result<usize, GameError> {
        self.replace_extent(0, bytes, "fact_storage_admission")
    }
    pub(super) fn release_fact_storage(&mut self, bytes: usize) {
        self.usage.retained_bytes_bound -= bytes;
    }
    pub(super) fn fact_ordinal(&mut self) -> Result<u64, GameError> {
        next(&mut self.next_fact, "fact_ordinal")
    }
    fn actor(&mut self, actor: UnitInstanceId) -> Result<(), GameError> {
        if self.actors.contains_key(&actor) {
            return Ok(());
        }
        control::limit(
            Code::ActorLimit,
            "fact_actor",
            self.limits.max_actors,
            self.actors.len().checked_add(1),
        )?;
        self.reserve_fact_storage(std::mem::size_of::<ActorFactSummary>().checked_add(128))?;
        writable_actor_summaries(&mut self.actors)?.insert(
            actor,
            ActorFactSummary::default(),
            self.limits.max_actors,
        )?;
        self.usage.actor_summaries += 1;
        Ok(())
    }
    pub(super) fn register_provider(
        &mut self,
        key: EquipmentSourceSnapshot,
        actor: Option<UnitInstanceId>,
    ) -> Result<(), GameError> {
        if self.providers.contains_key(&key) {
            return Ok(());
        }
        control::limit(
            Code::ProviderLimit,
            "fact_provider",
            self.limits.max_providers,
            self.providers.len().checked_add(1),
        )?;
        self.reserve_fact_storage(fact_tree_growth::<
            EquipmentSourceSnapshot,
            ProviderFactSummary,
        >(self.providers.len()))?;
        writable_fact_map(&mut self.providers, Some(0))?.insert(
            key,
            ProviderFactSummary {
                actor,
                ..Default::default()
            },
        );
        self.usage.provider_summaries += 1;
        Ok(())
    }
    fn update_provider(
        &mut self,
        source: &RuntimeFactSource,
        update: &FactUpdate,
        segments: u64,
        calculated: u64,
        modifiers: u64,
    ) -> Result<(), GameError> {
        let Some(key) = source.equipment else {
            return Ok(());
        };
        self.register_provider(key, source.actor)?;
        let prior = &self.providers[&key];
        let old = provider_extent(prior).ok_or_else(|| overflow("provider_extent"))?;
        let mut value = prior.clone();
        add_u64(&mut value.damage_segments, segments)?;
        add_u64(&mut value.calculated_damage, calculated)?;
        add_u64(&mut value.modifier_contributions, modifiers)?;
        match update {
            FactUpdate::Damage(v, ..) => value.latest_damage = Some(v.clone()),
            FactUpdate::Heal(v) => value.latest_heal = Some(v.clone()),
            FactUpdate::Cost(v) => value.latest_cost = Some(v.clone()),
            FactUpdate::ResourceChange(v) => value.latest_resource_change = Some(v.clone()),
            FactUpdate::Death(v) => value.latest_death = Some(v.clone()),
        }
        self.replace_extent(old, provider_extent(&value), "provider_witness")?;
        writable_fact_map(&mut self.providers, Some(0))?.insert(key, value);
        Ok(())
    }
    pub(super) fn record_fact(&mut self, update: FactUpdate) -> Result<(), GameError> {
        crate::authoring::control::charge(1, "record_fact")?;
        let old = summary_extent(&self.summary).ok_or_else(|| overflow("summary_extent"))?;
        let mut summary = self.summary.clone();
        update.apply(&mut summary)?;
        self.replace_extent(old, summary_extent(&summary), "attempt_witness")?;
        self.summary = summary;
        for (actor, caused) in [(update.source().actor, true), (update.recipient(), false)] {
            let Some(actor) = actor else {
                continue;
            };
            self.actor(actor)?;
            let value = &self.actors[&actor];
            let prior = if caused {
                &value.caused
            } else {
                &value.received
            };
            let old = summary_extent(prior).ok_or_else(|| overflow("actor_extent"))?;
            let mut summary = prior.clone();
            update.apply(&mut summary)?;
            self.replace_extent(old, summary_extent(&summary), "actor_witness")?;
            let value = writable_actor_summaries(&mut self.actors)?
                .get_mut(&actor)
                .expect("admitted actor");
            if caused {
                value.caused = summary;
            } else {
                value.received = summary;
            }
        }
        if let Some(action) = update
            .source()
            .action
            .as_ref()
            .filter(|action| self.live(action))
        {
            let value = &self.active[&action.ordinal()];
            let old =
                summary_extent(&value.snapshot.facts).ok_or_else(|| overflow("action_extent"))?;
            let mut summary = value.snapshot.facts.clone();
            update.apply(&mut summary)?;
            let new = summary_extent(&summary).ok_or_else(|| overflow("action_extent"))?;
            let extent = value
                .extent
                .checked_sub(old)
                .and_then(|n| n.checked_add(new));
            let extent = self.replace_extent(value.extent, extent, "action_witness")?;
            let value = self
                .active_for_write()?
                .get_mut(&action.ordinal())
                .expect("live action");
            value.snapshot.facts = summary;
            value.extent = extent;
        }
        match &update {
            FactUpdate::Damage(value, ..) => {
                for segment in &value.segments {
                    crate::authoring::control::charge(1, "provider_segment")?;
                    self.update_provider(
                        &segment.source,
                        &update,
                        1,
                        u64::from(segment.calculated_damage),
                        0,
                    )?;
                }
                for source in &value.modifier_sources {
                    crate::authoring::control::charge(1, "provider_modifier")?;
                    self.update_provider(source, &update, 0, 0, 1)?;
                }
            }
            _ => self.update_provider(update.source(), &update, 0, 0, 0)?,
        }
        Ok(())
    }
}

impl BattleCore {
    pub(super) fn prepare_runtime_resources(&mut self) -> Result<(), GameError> {
        if self.runtime_facts.resources.is_none()
            && !self.game_data.resource_data.resources.is_empty()
        {
            self.runtime_facts.resources = Some(Box::new(
                crate::game::resource::runtime::RuntimeResources::new(
                    self.game_data.resource_data.clone(),
                )?,
            ));
        }
        Ok(())
    }

    pub(super) fn resource_context_for_source(
        &self,
        source: &RuntimeFactSource,
    ) -> Option<Arc<crate::game::resource::runtime::ResourceContext>> {
        source.resource_context().cloned().or_else(|| {
            source.action.as_ref().and_then(|action| {
                self.runtime_facts
                    .resources
                    .as_ref()
                    .and_then(|resources| resources.context(action).cloned())
            })
        })
    }

    pub(super) fn reserve_finite_source(
        &mut self,
        source: RuntimeFactSource,
    ) -> Result<RuntimeFactSource, GameError> {
        self.reserve_finite_source_with_parent(source, None)
    }

    pub(super) fn reserve_carrier_completion_source(
        &mut self,
        source: RuntimeFactSource,
    ) -> Result<RuntimeFactSource, GameError> {
        let parent = source.resource_delivery().cloned().or_else(|| {
            source
                .delivery
                .as_ref()
                .and_then(|ticket| ticket.resource.clone())
        });
        if self.runtime_facts.resources.is_some() && parent.is_none() {
            return Err(crate::game::resource::store::ResourceError::from(
                crate::game::resource::store::ResourceFaultCode::WrongLifetime,
            )
            .into_game_error());
        }
        self.reserve_finite_source_with_parent(source, parent.as_deref())
    }

    fn reserve_finite_source_with_parent(
        &mut self,
        mut source: RuntimeFactSource,
        parent: Option<&crate::game::resource::runtime::ResourceDelivery>,
    ) -> Result<RuntimeFactSource, GameError> {
        self.prepare_runtime_resources()?;
        let context = self.resource_context_for_source(&source);
        source.resource_capabilities = RuntimeFactSource::resource_capabilities(context, None)?;
        source.delivery = match source
            .action
            .as_ref()
            .filter(|_| source.allows_finite_children)
        {
            Some(action) => self
                .runtime_facts
                .delivery_with_resource_parent(action, parent)?,
            None => None,
        };
        if source.delivery.is_none() {
            if let Some(resources) = self.runtime_facts.resources.as_mut() {
                let (context, delivery) = match parent {
                    Some(parent) => resources
                        .finite_completion(source.resource_context().map(Arc::as_ref), parent)?,
                    None => resources.finite(source.resource_context().map(Arc::as_ref))?,
                };
                source.resource_capabilities =
                    RuntimeFactSource::resource_capabilities(Some(context), Some(delivery))?;
            }
        }
        Ok(source)
    }

    pub(super) fn close_source_resource_delivery(
        &mut self,
        delivery: Option<&crate::game::resource::runtime::ResourceDelivery>,
    ) -> Result<(), GameError> {
        if let (Some(resources), Some(delivery)) = (self.runtime_facts.resources.as_mut(), delivery)
        {
            resources.close_finite(delivery)?;
        }
        Ok(())
    }

    pub(super) fn enqueue_finite_effect(
        &mut self,
        event: crate::game::battle::enums::BattleEvent,
        legacy_source: Option<Arc<crate::game::battle::skill_source::SkillEffectExecutionSource>>,
        source: RuntimeFactSource,
    ) -> Result<(), GameError> {
        let source = self.reserve_finite_source(source)?;
        self.event_queue
            .push_completion(event, legacy_source, source);
        crate::authoring::control::poll("finite_effect_enqueued")
    }

    pub(super) fn enqueue_carrier_completion(
        &mut self,
        event: crate::game::battle::enums::BattleEvent,
        legacy_source: Option<Arc<crate::game::battle::skill_source::SkillEffectExecutionSource>>,
        source: RuntimeFactSource,
    ) -> Result<(), GameError> {
        let source = self.reserve_carrier_completion_source(source)?;
        self.event_queue
            .push_completion(event, legacy_source, source);
        crate::authoring::control::poll("carrier_completion_enqueued")
    }

    pub(super) fn cancel_pending_action(
        &mut self,
        action: Option<&ActionHandle>,
        time_ms: u64,
        reason: ActionCancelReason,
    ) {
        if let Some(action) = action {
            let _ =
                self.runtime_facts
                    .terminal(action, ActionTerminalKind::Cancelled(reason), time_ms);
        }
    }

    /// Bounds only R04-owned state and the added source/handle contribution in
    /// legacy carriers. It does not infer storage history for public units maps.
    pub(crate) fn retained_runtime_fact_extent(&self) -> Result<usize, GameError> {
        use crate::game::battle::enums::BattleEvent;
        let mut bytes = self
            .runtime_facts
            .usage
            .retained_bytes_bound
            .checked_add(std::mem::size_of::<RuntimeFactState>())
            .ok_or_else(|| overflow("fact_clone_extent"))?;
        let mut add = |extent: Option<usize>| -> Result<(), GameError> {
            bytes = extent
                .and_then(|n| bytes.checked_add(n))
                .ok_or_else(|| overflow("fact_clone_extent"))?;
            control::limit(
                Code::RetainedByteLimit,
                "fact_clone_extent",
                self.runtime_facts.limits.max_retained_bytes,
                Some(bytes),
            )
        };
        add(self.scenario_fact_binding_extent)?;
        if let Some(owner) = &self.relic_numeric_owner {
            add(Some(owner.retained_extent()))?;
        }
        if let Some(owner) = &self.effective_owner {
            add(Some(owner.retained_extent()))?;
        }
        add(self.sustained.retained_extent())?;
        crate::authoring::control::charge(
            self.independent_objects.entity_count() as u64,
            "independent_object_extent_scan",
        )?;
        add(self.independent_objects.retained_extent())?;
        crate::authoring::control::charge(
            self.spawn_kernel.request_count() as u64,
            "independent_request_extent_scan",
        )?;
        add(self.spawn_kernel.independent_retained_extent())?;
        if let Some(resources) = &self.runtime_facts.resources {
            add(Some(resources.retained_extent()?))?;
        }
        add(Some(128))?; // Fixed Arc owners for spawn registry and movement backend.
                         // A fixed contribution per unit/item does not inspect allocation history.
        let actor_capacity = self.runtime_facts.actors.rows.capacity();
        add(actor_capacity
            .checked_mul(std::mem::size_of::<(UnitInstanceId, ActorFactSummary)>())
            .and_then(|bytes| bytes.checked_add(if actor_capacity == 0 { 0 } else { 64 })))?;
        add(self
            .units
            .len()
            .checked_mul(super::types::RuntimeUnit::fact_slot_extent()))?;
        add(self
            .units
            .values()
            .filter(|unit| unit.equipment_slots.is_some())
            .count()
            .checked_mul(super::types::BattleEquipmentSlots::retained_extent()))?;
        for unit in self.units.values() {
            add(unit.independent_retained_extent())?;
            if let Some(use_state) = &unit.resonance_use {
                add(Some(use_state.skill_id.as_str().len() + 64))?;
            }
            if let Some(fragment) = &unit.fragment_source {
                add(fragment.retained_extent().and_then(|n| n.checked_add(64)))?;
            }
        }
        add(self
            .items
            .len()
            .checked_mul(std::mem::size_of::<Option<uuid::Uuid>>()))?;
        if !self.independent_objects.objects.is_empty() {
            crate::authoring::control::charge(
                self.scenario_runtime.spawned.unit_refs.len() as u64,
                "object_adapter_reference_extent",
            )?;
            for (unit_ref, id) in &self.scenario_runtime.spawned.unit_refs {
                if self.independent_objects.objects.contains_key(id) {
                    add(unit_ref.0.capacity().checked_add(
                        16 * (std::mem::size_of::<(
                            crate::game::battle::scenario::ScenarioUnitRef,
                            UnitInstanceId,
                        )>() + 16),
                    ))?;
                }
            }
        }
        add(self.runtime_fact_source.retained_extent())?;
        if let Some(record) = self
            .authoring_observation
            .as_ref()
            .and_then(|owner| owner.latest_amount.as_ref())
        {
            add(Some(record.retained_extent()))?;
        }
        let carriers = self
            .event_queue
            .len()
            .checked_add(self.buffs.len())
            .and_then(|n| n.checked_add(self.sustained.len()))
            .and_then(|n| n.checked_add(self.projectiles.len()))
            .and_then(|n| n.checked_add(self.active_projectiles.len()))
            .and_then(|n| n.checked_add(self.active_areas.len()))
            .and_then(|n| n.checked_add(self.active_skill_casts.len()))
            .ok_or_else(|| overflow("source_carrier_count"))?;
        crate::authoring::control::charge(carriers as u64, "source_carrier_scan")?;
        control::limit(
            Code::CloneLimit,
            "source_carrier_count",
            self.runtime_facts.limits.max_clone_slots,
            Some(carriers),
        )?;
        add(self.event_queue.slot_extent())?;
        for queued in self.event_queue.iter_queued() {
            add(queued.fact_source.retained_extent())?;
            add(Some(queued.event.payload_extent()))?;
            match queued.event.view().as_ref() {
                BattleEvent::AttackResolve {
                    source_snapshot, ..
                }
                | BattleEvent::SkillProjectileImpact {
                    source_snapshot, ..
                } => add(source_snapshot.fact_retained_extent())?,
                _ => {}
            }
        }
        for buff in self.buffs.values() {
            add(buff.latest_application_source.retained_extent())?;
        }
        for projectile in self.projectiles.values() {
            if let Some(spatial) = &projectile.spatial {
                add(spatial
                    .hit_unit_ids
                    .capacity()
                    .checked_mul(std::mem::size_of::<crate::game::battle::ids::UnitInstanceId>()))?;
            }
        }
        for source in self
            .projectiles
            .values()
            .map(|p| &p.source_snapshot)
            .chain(self.active_projectiles.values().map(|p| &p.source_snapshot))
            .chain(self.active_areas.values().map(|p| &p.source_snapshot))
        {
            add(source.fact_retained_extent())?;
        }
        for area in self.active_areas.values() {
            add(area.effective_history_extent())?;
        }
        for cast in self.active_skill_casts.values() {
            if let Some(flow) = &cast.flow {
                add(flow.effect_scope_extent())?;
                add(flow
                    .nodes
                    .capacity()
                    .checked_mul(std::mem::size_of::<super::skill_runtime::flow::NodeState>())
                    .and_then(|bytes| {
                        bytes.checked_add(
                            std::mem::size_of::<super::skill_runtime::flow::FlowState>(),
                        )
                    }))?;
            }
            add(cast.fact_source.retained_extent())?;
            if let Some(source) = &cast.execution_source {
                add(std::mem::size_of_val(source.as_ref())
                    .checked_add(source.skill_id.as_str().len()))?;
            }
        }
        #[cfg(feature = "runtime-observation")]
        {
            crate::runtime_observation::extent(bytes);
            crate::runtime_observation::sample(self.observation_usage());
        }
        Ok(bytes)
    }

    pub(crate) fn preflight_runtime_fact_clone(&self) -> Result<(), GameError> {
        use crate::authoring::{clone_shape::CloneShape, control::CloneMeter};
        let bytes = self.retained_runtime_fact_extent()?;
        control::seed_retained_extent(bytes)?;
        // Retained witnesses include shared Arc payload extents; native Clone
        // copies only their handles. Price the closed state shape separately
        // instead of charging those immutable payloads as deep copies each tick.
        let batch_native_scan = control::is_active() && !crate::authoring::control::is_active();
        let mut meter = if batch_native_scan {
            CloneMeter::for_native_scan()?
        } else {
            CloneMeter::default()
        };
        self.scenario_fact_binding_extent
            .visit_clone_shape(&mut meter)?;
        self.scenario.visit_clone_shape(&mut meter)?;
        self.scenario_runtime
            .spawned
            .visit_clone_shape(&mut meter)?;
        self.movement_backend.visit_clone_shape(&mut meter)?;
        self.runtime_facts.visit_clone_shape(&mut meter)?;
        self.runtime_strike_scope.visit_clone_shape(&mut meter)?;
        // Retained source extent includes shared authored/cast payloads. Their
        // pointees survive a clone but are not copied by it. Visit only the
        // added carrier-owned fields with the same closed ownership rules.
        meter.scan_storage(self.units.len().checked_mul(12))?;
        meter.scan_storage(self.items.len().checked_mul(3))?;
        self.runtime_fact_source.visit_clone_shape(&mut meter)?;
        // BinaryHeap::clone copies live slots into a len-sized buffer; its
        // source's spare capacity belongs only to retained extent above.
        meter.scan_storage(self.event_queue.len().checked_add(1))?;
        let damage_source = |source: &crate::game::battle::damage::DamageSourceSnapshot,
                             meter: &mut CloneMeter|
         -> Result<(), GameError> {
            source.fact_source.visit_clone_shape(meter)?;
            source.basic_lifesteal.visit_clone_shape(meter)?;
            source.on_attack_bonus_damage.visit_clone_shape(meter)?;
            source.on_attack_modifier_sources.visit_clone_shape(meter)
        };
        for queued in self.event_queue.iter_queued() {
            queued.fact_source.visit_clone_shape(&mut meter)?;
            queued
                .close_delivery_on_dispatch
                .visit_clone_shape(&mut meter)?;
            queued.event.visit_clone_shape(&mut meter)?;
        }
        for buff in self.buffs.values() {
            buff.latest_application_source
                .visit_clone_shape(&mut meter)?;
        }
        for projectile in self.projectiles.values() {
            projectile.spatial.visit_clone_shape(&mut meter)?;
        }
        for source in self
            .projectiles
            .values()
            .map(|p| &p.source_snapshot)
            .chain(self.active_projectiles.values().map(|p| &p.source_snapshot))
            .chain(self.active_areas.values().map(|p| &p.source_snapshot))
        {
            damage_source(source, &mut meter)?;
        }
        for cast in self.active_skill_casts.values() {
            cast.flow.visit_clone_shape(&mut meter)?;
            cast.fact_source.visit_clone_shape(&mut meter)?;
            cast.execution_source.visit_clone_shape(&mut meter)?;
        }
        let slots = meter
            .native_copy_slots()
            .ok_or_else(|| overflow("fact_clone_slots"))?;
        if batch_native_scan {
            control::charge(meter.native_scan_slots() as u64, "owned_clone_scan")?;
        }
        control::charge_clone(slots, "fact_clone_slots")
    }

    /// Preserve the deployment entry path; only development operations clone
    /// a settings handle to outlive the mutable owner borrow.
    pub(super) fn run_profile_budget<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let limits = self.runtime_facts.limits;
        if self
            .game_data
            .runtime_safety()
            .development_measurement()
            .is_none()
        {
            #[cfg(feature = "runtime-observation")]
            let warning = (!control::is_active()).then(|| {
                self.game_data
                    .runtime_safety()
                    .nominal_native_runtime()
                    .max_retained_bytes
            });
            return control::run(limits, || {
                #[cfg(feature = "runtime-observation")]
                if let Some(warning) = warning {
                    control::observe_profile_warning(warning);
                }
                operation(self)
            });
        }
        let settings = Arc::clone(self.game_data.runtime_safety());
        control::run_with_profile(limits, &settings, || operation(self))
    }

    pub(super) fn runtime_transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        self.run_profile_budget(|core| {
            let mut candidate = crate::authoring::clone_shape::controlled_clone(core)?;
            candidate.prepare_runtime_resources()?;
            candidate.event_log.prepare_authoring_write()?;
            let value = operation(&mut candidate)?;
            candidate.runtime_facts.release_empty_action_storage();
            candidate.retained_runtime_fact_extent()?;
            crate::authoring::control::poll("runtime_transaction_commit")?;
            *core = candidate;
            Ok(value)
        })
    }

    pub(super) fn runtime_timestamp<T>(
        &mut self,
        state: &mut super::sim::BattleExecutionState,
        operation: impl FnOnce(&mut Self, &mut super::sim::BattleExecutionState) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        self.runtime_timestamp_reusing(state, &mut None, operation)
    }

    pub(super) fn runtime_timestamp_reusing<T>(
        &mut self,
        state: &mut super::sim::BattleExecutionState,
        buffer: &mut Option<(
            std::collections::HashMap<UnitInstanceId, super::types::RuntimeUnit>,
            crate::owned_collections::MapCloneHistory,
        )>,
        operation: impl FnOnce(&mut Self, &mut super::sim::BattleExecutionState) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        self.run_profile_budget(|core| {
            let native = !crate::authoring::control::is_active();
            let mut candidate = if native {
                core.preflight_runtime_fact_clone()?;
                // The buffer belongs to this call's earlier native clone. Only
                // value-only transitions can reuse it: no table reconstruction,
                // insertion, deletion or hasher replacement happens here.
                crate::authoring::control::charge(
                    core.units.len() as u64,
                    "timestamp_unit_lineage",
                )?;
                match buffer.take() {
                    Some((units, history))
                        if history.same_unsaturated_generation(&core.units_clone_history)
                            && units.len() == core.units.len()
                            && units.keys().eq(core.units.keys()) =>
                    {
                        core.clone_reusing_timestamp_units(units, history)
                    }
                    _ => core.clone(),
                }
            } else {
                crate::authoring::clone_shape::controlled_clone(core)?
            };
            let mut execution = state.clone();
            candidate.prepare_runtime_resources()?;
            candidate.event_log.prepare_authoring_write()?;
            let value = operation(&mut candidate, &mut execution)?;
            candidate.runtime_facts.release_empty_action_storage();
            candidate.retained_runtime_fact_extent()?;
            crate::authoring::control::poll("runtime_timestamp_commit")?;
            let previous = std::mem::replace(core, candidate);
            *state = execution;
            if native {
                *buffer = Some((previous.units, previous.units_clone_history));
            }
            Ok(value)
        })
    }

    pub fn query_action(&self, handle: &ActionHandle) -> Result<ActionSnapshot, ActionQueryError> {
        self.runtime_facts.query(handle)
    }
    pub fn take_settled_action_result(
        &mut self,
        handle: &ActionHandle,
    ) -> Result<Arc<SettledActionReceipt>, ActionQueryError> {
        if self.corrosion_freeze.is_some() {
            return Err(ActionQueryError::ForcedPause);
        }
        self.runtime_facts.take(handle)
    }
    pub fn runtime_fact_summary(&self) -> &FactSummary {
        &self.runtime_facts.summary
    }
    pub fn runtime_fact_usage(&self) -> RuntimeFactUsage {
        self.runtime_facts.usage
    }
    pub(crate) fn runtime_fact_limits(&self) -> RuntimeFactLimits {
        self.runtime_facts.limits
    }
    pub fn actor_fact_summary(&self, actor: UnitInstanceId) -> Option<&ActorFactSummary> {
        self.runtime_facts.actors.get(&actor)
    }
    pub fn provider_fact_summary(
        &self,
        provider: &EquipmentSourceSnapshot,
    ) -> Option<&ProviderFactSummary> {
        self.runtime_facts.providers.get(provider)
    }

    pub fn with_runtime_fact_limits(
        mut self,
        limits: RuntimeFactLimits,
    ) -> Result<Self, GameError> {
        limits.validate()?;
        if self.runtime_facts.usage.accepted_actions != 0 || !self.units.is_empty() {
            return Err(GameError::InvalidAction);
        }
        self.runtime_facts = RuntimeFactState::new(limits);
        Ok(self)
    }
}

impl BattleCore {
    pub(super) fn prepare_damage_fact(
        &mut self,
        result: &crate::game::battle::damage::DamageResult,
        protection: crate::game::battle::runtime_facts::ProtectionReceipt,
        admitted: AdmittedStrike,
        snapshot: &crate::game::battle::damage::DamageSourceSnapshot,
        on_hit: &[crate::game::battle::cooldown::SourcedEffect],
        incoming: crate::game::battle::damage::DamageModifiers,
        hp_before: u32,
        time_ms: u64,
    ) -> Result<(Arc<DamageFact>, bool, bool), GameError> {
        use crate::game::{battle::cooldown::CooldownSource, stats::Effect};
        let hp_after = hp_before.saturating_sub(result.final_damage);
        let (strike, first_segment, first, first_positive) =
            self.fact_strike_segments(admitted, result.breakdown.len(), hp_after < hp_before)?;
        let source = self.runtime_fact_source.observation();
        let component_source = |index: usize| {
            let mut value = source.clone();
            if let Some(Some(component)) = result.component_sources.get(index) {
                value.actor = Some(component.actor);
                value.equipment = component.equipment;
                value.authored = None;
            }
            value
        };
        let modifier_source = |actor, equipment| RuntimeFactSource {
            actor: Some(actor),
            equipment,
            authored: None,
            ..source.clone()
        };
        let support = snapshot.fact_source.regular_support();
        let support_count = support.map_or(0, |support| support.contributors.len());
        let modifier_count = snapshot
            .on_attack_modifier_sources
            .len()
            .checked_add(support_count)
            .and_then(|count| {
                count.checked_add(
                    on_hit
                        .iter()
                        .filter(|v| matches!(v.effect, Effect::ModifyDamage(_)))
                        .count(),
                )
            });
        control::limit(
            Code::StrikeSegmentLimit,
            "damage_modifier_sources",
            self.runtime_facts.limits.max_strike_segments,
            modifier_count,
        )?;
        let mut extent = source
            .retained_extent()
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<DamageFact>()))
            .and_then(|bytes| bytes.checked_add(64));
        for index in 0..result.breakdown.len() {
            crate::authoring::control::charge(1, "damage_component_extent")?;
            extent = extent
                .and_then(|n| n.checked_add(std::mem::size_of::<DamageSegmentFact>() + 64))
                .and_then(|n| {
                    component_source(index)
                        .retained_extent()
                        .and_then(|e| n.checked_add(e))
                });
        }
        // Contributor sources have no authored payload and otherwise share the
        // same bounded cast/action metadata. Count each owner conservatively.
        let modifier_extent = modifier_source(result.attacker_id, None)
            .retained_extent()
            .and_then(|n| n.checked_mul(modifier_count.unwrap_or(0)));
        extent = extent.and_then(|n| modifier_extent.and_then(|e| n.checked_add(e)));
        if let Some(support) = support {
            for contributor in &support.contributors {
                extent = extent.and_then(|bytes| {
                    contributor
                        .source
                        .retained_extent()
                        .and_then(|n| bytes.checked_add(n))
                });
            }
        }
        extent = extent.and_then(|n| protection.retained_extent().and_then(|p| n.checked_add(p)));
        let extent = self.runtime_facts.reserve_fact_storage(extent)?;
        let shared_source = self.runtime_facts.observed_source(&source)?;
        let mut segments = Vec::with_capacity(result.breakdown.len());
        for (index, component) in result.breakdown.iter().enumerate() {
            segments.push(DamageSegmentFact {
                ordinal: first_segment + index as u32,
                damage_type: component.damage_type,
                raw_damage: component.raw_damage,
                calculated_damage: component.final_damage,
                source: {
                    let source = component_source(index);
                    if source == *shared_source {
                        Arc::clone(&shared_source)
                    } else {
                        self.runtime_facts.observed_source(&source)?
                    }
                },
            });
        }
        let mut modifier_sources =
            Vec::with_capacity(modifier_count.expect("admitted modifier count"));
        for contributor in &snapshot.on_attack_modifier_sources {
            let actor = match contributor.source {
                CooldownSource::Unit { unit_instance_id } => unit_instance_id,
                _ => snapshot.source_id,
            };
            modifier_sources.push(modifier_source(actor, contributor.equipment_source));
        }
        for contributor in on_hit
            .iter()
            .filter(|v| matches!(v.effect, Effect::ModifyDamage(_)))
        {
            let actor = match contributor.source {
                CooldownSource::Unit { unit_instance_id } => unit_instance_id,
                _ => result.target_id,
            };
            modifier_sources.push(modifier_source(actor, contributor.equipment_source));
        }
        if let Some(support) = support {
            for contributor in &support.contributors {
                crate::authoring::control::charge(1, "aura_damage_contributor")?;
                modifier_sources.push(contributor.source.clone());
            }
        }
        let ordinal = self.runtime_facts.fact_ordinal()?;
        Ok((
            Arc::new(DamageFact {
                retained_extent: extent,
                ordinal,
                time_ms,
                strike,
                target: result.target_id,
                source: shared_source,
                requested_damage: result.raw_damage,
                protection,
                hp_before,
                hp_after,
                segments,
                modifier_sources,
                unattributed_modifiers: snapshot.modifiers.merge(incoming),
            }),
            first,
            first_positive,
        ))
    }

    pub(super) fn record_heal_fact(
        &mut self,
        source_actor: Option<UnitInstanceId>,
        target: UnitInstanceId,
        requested_delta: i64,
        hp_before: u32,
        hp_after: u32,
        time_ms: u64,
        reason: crate::game::battle::event_log::HpChangeReason,
    ) -> Result<(), GameError> {
        let source = RuntimeFactSource {
            actor: source_actor,
            ..self.runtime_fact_source.observation()
        };
        let extent = self.runtime_facts.reserve_fact_storage(
            source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<HealFact>())),
        )?;
        let ordinal = self.runtime_facts.fact_ordinal()?;
        let value = Arc::new(HealFact {
            retained_extent: extent,
            ordinal,
            time_ms,
            target,
            source,
            requested_delta,
            hp_before,
            hp_after,
            reason,
        });
        self.runtime_facts
            .record_fact(FactUpdate::Heal(value.clone()))?;
        self.observe_effect_heal_result(&value.source, i64::from(hp_after) - i64::from(hp_before));
        self.runtime_facts.release_fact_storage(extent);
        Ok(())
    }
}

pub(super) struct PreparedDeath {
    pub(super) cause: crate::game::battle::runtime_facts::DeathCause,
    snapshot: PredeathSnapshot,
    source: RuntimeFactSource,
    extent: usize,
}
impl BattleCore {
    pub(super) fn prepare_death_fact(
        &mut self,
        actor: Option<UnitInstanceId>,
        victim: UnitInstanceId,
        time_ms: u64,
    ) -> Result<Option<PreparedDeath>, GameError> {
        // Independent objects retire without unit death facts or death procs.
        if self.independent_objects.objects.contains_key(&victim) {
            return Ok(None);
        }
        let Some(target) = self.units.get(&victim).filter(|unit| unit.is_active()) else {
            return Ok(None);
        };
        crate::authoring::control::charge(self.buffs.len() as u64, "predeath_buff_scan")?;
        let source = RuntimeFactSource {
            actor,
            ..self.runtime_fact_source.observation()
        };
        let mut count = 0usize;
        let mut extent = source
            .retained_extent()
            .and_then(|n| n.checked_add(std::mem::size_of::<DeathFact>()));
        for (key, buff) in &self.buffs {
            if key.target_instance_id != victim || time_ms > buff.expires_at_ms {
                continue;
            }
            count = count
                .checked_add(1)
                .ok_or_else(|| overflow("predeath_buff_count"))?;
            control::limit(
                Code::PredeathBuffLimit,
                "predeath_buff_count",
                self.runtime_facts.limits.max_predeath_buffs,
                Some(count),
            )?;
            extent = extent
                .and_then(|n| n.checked_add(std::mem::size_of::<PredeathBuffSnapshot>()))
                .and_then(|n| {
                    buff.latest_application_source
                        .retained_extent()
                        .and_then(|e| n.checked_add(e))
                });
        }
        let extent = self.runtime_facts.reserve_fact_storage(extent)?;
        let mut buffs = Vec::with_capacity(count);
        for (key, buff) in &self.buffs {
            if key.target_instance_id != victim || time_ms > buff.expires_at_ms {
                continue;
            }
            buffs.push(PredeathBuffSnapshot {
                buff_id: key.buff_id,
                caster: key.caster_instance_id,
                stacks: buff.stacks,
                expires_at_ms: buff.expires_at_ms,
                latest_application_source: buff.latest_application_source.observation(),
            });
        }
        buffs.sort_by_key(|buff| (buff.buff_id.as_u64(), buff.caster));
        Ok(Some(PreparedDeath {
            cause: crate::game::battle::runtime_facts::DeathCause::Ordinary,
            snapshot: PredeathSnapshot {
                victim,
                owner: target.owner,
                hp_before_lethal: target.stats.current_health,
                position: Some(target.body.projected_tile()),
                buffs,
            },
            source,
            extent,
        }))
    }
    pub(super) fn record_death_fact(
        &mut self,
        prepared: PreparedDeath,
        lethal_damage: Option<Arc<DamageFact>>,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let additional = lethal_damage
            .as_ref()
            .map_or(0, |value| value.retained_extent);
        self.runtime_facts.reserve_fact_storage(Some(additional))?;
        let extent = prepared
            .extent
            .checked_add(additional)
            .ok_or_else(|| overflow("death_extent"))?;
        let ordinal = self.runtime_facts.fact_ordinal()?;
        let fact = Arc::new(DeathFact {
            cause: prepared.cause,
            retained_extent: extent,
            ordinal,
            time_ms,
            source: prepared.source,
            predeath: prepared.snapshot,
            lethal_damage,
        });
        self.runtime_facts.record_fact(FactUpdate::Death(fact))?;
        self.runtime_facts.release_fact_storage(extent);
        Ok(())
    }
}

impl BattleCore {
    pub(crate) fn record_resource_fact(
        &mut self,
        owner: ResourceOwner,
        resource: FactResource,
        payment: Option<CostReason>,
        source: RuntimeFactSource,
        requested_delta: i64,
        before: u32,
        after: u32,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let source = source.observation();
        let size = if payment.is_some() {
            std::mem::size_of::<CostFact>()
        } else {
            std::mem::size_of::<ResourceChangeFact>()
        };
        let extra = match &resource {
            FactResource::Typed { id, .. } => {
                std::mem::size_of_val(id.as_ref()) + 64 + id.as_str().len()
            }
            _ => 0,
        };
        let extent = self.runtime_facts.reserve_fact_storage(
            source
                .retained_extent()
                .and_then(|n| n.checked_add(size))
                .and_then(|n| n.checked_add(extra)),
        )?;
        let ordinal = self.runtime_facts.fact_ordinal()?;
        let update = match payment {
            Some(reason) => FactUpdate::Cost(Arc::new(CostFact {
                retained_extent: extent,
                ordinal,
                time_ms,
                owner,
                resource,
                reason,
                source,
                requested_delta,
                before,
                after,
            })),
            None => FactUpdate::ResourceChange(Arc::new(ResourceChangeFact {
                retained_extent: extent,
                ordinal,
                time_ms,
                owner,
                resource,
                source,
                requested_delta,
                before,
                after,
            })),
        };
        self.runtime_facts.record_fact(update.clone())?;
        if let FactUpdate::ResourceChange(value) = &update {
            self.observe_effect_resource_result(
                &value.source,
                &value.resource,
                value.actual_delta(),
            );
        }
        self.runtime_facts.release_fact_storage(extent);
        Ok(())
    }
}

impl BattleCore {
    pub(crate) fn record_tactical_resource_signal(
        &mut self,
        source: RuntimeFactSource,
        amount: i32,
        before: u32,
        after: u32,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let ticket = source.delivery.clone();
        self.record_resource_fact(
            ResourceOwner::Side(crate::game::enums::Side::Player),
            FactResource::TacticalPoints,
            None,
            source,
            i64::from(amount),
            before,
            after,
            time_ms,
        )?;
        if let Some(ticket) = ticket {
            self.runtime_facts.close_delivery(&ticket, time_ms)?;
        }
        Ok(())
    }
}

impl<'a> RuntimeFactView<'a> {
    pub fn summary(self) -> &'a FactSummary {
        self.live.map_or_else(
            || &self.snapshot.expect("fact view owner").summary,
            |live| &live.summary,
        )
    }
    pub fn latest_settled_action(self) -> Option<&'a ActionSnapshot> {
        match self.live {
            Some(live) => live.latest_settled_action.as_deref(),
            None => self
                .snapshot
                .expect("fact view owner")
                .latest_settled_action
                .as_deref(),
        }
    }
    pub fn usage(self) -> RuntimeFactUsage {
        self.live.map_or_else(
            || self.snapshot.expect("fact view owner").usage,
            |live| live.usage,
        )
    }
    pub fn active_actions(self) -> impl Iterator<Item = &'a ActionSnapshot> {
        self.live
            .into_iter()
            .flat_map(|live| live.active.values().map(|action| &action.snapshot))
            .chain(
                self.snapshot
                    .into_iter()
                    .flat_map(|snapshot| snapshot.active_actions.iter()),
            )
    }
    pub fn actors(self) -> impl Iterator<Item = (&'a UnitInstanceId, &'a ActorFactSummary)> {
        self.live
            .into_iter()
            .flat_map(|live| live.actors.iter())
            .chain(
                self.snapshot.into_iter().flat_map(|snapshot| {
                    snapshot.actors.iter().map(|row| (&row.actor, &row.summary))
                }),
            )
    }
    pub fn providers(
        self,
    ) -> impl Iterator<Item = (&'a EquipmentSourceSnapshot, &'a ProviderFactSummary)> {
        self.live
            .into_iter()
            .flat_map(|live| live.providers.iter())
            .chain(self.snapshot.into_iter().flat_map(|snapshot| {
                snapshot
                    .providers
                    .iter()
                    .map(|row| (&row.provider, &row.summary))
            }))
    }
}
impl serde::Serialize for RuntimeFactView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Actions<'a>(RuntimeFactView<'a>);
        impl serde::Serialize for Actions<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(Some(self.0.usage().active_actions))?;
                for action in self.0.active_actions() {
                    seq.serialize_element(action)?;
                }
                seq.end()
            }
        }
        struct Actors<'a>(RuntimeFactView<'a>);
        impl serde::Serialize for Actors<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #[derive(serde::Serialize)]
                struct Row<'a> {
                    actor: &'a UnitInstanceId,
                    summary: &'a ActorFactSummary,
                }
                let mut seq = serializer.serialize_seq(Some(self.0.usage().actor_summaries))?;
                for (actor, summary) in self.0.actors() {
                    seq.serialize_element(&Row { actor, summary })?;
                }
                seq.end()
            }
        }
        struct Providers<'a>(RuntimeFactView<'a>);
        impl serde::Serialize for Providers<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #[derive(serde::Serialize)]
                struct Row<'a> {
                    provider: &'a EquipmentSourceSnapshot,
                    summary: &'a ProviderFactSummary,
                }
                let mut seq = serializer.serialize_seq(Some(self.0.usage().provider_summaries))?;
                for (provider, summary) in self.0.providers() {
                    seq.serialize_element(&Row { provider, summary })?;
                }
                seq.end()
            }
        }
        let mut value = serializer.serialize_struct("RuntimeFactSnapshot", 6)?;
        value.serialize_field("summary", self.summary())?;
        value.serialize_field("latest_settled_action", &self.latest_settled_action())?;
        value.serialize_field("active_actions", &Actions(*self))?;
        value.serialize_field("actors", &Actors(*self))?;
        value.serialize_field("providers", &Providers(*self))?;
        value.serialize_field("usage", &self.usage())?;
        value.end()
    }
}

impl BattleCore {
    pub fn runtime_fact_view(&self) -> RuntimeFactView<'_> {
        RuntimeFactView {
            live: Some(&self.runtime_facts),
            snapshot: None,
        }
    }
    pub fn runtime_fact_snapshot(&self) -> Result<Arc<RuntimeFactSnapshot>, GameError> {
        let facts = &self.runtime_facts;
        let rows = facts
            .active
            .len()
            .checked_add(facts.actors.len())
            .and_then(|n| n.checked_add(facts.providers.len()))
            .ok_or_else(|| overflow("fact_projection_rows"))?;
        crate::authoring::control::charge(rows as u64, "fact_projection_rows")?;
        let extent = || -> Option<usize> {
            let mut bytes = std::mem::size_of::<RuntimeFactSnapshot>()
                .checked_add(summary_extent(&facts.summary)?)?;
            if let Some(action) = &facts.latest_settled_action {
                bytes = bytes.checked_add(action_snapshot_extent(action)?)?;
            }
            for action in facts.active.values() {
                bytes = bytes.checked_add(action_snapshot_extent(&action.snapshot)?)?;
            }
            for actor in facts.actors.values() {
                bytes = bytes
                    .checked_add(std::mem::size_of::<ActorFactObservation>())?
                    .checked_add(summary_extent(&actor.caused)?)?
                    .checked_add(summary_extent(&actor.received)?)?;
                if let Some(action) = &actor.latest_settled_action {
                    bytes = bytes.checked_add(action_snapshot_extent(action)?)?;
                }
            }
            for provider in facts.providers.values() {
                bytes = bytes
                    .checked_add(std::mem::size_of::<ProviderFactObservation>())?
                    .checked_add(provider_extent(provider)?)?;
            }
            Some(bytes)
        }();
        control::limit(
            Code::RetainedByteLimit,
            "fact_projection_extent",
            facts.limits.max_retained_bytes,
            extent,
        )?;
        control::admit_retained_growth_with_owner_limits(
            0,
            extent,
            "fact_projection_allocate",
            facts.limits,
        )?;
        let extent = extent.expect("admitted projection extent");
        crate::authoring::control::charge((extent / 8) as u64, "fact_projection_extent")?;
        let snapshot = RuntimeFactSnapshot {
            retained_extent: extent,
            summary: facts.summary.clone(),
            latest_settled_action: facts.latest_settled_action.clone(),
            active_actions: facts
                .active
                .values()
                .map(|action| action.snapshot.clone())
                .collect(),
            actors: facts
                .actors
                .iter()
                .map(|(actor, summary)| ActorFactObservation {
                    actor: *actor,
                    summary: summary.clone(),
                })
                .collect(),
            providers: facts
                .providers
                .iter()
                .map(|(provider, summary)| ProviderFactObservation {
                    provider: *provider,
                    summary: summary.clone(),
                })
                .collect(),
            usage: facts.usage,
        };
        Ok(Arc::new(snapshot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmrp_core() -> (
        BattleCore,
        Arc<crate::runtime_safety::RuntimeSafetySettings>,
    ) {
        let mut source =
            String::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON.to_vec()).unwrap();
        let end = source.rfind(')').unwrap();
        source.insert_str(
            end,
            "development_measurement: Some((revision: 1, emergency_retained_bytes: 16777216)),\n",
        );
        let settings = Arc::new(
            crate::runtime_safety::RuntimeSafetySettings::from_ron(
                "trusted core fixture",
                source.as_bytes(),
            )
            .unwrap(),
        );
        let data = Arc::new(
            crate::game::data::GameDataBuilder::empty()
                .build_with_runtime_safety(settings.clone())
                .unwrap(),
        );
        let core = BattleCore::new_from_scenario(
            crate::game::battle::scenario::BattleScenario::empty((2, 2)),
            data,
            41,
        );
        (core, settings)
    }

    #[cfg(feature = "runtime-observation")]
    #[test]
    fn cmrp_raw_outer_nested_default_core_keeps_unknown_warning() {
        use crate::runtime_observation as observation;
        let mut core = BattleCore::new_from_scenario(
            crate::game::battle::scenario::BattleScenario::empty((2, 2)),
            crate::game::data::GameDataBuilder::empty().build_arc(),
            41,
        );
        let (sender, receiver) = observation::channel();
        let (mut observer, _) = sender.session(102);
        let guard = observer.begin(observation::Operation::HostTick, 0).unwrap();
        let result = control::run(Default::default(), || {
            core.run_profile_budget(|_| {
                control::seed_retained_extent(0)?;
                control::admit_retained_growth(0, Some(8), "raw_outer_nested_core")
            })
        })
        .map_err(crate::application::ApplicationError::from_domain);
        guard.finish(&result);
        result.unwrap();
        assert_eq!(
            receiver
                .try_recv()
                .unwrap()
                .retained
                .unwrap()
                .profile_warning_bytes,
            None
        );
        let guard = observer.begin(observation::Operation::HostTick, 0).unwrap();
        let result = core
            .run_profile_budget(|_| {
                control::seed_retained_extent(0)?;
                control::admit_retained_growth(0, Some(8), "normal_core_entry")
            })
            .map_err(crate::application::ApplicationError::from_domain);
        guard.finish(&result);
        result.unwrap();
        assert_eq!(
            receiver
                .try_recv()
                .unwrap()
                .retained
                .unwrap()
                .profile_warning_bytes,
            Some(8 * 1024 * 1024)
        );
    }

    #[test]
    fn cmrp_public_override_keeps_small_cap_and_rejects_raw_development_cap() {
        let (core, settings) = cmrp_core();
        assert_eq!(
            core.runtime_fact_limits().max_retained_bytes,
            16 * 1024 * 1024
        );
        assert!(matches!(
            core.clone()
                .with_runtime_fact_limits(settings.native_runtime()),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::InvalidLimits,
                site: "max_retained_bytes",
                limit: Some(8388608),
                requested: Some(16777216)
            }))
        ));
        let small = core
            .with_runtime_fact_limits(RuntimeFactLimits {
                max_retained_bytes: 1,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(small.runtime_fact_limits().max_retained_bytes, 1);
        assert_eq!(
            small.game_data.runtime_safety().effective_sha256(),
            settings.effective_sha256()
        );
        assert!(matches!(
            small.runtime_fact_snapshot(),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::RetainedByteLimit,
                site: "fact_projection_extent",
                limit: Some(1),
                ..
            }))
        ));
    }

    #[test]
    fn cmrp_actual_snapshot_uses_trusted_owner_outside_and_cumulative_budget_inside() {
        let (mut core, settings) = cmrp_core();
        // Representation boundary fixture, not an authored-content sample: the
        // admitted owner retains a large identifier to exercise a >8MiB view.
        let skill = SkillId::new("x".repeat(9 * 1024 * 1024));
        control::run_with_profile(core.runtime_fact_limits(), &settings, || {
            core.runtime_facts.accept(
                actor(),
                Some(&skill),
                RuntimeFactSource::default(),
                0,
                true,
            )?;
            Ok(())
        })
        .unwrap();
        assert!(!control::is_active());
        let snapshot = core.runtime_fact_snapshot().unwrap();
        assert!(snapshot.retained_extent > 8 * 1024 * 1024);
        assert!(snapshot.retained_extent < 16 * 1024 * 1024);
        assert_eq!(snapshot.active_actions.len(), 1);
        assert_eq!(
            snapshot.active_actions[0]
                .skill_id
                .as_ref()
                .unwrap()
                .as_str()
                .len(),
            9 * 1024 * 1024
        );
        let error = control::run(
            RuntimeFactLimits {
                max_retained_bytes: 4096,
                ..Default::default()
            },
            || core.runtime_fact_snapshot(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::RetainedByteLimit,
                site: "fact_projection_allocate",
                limit: Some(4096),
                ..
            })
        ));
        assert_eq!(
            core.runtime_fact_snapshot().unwrap().retained_extent,
            snapshot.retained_extent
        );
    }

    #[test]
    fn actor_rows_grow_in_bounded_steps_and_cow_copies_only_live_rows() {
        let mut state = RuntimeFactState::default();
        for id in [4_u128, 2, 3, 1] {
            state.actor(uuid::Uuid::from_u128(id).into()).unwrap();
        }
        assert_eq!(state.actors.len(), 4);
        assert_eq!(state.actors.rows.capacity(), 4);
        assert_eq!(
            state.actors.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            (1_u128..=4)
                .map(|id| uuid::Uuid::from_u128(id).into())
                .collect::<Vec<_>>()
        );
        let mut candidate = state.clone();
        candidate.actor(uuid::Uuid::from_u128(5).into()).unwrap();
        assert_eq!(state.actors.len(), 4);
        assert_eq!(candidate.actors.len(), 5);
        assert_eq!(candidate.actors.rows.capacity(), 6);
        assert!(!Arc::ptr_eq(&state.actors, &candidate.actors));
        let mut branch = candidate.clone();
        writable_actor_summaries(&mut branch.actors)
            .unwrap()
            .get_mut(&uuid::Uuid::from_u128(5).into())
            .unwrap()
            .caused
            .damage_mutations = 1;
        assert_eq!(branch.actors.rows.capacity(), 5);
        assert_eq!(candidate.actors.rows.capacity(), 6);
        assert_eq!(
            candidate
                .actors
                .get(&uuid::Uuid::from_u128(5).into())
                .unwrap()
                .caused
                .damage_mutations,
            0
        );
        writable_actor_summaries(&mut candidate.actors)
            .unwrap()
            .clear();
        assert_eq!(candidate.actors.rows.capacity(), 0);
        assert_eq!(state.actors.rows.capacity(), 4);
    }

    #[test]
    fn actor_row_growth_uses_a_fitting_minimum_without_poisoning_the_attempt() {
        let mut rows = ActorSummaries::default();
        for id in 1_u128..=4 {
            rows.insert(
                uuid::Uuid::from_u128(id).into(),
                ActorFactSummary::default(),
                100,
            )
            .unwrap();
        }
        let row_bytes = std::mem::size_of::<(UnitInstanceId, ActorFactSummary)>();
        let minimum_bytes = 5 * row_bytes + 64;
        control::run(
            RuntimeFactLimits {
                max_retained_bytes: minimum_bytes,
                ..Default::default()
            },
            || {
                control::seed_retained_extent(0)?;
                rows.insert(
                    uuid::Uuid::from_u128(5).into(),
                    ActorFactSummary::default(),
                    100,
                )?;
                control::poll()
            },
        )
        .unwrap();
        assert_eq!(rows.rows.capacity(), 5);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn runtime_strike_admission_precedes_formula_limits_and_rolls_back() {
        use crate::game::ability::{DeliveryDef, SkillEffectDef, SkillTarget, UnitTargetRule};
        use crate::game::battle::{
            cooldown::CooldownSource,
            core::{
                damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
                tests::{native_action_fixture, single_step_skill},
            },
            damage::{DamageBonusSnapshot, DamageModifiers, DamageSource, DamageType},
        };
        let skill = single_step_skill(
            "strike_admission_boundary",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            3,
            100,
            DeliveryDef::Instant,
            vec![SkillEffectDef::Damage {
                amount: 1,
                damage_type: DamageType::True,
            }],
        );
        for policy in [
            CommittedDamagePolicy::GeneralCommand,
            CommittedDamagePolicy::BasicAttack {
                occurrence_id: uuid::Uuid::from_u128(0xFA10),
                occurrence_time_ms: 0,
            },
        ] {
            let (mut core, _, caster, target, _) = native_action_fixture(
                skill.clone(),
                RuntimeFactLimits {
                    max_strike_segments: 1,
                    ..Default::default()
                },
            );
            let mut snapshot = core
                .damage_source_snapshot_for_unit(
                    caster,
                    target,
                    DamageSource::Ability,
                    DamageType::True,
                    1,
                    DamageModifiers::default(),
                    0,
                    0,
                    false,
                )
                .unwrap();
            snapshot.on_attack_bonus_damage.push(DamageBonusSnapshot {
                source: CooldownSource::Unit {
                    unit_instance_id: caster,
                },
                equipment_source: None,
                flat: 1,
                percent: 0,
                damage_type: DamageType::True,
            });
            let input = CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: 0,
                policy,
            };
            core.runtime_facts.next_strike = u64::MAX;
            let before = core.transaction_debug_snapshot();
            let error = core
                .runtime_transaction(|candidate| {
                    Ok(candidate.execute_committed_damage(input.clone()))
                })
                .unwrap_err();
            // Both identity and formula capacity are exhausted. The first fault
            // establishes that direct identity exists before formula admission.
            assert!(
                matches!(
                    error,
                    GameError::BattleRuntimeFault(BattleRuntimeFault {
                        code: Code::CounterOverflow,
                        site: "strike_ordinal",
                        ..
                    })
                ),
                "{error:?}"
            );
            assert_eq!(core.transaction_debug_snapshot(), before);
            assert_eq!(core.runtime_facts.next_strike, u64::MAX);
            assert_eq!(core.units[&target].stats.current_health, 100);
            assert_eq!(core.runtime_fact_summary().damage_mutations, 0);

            core.runtime_facts.next_strike = u64::MAX - 1;
            let before_formula_failure = core.transaction_debug_snapshot();
            let error = core
                .runtime_transaction(|candidate| {
                    Ok(candidate.execute_committed_damage(input.clone()))
                })
                .unwrap_err();
            assert!(
                matches!(
                    error,
                    GameError::BattleRuntimeFault(BattleRuntimeFault {
                        code: Code::StrikeSegmentLimit,
                        site: "damage_formula_components",
                        ..
                    })
                ),
                "{error:?}"
            );
            assert_eq!(core.transaction_debug_snapshot(), before_formula_failure);
            assert_eq!(core.runtime_facts.next_strike, u64::MAX - 1);

            core.runtime_facts.limits.max_strike_segments = 4;
            let prior_fact_source = core.runtime_fact_source.clone();
            core.runtime_transaction(|candidate| {
                let prior = candidate.begin_strike_scope();
                for _ in 0..2 {
                    assert!(candidate.execute_committed_damage(input.clone()).is_some());
                }
                candidate.end_strike_scope(prior);
                Ok(())
            })
            .unwrap();
            let summary = core.runtime_fact_summary();
            assert_eq!(summary.damage_strikes, 1);
            assert_eq!(summary.damage_mutations, 2);
            assert_eq!(summary.damage_hp_lost, 4);
            let last = summary.latest_damage.as_ref().unwrap();
            // The allocator returns the current ordinal, then advances the
            // next counter. MAX - 1 is therefore its last admissible identity.
            assert_eq!(last.strike, StrikeOrdinal(u64::MAX - 1));
            assert_eq!(
                last.segments.iter().map(|s| s.ordinal).collect::<Vec<_>>(),
                vec![2, 3]
            );
            assert_eq!(core.runtime_facts.next_strike, u64::MAX);
            assert_eq!(core.runtime_fact_source, prior_fact_source);
            assert!(core.runtime_strike_scope.is_none());
        }
    }

    fn actor() -> UnitInstanceId {
        uuid::Uuid::from_u128(900).into()
    }
    fn accept(state: &mut RuntimeFactState, reserved: bool) -> ActionHandle {
        state
            .accept(actor(), None, RuntimeFactSource::default(), 0, reserved)
            .unwrap()
    }
    fn finish(state: &mut RuntimeFactState, handle: &ActionHandle, time: u64) {
        state.release(handle, 1, time).unwrap();
        state.dispatched(handle, 0, time).unwrap();
    }

    #[test]
    fn native_summary_cow_admits_detachment_in_native_and_authoring_scopes() {
        use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut original = RuntimeFactState::default();
        original.actor(actor()).unwrap();
        for authoring in [false, true] {
            let mut candidate = original.clone();
            assert!(Arc::ptr_eq(&candidate.actors, &original.actors));
            let mut operation = || {
                control::run(
                    RuntimeFactLimits {
                        max_clone_slots: 1,
                        ..Default::default()
                    },
                    || {
                        writable_actor_summaries(&mut candidate.actors)?.clear();
                        Ok(())
                    },
                )
            };
            let result = if authoring {
                crate::authoring::control::run(
                    &ExecutionLimits::default(),
                    &CancellationToken::default(),
                    ExecutionPhase::TrialExecution,
                    operation,
                )
                .unwrap()
                .0
            } else {
                operation()
            };
            assert!(
                matches!(result, Err(GameError::BattleRuntimeFault(f)) if f.code == Code::CloneLimit)
            );
            assert!(Arc::ptr_eq(&candidate.actors, &original.actors));
            assert_eq!(candidate.actors.len(), 1);
            control::run(Default::default(), || {
                writable_actor_summaries(&mut candidate.actors)?.clear();
                Ok(())
            })
            .unwrap();
            assert!(candidate.actors.is_empty());
            assert_eq!(original.actors.len(), 1);
        }
    }

    #[test]
    fn native_active_cow_keeps_nested_steps_and_deliveries_independent_on_failure_and_retry() {
        let mut original = RuntimeFactState::default();
        let handle = accept(&mut original, true);
        original.release(&handle, 3, 0).unwrap();
        let ticket = original.delivery(&handle).unwrap().unwrap();
        let mut candidate = original.clone();
        assert!(Arc::ptr_eq(&original.active, &candidate.active));
        let result = control::run(
            RuntimeFactLimits {
                max_clone_slots: 1,
                ..Default::default()
            },
            || candidate.dispatched(&handle, 0, 10),
        );
        assert!(
            matches!(result, Err(GameError::BattleRuntimeFault(f)) if f.code == Code::CloneLimit)
        );
        assert!(Arc::ptr_eq(&original.active, &candidate.active));
        assert_eq!(candidate.query(&handle).unwrap().pending_control_steps, 3);
        control::run(Default::default(), || candidate.dispatched(&handle, 0, 10)).unwrap();
        assert!(!Arc::ptr_eq(&original.active, &candidate.active));
        assert_eq!(candidate.query(&handle).unwrap().pending_control_steps, 2);
        assert_eq!(original.query(&handle).unwrap().pending_control_steps, 3);
        candidate.close_delivery(&ticket, 20).unwrap();
        assert_eq!(candidate.query(&handle).unwrap().outstanding_deliveries, 0);
        assert_eq!(original.query(&handle).unwrap().outstanding_deliveries, 1);
    }

    #[test]
    fn runtime_actions_clone_relative_validity_and_aborted_ordinal_reuse() {
        let mut original = RuntimeFactState::default();
        let handle = accept(&mut original, true);
        let mut cloned = original.clone();
        finish(&mut original, &handle, 100);
        assert!(original
            .query(&handle)
            .unwrap()
            .effects_settled_at_ms
            .is_some());
        assert!(cloned.query(&handle).unwrap().released_at_ms.is_none());
        let receipt = original.take(&handle).unwrap();
        assert!(matches!(
            original.query(&handle),
            Err(ActionQueryError::Unavailable)
        ));
        assert!(cloned.query(&handle).is_ok());
        finish(&mut cloned, &handle, 200);
        assert_eq!(
            cloned.take(&handle).unwrap().action.effects_settled_at_ms,
            Some(200)
        );
        assert_eq!(receipt.action.effects_settled_at_ms, Some(100));
        let mut discarded_candidate = original.clone();
        let escaped = accept(&mut discarded_candidate, true);
        let committed = accept(&mut original, true);
        assert_eq!(escaped.ordinal(), committed.ordinal());
        assert!(matches!(
            original.query(&escaped),
            Err(ActionQueryError::StaleOccurrence)
        ));
        assert!(matches!(
            RuntimeFactState::default().query(&handle),
            Err(ActionQueryError::DifferentAttempt)
        ));
    }

    #[test]
    fn runtime_actions_public_slot_reserved_at_acceptance_and_reused_only_after_take() {
        let limits = RuntimeFactLimits {
            max_retained_results: 1,
            ..Default::default()
        };
        let mut state = RuntimeFactState::new(limits);
        let handle = accept(&mut state, true);
        assert!(matches!(
            state.take(&handle),
            Err(ActionQueryError::NotSettled)
        ));
        assert!(matches!(
            state.accept(actor(), None, RuntimeFactSource::default(), 1, true),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::RetainedResultLimit,
                ..
            }))
        ));
        // Internal execution does not consume a promised-result slot.
        let internal = accept(&mut state, false);
        finish(&mut state, &internal, 20);
        finish(&mut state, &handle, 30);
        assert_eq!(state.usage.public_result_reservations, 1);
        assert_eq!(state.usage.completed_results, 1);
        let receipt = state.take(&handle).unwrap();
        assert_eq!(receipt.action.effects_settled_at_ms, Some(30));
        assert!(matches!(
            state.take(&handle),
            Err(ActionQueryError::Unavailable)
        ));
        assert_eq!(state.usage.public_result_reservations, 0);
        assert!(state
            .accept(actor(), None, RuntimeFactSource::default(), 40, true)
            .is_ok());
    }

    #[test]
    fn runtime_actions_cancelled_control_waits_for_actual_finite_delivery_once() {
        let mut state = RuntimeFactState::default();
        let handle = accept(&mut state, true);
        state.release(&handle, 2, 100).unwrap();
        let ticket = state.delivery(&handle).unwrap().unwrap();
        state.dispatched(&handle, 0, 100).unwrap();
        state
            .terminal(
                &handle,
                ActionTerminalKind::Cancelled(ActionCancelReason::Withdrawn),
                150,
            )
            .unwrap();
        let pending = state.query(&handle).unwrap();
        assert_eq!(pending.pending_control_steps, 0);
        assert_eq!(pending.outstanding_deliveries, 1);
        assert!(pending.effects_settled_at_ms.is_none());
        let mut missing = ticket.as_ref().clone();
        missing.ordinal = DeliveryOrdinal(99);
        state.close_delivery(&missing, 180).unwrap();
        assert_eq!(state.query(&handle).unwrap().outstanding_deliveries, 1);
        state.close_delivery(&ticket, 300).unwrap();
        state.close_delivery(&ticket, 400).unwrap();
        state
            .terminal(&handle, ActionTerminalKind::Finished, 400)
            .unwrap();
        let settled = state.query(&handle).unwrap();
        assert_eq!(settled.effects_settled_at_ms, Some(300));
        assert_eq!(
            settled.terminal.unwrap(),
            ActionTerminal {
                time_ms: 150,
                kind: ActionTerminalKind::Cancelled(ActionCancelReason::Withdrawn)
            }
        );
        assert_eq!(state.usage.settled_actions, 1);
        assert_eq!(state.usage.outstanding_deliveries, 0);
    }

    #[test]
    fn runtime_actions_long_internal_execution_retains_bounded_terminal_witnesses() {
        let mut state = RuntimeFactState::default();
        let mut settled_extent = None;
        for time in 0..2000 {
            let handle = accept(&mut state, false);
            finish(&mut state, &handle, time);
            assert!(state.active.is_empty());
            assert!(state.retained.is_empty());
            assert_eq!(state.usage.public_result_reservations, 0);
            assert_eq!(state.usage.actor_summaries, 1);
            let expected = *settled_extent.get_or_insert(state.usage.retained_bytes_bound);
            assert_eq!(state.usage.retained_bytes_bound, expected);
            assert_eq!(
                state
                    .latest_settled_action
                    .as_ref()
                    .unwrap()
                    .effects_settled_at_ms,
                Some(time)
            );
        }
        assert_eq!(state.usage.accepted_actions, 2000);
        assert_eq!(state.usage.settled_actions, 2000);
        assert_eq!(
            state.actors[&actor()]
                .latest_settled_action
                .as_ref()
                .unwrap()
                .ordinal,
            ActionOrdinal(1999)
        );
    }

    #[test]
    fn runtime_actions_active_control_and_delivery_limits_are_inclusive_and_reusable() {
        let mut state = RuntimeFactState::new(RuntimeFactLimits {
            max_active_actions: 1,
            max_control_steps: 1,
            max_deliveries: 1,
            ..Default::default()
        });
        let handle = accept(&mut state, false);
        assert!(matches!(
            state.accept(actor(), None, RuntimeFactSource::default(), 1, false),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::ActiveActionLimit,
                ..
            }))
        ));
        assert!(matches!(
            state.release(&handle, 2, 10),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::ControlStepLimit,
                ..
            }))
        ));
        assert!(state.query(&handle).unwrap().released_at_ms.is_none());
        state.release(&handle, 1, 10).unwrap();
        let first = state.delivery(&handle).unwrap().unwrap();
        assert!(matches!(
            state.delivery(&handle),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::DeliveryLimit,
                ..
            }))
        ));
        state.close_delivery(&first, 20).unwrap();
        let second = state.delivery(&handle).unwrap().unwrap();
        assert_ne!(first.ordinal, second.ordinal);
        state.close_delivery(&first, 30).unwrap();
        assert_eq!(state.query(&handle).unwrap().outstanding_deliveries, 1);
        state.dispatched(&handle, 0, 30).unwrap();
        state.close_delivery(&second, 40).unwrap();
        assert_eq!(state.usage.active_actions, 0);
        assert_eq!(state.usage.outstanding_deliveries, 0);
        accept(&mut state, false);
    }

    #[test]
    fn runtime_actions_actor_provider_and_ordinal_limits_fail_explicitly() {
        let mut state = RuntimeFactState::new(RuntimeFactLimits {
            max_actors: 1,
            max_providers: 1,
            ..Default::default()
        });
        state.actor(actor()).unwrap();
        state.actor(actor()).unwrap();
        assert!(matches!(
            state.actor(uuid::Uuid::from_u128(901).into()),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::ActorLimit,
                ..
            }))
        ));
        let provider = EquipmentSourceSnapshot {
            definition_uuid: uuid::Uuid::from_u128(1),
            runtime_item_instance_id: uuid::Uuid::from_u128(2),
            persistent_copy_uuid: None,
        };
        state.register_provider(provider, Some(actor())).unwrap();
        state.register_provider(provider, Some(actor())).unwrap();
        assert!(matches!(
            state.register_provider(
                EquipmentSourceSnapshot {
                    runtime_item_instance_id: uuid::Uuid::from_u128(3),
                    ..provider
                },
                Some(actor())
            ),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::ProviderLimit,
                ..
            }))
        ));
        state.next_action = u64::MAX;
        assert!(matches!(
            state.accept(actor(), None, RuntimeFactSource::default(), 0, false),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::CounterOverflow,
                site: "action_ordinal",
                ..
            }))
        ));
        assert!(state.active.is_empty());
        assert_eq!(state.usage.accepted_actions, 0);
    }

    #[test]
    fn runtime_fact_tree_growth_is_admitted_before_insertion_and_released_after_take() {
        let mut state = RuntimeFactState::default();
        let empty = state.usage.retained_bytes_bound;
        let handles: Vec<_> = (0..10).map(|_| accept(&mut state, true)).collect();
        let before = state.clone();
        let tree_growth = fact_tree_growth::<ActionOrdinal, ActiveAction>(state.active.len())
            .unwrap()
            + fact_tree_growth::<ActionOrdinal, RetainedAction>(
                state.usage.public_result_reservations,
            )
            .unwrap();
        assert!(tree_growth > std::mem::size_of::<ActiveAction>());
        state.limits.max_retained_bytes = state.usage.retained_bytes_bound + tree_growth - 1;
        assert!(matches!(
            state.accept(actor(), None, RuntimeFactSource::default(), 0, true),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::RetainedByteLimit,
                site: "action_tree_growth",
                ..
            }))
        ));
        assert!(Arc::ptr_eq(&state.active, &before.active));
        assert_eq!(state.active.len(), 10);
        assert_eq!(
            state.usage.retained_bytes_bound,
            before.usage.retained_bytes_bound
        );
        state.limits = RuntimeFactLimits::default();
        for handle in handles {
            finish(&mut state, &handle, 100);
            state.take(&handle).unwrap();
        }
        let witnesses = action_snapshot_extent(state.latest_settled_action.as_ref().unwrap())
            .unwrap()
            + action_snapshot_extent(
                state.actors[&actor()]
                    .latest_settled_action
                    .as_ref()
                    .unwrap(),
            )
            .unwrap()
            + std::mem::size_of::<ActorFactSummary>()
            + 128;
        assert_eq!(state.usage.retained_bytes_bound, empty + witnesses);
        assert!(state.active.is_empty());
        assert!(state.retained.is_empty());
    }

    #[test]
    fn runtime_actions_retained_source_admission_is_inclusive() {
        let mut reference = RuntimeFactState::default();
        accept(&mut reference, true);
        let required = reference.usage.retained_bytes_bound;
        let mut exact = RuntimeFactState::new(RuntimeFactLimits {
            max_retained_bytes: required,
            ..Default::default()
        });
        accept(&mut exact, true);
        let mut short = RuntimeFactState::new(RuntimeFactLimits {
            max_retained_bytes: required - 1,
            ..Default::default()
        });
        assert!(matches!(
            short.accept(actor(), None, RuntimeFactSource::default(), 0, true),
            Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: Code::RetainedByteLimit,
                ..
            }))
        ));
        assert_eq!(short.usage.accepted_actions, 0);
        assert!(short.active.is_empty());
    }
}
