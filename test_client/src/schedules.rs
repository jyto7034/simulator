use std::collections::{HashMap, HashSet};

use crate::{
    behaviors::BehaviorType,
    observer_actor::{message::EventType, Phase, PhaseCondition},
};

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
                    required_events: HashSet::from([EventType::QueueSizeChanged]),
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
                    required_events: HashSet::from([EventType::QueueSizeChanged]),
                    transition_event: EventType::Dequeued,
                    transition_matcher: None,
                    next_phase: Phase::Finished,
                },
            );
            schedule
        }

        BehaviorType::Invalid { .. } => {
            // Invalid: Error 이벤트를 기다리고 Finished로 전환
            // Error 이벤트는 이제 Redis stream으로도 발행됨
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
    build_schedule_for_behavior(victim_behavior)
}

/*
=== 구현된 Behavior들 ===

✅ Normal: 정상 흐름
✅ QuitBeforeMatch: 연결 후 즉시 종료
✅ QuitAfterEnqueue: Enqueue 후 Dequeue

✅ Invalid { mode }:
  - InvalidGameMode: 존재하지 않는 game_mode로 Enqueue
  - LargeMetadata: 비정상적으로 큰 metadata (1MB)로 Enqueue
  - MalformedJson: 잘못된 JSON 구조로 전송
  - IdleToDequeue: Idle 상태에서 Dequeue 시도 (state machine 위반)
  - UnknownType: 존재하지 않는 메시지 타입 전송
  - MissingField: 필수 필드 누락된 메시지 전송
  - DuplicateEnqueue: Enqueued 상태에서 다시 Enqueue
  - WrongPlayerId: 다른 player_id로 Dequeue 시도

=== 구현 불가능한 Behavior ===

❌ NoHeartbeat (무응답/느린 응답):
   - tokio-tungstenite가 자동으로 ping/pong 처리
   - 클라이언트 레벨에서 pong 무시 불가능
   - 서버 timeout 테스트는 수동으로 연결 끊기로 대체 가능

=== 사용 예시 ===

// Invalid behavior 생성
BehaviorType::Invalid { mode: InvalidMode::LargeMetadata }
BehaviorType::Invalid { mode: InvalidMode::IdleToDequeue }
BehaviorType::Invalid { mode: InvalidMode::DuplicateEnqueue }
*/
