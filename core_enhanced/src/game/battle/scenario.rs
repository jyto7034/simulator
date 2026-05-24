use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    battle::types::{BattleUnitDraft, BattleWinner},
    behavior::GameError,
    combat_preview::BattlefieldArchetype,
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
    SurviveUntil {
        time_ms: u64,
    },
    RecoverHoldAndExtract {
        target_point_id: TacticalPointId,
        extraction_point_id: TacticalPointId,
        target_radius: f32,
        extraction_radius: f32,
        hold_duration_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TacticalPointId(pub String);

impl TacticalPointId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TacticalGroupPlanId(pub String);

impl TacticalGroupPlanId {
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
pub enum TacticalGroupMembers {
    SpawnGroup(ScenarioGroupId),
    ExplicitUnits(Vec<ScenarioUnitRef>),
    SideAll(Side),
}

#[derive(Debug, Clone)]
pub enum GroupObjective {
    FollowBattleObjective,
    HoldArea { point_id: TacticalPointId },
    AdvanceToPoint { point_id: TacticalPointId },
    AdvanceAlongPath { point_ids: Vec<TacticalPointId> },
    ReconnectToGroup { group_id: TacticalGroupPlanId },
}

#[derive(Debug, Clone)]
pub enum FormationKind {
    Loose,
    Column,
    Line,
    Wedge,
}

#[derive(Debug, Clone)]
pub struct TacticalGroupPlan {
    pub id: TacticalGroupPlanId,
    pub side: Side,
    pub members: TacticalGroupMembers,
    pub objective: GroupObjective,
    pub formation: FormationKind,
    pub cohesion_radius: f32,
    pub engage_radius: f32,
}

#[derive(Debug, Clone)]
pub enum BattleObjective {
    SuppressAll,
    HoldArea {
        point_id: TacticalPointId,
    },
    AdvanceToPoint {
        point_id: TacticalPointId,
    },
    DefeatBoss {
        unit_ref: ScenarioUnitRef,
    },
    ProtectUnit {
        unit_ref: ScenarioUnitRef,
    },
    Survive {
        time_ms: u64,
    },
    RecoverHoldAndExtract {
        target_point_id: TacticalPointId,
        extraction_point_id: TacticalPointId,
        hold_duration_ms: u64,
    },
}

#[derive(Debug, Clone)]
pub enum PlayerMovementPlan {
    FreeEngage,
    FixedDefense,
    HoldDeployment {
        guard_radius: f32,
        leash_radius: f32,
        chase_radius: f32,
        return_to_anchor: bool,
    },
    CautiousEngage {
        leash_radius: f32,
        chase_radius: f32,
    },
}

#[derive(Debug, Clone)]
pub enum EnemyMovementPlan {
    AssaultPlayer,
    PathToPoint { point_id: TacticalPointId },
    PathAlongPath { point_ids: Vec<TacticalPointId> },
}

#[derive(Debug, Clone)]
pub struct TacticalPlan {
    pub objective: BattleObjective,
    pub player_plan: PlayerMovementPlan,
    pub enemy_plan: EnemyMovementPlan,
    pub points: Vec<TacticalPoint>,
    pub group_plans: Vec<TacticalGroupPlan>,
}

impl TacticalPlan {
    pub fn free_engage() -> Self {
        Self {
            objective: BattleObjective::SuppressAll,
            player_plan: PlayerMovementPlan::FreeEngage,
            enemy_plan: EnemyMovementPlan::AssaultPlayer,
            points: Vec::new(),
            group_plans: vec![Self::default_player_main_group()],
        }
    }

    pub fn hold_deployment_default() -> Self {
        Self {
            objective: BattleObjective::SuppressAll,
            player_plan: PlayerMovementPlan::HoldDeployment {
                guard_radius: 1.5,
                leash_radius: 2.5,
                chase_radius: 0.75,
                return_to_anchor: true,
            },
            enemy_plan: EnemyMovementPlan::AssaultPlayer,
            points: Vec::new(),
            group_plans: vec![Self::default_player_main_group()],
        }
    }

    pub fn point(&self, point_id: &TacticalPointId) -> Option<&TacticalPoint> {
        self.points.iter().find(|point| &point.id == point_id)
    }

    pub fn group_plan(&self, group_id: &TacticalGroupPlanId) -> Option<&TacticalGroupPlan> {
        self.group_plans.iter().find(|group| &group.id == group_id)
    }

    pub fn for_archetype(archetype: BattlefieldArchetype) -> Self {
        match archetype {
            BattlefieldArchetype::OpenHall => Self::free_engage(),
            BattlefieldArchetype::Corridor => Self {
                objective: BattleObjective::SuppressAll,
                player_plan: PlayerMovementPlan::CautiousEngage {
                    leash_radius: 3.0,
                    chase_radius: 1.0,
                },
                enemy_plan: EnemyMovementPlan::AssaultPlayer,
                points: Vec::new(),
                group_plans: vec![Self::default_player_main_group()],
            },
            BattlefieldArchetype::ChokePoint
            | BattlefieldArchetype::Ambush
            | BattlefieldArchetype::Surrounded
            | BattlefieldArchetype::SplitRoom => Self::hold_deployment_default(),
            BattlefieldArchetype::ObstacleRoom => Self {
                objective: BattleObjective::SuppressAll,
                player_plan: PlayerMovementPlan::CautiousEngage {
                    leash_radius: 2.5,
                    chase_radius: 0.75,
                },
                enemy_plan: EnemyMovementPlan::AssaultPlayer,
                points: Vec::new(),
                group_plans: vec![Self::default_player_main_group()],
            },
            BattlefieldArchetype::BossArena => Self {
                objective: BattleObjective::SuppressAll,
                player_plan: PlayerMovementPlan::FreeEngage,
                enemy_plan: EnemyMovementPlan::AssaultPlayer,
                points: Vec::new(),
                group_plans: vec![Self::default_player_main_group()],
            },
        }
    }

    pub fn default_player_main_group_id() -> TacticalGroupPlanId {
        TacticalGroupPlanId::new("player_main")
    }

    fn default_player_main_group() -> TacticalGroupPlan {
        TacticalGroupPlan {
            id: Self::default_player_main_group_id(),
            side: Side::Player,
            members: TacticalGroupMembers::SideAll(Side::Player),
            objective: GroupObjective::FollowBattleObjective,
            formation: FormationKind::Loose,
            cohesion_radius: 4.0,
            engage_radius: 3.0,
        }
    }
}

impl Default for TacticalPlan {
    fn default() -> Self {
        Self::free_engage()
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
        let known_point_ids = self.tactical_point_ids()?;
        let group_ids = self.group_ids();
        let unit_refs = self.unit_refs();
        let tactical_group_ids = self.tactical_group_ids()?;

        match &self.tactical_plan.objective {
            BattleObjective::SuppressAll | BattleObjective::Survive { .. } => {}
            BattleObjective::HoldArea { point_id }
            | BattleObjective::AdvanceToPoint { point_id } => {
                ensure_point_exists(&known_point_ids, point_id, "battle objective")?;
            }
            BattleObjective::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                ..
            } => {
                ensure_point_exists(&known_point_ids, target_point_id, "battle objective")?;
                ensure_point_exists(&known_point_ids, extraction_point_id, "battle objective")?;
            }
            BattleObjective::DefeatBoss { unit_ref }
            | BattleObjective::ProtectUnit { unit_ref } => {
                ensure_unit_ref_exists(&unit_refs, unit_ref, "battle objective")?;
            }
        }

        match &self.tactical_plan.enemy_plan {
            EnemyMovementPlan::AssaultPlayer => {}
            EnemyMovementPlan::PathToPoint { point_id } => {
                ensure_point_exists(&known_point_ids, point_id, "enemy movement plan")?;
            }
            EnemyMovementPlan::PathAlongPath { point_ids } => {
                if point_ids.is_empty() {
                    return Err(invalid_scenario(
                        "enemy PathAlongPath must reference at least one tactical point",
                    ));
                }
                for point_id in point_ids {
                    ensure_point_exists(&known_point_ids, point_id, "enemy movement path")?;
                }
            }
        }

        for group_plan in &self.tactical_plan.group_plans {
            if group_plan.cohesion_radius < 0.0
                || group_plan.engage_radius < 0.0
                || !group_plan.cohesion_radius.is_finite()
                || !group_plan.engage_radius.is_finite()
            {
                return Err(invalid_scenario(format!(
                    "tactical group '{}' radii must be finite non-negative values",
                    group_plan.id.0
                )));
            }

            match &group_plan.members {
                TacticalGroupMembers::SpawnGroup(group_id) => {
                    if !group_ids.contains(group_id) {
                        return Err(invalid_scenario(format!(
                            "tactical group '{}' references unknown spawn group '{}'",
                            group_plan.id.0, group_id.0
                        )));
                    }
                }
                TacticalGroupMembers::ExplicitUnits(unit_ids) => {
                    for unit_ref in unit_ids {
                        ensure_unit_ref_exists(&unit_refs, unit_ref, "tactical group members")?;
                    }
                }
                TacticalGroupMembers::SideAll(_) => {}
            }

            match &group_plan.objective {
                GroupObjective::FollowBattleObjective => {}
                GroupObjective::HoldArea { point_id }
                | GroupObjective::AdvanceToPoint { point_id } => {
                    ensure_point_exists(&known_point_ids, point_id, "tactical group objective")?;
                }
                GroupObjective::AdvanceAlongPath { point_ids } => {
                    if point_ids.is_empty() {
                        return Err(invalid_scenario(format!(
                            "tactical group '{}' AdvanceAlongPath must reference at least one tactical point",
                            group_plan.id.0
                        )));
                    }
                    for point_id in point_ids {
                        ensure_point_exists(&known_point_ids, point_id, "tactical group path")?;
                    }
                }
                GroupObjective::ReconnectToGroup { group_id } => {
                    if group_id == &group_plan.id {
                        return Err(invalid_scenario(format!(
                            "tactical group '{}' cannot reconnect to itself",
                            group_plan.id.0
                        )));
                    }
                    if !tactical_group_ids.contains(group_id) {
                        return Err(invalid_scenario(format!(
                            "tactical group '{}' reconnects to unknown group '{}'",
                            group_plan.id.0, group_id.0
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_win_condition(&self) -> Result<(), GameError> {
        let unit_refs = self.unit_refs();
        let point_ids = self.tactical_point_ids()?;

        match &self.win_condition {
            WinCondition::AllRequiredEnemyGroupsDefeated | WinCondition::SurviveUntil { .. } => {}
            WinCondition::DefeatUnit { unit_ref } | WinCondition::ProtectUnit { unit_ref } => {
                ensure_unit_ref_exists(&unit_refs, unit_ref, "win condition")?;
            }
            WinCondition::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                target_radius,
                extraction_radius,
                ..
            } => {
                ensure_point_exists(&point_ids, target_point_id, "win condition")?;
                ensure_point_exists(&point_ids, extraction_point_id, "win condition")?;
                if *target_radius < 0.0 || !target_radius.is_finite() {
                    return Err(invalid_scenario(
                        "RecoverHoldAndExtract target radius must be a finite non-negative value",
                    ));
                }
                if *extraction_radius < 0.0 || !extraction_radius.is_finite() {
                    return Err(invalid_scenario(
                        "RecoverHoldAndExtract extraction radius must be a finite non-negative value",
                    ));
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

    fn group_ids(&self) -> HashSet<ScenarioGroupId> {
        self.groups.iter().map(|group| group.id.clone()).collect()
    }

    fn unit_refs(&self) -> HashSet<ScenarioUnitRef> {
        self.groups
            .iter()
            .flat_map(|group| group.spawns.iter().map(|spawn| spawn.unit_ref.clone()))
            .collect()
    }

    fn tactical_group_ids(&self) -> Result<HashSet<TacticalGroupPlanId>, GameError> {
        let mut ids = HashSet::new();
        for group in &self.tactical_plan.group_plans {
            if !ids.insert(group.id.clone()) {
                return Err(invalid_scenario(format!(
                    "duplicate tactical group id '{}'",
                    group.id.0
                )));
            }
        }
        Ok(ids)
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

fn ensure_point_exists(
    point_ids: &HashSet<TacticalPointId>,
    point_id: &TacticalPointId,
    label: &str,
) -> Result<(), GameError> {
    if !point_ids.contains(point_id) {
        return Err(invalid_scenario(format!(
            "{label} references unknown tactical point '{}'",
            point_id.0
        )));
    }
    Ok(())
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
            PlayerMovementPlan::FreeEngage
        ));
    }

    #[test]
    fn tactical_plan_defaults_preserve_free_engage_for_open_hall() {
        let plan = TacticalPlan::for_archetype(BattlefieldArchetype::OpenHall);

        assert!(matches!(plan.objective, BattleObjective::SuppressAll));
        assert!(matches!(plan.player_plan, PlayerMovementPlan::FreeEngage));
        assert!(matches!(plan.enemy_plan, EnemyMovementPlan::AssaultPlayer));
        assert!(plan
            .group_plan(&TacticalPlan::default_player_main_group_id())
            .is_some());
        assert!(plan.group_plans.iter().any(|group| {
            group.side == Side::Player
                && matches!(group.members, TacticalGroupMembers::SideAll(Side::Player))
                && matches!(group.objective, GroupObjective::FollowBattleObjective)
        }));
    }

    #[test]
    fn tactical_plan_maps_defensive_archetypes_to_hold_deployment() {
        for archetype in [
            BattlefieldArchetype::ChokePoint,
            BattlefieldArchetype::Ambush,
            BattlefieldArchetype::Surrounded,
            BattlefieldArchetype::SplitRoom,
        ] {
            let plan = TacticalPlan::for_archetype(archetype);
            assert!(matches!(
                plan.player_plan,
                PlayerMovementPlan::HoldDeployment { .. }
            ));
        }
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

    #[test]
    fn scenario_validation_rejects_unknown_enemy_path_point() {
        let mut scenario = BattleScenario::empty((5, 6));
        scenario.tactical_plan.enemy_plan = EnemyMovementPlan::PathToPoint {
            point_id: TacticalPointId::new("exit"),
        };

        assert_invalid_scenario_contains(scenario.validate(), "unknown tactical point 'exit'");
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
