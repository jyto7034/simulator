use crate::ecs::resources::GameProgression;
use crate::game::enums::{MoveTo, OrdealType, PhaseType};

/// Phase 진행 (성공 시 새 Phase 반환)
pub fn advance_phase(progression: &mut GameProgression) -> Option<PhaseType> {
    let max_phases = progression.current_ordeal.max_phases();

    if let Some(next) = progression.current_phase.next() {
        if next.value() <= max_phases {
            progression.current_phase = next;
            return Some(next);
        }
    }
    None
}

/// Ordeal 진행 (성공 시 새 Ordeal 반환)
pub fn advance_ordeal(progression: &mut GameProgression) -> Option<OrdealType> {
    if let Some(next) = progression.current_ordeal.next() {
        progression.current_ordeal = next;
        progression.current_phase = PhaseType::I;
        return Some(next);
    }
    None
}

/// Phase와 Ordeal을 순차적으로 진행
pub fn advance(progression: &mut GameProgression) -> ProgressionResult {
    // Phase 먼저 시도
    if let Some(next_phase) = advance_phase(progression) {
        return ProgressionResult::NextPhase(next_phase);
    }

    // Phase 완료 → Ordeal 진행
    if let Some(next_ordeal) = advance_ordeal(progression) {
        return ProgressionResult::NextOrdeal(next_ordeal);
    }

    // 게임 종료
    ProgressionResult::GameComplete
}

#[derive(Debug, Clone)]
pub enum ProgressionResult {
    NextPhase(PhaseType),
    NextOrdeal(OrdealType),
    GameComplete,
}
