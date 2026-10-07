use crate::game::battle::{
    runtime_facts::RuntimeFactSource, skill_source::SkillEffectExecutionSource,
};
use crate::{
    authoring::control::{self, Resource},
    game::battle::enums::BattleEvent,
};
use std::{borrow::Cow, cmp::Ordering, collections::BinaryHeap, sync::Arc};

/// Private execution metadata moves with its event through heap and time buckets.
/// It never participates in gameplay priority or changes the public event type.
#[derive(Debug, Clone)]
pub(super) struct QueuedEvent {
    pub(super) close_delivery_on_dispatch: bool,
    pub(super) event: PackedEvent,
    pub(super) source: Option<Arc<SkillEffectExecutionSource>>,
    pub(super) fact_source: QueuedFactSource,
}

/// Small periodic events do not reserve the largest BattleEvent variant in
/// every heap/bucket slot. All comparisons still use the original event value.
#[derive(Debug, Clone)]
pub(super) enum PackedEvent {
    MovementTick {
        time_ms: u64,
    },
    AttackStart {
        time_ms: u64,
        attacker_instance_id: crate::game::battle::ids::UnitInstanceId,
        target_instance_id: Option<crate::game::battle::ids::UnitInstanceId>,
        schedule_next: bool,
        cause: crate::game::battle::event_log::BattleEventCause,
    },
    Large(Arc<BattleEvent>),
}
impl crate::authoring::clone_shape::CloneShape for PackedEvent {
    fn visit_clone_shape(
        &self,
        meter: &mut control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::Large(event) => event.visit_clone_shape(meter),
            _ => self.view().as_ref().visit_clone_shape(meter),
        }
    }
}
impl PackedEvent {
    fn new(event: BattleEvent) -> Result<Self, crate::game::behavior::GameError> {
        Ok(match event {
            BattleEvent::ContinuousMovementTick { time_ms } => Self::MovementTick { time_ms },
            BattleEvent::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => Self::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            },
            event => {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    std::mem::size_of::<BattleEvent>().checked_add(64),
                    "queued_event_payload",
                )?;
                Self::Large(Arc::new(event))
            }
        })
    }
    pub(super) fn view(&self) -> Cow<'_, BattleEvent> {
        match self {
            Self::MovementTick { time_ms } => {
                Cow::Owned(BattleEvent::ContinuousMovementTick { time_ms: *time_ms })
            }
            Self::AttackStart {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                schedule_next,
                cause,
            } => Cow::Owned(BattleEvent::AttackStart {
                time_ms: *time_ms,
                attacker_instance_id: *attacker_instance_id,
                target_instance_id: *target_instance_id,
                schedule_next: *schedule_next,
                cause: *cause,
            }),
            Self::Large(event) => Cow::Borrowed(event.as_ref()),
        }
    }
    fn time_ms(&self) -> u64 {
        match self {
            Self::MovementTick { time_ms } | Self::AttackStart { time_ms, .. } => *time_ms,
            Self::Large(event) => event.time_ms(),
        }
    }
    pub(super) fn payload_extent(&self) -> usize {
        match self {
            Self::Large(_) => std::mem::size_of::<BattleEvent>() + 64,
            _ => 0,
        }
    }
    pub(super) fn into_owned(self) -> Result<BattleEvent, crate::game::behavior::GameError> {
        use crate::authoring::{
            clone_shape::{controlled_clone, CloneShape},
            control::CloneMeter,
        };
        use crate::game::battle::runtime_control as native;
        match self {
            Self::Large(event) => match Arc::try_unwrap(event) {
                Ok(event) => Ok(event),
                Err(event) => {
                    if native::is_active() {
                        let mut meter = CloneMeter::for_native_scan()?;
                        event.as_ref().visit_clone_shape(&mut meter)?;
                        native::charge(
                            meter.native_scan_slots() as u64,
                            "queued_event_dispatch_scan",
                        )?;
                        let slots = meter.native_copy_slots().ok_or_else(|| {
                            native::reject(native::BattleRuntimeFault {
                                code: native::RuntimeFaultCode::CounterOverflow,
                                site: "queued_event_dispatch_slots",
                                limit: None,
                                requested: None,
                            })
                        })?;
                        native::charge_clone(slots, "queued_event_dispatch_copy")?;
                    }
                    match event.as_ref() {
                        BattleEvent::AttackResolve {
                            source_snapshot, ..
                        }
                        | BattleEvent::SkillProjectileImpact {
                            source_snapshot, ..
                        } => source_snapshot.admit_fact_owner()?,
                        _ => {}
                    }
                    controlled_clone(event.as_ref())
                }
            },
            common => Ok(common.view().into_owned()),
        }
    }
}

