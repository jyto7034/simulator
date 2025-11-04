use crate::game::enums::{OrdealType, PhaseEventType, PhaseSchedule};

pub struct OrdealScheduler;

impl OrdealScheduler {
    /// 시련 단계에 따른 Phase 리스트 반환 (구조만 정의)
    pub fn get_phase_schedule(ordeal: OrdealType) -> Vec<PhaseSchedule> {
        match ordeal {
            OrdealType::Dawn | OrdealType::Noon | OrdealType::White => {
                // 6 Phase 구조
                vec![
                    PhaseSchedule {
                        phase_number: 1,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 2,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 3,
                        event_type: PhaseEventType::Suppression,
                    },
                    PhaseSchedule {
                        phase_number: 4,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 5,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 6,
                        event_type: PhaseEventType::Ordeal,
                    },
                ]
            }
            OrdealType::Dusk | OrdealType::Midnight => {
                // 5 Phase 구조
                vec![
                    PhaseSchedule {
                        phase_number: 1,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 2,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 3,
                        event_type: PhaseEventType::Suppression,
                    },
                    PhaseSchedule {
                        phase_number: 4,
                        event_type: PhaseEventType::EventSelection,
                    },
                    PhaseSchedule {
                        phase_number: 5,
                        event_type: PhaseEventType::Ordeal, // 시련
                    },
                ]
            }
        }
    }
}
