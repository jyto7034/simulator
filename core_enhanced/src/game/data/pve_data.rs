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
            BattleObjective, TacticalPlan, TacticalPoint, TacticalPointId, WinCondition,
        },
        combat_preview::{
            BattlefieldArchetype, BattlefieldSizeClass, CombatMissionVariant, CombatNodeType,
            EnemyKind,
        },
        data::{build_string_index, once_lock_with},
        enums::{RewardMode, RiskLevel, Tier},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PveEncounterClass {
    Normal,
    Elite,
    NormalBoss,
    FinalBoss,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum PveWaveEnemyData {
    Abnormality {
        abnormality_id: String,
        tier: Tier,
        count: u32,
    },
    CorrodedEmployee {
        profile_id: String,
        tier: Tier,
        count: u32,
    },
    FacilityEntity {
        profile_id: String,
        tier: Tier,
        count: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct PveWaveData {
    pub id: String,
    pub time_ms: u32,
    #[serde(default)]
    pub spawn_zone_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(default = "default_required_for_victory")]
    pub required_for_victory: bool,
    pub source: PveWaveSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveStaticObstacleData {
    pub position: PvePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PveBattlefieldOverrideData {
    #[serde(default)]
    pub archetype: Option<BattlefieldArchetype>,
    #[serde(default)]
    pub size_class: Option<BattlefieldSizeClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveTacticalPointData {
    pub id: String,
    pub position: PvePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PveBattleObjectiveData {
    DefeatBoss { unit_ref: String },
    ProtectUnit { unit_ref: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PveWinConditionData {
    AllRequiredEnemyGroupsDefeated,
    DefeatUnit { unit_ref: String },
    ProtectUnit { unit_ref: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveSuppressionResearchData {
    #[serde(default)]
    pub bonus_objectives: Vec<PveBonusObjectiveData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveBonusObjectiveData {
    pub id: String,
    pub condition: PveBonusObjectiveConditionData,
    pub research_bonus: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PveBonusObjectiveConditionData {
    ClearWithin {
        time_ms: u64,
    },
    DecisiveDamage {
        minimum_damage_percent_of_max_hp: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PveTacticalPlanData {
    #[serde(default)]
    pub points: Vec<PveTacticalPointData>,
    #[serde(default)]
    pub objective: Option<PveBattleObjectiveData>,
}

fn default_reward_mode() -> RewardMode {
    RewardMode::ClaimAll
}

fn default_required_for_victory() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct PveEncounter {
    pub id: String,
    pub encounter_class: PveEncounterClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_abnormality_id: Option<String>,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub node_type: Option<CombatNodeType>,
    #[serde(default)]
    pub mission_variant: Option<CombatMissionVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survive_timer_ms: Option<u64>,
    #[serde(default = "default_reward_mode")]
    pub reward_mode: RewardMode,
    #[serde(default)]
    pub reward_uuids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppression_research: Option<PveSuppressionResearchData>,
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
    pub fn primary_abnormality_id(&self) -> Option<&str> {
        self.primary_abnormality_id.as_deref()
    }

    pub fn requires_primary_abnormality(&self) -> bool {
        matches!(
            self.encounter_class,
            PveEncounterClass::Elite | PveEncounterClass::NormalBoss | PveEncounterClass::FinalBoss
        )
    }

    pub fn bonus_objectives(&self) -> &[PveBonusObjectiveData] {
        self.suppression_research
            .as_ref()
            .map(|research| research.bonus_objectives.as_slice())
            .unwrap_or(&[])
    }

    pub fn has_bonus_objectives(&self) -> bool {
        !self.bonus_objectives().is_empty()
    }

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
            PveWaveSource::Manual(enemies) => enemies,
            PveWaveSource::GeneratedCorroded { .. } => &[],
        }
    }
}

impl PveWaveEnemyData {
    pub fn kind(&self) -> EnemyKind {
        match self {
            Self::Abnormality { .. } => EnemyKind::Abnormality,
            Self::CorrodedEmployee { .. } => EnemyKind::CorrodedEmployee,
            Self::FacilityEntity { .. } => EnemyKind::FacilityEntity,
        }
    }

    pub fn tier(&self) -> Tier {
        match self {
            Self::Abnormality { tier, .. }
            | Self::CorrodedEmployee { tier, .. }
            | Self::FacilityEntity { tier, .. } => *tier,
        }
    }

    pub fn count(&self) -> u32 {
        match self {
            Self::Abnormality { count, .. }
            | Self::CorrodedEmployee { count, .. }
            | Self::FacilityEntity { count, .. } => *count,
        }
    }

    pub fn abnormality_id(&self) -> Option<&str> {
        match self {
            Self::Abnormality { abnormality_id, .. } => Some(abnormality_id.as_str()),
            Self::CorrodedEmployee { .. } | Self::FacilityEntity { .. } => None,
        }
    }

    pub fn profile_id(&self) -> Option<&str> {
        match self {
            Self::Abnormality { .. } => None,
            Self::CorrodedEmployee { profile_id, .. } | Self::FacilityEntity { profile_id, .. } => {
                Some(profile_id.as_str())
            }
        }
    }
}

impl PveBattleObjectiveData {
    fn to_battle_objective(&self) -> BattleObjective {
        match self {
            Self::DefeatBoss { unit_ref } => BattleObjective::DefeatBoss {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
            },
            Self::ProtectUnit { unit_ref } => BattleObjective::ProtectUnit {
                unit_ref: crate::game::battle::scenario::ScenarioUnitRef::new(unit_ref.clone()),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveEncounterDatabase {
    pub encounters: Vec<PveEncounter>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl PveEncounterDatabase {
    pub fn new(encounters: Vec<PveEncounter>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter id",
            |encounter| &encounter.id,
        ));
        Self { encounters, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.encounters, "pve encounter id", |encounter| {
                &encounter.id
            })
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        self.validate_contract()
            .expect("invalid pve encounter authoring contract");
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (encounter_index, encounter) in self.encounters.iter().enumerate() {
            validate_encounter_authoring_contract(
                encounter,
                &format!("encounters[{encounter_index}]"),
            )?;
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&PveEncounter> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_by_risk_level(&self, level: RiskLevel) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.risk_level == level)
            .collect()
    }
}

fn pve_error(
    encounter: &PveEncounter,
    field_path: impl Into<String>,
    cause: impl Into<String>,
) -> crate::game::data::DataValidationError {
    crate::game::data::DataValidationError::at(&encounter.id, field_path, cause)
}

fn validate_encounter_authoring_contract(
    encounter: &PveEncounter,
    encounter_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if encounter.waves.is_empty() {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.waves"),
            "encounter must define at least one wave",
        ));
    }

    match encounter.encounter_class {
        PveEncounterClass::Normal => {
            if encounter.primary_abnormality_id.is_some() {
                return Err(pve_error(
                    encounter,
                    format!("{encounter_path}.primary_abnormality_id"),
                    "normal encounter must not define primary_abnormality_id",
                ));
            }
            if encounter.suppression_research.is_some() {
                return Err(pve_error(
                    encounter,
                    format!("{encounter_path}.suppression_research"),
                    "normal encounter must not define suppression_research",
                ));
            }
        }
        PveEncounterClass::Elite | PveEncounterClass::NormalBoss | PveEncounterClass::FinalBoss => {
            if !encounter
                .primary_abnormality_id
                .as_deref()
                .is_some_and(|id| !id.trim().is_empty())
            {
                return Err(pve_error(
                    encounter,
                    format!("{encounter_path}.primary_abnormality_id"),
                    format!(
                        "{:?} encounter must define primary_abnormality_id",
                        encounter.encounter_class
                    ),
                ));
            }
        }
    }

    let mut bonus_objective_ids = HashSet::new();
    for (objective_index, objective) in encounter.bonus_objectives().iter().enumerate() {
        let objective_path =
            format!("{encounter_path}.suppression_research.bonus_objectives[{objective_index}]");
        if objective.id.trim().is_empty() {
            return Err(pve_error(
                encounter,
                format!("{objective_path}.id"),
                "bonus objective id must not be empty",
            ));
        }
        if !bonus_objective_ids.insert(objective.id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{objective_path}.id"),
                format!("duplicate bonus objective id '{}'", objective.id),
            ));
        }
        if objective.research_bonus == 0 {
            return Err(pve_error(
                encounter,
                format!("{objective_path}.research_bonus"),
                "research_bonus must be greater than zero",
            ));
        }
        if let Some(presentation) = &objective.presentation {
            if presentation.trim().is_empty() {
                return Err(pve_error(
                    encounter,
                    format!("{objective_path}.presentation"),
                    "presentation must not be empty",
                ));
            }
        }
        match objective.condition {
            PveBonusObjectiveConditionData::ClearWithin { time_ms } => {
                if time_ms == 0 {
                    return Err(pve_error(
                        encounter,
                        format!("{objective_path}.condition.ClearWithin.time_ms"),
                        "time_ms must be greater than zero",
                    ));
                }
            }
            PveBonusObjectiveConditionData::DecisiveDamage {
                minimum_damage_percent_of_max_hp,
            } => {
                if minimum_damage_percent_of_max_hp == 0 {
                    return Err(pve_error(
                        encounter,
                        format!(
                            "{objective_path}.condition.DecisiveDamage.minimum_damage_percent_of_max_hp"
                        ),
                        "minimum_damage_percent_of_max_hp must be greater than zero",
                    ));
                }
            }
        }
    }
    let mut primary_target_count = 0_u32;
    let mut non_primary_abnormalities = Vec::new();
    let primary_abnormality_id = encounter.primary_abnormality_id();
    for wave in &encounter.waves {
        for enemy in wave.manual_enemies() {
            if let PveWaveEnemyData::Abnormality {
                abnormality_id,
                count,
                ..
            } = enemy
            {
                if Some(abnormality_id.as_str()) == primary_abnormality_id {
                    primary_target_count = primary_target_count.saturating_add(*count);
                } else {
                    non_primary_abnormalities.push(abnormality_id.as_str());
                }
            }
        }
    }

    if encounter.encounter_class == PveEncounterClass::Normal {
        if primary_target_count != 0 || !non_primary_abnormalities.is_empty() {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves"),
                "normal encounter must spawn only corroded employees",
            ));
        }
    }

    if encounter.requires_primary_abnormality() {
        let primary = encounter.primary_abnormality_id().ok_or_else(|| {
            pve_error(
                encounter,
                format!("{encounter_path}.primary_abnormality_id"),
                "required primary abnormality id is missing",
            )
        })?;
        if primary_target_count == 0 {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves"),
                format!(
                    "{:?} encounter must spawn its primary abnormality target '{primary}'",
                    encounter.encounter_class
                ),
            ));
        }
        if !non_primary_abnormalities.is_empty() {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves"),
                format!(
                    "{:?} encounter spawns non-primary abnormality targets {:?}; multi-primary target policy is out of scope",
                    encounter.encounter_class, non_primary_abnormalities
                ),
            ));
        }
    }

    if encounter.has_bonus_objectives() {
        let primary = encounter.primary_abnormality_id().ok_or_else(|| {
            pve_error(
                encounter,
                format!("{encounter_path}.primary_abnormality_id"),
                "bonus objective encounter must have a primary abnormality",
            )
        })?;
        if primary_target_count == 0 {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves"),
                format!(
                    "encounter defines bonus objectives but does not spawn its primary abnormality target '{primary}'"
                ),
            ));
        }
    }

    let mut wave_ids = HashSet::new();
    for (wave_index, wave) in encounter.waves.iter().enumerate() {
        let wave_path = format!("{encounter_path}.waves[{wave_index}]");
        if !wave_ids.insert(wave.id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{wave_path}.id"),
                format!("duplicate wave id '{}'", wave.id),
            ));
        }
        validate_pve_wave_authoring_contract(
            wave,
            encounter.node_type == Some(CombatNodeType::Defense),
        )
        .map_err(|cause| pve_error(encounter, wave_path, cause))?;
    }

    let Some(tactical_plan) = &encounter.tactical_plan else {
        return Ok(());
    };

    let mut point_ids = HashSet::new();
    for (point_index, point) in tactical_plan.points.iter().enumerate() {
        if !point_ids.insert(point.id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.tactical_plan.points[{point_index}].id"),
                format!("duplicate tactical point '{}'", point.id),
            ));
        }
    }

    if let (Some(objective), Some(win_condition)) =
        (&tactical_plan.objective, &encounter.win_condition)
    {
        validate_objective_win_condition_consistency(
            encounter,
            objective,
            win_condition,
            encounter_path,
        )?;
    }
    Ok(())
}

