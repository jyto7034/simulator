use crate::game::enums::{OrdealType, PhaseEventType, PhaseSchedule, PhaseType};

pub struct OrdealScheduler;

impl OrdealScheduler {
    /// 시련 단계에 따른 Phase 스케줄 반환
    pub fn get_phase_schedule(ordeal: OrdealType) -> Vec<PhaseSchedule> {
        use PhaseEventType::*;
        use PhaseType::*;

        match ordeal {
            OrdealType::Dawn | OrdealType::Noon | OrdealType::White => {
                // 6 Phase 구조
                vec![
                    Self::schedule(I, EventSelection),
                    Self::schedule(II, EventSelection),
                    Self::schedule(III, Suppression),
                    Self::schedule(IV, EventSelection),
                    Self::schedule(V, EventSelection),
                    Self::schedule(VI, Ordeal),
                ]
            }
            OrdealType::Dusk | OrdealType::Midnight => {
                // 5 Phase 구조
                vec![
                    Self::schedule(I, EventSelection),
                    Self::schedule(II, EventSelection),
                    Self::schedule(III, Suppression),
                    Self::schedule(IV, EventSelection),
                    Self::schedule(V, Ordeal),
                ]
            }
        }
    }

    /// 특정 Ordeal의 특정 Phase 이벤트 타입 조회
    pub fn get_phase_event_type(ordeal: OrdealType, phase: PhaseType) -> Option<PhaseEventType> {
        Self::get_phase_schedule(ordeal)
            .into_iter()
            .find(|s| s.phase == phase)
            .map(|s| s.event_type)
    }

    /// PhaseSchedule 생성 헬퍼
    const fn schedule(phase: PhaseType, event_type: PhaseEventType) -> PhaseSchedule {
        PhaseSchedule { phase, event_type }
    }
}
