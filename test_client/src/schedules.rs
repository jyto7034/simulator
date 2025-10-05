use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::Value;

use crate::{
    behaviors::BehaviorType,
    observer_actor::{message::EventType, Phase, PhaseCondition},
};

fn phase_matching_to_loading() -> PhaseCondition {
    PhaseCondition {
        required_events: HashSet::from([EventType::Enqueued]),
        transition_event: EventType::StartLoading,
        transition_matcher: None,
        next_phase: Phase::Loading,
    }
}

fn is_match_found_payload(data: &Value) -> bool {
    data.get("session_id").is_some() && data.get("server_address").is_some()
}

fn phase_loading_to_finished_match_found() -> PhaseCondition {
    PhaseCondition {
        required_events: HashSet::new(),
        transition_event: EventType::MatchFound,
        transition_matcher: Some(Arc::new(|data| is_match_found_payload(data))),
        next_phase: Phase::Finished,
    }
}

fn error_message_contains_any(data: &Value, needles: &[&str]) -> bool {
    if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
        let msg = msg.to_lowercase();
        needles.iter().any(|needle| msg.contains(needle))
    } else {
        true
    }
}

fn phase_loading_to_finished_error<F>(predicate: F) -> PhaseCondition
where
    F: Fn(&Value) -> bool + Send + Sync + 'static,
{
    PhaseCondition {
        required_events: HashSet::new(),
        transition_event: EventType::Error,
        transition_matcher: Some(Arc::new(predicate)),
        next_phase: Phase::Finished,
    }
}

fn phase_loading_to_finished_error_with_required(
    required: HashSet<EventType>,
    needles: &'static [&'static str],
) -> PhaseCondition {
    PhaseCondition {
        required_events: required,
        transition_event: EventType::Error,
        transition_matcher: Some(Arc::new(move |data| {
            error_message_contains_any(data, needles)
        })),
        next_phase: Phase::Finished,
    }
}

fn build_schedule_for_behavior(behavior: &BehaviorType) -> HashMap<Phase, PhaseCondition> {
    match behavior {
        BehaviorType::Normal => {
            // Normal: Matching → Finished (MatchFound 받으면 종료)
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::MatchFound,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
        BehaviorType::QuitBeforeMatch => {
            // 큐 잡히기 전 종료: Matching 단계에서 종료
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::Error,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
        BehaviorType::QuitAfterEnqueue => {
            // Enqueue 후 Dequeue: Dequeued 받으면 종료
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::Dequeued,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
        BehaviorType::Invalid { .. } => {
            // Invalid: Error 받으면 종료
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::Error,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
    }
}

pub fn get_schedule_for_perpetrator(
    perpetrator_behavior: &BehaviorType,
) -> HashMap<Phase, PhaseCondition> {
    build_schedule_for_behavior(perpetrator_behavior)
}

pub fn get_schedule_for_victim(victim_behavior: &BehaviorType) -> HashMap<Phase, PhaseCondition> {
    match victim_behavior {
        BehaviorType::Normal => {
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::MatchFound,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
        _ => {
            // 다른 behavior는 Error 또는 종료로 간주
            let mut schedule = HashMap::new();
            schedule.insert(
                Phase::Matching,
                PhaseCondition {
                    required_events: HashSet::new(),
                    transition_event: EventType::Error,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }
    }
}
