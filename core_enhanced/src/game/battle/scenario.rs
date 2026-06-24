use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    battle::types::{BattleUnitDraft, BattleWinner},
    behavior::GameError,
    enums::Side,
    resources::Position,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScenarioGroupId(pub String);

impl ScenarioGroupId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
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

#[derive(Debug, Clone)]
pub struct BattleFieldSpec {
    pub width: u8,
    pub height: u8,
    /// Empty means every tile inside width x height is valid.
    /// Non-empty enables non-rectangular battlefields authored from ASCII templates.
    pub valid_tiles: Vec<Position>,
    pub obstacles: Vec<Position>,
}

#[derive(Debug, Clone)]
pub struct ScenarioArtifact {
    pub side: Side,
    pub base_uuid: Uuid,
    pub instance_salt: u32,
}

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
}

#[derive(Debug, Clone)]
pub enum ScenarioAction {
    SpawnGroup { group_id: ScenarioGroupId },
    EndBattle { winner: BattleWinner },
}

#[derive(Debug, Clone)]
pub struct ScenarioEvent {
    pub id: ScenarioEventId,
    pub trigger: ScenarioTrigger,
    pub action: ScenarioAction,
    pub once: bool,
}

#[derive(Debug, Clone)]
pub enum WinCondition {
    AllRequiredEnemyGroupsDefeated,
    DefeatUnit {
        unit_ref: ScenarioUnitRef,
    },
    ProtectUnit {
        unit_ref: ScenarioUnitRef,
    },
    ProtectUnitUntil {
        unit_ref: ScenarioUnitRef,
        time_ms: u64,
    },
    SurviveUntil {
        time_ms: u64,
    },
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

#[derive(Debug, Clone)]
pub struct TacticalPlan {
    pub objective: BattleObjective,
    pub player_plan: PlayerMovementPlan,
    pub enemy_plan: EnemyMovementPlan,
    pub points: Vec<TacticalPoint>,
}

impl TacticalPlan {
    pub fn fixed_defense() -> Self {
        Self {
            objective: BattleObjective::Survive { time_ms: 0 },
            player_plan: PlayerMovementPlan::FixedDefense,
            enemy_plan: EnemyMovementPlan::PathAlongCells { cells: Vec::new() },
            points: Vec::new(),
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
    pub artifacts: Vec<ScenarioArtifact>,
    pub groups: Vec<ScenarioSpawnGroup>,
    pub events: Vec<ScenarioEvent>,
    pub win_condition: WinCondition,
    pub tactical_plan: TacticalPlan,
}

impl BattleScenario {
    pub fn empty(field_size: (u8, u8)) -> Self {
        Self {
            battlefield: BattleFieldSpec {
                width: field_size.0,
                height: field_size.1,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition: WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: TacticalPlan::default(),
        }
    }

    pub fn group(&self, group_id: &ScenarioGroupId) -> Option<&ScenarioSpawnGroup> {
        self.groups.iter().find(|group| &group.id == group_id)
    }

    pub fn validate(&self) -> Result<(), GameError> {
        self.validate_battlefield()?;
        self.validate_groups_and_events()?;
        self.validate_tactical_plan()?;
        self.validate_win_condition()?;
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
                ScenarioAction::EndBattle { .. } => {}
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

    fn validate_win_condition(&self) -> Result<(), GameError> {
        let unit_refs = self.unit_refs();

        match &self.win_condition {
            WinCondition::AllRequiredEnemyGroupsDefeated | WinCondition::SurviveUntil { .. } => {}
            WinCondition::DefeatUnit { unit_ref }
            | WinCondition::ProtectUnit { unit_ref }
            | WinCondition::ProtectUnitUntil { unit_ref, .. } => {
                ensure_unit_ref_exists(&unit_refs, unit_ref, "win condition")?;
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
        assert!(scenario.artifacts.is_empty());
        assert!(scenario.groups.is_empty());
        assert!(scenario.events.is_empty());
        assert!(matches!(
            scenario.win_condition,
            WinCondition::AllRequiredEnemyGroupsDefeated
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