/// Direct fixture dispatch remains allocation-free; only persistent queue
/// insertion packs a large event into an admitted immutable owner.
pub(super) enum DispatchEvent {
    Raw(BattleEvent),
    Queued(QueuedEvent),
}
impl From<BattleEvent> for DispatchEvent {
    fn from(event: BattleEvent) -> Self {
        Self::Raw(event)
    }
}
impl From<QueuedEvent> for DispatchEvent {
    fn from(event: QueuedEvent) -> Self {
        Self::Queued(event)
    }
}

/// The timestamp bucket keeps the same geometric Vec growth and entry order,
/// but admits its physical slots before allocating or moving queued owners.
pub(super) fn reserve_timestamp_bucket_slot(
    bucket: &mut Vec<QueuedEvent>,
    site: &'static str,
) -> Result<(), crate::game::behavior::GameError> {
    if bucket.len() == bucket.capacity() {
        let capacity = crate::game::battle::runtime_control::admitted_buffer_capacity(
            bucket.len(),
            bucket.capacity(),
            control::container_capacity_limit(Resource::BucketEvents).unwrap_or(usize::MAX),
            std::mem::size_of::<QueuedEvent>(),
            64,
            site,
        )?;
        bucket.reserve_exact(capacity - bucket.len());
    }
    Ok(())
}

/// Source-free clock/attack-start events need no payload allocation. Sourced
/// entries share immutable metadata across candidate heap and bucket copies.
#[derive(Debug, Clone, Default)]
pub(super) struct QueuedFactSource(Option<Arc<RuntimeFactSource>>);
crate::authoring::clone_shape::newtype_shape!(QueuedFactSource);
impl std::ops::Deref for QueuedFactSource {
    type Target = RuntimeFactSource;
    fn deref(&self) -> &Self::Target {
        static EMPTY: RuntimeFactSource = RuntimeFactSource {
            extended_observation: None,
            use_control: false,
            reaction: false,
            flow: None,
            actor: None,
            equipment: None,
            fragment: None,
            cast: None,
            authored: None,
            action: None,
            delivery: None,
            allows_finite_children: false,
            resource_capabilities: None,
        };
        self.0.as_deref().unwrap_or(&EMPTY)
    }
}
impl QueuedFactSource {
    fn new(source: RuntimeFactSource) -> Result<Self, crate::game::behavior::GameError> {
        if source == RuntimeFactSource::default() {
            return Ok(Self::default());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(64),
            "queued_fact_owner",
        )?;
        Ok(Self(Some(Arc::new(source))))
    }
    pub(super) fn into_owned(self) -> Result<RuntimeFactSource, crate::game::behavior::GameError> {
        use crate::authoring::{
            clone_shape::{controlled_clone, CloneShape},
            control::CloneMeter,
        };
        use crate::game::battle::runtime_control as native;
        let Some(source) = self.0 else {
            return Ok(RuntimeFactSource::default());
        };
        match Arc::try_unwrap(source) {
            Ok(source) => Ok(source),
            Err(source) => {
                if native::is_active() {
                    let mut meter = CloneMeter::for_native_scan()?;
                    source.as_ref().visit_clone_shape(&mut meter)?;
                    native::charge(
                        meter.native_scan_slots() as u64,
                        "queued_dispatch_source_scan",
                    )?;
                    let slots = meter.native_copy_slots().ok_or_else(|| {
                        native::reject(native::BattleRuntimeFault {
                            code: native::RuntimeFaultCode::CounterOverflow,
                            site: "queued_dispatch_source_slots",
                            limit: None,
                            requested: None,
                        })
                    })?;
                    native::charge_clone(slots, "queued_dispatch_source_copy")?;
                }
                controlled_clone(source.as_ref())
            }
        }
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.0
            .as_ref()
            .map_or(Some(0), |source| source.retained_extent()?.checked_add(64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authoring::{
            clone_shape::controlled_clone, CancellationToken, ExecutionLimits, ExecutionPhase,
        },
        game::{
            ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
            battle::{
                buffs::BuffId, event_log::BattleEventCause, skill_source::SkillCastExecutionSource,
            },
        },
    };

    fn source(identity: u64) -> Arc<SkillEffectExecutionSource> {
        Arc::new(SkillEffectExecutionSource {
            authored: Arc::new(EffectSource {
                source_file: "queue_order_fixture".into(),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: "buff".into(),
                        action_id: "apply".into(),
                    },
                    effect_id: "one".into(),
                },
                field_path: "effects[0]".into(),
                origin_definition_id: "buff".into(),
                origin: EffectOrigin::Original,
            }),
            cast: Arc::new(SkillCastExecutionSource {
                skill_id: "buff".into(),
                caster_instance_id: uuid::Uuid::from_u128(1).into(),
                cast_seq: identity,
                ability_event_seq: identity + 1,
                cause: BattleEventCause::default(),
                proc_origin: None,
            }),
        })
    }

