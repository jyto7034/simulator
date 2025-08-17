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
        BehaviorType::Normal | BehaviorType::SlowLoader { .. } | BehaviorType::SpikyLoader { .. } => {
            let mut schedule = HashMap::new();
            schedule.insert(Phase::Matching, phase_matching_to_loading());
            schedule.insert(Phase::Loading, phase_loading_to_finished_match_found());
            schedule
        }
        BehaviorType::TimeoutLoader | BehaviorType::QuitDuringLoading => {
            let mut schedule = HashMap::new();
            schedule.insert(Phase::Matching, phase_matching_to_loading());
            schedule.insert(
                Phase::Loading,
                phase_loading_to_finished_error_with_required(
                    HashSet::new(),
                    &["quit", "disconnect", "timeout"],
                ),
            );
            schedule
        }
        BehaviorType::QuitBeforeMatch => {
            // 큐 잡히기 전 종료: Loading 단계로 가지 않으며, Error/Dequeued 등을 관찰하고 종료로 간주
            let mut schedule = HashMap::new();
            // Matching 단계에서 에러가 오면 Finished로 전환하는 간단한 정책
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
        BehaviorType::Invalid { .. } => {
            let mut schedule = HashMap::new();
            schedule.insert(Phase::Matching, phase_matching_to_loading());
            schedule.insert(
                Phase::Loading,
                phase_loading_to_finished_error_with_required(
                    HashSet::new(),
                    &["error", "invalid", "bad", "timeout"],
                ),
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
        BehaviorType::Normal | BehaviorType::SlowLoader { .. } => {
            let mut schedule = HashMap::new();
            schedule.insert(Phase::Matching, phase_matching_to_loading());
            schedule.insert(Phase::Loading, phase_loading_to_finished_match_found());
            schedule
        }
        _ => {
            let mut schedule = HashMap::new();
            schedule.insert(Phase::Matching, phase_matching_to_loading());
            schedule.insert(
                Phase::Loading,
                phase_loading_to_finished_error(|_data| true),
            );
            schedule
        }
    }
}
