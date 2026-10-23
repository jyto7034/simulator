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
            BattleObjective, FieldWorkCellDef, FieldWorkCellId, FieldWorkCompletionEffect,
            FieldWorkObjectDef, FieldWorkObjectId, FieldWorkPresentationKind, ScenarioUnitRef,
            TacticalPlan, TacticalPoint, TacticalPointId, TerminalCondition, TerminalOutcome,
        },
        battle::types::{BattleOutcomeId, BattleVerdict, TacticalTileKind},
        combat_preview::{
            BattlefieldArchetype, BattlefieldSizeClass, CombatMissionVariant, CombatNodeType,
            EnemyKind,
        },
        data::{
            boss_runtime_data::{
                BossActorSlotId, BossEncounterRuntimeId, BossRuntimeDatabase, BossSignalId,
            },
            build_string_index, once_lock_with,
            skill_fragment_data::SkillFragmentId,
        },
        enums::{RewardMode, RiskLevel, Tier},
        reward::RewardEffect,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PveEncounterClass {
    Normal,
    /// Activity-only continuous defense; excluded from ordinary map selectors.
    StressCombat,
    Elite,
    NormalBoss,
    FinalBoss,
    /// Nonproduction, opt-in reference content. It may exercise the FinalBoss
    /// runtime shape, but production selectors, rewards and progression must
    /// never treat it as a live FinalBoss.
    ReferenceFinalBoss,
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
pub struct PveWaveActivationData {
    pub trigger_id: String,
    pub priority: i32,
    pub primary: PveWavePrimaryTriggerData,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,
}

impl PveWaveActivationData {
    pub fn at_time(trigger_id: impl Into<String>, time_ms: u64) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            priority: 0,
            primary: PveWavePrimaryTriggerData::AtTimeMs { time_ms },
            deadline_ms: None,
        }
    }

    pub fn at_time_ms(&self) -> Option<u64> {
        match self.primary {
            PveWavePrimaryTriggerData::AtTimeMs { time_ms } => Some(time_ms),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum PveWavePrimaryTriggerData {
    AtTimeMs {
        time_ms: u64,
    },
    GroupCleared {
        wave_id: String,
    },
    ActorHealthAtOrBelowPercent {
        actor_slot_id: BossActorSlotId,
        percent: u8,
    },
    ActorDefeated {
        actor_slot_id: BossActorSlotId,
    },
    ActorReachedPoint {
        actor_slot_id: BossActorSlotId,
        point_id: String,
        radius_cells: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveWaveData {
    pub id: String,
    pub activation: PveWaveActivationData,
    #[serde(default)]
    pub spawn_zone_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum PveTerminalConditionData {
    AllRequiredEnemyGroupsDefeated,
    UnitDefeated { unit_ref: String },
    AllPlayerCombatantsDefeated,
    TimeReached { time_ms: u64 },
    BossSignalLatched { signal_id: BossSignalId },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveTerminalOutcomeData {
    pub id: String,
    pub verdict: BattleVerdict,
    pub condition: PveTerminalConditionData,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveOutcomePolicyData {
    pub outcome_id: String,
    pub reward_mode: RewardMode,
    pub reward_uuids: Vec<Uuid>,
    pub grant_node_bonus: bool,
    pub advance_encounter_progression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PveTacticalPlanData {
    #[serde(default)]
    pub points: Vec<PveTacticalPointData>,
    #[serde(default)]
    pub objective: Option<PveBattleObjectiveData>,
    #[serde(default)]
    pub field_work_objects: Vec<PveFieldWorkObjectData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveFieldWorkObjectData {
    pub id: String,
    pub presentation_kind: PveFieldWorkPresentationKind,
    pub position: PvePosition,
    pub work_cells: Vec<PveFieldWorkCellData>,
    pub required_work_ms: u64,
    pub completion_effect: PveFieldWorkCompletionEffectData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveFieldWorkCellData {
    pub id: String,
    pub position: PvePosition,
    pub tile_kind: TacticalTileKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PveFieldWorkPresentationKind {
    EmergencyRepairTerminal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum PveFieldWorkCompletionEffectData {
    RepairProtectedUnit { unit_ref: String, flat_hp: u32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    #[serde(default)]
    pub deterrence_tuning: PveDeterrenceTuning,
    #[serde(default)]
    pub bonus_objectives: Vec<PveBonusObjective>,
    pub id: String,
    pub encounter_class: PveEncounterClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_abnormality_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_runtime: Option<PveBossRuntimeBindingData>,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub node_type: Option<CombatNodeType>,
    #[serde(default)]
    pub mission_variant: Option<CombatMissionVariant>,
    #[serde(default)]
    pub terminal_outcomes: Vec<PveTerminalOutcomeData>,
    #[serde(default)]
    pub outcome_policies: Vec<PveOutcomePolicyData>,
    #[serde(default)]
    pub battlefield: Option<PveBattlefieldOverrideData>,
    #[serde(default)]
    pub tactical_plan: Option<PveTacticalPlanData>,
    #[serde(default)]
    pub waves: Vec<PveWaveData>,
    #[serde(default)]
    pub static_obstacles: Vec<PveStaticObstacleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveBonusObjective {
    pub id: String,
    pub condition: PveBonusCondition,
    pub effects: Vec<RewardEffect>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PveBonusCondition {
    ClearWithin { time_ms: u64 },
    DecisiveDamage { max_health_percent: u32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveDeterrenceTuning {
    pub unrestrained_health_percent: u32,
    pub unrestrained_attack_percent: u32,
    pub seal_health_percent: u32,
    pub seal_attack_percent: u32,
}
impl Default for PveDeterrenceTuning {
    fn default() -> Self {
        Self { unrestrained_health_percent: 150, unrestrained_attack_percent: 150,
            seal_health_percent: 150, seal_attack_percent: 150 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveBossRuntimeBindingData {
    pub encounter_id: BossEncounterRuntimeId,
    pub actors: Vec<PveBossActorBindingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PveBossActorBindingData {
    pub actor_slot_id: BossActorSlotId,
    pub wave_id: String,
    pub spawn_index: u32,
}

impl PveEncounter {
    pub fn is_rewardless_story_encounter(&self) -> bool {
        self.primary_abnormality_id() == Some("t-02-43_freischutz")
    }

    #[cfg(test)]
    pub(crate) fn fixture_terminal_outcomes(
        survive_timer_ms: Option<u64>,
    ) -> Vec<PveTerminalOutcomeData> {
        vec![
            PveTerminalOutcomeData {
                id: "victory".to_string(),
                verdict: BattleVerdict::Victory,
                condition: survive_timer_ms.map_or(
                    PveTerminalConditionData::AllRequiredEnemyGroupsDefeated,
                    |time_ms| PveTerminalConditionData::TimeReached { time_ms },
                ),
                priority: 0,
            },
            PveTerminalOutcomeData {
                id: "defeat".to_string(),
                verdict: BattleVerdict::Defeat,
                condition: PveTerminalConditionData::AllPlayerCombatantsDefeated,
                priority: 0,
            },
        ]
    }

    #[cfg(test)]
    pub(crate) fn fixture_protected_terminal_outcomes(
        protected_unit_ref: &str,
        survive_timer_ms: Option<u64>,
    ) -> Vec<PveTerminalOutcomeData> {
        let mut outcomes = Self::fixture_terminal_outcomes(survive_timer_ms);
        outcomes[1].condition = PveTerminalConditionData::UnitDefeated {
            unit_ref: protected_unit_ref.to_string(),
        };
        outcomes
    }

    #[cfg(test)]
    pub(crate) fn fixture_outcome_policies(
        reward_mode: RewardMode,
        reward_uuids: Vec<Uuid>,
    ) -> Vec<PveOutcomePolicyData> {
        vec![
            PveOutcomePolicyData {
                outcome_id: "victory".to_string(),
                reward_mode,
                reward_uuids,
                grant_node_bonus: true,
                advance_encounter_progression: true,
            },
            PveOutcomePolicyData {
                outcome_id: "defeat".to_string(),
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                grant_node_bonus: false,
                advance_encounter_progression: false,
            },
        ]
    }

    pub fn primary_abnormality_id(&self) -> Option<&str> {
        self.primary_abnormality_id.as_deref()
    }

    pub fn requires_primary_abnormality(&self) -> bool {
        matches!(
            self.encounter_class,
            PveEncounterClass::Elite
                | PveEncounterClass::NormalBoss
                | PveEncounterClass::FinalBoss
                | PveEncounterClass::ReferenceFinalBoss
        )
    }

    pub fn is_reference_only(&self) -> bool {
        self.encounter_class == PveEncounterClass::ReferenceFinalBoss
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
        plan.field_work_objects = authored
            .field_work_objects
            .iter()
            .map(PveFieldWorkObjectData::to_scenario_def)
            .collect();
    }

    pub fn authored_terminal_outcomes(&self) -> Vec<TerminalOutcome> {
        self.terminal_outcomes
            .iter()
            .map(PveTerminalOutcomeData::to_terminal_outcome)
            .collect()
    }

    pub fn survive_timer_ms(&self) -> Option<u64> {
        self.terminal_outcomes.iter().find_map(|outcome| {
            (outcome.verdict == BattleVerdict::Victory)
                .then_some(&outcome.condition)
                .and_then(|condition| match condition {
                    PveTerminalConditionData::TimeReached { time_ms } => Some(*time_ms),
                    _ => None,
                })
        })
    }

    pub fn outcome_policy(&self, outcome_id: &BattleOutcomeId) -> Option<&PveOutcomePolicyData> {
        self.outcome_policies
            .iter()
            .find(|policy| policy.outcome_id == outcome_id.as_str())
    }
}

impl PveFieldWorkObjectData {
    fn to_scenario_def(&self) -> FieldWorkObjectDef {
        FieldWorkObjectDef {
            id: FieldWorkObjectId::new(self.id.clone()),
            presentation_kind: match self.presentation_kind {
                PveFieldWorkPresentationKind::EmergencyRepairTerminal => {
                    FieldWorkPresentationKind::EmergencyRepairTerminal
                }
            },
            position: self.position.into(),
            work_cells: self
                .work_cells
                .iter()
                .map(|cell| FieldWorkCellDef {
                    id: FieldWorkCellId::new(cell.id.clone()),
                    position: cell.position.into(),
                    tile_kind: cell.tile_kind,
                })
                .collect(),
            required_work_ms: self.required_work_ms,
            completion_effect: match &self.completion_effect {
                PveFieldWorkCompletionEffectData::RepairProtectedUnit { unit_ref, flat_hp } => {
                    FieldWorkCompletionEffect::RepairProtectedUnit {
                        unit_ref: ScenarioUnitRef::new(unit_ref.clone()),
                        flat_hp: *flat_hp,
                    }
                }
            },
        }
    }
}

impl PveWaveData {
    pub fn manual_enemies(&self) -> &[PveWaveEnemyData] {
        match &self.source {
            PveWaveSource::Manual(enemies) => enemies,
            PveWaveSource::GeneratedCorroded { .. } => &[],
        }
    }

    pub fn authored_time_ms(&self) -> Option<u64> {
        self.activation.at_time_ms()
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

impl PveTerminalOutcomeData {
    fn to_terminal_outcome(&self) -> TerminalOutcome {
        TerminalOutcome {
            id: BattleOutcomeId::new(self.id.clone()),
            verdict: self.verdict,
            condition: match &self.condition {
                PveTerminalConditionData::AllRequiredEnemyGroupsDefeated => {
                    TerminalCondition::AllRequiredEnemyGroupsDefeated
                }
                PveTerminalConditionData::UnitDefeated { unit_ref } => {
                    TerminalCondition::UnitDefeated {
                        unit_ref: ScenarioUnitRef::new(unit_ref.clone()),
                    }
                }
                PveTerminalConditionData::AllPlayerCombatantsDefeated => {
                    TerminalCondition::AllPlayerCombatantsDefeated
                }
                PveTerminalConditionData::TimeReached { time_ms } => {
                    TerminalCondition::TimeReached { time_ms: *time_ms }
                }
                PveTerminalConditionData::BossSignalLatched { signal_id } => {
                    TerminalCondition::BossSignalLatched {
                        signal_id: signal_id.clone(),
                    }
                }
            },
            priority: self.priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveEncounterRewardSource {
    pub reward_source_id: String,
    pub encounter_id: String,
    pub fragment_id: SkillFragmentId,
    pub first_equipment_ids: Vec<String>,
    pub repeat_rewards: Vec<RewardEffect>,
    pub revival: Option<crate::game::revival::BossRevivalPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PveEncounterDatabase {
    pub encounters: Vec<PveEncounter>,
    pub reward_sources: Vec<PveEncounterRewardSource>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl PveEncounterDatabase {
    // Only assemble here: final cross-catalog validation must precede indexes.
    pub(crate) fn from_raw_parts(
        encounters: Vec<PveEncounter>,
        reward_sources: Vec<PveEncounterRewardSource>,
    ) -> Self {
        Self {
            encounters,
            reward_sources,
            by_id: OnceLock::new(),
        }
    }

    pub fn new(encounters: Vec<PveEncounter>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter id",
            |encounter| &encounter.id,
        ));
        Self {
            encounters,
            reward_sources: Vec::new(),
            by_id,
        }
    }

    pub fn new_with_reward_sources(
        encounters: Vec<PveEncounter>,
        reward_sources: Vec<PveEncounterRewardSource>,
    ) -> Self {
        let mut database = Self::new(encounters);
        database.reward_sources = reward_sources;
        database
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
        self.validate_reward_source_contract()?;
        Ok(())
    }

    fn validate_reward_source_contract(
        &self,
    ) -> Result<(), crate::game::data::DataValidationError> {
        let mut reward_source_ids = HashSet::new();
        let mut encounter_ids = HashSet::new();
        let mut fragment_ids = HashSet::new();

        for (source_index, source) in self.reward_sources.iter().enumerate() {
            let source_path = format!("reward_sources[{source_index}]");
            if source.reward_source_id.trim().is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    "pve_reward_source",
                    format!("{source_path}.reward_source_id"),
                    "reward source id must not be empty",
                ));
            }
            if !reward_source_ids.insert(source.reward_source_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &source.reward_source_id,
                    format!("{source_path}.reward_source_id"),
                    "duplicate reward source id",
                ));
            }
            if !encounter_ids.insert(source.encounter_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &source.reward_source_id,
                    format!("{source_path}.encounter_id"),
                    "an encounter may author exactly one reward source",
                ));
            }
            if !fragment_ids.insert(source.fragment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &source.reward_source_id,
                    format!("{source_path}.fragment_id"),
                    "a fragment may belong to exactly one reward source",
                ));
            }

            let encounter = self
                .encounters
                .iter()
                .find(|encounter| encounter.id == source.encounter_id)
                .ok_or_else(|| {
                    crate::game::data::DataValidationError::at(
                        &source.reward_source_id,
                        format!("{source_path}.encounter_id"),
                        format!("references missing PVE encounter '{}'", source.encounter_id),
                    )
                })?;
            if matches!(encounter.encounter_class, PveEncounterClass::Normal | PveEncounterClass::StressCombat | PveEncounterClass::ReferenceFinalBoss) {
                return Err(crate::game::data::DataValidationError::at(&source.reward_source_id, format!("{source_path}.encounter_id"), "ordinary/reference encounters cannot author abnormality reward sources"));
            }
            if encounter.encounter_class != PveEncounterClass::NormalBoss && source.revival.is_some() {
                return Err(crate::game::data::DataValidationError::at(&source.reward_source_id, format!("{source_path}.revival"), "only normal bosses can leave revival eggs"));
            }
            if encounter.primary_abnormality_id().is_none() {
                return Err(crate::game::data::DataValidationError::at(
                    &source.reward_source_id,
                    format!("{source_path}.encounter_id"),
                    "reward source encounter must author a primary abnormality",
                ));
            }
            if source.repeat_rewards.iter().any(|effect| !matches!(effect,
                RewardEffect::GrantEnkephalin { .. } | RewardEffect::GrantEquipmentDust { .. }
                | RewardEffect::GrantExperience { .. } | RewardEffect::GrantEquipmentMaterial { .. }
                | RewardEffect::GrantEquipment { .. } | RewardEffect::GrantRelic { .. })) {
                return Err(crate::game::data::DataValidationError::at(&source.reward_source_id, format!("{source_path}.repeat_rewards"), "repeat rewards cannot regrant fragments or use legacy fragment dust"));
            }

        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&PveEncounter> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_reward_source_by_encounter_id(
        &self,
        encounter_id: &str,
    ) -> Option<&PveEncounterRewardSource> {
        self.reward_sources
            .iter()
            .find(|source| source.encounter_id == encounter_id)
    }

    pub fn get_by_risk_level(&self, level: RiskLevel) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.risk_level == level)
            .collect()
    }

    pub(crate) fn validate_boss_runtime_references(
        &self,
        boss_runtime: &BossRuntimeDatabase,
    ) -> Result<(), crate::game::data::DataValidationError> {
        for (encounter_index, encounter) in self.encounters.iter().enumerate() {
            let Some(binding) = &encounter.boss_runtime else {
                continue;
            };
            let binding_path = format!("encounters[{encounter_index}].boss_runtime");
            let definition = boss_runtime
                .get_encounter(&binding.encounter_id)
                .ok_or_else(|| {
                    pve_error(
                        encounter,
                        format!("{binding_path}.encounter_id"),
                        format!(
                            "references missing Boss encounter definition '{}'",
                            binding.encounter_id.as_str()
                        ),
                    )
                })?;
            for (outcome_index, outcome) in encounter.terminal_outcomes.iter().enumerate() {
                let PveTerminalConditionData::BossSignalLatched { signal_id } = &outcome.condition
                else {
                    continue;
                };
                if !boss_runtime.declares_signal(signal_id) {
                    return Err(pve_error(
                        encounter,
                        format!(
                            "encounters[{encounter_index}].terminal_outcomes[{outcome_index}].condition.signal_id"
                        ),
                        format!("references undeclared Boss signal '{}'", signal_id.as_str()),
                    ));
                }
            }
            if binding.actors.is_empty() {
                return Err(pve_error(
                    encounter,
                    format!("{binding_path}.actors"),
                    "Boss runtime binding must bind every actor slot",
                ));
            }
            let expected_slots = definition
                .actor_slots
                .iter()
                .cloned()
                .collect::<HashSet<_>>();
            let mut actual_slots = HashSet::new();
            let mut spawn_bindings = HashSet::new();
            for (actor_index, actor) in binding.actors.iter().enumerate() {
                let actor_path = format!("{binding_path}.actors[{actor_index}]");
                if !actual_slots.insert(actor.actor_slot_id.clone()) {
                    return Err(pve_error(
                        encounter,
                        format!("{actor_path}.actor_slot_id"),
                        format!(
                            "duplicate Boss actor slot binding '{}'",
                            actor.actor_slot_id.as_str()
                        ),
                    ));
                }
                if !expected_slots.contains(&actor.actor_slot_id) {
                    return Err(pve_error(
                        encounter,
                        format!("{actor_path}.actor_slot_id"),
                        format!(
                            "references undeclared Boss actor slot '{}'",
                            actor.actor_slot_id.as_str()
                        ),
                    ));
                }
                if actor.wave_id.trim().is_empty() {
                    return Err(pve_error(
                        encounter,
                        format!("{actor_path}.wave_id"),
                        "Boss actor wave_id must not be empty",
                    ));
                }
                let wave = encounter
                    .waves
                    .iter()
                    .find(|wave| wave.id == actor.wave_id)
                    .ok_or_else(|| {
                        pve_error(
                            encounter,
                            format!("{actor_path}.wave_id"),
                            format!("references missing wave '{}'", actor.wave_id),
                        )
                    })?;
                if !spawn_bindings.insert((actor.wave_id.as_str(), actor.spawn_index)) {
                    return Err(pve_error(
                        encounter,
                        actor_path,
                        "multiple Boss actor slots bind the same wave spawn",
                    ));
                }
                if let PveWaveSource::Manual(enemies) = &wave.source {
                    let authored_count = enemies.iter().fold(0_u64, |total, enemy| {
                        total.saturating_add(enemy.count() as u64)
                    });
                    if u64::from(actor.spawn_index) >= authored_count {
                        return Err(pve_error(
                            encounter,
                            format!("{actor_path}.spawn_index"),
                            format!(
                                "spawn_index {} is outside manual wave '{}' count {}",
                                actor.spawn_index, actor.wave_id, authored_count
                            ),
                        ));
                    }
                }
            }
            if actual_slots != expected_slots {
                return Err(pve_error(
                    encounter,
                    format!("{binding_path}.actors"),
                    "Boss runtime binding must bind every actor slot exactly once",
                ));
            }
        }
        Ok(())
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
    let mut bonus_ids = HashSet::new();
    for objective in &encounter.bonus_objectives {
        let invalid = || pve_error(encounter, format!("{encounter_path}.bonus_objectives"),
            "bonus requires unique identity, primary abnormality, valid threshold, and material/AliveRoster XP effects");
        let Some(primary) = encounter.primary_abnormality_id.as_deref() else { return Err(invalid()); };
        if objective.id.trim().is_empty() || !bonus_ids.insert(&objective.id) || objective.effects.is_empty() {
            return Err(invalid());
        }
        match objective.condition {
            PveBonusCondition::ClearWithin { time_ms } => {
                if time_ms == 0 || !encounter.waves.iter().any(|wave| wave.authored_time_ms() == Some(0)
                    && wave.manual_enemies().iter().any(|enemy| matches!(enemy,
                        PveWaveEnemyData::Abnormality { abnormality_id, .. } if abnormality_id == primary))) {
                    return Err(invalid());
                }
            }
            PveBonusCondition::DecisiveDamage { max_health_percent } => {
                if !(1..=100).contains(&max_health_percent) { return Err(invalid()); }
            }
        }
        for effect in &objective.effects {
            let amount = match effect {
                RewardEffect::GrantEnkephalin { amount } | RewardEffect::GrantEquipmentDust { amount }
                | RewardEffect::GrantEquipmentMaterial { amount, .. } => *amount,
                RewardEffect::GrantExperience { amount, target: crate::game::reward::ExperienceTargetPolicy::AliveRoster } => *amount,
                _ => return Err(invalid()),
            };
            if amount == 0 { return Err(invalid()); }
        }
    }
    let tuning = encounter.deterrence_tuning;
    if !(150..=200).contains(&tuning.unrestrained_health_percent)
        || !(150..=200).contains(&tuning.unrestrained_attack_percent)
        || tuning.seal_health_percent <= 100
        || tuning.seal_attack_percent <= 100 {
        return Err(pve_error(encounter, format!("{encounter_path}.deterrence_tuning"),
            "unrestrained HP/ATK must be 150..=200%; seal HP/ATK must exceed 100%"));
    }
    if encounter.waves.is_empty() {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.waves"),
            "encounter must define at least one wave",
        ));
    }

    match encounter.encounter_class {
        PveEncounterClass::Normal | PveEncounterClass::StressCombat => {
            if encounter.primary_abnormality_id.is_some() {
                return Err(pve_error(
                    encounter,
                    format!("{encounter_path}.primary_abnormality_id"),
                    "normal encounter must not define primary_abnormality_id",
                ));
            }
        }
        PveEncounterClass::Elite
        | PveEncounterClass::NormalBoss
        | PveEncounterClass::FinalBoss
        | PveEncounterClass::ReferenceFinalBoss => {
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

    if encounter.is_reference_only() {
        for (policy_index, policy) in encounter.outcome_policies.iter().enumerate() {
            if !policy.reward_uuids.is_empty()
                || policy.grant_node_bonus
                || policy.advance_encounter_progression
            {
                return Err(pve_error(
                    encounter,
                    format!("{encounter_path}.outcome_policies[{policy_index}]"),
                    "reference-only outcomes cannot grant rewards, node bonuses, or encounter progression",
                ));
            }
        }
    }

    if encounter.is_rewardless_story_encounter()
        && (!encounter.bonus_objectives.is_empty() || encounter.outcome_policies.iter().any(|policy|
            !policy.reward_uuids.is_empty() || policy.grant_node_bonus || policy.advance_encounter_progression)) {
        return Err(pve_error(encounter, format!("{encounter_path}.outcome_policies"),
            "Der Freischutz story encounters cannot grant ordinary rewards or encounter progression"));
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
    validate_wave_activation_graph(encounter, encounter_path)?;
    validate_terminal_outcome_contract(encounter, encounter_path)?;

    let Some(tactical_plan) = &encounter.tactical_plan else {
        return Ok(());
    };

    validate_field_work_authoring(encounter, tactical_plan, encounter_path)?;

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

    if let Some(PveBattleObjectiveData::ProtectUnit { unit_ref }) = &tactical_plan.objective {
        let has_matching_defeat = encounter.terminal_outcomes.iter().any(|outcome| {
            outcome.verdict == BattleVerdict::Defeat
                && matches!(
                    &outcome.condition,
                    PveTerminalConditionData::UnitDefeated { unit_ref: terminal_ref }
                        if terminal_ref == unit_ref
                )
        });
        if !has_matching_defeat {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.tactical_plan.objective"),
                "protect objective requires a matching UnitDefeated Defeat outcome",
            ));
        }
    }
    Ok(())
}

fn validate_terminal_outcome_contract(
    encounter: &PveEncounter,
    encounter_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if encounter.terminal_outcomes.is_empty() {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.terminal_outcomes"),
            "encounter must declare at least one terminal outcome",
        ));
    }

    let mut outcome_ids = HashSet::new();
    let mut has_victory = false;
    let mut has_time_victory = false;
    for (index, outcome) in encounter.terminal_outcomes.iter().enumerate() {
        let outcome_path = format!("{encounter_path}.terminal_outcomes[{index}]");
        if outcome.id.trim().is_empty() || !outcome_ids.insert(outcome.id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{outcome_path}.id"),
                "terminal outcome id must be nonempty and unique",
            ));
        }
        has_victory |= outcome.verdict == BattleVerdict::Victory;
        match &outcome.condition {
            PveTerminalConditionData::AllRequiredEnemyGroupsDefeated
            | PveTerminalConditionData::AllPlayerCombatantsDefeated => {}
            PveTerminalConditionData::UnitDefeated { unit_ref } => {
                if unit_ref.trim().is_empty() {
                    return Err(pve_error(
                        encounter,
                        format!("{outcome_path}.condition.unit_ref"),
                        "terminal unit_ref must not be empty",
                    ));
                }
            }
            PveTerminalConditionData::TimeReached { time_ms } => {
                if *time_ms == 0 || *time_ms > u32::MAX as u64 {
                    return Err(pve_error(
                        encounter,
                        format!("{outcome_path}.condition.time_ms"),
                        format!("terminal time_ms must be within 1..={}", u32::MAX),
                    ));
                }
                has_time_victory |= outcome.verdict == BattleVerdict::Victory;
            }
            PveTerminalConditionData::BossSignalLatched { signal_id } => {
                if signal_id.as_str().trim().is_empty() || encounter.boss_runtime.is_none() {
                    return Err(pve_error(
                        encounter,
                        format!("{outcome_path}.condition.signal_id"),
                        "BossSignalLatched requires a nonempty signal and Boss Runtime binding",
                    ));
                }
            }
        }
    }
    if !has_victory {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.terminal_outcomes"),
            "encounter must declare at least one reachable Victory outcome",
        ));
    }
    if has_time_victory && encounter.waves.iter().any(|wave| wave.required_for_victory) {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.waves"),
            "time-Victory encounter waves must all set required_for_victory to false",
        ));
    }

    let mut policy_ids = HashSet::new();
    for (index, policy) in encounter.outcome_policies.iter().enumerate() {
        let policy_path = format!("{encounter_path}.outcome_policies[{index}]");
        if policy.outcome_id.trim().is_empty()
            || !policy_ids.insert(policy.outcome_id.as_str())
            || !outcome_ids.contains(policy.outcome_id.as_str())
        {
            return Err(pve_error(
                encounter,
                format!("{policy_path}.outcome_id"),
                "outcome policy must reference one unique declared outcome",
            ));
        }
        let mut rewards = HashSet::new();
        if policy
            .reward_uuids
            .iter()
            .any(|reward| !rewards.insert(*reward))
        {
            return Err(pve_error(
                encounter,
                format!("{policy_path}.reward_uuids"),
                "outcome policy contains duplicate reward UUIDs",
            ));
        }
        let verdict = encounter
            .terminal_outcomes
            .iter()
            .find(|outcome| outcome.id == policy.outcome_id)
            .map(|outcome| outcome.verdict)
            .expect("declared outcome existence was checked");
        if verdict == BattleVerdict::Defeat
            && (!policy.reward_uuids.is_empty()
                || policy.grant_node_bonus
                || policy.advance_encounter_progression)
        {
            return Err(pve_error(
                encounter,
                policy_path,
                "Defeat outcome policy cannot grant rewards, node bonus, or progression",
            ));
        }
    }
    if policy_ids != outcome_ids {
        return Err(pve_error(
            encounter,
            format!("{encounter_path}.outcome_policies"),
            "outcome policies must cover every terminal outcome exactly once",
        ));
    }
    Ok(())
}

fn validate_wave_activation_graph(
    encounter: &PveEncounter,
    encounter_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    let wave_indices = encounter
        .waves
        .iter()
        .enumerate()
        .map(|(index, wave)| (wave.id.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut trigger_ids = HashSet::new();
    let mut dependencies = vec![Vec::<usize>::new(); encounter.waves.len()];
    let actor_sources = encounter
        .boss_runtime
        .as_ref()
        .map(|binding| {
            binding
                .actors
                .iter()
                .map(|actor| (actor.actor_slot_id.clone(), actor.wave_id.as_str()))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let point_ids = encounter
        .tactical_plan
        .as_ref()
        .map(|plan| {
            plan.points
                .iter()
                .map(|point| point.id.as_str())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    for (wave_index, wave) in encounter.waves.iter().enumerate() {
        let activation_path = format!("{encounter_path}.waves[{wave_index}].activation");
        if !trigger_ids.insert(wave.activation.trigger_id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{activation_path}.trigger_id"),
                format!("duplicate wave trigger_id '{}'", wave.activation.trigger_id),
            ));
        }

        let dependency_wave_id = match &wave.activation.primary {
            PveWavePrimaryTriggerData::AtTimeMs { .. } => None,
            PveWavePrimaryTriggerData::GroupCleared { wave_id } => Some(wave_id.as_str()),
            PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent { actor_slot_id, .. }
            | PveWavePrimaryTriggerData::ActorDefeated { actor_slot_id }
            | PveWavePrimaryTriggerData::ActorReachedPoint { actor_slot_id, .. } => {
                let Some(source_wave_id) = actor_sources.get(actor_slot_id) else {
                    return Err(pve_error(
                        encounter,
                        format!("{activation_path}.primary.actor_slot_id"),
                        format!(
                            "references unbound Boss actor slot '{}'",
                            actor_slot_id.as_str()
                        ),
                    ));
                };
                Some(*source_wave_id)
            }
        };

        if let PveWavePrimaryTriggerData::ActorReachedPoint { point_id, .. } =
            &wave.activation.primary
        {
            if !point_ids.contains(point_id.as_str()) {
                return Err(pve_error(
                    encounter,
                    format!("{activation_path}.primary.point_id"),
                    format!("references unknown tactical point '{point_id}'"),
                ));
            }
        }

        if let Some(source_wave_id) = dependency_wave_id {
            let Some(&source_index) = wave_indices.get(source_wave_id) else {
                return Err(pve_error(
                    encounter,
                    format!("{activation_path}.primary"),
                    format!("references missing source wave '{source_wave_id}'"),
                ));
            };
            if source_index == wave_index {
                return Err(pve_error(
                    encounter,
                    format!("{activation_path}.primary"),
                    "wave activation cannot depend on its own wave",
                ));
            }
            if matches!(
                wave.activation.primary,
                PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent { .. }
                    | PveWavePrimaryTriggerData::ActorDefeated { .. }
                    | PveWavePrimaryTriggerData::ActorReachedPoint { .. }
            ) && matches!(
                encounter.waves[source_index].source,
                PveWaveSource::GeneratedCorroded { .. }
            ) {
                return Err(pve_error(
                    encounter,
                    format!("{activation_path}.primary.actor_slot_id"),
                    format!(
                        "actor trigger source wave '{source_wave_id}' is generated; generated waves support group observation only"
                    ),
                ));
            }
            dependencies[wave_index].push(source_index);

            if let Some(deadline_ms) = wave.activation.deadline_ms {
                let source_time_ms = encounter.waves[source_index]
                    .authored_time_ms()
                    .or(encounter.waves[source_index].activation.deadline_ms);
                if let Some(source_time_ms) = source_time_ms {
                    if deadline_ms <= source_time_ms {
                        return Err(pve_error(
                            encounter,
                            format!("{activation_path}.deadline_ms"),
                            format!(
                                "deadline_ms {deadline_ms} must be later than source wave '{source_wave_id}' time {source_time_ms}"
                            ),
                        ));
                    }
                }
            }
        }
    }

    fn visit(index: usize, dependencies: &[Vec<usize>], marks: &mut [u8]) -> Result<(), ()> {
        if marks[index] == 1 {
            return Err(());
        }
        if marks[index] == 2 {
            return Ok(());
        }
        marks[index] = 1;
        for dependency in &dependencies[index] {
            visit(*dependency, dependencies, marks)?;
        }
        marks[index] = 2;
        Ok(())
    }

    let mut marks = vec![0_u8; encounter.waves.len()];
    for index in 0..encounter.waves.len() {
        if visit(index, &dependencies, &mut marks).is_err() {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves"),
                "wave activation dependency graph contains a cycle",
            ));
        }
    }

    fn may_cancel(
        index: usize,
        encounter: &PveEncounter,
        dependencies: &[Vec<usize>],
        memo: &mut [Option<bool>],
    ) -> bool {
        if let Some(value) = memo[index] {
            return value;
        }
        let wave = &encounter.waves[index];
        let value = if wave.required_for_victory || wave.activation.deadline_ms.is_some() {
            false
        } else {
            match wave.activation.primary {
                PveWavePrimaryTriggerData::AtTimeMs { .. } => false,
                PveWavePrimaryTriggerData::GroupCleared { .. } => dependencies[index]
                    .first()
                    .is_some_and(|source| may_cancel(*source, encounter, dependencies, memo)),
                PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent { .. }
                | PveWavePrimaryTriggerData::ActorDefeated { .. }
                | PveWavePrimaryTriggerData::ActorReachedPoint { .. } => true,
            }
        };
        memo[index] = Some(value);
        value
    }

    let mut cancellation_memo = vec![None; encounter.waves.len()];
    for (index, wave) in encounter.waves.iter().enumerate() {
        if !wave.required_for_victory || wave.activation.deadline_ms.is_some() {
            continue;
        }
        if dependencies[index]
            .iter()
            .any(|source| may_cancel(*source, encounter, &dependencies, &mut cancellation_memo))
        {
            return Err(pve_error(
                encounter,
                format!("{encounter_path}.waves[{index}].activation"),
                "required wave depends on a cancelable source and must define a deadline_ms",
            ));
        }
    }

    Ok(())
}

fn validate_field_work_authoring(
    encounter: &PveEncounter,
    tactical_plan: &PveTacticalPlanData,
    encounter_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    let mut object_ids = HashSet::new();
    let mut work_cell_positions = HashSet::new();
    for (object_index, object) in tactical_plan.field_work_objects.iter().enumerate() {
        let object_path =
            format!("{encounter_path}.tactical_plan.field_work_objects[{object_index}]");
        if object.id.trim().is_empty() {
            return Err(pve_error(
                encounter,
                format!("{object_path}.id"),
                "field work object id must not be empty",
            ));
        }
        if !object_ids.insert(object.id.as_str()) {
            return Err(pve_error(
                encounter,
                format!("{object_path}.id"),
                format!("duplicate field work object id '{}'", object.id),
            ));
        }
        if object.required_work_ms == 0 {
            return Err(pve_error(
                encounter,
                format!("{object_path}.required_work_ms"),
                "required_work_ms must be greater than zero",
            ));
        }
        if object.work_cells.is_empty() {
            return Err(pve_error(
                encounter,
                format!("{object_path}.work_cells"),
                "field work object must define at least one work cell",
            ));
        }

        let mut cell_ids = HashSet::new();
        for (cell_index, cell) in object.work_cells.iter().enumerate() {
            let cell_path = format!("{object_path}.work_cells[{cell_index}]");
            if cell.id.trim().is_empty() {
                return Err(pve_error(
                    encounter,
                    format!("{cell_path}.id"),
                    "field work cell id must not be empty",
                ));
            }
            if !cell_ids.insert(cell.id.as_str()) {
                return Err(pve_error(
                    encounter,
                    format!("{cell_path}.id"),
                    format!("duplicate field work cell id '{}'", cell.id),
                ));
            }
            if !work_cell_positions.insert(cell.position) {
                return Err(pve_error(
                    encounter,
                    format!("{cell_path}.position"),
                    format!(
                        "duplicate field work cell position ({}, {})",
                        cell.position.x, cell.position.y
                    ),
                ));
            }
        }

        match &object.completion_effect {
            PveFieldWorkCompletionEffectData::RepairProtectedUnit { unit_ref, flat_hp } => {
                if unit_ref.trim().is_empty() {
                    return Err(pve_error(
                        encounter,
                        format!("{object_path}.completion_effect.RepairProtectedUnit.unit_ref"),
                        "repair target unit_ref must not be empty",
                    ));
                }
                if *flat_hp == 0 {
                    return Err(pve_error(
                        encounter,
                        format!("{object_path}.completion_effect.RepairProtectedUnit.flat_hp"),
                        "repair flat_hp must be greater than zero",
                    ));
                }
                let protects_same_unit = matches!(
                    &tactical_plan.objective,
                    Some(PveBattleObjectiveData::ProtectUnit { unit_ref: objective_ref })
                        if objective_ref == unit_ref
                ) && encounter.terminal_outcomes.iter().any(|outcome| {
                    outcome.verdict == BattleVerdict::Defeat
                        && matches!(
                            &outcome.condition,
                            PveTerminalConditionData::UnitDefeated { unit_ref: terminal_ref }
                                if terminal_ref == unit_ref
                        )
                });
                if !protects_same_unit {
                    return Err(pve_error(
                        encounter,
                        format!("{object_path}.completion_effect.RepairProtectedUnit.unit_ref"),
                        "repair target must match the authored protect objective and win condition",
                    ));
                }
            }
        }
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
    if wave.activation.trigger_id.trim().is_empty() {
        return Err("wave activation trigger_id must not be empty".to_string());
    }
    const MAX_AUTHORED_TIME_MS: u64 = u32::MAX as u64;
    if let Some(deadline_ms) = wave.activation.deadline_ms {
        if deadline_ms == 0 || deadline_ms > MAX_AUTHORED_TIME_MS {
            return Err(format!(
                "wave activation deadline_ms must be within 1..={MAX_AUTHORED_TIME_MS}"
            ));
        }
    }
    match &wave.activation.primary {
        PveWavePrimaryTriggerData::AtTimeMs { time_ms } => {
            if *time_ms > MAX_AUTHORED_TIME_MS {
                return Err(format!(
                    "AtTimeMs.time_ms must be within 0..={MAX_AUTHORED_TIME_MS}"
                ));
            }
            if wave.activation.deadline_ms.is_some() {
                return Err("AtTimeMs must not define a separate deadline_ms".to_string());
            }
        }
        PveWavePrimaryTriggerData::GroupCleared { wave_id } => {
            if wave_id.trim().is_empty() {
                return Err("GroupCleared.wave_id must not be empty".to_string());
            }
        }
        PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent { percent, .. } => {
            if !(1..=100).contains(percent) {
                return Err(
                    "ActorHealthAtOrBelowPercent.percent must be within 1..=100".to_string()
                );
            }
        }
        PveWavePrimaryTriggerData::ActorDefeated { .. } => {}
        PveWavePrimaryTriggerData::ActorReachedPoint {
            point_id,
            radius_cells,
            ..
        } => {
            if point_id.trim().is_empty() {
                return Err("ActorReachedPoint.point_id must not be empty".to_string());
            }
            if *radius_cells == 0 {
                return Err("ActorReachedPoint.radius_cells must be greater than zero".to_string());
            }
            if wave.required_for_victory && wave.activation.deadline_ms.is_none() {
                return Err("required ActorReachedPoint wave must define deadline_ms".to_string());
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_wave(
        id: &str,
        primary: PveWavePrimaryTriggerData,
        deadline_ms: Option<u64>,
        required_for_victory: bool,
    ) -> PveWaveData {
        PveWaveData {
            id: id.to_string(),
            activation: PveWaveActivationData {
                trigger_id: format!("trigger_{id}"),
                priority: 0,
                primary,
                deadline_ms,
            },
            spawn_zone_ids: Vec::new(),
            route_id: None,
            required_for_victory,
            source: PveWaveSource::GeneratedCorroded {
                preset_id: "fixture".to_string(),
                budget_override: None,
                seed_salt: None,
            },
        }
    }

    fn graph_encounter(waves: Vec<PveWaveData>) -> PveEncounter {
        PveEncounter {
            deterrence_tuning: Default::default(),
            bonus_objectives: Vec::new(),
            id: "wave_graph".to_string(),
            encounter_class: PveEncounterClass::Normal,
            primary_abnormality_id: None,
            boss_runtime: None,
            risk_level: RiskLevel::ZAYIN,
            node_type: None,
            mission_variant: None,
            terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
            outcome_policies: PveEncounter::fixture_outcome_policies(
                RewardMode::ClaimAll,
                Vec::new(),
            ),
            battlefield: None,
            tactical_plan: None,
            waves,
            static_obstacles: Vec::new(),
        }
    }

    #[test]
    fn pve_database_requires_explicit_reward_source_collection() {
        let error =
            ron::de::from_str::<PveEncounterDatabase>(r#"PveEncounterDatabase(encounters: [])"#)
                .expect_err("reward_sources must be explicit in authored PVE RON");

        assert!(error.to_string().contains("reward_sources"));
    }

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
    fn wave_activation_and_victory_gate_have_no_deserialization_defaults() {
        let missing_activation = ron::de::from_str::<PveWaveData>(
            r#"(
                id: "wave",
                required_for_victory: true,
                source: GeneratedCorroded(preset_id: "fixture"),
            )"#,
        )
        .expect_err("activation must be explicit");
        assert!(missing_activation.to_string().contains("activation"));

        let missing_victory_gate = ron::de::from_str::<PveWaveData>(
            r#"(
                id: "wave",
                activation: (
                    trigger_id: "wave",
                    priority: 0,
                    primary: AtTimeMs(time_ms: 0),
                ),
                source: GeneratedCorroded(preset_id: "fixture"),
            )"#,
        )
        .expect_err("required_for_victory must be explicit");
        assert!(missing_victory_gate
            .to_string()
            .contains("required_for_victory"));
    }

    #[test]
    fn wave_activation_graph_rejects_missing_sources_cycles_and_reversed_deadlines() {
        let missing = graph_encounter(vec![graph_wave(
            "target",
            PveWavePrimaryTriggerData::GroupCleared {
                wave_id: "missing".to_string(),
            },
            None,
            true,
        )]);
        let error = PveEncounterDatabase::new(vec![missing])
            .validate_contract()
            .expect_err("missing source must reject");
        assert!(error.cause.contains("missing source wave"));

        let cycle = graph_encounter(vec![
            graph_wave(
                "a",
                PveWavePrimaryTriggerData::GroupCleared {
                    wave_id: "b".to_string(),
                },
                None,
                true,
            ),
            graph_wave(
                "b",
                PveWavePrimaryTriggerData::GroupCleared {
                    wave_id: "a".to_string(),
                },
                None,
                true,
            ),
        ]);
        let error = PveEncounterDatabase::new(vec![cycle])
            .validate_contract()
            .expect_err("cycle must reject");
        assert!(error.cause.contains("cycle"));

        let reversed = graph_encounter(vec![
            graph_wave(
                "source",
                PveWavePrimaryTriggerData::AtTimeMs { time_ms: 10 },
                None,
                true,
            ),
            graph_wave(
                "target",
                PveWavePrimaryTriggerData::GroupCleared {
                    wave_id: "source".to_string(),
                },
                Some(10),
                true,
            ),
        ]);
        let error = PveEncounterDatabase::new(vec![reversed])
            .validate_contract()
            .expect_err("deadline at or before its source must reject");
        assert!(error.cause.contains("must be later"));
    }

    #[test]
    fn wave_activation_rejects_invalid_typed_payloads_and_generated_actor_sources() {
        let at_time_with_deadline = graph_wave(
            "time",
            PveWavePrimaryTriggerData::AtTimeMs { time_ms: 10 },
            Some(20),
            true,
        );
        let error = validate_pve_wave_authoring_contract(&at_time_with_deadline, false)
            .expect_err("AtTimeMs cannot have a second deadline");
        assert!(error.contains("must not define"));

        let invalid_hp = graph_wave(
            "hp",
            PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent {
                actor_slot_id: BossActorSlotId::new("primary"),
                percent: 0,
            },
            None,
            false,
        );
        let error = validate_pve_wave_authoring_contract(&invalid_hp, false)
            .expect_err("zero percent must reject");
        assert!(error.contains("1..=100"));

        let generated_actor_source = graph_wave(
            "generated_actor",
            PveWavePrimaryTriggerData::AtTimeMs { time_ms: 0 },
            None,
            true,
        );
        let actor_observer = graph_wave(
            "observer",
            PveWavePrimaryTriggerData::ActorDefeated {
                actor_slot_id: BossActorSlotId::new("primary"),
            },
            Some(100),
            true,
        );
        let mut encounter = graph_encounter(vec![generated_actor_source, actor_observer]);
        encounter.boss_runtime = Some(PveBossRuntimeBindingData {
            encounter_id: BossEncounterRuntimeId::new("fixture"),
            actors: vec![PveBossActorBindingData {
                actor_slot_id: BossActorSlotId::new("primary"),
                wave_id: "generated_actor".to_string(),
                spawn_index: 0,
            }],
        });
        let error = validate_wave_activation_graph(&encounter, "encounters[0]")
            .expect_err("generated Wave actor observation must reject");
        assert!(error.cause.contains("group observation only"));
    }

    #[test]
    fn required_wave_cannot_depend_indefinitely_on_a_cancelable_actor_wave() {
        let mut actor_source = graph_wave(
            "actor_source",
            PveWavePrimaryTriggerData::AtTimeMs { time_ms: 0 },
            None,
            true,
        );
        actor_source.source = PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
            abnormality_id: "enemy".to_string(),
            tier: Tier::I,
            count: 1,
        }]);
        let mut actor_wave = graph_wave(
            "actor_wave",
            PveWavePrimaryTriggerData::ActorDefeated {
                actor_slot_id: BossActorSlotId::new("primary"),
            },
            None,
            false,
        );
        actor_wave.source = PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
            profile_id: "guard".to_string(),
            tier: Tier::I,
            count: 1,
        }]);
        let mut required = graph_wave(
            "required",
            PveWavePrimaryTriggerData::GroupCleared {
                wave_id: "actor_wave".to_string(),
            },
            None,
            true,
        );
        required.source = PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
            profile_id: "guard".to_string(),
            tier: Tier::I,
            count: 1,
        }]);
        let mut encounter = graph_encounter(vec![actor_source, actor_wave, required]);
        encounter.encounter_class = PveEncounterClass::Elite;
        encounter.primary_abnormality_id = Some("enemy".to_string());
        encounter.boss_runtime = Some(PveBossRuntimeBindingData {
            encounter_id: BossEncounterRuntimeId::new("fixture"),
            actors: vec![PveBossActorBindingData {
                actor_slot_id: BossActorSlotId::new("primary"),
                wave_id: "actor_source".to_string(),
                spawn_index: 0,
            }],
        });

        let error = PveEncounterDatabase::new(vec![encounter])
            .validate_contract()
            .expect_err("required downstream of cancelable actor source must reject");
        assert!(error.cause.contains("cancelable source"));
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
                        activation: (
                            trigger_id: "wave_0",
                            priority: 0,
                            primary: AtTimeMs(time_ms: 0),
                        ),
                        spawn_zone_ids: ["north_entry"],
                        required_for_victory: true,
                        source: Manual([
                            CorrodedEmployee(profile_id: "enemy_a", tier: I, count: 2),
                            Abnormality(abnormality_id: "enemy_b", tier: II, count: 1),
                        ]),
                    ),
                    (
                        id: "wave_1",
                        activation: (
                            trigger_id: "wave_1",
                            priority: 0,
                            primary: AtTimeMs(time_ms: 15000),
                        ),
                        required_for_victory: true,
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
                terminal_outcomes: [
                    (id: "victory", verdict: victory, condition: TimeReached(time_ms: 45000), priority: 0),
                    (id: "defeat", verdict: defeat, condition: UnitDefeated(unit_ref: "black_box"), priority: 0),
                ],
                outcome_policies: [
                    (outcome_id: "victory", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: true, advance_encounter_progression: true),
                    (outcome_id: "defeat", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: false, advance_encounter_progression: false),
                ],
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
                waves: [
                    (
                        id: "wave_0",
                        activation: (
                            trigger_id: "wave_0",
                            priority: 0,
                            primary: AtTimeMs(time_ms: 0),
                        ),
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
        assert_eq!(encounter.survive_timer_ms(), Some(45_000));
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
        assert!(encounter.terminal_outcomes.iter().any(|outcome| matches!(
            &outcome.condition,
            PveTerminalConditionData::UnitDefeated { unit_ref } if unit_ref == "black_box"
        )));
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
    fn pve_encounter_validation_rejects_protect_objective_without_matching_defeat_outcome() {
        let encounter: PveEncounter = ron::de::from_str(
            r#"(
                id: "conflicting_contract",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                terminal_outcomes: [
                    (id: "victory", verdict: victory, condition: AllRequiredEnemyGroupsDefeated, priority: 0),
                    (id: "defeat", verdict: defeat, condition: AllPlayerCombatantsDefeated, priority: 0),
                ],
                outcome_policies: [
                    (outcome_id: "victory", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: true, advance_encounter_progression: true),
                    (outcome_id: "defeat", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: false, advance_encounter_progression: false),
                ],
                tactical_plan: Some((
                    objective: Some(ProtectUnit(unit_ref: "black_box")),
                )),
                waves: [
                    (
                        id: "wave_0",
                        activation: (
                            trigger_id: "wave_0",
                            priority: 0,
                            primary: AtTimeMs(time_ms: 0),
                        ),
                        required_for_victory: true,
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
        assert!(error.cause.contains("matching UnitDefeated Defeat outcome"));
    }

    #[test]
    fn named_outcome_contract_rejects_empty_duplicate_unreachable_and_positive_defeat() {
        let base = graph_encounter(vec![graph_wave(
            "wave_0",
            PveWavePrimaryTriggerData::AtTimeMs { time_ms: 0 },
            None,
            true,
        )]);

        let mut empty = base.clone();
        empty.terminal_outcomes.clear();
        let error = PveEncounterDatabase::new(vec![empty])
            .validate_contract()
            .expect_err("empty terminal outcome set must reject");
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].terminal_outcomes")
        );

        let mut duplicate = base.clone();
        duplicate.terminal_outcomes[1].id = duplicate.terminal_outcomes[0].id.clone();
        let error = PveEncounterDatabase::new(vec![duplicate])
            .validate_contract()
            .expect_err("duplicate terminal outcome ID must reject");
        assert!(error.cause.contains("nonempty and unique"));

        let mut no_victory = base.clone();
        for outcome in &mut no_victory.terminal_outcomes {
            outcome.verdict = BattleVerdict::Defeat;
        }
        let error = PveEncounterDatabase::new(vec![no_victory])
            .validate_contract()
            .expect_err("terminal outcome set without Victory must reject");
        assert!(error.cause.contains("at least one reachable Victory"));

        let mut non_exhaustive = base.clone();
        non_exhaustive.outcome_policies.pop();
        let error = PveEncounterDatabase::new(vec![non_exhaustive])
            .validate_contract()
            .expect_err("non-exhaustive outcome policy must reject");
        assert!(error.cause.contains("cover every terminal outcome"));

        let mut positive_defeat = base;
        positive_defeat
            .outcome_policies
            .iter_mut()
            .find(|policy| policy.outcome_id == "defeat")
            .expect("Defeat fixture policy")
            .grant_node_bonus = true;
        let error = PveEncounterDatabase::new(vec![positive_defeat])
            .validate_contract()
            .expect_err("positive Defeat outcome policy must reject");
        assert!(error.cause.contains("Defeat outcome policy cannot grant"));
    }

    fn valid_field_work_encounter() -> PveEncounter {
        ron::de::from_str(
            r#"(
                id: "field_work_contract",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                terminal_outcomes: [
                    (id: "victory", verdict: victory, condition: AllRequiredEnemyGroupsDefeated, priority: 0),
                    (id: "defeat", verdict: defeat, condition: UnitDefeated(unit_ref: "black_box"), priority: 0),
                ],
                outcome_policies: [
                    (outcome_id: "victory", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: true, advance_encounter_progression: true),
                    (outcome_id: "defeat", reward_mode: ClaimAll, reward_uuids: [], grant_node_bonus: false, advance_encounter_progression: false),
                ],
                tactical_plan: Some((
                    objective: Some(ProtectUnit(unit_ref: "black_box")),
                    field_work_objects: [(
                        id: "repair",
                        presentation_kind: EmergencyRepairTerminal,
                        position: (x: 2, y: 2),
                        work_cells: [(
                            id: "cell_a",
                            position: (x: 2, y: 2),
                            tile_kind: ground,
                        )],
                        required_work_ms: 1000,
                        completion_effect: RepairProtectedUnit(
                            unit_ref: "black_box",
                            flat_hp: 100,
                        ),
                    )],
                )),
                waves: [(
                    id: "wave_0",
                    activation: (
                        trigger_id: "wave_0",
                        priority: 0,
                        primary: AtTimeMs(time_ms: 0),
                    ),
                    required_for_victory: true,
                    source: Manual([
                        Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                    ]),
                )],
            )"#,
        )
        .expect("valid field work fixture should deserialize")
    }

    #[test]
    fn field_work_authoring_rejects_zero_duplicate_and_wrong_repair_target() {
        let mut zero_work = valid_field_work_encounter();
        zero_work.tactical_plan.as_mut().unwrap().field_work_objects[0].required_work_ms = 0;
        let error = PveEncounterDatabase::new(vec![zero_work])
            .validate_contract()
            .expect_err("zero required work must reject");
        assert!(error.cause.contains("greater than zero"));

        let mut duplicate_position = valid_field_work_encounter();
        let object = &mut duplicate_position
            .tactical_plan
            .as_mut()
            .unwrap()
            .field_work_objects[0];
        object.work_cells.push(PveFieldWorkCellData {
            id: "cell_b".to_string(),
            position: PvePosition { x: 2, y: 2 },
            tile_kind: TacticalTileKind::Ground,
        });
        let error = PveEncounterDatabase::new(vec![duplicate_position])
            .validate_contract()
            .expect_err("duplicate work cell position must reject");
        assert!(error.cause.contains("duplicate field work cell position"));

        let mut wrong_target = valid_field_work_encounter();
        wrong_target
            .tactical_plan
            .as_mut()
            .unwrap()
            .field_work_objects[0]
            .completion_effect = PveFieldWorkCompletionEffectData::RepairProtectedUnit {
            unit_ref: "someone_else".to_string(),
            flat_hp: 100,
        };
        let error = PveEncounterDatabase::new(vec![wrong_target])
            .validate_contract()
            .expect_err("repair target outside protect contract must reject");
        assert!(error.cause.contains("must match the authored protect"));

        let missing_required_work = ron::de::from_str::<PveFieldWorkObjectData>(
            r#"(
                id: "repair",
                presentation_kind: EmergencyRepairTerminal,
                position: (x: 2, y: 2),
                work_cells: [(
                    id: "cell",
                    position: (x: 2, y: 2),
                    tile_kind: ground,
                )],
                completion_effect: RepairProtectedUnit(unit_ref: "black_box", flat_hp: 100),
            )"#,
        );
        assert!(
            missing_required_work.is_err(),
            "required_work_ms has no default compatibility path"
        );

        let reset_presentation = ron::de::from_str::<PveFieldWorkObjectData>(
            r#"(
                id: "repair",
                presentation_kind: Reset,
                position: (x: 2, y: 2),
                work_cells: [(
                    id: "cell",
                    position: (x: 2, y: 2),
                    tile_kind: ground,
                )],
                required_work_ms: 1000,
                completion_effect: RepairProtectedUnit(unit_ref: "black_box", flat_hp: 100),
            )"#,
        );
        assert!(
            reset_presentation.is_err(),
            "the first slice deliberately has no generic Reset presentation"
        );

        let unknown_compatibility_field = ron::de::from_str::<PveFieldWorkObjectData>(
            r#"(
                id: "repair",
                presentation_kind: EmergencyRepairTerminal,
                position: (x: 2, y: 2),
                work_cells: [(
                    id: "cell",
                    position: (x: 2, y: 2),
                    tile_kind: ground,
                )],
                required_work_ms: 1000,
                completion_effect: RepairProtectedUnit(unit_ref: "black_box", flat_hp: 100),
                progress_compatibility_mode: "legacy",
            )"#,
        );
        assert!(
            unknown_compatibility_field.is_err(),
            "unknown field work compatibility fields must fail closed"
        );
    }

    #[test]
    fn removed_suppression_research_field_is_rejected() {
        let parsed = ron::de::from_str::<PveEncounter>(
            r#"(
                id: "removed_research_contract",
                encounter_class: Elite,
                primary_abnormality_id: Some("enemy"),
                risk_level: HE,
                suppression_research: Some((bonus_objectives: [])),
                waves: [(
                    id: "wave_0",
                    activation: (
                        trigger_id: "wave_0",
                        priority: 0,
                        primary: AtTimeMs(time_ms: 0),
                    ),
                    required_for_victory: true,
                    source: Manual([
                        Abnormality(abnormality_id: "enemy", tier: I, count: 1),
                    ]),
                )],
            )"#,
        );
        assert!(
            parsed.is_err(),
            "removed research authoring must fail closed"
        );
    }
}