pub(super) fn validate_pve_wave_authoring_contract(
    wave: &PveWaveData,
    defense_route_required: bool,
) -> Result<(), String> {
    if wave.id.trim().is_empty() {
        return Err("wave id must not be empty".to_string());
    }
    match &wave.source {
        PveWaveSource::Manual(enemies) if enemies.is_empty() => {
            return Err("manual source must contain at least one enemy entry".to_string());
        }
        PveWaveSource::GeneratedCorroded {
            preset_id,
            budget_override,
            ..
        } => {
            if preset_id.trim().is_empty() {
                return Err("generated corroded source must reference a preset".to_string());
            }
            if budget_override.is_some_and(|budget| budget == 0) {
                return Err(
                    "generated corroded budget_override must be greater than zero".to_string(),
                );
            }
        }
        PveWaveSource::Manual(_) => {}
    }

    let mut spawn_zone_ids = HashSet::new();
    for spawn_zone_id in &wave.spawn_zone_ids {
        if spawn_zone_id.trim().is_empty() {
            return Err("spawn zone id must not be empty".to_string());
        }
        if !spawn_zone_ids.insert(spawn_zone_id.as_str()) {
            return Err(format!("duplicate spawn zone id '{spawn_zone_id}'"));
        }
    }
    if let Some(route_id) = &wave.route_id {
        if route_id.trim().is_empty() {
            return Err("route id must not be empty".to_string());
        }
    } else if defense_route_required {
        return Err("defense wave must define route_id".to_string());
    }

    for enemy in wave.manual_enemies() {
        let (kind, id) = match enemy {
            PveWaveEnemyData::Abnormality { abnormality_id, .. } => {
                ("enemy abnormality", abnormality_id)
            }
            PveWaveEnemyData::CorrodedEmployee { profile_id, .. } => {
                ("corroded employee profile", profile_id)
            }
            PveWaveEnemyData::FacilityEntity { profile_id, .. } => {
                ("facility entity profile", profile_id)
            }
        };
        if id.trim().is_empty() {
            return Err(format!("{kind} id must not be empty"));
        }
        if enemy.count() == 0 {
            return Err(format!("{kind} '{id}' count must be greater than zero"));
        }
    }
    Ok(())
}

