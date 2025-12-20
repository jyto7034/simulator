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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Dawn Tests
    // ============================================================

    #[test]
    fn test_dawn_schedule_length() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Dawn);
        // Then: Dawn은 6개 Phase
        assert_eq!(schedule.len(), 6);
    }

    #[test]
    fn test_dawn_schedule_order() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Dawn);

        assert_eq!(schedule[0].phase, PhaseType::I);
        assert_eq!(schedule[0].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[1].phase, PhaseType::II);
        assert_eq!(schedule[1].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[2].phase, PhaseType::III);
        assert_eq!(schedule[2].event_type, PhaseEventType::Suppression);

        assert_eq!(schedule[3].phase, PhaseType::IV);
        assert_eq!(schedule[3].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[4].phase, PhaseType::V);
        assert_eq!(schedule[4].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[5].phase, PhaseType::VI);
        assert_eq!(schedule[5].event_type, PhaseEventType::Ordeal);
    }

    // ============================================================
    // Noon Tests
    // ============================================================

    #[test]
    fn test_noon_schedule_length() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Noon);
        assert_eq!(schedule.len(), 6);
    }

    #[test]
    fn test_noon_schedule_structure() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Noon);

        // Then: Noon도 Dawn과 같은 구조
        assert_eq!(schedule[2].event_type, PhaseEventType::Suppression);
        assert_eq!(schedule[5].event_type, PhaseEventType::Ordeal);
    }

    // ============================================================
    // Dusk Tests
    // ============================================================

    #[test]
    fn test_dusk_schedule_length() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Dusk);
        // Then: Dusk는 5개 Phase
        assert_eq!(schedule.len(), 5);
    }

    #[test]
    fn test_dusk_schedule_order() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Dusk);

        assert_eq!(schedule[0].phase, PhaseType::I);
        assert_eq!(schedule[0].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[1].phase, PhaseType::II);
        assert_eq!(schedule[1].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[2].phase, PhaseType::III);
        assert_eq!(schedule[2].event_type, PhaseEventType::Suppression);

        assert_eq!(schedule[3].phase, PhaseType::IV);
        assert_eq!(schedule[3].event_type, PhaseEventType::EventSelection);

        assert_eq!(schedule[4].phase, PhaseType::V);
        assert_eq!(schedule[4].event_type, PhaseEventType::Ordeal);
    }

    // ============================================================
    // Midnight Tests
    // ============================================================

    #[test]
    fn test_midnight_schedule_length() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Midnight);
        assert_eq!(schedule.len(), 5);
    }

    #[test]
    fn test_midnight_schedule_structure() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::Midnight);

        // Then: Midnight는 Dusk와 같은 구조
        assert_eq!(schedule[2].event_type, PhaseEventType::Suppression);
        assert_eq!(schedule[4].event_type, PhaseEventType::Ordeal);
    }

    // ============================================================
    // White Tests
    // ============================================================

    #[test]
    fn test_white_schedule_length() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::White);
        assert_eq!(schedule.len(), 6);
    }

    #[test]
    fn test_white_schedule_structure() {
        let schedule = OrdealScheduler::get_phase_schedule(OrdealType::White);

        // Then: White는 Dawn, Noon과 같은 구조
        assert_eq!(schedule[2].event_type, PhaseEventType::Suppression);
        assert_eq!(schedule[5].event_type, PhaseEventType::Ordeal);
    }

    // ============================================================
    // get_phase_event_type Tests
    // ============================================================

    #[test]
    fn test_get_phase_event_type_dawn() {
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::I),
            Some(PhaseEventType::EventSelection)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::VI),
            Some(PhaseEventType::Ordeal)
        );
    }

    #[test]
    fn test_get_phase_event_type_dusk() {
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dusk, PhaseType::I),
            Some(PhaseEventType::EventSelection)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dusk, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dusk, PhaseType::V),
            Some(PhaseEventType::Ordeal)
        );
    }

    #[test]
    fn test_get_phase_event_type_invalid_phase() {
        // NOTE: Dawn은 스케줄이 I~VI까지 정의되어 있어서 Phase VI도 Some을 반환해야 함
        assert!(OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::VI).is_some());
    }

    // ============================================================
    // Cross-Ordeal Consistency Tests
    // ============================================================

    #[test]
    fn test_all_ordeals_have_suppression_at_phase_iii() {
        // Then: 모든 시련은 Phase III에 진압 작업이 있어야 함
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Noon, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dusk, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Midnight, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::White, PhaseType::III),
            Some(PhaseEventType::Suppression)
        );
    }

    #[test]
    fn test_six_phase_ordeals_have_ordeal_at_phase_vi() {
        // Then: 6 Phase 시련들은 Phase VI에 시련 전투가 있어야 함
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dawn, PhaseType::VI),
            Some(PhaseEventType::Ordeal)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Noon, PhaseType::VI),
            Some(PhaseEventType::Ordeal)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::White, PhaseType::VI),
            Some(PhaseEventType::Ordeal)
        );
    }

    #[test]
    fn test_five_phase_ordeals_have_ordeal_at_phase_v() {
        // Then: 5 Phase 시련들은 Phase V에 시련 전투가 있어야 함
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Dusk, PhaseType::V),
            Some(PhaseEventType::Ordeal)
        );
        assert_eq!(
            OrdealScheduler::get_phase_event_type(OrdealType::Midnight, PhaseType::V),
            Some(PhaseEventType::Ordeal)
        );
    }
}