    fn event(time_ms: u64, duration_ms: u64) -> BattleEvent {
        BattleEvent::ApplyBuff {
            time_ms,
            duration_ms,
            caster_instance_id: uuid::Uuid::from_u128(1).into(),
            target_instance_id: uuid::Uuid::from_u128(2).into(),
            buff_id: BuffId::from_name("silence"),
            cause: BattleEventCause::default(),
        }
    }

    #[test]
    fn queued_slots_grow_by_steps_fall_back_to_minimum_and_preserve_retry_order() {
        use crate::game::battle::runtime_control::{
            self as native, BattleRuntimeFault, RuntimeFactLimits, RuntimeFaultCode,
        };
        let movement = |time_ms| BattleEvent::ContinuousMovementTick { time_ms };
        let mut queue = EventQueue::new();
        assert_eq!(queue.0.capacity(), 0);
        for time in [4, 2, 3, 1] {
            queue.push(movement(time));
        }
        assert_eq!(queue.0.capacity(), 4);
        let slot = std::mem::size_of::<QueuedEvent>();
        let source_admission = RuntimeFactSource::default().retained_extent().unwrap();
        let minimum_bytes = 5 * slot + 64;
        let before_extent = queue.slot_extent().unwrap();
        native::run(
            RuntimeFactLimits {
                max_retained_bytes: before_extent + source_admission + minimum_bytes,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(before_extent)?;
                queue.push(movement(5));
                native::poll()
            },
        )
        .unwrap();
        assert_eq!(queue.0.capacity(), 5);
        let mut copy = queue.clone();
        assert_eq!(copy.0.capacity(), copy.len());
        assert_eq!(queue.len(), 5);

        let before_extent = queue.slot_extent().unwrap();
        let error = native::run(
            RuntimeFactLimits {
                max_retained_bytes: before_extent + source_admission + 6 * slot + 63,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(before_extent)?;
                queue.push(movement(6));
                native::poll()
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            crate::game::behavior::GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedByteLimit,
                site: "queued_event_slots",
                ..
            })
        ));
        assert_eq!(queue.len(), 5);
        queue.push(movement(6));
        assert_eq!(queue.len(), 6);
        let mut raw: BinaryHeap<_> = [4, 2, 3, 1, 5, 6].map(movement).into_iter().collect();
        while let Some(expected) = raw.pop() {
            assert_eq!(queue.pop().unwrap().event.into_owned().unwrap(), expected);
        }
        assert_eq!(copy.len(), 5);
        copy.clear();
        assert_eq!(copy.0.capacity(), 5);
        drop(copy);
    }

    #[test]
    fn native_packed_queue_preserves_mixed_priority_and_admits_shared_unpack() {
        use crate::game::battle::runtime_control::{
            self as native, RuntimeFactLimits, RuntimeFaultCode,
        };
        let events = [
            event(50, 700),
            BattleEvent::ContinuousMovementTick { time_ms: 50 },
            BattleEvent::AttackStart {
                time_ms: 50,
                attacker_instance_id: uuid::Uuid::from_u128(3).into(),
                target_instance_id: None,
                schedule_next: true,
                cause: BattleEventCause::default(),
            },
            event(40, 800),
            BattleEvent::AttackStart {
                time_ms: 50,
                attacker_instance_id: uuid::Uuid::from_u128(2).into(),
                target_instance_id: Some(uuid::Uuid::from_u128(4).into()),
                schedule_next: false,
                cause: BattleEventCause::Parent { seq: 12 },
            },
        ];
        let mut raw: BinaryHeap<_> = events.iter().cloned().collect();
        let mut queue = EventQueue::new();
        for event in events {
            queue.push(event);
        }
        let mut branch = queue.clone();
        let event = branch.0.peek().unwrap().event.clone();
        assert!(matches!(event, PackedEvent::Large(_)));
        let result = native::run(
            RuntimeFactLimits {
                max_clone_slots: 1,
                ..Default::default()
            },
            || event.into_owned(),
        );
        assert!(matches!(
            result,
            Err(crate::game::behavior::GameError::BattleRuntimeFault(
                native::BattleRuntimeFault {
                    code: RuntimeFaultCode::CloneLimit,
                    ..
                }
            ))
        ));
        while let Some(expected) = raw.pop() {
            assert_eq!(branch.pop().unwrap().event.into_owned().unwrap(), expected);
            assert_eq!(queue.pop().unwrap().event.into_owned().unwrap(), expected);
        }
        assert!(branch.pop().is_none());
        assert!(queue.pop().is_none());
    }

    #[test]
    fn native_queued_fact_owner_is_lazy_shared_and_admitted_before_allocation() {
        use crate::game::battle::runtime_control::{
            self as native, RuntimeFactLimits, RuntimeFaultCode,
        };
        let mut queue = EventQueue::new();
        queue.push(event(1, 1));
        assert!(queue.0.peek().unwrap().fact_source.0.is_none());
        let source = RuntimeFactSource {
            actor: Some(uuid::Uuid::from_u128(7).into()),
            ..Default::default()
        };
        let source_bytes = source.retained_extent().unwrap();
        let result = native::run(
            RuntimeFactLimits {
                max_retained_bytes: source_bytes + 63,
                ..Default::default()
            },
            || {
                queue.push_with_context(event(2, 2), None, source.clone());
                Ok(())
            },
        );
        assert!(matches!(
            result,
            Err(crate::game::behavior::GameError::BattleRuntimeFault(
                native::BattleRuntimeFault {
                    code: RuntimeFaultCode::RetainedByteLimit,
                    site: "queued_fact_owner",
                    ..
                }
            ))
        ));
        assert_eq!(queue.len(), 1);
        queue.clear();
        queue.push_with_context(event(2, 2), None, source.clone());
        let weak = Arc::downgrade(queue.0.peek().unwrap().fact_source.0.as_ref().unwrap());
        let mut branch = queue.clone();
        queue.clear();
        assert!(weak.upgrade().is_some());
        assert_eq!(
            branch.pop().unwrap().fact_source.into_owned().unwrap(),
            source
        );
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn authoring_buff_queue_preserves_raw_priority_and_reclaims_all_metadata_owners() {
        let events = [event(60, 700), event(60, 800), event(50, 700)];
        let mut raw: BinaryHeap<_> = events.iter().cloned().collect();
        let mut queue = EventQueue::new();
        let mut weak = Vec::new();
        for event in events {
            let BattleEvent::ApplyBuff {
                time_ms,
                duration_ms,
                ..
            } = &event
            else {
                unreachable!()
            };
            let source = source(time_ms + duration_ms);
            weak.push(Arc::downgrade(&source));
            queue.push_with_source(event, Some(source));
        }
        let (cloned, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || controlled_clone(&queue),
        )
        .unwrap();
        let mut cloned = cloned.unwrap();
        queue.clear();
        assert!(weak.iter().all(|source| source.upgrade().is_some()));
        while let Some(queued) = cloned.pop() {
            assert_eq!(Some(queued.event.view().as_ref()), raw.peek());
            raw.pop();
            let event_view = queued.event.view();
            let BattleEvent::ApplyBuff {
                time_ms,
                duration_ms,
                ..
            } = event_view.as_ref()
            else {
                unreachable!()
            };
            assert_eq!(
                queued.source.as_ref().unwrap().cast.cast_seq,
                time_ms + duration_ms
            );
        }
        assert!(raw.is_empty());
        assert!(weak.iter().all(|source| source.upgrade().is_none()));

        queue.push(event(1, 1));
        let refused = source(100);
        let weak = Arc::downgrade(&refused);
        let error = control::run(
            &ExecutionLimits {
                max_pending_events: 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || queue.push_with_source(event(2, 2), Some(refused)),
        )
        .unwrap_err();
        assert_eq!(error.site, "event_enqueue");
        assert!(
            weak.upgrade().is_none(),
            "rejected insertion releases its introduced source handle"
        );
        assert_eq!(queue.iter().count(), 1);
        assert!(queue.pop().unwrap().source.is_none());
    }
}
crate::authoring::clone_shape::struct_shape!(QueuedEvent {
    close_delivery_on_dispatch,
    event,
    source,
    fact_source
});
impl QueuedEvent {
    pub(super) fn time_ms(&self) -> u64 {
        self.event.time_ms()
    }
}
impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.view().as_ref() == other.event.view().as_ref()
    }
}
impl Eq for QueuedEvent {}
impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.event.view().as_ref().cmp(other.event.view().as_ref())
    }
}
impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Preserves heap order while closing every existing enqueue family over the
/// current attempt's admission. No mutable inner heap is exposed.
#[derive(Clone, Default)]
pub(super) struct EventQueue(BinaryHeap<QueuedEvent>);