fn validate_objective_win_condition_consistency(
    encounter: &PveEncounter,
    objective: &PveBattleObjectiveData,
    win_condition: &PveWinConditionData,
    encounter_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    let Some(objective_contract) = ObjectiveWinContract::from_objective(objective) else {
        return Ok(());
    };
    let win_contract = ObjectiveWinContract::from_win_condition(win_condition);
    if objective_contract != win_contract {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.tactical_plan.objective"),
            format!(
                "tactical objective {objective:?} conflicts with win condition {win_condition:?}"
            ),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObjectiveWinContract {
    DefeatAllRequiredEnemies,
    DefeatUnit(String),
    ProtectUnit(String),
}

impl ObjectiveWinContract {
    fn from_objective(objective: &PveBattleObjectiveData) -> Option<Self> {
        match objective {
            PveBattleObjectiveData::DefeatBoss { .. } => None,
            PveBattleObjectiveData::ProtectUnit { unit_ref } => {
                Some(Self::ProtectUnit(unit_ref.clone()))
            }
        }
    }

    fn from_win_condition(win_condition: &PveWinConditionData) -> Self {
        match win_condition {
            PveWinConditionData::AllRequiredEnemyGroupsDefeated => Self::DefeatAllRequiredEnemies,
            PveWinConditionData::DefeatUnit { unit_ref } => Self::DefeatUnit(unit_ref.clone()),
            PveWinConditionData::ProtectUnit { unit_ref } => Self::ProtectUnit(unit_ref.clone()),
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
                encounter_class: Normal,
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
                encounter_class: Normal,
                risk_level: ZAYIN,
                waves: [],
            )"#,
        )
        .expect("encounter without obstacles should deserialize");
        assert!(without_obstacle.static_obstacles.is_empty());
    }

    #[test]
    fn pve_encounter_rejects_unknown_authoring_fields() {
        let result = ron::de::from_str::<PveEncounter>(
            r#"(
                id: "typo_contract",
                encounter_class: Normal,
                risk_level: ZAYIN,
                encounter_difficulty: 3,
                waves: [
                    (
                        id: "wave_0",
                        source: Manual([
                            CorrodedEmployee(profile_id: "corroded_guard"),
                        ]),
                    ),
                ],
            )"#,
        );

        assert!(
            result.is_err(),
            "unknown gameplay authoring fields must not deserialize"
        );
        let error = result.expect_err("unknown field should be rejected");
        assert!(
            error.to_string().contains("encounter_difficulty"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn pve_wave_rejects_unknown_nested_authoring_fields() {
        let result = ron::de::from_str::<PveEncounter>(
            r#"(
                id: "typo_wave_contract",
                encounter_class: Normal,
                risk_level: ZAYIN,
                waves: [
                    (
                        id: "wave_0",
                        spawn_delay_ms: 1000,
                        source: Manual([
                            CorrodedEmployee(profile_id: "corroded_guard"),
                        ]),
                    ),
                ],
            )"#,
        );

        assert!(
            result.is_err(),
            "unknown nested wave fields must not deserialize"
        );
        let error = result.expect_err("unknown nested field should be rejected");
        assert!(
            error.to_string().contains("spawn_delay_ms"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn pve_encounter_deserializes_authored_waves() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "with_waves",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy_b"),
                risk_level: ZAYIN,
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        spawn_zone_ids: ["north_entry"],
                        required_for_victory: true,
                        source: Manual([
                            CorrodedEmployee(profile_id: "enemy_a", tier: I, count: 2),
                            Abnormality(abnormality_id: "enemy_b", tier: II, count: 1),
                        ]),
                    ),
                    (
                        id: "wave_1",
                        time_ms: 15000,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy_b", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("authored encounter waves should deserialize");

        assert_eq!(encounter.waves.len(), 2);
        assert_eq!(encounter.waves[0].spawn_zone_ids, ["north_entry"]);
        assert!(encounter.waves[0].required_for_victory);
        assert!(encounter.waves[1].required_for_victory);
        let PveWaveSource::Manual(first_wave) = &encounter.waves[0].source else {
            panic!("first wave should be manual");
        };
        let PveWaveSource::Manual(second_wave) = &encounter.waves[1].source else {
            panic!("second wave should be manual");
        };
        assert_eq!(first_wave[0].count(), 2);
        assert_eq!(first_wave[0].kind(), EnemyKind::CorrodedEmployee);
        assert_eq!(first_wave[1].kind(), EnemyKind::Abnormality);
        assert_eq!(second_wave[0].tier(), Tier::I);
        assert_eq!(second_wave[0].kind(), EnemyKind::Abnormality);
        assert_eq!(encounter.wave_definitions(), encounter.waves);
    }

    #[test]
    fn pve_encounter_deserializes_scenario_authoring_overrides() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "protect_object_route",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                node_type: Some(Defense),
                survive_timer_ms: Some(45000),
                battlefield: Some((
                    archetype: Some(ChokePoint),
                    size_class: Some(Small),
                )),
                tactical_plan: Some((
                    points: [
                        (id: "black_box_anchor", position: (x: 3, y: 6)),
                    ],
                    objective: Some(ProtectUnit(unit_ref: "black_box")),
                )),
                win_condition: Some(ProtectUnit(unit_ref: "black_box")),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        spawn_zone_ids: ["north_entry"],
                        required_for_victory: false,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("scenario authoring overrides should deserialize");

        assert_eq!(encounter.node_type, Some(CombatNodeType::Defense));
        assert_eq!(encounter.survive_timer_ms, Some(45_000));
        assert_eq!(
            encounter
                .battlefield
                .as_ref()
                .and_then(|battlefield| battlefield.archetype),
            Some(BattlefieldArchetype::ChokePoint)
        );
        assert!(!encounter.waves[0].required_for_victory);

        let mut plan = TacticalPlan::default();
        encounter.apply_authored_tactical_plan(&mut plan);
        assert_eq!(plan.points.len(), 1);
        assert!(matches!(
            plan.objective,
            BattleObjective::ProtectUnit { ref unit_ref } if unit_ref.0 == "black_box"
        ));
        assert!(matches!(
            encounter.authored_win_condition(),
            Some(WinCondition::ProtectUnit { ref unit_ref }) if unit_ref.0 == "black_box"
        ));
    }

    #[test]
    fn pve_encounter_validation_rejects_empty_wave_contract() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "empty_waves",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: ZAYIN,
                waves: [],
            )"#,
        )
        .expect("empty wave contract should deserialize before validation");

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("empty wave contract must reject");
        assert_eq!(error.object_id.as_deref(), Some("empty_waves"));
        assert_eq!(error.field_path.as_deref(), Some("encounters[0].waves"));
        assert!(error.cause.contains("at least one wave"));
    }

    #[test]
    fn pve_encounter_validation_rejects_conflicting_objective_and_win_condition() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "conflicting_contract",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                tactical_plan: Some((
                    objective: Some(ProtectUnit(unit_ref: "black_box")),
                )),
                win_condition: Some(AllRequiredEnemyGroupsDefeated),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("conflicting authored contract should deserialize before validation");

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("conflicting objective and win condition must reject");
        assert_eq!(error.object_id.as_deref(), Some("conflicting_contract"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].tactical_plan.objective")
        );
        assert!(error.cause.contains("conflicts with win condition"));
    }

    #[test]
    fn pve_encounter_deserializes_bonus_objectives() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "bonus_objective_contract",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                suppression_research: Some((
                    bonus_objectives: [
                        (
                            id: "fast_clear",
                            condition: ClearWithin(time_ms: 90000),
                            research_bonus: 20,
                            presentation: Some("abnormality_part_obtained"),
                        ),
                        (
                            id: "decisive_damage",
                            condition: DecisiveDamage(minimum_damage_percent_of_max_hp: 10),
                            research_bonus: 30,
                        ),
                    ],
                )),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("bonus objective contract should deserialize");

        assert_eq!(encounter.bonus_objectives().len(), 2);
        PveEncounterDatabase::new(vec![encounter]).validate_indexes();
    }

    #[test]
    fn pve_encounter_validation_rejects_duplicate_bonus_objective_ids() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "duplicate_bonus_objectives",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                suppression_research: Some((
                    bonus_objectives: [
                        (id: "same", condition: ClearWithin(time_ms: 90000), research_bonus: 20),
                        (id: "same", condition: DecisiveDamage(minimum_damage_percent_of_max_hp: 10), research_bonus: 20),
                    ],
                )),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("duplicate bonus objective contract should deserialize before validation");

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("duplicate bonus objective id must reject");
        assert_eq!(
            error.object_id.as_deref(),
            Some("duplicate_bonus_objectives")
        );
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].suppression_research.bonus_objectives[1].id")
        );
        assert!(error.cause.contains("duplicate bonus objective id"));
    }

    #[test]
    fn pve_encounter_validation_rejects_zero_bonus_research() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "zero_bonus_research",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                suppression_research: Some((
                    bonus_objectives: [
                        (id: "fast_clear", condition: ClearWithin(time_ms: 90000), research_bonus: 0),
                    ],
                )),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        source: Manual([
                            Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("zero bonus objective contract should deserialize before validation");

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("zero research bonus must reject");
        assert_eq!(error.object_id.as_deref(), Some("zero_bonus_research"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].suppression_research.bonus_objectives[0].research_bonus")
        );
        assert!(error.cause.contains("greater than zero"));
    }

    #[test]
    fn pve_encounter_validation_rejects_bonus_objective_without_primary_target_spawn() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "missing_primary_bonus_target",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                suppression_research: Some((
                    bonus_objectives: [
                        (id: "fast_clear", condition: ClearWithin(time_ms: 90000), research_bonus: 20),
                    ],
                )),
                waves: [
                    (
                        id: "wave_0",
                        time_ms: 0,
                        source: Manual([
                            CorrodedEmployee(profile_id: "corroded_guard", tier: I, count: 1),
                        ]),
                    ),
                ],
            )"#,
        )
        .expect("missing primary target contract should deserialize before validation");

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("missing primary target spawn must reject");
        assert_eq!(
            error.object_id.as_deref(),
            Some("missing_primary_bonus_target")
        );
        assert_eq!(error.field_path.as_deref(), Some("encounters[0].waves"));
        assert!(error
            .cause
            .contains("must spawn its primary abnormality target"));
    }
}
