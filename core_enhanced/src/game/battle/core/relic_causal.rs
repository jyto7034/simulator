//! Candidate-owned causal paths. Authored subscriber IDs, rather than generated
//! action aliases or logging rows, identify visits. Forks never share a mutable
//! visited set; joins collect only their named live participants.
use super::BattleCore;
use crate::game::{
    battle::{ids::UnitInstanceId, runtime_control as control, runtime_facts::RuntimeFactSource},
    behavior::GameError,
    data::ValidatedGameData,
    relic::structural::{
        authoring::*,
        causal::{CausalPayloadKind, EffectiveCausalProgram},
        EmployeeEffectivePlan, PlanContribution,
    },
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct VisitKey {
    pub copy_uuid: Uuid,
    pub rule_id: String,
}
crate::authoring::clone_shape::struct_shape!(VisitKey { copy_uuid, rule_id });
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CausalContext {
    pub root: u64,
    pub visited: BTreeSet<VisitKey>,
    /// Only actual Join admission can create these private memberships.
    pub participants: Vec<(u64, usize)>,
    pub suppress_reactions: bool,
}
crate::authoring::clone_shape::struct_shape!(CausalContext {
    root,
    visited,
    participants,
    suppress_reactions
});
#[derive(Clone)]
pub(crate) struct CausalProgramBinding {
    pub generation: ValidatedGameData,
    pub employee_uuid: Uuid,
    pub selected_program: Arc<EffectiveCausalProgram>,
    pub numeric_owner: Option<Arc<super::relic_numeric::RelicNumericOwner>>,
    pub owner: UnitInstanceId,
    pub source: Option<UnitInstanceId>,
    pub target: Option<UnitInstanceId>,
    pub killer: Option<UnitInstanceId>,
}
impl std::fmt::Debug for CausalProgramBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CausalProgramBinding")
            .field("generation", self.generation.identity())
            .field("program", &self.selected_program.id)
            .field("owner", &self.owner)
            .finish()
    }
}
impl PartialEq for CausalProgramBinding {
    fn eq(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.selected_program, &o.selected_program)
            && self.generation.identity() == o.generation.identity()
            && self.employee_uuid == o.employee_uuid
            && self.owner == o.owner
            && self.source == o.source
            && self.target == o.target
            && self.killer == o.killer
            && match (&self.numeric_owner, &o.numeric_owner) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
impl Eq for CausalProgramBinding {}
impl crate::authoring::clone_shape::CloneShape for CausalProgramBinding {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        m.scan_storage(Some(8))?;
        self.generation.visit_clone_shape(m)?;
        self.selected_program.visit_clone_shape(m)?;
        self.numeric_owner.visit_clone_shape(m)
    }
}
impl CausalProgramBinding {
    pub fn execution(&self) -> &crate::game::relic::structural::EffectiveProviderPlan {
        &self.selected_program.execution
    }
    pub fn contribution(&self) -> &PlanContribution {
        &self.selected_program.contribution
    }
    pub fn retained_extent(&self) -> Option<usize> {
        // The same immutable-generation convention as the enclosing effective
        // plan: retain the validated handle, price identity ownership, and do
        // not price another copy of the shared canonical catalogs.
        let mut bytes = std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.selected_program.retained_extent())?
            .checked_add(
                std::mem::size_of::<crate::game::data::content_pack::ContentIdentity>() + 64,
            )?
            .checked_add(self.generation.identity().semantic_sha256.capacity())?
            .checked_add(64)?;
        if let Some(owner) = &self.numeric_owner {
            bytes = bytes.checked_add(owner.retained_extent())?;
        }
        Some(bytes)
    }
    pub fn role(&self, step: usize) -> Option<UnitInstanceId> {
        match self.selected_program.targets.get(step)? {
            CausalTargetDef::Owner => Some(self.owner),
            CausalTargetDef::EventSource => self.source,
            CausalTargetDef::EventTarget => self.target,
            CausalTargetDef::Killer => self.killer,
        }
    }
}
/// A bounded immutable cause description, with no action/delivery handle,
/// compiled binding, resource capability or execution authority. Keep only the
/// root producer and this immediate triggering fact, never a recursive history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CausalFactObservation {
    pub actor: Option<UnitInstanceId>,
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    pub fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub cast: Option<Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    pub authored: Option<Arc<crate::game::ability::authoring::EffectSource>>,
}
crate::authoring::clone_shape::struct_shape!(CausalFactObservation {
    actor,
    equipment,
    fragment,
    cast,
    authored
});
impl CausalFactObservation {
    fn from_source(source: &RuntimeFactSource) -> Self {
        Self {
            actor: source.actor,
            equipment: source.equipment,
            fragment: source.fragment.clone(),
            cast: source.cast.clone(),
            authored: source.authored.clone(),
        }
    }
    fn retained_extent(&self) -> Option<usize> {
        let source = RuntimeFactSource {
            actor: self.actor,
            equipment: self.equipment,
            fragment: self.fragment.clone(),
            cast: self.cast.clone(),
            authored: self.authored.clone(),
            ..Default::default()
        };
        source
            .retained_extent()?
            .checked_add(std::mem::size_of::<Self>() + 64)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CausalCauseObservation {
    pub root: u64,
    pub owner: UnitInstanceId,
    pub root_source: Arc<CausalFactObservation>,
    pub triggering_source: Arc<CausalFactObservation>,
    pub actual_killer: Option<UnitInstanceId>,
    pub event_target: Option<UnitInstanceId>,
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub variant: crate::game::relic::RelicVariant,
    pub rule_id: String,
    pub event: Option<CausalEventDef>,
}
crate::authoring::clone_shape::struct_shape!(CausalCauseObservation {
    root,
    owner,
    root_source,
    triggering_source,
    actual_killer,
    event_target,
    copy_uuid,
    definition_id,
    variant,
    rule_id,
    event
});
impl CausalCauseObservation {
    fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.rule_id.capacity())?
            .checked_add(self.root_source.retained_extent()?)?
            .checked_add(self.triggering_source.retained_extent()?)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CausalSourceData {
    pub context: Option<Arc<CausalContext>>,
    pub program: Option<Arc<CausalProgramBinding>>,
    pub amount: Option<Arc<super::relic_wedge::PreNumericAmount>>,
    pub cause: Option<Arc<CausalCauseObservation>>,
}
crate::authoring::clone_shape::struct_shape!(CausalSourceData {
    context,
    program,
    amount,
    cause
});
impl CausalSourceData {
    pub fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>() + 64;
        if let Some(c) = &self.context {
            n = n.checked_add(context_extent(c)?)?;
        }
        if let Some(p) = &self.program {
            n = n.checked_add(p.retained_extent()?)?;
        }
        if self.amount.is_some() {
            n = n.checked_add(std::mem::size_of::<super::relic_wedge::PreNumericAmount>() + 64)?;
        }
        if let Some(cause) = &self.cause {
            n = n.checked_add(cause.retained_extent()?)?;
        }
        Some(n)
    }
}
impl CausalRuntime {
    pub(super) fn new_root(
        &mut self,
        suppress_reactions: bool,
    ) -> Result<Arc<CausalContext>, GameError> {
        let root = checked_next(&mut self.next_root)?;
        admit::<CausalContext>(0, "causal_root")?;
        Ok(Arc::new(CausalContext {
            root,
            visited: BTreeSet::new(),
            participants: Vec::new(),
            suppress_reactions,
        }))
    }
    pub(super) fn cancel_participants(
        &mut self,
        source: &RuntimeFactSource,
    ) -> Result<(), GameError> {
        if let Some(c) = source.causal().and_then(|c| c.context.as_ref()) {
            control::charge(c.participants.len() as u64, "causal_participant_cancel")?;
            for (id, index) in &c.participants {
                if let Some(j) = self.joins.get_mut(id) {
                    *j.cancelled
                        .get_mut(*index)
                        .ok_or_else(|| invalid("causal participant differs"))? = true;
                }
            }
        }
        Ok(())
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(tree_extent::<u64, JoinRecord>(self.joins.len())?)?
            .checked_add(tree_extent::<
                super::relic_wedge::WedgeKey,
                super::relic_wedge::WedgeProgress,
            >(self.wedges.len())?)?;
        for (_, j) in &self.joins {
            n = n
                .checked_add(j.source.retained_extent()?)?
                .checked_add(j.binding.retained_extent()?)?
                .checked_add(context_extent(&j.context)?)?;
            for visits in &j.participants {
                n = n.checked_add(tree_extent::<VisitKey, ()>(visits.len())?)?;
                for key in visits {
                    n = n.checked_add(key.rule_id.capacity().checked_add(64)?)?;
                }
            }
            n = n
                .checked_add(j.started.capacity())?
                .checked_add(j.cancelled.capacity())?
                .checked_add(
                    j.participants
                        .capacity()
                        .checked_mul(std::mem::size_of::<BTreeSet<VisitKey>>())?,
                )?;
        }
        for (key, _) in &self.wedges {
            n = n.checked_add(key.rule.capacity().checked_add(64)?)?;
        }
        Some(n)
    }
}

/// Owned, live-only observation. No action handle, executable binding or
/// replayable completion token can be recovered from this projection.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CausalRuntimeSnapshot {
    pub contexts: Vec<CausalContextSnapshot>,
    pub causes: Vec<CausalCauseObservation>,
    pub joins: Vec<CausalJoinSnapshot>,
    pub wedge_counter_count: usize,
    pub independent_periodic_count: usize,
    pub retained_bytes: usize,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CausalContextSnapshot {
    pub root: u64,
    pub visited: Vec<(Uuid, String)>,
    pub participants: Vec<(u64, usize)>,
    pub suppress_reactions: bool,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CausalJoinSnapshot {
    pub occurrence: u64,
    pub copy_uuid: Uuid,
    pub rule_id: String,
    pub participants: Vec<CausalParticipantSnapshot>,
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CausalParticipantSnapshot {
    pub index: usize,
    pub started: bool,
    pub cancelled: bool,
    pub live_carriers: usize,
    pub visited: Vec<(Uuid, String)>,
}
#[derive(Debug, Clone)]
pub(super) struct JoinRecord {
    binding: Arc<CausalProgramBinding>,
    context: Arc<CausalContext>,
    participants: Vec<BTreeSet<VisitKey>>,
    started: Vec<bool>,
    cancelled: Vec<bool>,
    source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(JoinRecord {
    binding,
    context,
    participants,
    started,
    cancelled,
    source
});
#[derive(Debug, Clone, Default)]
pub(super) struct CausalRuntime {
    next_root: u64,
    next_join: u64,
    last_time_ms: u64,
    joins: BTreeMap<u64, JoinRecord>,
    pub(super) wedges: BTreeMap<super::relic_wedge::WedgeKey, super::relic_wedge::WedgeProgress>,
}
crate::authoring::clone_shape::struct_shape!(CausalRuntime {
    next_root,
    next_join,
    last_time_ms,
    joins,
    wedges
});
fn checked_next(value: &mut u64) -> Result<u64, GameError> {
    let n = *value;
    *value = n
        .checked_add(1)
        .ok_or_else(|| invalid("causal occurrence counter overflow"))?;
    Ok(n)
}
fn invalid(_s: &str) -> GameError {
    control::invariant("causal_runtime_contract")
}
fn admit<T>(extra: usize, site: &'static str) -> Result<(), GameError> {
    control::admit_retained_growth(
        0,
        std::mem::size_of::<T>()
            .checked_add(128)
            .and_then(|n| n.checked_add(extra)),
        site,
    )
}
fn tree_extent<K, V>(len: usize) -> Option<usize> {
    if len == 0 {
        return Some(0);
    }
    (1usize.checked_add(len.saturating_sub(1) / 5)?).checked_mul(
        std::mem::size_of::<(K, V)>()
            .checked_mul(11)?
            .checked_add(12 * std::mem::size_of::<usize>() + 128)?,
    )
}
pub(super) fn admit_tree_row<K, V>(
    len: usize,
    extra: usize,
    site: &'static str,
) -> Result<(), GameError> {
    let growth = tree_extent::<K, V>(
        len.checked_add(1)
            .ok_or_else(|| invalid("causal tree length"))?,
    )
    .and_then(|n| n.checked_sub(tree_extent::<K, V>(len)?))
    .and_then(|n| n.checked_add(extra));
    control::admit_retained_growth(0, growth, site)
}
fn admit_vec_push<T>(v: &mut Vec<T>, extra: usize, site: &'static str) -> Result<(), GameError> {
    let needed = v
        .len()
        .checked_add(1)
        .ok_or_else(|| invalid("causal vector length"))?;
    let bytes = if needed > v.capacity() {
        needed
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|n| n.checked_add(64))
    } else {
        Some(0)
    };
    control::admit_retained_growth(0, bytes.and_then(|n| n.checked_add(extra)), site)?;
    if needed > v.capacity() {
        v.reserve_exact(needed - v.len());
    }
    Ok(())
}
fn context_extent(c: &CausalContext) -> Option<usize> {
    let mut n = std::mem::size_of::<CausalContext>()
        .checked_add(64)?
        .checked_add(tree_extent::<VisitKey, ()>(c.visited.len())?)?
        .checked_add(
            c.participants
                .capacity()
                .checked_mul(std::mem::size_of::<(u64, usize)>())?,
        )?;
    for key in &c.visited {
        n = n.checked_add(key.rule_id.capacity().checked_add(64)?)?;
    }
    Some(n)
}
fn clone_context(c: &CausalContext) -> Result<CausalContext, GameError> {
    control::admit_retained_growth(0, context_extent(c), "causal_ancestry_copy")?;
    crate::authoring::clone_shape::controlled_clone(c)
}

impl BattleCore {
    pub(super) fn cancel_causal_periodics_for(
        &mut self,
        owner: UnitInstanceId,
    ) -> Result<(), GameError> {
        if self.runtime_facts.causal.is_none() {
            return Ok(());
        }
        self.event_queue.retain_causal_periodics(|source| {
            source
                .causal()
                .and_then(|c| c.program.as_ref())
                .is_none_or(|b| b.owner != owner)
        })
    }
    fn causal_live_sources(&self) -> impl Iterator<Item = &RuntimeFactSource> {
        self.runtime_facts
            .live_sources()
            // A nested join's promised continuation is itself a finite live
            // descendant of its outer participants. Its source has no
            // membership in this join, so it cannot keep itself pending.
            .chain(
                self.runtime_facts
                    .causal
                    .iter()
                    .flat_map(|s| s.joins.values().map(|j| &j.source)),
            )
            .chain(
                self.event_queue
                    .iter_queued()
                    .filter(|q| {
                        !matches!(
                            q.event.view().as_ref(),
                            crate::game::battle::enums::BattleEvent::CausalPeriodic { .. }
                        )
                    })
                    .map(|q| &*q.fact_source),
            )
            .chain(self.active_skill_casts.values().map(|c| &c.fact_source))
            .chain(self.buffs.values().map(|b| &b.latest_application_source))
            .chain(
                self.active_projectiles
                    .values()
                    .map(|p| &p.source_snapshot.fact_source),
            )
            .chain(
                self.active_areas
                    .values()
                    .map(|p| &p.source_snapshot.fact_source),
            )
            .chain(
                self.projectiles
                    .values()
                    .map(|p| &p.source_snapshot.fact_source),
            )
            .chain(self.independent_objects.objects.values().map(|o| &o.source))
    }
    pub fn causal_runtime_snapshot(&self) -> Result<CausalRuntimeSnapshot, GameError> {
        let mut view = CausalRuntimeSnapshot {
            contexts: Vec::new(),
            causes: Vec::new(),
            joins: Vec::new(),
            wedge_counter_count: 0,
            independent_periodic_count: 0,
            retained_bytes: 0,
        };
        for source in self.causal_live_sources() {
            control::charge(1, "causal_snapshot_source")?;
            if let Some(cause) = source.causal().and_then(|c| c.cause.as_ref()) {
                let mut duplicate = false;
                for old in &view.causes {
                    control::charge(1, "causal_snapshot_cause_identity")?;
                    if old == cause.as_ref() {
                        duplicate = true;
                        break;
                    }
                }
                if !duplicate {
                    admit_vec_push(
                        &mut view.causes,
                        cause
                            .retained_extent()
                            .ok_or_else(|| invalid("causal cause extent"))?,
                        "causal_snapshot_cause",
                    )?;
                    view.causes
                        .push(crate::authoring::clone_shape::controlled_clone(
                            cause.as_ref(),
                        )?);
                }
            }
            let Some(c) = source.causal().and_then(|c| c.context.as_ref()) else {
                continue;
            };
            let mut duplicate = false;
            for old in &view.contexts {
                control::charge(
                    c.visited.len() as u64 + c.participants.len() as u64 + 1,
                    "causal_snapshot_identity",
                )?;
                if old.root == c.root
                    && old.suppress_reactions == c.suppress_reactions
                    && old.participants == c.participants
                    && old
                        .visited
                        .iter()
                        .map(|(copy, id)| (*copy, id.as_str()))
                        .eq(c.visited.iter().map(|k| (k.copy_uuid, k.rule_id.as_str())))
                {
                    duplicate = true;
                    break;
                }
            }
            if duplicate {
                continue;
            }
            let bytes = c
                .visited
                .iter()
                .try_fold(0usize, |n, k| {
                    n.checked_add(
                        k.rule_id
                            .len()
                            .checked_add(std::mem::size_of::<(Uuid, String)>() + 64)?,
                    )
                })
                .and_then(|n| {
                    n.checked_add(
                        c.participants
                            .len()
                            .checked_mul(std::mem::size_of::<(u64, usize)>())?,
                    )
                })
                .ok_or_else(|| invalid("causal snapshot context extent"))?;
            admit_vec_push(&mut view.contexts, bytes, "causal_snapshot_context")?;
            crate::authoring::control::charge_serialized(
                c.as_ref(),
                "causal_snapshot_context_copy",
            )?;
            view.contexts.push(CausalContextSnapshot {
                root: c.root,
                visited: c
                    .visited
                    .iter()
                    .map(|k| (k.copy_uuid, k.rule_id.clone()))
                    .collect(),
                participants: c.participants.clone(),
                suppress_reactions: c.suppress_reactions,
            });
        }
        for q in self.event_queue.iter_queued() {
            control::charge(1, "causal_snapshot_periodic")?;
            if matches!(
                q.event.view().as_ref(),
                crate::game::battle::enums::BattleEvent::CausalPeriodic { .. }
            ) {
                view.independent_periodic_count += 1;
            }
        }
        if let Some(state) = &self.runtime_facts.causal {
            view.retained_bytes = state
                .retained_extent()
                .ok_or_else(|| invalid("causal snapshot extent overflow"))?;
            view.wedge_counter_count = state.wedges.len();
            for (join, j) in &state.joins {
                let rule_id = &j.binding.contribution().rule_id;
                admit_vec_push(&mut view.joins, rule_id.len() + 64, "causal_snapshot_join")?;
                let mut participants = Vec::new();
                for (index, visits) in j.participants.iter().enumerate() {
                    let bytes = visits
                        .iter()
                        .try_fold(0usize, |n, k| {
                            n.checked_add(
                                k.rule_id
                                    .len()
                                    .checked_add(std::mem::size_of::<(Uuid, String)>() + 64)?,
                            )
                        })
                        .ok_or_else(|| invalid("causal snapshot visit extent"))?;
                    admit_vec_push(&mut participants, bytes, "causal_snapshot_participant")?;
                    let mut live = 0;
                    for source in self.causal_live_sources() {
                        control::charge(1, "causal_snapshot_live_scan")?;
                        if let Some(c) = source.causal().and_then(|c| c.context.as_ref()) {
                            control::charge(
                                c.participants.len() as u64,
                                "causal_snapshot_membership_scan",
                            )?;
                            if c.participants.contains(&(*join, index)) {
                                live += 1;
                            }
                        }
                    }
                    for k in visits {
                        control::charge(k.rule_id.len() as u64 + 1, "causal_snapshot_visit_copy")?;
                    }
                    participants.push(CausalParticipantSnapshot {
                        index,
                        started: j.started[index],
                        cancelled: j.cancelled[index],
                        live_carriers: live,
                        visited: visits
                            .iter()
                            .map(|k| (k.copy_uuid, k.rule_id.clone()))
                            .collect(),
                    });
                }
                control::charge(rule_id.len() as u64, "causal_snapshot_join_identity")?;
                view.joins.push(CausalJoinSnapshot {
                    occurrence: *join,
                    copy_uuid: j.binding.contribution().copy_uuid,
                    rule_id: rule_id.clone(),
                    participants,
                });
            }
        }
        Ok(view)
    }
    pub(super) fn settle_causal_runtime(
        &mut self,
        time_ms: u64,
        pending_events: &[super::queue::QueuedEvent],
    ) -> Result<(), GameError> {
        if let Some(state) = self.runtime_facts.causal.as_deref_mut() {
            state.last_time_ms = time_ms;
        }
        if self
            .runtime_facts
            .causal
            .as_ref()
            .is_none_or(|s| s.joins.is_empty() && s.wedges.is_empty())
        {
            return Ok(());
        }
        let mut live = BTreeSet::new();
        let mut casts = BTreeSet::new();
        // Timestamp buckets temporarily own queued carriers outside the heap.
        // They remain live until their individual dispatch completes.
        for source in self.causal_live_sources().chain(
            pending_events
                .iter()
                .filter(|q| {
                    !matches!(
                        q.event.view().as_ref(),
                        crate::game::battle::enums::BattleEvent::CausalPeriodic { .. }
                    )
                })
                .map(|q| &*q.fact_source),
        ) {
            control::charge(1, "causal_live_carrier_scan")?;
            if let Some(c) = source.causal().and_then(|c| c.context.as_ref()) {
                for membership in &c.participants {
                    if !live.contains(membership) {
                        admit_tree_row::<(u64, usize), ()>(
                            live.len(),
                            0,
                            "causal_live_participant",
                        )?;
                        live.insert(*membership);
                    }
                }
                control::charge(c.participants.len() as u64, "causal_live_membership_scan")?;
            }
            if let Some(cast) = &source.cast {
                if !casts.contains(&cast.cast_seq) {
                    admit_tree_row::<u64, ()>(casts.len(), 0, "wedge_live_cast")?;
                    casts.insert(cast.cast_seq);
                }
            }
        }
        let mut ready = Vec::new();
        for (id, j) in &self
            .runtime_facts
            .causal
            .as_ref()
            .expect("live causal owner")
            .joins
        {
            control::charge(j.participants.len() as u64, "causal_join_completion_scan")?;
            if j.cancelled.iter().any(|c| *c)
                || j.started
                    .iter()
                    .enumerate()
                    .all(|(i, started)| *started && !live.contains(&(*id, i)))
            {
                admit_vec_push(&mut ready, 0, "causal_ready_join")?;
                ready.push(*id);
            }
        }
        for id in ready {
            let j = self
                .causal_state()?
                .joins
                .remove(&id)
                .expect("selected live join");
            if j.cancelled.iter().any(|c| *c) {
                self.close_source_resource_delivery(j.source.resource_delivery().map(Arc::as_ref))?;
                if let Some(ticket) = j.source.delivery.as_deref() {
                    self.runtime_facts.close_delivery(ticket, time_ms)?;
                }
                continue;
            }
            let mut context = clone_context(j.context.as_ref())?;
            for participant in j.participants {
                for key in participant {
                    if !context.visited.contains(&key) {
                        admit_tree_row::<VisitKey, ()>(
                            context.visited.len(),
                            key.rule_id.len() + 64,
                            "causal_join_receipt_union",
                        )?;
                        context.visited.insert(key);
                    }
                }
            }
            self.note_participant_visits(&context)?;
            let mut source = self
                .with_causal_context(&j.source, Arc::new(context), Some(j.binding.clone()))?
                .causal_program_source()?;
            source.actor = Some(j.binding.owner);
            source.cast = None;
            source.authored = None;
            source.flow = None;
            source.use_control = false;
            source.fragment = None;
            source.equipment = None;
            // Dispatch the already admitted continuation lease exactly once.
            self.event_queue.push_completion(
                crate::game::battle::enums::BattleEvent::CausalInvoke { time_ms },
                None,
                source,
            );
            crate::authoring::control::poll("causal_join_continuation")?;
        }
        self.causal_state()?
            .wedges
            .retain(|key, _| casts.contains(&key.cast));
        Ok(())
    }
    fn effect_predicate(
        &self,
        p: &EffectPredicateDef,
        tags: &[String],
        damage: bool,
        footprint: bool,
    ) -> Result<bool, GameError> {
        control::charge(
            tags.len() as u64 + p.all_tags.len() as u64 + 1,
            "causal_effect_predicate",
        )?;
        Ok(p.op
            .is_none_or(|op| damage && op == EffectOperation::Damage)
            && p.all_tags.iter().all(|tag| tags.contains(tag))
            && p.has_property.is_none_or(|v| {
                damage
                    && (v == RelicProperty::DamageAmount
                        || footprint && v == RelicProperty::DamageFootprint)
            }))
    }
    fn causal_condition(
        &self,
        c: &RelicConditionDef,
        plan: &EmployeeEffectivePlan,
        tags: &[String],
        damage: bool,
        footprint: bool,
    ) -> Result<bool, GameError> {
        control::charge(1, "causal_condition")?;
        match c {
            RelicConditionDef::Always => Ok(true),
            RelicConditionDef::SameEffect(p) => {
                if damage {
                    self.effect_predicate(p, tags, damage, footprint)
                } else {
                    Ok(false)
                }
            }
            RelicConditionDef::Exists(selector) => {
                for node in plan.effects.iter().filter(|e| e.present) {
                    control::charge(node.tags.len() as u64 + 1, "causal_exists_effect")?;
                    let provider = &plan.providers[node.provider];
                    let matches = match provider {
                        crate::game::relic::structural::ProspectiveProvider::Equipment {
                            definition_id,
                            ..
                        } => {
                            selector.provider.fragment_id.is_none()
                                && selector.provider.form_id.is_none()
                                && selector
                                    .provider
                                    .equipment_id
                                    .as_deref()
                                    .is_none_or(|id| id == definition_id)
                        }
                        crate::game::relic::structural::ProspectiveProvider::Fragment(f) => {
                            selector.provider.equipment_id.is_none()
                                && selector
                                    .provider
                                    .fragment_id
                                    .as_deref()
                                    .is_none_or(|id| id == f.definition_id.as_str())
                                && selector
                                    .provider
                                    .form_id
                                    .as_deref()
                                    .is_none_or(|id| id == f.form_id)
                        }
                    };
                    if matches
                        && selector.effect.op.is_none_or(|o| o == node.operation)
                        && selector
                            .effect
                            .all_tags
                            .iter()
                            .all(|tag| node.tags.contains(tag))
                        && selector
                            .effect
                            .has_property
                            .is_none_or(|p| node.has_property(p))
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            RelicConditionDef::All(v) => {
                for c in v {
                    if !self.causal_condition(c, plan, tags, damage, footprint)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            RelicConditionDef::Any(v) => {
                for c in v {
                    if self.causal_condition(c, plan, tags, damage, footprint)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            RelicConditionDef::Not(c) => {
                Ok(!self.causal_condition(c, plan, tags, damage, footprint)?)
            }
        }
    }
    fn causal_tags<'a>(
        &'a self,
        source: &'a RuntimeFactSource,
    ) -> Result<(&'a [String], bool), GameError> {
        let Some(a) = source.authored.as_ref() else {
            return Ok((&[], false));
        };
        let Some(skill) = self.skill_for_source(&a.site.action.definition_id, source) else {
            crate::authoring::control::poll("causal_effect_metadata")?;
            return Ok((&[], false));
        };
        control::charge(skill.steps.len() as u64, "causal_effect_step_metadata")?;
        let Some(step) = skill.steps.iter().find(|s| s.id == a.site.action.action_id) else {
            return Ok((&[], false));
        };
        control::charge(step.effects.len() as u64, "causal_effect_site_metadata")?;
        let tags = step
            .effects
            .iter()
            .find(|e| e.id == a.site.effect_id)
            .map_or(&[][..], |e| e.tags.as_slice());
        Ok((
            tags,
            matches!(
                step.delivery,
                crate::game::ability::DeliveryDef::TileArea { .. }
            ),
        ))
    }
    fn subscriber_candidates(
        &self,
    ) -> Result<Vec<(UnitInstanceId, Arc<EmployeeEffectivePlan>, usize)>, GameError> {
        let mut candidates = Vec::new();
        let Some(owner) = &self.effective_owner else {
            return Ok(candidates);
        };
        control::charge(self.units.len() as u64, "causal_subscriber_owners")?;
        for unit in self.units.values() {
            let Some(plan) = owner.plans.get(&unit.source_owned_uuid) else {
                continue;
            };
            for (index, rule) in plan.causal_rules.iter().enumerate() {
                control::charge(1, "causal_subscriber_rules")?;
                if rule.event.is_none() {
                    continue;
                }
                admit_vec_push(&mut candidates, 0, "causal_subscriber_selection")?;
                candidates.push((unit.instance_id, Arc::clone(plan), index));
            }
        }
        candidates.sort_by(|a, b| {
            a.0.as_bytes()
                .cmp(b.0.as_bytes())
                .then_with(|| a.2.cmp(&b.2))
        });
        Ok(candidates)
    }
    pub(super) fn observe_causal_damage(
        &mut self,
        fact: &crate::game::battle::runtime_facts::DamageFact,
        damage_source: crate::game::battle::damage::DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
    ) -> Result<(), GameError> {
        if fact.actual_hp_loss() == 0 {
            return Ok(());
        }
        let candidates = self.subscriber_candidates()?;
        if candidates.is_empty() {
            return Ok(());
        }
        let source = self.causal_root_source(fact.source.as_ref(), false)?;
        for (owner, plan, index) in candidates {
            let rule = &plan.causal_rules[index];
            let Some(CausalEventDef::DamageHpLost { perspective }) = rule.event else {
                continue;
            };
            if !self.units.get(&owner).is_some_and(|u| u.is_active()) {
                continue;
            }
            let matches = match perspective {
                CausalDamagePerspective::Dealt => source.actor == Some(owner),
                CausalDamagePerspective::Taken => fact.target == owner,
                CausalDamagePerspective::Any => true,
            };
            if !matches {
                continue;
            }
            let (tags, footprint) = self.causal_tags(&source)?;
            if !crate::game::relic::structural::causal::owner_provider_matches(
                &rule.selector.provider,
                &plan.providers,
            ) || !self.effect_predicate(&rule.selector.effect, tags, true, footprint)?
                || !self.causal_condition(&rule.condition, &plan, tags, true, footprint)?
            {
                continue;
            }
            use crate::game::data::relic_data::RelicDamageTarget;
            let kind = match damage_source {
                crate::game::battle::damage::DamageSource::BasicAttack => {
                    RelicDamageTarget::BasicAttackDamage
                }
                crate::game::battle::damage::DamageSource::Ability => {
                    RelicDamageTarget::SkillDamage
                }
                _ => RelicDamageTarget::AnyDamage,
            };
            let context = super::relic_numeric::RelicSelectorContext {
                target: kind,
                tags,
                damage_type,
                equipment: source.equipment.map(|e| e.definition_uuid),
                fragment: source.fragment.as_deref(),
            };
            if !matches!(
                super::relic_numeric::selector_state(&self.game_data, &rule.source, &context)?,
                crate::game::relic::plan::RelicNumericPlanEffectState::Active
            ) {
                continue;
            }
            self.start_causal_payload(
                plan,
                index,
                owner,
                source.actor,
                Some(fact.target),
                None,
                &source,
                fact.time_ms,
            )?;
        }
        Ok(())
    }
    pub(super) fn observe_causal_death(
        &mut self,
        killer: Option<UnitInstanceId>,
        victim: UnitInstanceId,
        side: crate::game::enums::Side,
        time_ms: u64,
        dispatch: bool,
    ) -> Result<(), GameError> {
        if !dispatch {
            return Ok(());
        }
        let candidates = self.subscriber_candidates()?;
        if candidates.is_empty() {
            return Ok(());
        }
        let parent = self.runtime_fact_source.clone();
        let parent = self.causal_root_source(&parent, false)?;
        // Same native order as equipment reactions; these categories are not a
        // global ban on already-authorized ordinary effects from a dead killer.
        for event in [
            CausalEventDef::Death,
            CausalEventDef::Kill,
            CausalEventDef::AllyDeath,
        ] {
            for (owner, plan, index) in &candidates {
                let rule = &plan.causal_rules[*index];
                if rule.event != Some(event) {
                    continue;
                }
                let eligible = match event {
                    CausalEventDef::Death => *owner == victim,
                    CausalEventDef::Kill => killer == Some(*owner),
                    CausalEventDef::AllyDeath => {
                        *owner != victim
                            && self
                                .units
                                .get(owner)
                                .is_some_and(|u| u.is_active() && u.owner == side)
                    }
                    _ => false,
                };
                if eligible
                    && crate::game::relic::structural::causal::owner_provider_matches(
                        &rule.selector.provider,
                        &plan.providers,
                    )
                    && self.effect_predicate(&rule.selector.effect, &[], false, false)?
                    && self.causal_condition(&rule.condition, plan, &[], false, false)?
                {
                    self.start_causal_payload(
                        plan.clone(),
                        *index,
                        *owner,
                        parent.actor,
                        Some(victim),
                        killer,
                        &parent,
                        time_ms,
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn start_causal_periodics(&mut self, time_ms: Option<u64>) -> Result<(), GameError> {
        // Owner installation can precede both startup reset and live deployment.
        // Reestablish feature state after reset, and admit producers only once
        // their real owner unit exists. Ordinary battles have no added scan.
        if let Some(owner) = &self.effective_owner {
            control::charge(owner.plans.len() as u64, "causal_owner_presence")?;
            if owner
                .plans
                .values()
                .any(|p| !p.causal_rules.is_empty() || !p.wedge_rules.is_empty())
            {
                self.causal_state()?;
            }
        }
        if self.runtime_facts.causal.is_none() {
            return Ok(());
        }
        let current = &self.effective_owner;
        let units = &self.units;
        self.event_queue.retain_causal_periodics(|source| {
            source
                .causal()
                .and_then(|c| c.program.as_ref())
                .is_some_and(|b| {
                    units
                        .get(&b.owner)
                        .filter(|u| u.is_active())
                        .and_then(|u| {
                            current
                                .as_ref()
                                .and_then(|o| o.plans.get(&u.source_owned_uuid))
                        })
                        .is_some_and(|p| {
                            p.causal_rules.iter().any(|r| {
                                r.interval_ms.is_some()
                                    && r.contribution.copy_uuid == b.contribution().copy_uuid
                                    && r.contribution.rule_id == b.contribution().rule_id
                                    && r.contribution.variant == b.contribution().variant
                            })
                        })
                })
        })?;
        let Some(owner) = self.effective_owner.clone() else {
            return Ok(());
        };
        control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<(UnitInstanceId, Uuid)>())
                .and_then(|n| n.checked_add(64)),
            "causal_periodic_owners",
        )?;
        let units = self
            .units
            .values()
            .map(|u| (u.instance_id, u.source_owned_uuid))
            .collect::<Vec<_>>();
        control::charge(units.len() as u64, "causal_periodic_owner_scan")?;
        for (actor, employee) in units {
            if !self.units.get(&actor).is_some_and(|u| u.is_active()) {
                continue;
            }
            let Some(plan) = owner.plans.get(&employee) else {
                continue;
            };
            for rule in &plan.causal_rules {
                let Some(interval) = rule.interval_ms else {
                    continue;
                };
                control::charge(plan.providers.len() as u64, "causal_periodic_provider")?;
                if !crate::game::relic::structural::causal::owner_provider_matches(
                    &rule.selector.provider,
                    &plan.providers,
                ) || !self.causal_condition(&rule.condition, plan, &[], false, false)?
                {
                    continue;
                }
                control::charge(
                    self.event_queue.len() as u64,
                    "causal_periodic_existing_scan",
                )?;
                if self.event_queue.iter_queued().any(|q| {
                    matches!(
                        q.event.view().as_ref(),
                        crate::game::battle::enums::BattleEvent::CausalPeriodic { .. }
                    ) && q
                        .fact_source
                        .causal()
                        .and_then(|s| s.program.as_ref())
                        .is_some_and(|b| {
                            b.owner == actor
                                && b.contribution().copy_uuid == rule.contribution.copy_uuid
                                && b.contribution().rule_id == rule.contribution.rule_id
                                && b.contribution().variant == rule.contribution.variant
                        })
                }) {
                    continue;
                }

                admit::<CausalProgramBinding>(0, "causal_periodic_binding")?;
                let binding = Arc::new(CausalProgramBinding {
                    generation: plan.generation.clone(),
                    employee_uuid: plan.employee_uuid,
                    selected_program: Arc::clone(&rule.programs[0]),
                    numeric_owner: self.relic_numeric_owner.clone(),
                    owner: actor,
                    source: Some(actor),
                    target: Some(actor),
                    killer: None,
                });
                admit::<CausalSourceData>(0, "causal_periodic_source")?;
                let source = RuntimeFactSource {
                    actor: Some(actor),
                    ..Default::default()
                }
                .with_causal(Some(Arc::new(CausalSourceData {
                    context: None,
                    program: Some(binding),
                    amount: None,
                    cause: None,
                })))?;
                let next = time_ms
                    .unwrap_or_else(|| {
                        self.runtime_facts
                            .causal
                            .as_ref()
                            .map_or(0, |s| s.last_time_ms)
                    })
                    .checked_add(interval)
                    .ok_or_else(|| invalid("independent periodic schedule overflow"))?;
                self.event_queue.push_with_context(
                    crate::game::battle::enums::BattleEvent::CausalPeriodic { time_ms: next },
                    None,
                    source,
                );
                crate::authoring::control::poll("causal_periodic_start")?;
            }
        }
        Ok(())
    }
    pub(super) fn process_causal_periodic(&mut self, time_ms: u64) -> Result<(), GameError> {
        let source = self.runtime_fact_source.clone();
        let binding = source
            .causal()
            .and_then(|c| c.program.clone())
            .ok_or_else(|| invalid("periodic producer has no owner binding"))?;
        let Some(unit) = self.units.get(&binding.owner).filter(|u| u.is_active()) else {
            return Ok(());
        };
        let Some(current) = self
            .effective_owner
            .as_ref()
            .and_then(|o| o.plans.get(&unit.source_owned_uuid))
            .cloned()
        else {
            return Ok(());
        };
        let old = binding.contribution();
        let Some(index) = current.causal_rules.iter().position(|r| {
            r.interval_ms.is_some()
                && r.contribution.copy_uuid == old.copy_uuid
                && r.contribution.rule_id == old.rule_id
                && r.contribution.variant == old.variant
        }) else {
            return Ok(());
        };
        let rule = &current.causal_rules[index];
        let interval = rule
            .interval_ms
            .ok_or_else(|| invalid("not an independent periodic producer"))?;
        // Only this owner producer mints a new root. Derived deliveries retain
        // their accepted plan even after the current producer is removed.
        let root = self.causal_root_source(
            &RuntimeFactSource {
                actor: Some(binding.owner),
                ..Default::default()
            },
            false,
        )?;
        if crate::game::relic::structural::causal::owner_provider_matches(
            &rule.selector.provider,
            &current.providers,
        ) && self.effect_predicate(&rule.selector.effect, &[], false, false)?
            && self.causal_condition(&rule.condition, &current, &[], false, false)?
        {
            self.start_causal_payload(
                current.clone(),
                index,
                binding.owner,
                Some(binding.owner),
                Some(binding.owner),
                None,
                &root,
                time_ms,
            )?;
        }
        let next = time_ms
            .checked_add(interval)
            .ok_or_else(|| invalid("independent periodic schedule overflow"))?;
        admit::<CausalProgramBinding>(0, "causal_periodic_next_binding")?;
        let next_binding = Arc::new(CausalProgramBinding {
            generation: current.generation.clone(),
            employee_uuid: current.employee_uuid,
            selected_program: Arc::clone(&rule.programs[0]),
            numeric_owner: self.relic_numeric_owner.clone(),
            owner: binding.owner,
            source: Some(binding.owner),
            target: Some(binding.owner),
            killer: None,
        });
        admit::<CausalSourceData>(0, "causal_periodic_next_source")?;
        let next_source = RuntimeFactSource {
            actor: Some(binding.owner),
            ..Default::default()
        }
        .with_causal(Some(Arc::new(CausalSourceData {
            context: None,
            program: Some(next_binding),
            amount: None,
            cause: None,
        })))?;
        self.event_queue.push_with_context(
            crate::game::battle::enums::BattleEvent::CausalPeriodic { time_ms: next },
            None,
            next_source,
        );
        crate::authoring::control::poll("causal_periodic_rescheduled")
    }
    pub(super) fn causal_state(&mut self) -> Result<&mut CausalRuntime, GameError> {
        if self.runtime_facts.causal.is_none() {
            admit::<CausalRuntime>(0, "causal_runtime_owner")?;
            self.runtime_facts.causal = Some(Box::new(CausalRuntime::default()));
        }
        Ok(self
            .runtime_facts
            .causal
            .as_deref_mut()
            .expect("admitted causal owner"))
    }
    pub(super) fn causal_root_source(
        &mut self,
        source: &RuntimeFactSource,
        suppress: bool,
    ) -> Result<RuntimeFactSource, GameError> {
        if let Some(context) = source.causal().and_then(|c| c.context.as_ref()) {
            if suppress && !context.suppress_reactions {
                let mut context = clone_context(context)?;
                context.suppress_reactions = true;
                return self.with_causal_context(
                    source,
                    Arc::new(context),
                    source.causal().and_then(|c| c.program.clone()),
                );
            }
            return Ok(source.clone());
        }
        let context = self.causal_state()?.new_root(suppress)?;
        self.with_causal_context(source, context, None)
    }
    /// An independent periodic producer starts one new causal path for all
    /// recipients of this event. Preserve immutable payload/value ownership and
    /// reaction suppression, but never inherit another event's visit history.
    pub(super) fn independent_periodic_source(
        &mut self,
        source: &RuntimeFactSource,
    ) -> Result<RuntimeFactSource, GameError> {
        let causal = source.causal();
        let suppress = causal.and_then(|c| c.context.as_ref())
            .is_some_and(|c| c.suppress_reactions);
        let context = self.causal_state()?.new_root(suppress)?;
        admit::<CausalSourceData>(0, "independent_periodic_source")?;
        source.with_causal(Some(Arc::new(CausalSourceData {
            context: Some(context),
            program: causal.and_then(|c| c.program.clone()),
            amount: causal.and_then(|c| c.amount.clone()),
            cause: None,
        })))
    }
    fn with_causal_context(
        &self,
        source: &RuntimeFactSource,
        context: Arc<CausalContext>,
        program: Option<Arc<CausalProgramBinding>>,
    ) -> Result<RuntimeFactSource, GameError> {
        admit::<CausalSourceData>(0, "causal_source_owner")?;
        source.with_causal(Some(Arc::new(CausalSourceData {
            context: Some(context),
            program,
            amount: None,
            cause: source.causal().and_then(|c| c.cause.clone()),
        })))
    }
    fn note_participant_visits(&mut self, c: &CausalContext) -> Result<(), GameError> {
        for (join, branch) in &c.participants {
            control::charge(c.visited.len() as u64, "causal_participant_visits")?;
            if let Some(j) = self
                .runtime_facts
                .causal
                .as_deref_mut()
                .and_then(|s| s.joins.get_mut(join))
            {
                let set = j
                    .participants
                    .get_mut(*branch)
                    .ok_or_else(|| invalid("causal participant differs"))?;
                for key in &c.visited {
                    if !set.contains(key) {
                        admit_tree_row::<VisitKey, ()>(
                            set.len(),
                            key.rule_id.len() + 64,
                            "causal_participant_union",
                        )?;
                        set.insert(key.clone());
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn start_causal_payload(
        &mut self,
        plan: Arc<EmployeeEffectivePlan>,
        rule: usize,
        owner: UnitInstanceId,
        source: Option<UnitInstanceId>,
        target: Option<UnitInstanceId>,
        killer: Option<UnitInstanceId>,
        parent: &RuntimeFactSource,
        time_ms: u64,
    ) -> Result<(), GameError> {
        let r = &plan.causal_rules[rule];
        let parent = self.causal_root_source(parent, false)?;
        let old = parent
            .causal()
            .and_then(|s| s.context.as_ref())
            .expect("native root");
        control::charge(old.visited.len() as u64 + 1, "causal_subscriber_entry")?;
        if old.suppress_reactions
            || old.visited.iter().any(|k| {
                k.copy_uuid == r.contribution.copy_uuid && k.rule_id == r.contribution.rule_id
            })
        {
            return Ok(());
        }
        admit::<VisitKey>(r.contribution.rule_id.len(), "causal_visit_key")?;
        let key = VisitKey {
            copy_uuid: r.contribution.copy_uuid,
            rule_id: r.contribution.rule_id.clone(),
        };
        admit::<CausalFactObservation>(0, "causal_trigger_observation")?;
        let triggering_source = Arc::new(CausalFactObservation::from_source(&parent));
        let root_source = parent.causal().and_then(|c| c.cause.as_ref()).map_or_else(
            || Arc::clone(&triggering_source),
            |cause| Arc::clone(&cause.root_source),
        );
        admit::<CausalCauseObservation>(
            r.contribution.definition_id.len() + r.contribution.rule_id.len(),
            "causal_cause_observation",
        )?;
        crate::authoring::control::charge_serialized(
            &r.contribution,
            "causal_cause_identity_copy",
        )?;
        let cause = Arc::new(CausalCauseObservation {
            root: old.root,
            owner,
            root_source,
            triggering_source,
            actual_killer: killer,
            event_target: target,
            copy_uuid: r.contribution.copy_uuid,
            definition_id: r.contribution.definition_id.clone(),
            variant: r.contribution.variant,
            rule_id: r.contribution.rule_id.clone(),
            event: r.event,
        });
        admit::<CausalSourceData>(0, "causal_cause_source")?;
        let parent = parent.with_causal(Some(Arc::new(CausalSourceData {
            context: parent.causal().and_then(|c| c.context.clone()),
            program: parent.causal().and_then(|c| c.program.clone()),
            amount: None,
            cause: Some(cause),
        })))?;
        let old = parent
            .causal()
            .and_then(|c| c.context.as_ref())
            .expect("native root");

        admit::<CausalContext>(
            old.visited.len() * 128 + key.rule_id.len(),
            "causal_ancestry_fork",
        )?;
        let mut context = clone_context(old.as_ref())?;
        admit_tree_row::<VisitKey, ()>(
            context.visited.len(),
            key.rule_id.len() + 64,
            "causal_visit_entry",
        )?;
        context.visited.insert(key);
        self.note_participant_visits(&context)?;
        admit::<CausalProgramBinding>(0, "causal_payload_binding")?;
        let binding = Arc::new(CausalProgramBinding {
            generation: plan.generation.clone(),
            employee_uuid: plan.employee_uuid,
            selected_program: Arc::clone(&r.programs[0]),
            numeric_owner: self.relic_numeric_owner.clone(),
            owner,
            source,
            target,
            killer,
        });
        let count = match &r.kind {
            CausalPayloadKind::Program => 1,
            CausalPayloadKind::Fork => r.programs.len(),
            CausalPayloadKind::Join { then_program } => *then_program,
        };
        let join = if let CausalPayloadKind::Join { then_program } = &r.kind {
            let join = checked_next(&mut self.causal_state()?.next_join)?;
            let bytes = count
                .checked_mul(std::mem::size_of::<BTreeSet<VisitKey>>() + 2)
                .and_then(|n| n.checked_add(std::mem::size_of::<CausalProgramBinding>() + 128))
                .ok_or_else(|| invalid("causal join participants extent"))?;
            admit::<JoinRecord>(bytes, "causal_join_participants")?;
            let mut then = binding.as_ref().clone();
            then.selected_program = Arc::clone(&r.programs[*then_program]);
            // Keep one finite completion lease for the join continuation. This
            // admits its authority before any participant can settle.
            let held_source = self.reserve_finite_source(parent.clone())?;
            let existing = self.causal_state()?.joins.len();
            admit_tree_row::<u64, JoinRecord>(existing, 0, "causal_join_table")?;
            self.causal_state()?.joins.insert(
                join,
                JoinRecord {
                    binding: Arc::new(then),
                    context: Arc::new(clone_context(&context)?),
                    participants: vec![BTreeSet::new(); count],
                    started: vec![false; count],
                    cancelled: vec![false; count],
                    source: held_source,
                },
            );
            Some(join)
        } else {
            None
        };
        for program in 0..count {
            control::charge(1, "causal_payload_branch")?;
            let mut branch = clone_context(&context)?;
            if let Some(join) = join {
                control::admit_retained_growth(
                    0,
                    branch
                        .participants
                        .len()
                        .checked_add(1)
                        .and_then(|n| n.checked_mul(std::mem::size_of::<(u64, usize)>()))
                        .and_then(|n| n.checked_add(64)),
                    "causal_participant_membership",
                )?;
                branch.participants.reserve_exact(1);
                branch.participants.push((join, program));
            }
            self.note_participant_visits(&branch)?;
            admit::<CausalProgramBinding>(0, "causal_branch_binding")?;
            let mut b = binding.as_ref().clone();
            b.selected_program = Arc::clone(&r.programs[program]);
            let mut s = self
                .with_causal_context(&parent, Arc::new(branch), Some(Arc::new(b)))?
                .causal_program_source()?;
            // A child is an ordinary new accepted action. Provider identity from
            // the triggering effect remains observation, not cast authority.
            s.actor = Some(owner);
            s.cast = None;
            s.authored = None;
            s.flow = None;
            s.use_control = false;
            s.fragment = None;
            s.equipment = None;
            self.enqueue_finite_effect(
                crate::game::battle::enums::BattleEvent::CausalInvoke { time_ms },
                None,
                s,
            )?;
        }
        Ok(())
    }
    pub(super) fn invoke_causal_program(&mut self, time_ms: u64) -> Result<(), GameError> {
        let s = self.runtime_fact_source.clone();
        let data = s
            .causal()
            .ok_or_else(|| invalid("missing causal program source"))?;
        let b = data
            .program
            .as_ref()
            .ok_or_else(|| invalid("missing causal program binding"))?
            .clone();
        if self.content_identity.as_deref() != Some(b.generation.identity()) {
            return Err(invalid("causal program generation differs"));
        }
        if let Some(c) = &data.context {
            for (join, branch) in &c.participants {
                control::charge(1, "causal_participant_start")?;
                if let Some(j) = self.causal_state()?.joins.get_mut(join) {
                    *j.started
                        .get_mut(*branch)
                        .ok_or_else(|| invalid("causal participant differs"))? = true;
                }
            }
        }
        let id = b.execution().skill.id.clone();
        let allow_dead = b.selected_program.event == Some(CausalEventDef::Death);
        if !allow_dead && !self.units.get(&b.owner).is_some_and(|u| u.is_active()) {
            self.causal_state()?.cancel_participants(&s)?;
            return Ok(());
        }
        self.invoke_ability(
            time_ms,
            b.owner,
            id.as_str(),
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: b.owner,
            }),
            crate::game::battle::event_log::BattleEventCause::default(),
            allow_dead,
        );
        crate::authoring::control::poll("causal_program_invoked")
    }
}