crate::authoring::clone_shape::newtype_shape!(EventQueue);

impl EventQueue {
    pub(super) fn new() -> Self {
        Self::default()
    }
    pub(super) fn push(&mut self, event: BattleEvent) {
        self.push_with_source(event, None);
    }
    pub(super) fn push_with_source(
        &mut self,
        event: BattleEvent,
        source: Option<Arc<SkillEffectExecutionSource>>,
    ) {
        self.push_with_context(event, source, Default::default());
    }
    pub(super) fn push_with_context(
        &mut self,
        event: BattleEvent,
        source: Option<Arc<SkillEffectExecutionSource>>,
        fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
    ) {
        self.push_queued(event, source, fact_source, false);
    }
    pub(super) fn push_completion(
        &mut self,
        event: BattleEvent,
        source: Option<Arc<SkillEffectExecutionSource>>,
        fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
    ) {
        self.push_queued(event, source, fact_source, true);
    }
    fn push_queued(
        &mut self,
        event: BattleEvent,
        source: Option<Arc<SkillEffectExecutionSource>>,
        fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
        close_delivery_on_dispatch: bool,
    ) {
        if control::charge(1, "event_enqueue").is_err()
            || control::admit(Resource::PendingEvents, self.0.len(), 1, "event_enqueue").is_err()
        {
            return;
        }
        if fact_source.admit_owner("queued_fact_source").is_err() {
            return;
        }
        match &event {
            BattleEvent::AttackResolve {
                source_snapshot, ..
            }
            | BattleEvent::SkillProjectileImpact {
                source_snapshot, ..
            } => {
                if source_snapshot.admit_fact_owner().is_err() {
                    return;
                }
            }
            _ => {}
        }
        let Ok(fact_source) = QueuedFactSource::new(fact_source) else {
            return;
        };
        // Keep the private heap's physical slots bounded by actual demand.
        // Reallocation preserves the heap's existing array and equal-key order.
        if self.0.len() == self.0.capacity() {
            let capacity = match crate::game::battle::runtime_control::admitted_buffer_capacity(
                self.0.len(),
                self.0.capacity(),
                control::container_capacity_limit(Resource::PendingEvents).unwrap_or(usize::MAX),
                std::mem::size_of::<QueuedEvent>(),
                64,
                "queued_event_slots",
            ) {
                Ok(capacity) => capacity,
                Err(_) => return,
            };
            self.0.reserve_exact(capacity - self.0.len());
        }
        let Ok(event) = PackedEvent::new(event) else {
            return;
        };
        self.0.push(QueuedEvent {
            close_delivery_on_dispatch,
            event,
            source,
            fact_source,
        });
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::peak(crate::runtime_observation::EVENTS, self.0.len() as u64);
    }
    pub(super) fn requeue_existing(
        &mut self,
        event: QueuedEvent,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(1, "forced_pause_requeue")?;
        control::admit(
            Resource::PendingEvents,
            self.0.len(),
            1,
            "forced_pause_requeue",
        )?;
        if self.0.len() == self.0.capacity() {
            let capacity = crate::game::battle::runtime_control::admitted_buffer_capacity(
                self.0.len(),
                self.0.capacity(),
                control::container_capacity_limit(Resource::PendingEvents).unwrap_or(usize::MAX),
                std::mem::size_of::<QueuedEvent>(),
                64,
                "forced_pause_requeue_storage",
            )?;
            self.0.reserve_exact(capacity - self.0.len());
        }
        self.0.push(event);
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::peak(crate::runtime_observation::EVENTS, self.0.len() as u64);
        Ok(())
    }
    pub(super) fn pop(&mut self) -> Option<QueuedEvent> {
        self.0.pop()
    }
    pub(super) fn peek_time_ms(&self) -> Option<u64> {
        self.0.peek().map(|queued| queued.event.time_ms())
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = Cow<'_, BattleEvent>> {
        self.0.iter().map(|queued| queued.event.view())
    }
    pub(super) fn iter_queued(&self) -> impl Iterator<Item = &QueuedEvent> {
        self.0.iter()
    }
    pub(super) fn slot_extent(&self) -> Option<usize> {
        self.0
            .capacity()
            .checked_mul(std::mem::size_of::<QueuedEvent>())
    }
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    pub(super) fn discard_channel_periodic_starts(
        &mut self,
        unit: crate::game::battle::ids::UnitInstanceId,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "channel_periodic_queue_scan")?;
        // Periodic starts own no delivery. Triggered attacks and already released
        // payloads keep their normal completion and lifetime bookkeeping.
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::AttackStart { attacker_instance_id, schedule_next: true, .. } if *attacker_instance_id == unit));
        Ok(())
    }
    /// Extract only this continuation's ready event; unrelated queue identity and
    /// ordering are preserved. The enclosing live-command candidate owns rollback.
    pub(super) fn take_reinput_step(
        &mut self,
        cast: u64,
        now: u64,
    ) -> Result<Option<QueuedEvent>, crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "reinput_step_queue_scan")?;
        let selected = self.0.iter().find(|q| matches!(q.event.view().as_ref(), BattleEvent::SkillStep { cast_seq, time_ms, .. } if *cast_seq == cast && *time_ms == now));
        let Some(selected) = selected else {
            return Ok(None);
        };
        let owned = crate::authoring::clone_shape::controlled_clone(selected)?;
        self.0.retain(|q| q != &owned);
        Ok(Some(owned))
    }
    pub(super) fn discard_skill_continuations(
        &mut self,
        cast: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "boss_action_queue_scan")?;
        self.0.retain(|q| {
            !matches!(q.event.view().as_ref(),
            BattleEvent::SkillStep { cast_seq, .. }
            | BattleEvent::SkillFlowWake { cast_seq, .. } if *cast_seq == cast)
        });
        control::poll("boss_action_queue_pruned")
    }
    pub(super) fn discard_sustained_expiry(
        &mut self,
        activation: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "sustained_expiry_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::SustainedExpiry { activation: id, .. } if *id == activation));
        Ok(())
    }
    pub(super) fn discard_independent_form_starts(
        &mut self,
        unit: crate::game::battle::ids::UnitInstanceId,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "object_form_queue_scan")?;
        self.0.retain(|queued| {
            !matches!(queued.event.view().as_ref(),
            BattleEvent::AttackStart { attacker_instance_id: id, .. }
            | BattleEvent::AutoCastStart { caster_instance_id: id, .. }
            | BattleEvent::SkillStartGate { caster_instance_id: id, .. } if *id == unit)
        });
        Ok(())
    }

    pub(super) fn discard_independent_object_expiry(
        &mut self,
        object_id: crate::game::battle::ids::UnitInstanceId,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "object_expiry_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::IndependentObjectExpiry { object_id: id, .. } if *id == object_id));
        Ok(())
    }
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
    #[cfg(test)]
    pub(super) fn into_vec(self) -> Vec<QueuedEvent> {
        self.0.into_vec()
    }
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
