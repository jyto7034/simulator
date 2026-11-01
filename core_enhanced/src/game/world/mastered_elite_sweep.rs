use crate::game::behavior::{ActionKind, ActionDisabledReasonCode, BehaviorResult, GameError, MasteredEliteResolutionDto, MasteredEliteSweepResultDto};
use super::GameCore;

pub(super) struct MasteredEliteResolutionEvaluation { pub(super) dto: MasteredEliteResolutionDto }
impl GameCore {
    pub(super) fn mastered_elite_resolution_evaluation(&self) -> Option<MasteredEliteResolutionEvaluation> { None }
    pub(super) fn mastered_elite_resolution_dto(&self) -> Option<MasteredEliteResolutionDto> { None }
    pub(super) fn staged_mastered_elite_sweep_result(&self) -> Option<&MasteredEliteSweepResultDto> { None }
    pub(super) fn retired_encounter_action(action: ActionKind) -> GameError {
        GameError::ActionUnavailable { action, reason: ActionDisabledReasonCode::RetiredFeature }
    }
    pub(super) fn handle_sweep_mastered_elite(&mut self) -> Result<BehaviorResult, GameError> {
        Err(Self::retired_encounter_action(ActionKind::SweepMasteredElite))
    }
    pub(super) fn handle_complete_mastered_elite_sweep(&mut self) -> Result<BehaviorResult, GameError> {
        Err(Self::retired_encounter_action(ActionKind::CompleteMasteredEliteSweep))
    }
}
