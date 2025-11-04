use std::collections::HashMap;

use crate::{
    behaviors::BehaviorType,
    observer_actor::{message::EventType, EventRequirement, Phase, PhaseCondition},
};

fn build_schedule_for_behavior(behavior: &BehaviorType) -> HashMap<Phase, PhaseCondition> {
    match behavior {
        BehaviorType::Normal => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged 필수, PlayerMatchFound 받으면 Matched로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![EventRequirement::new(EventType::GlobalQueueSizeChanged)],
                    transition_event: EventRequirement::new(EventType::PlayerMatchFound),
                    next_phase: Phase::Matched,
                },
            );

            // Matched는 종료 상태 (추가 schedule 불필요)

            schedule
        }
        // Re enqueue 되면서 queue_size_changed 이벤트를 다시 받음
        // schedules 에선 이 상황에 대해 처리해둔게 없어서 re enqueue 시 enqueue 발행이 아니라, re enqueue 라는 메시지를 따로 만들어서 발행시키는게 맞을듯.
        BehaviorType::QuitBeforeMatch => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged 필수, PlayerDequeued 받으면 Dequeued로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![EventRequirement::new(EventType::GlobalQueueSizeChanged)],
                    transition_event: EventRequirement::new(EventType::PlayerDequeued),
                    next_phase: Phase::Dequeued,
                },
            );

            // Dequeued Phase: GlobalQueueSizeChanged 받으면 Finished로 전환
            schedule.insert(
                Phase::Dequeued,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::GlobalQueueSizeChanged),
                    next_phase: Phase::Finished,
                },
            );

            schedule
        }
        BehaviorType::QuitAfterEnqueue => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged 필수, PlayerDequeued 받으면 Dequeued로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![EventRequirement::new(EventType::GlobalQueueSizeChanged)],
                    transition_event: EventRequirement::new(EventType::PlayerDequeued),
                    next_phase: Phase::Dequeued,
                },
            );

            // Dequeued Phase: GlobalQueueSizeChanged 받으면 Finished로 전환
            schedule.insert(
                Phase::Dequeued,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::GlobalQueueSizeChanged),
                    next_phase: Phase::Finished,
                },
            );

            schedule
        }
        // Invalid Enqueue behaviors - Enqueued 후 잘못된 메시지로 인한 에러
        BehaviorType::InvalidEnqueueUnknownType
        | BehaviorType::InvalidEnqueueMissingField
        | BehaviorType::InvalidEnqueueDuplicate => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged, PlayerError 필수, PlayerDequeued 받으면 Dequeued로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![
                        EventRequirement::new(EventType::GlobalQueueSizeChanged),
                        EventRequirement::new(EventType::PlayerError),
                    ],
                    transition_event: EventRequirement::new(EventType::PlayerDequeued),
                    next_phase: Phase::Dequeued,
                },
            );

            // Dequeued Phase: GlobalQueueSizeChanged 받으면 Finished로 전환
            schedule.insert(
                Phase::Dequeued,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::GlobalQueueSizeChanged),
                    next_phase: Phase::Finished,
                },
            );

            schedule
        }
        // Invalid Dequeue behaviors - Enqueued 후 잘못된 Dequeue 시도로 인한 에러
        BehaviorType::InvalidDequeueUnknownType
        | BehaviorType::InvalidDequeueMissingField
        | BehaviorType::InvalidDequeueWrongPlayerId => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged, PlayerError 필수, PlayerDequeued 받으면 Dequeued로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![
                        EventRequirement::new(EventType::GlobalQueueSizeChanged),
                        EventRequirement::new(EventType::PlayerError),
                    ],
                    transition_event: EventRequirement::new(EventType::PlayerDequeued),
                    next_phase: Phase::Dequeued,
                },
            );

            // Dequeued Phase: GlobalQueueSizeChanged 받으면 Finished로 전환
            schedule.insert(
                Phase::Dequeued,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::GlobalQueueSizeChanged),
                    next_phase: Phase::Finished,
                },
            );

            schedule
        }
        // InvalidDequeueDuplicate - 정상 Dequeue 후 중복 Dequeue 시도
        BehaviorType::InvalidDequeueDuplicate => {
            let mut schedule = HashMap::new();

            // Enqueuing Phase: PlayerEnqueued 받으면 InQueue로 전환
            schedule.insert(
                Phase::Enqueuing,
                PhaseCondition {
                    required_events: vec![],
                    transition_event: EventRequirement::new(EventType::PlayerEnqueued),
                    next_phase: Phase::InQueue,
                },
            );

            // InQueue Phase: GlobalQueueSizeChanged 필수, PlayerDequeued 받으면 Dequeued로 전환
            schedule.insert(
                Phase::InQueue,
                PhaseCondition {
                    required_events: vec![EventRequirement::new(EventType::GlobalQueueSizeChanged)],
                    transition_event: EventRequirement::new(EventType::PlayerDequeued),
                    next_phase: Phase::Dequeued,
                },
            );

            // Dequeued Phase: GlobalQueueSizeChanged 받고, PlayerError 필수, 두 번째 GlobalQueueSizeChanged 받으면 Finished
            schedule.insert(
                Phase::Dequeued,
                PhaseCondition {
                    required_events: vec![
                        EventRequirement::new(EventType::GlobalQueueSizeChanged),
                        EventRequirement::new(EventType::PlayerError),
                    ],
                    transition_event: EventRequirement::new(EventType::GlobalQueueSizeChanged),
                    next_phase: Phase::Finished,
                },
            );

            schedule
        }
    }
}

pub fn get_schedule_for_normal(normal_behavior: &BehaviorType) -> HashMap<Phase, PhaseCondition> {
    build_schedule_for_behavior(normal_behavior)
}

pub fn get_schedule_for_abnormal(
    abnormal_behavior: &BehaviorType,
) -> HashMap<Phase, PhaseCondition> {
    build_schedule_for_behavior(abnormal_behavior)
}
