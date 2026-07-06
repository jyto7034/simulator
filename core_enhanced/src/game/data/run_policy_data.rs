use serde::{Deserialize, Serialize};

use crate::game::{
    battle::result_stats::BattleResultMetricKind, data::pve_data::PveWaveData, enums::Tier,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPolicyData {
    pub setup: RunSetupPolicy,
    pub growth: GrowthPolicy,
    pub abnormality_research: AbnormalityResearchPolicy,
    pub encounter_repeat_weighting: EncounterRepeatWeightingPolicy,
    pub battle_runtime: BattleRuntimePolicy,
    pub floor_combat_scaling: FloorCombatScalingPolicy,
    pub battle_result_stats: BattleResultStatsPolicy,
    pub live_deployment: LiveBattleDeploymentPolicy,
    pub post_battle: PostBattleResolutionPolicy,
    pub support: SupportPolicy,
    pub headquarters: HeadquartersPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSetupPolicy {
    pub standard_floor_count: u8,
    pub starter_employee_count: usize,
    pub starter_enkephalin: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrowthPolicy {
    pub xp_required_per_level: Vec<u32>,
    pub post_battle_survival_xp: u32,
    pub battle_tier_by_level: Vec<BattleTierByLevelPolicy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbnormalityResearchPolicy {
    pub default_research_required: u32,
    pub victory_research_gain: u32,
    pub repeat_complete_fragment_dust: RepeatCompleteFragmentDustPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepeatCompleteFragmentDustPolicy {
    pub elite: u32,
    pub boss: u32,
    pub final_boss: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterRepeatWeightingPolicy {
    pub recent_floor_exclusion_window: u32,
    pub response_complete_weight_multiplier_percent: u32,
    pub incomplete_weight_multiplier_percent: u32,
    pub empty_pool_fallback: EmptyEncounterCandidateFallbackPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmptyEncounterCandidateFallbackPolicy {
    ResetSoftRepeatConstraints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleTierByLevelPolicy {
    pub min_level: u32,
    pub tier: Tier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleRuntimePolicy {
    pub movement_tick_ms: u64,
    pub max_battle_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloorCombatScalingPolicy {
    pub stages: Vec<FloorCombatScalingStage>,
    pub overflow_policy: FloorCombatScalingOverflowPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloorCombatScalingStage {
    pub min_floor: u32,
    pub max_floor: u32,
    pub max_health_multiplier_percent: u32,
    pub attack_multiplier_percent: u32,
    pub defense_multiplier_percent: u32,
    pub generated_wave_budget_multiplier_percent: u32,
    #[serde(default)]
    pub extra_waves: Vec<PveWaveData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloorCombatScalingOverflowPolicy {
    ClampToLastStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleResultStatsPolicy {
    pub mvp: BattleResultMvpPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleResultMvpPolicy {
    pub metric_weights: Vec<BattleResultMvpMetricWeight>,
    pub tiebreakers: Vec<BattleResultMvpTieBreaker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleResultMvpMetricWeight {
    pub metric: BattleResultMetricKind,
    pub weight: i64,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleResultMvpTieBreaker {
    Score,
    DamageDealt,
    KillCount,
    SkillCastCount,
    BasicAttackCount,
    DeployedTimeMs,
    EmployeeUuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostBattleResolutionPolicy {
    pub incapacitation_trauma: u32,
    pub incapacitation_run_hp_loss_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportPolicy {
    pub save_point_trauma_reduction_percent: u32,
    pub rest_trauma_heal: u32,
    pub gate_transition_trauma_recovery_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeadquartersPolicy {
    pub emergency_enkephalin: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveBattleDeploymentPolicy {
    pub initial_cost: u32,
    pub max_cost: u32,
    pub cost_per_second: u32,
    pub base_deploy_cost: u32,
    pub withdraw_redeploy_cooldown_ms: u64,
    pub defeat_redeploy_cooldown_ms: u64,
    pub redeploy_cost_multiplier_pct: u32,
    pub first_instance_salt: u32,
}

impl RunPolicyData {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        let policy: Self = ron::de::from_str(input).map_err(|err| err.to_string())?;
        policy.validate_contract()?;
        Ok(policy)
    }

    /// Load the embedded run policy.
    ///
    /// `RunPolicyData` owns this domain-level policy loader because many
    /// focused unit tests need run rules without constructing the full live
    /// `GameDataBase`. Production startup still reaches this through
    /// `GameDataBase::load_live_embedded()`.
    pub fn builtin() -> Self {
        Self::from_ron_str(include_str!(
            "../../../../game_resources/data/run/policy.ron"
        ))
        .expect("built-in run policy must be valid RON and satisfy contract")
    }

    pub fn validate_contract(&self) -> Result<(), String> {
        if self.setup.standard_floor_count == 0 {
            return Err("run policy setup.standard_floor_count must be greater than 0".to_string());
        }
        if self.setup.starter_employee_count == 0 {
            return Err(
                "run policy setup.starter_employee_count must be greater than 0".to_string(),
            );
        }
        if self.growth.xp_required_per_level.is_empty() {
            return Err("run policy growth.xp_required_per_level must not be empty".to_string());
        }
        if self
            .growth
            .xp_required_per_level
            .iter()
            .any(|required| *required == 0)
        {
            return Err(
                "run policy growth.xp_required_per_level entries must be greater than 0"
                    .to_string(),
            );
        }
        if self.growth.battle_tier_by_level.is_empty() {
            return Err("run policy growth.battle_tier_by_level must not be empty".to_string());
        }
        let mut previous_min_level = 0;
        for entry in &self.growth.battle_tier_by_level {
            if entry.min_level == 0 {
                return Err(
                    "run policy growth.battle_tier_by_level min_level must be greater than 0"
                        .to_string(),
                );
            }
            if entry.min_level <= previous_min_level {
                return Err(
                    "run policy growth.battle_tier_by_level must be sorted by increasing min_level"
                        .to_string(),
                );
            }
            previous_min_level = entry.min_level;
        }
        if self.abnormality_research.default_research_required == 0 {
            return Err(
                "run policy abnormality_research.default_research_required must be greater than 0"
                    .to_string(),
            );
        }
        if self.abnormality_research.victory_research_gain == 0 {
            return Err(
                "run policy abnormality_research.victory_research_gain must be greater than 0"
                    .to_string(),
            );
        }
        if self
            .encounter_repeat_weighting
            .response_complete_weight_multiplier_percent
            == 0
        {
            return Err(
                "run policy encounter_repeat_weighting.response_complete_weight_multiplier_percent must be greater than 0"
                    .to_string(),
            );
        }
        if self
            .encounter_repeat_weighting
            .incomplete_weight_multiplier_percent
            == 0
        {
            return Err(
                "run policy encounter_repeat_weighting.incomplete_weight_multiplier_percent must be greater than 0"
                    .to_string(),
            );
        }
        if self.battle_runtime.movement_tick_ms == 0 {
            return Err(
                "run policy battle_runtime.movement_tick_ms must be greater than 0".to_string(),
            );
        }
        if self.battle_runtime.max_battle_time_ms == 0 {
            return Err(
                "run policy battle_runtime.max_battle_time_ms must be greater than 0".to_string(),
            );
        }
        if self.battle_runtime.movement_tick_ms > self.battle_runtime.max_battle_time_ms {
            return Err(
                "run policy battle_runtime.movement_tick_ms must be <= max_battle_time_ms"
                    .to_string(),
            );
        }
        self.validate_floor_combat_scaling()?;
        if self.battle_result_stats.mvp.metric_weights.is_empty() {
            return Err(
                "run policy battle_result_stats.mvp.metric_weights must not be empty".to_string(),
            );
        }
        if self.battle_result_stats.mvp.tiebreakers.is_empty() {
            return Err(
                "run policy battle_result_stats.mvp.tiebreakers must not be empty".to_string(),
            );
        }
        if self.post_battle.incapacitation_run_hp_loss_percent > 100 {
            return Err(
                "run policy post_battle.incapacitation_run_hp_loss_percent must be <= 100"
                    .to_string(),
            );
        }
        if !(10..=20).contains(&self.support.save_point_trauma_reduction_percent) {
            return Err(
                "run policy support.save_point_trauma_reduction_percent must be between 10 and 20"
                    .to_string(),
            );
        }
        if self.support.gate_transition_trauma_recovery_percent > 100 {
            return Err(
                "run policy support.gate_transition_trauma_recovery_percent must be <= 100"
                    .to_string(),
            );
        }
        if self.live_deployment.initial_cost > self.live_deployment.max_cost {
            return Err("run policy live_deployment.initial_cost must be <= max_cost".to_string());
        }
        if self.live_deployment.base_deploy_cost > self.live_deployment.max_cost {
            return Err(
                "run policy live_deployment.base_deploy_cost must be <= max_cost".to_string(),
            );
        }
        if self.live_deployment.redeploy_cost_multiplier_pct == 0 {
            return Err(
                "run policy live_deployment.redeploy_cost_multiplier_pct must be greater than 0"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn validate_floor_combat_scaling(&self) -> Result<(), String> {
        if self.floor_combat_scaling.stages.is_empty() {
            return Err("run policy floor_combat_scaling.stages must not be empty".to_string());
        }
        let mut previous_max_floor = 0_u32;
        for stage in &self.floor_combat_scaling.stages {
            if stage.min_floor == 0 {
                return Err(
                    "run policy floor_combat_scaling stage min_floor must be greater than 0"
                        .to_string(),
                );
            }
            if stage.min_floor > stage.max_floor {
                return Err(
                    "run policy floor_combat_scaling stage min_floor must be <= max_floor"
                        .to_string(),
                );
            }
            if stage.min_floor <= previous_max_floor {
                return Err(
                    "run policy floor_combat_scaling stages must be sorted and non-overlapping"
                        .to_string(),
                );
            }
            for (field_name, value) in [
                (
                    "max_health_multiplier_percent",
                    stage.max_health_multiplier_percent,
                ),
                ("attack_multiplier_percent", stage.attack_multiplier_percent),
                (
                    "defense_multiplier_percent",
                    stage.defense_multiplier_percent,
                ),
                (
                    "generated_wave_budget_multiplier_percent",
                    stage.generated_wave_budget_multiplier_percent,
                ),
            ] {
                if value == 0 {
                    return Err(format!(
                        "run policy floor_combat_scaling stage {field_name} must be greater than 0"
                    ));
                }
            }
            previous_max_floor = stage.max_floor;
        }
        Ok(())
    }

    pub fn floor_combat_scaling_for_floor_index(
        &self,
        floor_index: u32,
    ) -> &FloorCombatScalingStage {
        let authored_floor = floor_index.saturating_add(1);
        self.floor_combat_scaling
            .stages
            .iter()
            .find(|stage| authored_floor >= stage.min_floor && authored_floor <= stage.max_floor)
            .unwrap_or_else(|| match self.floor_combat_scaling.overflow_policy {
                FloorCombatScalingOverflowPolicy::ClampToLastStage => self
                    .floor_combat_scaling
                    .stages
                    .last()
                    .expect("validated floor combat scaling must contain at least one stage"),
            })
    }

    pub fn xp_required_for_next_level(&self, current_level: u32) -> u32 {
        let index = current_level.saturating_sub(1) as usize;
        self.growth
            .xp_required_per_level
            .get(index)
            .copied()
            .or_else(|| self.growth.xp_required_per_level.last().copied())
            .expect("validated growth policy must contain at least one XP requirement")
    }

    pub fn battle_tier_for_level(&self, level: u32) -> Tier {
        self.growth
            .battle_tier_by_level
            .iter()
            .rev()
            .find(|entry| level >= entry.min_level)
            .or_else(|| self.growth.battle_tier_by_level.first())
            .map(|entry| entry.tier)
            .expect("validated growth policy must contain at least one battle tier")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_run_policy_validates_contract() {
        let policy = RunPolicyData::builtin();

        assert_eq!(policy.setup.standard_floor_count, 3);
        assert_eq!(policy.setup.starter_employee_count, 3);
        assert_eq!(policy.abnormality_research.default_research_required, 100);
        assert_eq!(policy.abnormality_research.victory_research_gain, 20);
        assert_eq!(
            policy
                .abnormality_research
                .repeat_complete_fragment_dust
                .elite,
            5
        );
        assert_eq!(
            policy
                .encounter_repeat_weighting
                .recent_floor_exclusion_window,
            1
        );
        assert_eq!(
            policy
                .encounter_repeat_weighting
                .response_complete_weight_multiplier_percent,
            25
        );
        assert_eq!(
            policy
                .encounter_repeat_weighting
                .incomplete_weight_multiplier_percent,
            100
        );
        assert_eq!(policy.battle_runtime.movement_tick_ms, 50);
        assert_eq!(policy.battle_runtime.max_battle_time_ms, 60000);
        assert_eq!(policy.live_deployment.initial_cost, 20);
        assert_eq!(policy.support.save_point_trauma_reduction_percent, 15);
        assert_eq!(policy.support.gate_transition_trauma_recovery_percent, 10);
    }

    #[test]
    fn floor_scaling_builtin_policy_maps_two_floor_stages_and_clamps() {
        let policy = RunPolicyData::builtin();

        let stage_1 = policy.floor_combat_scaling_for_floor_index(0);
        assert_eq!((stage_1.min_floor, stage_1.max_floor), (1, 2));
        assert_eq!(stage_1.max_health_multiplier_percent, 100);
        assert_eq!(stage_1.attack_multiplier_percent, 100);
        assert_eq!(stage_1.defense_multiplier_percent, 100);
        assert_eq!(stage_1.generated_wave_budget_multiplier_percent, 100);

        let stage_2 = policy.floor_combat_scaling_for_floor_index(2);
        assert_eq!((stage_2.min_floor, stage_2.max_floor), (3, 4));
        assert_eq!(stage_2.max_health_multiplier_percent, 110);

        let stage_4 = policy.floor_combat_scaling_for_floor_index(6);
        assert_eq!((stage_4.min_floor, stage_4.max_floor), (7, 8));
        assert_eq!(stage_4.max_health_multiplier_percent, 130);
        assert_eq!(stage_4.attack_multiplier_percent, 130);
        assert_eq!(stage_4.defense_multiplier_percent, 130);
        assert_eq!(stage_4.generated_wave_budget_multiplier_percent, 175);

        let overflow = policy.floor_combat_scaling_for_floor_index(99);
        assert_eq!((overflow.min_floor, overflow.max_floor), (7, 8));
    }

    #[test]
    fn floor_scaling_policy_rejects_invalid_stage_data() {
        let mut policy = RunPolicyData::builtin();
        policy.floor_combat_scaling.stages[1].min_floor = 2;
        let err = policy
            .validate_contract()
            .expect_err("overlapping floor scaling stages must fail");
        assert!(err.contains("sorted and non-overlapping"));

        let mut policy = RunPolicyData::builtin();
        policy.floor_combat_scaling.stages[0].attack_multiplier_percent = 0;
        let err = policy
            .validate_contract()
            .expect_err("zero scaling multipliers must fail");
        assert!(err.contains("attack_multiplier_percent"));
    }

    #[test]
    fn encounter_repeat_weighting_policy_rejects_zero_multipliers() {
        let mut policy = RunPolicyData::builtin();
        policy
            .encounter_repeat_weighting
            .response_complete_weight_multiplier_percent = 0;
        let err = policy
            .validate_contract()
            .expect_err("zero response-complete multiplier must fail");
        assert!(err.contains("response_complete_weight_multiplier_percent"));

        let mut policy = RunPolicyData::builtin();
        policy
            .encounter_repeat_weighting
            .incomplete_weight_multiplier_percent = 0;
        let err = policy
            .validate_contract()
            .expect_err("zero incomplete multiplier must fail");
        assert!(err.contains("incomplete_weight_multiplier_percent"));
    }

    #[test]
    fn run_policy_rejects_legacy_or_unknown_fields() {
        let err = RunPolicyData::from_ron_str(
            r#"(
                setup: (standard_floor_count: 3, starter_employee_count: 3, starter_enkephalin: 500),
                growth: (
                    xp_required_per_level: [100],
                    post_battle_survival_xp: 10,
                    battle_tier_by_level: [
                        (min_level: 1, tier: I),
                        (min_level: 3, tier: II),
                        (min_level: 6, tier: III),
                    ],
                ),
                abnormality_research: (
                    default_research_required: 100,
                    victory_research_gain: 20,
                    repeat_complete_fragment_dust: (
                        elite: 5,
                        boss: 10,
                        final_boss: 20,
                    ),
                ),
                encounter_repeat_weighting: (
                    recent_floor_exclusion_window: 1,
                    response_complete_weight_multiplier_percent: 25,
                    incomplete_weight_multiplier_percent: 100,
                    empty_pool_fallback: ResetSoftRepeatConstraints,
                ),
                battle_runtime: (
                    movement_tick_ms: 50,
                    max_battle_time_ms: 60000,
                ),
                battle_result_stats: (
                    mvp: (
                        metric_weights: [
                            (metric: DamageDealt, weight: 1, title: "Top Damage"),
                        ],
                        tiebreakers: [Score, DamageDealt, EmployeeUuid],
                    ),
                ),
                live_deployment: (
                    initial_cost: 20,
                    max_cost: 99,
                    cost_per_second: 1,
                    base_deploy_cost: 10,
                    withdraw_redeploy_cooldown_ms: 30000,
                    defeat_redeploy_cooldown_ms: 90000,
                    redeploy_cost_multiplier_pct: 150,
                    first_instance_salt: 10000,
                ),
                post_battle: (
                    incapacitation_trauma: 40,
                    incapacitation_run_hp_loss_percent: 25,
                ),
                support: (
                    save_point_trauma_reduction_percent: 15,
                    rest_trauma_heal: 10,
                    gate_transition_trauma_recovery_percent: 10,
                ),
                headquarters: (emergency_enkephalin: 120),
                legacy_fallback: true,
            )"#,
        )
        .expect_err("unknown fields must fail validation");

        assert!(err.contains("legacy_fallback"));
    }
}
