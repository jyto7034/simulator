pub mod types;

use std::sync::Arc;

use crate::game::data::GameDataBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineReplayViolationKind {
    TimelineVersionMismatch,
    AttackKindMissing,
    UnknownUnitReference,
    UnknownUnitBaseReference,
    UnknownItemReference,
    UnknownArtifactReference,
    UnknownBuffReference,
    AttackDuringCast,
    AutoCastWithoutFullResonance,
    ExpectedOutcomeMissing,
    ExpectedDecisionMissing,
    UnexpectedDecision,
    UnexpectedOutcome,
    OutcomeMismatch,
    InvalidBuffEvent,
    InvalidAutoCastEvent,
}

#[derive(Debug, Clone)]
pub struct TimelineReplayViolation {
    pub kind: TimelineReplayViolationKind,
    pub message: String,
    pub entry_index: Option<usize>,
}

pub struct TimelineReplayer {
    game_data: Arc<GameDataBase>,
}
