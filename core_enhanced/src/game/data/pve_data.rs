use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        battle::scenario::{
            BattleObjective, EnemyMovementPlan, TacticalPlan, TacticalPoint, TacticalPointId,
            WinCondition,
        },
        combat_preview::{BattlefieldArchetype, BattlefieldSizeClass, CombatNodeType, EnemyKind},
        data::{build_string_index, once_lock_with},
        enums::{RewardMode, RiskLevel, Tier},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PveWaveEnemyData {
    #[serde(default = "default_enemy_kind")]
    pub kind: EnemyKind,
    #[serde(default)]
    pub profile_id: Option<String>,
    pub abnormality_id: String,
    #[serde(default = "default_tier")]
    pub tier: Tier,
    #[serde(default = "default_count")]
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PveWaveSource {
    Manual(Vec<PveWaveEnemyData>),
    GeneratedCorroded {
        preset_id: String,
        #[serde(default)]
        budget_override: Option<u32>,
        #[serde(default)]
        seed_salt: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PveWaveData {
    pub id: String,
    #[serde(default)]
    pub time_ms: u32,
    #[serde(default)]
    pub spawn_zone_ids: Vec<String>,
    #[serde(default = "default_required_for_victory")]
    pub required_for_victory: bool,
    #[serde(default)]
    pub source: Option<PveWaveSource>,
    #[serde(default)]
    pub enemies: Vec<PveWaveEnemyData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveStaticObstacleData {
    pub position: PvePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PveBattlefieldOverrideData {
    #[serde(default)]
    pub archetype: Option<BattlefieldArchetype>,
    #[serde(default)]
    pub size_class: Option<BattlefieldSizeClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveTacticalPointData {
    pub id: String,
    pub position: PvePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PveBattleObjectiveData {
    SuppressAll,
    HoldArea {
        point_id: String,
    },
    AdvanceToPoint {
        point_id: String,
    },
    DefeatBoss {
        unit_ref: String,
    },
    ProtectUnit {
        unit_ref: String,
    },
    ProtectUnitForDuration {
        unit_ref: String,
        time_ms: u64,
        cleanup_required: bool,
    },
    Survive {
        time_ms: u64,
    },
    RecoverHoldAndExtract {
        target_point_id: String,
        extraction_point_id: String,
        hold_duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PveEnemyMovementPlanData {
    AssaultPlayer,
    PathToPoint { point_id: String },
    PathAlongPath { point_ids: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PveWinConditionData {
    AllRequiredEnemyGroupsDefeated,
    DefeatUnit {
        unit_ref: String,
    },
    ProtectUnit {
        unit_ref: String,
    },
    ProtectUnitForDuration {
        unit_ref: String,
        time_ms: u64,
        cleanup_required: bool,
    },
    SurviveUntil {
        time_ms: u64,
    },
    RecoverHoldAndExtract {
        target_point_id: String,
        extraction_point_id: String,
        target_radius: f32,
        extraction_radius: f32,
        hold_duration_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PveTacticalPlanData {
    #[serde(default)]
    pub points: Vec<PveTacticalPointData>,
    #[serde(default)]
    pub objective: Option<PveBattleObjectiveData>,
    #[serde(default)]
    pub enemy_plan: Option<PveEnemyMovementPlanData>,
}

fn default_tier() -> Tier {
    Tier::I
}

fn default_enemy_kind() -> EnemyKind {
    EnemyKind::Abnormality
}

fn default_reward_mode() -> RewardMode {
    RewardMode::ClaimAll
}

fn default_count() -> u32 {
    1
}

fn default_required_for_victory() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PvePosition {
    pub x: i32,
    pub y: i32,
}

impl From<PvePosition> for Position {
    fn from(pos: PvePosition) -> Self {
        Position::new(pos.x, pos.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveEncounter {
    pub id: String,
    pub abnormality_id: String,
    pub difficulty: u8,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub node_type: Option<CombatNodeType>,
    #[serde(default = "default_reward_mode")]
    pub reward_mode: RewardMode,
    #[serde(default)]
    pub reward_uuids: Vec<Uuid>,
    #[serde(default)]
    pub battlefield: Option<PveBattlefieldOverrideData>,
    #[serde(default)]
    pub tactical_plan: Option<PveTacticalPlanData>,
    #[serde(default)]
    pub win_condition: Option<PveWinConditionData>,
    #[serde(default)]
    pub waves: Vec<PveWaveData>,
    #[serde(default)]
    pub static_obstacles: Vec<PveStaticObstacleData>,
}

impl PveEncounter {
    pub fn wave_definitions(&self) -> Vec<PveWaveData> {
        self.waves.clone()
    }

    pub fn apply_authored_tactical_plan(&self, plan: &mut TacticalPlan) {
        let Some(authored) = &self.tactical_plan else {
            return;
        };

        plan.points = authored
            .points
            .iter()
            .map(|point| TacticalPoint {
                id: TacticalPointId::new(point.id.clone()),
                position: point.position.into(),
            })
            .collect();

        if let Some(objective) = &authored.objective {
            plan.objective = objective.to_battle_objective();
        }
        if let Some(enemy_plan) = &authored.enemy_plan {
            plan.enemy_plan = enemy_plan.to_enemy_movement_plan();
        }
    }

    pub fn authored_win_condition(&self) -> Option<WinCondition> {
        self.win_condition
            .as_ref()
            .map(PveWinConditionData::to_win_condition)
    }
}

impl PveWaveData {
    pub fn manual_enemies(&self) -> &[PveWaveEnemyData] {
        match &self.source {
            Some(PveWaveSource::Manual(enemies)) => enemies,
            Some(PveWaveSource::GeneratedCorroded { .. }) => &[],
            None => &self.enemies,
        }
    }
}

impl PveBattleObjectiveData {
    fn to_battle_objective(&self) -> BattleObjective {
        match self {
            Self::SuppressAll => BattleObjective::SuppressAll,
            Self::HoldArea { point_id } => BattleObjective::HoldArea {
                point_id: TacticalPointId::new(point_id.clone()),
            },
            Self::AdvanceToPoint { point_id } => BattleObjective::AdvanceToPoint {
                point_id: TacticalPointId::new(point_id.clone()),
            },
            Self::DefeatBoss { unit_ref } => BattleObjective::DefeatBoss {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
            },
            Self::ProtectUnit { unit_ref } => BattleObjective::ProtectUnit {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
            },
            Self::ProtectUnitForDuration {
                unit_ref,
                time_ms,
                cleanup_required,
            } => BattleObjective::ProtectUnitForDuration {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
                time_ms: *time_ms,
                cleanup_required: *cleanup_required,
            },
            Self::Survive { time_ms } => BattleObjective::Survive { time_ms: *time_ms },
            Self::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                hold_duration_ms,
            } => BattleObjective::RecoverHoldAndExtract {
                target_point_id: TacticalPointId::new(target_point_id.clone()),
                extraction_point_id: TacticalPointId::new(extraction_point_id.clone()),
                hold_duration_ms: *hold_duration_ms,
            },
        }
    }
}

impl PveEnemyMovementPlanData {
    fn to_enemy_movement_plan(&self) -> EnemyMovementPlan {
        match self {
            Self::AssaultPlayer => EnemyMovementPlan::AssaultPlayer,
            Self::PathToPoint { point_id } => EnemyMovementPlan::PathToPoint {
                point_id: TacticalPointId::new(point_id.clone()),
            },
            Self::PathAlongPath { point_ids } => EnemyMovementPlan::PathAlongPath {
                point_ids: point_ids
                    .iter()
                    .map(|point_id| TacticalPointId::new(point_id.clone()))
                    .collect(),
            },
        }
    }
}

impl PveWinConditionData {
    fn to_win_condition(&self) -> WinCondition {
        match self {
            Self::AllRequiredEnemyGroupsDefeated => WinCondition::AllRequiredEnemyGroupsDefeated,
            Self::DefeatUnit { unit_ref } => WinCondition::DefeatUnit {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
            },
            Self::ProtectUnit { unit_ref } => WinCondition::ProtectUnit {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
            },
            Self::ProtectUnitForDuration {
                unit_ref,
                time_ms,
                cleanup_required,
            } => WinCondition::ProtectUnitForDuration {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
                time_ms: *time_ms,
                cleanup_required: *cleanup_required,
            },
            Self::SurviveUntil { time_ms } => WinCondition::SurviveUntil { time_ms: *time_ms },
            Self::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                target_radius,
                extraction_radius,
                hold_duration_ms,
            } => WinCondition::RecoverHoldAndExtract {
                target_point_id: TacticalPointId::new(target_point_id.clone()),
                extraction_point_id: TacticalPointId::new(extraction_point_id.clone()),
                target_radius: *target_radius,
                extraction_radius: *extraction_radius,
                hold_duration_ms: *hold_duration_ms,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveEncounterDatabase {
    pub encounters: Vec<PveEncounter>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_abnormality_id: OnceLock<HashMap<String, usize>>,
}

impl PveEncounterDatabase {
    pub fn new(encounters: Vec<PveEncounter>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter id",
            |encounter| &encounter.id,
        ));
        let by_abnormality_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter abnormality_id",
            |encounter| &encounter.abnormality_id,
        ));

        Self {
            encounters,
            by_id,
            by_abnormality_id,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.encounters, "pve encounter id", |encounter| {
                &encounter.id
            })
        })
    }

    fn by_abnormality_id(&self) -> &HashMap<String, usize> {
        self.by_abnormality_id.get_or_init(|| {
            build_string_index(
                &self.encounters,
                "pve encounter abnormality_id",
                |encounter| &encounter.abnormality_id,
            )
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_abnormality_id();
        for encounter in &self.encounters {
            validate_encounter_authoring_contract(encounter);
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&PveEncounter> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_by_abnormality_id(&self, abnormality_id: &str) -> Option<&PveEncounter> {
        self.by_abnormality_id()
            .get(abnormality_id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_by_risk_level(&self, level: RiskLevel) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.risk_level == level)
            .collect()
    }

    pub fn get_by_difficulty_range(&self, min: u8, max: u8) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.difficulty >= min && e.difficulty <= max)
            .collect()
    }
}

fn validate_encounter_authoring_contract(encounter: &PveEncounter) {
    assert!(
        !encounter.waves.is_empty(),
        "pve encounter '{}' must define at least one wave",
        encounter.id
    );

    let mut wave_ids = HashSet::new();
    for wave in &encounter.waves {
        assert!(
            !wave.id.trim().is_empty(),
            "wave id in pve encounter '{}' must not be empty",
            encounter.id
        );
        assert!(
            wave_ids.insert(wave.id.as_str()),
            "duplicate wave id '{}' in pve encounter '{}'",
            wave.id,
            encounter.id
        );
        match &wave.source {
            Some(PveWaveSource::Manual(enemies)) => {
                assert!(
                    !enemies.is_empty(),
                    "manual wave '{}' in pve encounter '{}' must contain at least one enemy entry",
                    wave.id,
                    encounter.id
                );
                assert!(
                    wave.enemies.is_empty(),
                    "manual source wave '{}' in pve encounter '{}' must not also use legacy enemies",
                    wave.id,
                    encounter.id
                );
            }
            Some(PveWaveSource::GeneratedCorroded {
                preset_id,
                budget_override,
                ..
            }) => {
                assert!(
                    !preset_id.trim().is_empty(),
                    "generated corroded wave '{}' in pve encounter '{}' must reference a preset",
                    wave.id,
                    encounter.id
                );
                if let Some(budget_override) = budget_override {
                    assert!(
                        *budget_override > 0,
                        "generated corroded wave '{}' in pve encounter '{}' budget_override must be greater than zero",
                        wave.id,
                        encounter.id
                    );
                }
                assert!(
                    wave.enemies.is_empty(),
                    "generated corroded wave '{}' in pve encounter '{}' must not also use legacy enemies",
                    wave.id,
                    encounter.id
                );
            }
            None => {
                assert!(
                    !wave.enemies.is_empty(),
                    "wave '{}' in pve encounter '{}' must contain at least one enemy entry",
                    wave.id,
                    encounter.id
                );
            }
        }
        let mut spawn_zone_ids = HashSet::new();
        for spawn_zone_id in &wave.spawn_zone_ids {
            assert!(
                !spawn_zone_id.trim().is_empty(),
                "wave '{}' in pve encounter '{}' has empty spawn zone id",
                wave.id,
                encounter.id
            );
            assert!(
                spawn_zone_ids.insert(spawn_zone_id.as_str()),
                "wave '{}' in pve encounter '{}' references duplicate spawn zone id '{}'",
                wave.id,
                encounter.id,
                spawn_zone_id
            );
        }
        for enemy in wave.manual_enemies() {
            assert!(
                !enemy.abnormality_id.trim().is_empty(),
                "wave '{}' in pve encounter '{}' has empty enemy abnormality id",
                wave.id,
                encounter.id
            );
            assert!(
                enemy.count > 0,
                "wave '{}' in pve encounter '{}' enemy '{}' count must be greater than zero",
                wave.id,
                encounter.id,
                enemy.abnormality_id
            );
        }
    }

    let Some(tactical_plan) = &encounter.tactical_plan else {
        return;
    };

    let mut point_ids = HashSet::new();
    for point in &tactical_plan.points {
        assert!(
            point_ids.insert(point.id.as_str()),
            "duplicate tactical point '{}' in pve encounter '{}'",
            point.id,
            encounter.id
        );
    }

    if let Some(objective) = &tactical_plan.objective {
        validate_objective_points(encounter, &point_ids, objective);
    }
    if let Some(enemy_plan) = &tactical_plan.enemy_plan {
        validate_enemy_plan_points(encounter, &point_ids, enemy_plan);
    }
    if let Some(win_condition) = &encounter.win_condition {
        validate_win_condition_points(encounter, &point_ids, win_condition);
    }
    if let (Some(objective), Some(win_condition)) =
        (&tactical_plan.objective, &encounter.win_condition)
    {
        validate_objective_win_condition_consistency(encounter, objective, win_condition);
    }
}

fn validate_objective_points(
    encounter: &PveEncounter,
    point_ids: &HashSet<&str>,
    objective: &PveBattleObjectiveData,
) {
    match objective {
        PveBattleObjectiveData::HoldArea { point_id }
        | PveBattleObjectiveData::AdvanceToPoint { point_id } => {
            assert_known_point(encounter, point_ids, point_id, "battle objective");
        }
        PveBattleObjectiveData::RecoverHoldAndExtract {
            target_point_id,
            extraction_point_id,
            ..
        } => {
            assert_known_point(encounter, point_ids, target_point_id, "battle objective");
            assert_known_point(
                encounter,
                point_ids,
                extraction_point_id,
                "battle objective",
            );
        }
        PveBattleObjectiveData::SuppressAll
        | PveBattleObjectiveData::DefeatBoss { .. }
        | PveBattleObjectiveData::ProtectUnit { .. }
        | PveBattleObjectiveData::ProtectUnitForDuration { .. }
        | PveBattleObjectiveData::Survive { .. } => {}
    }
}

fn validate_enemy_plan_points(
    encounter: &PveEncounter,
    point_ids: &HashSet<&str>,
    enemy_plan: &PveEnemyMovementPlanData,
) {
    match enemy_plan {
        PveEnemyMovementPlanData::AssaultPlayer => {}
        PveEnemyMovementPlanData::PathToPoint { point_id } => {
            assert_known_point(encounter, point_ids, point_id, "enemy movement plan");
        }
        PveEnemyMovementPlanData::PathAlongPath { point_ids: path } => {
            assert!(
                !path.is_empty(),
                "enemy PathAlongPath in pve encounter '{}' must not be empty",
                encounter.id
            );
            for point_id in path {
                assert_known_point(encounter, point_ids, point_id, "enemy movement path");
            }
        }
    }
}

fn validate_win_condition_points(
    encounter: &PveEncounter,
    point_ids: &HashSet<&str>,
    win_condition: &PveWinConditionData,
) {
    match win_condition {
        PveWinConditionData::RecoverHoldAndExtract {
            target_point_id,
            extraction_point_id,
            ..
        } => {
            assert_known_point(encounter, point_ids, target_point_id, "win condition");
            assert_known_point(encounter, point_ids, extraction_point_id, "win condition");
        }
        PveWinConditionData::AllRequiredEnemyGroupsDefeated
        | PveWinConditionData::DefeatUnit { .. }
        | PveWinConditionData::ProtectUnit { .. }
        | PveWinConditionData::ProtectUnitForDuration { .. }
        | PveWinConditionData::SurviveUntil { .. } => {}
    }
}

fn assert_known_point(
    encounter: &PveEncounter,
    point_ids: &HashSet<&str>,
    point_id: &str,
    label: &str,
) {
    assert!(
        point_ids.contains(point_id),
        "{label} references unknown tactical point '{}' in pve encounter '{}'",
        point_id,
        encounter.id
    );
}

fn validate_objective_win_condition_consistency(
    encounter: &PveEncounter,
    objective: &PveBattleObjectiveData,
    win_condition: &PveWinConditionData,
) {
    let Some(objective_contract) = ObjectiveWinContract::from_objective(objective) else {
        return;
    };
    let win_contract = ObjectiveWinContract::from_win_condition(win_condition);
    assert_eq!(
        objective_contract, win_contract,
        "pve encounter '{}' tactical objective {:?} conflicts with win condition {:?}",
        encounter.id, objective, win_condition
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObjectiveWinContract {
    SuppressAll,
    DefeatUnit(String),
    ProtectUnit(String),
    Survive,
    Recover {
        target_point_id: String,
        extraction_point_id: String,
    },
}

impl ObjectiveWinContract {
    fn from_objective(objective: &PveBattleObjectiveData) -> Option<Self> {
        match objective {
            PveBattleObjectiveData::SuppressAll | PveBattleObjectiveData::DefeatBoss { .. } => None,
            PveBattleObjectiveData::ProtectUnit { unit_ref }
            | PveBattleObjectiveData::ProtectUnitForDuration { unit_ref, .. } => {
                Some(Self::ProtectUnit(unit_ref.clone()))
            }
            PveBattleObjectiveData::Survive { .. } => Some(Self::Survive),
            PveBattleObjectiveData::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                ..
            } => Some(Self::Recover {
                target_point_id: target_point_id.clone(),
                extraction_point_id: extraction_point_id.clone(),
            }),
            PveBattleObjectiveData::HoldArea { .. }
            | PveBattleObjectiveData::AdvanceToPoint { .. } => None,
        }
    }

    fn from_win_condition(win_condition: &PveWinConditionData) -> Self {
        match win_condition {
            PveWinConditionData::AllRequiredEnemyGroupsDefeated => Self::SuppressAll,
            PveWinConditionData::DefeatUnit { unit_ref } => Self::DefeatUnit(unit_ref.clone()),
            PveWinConditionData::ProtectUnit { unit_ref }
            | PveWinConditionData::ProtectUnitForDuration { unit_ref, .. } => {
                Self::ProtectUnit(unit_ref.clone())
            }
            PveWinConditionData::SurviveUntil { .. } => Self::Survive,
            PveWinConditionData::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                ..
            } => Self::Recover {
                target_point_id: target_point_id.clone(),
                extraction_point_id: extraction_point_id.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pve_encounter_deserializes_static_obstacles_and_defaults_to_empty() {
        let with_obstacle: PveEncounter = ron::de::from_str(
            r#"(
                id: "with_obstacle",
                abnormality_id: "enemy",
                difficulty: 1,
                risk_level: ZAYIN,
                waves: [],
                static_obstacles: [
                    (position: (x: 2, y: 3)),
                ],
            )"#,
        )
        .expect("encounter with obstacle should deserialize");
        assert_eq!(with_obstacle.static_obstacles.len(), 1);
        assert_eq!(with_obstacle.static_obstacles[0].position.x, 2);
        assert_eq!(with_obstacle.static_obstacles[0].position.y, 3);

        let without_obstacle: PveEncounter = ron::de::from_str(
            r#"(
                id: "without_obstacle",
                abnormality_id: "enemy",
                difficulty: 1,
                risk_level: ZAYIN,
                waves: [],
            )"#,
        )
        .expect("encounter without obstacles should deserialize");
        assert!(without_obstacle.static_obstacles.is_empty());
    }

    #[test]
    fn pve_encounter_deserializes_authored_waves() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "with_waves",
                abnormality_id: "enemy",
                difficulty: 1,
                risk_level: ZAYIN,
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        spawn_zone_ids: ["north_entry"],
                        required_for_victory: true,
                        enemies: [
                            (kind: CorrodedEmployee, abnormality_id: "enemy_a", tier: I, count: 2),
                            (abnormality_id: "enemy_b", tier: II, count: 1),
                        ],
                    ),
                    (
                        id: "wave_1",
                        time_ms: 15000,
                        enemies: [
                            (abnormality_id: "enemy_c"),
                        ],
                    ),
                ],
            )"#,
        )
        .expect("authored encounter waves should deserialize");

        assert_eq!(encounter.waves.len(), 2);
        assert_eq!(encounter.waves[0].spawn_zone_ids, ["north_entry"]);
        assert!(encounter.waves[0].required_for_victory);
        assert!(encounter.waves[1].required_for_victory);
        assert_eq!(encounter.waves[0].enemies[0].count, 2);
        assert_eq!(
            encounter.waves[0].enemies[0].kind,
            EnemyKind::CorrodedEmployee
        );
        assert_eq!(encounter.waves[0].enemies[1].kind, EnemyKind::Abnormality);
        assert_eq!(encounter.waves[1].enemies[0].tier, Tier::I);
        assert_eq!(encounter.waves[1].enemies[0].kind, EnemyKind::Abnormality);
        assert_eq!(encounter.wave_definitions(), encounter.waves);
    }

    #[test]
    fn pve_encounter_deserializes_scenario_authoring_overrides() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "protect_object_route",
                abnormality_id: "enemy",
                difficulty: 2,
                risk_level: HE,
                node_type: Some(Defense),
                battlefield: Some((
                    archetype: Some(ChokePoint),
                    size_class: Some(Small),
                )),
                tactical_plan: Some((
                    points: [
                        (id: "black_box_recovery", position: (x: 3, y: 6)),
                    ],
                    objective: Some(ProtectUnit(unit_ref: "black_box")),
                    enemy_plan: Some(PathToPoint(point_id: "black_box_recovery")),
                )),
                win_condition: Some(ProtectUnit(unit_ref: "black_box")),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        spawn_zone_ids: ["north_entry"],
                        required_for_victory: false,
                        enemies: [
                            (abnormality_id: "enemy", tier: I, count: 1),
                        ],
                    ),
                ],
            )"#,
        )
        .expect("scenario authoring overrides should deserialize");

        assert_eq!(encounter.node_type, Some(CombatNodeType::Defense));
        assert_eq!(
            encounter
                .battlefield
                .as_ref()
                .and_then(|battlefield| battlefield.archetype),
            Some(BattlefieldArchetype::ChokePoint)
        );
        assert!(!encounter.waves[0].required_for_victory);

        let mut plan = TacticalPlan::free_engage();
        encounter.apply_authored_tactical_plan(&mut plan);
        assert_eq!(plan.points.len(), 1);
        assert!(matches!(
            plan.objective,
            BattleObjective::ProtectUnit { ref unit_ref } if unit_ref.0 == "black_box"
        ));
        assert!(matches!(
            plan.enemy_plan,
            EnemyMovementPlan::PathToPoint { ref point_id } if point_id.0 == "black_box_recovery"
        ));
        assert!(matches!(
            encounter.authored_win_condition(),
            Some(WinCondition::ProtectUnit { ref unit_ref }) if unit_ref.0 == "black_box"
        ));
    }

    #[test]
    #[should_panic(expected = "must define at least one wave")]
    fn pve_encounter_validation_rejects_empty_wave_contract() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "empty_waves",
                abnormality_id: "enemy",
                difficulty: 1,
                risk_level: ZAYIN,
                waves: [],
            )"#,
        )
        .expect("empty wave contract should deserialize before validation");

        PveEncounterDatabase::new(vec![encounter]).validate_indexes();
    }

    #[test]
    #[should_panic(expected = "unknown tactical point")]
    fn pve_encounter_validation_rejects_unknown_tactical_point_reference() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "bad_route",
                abnormality_id: "enemy",
                difficulty: 2,
                risk_level: HE,
                tactical_plan: Some((
                    points: [],
                    enemy_plan: Some(PathToPoint(point_id: "missing")),
                )),
                waves: [
                    (
                        id: "wave_0",
                        enemies: [
                            (abnormality_id: "enemy"),
                        ],
                    ),
                ],
            )"#,
        )
        .expect("invalid authored reference should deserialize before validation");

        PveEncounterDatabase::new(vec![encounter]).validate_indexes();
    }

    #[test]
    #[should_panic(expected = "conflicts with win condition")]
    fn pve_encounter_validation_rejects_conflicting_objective_and_win_condition() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "conflicting_contract",
                abnormality_id: "enemy",
                difficulty: 2,
                risk_level: HE,
                tactical_plan: Some((
                    objective: Some(Survive(time_ms: 30000)),
                )),
                win_condition: Some(AllRequiredEnemyGroupsDefeated),
                waves: [
                    (
                        id: "wave_0",
                        enemies: [
                            (abnormality_id: "enemy"),
                        ],
                    ),
                ],
            )"#,
        )
        .expect("conflicting authored contract should deserialize before validation");

        PveEncounterDatabase::new(vec![encounter]).validate_indexes();
    }
}
