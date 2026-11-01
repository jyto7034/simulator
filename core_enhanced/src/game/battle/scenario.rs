use std::collections::HashSet;

use crate::game::{
    battle::types::{BattleOutcomeId, BattleUnitDraft, BattleVerdict, TacticalTileKind},
    behavior::GameError,
    data::boss_runtime_data::{BossSignalId, ResolvedBossEncounterSetup},
    enums::Side,
    resources::Position,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScenarioGroupId(pub String);

impl ScenarioGroupId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.0.capacity())?
            .checked_add(64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScenarioEventId(pub String);

impl ScenarioEventId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScenarioUnitRef(pub String);

impl ScenarioUnitRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldWorkObjectId(pub String);

impl FieldWorkObjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldWorkCellId(pub String);

impl FieldWorkCellId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldWorkPresentationKind {
    EmergencyRepairTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldWorkCellDef {
    pub id: FieldWorkCellId,
    pub position: Position,
    pub tile_kind: TacticalTileKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldWorkCompletionEffect {
    RepairProtectedUnit {
        unit_ref: ScenarioUnitRef,
        flat_hp: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldWorkObjectDef {
    pub id: FieldWorkObjectId,
    pub presentation_kind: FieldWorkPresentationKind,
    pub position: Position,
    pub work_cells: Vec<FieldWorkCellDef>,
    pub required_work_ms: u64,
    pub completion_effect: FieldWorkCompletionEffect,
}

#[derive(Debug, Clone)]
pub struct BattleFieldSpec {
    /// Union of every authored path, independent of spawned waves.
    pub walkable_paths: Vec<Position>,
    pub width: u8,
    pub height: u8,
    /// Empty means every tile inside width x height is valid.
    /// Non-empty enables non-rectangular battlefields authored from ASCII templates.
    pub valid_tiles: Vec<Position>,
    pub obstacles: Vec<Position>,
    /// Authoritative terrain for independent ground objects. Absence means
    /// this scenario does not publish that placement capability.
    pub object_placement: Option<IndependentPlacementDomain>,
}

#[derive(Debug, Clone, Default)]
pub struct IndependentPlacementDomain {
    pub ground: Vec<Position>,
    pub reserved: Vec<Position>,
}
crate::authoring::clone_shape::struct_shape!(IndependentPlacementDomain { ground, reserved });

#[derive(Debug, Clone)]
pub struct ScenarioUnitSpawn {
    pub unit_ref: ScenarioUnitRef,
    pub side: Side,
    pub draft: BattleUnitDraft,
    pub position: Position,
    pub instance_salt: u32,
}

#[derive(Debug, Clone)]
pub struct ScenarioSpawnGroup {
    pub id: ScenarioGroupId,
    pub side: Side,
    pub required_for_victory: bool,
    pub enemy_movement_plan: Option<EnemyMovementPlan>,
    pub spawns: Vec<ScenarioUnitSpawn>,
}

#[derive(Debug, Clone)]
pub enum ScenarioTrigger {
    AtBattleStart,
    AtTimeMs(u64),
    Wave(ScenarioWaveActivation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioWaveActivation {
    pub trigger_id: String,
    pub priority: i32,
    pub primary: ScenarioWavePrimaryTrigger,
    pub deadline_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioWavePrimaryTrigger {
    AtTimeMs {
        time_ms: u64,
    },
    GroupCleared {
        group_id: ScenarioGroupId,
    },
    ActorHealthAtOrBelowPercent {
        actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId,
        percent: u8,
    },
    ActorDefeated {
        actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId,
    },
    ActorReachedPoint {
        actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId,
        point_id: TacticalPointId,
        radius_cells: u32,
    },
}

#[derive(Debug, Clone)]
pub enum ScenarioAction {
    SpawnGroup { group_id: ScenarioGroupId },
}

#[derive(Debug, Clone)]
pub struct ScenarioEvent {
    pub id: ScenarioEventId,
    pub trigger: ScenarioTrigger,
    pub action: ScenarioAction,
}

#[derive(Debug, Clone)]
pub enum TerminalCondition {
    AllRequiredEnemyGroupsDefeated,
    UnitDefeated { unit_ref: ScenarioUnitRef },
    AllPlayerCombatantsDefeated,
    TimeReached { time_ms: u64 },
    BossSignalLatched { signal_id: BossSignalId },
}

#[derive(Debug, Clone)]
pub struct TerminalOutcome {
    pub id: BattleOutcomeId,
    pub verdict: BattleVerdict,
    pub condition: TerminalCondition,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TacticalPointId(pub String);

impl TacticalPointId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct TacticalPoint {
    pub id: TacticalPointId,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub enum BattleObjective {
    DefeatBoss { unit_ref: ScenarioUnitRef },
    ProtectUnit { unit_ref: ScenarioUnitRef },
    Survive { time_ms: u64 },
}

#[derive(Debug, Clone)]
pub enum PlayerMovementPlan {
    FixedDefense,
}

#[derive(Debug, Clone)]
pub enum EnemyMovementPlan {
    PathAlongCells { cells: Vec<Position> },
}

crate::authoring::clone_shape::enum_shape!(EnemyMovementPlan {
    units: [],
    tuples: [],
    records: [PathAlongCells { cells }]
});

#[derive(Debug, Clone)]
pub struct TacticalPlan {
    pub objective: BattleObjective,
    pub player_plan: PlayerMovementPlan,
    pub enemy_plan: EnemyMovementPlan,
    pub points: Vec<TacticalPoint>,
    pub field_work_objects: Vec<FieldWorkObjectDef>,
}

impl TacticalPlan {
    pub fn fixed_defense() -> Self {
        Self {
            objective: BattleObjective::Survive { time_ms: 0 },
            player_plan: PlayerMovementPlan::FixedDefense,
            enemy_plan: EnemyMovementPlan::PathAlongCells { cells: Vec::new() },
            points: Vec::new(),
            field_work_objects: Vec::new(),
        }
    }

    pub fn point(&self, point_id: &TacticalPointId) -> Option<&TacticalPoint> {
        self.points.iter().find(|point| &point.id == point_id)
    }
}

impl Default for TacticalPlan {
    fn default() -> Self {
        Self::fixed_defense()
    }
}

#[derive(Debug, Clone)]
pub struct BattleScenario {
    pub battlefield: BattleFieldSpec,
    pub groups: Vec<ScenarioSpawnGroup>,
    pub events: Vec<ScenarioEvent>,
    pub terminal_outcomes: Vec<TerminalOutcome>,
    pub tactical_plan: TacticalPlan,
    pub boss_runtime: Option<ResolvedBossEncounterSetup>,
}

crate::authoring::clone_shape::newtype_shape!(
    ScenarioGroupId,
    ScenarioEventId,
    ScenarioUnitRef,
    FieldWorkObjectId,
    FieldWorkCellId,
    TacticalPointId
);
crate::authoring::clone_shape::copy_shape!(FieldWorkPresentationKind);
crate::authoring::clone_shape::struct_shape!(BattleScenario {
    battlefield,
    groups,
    events,
    terminal_outcomes,
    tactical_plan,
    boss_runtime,
});
crate::authoring::clone_shape::struct_shape!(BattleFieldSpec {
    walkable_paths,
    width,
    height,
    valid_tiles,
    obstacles,
    object_placement
});
crate::authoring::clone_shape::struct_shape!(ScenarioUnitSpawn {
    unit_ref,
    side,
    draft,
    position,
    instance_salt
});
crate::authoring::clone_shape::struct_shape!(ScenarioSpawnGroup {
    id,
    side,
    required_for_victory,
    enemy_movement_plan,
    spawns,
});
crate::authoring::clone_shape::struct_shape!(ScenarioEvent {
    id,
    trigger,
    action
});
crate::authoring::clone_shape::struct_shape!(ScenarioWaveActivation {
    trigger_id,
    priority,
    primary,
    deadline_ms
});
crate::authoring::clone_shape::struct_shape!(TerminalOutcome {
    id,
    verdict,
    condition,
    priority
});
crate::authoring::clone_shape::struct_shape!(TacticalPoint { id, position });
crate::authoring::clone_shape::struct_shape!(TacticalPlan {
    objective,
    player_plan,
    enemy_plan,
    points,
    field_work_objects
});
crate::authoring::clone_shape::struct_shape!(FieldWorkCellDef {
    id,
    position,
    tile_kind
});
crate::authoring::clone_shape::struct_shape!(FieldWorkObjectDef {
    id,
    presentation_kind,
    position,
    work_cells,
    required_work_ms,
    completion_effect,
});
crate::authoring::clone_shape::enum_shape!(ScenarioTrigger {
    units: [AtBattleStart],
    tuples: [AtTimeMs(time), Wave(activation)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(ScenarioWavePrimaryTrigger {
    units: [],
    tuples: [],
    records: [
        AtTimeMs { time_ms },
        GroupCleared { group_id },
        ActorHealthAtOrBelowPercent {
            actor_slot_id,
            percent
        },
        ActorDefeated { actor_slot_id },
        ActorReachedPoint {
            actor_slot_id,
            point_id,
            radius_cells
        }
    ]
});
crate::authoring::clone_shape::enum_shape!(ScenarioAction {
    units: [],
    tuples: [],
    records: [SpawnGroup { group_id }]
});
crate::authoring::clone_shape::enum_shape!(TerminalCondition {
    units: [AllRequiredEnemyGroupsDefeated, AllPlayerCombatantsDefeated],
    tuples: [],
    records: [
        UnitDefeated { unit_ref },
        TimeReached { time_ms },
        BossSignalLatched { signal_id }
    ]
});
crate::authoring::clone_shape::enum_shape!(BattleObjective {
    units: [],
    tuples: [],
    records: [
        DefeatBoss { unit_ref },
        ProtectUnit { unit_ref },
        Survive { time_ms }
    ]
});
crate::authoring::clone_shape::enum_shape!(PlayerMovementPlan {
    units: [FixedDefense],
    tuples: [],
    records: []
});
crate::authoring::clone_shape::enum_shape!(FieldWorkCompletionEffect {
    units: [],
    tuples: [],
    records: [RepairProtectedUnit { unit_ref, flat_hp }]
});

impl BattleScenario {
    #[cfg(test)]
    pub(crate) fn fixture_required_group_outcomes() -> Vec<TerminalOutcome> {
        vec![
            TerminalOutcome {
                id: BattleOutcomeId::new("victory"),
                verdict: BattleVerdict::Victory,
                condition: TerminalCondition::AllRequiredEnemyGroupsDefeated,
                priority: 0,
            },
            TerminalOutcome {
                id: BattleOutcomeId::new("defeat"),
                verdict: BattleVerdict::Defeat,
                condition: TerminalCondition::AllPlayerCombatantsDefeated,
                priority: 0,
            },
        ]
    }

    #[cfg(test)]
    pub(crate) fn fixture_time_victory_outcomes(time_ms: u64) -> Vec<TerminalOutcome> {
        vec![TerminalOutcome {
            id: BattleOutcomeId::new("victory"),
            verdict: BattleVerdict::Victory,
            condition: TerminalCondition::TimeReached { time_ms },
            priority: 0,
        }]
    }

    #[cfg(test)]
    pub(crate) fn fixture_survival_outcomes(time_ms: u64) -> Vec<TerminalOutcome> {
        let mut outcomes = Self::fixture_time_victory_outcomes(time_ms);
        outcomes.push(TerminalOutcome {
            id: BattleOutcomeId::new("defeat"),
            verdict: BattleVerdict::Defeat,
            condition: TerminalCondition::AllPlayerCombatantsDefeated,
            priority: 0,
        });
        outcomes
    }

    #[cfg(test)]
    pub(crate) fn fixture_protected_outcomes(
        unit_ref: ScenarioUnitRef,
        survive_until_ms: Option<u64>,
    ) -> Vec<TerminalOutcome> {
        let mut outcomes = survive_until_ms.map_or_else(
            Self::fixture_required_group_outcomes,
            Self::fixture_survival_outcomes,
        );
        outcomes[1].condition = TerminalCondition::UnitDefeated { unit_ref };
        outcomes
    }

    pub fn empty(field_size: (u8, u8)) -> Self {
        Self {
            battlefield: BattleFieldSpec {
            walkable_paths: Vec::new(),
                object_placement: None,
                width: field_size.0,
                height: field_size.1,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: vec![
                TerminalOutcome {
                    id: BattleOutcomeId::new("victory_required_groups_defeated"),
                    verdict: BattleVerdict::Victory,
                    condition: TerminalCondition::AllRequiredEnemyGroupsDefeated,
                    priority: 0,
                },
                TerminalOutcome {
                    id: BattleOutcomeId::new("defeat_all_players_defeated"),
                    verdict: BattleVerdict::Defeat,
                    condition: TerminalCondition::AllPlayerCombatantsDefeated,
                    priority: 0,
                },
            ],
            tactical_plan: TacticalPlan::default(),
            boss_runtime: None,
        }
    }

    pub fn group(&self, group_id: &ScenarioGroupId) -> Option<&ScenarioSpawnGroup> {
        self.groups.iter().find(|group| &group.id == group_id)
    }

    pub fn validate(&self) -> Result<(), GameError> {
        self.validate_battlefield()?;
        self.validate_groups_and_events()?;
        self.validate_tactical_plan()?;
        self.validate_field_work_objects()?;
        self.validate_terminal_outcomes()?;
        self.validate_boss_runtime_setup()?;
        Ok(())
    }

    fn validate_boss_runtime_setup(&self) -> Result<(), GameError> {
        let Some(setup) = &self.boss_runtime else {
            return Ok(());
        };
        if setup.actors.len() != setup.definition.actor_slots.len() {
            return Err(invalid_scenario(
                "Boss Runtime setup must bind every actor slot exactly once",
            ));
        }
        let opponent_refs = self
            .groups
            .iter()
            .filter(|group| group.side == Side::Opponent)
            .flat_map(|group| group.spawns.iter())
            .filter(|spawn| spawn.side == Side::Opponent)
            .map(|spawn| &spawn.unit_ref)
            .collect::<HashSet<_>>();
        let mut seen_slots = HashSet::new();
        let mut seen_units = HashSet::new();
        for actor in &setup.actors {
            if actor.actor_slot_index >= setup.definition.actor_slots.len()
                || !seen_slots.insert(actor.actor_slot_index)
            {
                return Err(invalid_scenario(
                    "Boss Runtime setup has duplicate or invalid actor slot index",
                ));
            }
            if !opponent_refs.contains(&actor.unit_ref) || !seen_units.insert(&actor.unit_ref) {
                return Err(invalid_scenario(
                    "Boss Runtime setup must bind unique opponent scenario unit refs",
                ));
            }
        }
        Ok(())
    }

    fn validate_battlefield(&self) -> Result<(), GameError> {
        if self.battlefield.width == 0 || self.battlefield.height == 0 {
            return Err(invalid_scenario(
                "battlefield width and height must be greater than zero",
            ));
        }

        let mut obstacle_positions = HashSet::new();
        let mut valid_positions = HashSet::new();
        for valid_tile in &self.battlefield.valid_tiles {
            self.validate_position_in_rect(*valid_tile, "battlefield valid tile")?;
            if !valid_positions.insert(*valid_tile) {
                return Err(invalid_scenario(format!(
                    "duplicate battlefield valid tile at ({}, {})",
                    valid_tile.x, valid_tile.y
                )));
            }
        }

        for obstacle in &self.battlefield.obstacles {
            self.validate_position(*obstacle, "battlefield obstacle")?;
            if !obstacle_positions.insert(*obstacle) {
                return Err(invalid_scenario(format!(
                    "duplicate battlefield obstacle at ({}, {})",
                    obstacle.x, obstacle.y
                )));
            }
        }

        let mut paths = HashSet::new();
        for position in &self.battlefield.walkable_paths {
            self.validate_position(*position, "walkable path tile")?;
            if obstacle_positions.contains(position) || !paths.insert(*position) {
                return Err(invalid_scenario("walkable path tiles must be unique valid terrain"));
            }
        }
        if let Some(domain) = &self.battlefield.object_placement {
            let mut ground = HashSet::new();
            let mut reserved = HashSet::new();
            for position in &domain.ground {
                self.validate_position(*position, "independent object ground")?;
                if obstacle_positions.contains(position) || !ground.insert(*position) {
                    return Err(invalid_scenario(
                        "object ground must be unique walkable terrain",
                    ));
                }
            }
            for position in &domain.reserved {
                self.validate_position(*position, "independent object reserved tile")?;
                if !reserved.insert(*position) {
                    return Err(invalid_scenario("duplicate object reserved tile"));
                }
            }
        }

        Ok(())
    }

    fn validate_groups_and_events(&self) -> Result<(), GameError> {
        let mut group_ids = HashSet::new();
        let mut unit_refs = HashSet::new();
        let obstacle_positions = self
            .battlefield
            .obstacles
            .iter()
            .copied()
            .collect::<HashSet<_>>();

        for group in &self.groups {
            if !group_ids.insert(group.id.clone()) {
                return Err(invalid_scenario(format!(
                    "duplicate scenario group id '{}'",
                    group.id.0
                )));
            }

            let mut group_spawn_positions = HashSet::new();
            for spawn in &group.spawns {
                if spawn.side != group.side {
                    return Err(invalid_scenario(format!(
                        "spawn '{}' side {:?} does not match group '{}' side {:?}",
                        spawn.unit_ref.0, spawn.side, group.id.0, group.side
                    )));
                }
                if !unit_refs.insert(spawn.unit_ref.clone()) {
                    return Err(invalid_scenario(format!(
                        "duplicate scenario unit ref '{}'",
                        spawn.unit_ref.0
                    )));
                }
                self.validate_position(spawn.position, "scenario spawn")?;
                if obstacle_positions.contains(&spawn.position) {
                    return Err(invalid_scenario(format!(
                        "spawn '{}' overlaps obstacle at ({}, {})",
                        spawn.unit_ref.0, spawn.position.x, spawn.position.y
                    )));
                }
                if !group_spawn_positions.insert(spawn.position) {
                    return Err(invalid_scenario(format!(
                        "group '{}' has multiple spawns at ({}, {})",
                        group.id.0, spawn.position.x, spawn.position.y
                    )));
                }
            }
        }

        let mut event_ids = HashSet::new();
        let mut spawned_group_ids = HashSet::new();
        for event in &self.events {
            if !event_ids.insert(event.id.clone()) {
                return Err(invalid_scenario(format!(
                    "duplicate scenario event id '{}'",
                    event.id.0
                )));
            }
            match &event.action {
                ScenarioAction::SpawnGroup { group_id } => {
                    if !group_ids.contains(group_id) {
                        return Err(invalid_scenario(format!(
                            "event '{}' references unknown spawn group '{}'",
                            event.id.0, group_id.0
                        )));
                    }
                    spawned_group_ids.insert(group_id.clone());
                }
            }
        }

        for group in &self.groups {
            if group.side == Side::Opponent
                && group.required_for_victory
                && !spawned_group_ids.contains(&group.id)
            {
                return Err(invalid_scenario(format!(
                    "required enemy group '{}' is never spawned by a scenario event",
                    group.id.0
                )));
            }
        }

        Ok(())
    }

    fn validate_tactical_plan(&self) -> Result<(), GameError> {
        self.tactical_point_ids()?;
        let unit_refs = self.unit_refs();

        match &self.tactical_plan.objective {
            BattleObjective::Survive { .. } => {}
            BattleObjective::DefeatBoss { unit_ref }
            | BattleObjective::ProtectUnit { unit_ref } => {
                ensure_unit_ref_exists(&unit_refs, unit_ref, "battle objective")?;
            }
        }

        match &self.tactical_plan.enemy_plan {
            EnemyMovementPlan::PathAlongCells { cells } => {
                for cell in cells {
                    self.validate_position(*cell, "enemy movement path cell")?;
                }
            }
        }

        Ok(())
    }

    fn validate_terminal_outcomes(&self) -> Result<(), GameError> {
        if self.terminal_outcomes.is_empty() {
            return Err(invalid_scenario(
                "must declare at least one terminal outcome",
            ));
        }
        let unit_refs = self.unit_refs();
        let mut outcome_ids = HashSet::new();
        let mut has_victory = false;

        for outcome in &self.terminal_outcomes {
            if outcome.id.as_str().trim().is_empty() || !outcome_ids.insert(outcome.id.clone()) {
                return Err(invalid_scenario(format!(
                    "invalid or duplicate terminal outcome id '{}'",
                    outcome.id.as_str()
                )));
            }
            has_victory |= outcome.verdict == BattleVerdict::Victory;
            match &outcome.condition {
                TerminalCondition::AllRequiredEnemyGroupsDefeated
                | TerminalCondition::AllPlayerCombatantsDefeated => {}
                TerminalCondition::UnitDefeated { unit_ref } => {
                    ensure_unit_ref_exists(&unit_refs, unit_ref, "terminal condition")?;
                }
                TerminalCondition::TimeReached { time_ms } => {
                    if *time_ms == 0 {
                        return Err(invalid_scenario(format!(
                            "terminal outcome '{}' time must be greater than zero",
                            outcome.id.as_str()
                        )));
                    }
                }
                TerminalCondition::BossSignalLatched { signal_id } => {
                    if self.boss_runtime.is_none() {
                        return Err(invalid_scenario(format!(
                            "terminal outcome '{}' references Boss signal '{}' without Boss Runtime",
                            outcome.id.as_str(),
                            signal_id.as_str()
                        )));
                    }
                }
            }
        }

        if !has_victory {
            return Err(invalid_scenario(
                "must declare at least one reachable Victory outcome",
            ));
        }

        Ok(())
    }

    fn validate_field_work_objects(&self) -> Result<(), GameError> {
        let unit_refs = self.unit_refs();
        let mut object_ids = HashSet::new();
        let mut work_cell_positions = HashSet::new();
        for object in &self.tactical_plan.field_work_objects {
            if object.id.0.trim().is_empty() || !object_ids.insert(object.id.clone()) {
                return Err(invalid_scenario(format!(
                    "invalid or duplicate field work object id '{}'",
                    object.id.0
                )));
            }
            self.validate_position(object.position, "field work object")?;
            if object.required_work_ms == 0 || object.work_cells.is_empty() {
                return Err(invalid_scenario(format!(
                    "field work object '{}' requires positive work and at least one cell",
                    object.id.0
                )));
            }
            let mut cell_ids = HashSet::new();
            for cell in &object.work_cells {
                if cell.id.0.trim().is_empty() || !cell_ids.insert(cell.id.clone()) {
                    return Err(invalid_scenario(format!(
                        "field work object '{}' has invalid or duplicate cell id '{}'",
                        object.id.0, cell.id.0
                    )));
                }
                self.validate_position(cell.position, "field work cell")?;
                if !work_cell_positions.insert(cell.position) {
                    return Err(invalid_scenario(format!(
                        "duplicate field work cell position ({}, {})",
                        cell.position.x, cell.position.y
                    )));
                }
            }
            match &object.completion_effect {
                FieldWorkCompletionEffect::RepairProtectedUnit { unit_ref, flat_hp } => {
                    ensure_unit_ref_exists(&unit_refs, unit_ref, "field work repair effect")?;
                    if *flat_hp == 0 {
                        return Err(invalid_scenario(format!(
                            "field work object '{}' repair must be positive",
                            object.id.0
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_position(&self, position: Position, label: &str) -> Result<(), GameError> {
        self.validate_position_in_rect(position, label)?;
        if !self.battlefield.valid_tiles.is_empty()
            && !self.battlefield.valid_tiles.contains(&position)
        {
            return Err(invalid_scenario(format!(
                "{label} position ({}, {}) is outside valid battlefield tiles",
                position.x, position.y
            )));
        }
        Ok(())
    }

    fn validate_position_in_rect(&self, position: Position, label: &str) -> Result<(), GameError> {
        if position.x < 0
            || position.y < 0
            || position.x >= i32::from(self.battlefield.width)
            || position.y >= i32::from(self.battlefield.height)
        {
            return Err(invalid_scenario(format!(
                "{label} position ({}, {}) is outside battlefield {}x{}",
                position.x, position.y, self.battlefield.width, self.battlefield.height
            )));
        }
        Ok(())
    }

    fn unit_refs(&self) -> HashSet<ScenarioUnitRef> {
        self.groups
            .iter()
            .flat_map(|group| group.spawns.iter().map(|spawn| spawn.unit_ref.clone()))
            .collect()
    }

    fn tactical_point_ids(&self) -> Result<HashSet<TacticalPointId>, GameError> {
        let mut ids = HashSet::new();
        for point in &self.tactical_plan.points {
            self.validate_position(point.position, "tactical point")?;
            if !ids.insert(point.id.clone()) {
                return Err(invalid_scenario(format!(
                    "duplicate tactical point id '{}'",
                    point.id.0
                )));
            }
        }
        Ok(ids)
    }
}

fn ensure_unit_ref_exists(
    unit_refs: &HashSet<ScenarioUnitRef>,
    unit_ref: &ScenarioUnitRef,
    label: &str,
) -> Result<(), GameError> {
    if !unit_refs.contains(unit_ref) {
        return Err(invalid_scenario(format!(
            "{label} references unknown scenario unit '{}'",
            unit_ref.0
        )));
    }
    Ok(())
}

fn invalid_scenario(message: impl Into<String>) -> GameError {
    GameError::InvalidStaticData(format!("invalid battle scenario: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scenario_has_no_implicit_spawn_events() {
        let scenario = BattleScenario::empty((5, 6));

        assert_eq!(scenario.battlefield.width, 5);
        assert_eq!(scenario.battlefield.height, 6);
        assert!(scenario.groups.is_empty());
        assert!(scenario.events.is_empty());
        assert!(matches!(
            scenario.terminal_outcomes[0].condition,
            TerminalCondition::AllRequiredEnemyGroupsDefeated
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ));
    }

    #[test]
    fn scenario_validation_accepts_empty_manual_fixture() {
        let scenario = BattleScenario::empty((5, 6));

        assert!(scenario.validate().is_ok());
    }

    #[test]
    fn scenario_validation_rejects_zero_sized_battlefield() {
        let scenario = BattleScenario::empty((0, 6));

        assert_invalid_scenario_contains(scenario.validate(), "width and height");
    }

    #[test]
    fn scenario_validation_rejects_terminal_unknown_unit_reference() {
        let mut scenario = BattleScenario::empty((5, 6));
        scenario.terminal_outcomes[1].condition = TerminalCondition::UnitDefeated {
            unit_ref: ScenarioUnitRef::new("missing_protected_unit"),
        };

        assert_invalid_scenario_contains(
            scenario.validate(),
            "references unknown scenario unit 'missing_protected_unit'",
        );
    }

    #[test]
    fn scenario_validation_rejects_duplicate_tactical_points() {
        let mut scenario = BattleScenario::empty((5, 6));
        scenario.tactical_plan.points = vec![
            TacticalPoint {
                id: TacticalPointId::new("gate"),
                position: Position::new(1, 1),
            },
            TacticalPoint {
                id: TacticalPointId::new("gate"),
                position: Position::new(2, 2),
            },
        ];

        assert_invalid_scenario_contains(scenario.validate(), "duplicate tactical point id 'gate'");
    }

    #[test]
    fn scenario_event_validation_keeps_ids_and_group_references_authoritative() {
        let group_id = ScenarioGroupId::new("reinforcements");
        let event = ScenarioEvent {
            id: ScenarioEventId::new("spawn_reinforcements"),
            trigger: ScenarioTrigger::AtBattleStart,
            action: ScenarioAction::SpawnGroup {
                group_id: group_id.clone(),
            },
        };
        let mut scenario = BattleScenario::empty((5, 6));
        scenario.groups.push(ScenarioSpawnGroup {
            id: group_id,
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: Vec::new(),
        });
        scenario.events.push(event.clone());

        assert!(scenario.validate().is_ok());

        let mut duplicate_event = scenario.clone();
        duplicate_event.events.push(event);
        assert_invalid_scenario_contains(
            duplicate_event.validate(),
            "duplicate scenario event id 'spawn_reinforcements'",
        );

        let mut unknown_group = scenario;
        unknown_group.events[0].action = ScenarioAction::SpawnGroup {
            group_id: ScenarioGroupId::new("missing"),
        };
        assert_invalid_scenario_contains(
            unknown_group.validate(),
            "event 'spawn_reinforcements' references unknown spawn group 'missing'",
        );
    }

    fn assert_invalid_scenario_contains(result: Result<(), GameError>, expected: &str) {
        let Err(GameError::InvalidStaticData(message)) = result else {
            panic!("expected invalid static data, got {result:?}");
        };
        assert!(
            message.contains(expected),
            "expected '{message}' to contain '{expected}'"
        );
    }
}
