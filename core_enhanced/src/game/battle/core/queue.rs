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

/// Small events do not reserve the largest BattleEvent variant in
/// every heap/bucket slot. All comparisons still use the original event value.
#[derive(Debug, Clone)]
pub(super) enum PackedEvent {
    WaveTriggerWake {
        time_ms: u64,
        trigger_id: Arc<String>,
    },
    SpawnGroup {
        time_ms: u64,
        group_id: Arc<crate::game::battle::scenario::ScenarioGroupId>,
        cause: crate::game::battle::event_log::BattleEventCause,
    },
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
            BattleEvent::WaveTriggerWake {
                time_ms,
                trigger_id,
            } => {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    BattleEvent::wave_trigger_id_extent(trigger_id.capacity()),
                    "queued_wave_trigger_payload",
                )?;
                Self::WaveTriggerWake {
                    time_ms,
                    trigger_id,
                }
            }
            BattleEvent::SpawnGroup {
                time_ms,
                group_id,
                cause,
            } => {
                // A caller may supply its own raw event. Admit this retained
                // owner even when it did not use the internal constructor.
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    group_id.retained_extent(),
                    "queued_spawn_group_payload",
                )?;
                Self::SpawnGroup {
                    time_ms,
                    group_id,
                    cause,
                }
            }
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
            Self::WaveTriggerWake {
                time_ms,
                trigger_id,
            } => Cow::Owned(BattleEvent::WaveTriggerWake {
                time_ms: *time_ms,
                trigger_id: Arc::clone(trigger_id),
            }),
            Self::SpawnGroup {
                time_ms,
                group_id,
                cause,
            } => Cow::Owned(BattleEvent::SpawnGroup {
                time_ms: *time_ms,
                group_id: Arc::clone(group_id),
                cause: *cause,
            }),
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
            Self::WaveTriggerWake { time_ms, .. }
            | Self::SpawnGroup { time_ms, .. }
            | Self::MovementTick { time_ms }
            | Self::AttackStart { time_ms, .. } => *time_ms,
            Self::Large(event) => event.time_ms(),
        }
    }
    pub(super) fn payload_extent(&self) -> Option<usize> {
        match self {
            Self::WaveTriggerWake { trigger_id, .. } => {
                BattleEvent::wave_trigger_id_extent(trigger_id.capacity())
            }
            Self::SpawnGroup { group_id, .. } => group_id.retained_extent(),
            Self::Large(_) => std::mem::size_of::<BattleEvent>().checked_add(64),
            _ => Some(0),
        }
    }
    pub(super) fn into_owned(self) -> Result<BattleEvent, crate::game::behavior::GameError> {
        use crate::authoring::{
            clone_shape::{controlled_clone, CloneShape},
            control::CloneMeter,
        };
        use crate::game::battle::runtime_control as native;
        match self {
            Self::WaveTriggerWake {
                time_ms,
                trigger_id,
            } => Ok(BattleEvent::WaveTriggerWake {
                time_ms,
                trigger_id,
            }),
            Self::SpawnGroup {
                time_ms,
                group_id,
                cause,
            } => Ok(BattleEvent::SpawnGroup {
                time_ms,
                group_id,
                cause,
            }),
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
                        | BattleEvent::BasicAttackTileImpact { source_snapshot, .. }
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
            death_burst: false,
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
            | BattleEvent::BasicAttackTileImpact { source_snapshot, .. }
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
    pub(super) fn retain_causal_periodics(
        &mut self,
        eligible: impl Fn(&RuntimeFactSource) -> bool,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "causal_periodic_reconcile_queue")?;
        self.0.retain(|q| {
            !matches!(q.event.view().as_ref(), BattleEvent::CausalPeriodic { .. })
                || eligible(&q.fact_source)
        });
        Ok(())
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
    // Only status areas call this: their dwell events have no completion owner.
    pub(super) fn discard_ground_dwell_ticks(&mut self, area: uuid::Uuid) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "ground_dwell_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::SkillAreaTick { area_id, .. } if *area_id == area));
        Ok(())
    }
    pub(super) fn take_basic_tile_impact(
        &mut self,
    ) -> Result<Option<QueuedEvent>, crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "basic_tile_end_queue_scan")?;
        let selected = self.0.iter().filter(|q| matches!(q.event.view().as_ref(),
            BattleEvent::BasicAttackTileImpact { .. })).max();
        let Some(selected) = selected else { return Ok(None); };
        let owned = crate::authoring::clone_shape::controlled_clone(selected)?;
        self.0.retain(|q| q != &owned);
        Ok(Some(owned))
    }
    pub(super) fn take_area_event(
        &mut self, area: uuid::Uuid,
    ) -> Result<Option<QueuedEvent>, crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "area_end_queue_scan")?;
        let selected = self.0.iter().find(|q| matches!(q.event.view().as_ref(),
            BattleEvent::SkillAreaTick { area_id, .. } | BattleEvent::SkillAreaExpire { area_id, .. } if *area_id == area));
        let Some(selected) = selected else { return Ok(None); };
        let owned = crate::authoring::clone_shape::controlled_clone(selected)?;
        self.0.retain(|q| q != &owned);
        Ok(Some(owned))
    }
    pub(super) fn discard_counter_events(&mut self, owner: Option<uuid::Uuid>, gains: bool, channels: bool) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "counter_queue_scan")?;
        self.0.retain(|q| match q.event.view().as_ref() {
            BattleEvent::CounterGain { source, .. } => !(gains && owner.is_none_or(|o| o == *source)),
            BattleEvent::CounterChannelTick { source, .. } => !(channels && owner.is_none_or(|o| o == *source)),
            _ => true,
        });
        Ok(())
    }
    pub(super) fn discard_status_mobile_tick(&mut self)->Result<(),crate::game::behavior::GameError>{control::charge(self.0.len() as u64,"chase_queue_scan")?;self.0.retain(|q|!matches!(q.event.view().as_ref(),BattleEvent::StatusMobileTick{..}));Ok(())}
    pub(super) fn discard_mobile_tick(&mut self)->Result<(),crate::game::behavior::GameError>{control::charge(self.0.len() as u64,"mobile_queue_scan")?;self.0.retain(|q|!matches!(q.event.view().as_ref(),BattleEvent::MobileEffectsTick{..}));Ok(())}
    pub(super) fn discard_pair_tick(&mut self,id:Option<u64>)->Result<(),crate::game::behavior::GameError>{control::charge(self.0.len() as u64,"paired_queue_scan")?;self.0.retain(|q|!matches!(q.event.view().as_ref(),BattleEvent::PairedAreaTick{id:found,..} if id.is_none_or(|id|id==*found)));Ok(())}
    pub(super) fn discard_prepared_volley_events(&mut self,owner:Option<uuid::Uuid>,flights:bool)->Result<(),crate::game::behavior::GameError>{
        control::charge(self.0.len()as u64,"prepared_volley_queue_scan")?;
        self.0.retain(|q|match q.event.view().as_ref(){BattleEvent::PreparedVolleyTick{source,..}=>!owner.is_none_or(|o|o==*source),BattleEvent::VolleyFlightTick{..}=>!flights,_=>true});Ok(())
    }
    pub(super) fn discard_shared_cohort_events(&mut self) -> Result<(),crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "shared_cohort_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::HostileCohortTick{..}));
        Ok(())
    }
    pub(super) fn discard_input_plan_events(&mut self)->Result<(),crate::game::behavior::GameError>{
        control::charge(self.0.len()as u64,"shot_queue_scan")?;
        self.0.retain(|q|!matches!(q.event.view().as_ref(),BattleEvent::ShotStockTick{..}|BattleEvent::ShotFlightTick{..}));Ok(())
    }
    pub(super) fn take_immediate_action_event(&mut self,action:&crate::game::battle::runtime_facts::ActionHandle,caster:crate::game::battle::ids::UnitInstanceId,start_seq:u64,cast:Option<u64>,now:u64)->Result<Option<QueuedEvent>,crate::game::behavior::GameError>{
        control::charge(self.0.len()as u64,"shot_immediate_action_queue")?;
        let selected=self.0.iter().filter(|q|q.event.time_ms()==now&&(q.fact_source.action.as_ref()==Some(action)||matches!(q.event.view().as_ref(),BattleEvent::SkillStep{cast_seq,..}|BattleEvent::SkillFlowWake{cast_seq,..} if Some(*cast_seq)==cast)||matches!(q.event.view().as_ref(),BattleEvent::ManualCastEnd{caster_instance_id,cause,..}|BattleEvent::SkillStartGate{caster_instance_id,cause,..} if *caster_instance_id==caster && cause.parent_seq()==Some(start_seq)))).max();
        let Some(selected)=selected else{return Ok(None);};let owned=crate::authoring::clone_shape::controlled_clone(selected)?;self.0.retain(|q|q!=&owned);Ok(Some(owned))
    }
    pub(super) fn discard_relation_events(&mut self, owner: Option<uuid::Uuid>, gains: bool, channels: bool) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "relation_queue_scan")?;
        self.0.retain(|q| match q.event.view().as_ref() {
            BattleEvent::RelationGain { source, .. } => !(gains && owner.is_none_or(|o| o == *source)),
            BattleEvent::RelationActionTick { source, .. } => !(channels && owner.is_none_or(|o| o == *source)),
            _ => true,
        });
        Ok(())
    }
    pub(super) fn discard_equipment_death_bursts(&mut self, ids: Option<&std::collections::BTreeSet<u64>>) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "death_burst_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::EquipmentDeathBurst { id, .. } if ids.is_none_or(|ids| ids.contains(id))));
        Ok(())
    }
    pub(super) fn discard_tile_bursts(&mut self) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "tile_burst_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::TileBurstWave { .. }));
        Ok(())
    }
    pub(super) fn discard_presence_ticks(&mut self, source: Option<uuid::Uuid>, host: Option<crate::game::battle::ids::UnitInstanceId>) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "presence_tick_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::PresenceTick { source: id, host: target, .. } if source.is_none_or(|s|s==*id) && host.is_none_or(|h|h==*target)));
        Ok(())
    }
    pub(super) fn discard_sustained_expiry(
        &mut self,
        activation: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        control::charge(self.0.len() as u64, "sustained_expiry_queue_scan")?;
        self.0.retain(|q| !matches!(q.event.view().as_ref(), BattleEvent::SustainedExpiry { activation: id, .. } | BattleEvent::ChannelUpkeep { activation: id, .. } | BattleEvent::EquipmentStateTick { activation: id, .. } if *id == activation));
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

#[cfg(test)]
mod spawn_packing_tests {
    use super::*;
    use crate::authoring::{
        clone_shape::controlled_clone, control, CancellationToken, ExecutionLimits, ExecutionPhase,
    };
    use crate::game::battle::{
        event_log::BattleEventCause,
        runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode},
        scenario::ScenarioGroupId,
    };

    #[test]
    fn compact_spawn_queue_preserves_raw_order_shares_ids_and_releases_forks() {
        let spawn = |time, id: &str| {
            BattleEvent::spawn_group(time, ScenarioGroupId::new(id), BattleEventCause::default())
                .unwrap()
        };
        let long = spawn(10, &"group".repeat(1000));
        let BattleEvent::SpawnGroup { group_id, .. } = &long else {
            unreachable!()
        };
        let weak = Arc::downgrade(group_id);
        let events = vec![
            spawn(10, "z"),
            spawn(10, "a"),
            long,
            BattleEvent::ContinuousMovementTick { time_ms: 10 },
            spawn(9, "earlier"),
            BattleEvent::wave_trigger_wake(10, "wave-z").unwrap(),
            BattleEvent::wave_trigger_wake(10, "wave-a").unwrap(),
        ];
        let mut raw: BinaryHeap<_> = events.iter().cloned().collect();
        let mut queue = EventQueue::new();
        for event in events {
            queue.push(event);
        }
        assert!(
            queue
                .iter_queued()
                .filter(|q| matches!(q.event, PackedEvent::SpawnGroup { .. }))
                .count()
                == 4
        );
        let (fork, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || controlled_clone(&queue),
        )
        .unwrap();
        let mut fork = fork.unwrap();
        while let Some(expected) = raw.pop() {
            for q in [&mut queue, &mut fork] {
                let actual = q.pop().unwrap().event.into_owned().unwrap();
                if let (
                    BattleEvent::SpawnGroup { group_id: a, .. },
                    BattleEvent::SpawnGroup { group_id: b, .. },
                ) = (&actual, &expected)
                {
                    assert!(Arc::ptr_eq(a, b));
                }
                if let (
                    BattleEvent::WaveTriggerWake { trigger_id: a, .. },
                    BattleEvent::WaveTriggerWake { trigger_id: b, .. },
                ) = (&actual, &expected)
                {
                    assert!(Arc::ptr_eq(a, b));
                }
                assert_eq!(actual, expected);
            }
        }
        assert_eq!(queue.len(), 0);
        assert_eq!(fork.len(), 0);
        assert!(weak.upgrade().is_none());
        let small = PackedEvent::new(spawn(0, "small")).unwrap();
        assert!(small.payload_extent().unwrap() < std::mem::size_of::<BattleEvent>());
    }

    #[test]
    fn compact_spawn_owner_and_queue_admit_full_long_id_before_acceptance() {
        let id = ScenarioGroupId::new("bounded-group".repeat(1000));
        let extent = id.retained_extent().unwrap();
        for (cap, good) in [(extent - 1, false), (extent, true)] {
            let result = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || BattleEvent::spawn_group(7, id.clone(), BattleEventCause::default()),
            );
            if good {
                result.unwrap();
            } else {
                assert!(
                    matches!(result, Err(crate::game::behavior::GameError::BattleRuntimeFault(f))
                if f.code == RuntimeFaultCode::RetainedByteLimit && f.site == "spawn_group_id_owner")
                );
            }
        }
        let event = BattleEvent::spawn_group(7, id, BattleEventCause::default()).unwrap();
        for (cap, good) in [(extent - 1, false), (extent, true)] {
            let result = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || PackedEvent::new(event.clone()),
            );
            if good {
                assert_eq!(result.unwrap().payload_extent(), Some(extent));
            } else {
                assert!(
                    matches!(result, Err(crate::game::behavior::GameError::BattleRuntimeFault(f))
                if f.code == RuntimeFaultCode::RetainedByteLimit && f.site == "queued_spawn_group_payload")
                );
            }
        }
        let run = || {
            control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || PackedEvent::new(event.clone()),
            )
        };
        let cancelled = control::with_test_cancellation("queued_spawn_group_payload", 1, run);
        assert_eq!(cancelled.err().unwrap().code, "authoring_cancelled");
        run().unwrap().0.unwrap();
    }
    #[test]
    fn compact_wave_owner_and_queue_admit_full_id_and_release_after_cancellation() {
        let id = "bounded-wave".repeat(1000);
        let extent = BattleEvent::wave_trigger_id_extent(id.len()).unwrap();
        for (cap, good) in [(extent - 1, false), (extent, true)] {
            let result = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || BattleEvent::wave_trigger_wake(7, &id),
            );
            if good {
                result.unwrap();
            } else {
                assert!(
                    matches!(result, Err(crate::game::behavior::GameError::BattleRuntimeFault(f))
                if f.code == RuntimeFaultCode::RetainedByteLimit && f.site == "wave_trigger_id_owner")
                );
            }
        }
        let event = BattleEvent::wave_trigger_wake(7, &id).unwrap();
        let BattleEvent::WaveTriggerWake { trigger_id, .. } = &event else {
            unreachable!()
        };
        let weak = Arc::downgrade(trigger_id);
        for (cap, good) in [(extent - 1, false), (extent, true)] {
            let result = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || PackedEvent::new(event.clone()),
            );
            if good {
                assert_eq!(result.unwrap().payload_extent(), Some(extent));
            } else {
                assert!(
                    matches!(result, Err(crate::game::behavior::GameError::BattleRuntimeFault(f))
                if f.code == RuntimeFaultCode::RetainedByteLimit && f.site == "queued_wave_trigger_payload")
                );
            }
        }
        let run = || {
            control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || PackedEvent::new(event.clone()),
            )
        };
        let cancelled = control::with_test_cancellation("queued_wave_trigger_payload", 1, run);
        assert_eq!(cancelled.err().unwrap().code, "authoring_cancelled");
        run().unwrap().0.unwrap();
        drop(event);
        assert!(weak.upgrade().is_none());
    }
}
