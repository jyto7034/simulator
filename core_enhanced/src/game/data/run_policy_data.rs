use serde::{Deserialize, Serialize};

use crate::game::{battle::result_stats::BattleResultMetricKind, enums::Tier};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPolicyData {
    pub setup: RunSetupPolicy,
    pub growth: GrowthPolicy,
    pub battle_runtime: BattleRuntimePolicy,
    pub battle_result_stats: BattleResultStatsPolicy,
    pub live_deployment: LiveBattleDeploymentPolicy,
    pub post_battle: PostBattleResolutionPolicy,
    pub support: SupportPolicy,
    pub headquarters: HeadquartersPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSetupPolicy {
    pub default_max_acts: u8,
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

    pub fn builtin() -> Self {
        Self::from_ron_str(include_str!(
            "../../../../game_resources/data/run/policy.ron"
        ))
        .expect("built-in run policy must be valid RON and satisfy contract")
    }

    pub fn validate_contract(&self) -> Result<(), String> {
        if self.setup.default_max_acts == 0 {
            return Err("run policy setup.default_max_acts must be greater than 0".to_string());
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

        assert_eq!(policy.setup.default_max_acts, 3);
        assert_eq!(policy.setup.starter_employee_count, 3);
        assert_eq!(policy.battle_runtime.movement_tick_ms, 50);
        assert_eq!(policy.battle_runtime.max_battle_time_ms, 60000);
        assert_eq!(policy.live_deployment.initial_cost, 20);
        assert_eq!(policy.support.save_point_trauma_reduction_percent, 15);
    }

    #[test]
    fn run_policy_rejects_legacy_or_unknown_fields() {
        let err = RunPolicyData::from_ron_str(
            r#"(
                setup: (default_max_acts: 3, starter_employee_count: 3, starter_enkephalin: 500),
                growth: (
                    xp_required_per_level: [100],
                    post_battle_survival_xp: 10,
                    battle_tier_by_level: [
                        (min_level: 1, tier: I),
                        (min_level: 3, tier: II),
                        (min_level: 6, tier: III),
                    ],
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
                ),
                headquarters: (emergency_enkephalin: 120),
                legacy_fallback: true,
            )"#,
        )
        .expect_err("unknown fields must fail validation");

        assert!(err.contains("legacy_fallback"));
    }
}
