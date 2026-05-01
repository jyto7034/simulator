use crate::game::battle::types::PlayerDeckInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineViolationKind {
    TimelineVersionMismatch,
    MissingEntries,
    MissingBattleStart,
    MissingBattleEnd,
    NonContiguousSeq,
    TimeWentBackwards,
    AttackKindMissing,
    ParentSeqOutOfRange,
    ParentSeqInFuture,
    OutcomeMissingParent,
    AutoCastPairInvalid,
    SpawnStatsInvalid,
    StatsAfterInvalid,
    HpDeltaMismatch,
    HpBeforeMismatch,
    StatsBeforeMismatch,
    UnitDiedWhileAlive,
    BuffAppliedByDeadCaster,
    UnitSpawnCountMismatch,
    ItemSpawnCountMismatch,
    ArtifactSpawnCountMismatch,
    DuplicateUnitSpawn,
    DuplicateItemSpawn,
    DuplicateArtifactSpawn,
    UnknownUnitReference,
    UnitReferencedBeforeSpawn,
    AttackTargetsSameUnit,
    AttackTargetsAlly,
    UnitDiedDuplicate,
    DeadUnitActsAfterDeath,
    AutoAttackTooEarly,
    MissingExpectedAutoAttack,
    UnknownBuffId,
    BuffAppliedDurationZero,
    BuffTickInvalid,
    BuffExpiredInvalid,
    PositionOutOfBounds,
}

#[derive(Debug, Clone)]
pub struct TimelineViolation {
    pub kind: TimelineViolationKind,
    pub message: String,
    pub entry_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TimelineExpectedCounts {
    pub units: usize,
    pub items: usize,
    pub artifacts: usize,
}

impl TimelineExpectedCounts {
    pub fn from_decks(player: &PlayerDeckInfo, opponent: &PlayerDeckInfo) -> Self {
        let units = player.units.len() + opponent.units.len();
        let items = player
            .units
            .iter()
            .chain(opponent.units.iter())
            .map(|unit| unit.equipped_items.len())
            .sum();
        let artifacts = player.artifacts.len() + opponent.artifacts.len();
        Self {
            units,
            items,
            artifacts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineValidatorConfig {
    pub require_battle_start_end: bool,
    pub require_contiguous_seq: bool,
    pub require_non_decreasing_time: bool,
    pub require_attack_kind: bool,
    pub validate_parent_seq: bool,
    pub require_outcome_parent: bool,
    pub require_autocast_pairs: bool,
    pub require_spawn_stats_valid: bool,
    pub require_hp_delta_consistent: bool,
    pub forbid_dead_units_as_attackers: bool,
    pub forbid_dead_units_as_targets: bool,
    pub validate_positions_within_bounds: bool,
    pub require_movement_path_consistent: bool,
    pub validate_auto_attack_min_interval: bool,
    pub validate_auto_attack_presence: bool,
    pub auto_attack_timing_tolerance_ms: u64,
}

impl Default for TimelineValidatorConfig {
    fn default() -> Self {
        Self {
            require_battle_start_end: true,
            require_contiguous_seq: true,
            require_non_decreasing_time: true,
            require_attack_kind: true,
            validate_parent_seq: true,
            require_outcome_parent: true,
            require_autocast_pairs: true,
            require_spawn_stats_valid: true,
            require_hp_delta_consistent: true,
            forbid_dead_units_as_attackers: true,
            forbid_dead_units_as_targets: true,
            validate_positions_within_bounds: true,
            require_movement_path_consistent: true,
            validate_auto_attack_min_interval: true,
            validate_auto_attack_presence: false,
            auto_attack_timing_tolerance_ms: 2,
        }
    }
}
