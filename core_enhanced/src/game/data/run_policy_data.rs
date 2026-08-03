use std::collections::HashSet;

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
    pub monolith: MonolithPolicy,
    pub battle_runtime: BattleRuntimePolicy,
    pub combat_difficulty: CombatDifficultyPolicy,
    pub battle_result_stats: BattleResultStatsPolicy,
    pub live_deployment: LiveBattleDeploymentPolicy,
    pub post_battle: PostBattleResolutionPolicy,
    pub save_point_trauma_recovery: TraumaRecovery,
    pub gate_trauma_recovery: TraumaRecovery,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithPolicy {
    pub wandering_merchant_min_floor: u32,
    pub wandering_merchant_shop_id: String,
    pub first_page_offer_count: usize,
    pub paid_refresh_uses: u8,
    pub initial_uses: u8,
    pub remanifestation_max_attempts: u8,
    pub residual_exchange_threshold: usize,
    pub monolith_price: u32,
    pub monolith_offer_weight: u32,
    pub stage_2_stat_multiplier_percent: u32,
    pub stage_3_stat_multiplier_percent: u32,
    pub stage_2_reward_multiplier_percent: u32,
    pub stage_3_reward_multiplier_percent: u32,
    pub exchange_visual_novel_id: String,
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
pub struct CombatDifficultyPolicy {
    pub power_model_version: u32,
    pub power_weights: CertifiedCombatPowerWeights,
    pub threat_bands: RelativeThreatBandPolicy,
    pub standard_floor_anchors: Vec<StandardFloorPowerAnchorPolicy>,
    pub endless_floor_anchor_stages: Vec<EndlessFloorPowerAnchorStage>,
    pub profiles: Vec<CombatDifficultyProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CertifiedCombatPowerWeights {
    pub level_flat: u32,
    pub max_health_percent: u32,
    pub attack_dps_percent: u32,
    pub defense_per_point: u32,
    pub magic_resist_per_point: u32,
    pub block_capacity_per_unit: u32,
    pub basic_attack_range_per_cell: u32,
    pub active_skill_flat: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeThreatBandPolicy {
    pub low_max_ratio_percent: u32,
    pub equal_max_ratio_percent: u32,
    pub high_max_ratio_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardFloorPowerAnchorPolicy {
    pub floor: u8,
    pub authored_minimum_power: u64,
    pub minimum_rise: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndlessFloorPowerAnchorStage {
    pub min_floor: u32,
    pub max_floor: Option<u32>,
    pub authored_minimum_power: u64,
    pub minimum_rise_per_floor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatDifficultyProfile {
    pub id: String,
    pub ordinary_bands: Vec<OrdinaryCombatDifficultyTuning>,
    pub role_tuning: Vec<RoleCombatDifficultyTuning>,
    pub quota_rounding: CombatDifficultyQuotaRounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrdinaryCombatDifficultyBand {
    Easy,
    Standard,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatNodeDifficultyRole {
    Elite,
    NormalBoss,
    FinalBoss,
    BossOmen,
    MonolithRemanifestation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatDifficultyQuotaRounding {
    LargestRemainderThenSeededTieBreak,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinaryCombatDifficultyTuning {
    pub band: OrdinaryCombatDifficultyBand,
    pub quota_percent: u32,
    pub tuning: CombatDifficultyTuning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleCombatDifficultyTuning {
    pub role: CombatNodeDifficultyRole,
    pub tuning: CombatDifficultyTuning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatDifficultyTuning {
    pub target_power_percent: u32,
    pub max_health_multiplier_percent: u32,
    pub attack_multiplier_percent: u32,
    pub defense_multiplier_percent: u32,
    pub generated_wave_budget_multiplier_percent: u32,
    pub reward_multiplier_percent: u32,
    pub extra_waves: Vec<PveWaveData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedFloorPowerAnchorPolicy {
    pub authored_minimum_power: u64,
    pub minimum_rise: u64,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TraumaRecovery {
    Flat { amount: u32 },
    CurrentPercent { percent: u32 },
}

impl TraumaRecovery {
    pub fn validate(self, field_name: &str) -> Result<(), String> {
        let (kind, value) = match self {
            Self::Flat { amount } => ("amount", amount),
            Self::CurrentPercent { percent } => ("percent", percent),
        };
        if !(1..=100).contains(&value) {
            return Err(format!(
                "run policy {field_name}.{kind} must be between 1 and 100"
            ));
        }
        Ok(())
    }

    pub fn recovery_amount(self, current_trauma: u32) -> u32 {
        match self {
            Self::Flat { amount } => current_trauma.min(amount),
            Self::CurrentPercent { percent } => {
                let quotient = current_trauma / 100;
                let remainder = current_trauma % 100;
                let whole = quotient.saturating_mul(percent);
                let fractional_product = remainder.saturating_mul(percent);
                whole
                    .saturating_add(fractional_product / 100)
                    .saturating_add((fractional_product % 100 != 0) as u32)
                    .min(current_trauma)
            }
        }
    }

    pub fn apply(self, current_trauma: u32) -> u32 {
        current_trauma.saturating_sub(self.recovery_amount(current_trauma))
    }
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
    /// `GameDataBase`. Production startup does not call this helper; it parses
    /// the same file through `GameDataBase::load_live_embedded()`'s versioned
    /// source set.
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
        self.validate_combat_difficulty()?;
        self.validate_monolith()?;
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
        self.save_point_trauma_recovery
            .validate("save_point_trauma_recovery")?;
        self.gate_trauma_recovery.validate("gate_trauma_recovery")?;
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

    fn validate_monolith(&self) -> Result<(), String> {
        let policy = &self.monolith;
        if policy.wandering_merchant_min_floor < 4 {
            return Err(
                "run policy monolith.wandering_merchant_min_floor must be at least 4".to_string(),
            );
        }
        if policy.wandering_merchant_shop_id.trim().is_empty()
            || policy.exchange_visual_novel_id.trim().is_empty()
        {
            return Err(
                "run policy monolith shop and visual-novel ids must not be empty".to_string(),
            );
        }
        if policy.first_page_offer_count != 3 || policy.paid_refresh_uses != 1 {
            return Err(
                "run policy monolith requires exactly three first-page offers and one paid refresh"
                    .to_string(),
            );
        }
        for (field, value) in [
            ("initial_uses", usize::from(policy.initial_uses)),
            (
                "remanifestation_max_attempts",
                usize::from(policy.remanifestation_max_attempts),
            ),
            (
                "residual_exchange_threshold",
                policy.residual_exchange_threshold,
            ),
            ("monolith_price", policy.monolith_price as usize),
            (
                "monolith_offer_weight",
                policy.monolith_offer_weight as usize,
            ),
            (
                "stage_2_stat_multiplier_percent",
                policy.stage_2_stat_multiplier_percent as usize,
            ),
            (
                "stage_3_stat_multiplier_percent",
                policy.stage_3_stat_multiplier_percent as usize,
            ),
            (
                "stage_2_reward_multiplier_percent",
                policy.stage_2_reward_multiplier_percent as usize,
            ),
            (
                "stage_3_reward_multiplier_percent",
                policy.stage_3_reward_multiplier_percent as usize,
            ),
        ] {
            if value == 0 {
                return Err(format!(
                    "run policy monolith.{field} must be greater than 0"
                ));
            }
        }
        if policy.initial_uses < 2 || policy.remanifestation_max_attempts != 2 {
            return Err(
                "run policy monolith requires at least two uses and exactly two remanifestation attempts"
                    .to_string(),
            );
        }
        if policy.stage_3_reward_multiplier_percent <= policy.stage_2_reward_multiplier_percent {
            return Err(
                "run policy monolith stage 3 reward multiplier must exceed stage 2".to_string(),
            );
        }
        if policy.stage_3_stat_multiplier_percent <= policy.stage_2_stat_multiplier_percent {
            return Err(
                "run policy monolith stage 3 stat multiplier must exceed stage 2".to_string(),
            );
        }
        Ok(())
    }

    fn validate_combat_difficulty(&self) -> Result<(), String> {
        let policy = &self.combat_difficulty;
        if policy.power_model_version == 0 {
            return Err(
                "run policy combat_difficulty.power_model_version must be greater than 0"
                    .to_string(),
            );
        }
        for (field, value) in [
            ("level_flat", policy.power_weights.level_flat),
            (
                "max_health_percent",
                policy.power_weights.max_health_percent,
            ),
            (
                "attack_dps_percent",
                policy.power_weights.attack_dps_percent,
            ),
            ("defense_per_point", policy.power_weights.defense_per_point),
            (
                "magic_resist_per_point",
                policy.power_weights.magic_resist_per_point,
            ),
            (
                "block_capacity_per_unit",
                policy.power_weights.block_capacity_per_unit,
            ),
            (
                "basic_attack_range_per_cell",
                policy.power_weights.basic_attack_range_per_cell,
            ),
            ("active_skill_flat", policy.power_weights.active_skill_flat),
        ] {
            if value == 0 {
                return Err(format!(
                    "run policy combat_difficulty.power_weights.{field} must be greater than 0"
                ));
            }
        }
        let bands = policy.threat_bands;
        if bands.low_max_ratio_percent == 0
            || bands.low_max_ratio_percent >= bands.equal_max_ratio_percent
            || bands.equal_max_ratio_percent >= bands.high_max_ratio_percent
        {
            return Err(
                "run policy combat_difficulty.threat_bands must be strictly increasing and non-zero"
                    .to_string(),
            );
        }

        if policy.standard_floor_anchors.len() != usize::from(self.setup.standard_floor_count) {
            return Err(format!(
                "run policy combat_difficulty.standard_floor_anchors must define exactly {} floors",
                self.setup.standard_floor_count
            ));
        }
        for (index, anchor) in policy.standard_floor_anchors.iter().enumerate() {
            let expected_floor = u8::try_from(index + 1)
                .map_err(|_| "Standard power anchor floor overflowed u8".to_string())?;
            if anchor.floor != expected_floor || anchor.authored_minimum_power == 0 {
                return Err(format!(
                    "run policy combat_difficulty.standard_floor_anchors[{index}] must define floor {expected_floor} with non-zero authored_minimum_power"
                ));
            }
            if index > 0 && anchor.minimum_rise == 0 {
                return Err(format!(
                    "run policy combat_difficulty.standard_floor_anchors[{index}].minimum_rise must be greater than 0"
                ));
            }
        }

        if policy.endless_floor_anchor_stages.is_empty() {
            return Err(
                "run policy combat_difficulty.endless_floor_anchor_stages must not be empty"
                    .to_string(),
            );
        }
        let mut expected_min_floor = 1_u32;
        for (index, stage) in policy.endless_floor_anchor_stages.iter().enumerate() {
            if stage.min_floor != expected_min_floor
                || stage.authored_minimum_power == 0
                || stage.minimum_rise_per_floor == 0
            {
                return Err(format!(
                    "run policy combat_difficulty.endless_floor_anchor_stages[{index}] has an invalid start, minimum, or rise"
                ));
            }
            match stage.max_floor {
                Some(max_floor) if max_floor >= stage.min_floor => {
                    expected_min_floor = max_floor.checked_add(1).ok_or_else(|| {
                        "Endless power anchor stage max_floor overflowed u32".to_string()
                    })?;
                }
                Some(_) => {
                    return Err(format!(
                        "run policy combat_difficulty.endless_floor_anchor_stages[{index}] has an inverted range"
                    ))
                }
                None if index + 1 == policy.endless_floor_anchor_stages.len() => {}
                None => {
                    return Err(
                        "only the final Endless power anchor stage may be open-ended".to_string(),
                    )
                }
            }
        }
        if policy
            .endless_floor_anchor_stages
            .last()
            .is_some_and(|stage| stage.max_floor.is_some())
        {
            return Err("final Endless power anchor stage must be open-ended".to_string());
        }

        if policy.profiles.is_empty() {
            return Err("run policy combat_difficulty.profiles must not be empty".to_string());
        }
        let expected_bands = HashSet::from([
            OrdinaryCombatDifficultyBand::Easy,
            OrdinaryCombatDifficultyBand::Standard,
            OrdinaryCombatDifficultyBand::Hard,
        ]);
        let expected_roles = HashSet::from([
            CombatNodeDifficultyRole::Elite,
            CombatNodeDifficultyRole::NormalBoss,
            CombatNodeDifficultyRole::FinalBoss,
            CombatNodeDifficultyRole::BossOmen,
            CombatNodeDifficultyRole::MonolithRemanifestation,
        ]);
        let mut profile_ids = HashSet::new();
        for profile in &policy.profiles {
            if profile.id.trim().is_empty() || !profile_ids.insert(profile.id.as_str()) {
                return Err(format!(
                    "run policy combat_difficulty has invalid or duplicate profile '{}'",
                    profile.id
                ));
            }
            let actual_bands = profile
                .ordinary_bands
                .iter()
                .map(|entry| entry.band)
                .collect::<HashSet<_>>();
            let quota_total = profile
                .ordinary_bands
                .iter()
                .map(|entry| entry.quota_percent)
                .sum::<u32>();
            if actual_bands != expected_bands
                || profile.ordinary_bands.len() != expected_bands.len()
                || quota_total != 100
                || profile
                    .ordinary_bands
                    .iter()
                    .any(|entry| entry.quota_percent == 0)
            {
                return Err(format!(
                    "run policy combat_difficulty profile '{}' must define Easy/Standard/Hard once with non-zero quotas totaling 100",
                    profile.id
                ));
            }
            let actual_roles = profile
                .role_tuning
                .iter()
                .map(|entry| entry.role)
                .collect::<HashSet<_>>();
            if actual_roles != expected_roles || profile.role_tuning.len() != expected_roles.len() {
                return Err(format!(
                    "run policy combat_difficulty profile '{}' must define every special combat role exactly once",
                    profile.id
                ));
            }
            for tuning in profile
                .ordinary_bands
                .iter()
                .map(|entry| &entry.tuning)
                .chain(profile.role_tuning.iter().map(|entry| &entry.tuning))
            {
                validate_combat_difficulty_tuning(&profile.id, tuning)?;
            }
            let standard_reward_multiplier = profile
                .ordinary_bands
                .iter()
                .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Standard)
                .expect("validated profile contains Standard")
                .tuning
                .reward_multiplier_percent;
            let hard_reward_multiplier = profile
                .ordinary_bands
                .iter()
                .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Hard)
                .expect("validated profile contains Hard")
                .tuning
                .reward_multiplier_percent;
            if hard_reward_multiplier <= standard_reward_multiplier {
                return Err(format!(
                    "run policy combat_difficulty profile '{}' Hard reward multiplier must be greater than Standard",
                    profile.id
                ));
            }
            if profile
                .role_tuning
                .iter()
                .any(|entry| entry.tuning.reward_multiplier_percent <= standard_reward_multiplier)
            {
                return Err(format!(
                    "run policy combat_difficulty profile '{}' special-role reward multipliers must be greater than Standard",
                    profile.id
                ));
            }
        }
        Ok(())
    }

    pub fn combat_difficulty_profile(&self, id: &str) -> Option<&CombatDifficultyProfile> {
        self.combat_difficulty
            .profiles
            .iter()
            .find(|profile| profile.id == id)
    }

    /// Explicit Standard contract for non-manifested validation fixtures and
    /// Event-authored combat. Manifested Combat/Boss nodes must never use it.
    pub fn standard_non_manifested_combat_tuning(&self) -> &CombatDifficultyTuning {
        &self
            .combat_difficulty
            .profiles
            .first()
            .expect("validated combat difficulty policy must contain a profile")
            .ordinary_bands
            .iter()
            .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Standard)
            .expect("validated combat difficulty profile must contain Standard")
            .tuning
    }

    pub fn resolved_tuning_for_manifested_node(
        &self,
        difficulty: &crate::game::map::MapNodeCombatDifficulty,
    ) -> Option<CombatDifficultyTuning> {
        if difficulty.power_model_version != self.combat_difficulty.power_model_version {
            return None;
        }
        let profile = self.combat_difficulty_profile(&difficulty.difficulty_profile_id)?;
        let mut tuning = match difficulty.class {
            crate::game::map::MapNodeCombatDifficultyClass::Ordinary(band) => profile
                .ordinary_bands
                .iter()
                .find(|entry| entry.band == band)
                .map(|entry| entry.tuning.clone()),
            crate::game::map::MapNodeCombatDifficultyClass::Role(role) => profile
                .role_tuning
                .iter()
                .find(|entry| entry.role == role)
                .map(|entry| entry.tuning.clone()),
        }?;
        let is_monolith_role = matches!(
            difficulty.class,
            crate::game::map::MapNodeCombatDifficultyClass::Role(
                CombatNodeDifficultyRole::MonolithRemanifestation
            )
        );
        match (is_monolith_role, difficulty.monolith_stage) {
            (false, None) => Some(tuning),
            (true, Some(stage)) => {
                let (stat_multiplier, reward_multiplier) = match stage {
                    crate::game::map::MonolithRemanifestationStage::Stage2 => (
                        self.monolith.stage_2_stat_multiplier_percent,
                        self.monolith.stage_2_reward_multiplier_percent,
                    ),
                    crate::game::map::MonolithRemanifestationStage::Stage3 => (
                        self.monolith.stage_3_stat_multiplier_percent,
                        self.monolith.stage_3_reward_multiplier_percent,
                    ),
                };
                tuning.target_power_percent =
                    percent_product_ceil(tuning.target_power_percent, stat_multiplier);
                tuning.max_health_multiplier_percent =
                    percent_product_ceil(tuning.max_health_multiplier_percent, stat_multiplier);
                tuning.attack_multiplier_percent =
                    percent_product_ceil(tuning.attack_multiplier_percent, stat_multiplier);
                tuning.defense_multiplier_percent =
                    percent_product_ceil(tuning.defense_multiplier_percent, stat_multiplier);
                tuning.reward_multiplier_percent =
                    percent_product_ceil(tuning.reward_multiplier_percent, reward_multiplier);
                Some(tuning)
            }
            _ => None,
        }
    }

    pub fn floor_power_anchor_policy(
        &self,
        mode: crate::game::map::GameMode,
        floor_index: u32,
    ) -> Option<ResolvedFloorPowerAnchorPolicy> {
        let authored_floor = floor_index.checked_add(1)?;
        match mode {
            crate::game::map::GameMode::Standard => self
                .combat_difficulty
                .standard_floor_anchors
                .iter()
                .find(|entry| u32::from(entry.floor) == authored_floor)
                .map(|entry| ResolvedFloorPowerAnchorPolicy {
                    authored_minimum_power: entry.authored_minimum_power,
                    minimum_rise: entry.minimum_rise,
                }),
            crate::game::map::GameMode::Endless => self
                .combat_difficulty
                .endless_floor_anchor_stages
                .iter()
                .find(|stage| {
                    authored_floor >= stage.min_floor
                        && stage
                            .max_floor
                            .is_none_or(|max_floor| authored_floor <= max_floor)
                })
                .map(|stage| {
                    let offset = authored_floor.saturating_sub(stage.min_floor);
                    ResolvedFloorPowerAnchorPolicy {
                        authored_minimum_power: stage.authored_minimum_power.saturating_add(
                            stage
                                .minimum_rise_per_floor
                                .saturating_mul(u64::from(offset)),
                        ),
                        minimum_rise: stage.minimum_rise_per_floor,
                    }
                }),
        }
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

fn percent_product_ceil(left_percent: u32, right_percent: u32) -> u32 {
    let product = u64::from(left_percent).saturating_mul(u64::from(right_percent));
    u32::try_from(product.saturating_add(99) / 100).unwrap_or(u32::MAX)
}

fn validate_combat_difficulty_tuning(
    profile_id: &str,
    tuning: &CombatDifficultyTuning,
) -> Result<(), String> {
    for (field, value) in [
        ("target_power_percent", tuning.target_power_percent),
        (
            "max_health_multiplier_percent",
            tuning.max_health_multiplier_percent,
        ),
        (
            "attack_multiplier_percent",
            tuning.attack_multiplier_percent,
        ),
        (
            "defense_multiplier_percent",
            tuning.defense_multiplier_percent,
        ),
        (
            "generated_wave_budget_multiplier_percent",
            tuning.generated_wave_budget_multiplier_percent,
        ),
        (
            "reward_multiplier_percent",
            tuning.reward_multiplier_percent,
        ),
    ] {
        if value == 0 {
            return Err(format!(
                "run policy combat_difficulty profile '{profile_id}' tuning {field} must be greater than 0"
            ));
        }
    }
    Ok(())
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
        assert_eq!(
            policy.save_point_trauma_recovery,
            TraumaRecovery::CurrentPercent { percent: 15 }
        );
        assert_eq!(
            policy.gate_trauma_recovery,
            TraumaRecovery::CurrentPercent { percent: 10 }
        );
    }

    #[test]
    fn monolith_stage_three_stat_multiplier_must_exceed_stage_two() {
        let mut policy = RunPolicyData::builtin();
        policy.monolith.stage_3_stat_multiplier_percent =
            policy.monolith.stage_2_stat_multiplier_percent;

        assert!(policy
            .validate_contract()
            .unwrap_err()
            .contains("stage 3 stat multiplier must exceed stage 2"));
    }

    #[test]
    fn monolith_stage_resolver_composes_stats_and_reward_but_not_waves() {
        let policy = RunPolicyData::builtin();
        let difficulty = |stage| crate::game::map::MapNodeCombatDifficulty {
            power_model_version: policy.combat_difficulty.power_model_version,
            difficulty_profile_id: policy.combat_difficulty.profiles[0].id.clone(),
            class: crate::game::map::MapNodeCombatDifficultyClass::Role(
                CombatNodeDifficultyRole::MonolithRemanifestation,
            ),
            monolith_stage: Some(stage),
            floor_power_anchor: 100,
            target_power: 0,
        };

        let stage_two = policy
            .resolved_tuning_for_manifested_node(&difficulty(
                crate::game::map::MonolithRemanifestationStage::Stage2,
            ))
            .unwrap();
        let stage_three = policy
            .resolved_tuning_for_manifested_node(&difficulty(
                crate::game::map::MonolithRemanifestationStage::Stage3,
            ))
            .unwrap();

        assert_eq!(stage_two.target_power_percent, 220);
        assert_eq!(stage_two.max_health_multiplier_percent, 165);
        assert_eq!(stage_two.attack_multiplier_percent, 145);
        assert_eq!(stage_two.defense_multiplier_percent, 145);
        assert_eq!(stage_two.generated_wave_budget_multiplier_percent, 200);
        assert_eq!(stage_two.reward_multiplier_percent, 250);
        assert_eq!(stage_three.target_power_percent, 275);
        assert_eq!(stage_three.max_health_multiplier_percent, 207);
        assert_eq!(stage_three.attack_multiplier_percent, 182);
        assert_eq!(stage_three.defense_multiplier_percent, 182);
        assert_eq!(stage_three.generated_wave_budget_multiplier_percent, 200);
        assert_eq!(stage_three.reward_multiplier_percent, 313);
        assert_eq!(stage_three.extra_waves, stage_two.extra_waves);
    }

    #[test]
    fn monolith_stage_and_role_must_be_present_together() {
        let policy = RunPolicyData::builtin();
        let mut difficulty = crate::game::map::MapNodeCombatDifficulty {
            power_model_version: policy.combat_difficulty.power_model_version,
            difficulty_profile_id: policy.combat_difficulty.profiles[0].id.clone(),
            class: crate::game::map::MapNodeCombatDifficultyClass::Role(
                CombatNodeDifficultyRole::MonolithRemanifestation,
            ),
            monolith_stage: None,
            floor_power_anchor: 100,
            target_power: 220,
        };
        assert!(policy
            .resolved_tuning_for_manifested_node(&difficulty)
            .is_none());

        difficulty.class = crate::game::map::MapNodeCombatDifficultyClass::Ordinary(
            OrdinaryCombatDifficultyBand::Standard,
        );
        difficulty.monolith_stage = Some(crate::game::map::MonolithRemanifestationStage::Stage2);
        assert!(policy
            .resolved_tuning_for_manifested_node(&difficulty)
            .is_none());
    }

    #[test]
    fn power_anchor_policy_resolves_standard_and_open_ended_endless_floors() {
        let policy = RunPolicyData::builtin();

        assert_eq!(
            policy.floor_power_anchor_policy(crate::game::map::GameMode::Standard, 0),
            Some(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 100,
                minimum_rise: 0,
            })
        );
        assert_eq!(
            policy.floor_power_anchor_policy(crate::game::map::GameMode::Standard, 2),
            Some(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 150,
                minimum_rise: 25,
            })
        );
        assert_eq!(
            policy.floor_power_anchor_policy(crate::game::map::GameMode::Standard, 3),
            None
        );
        assert_eq!(
            policy.floor_power_anchor_policy(crate::game::map::GameMode::Endless, 6),
            Some(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 265,
                minimum_rise: 30,
            })
        );
    }

    #[test]
    fn combat_difficulty_policy_rejects_invalid_anchor_and_tuning_data() {
        let mut policy = RunPolicyData::builtin();
        policy.combat_difficulty.endless_floor_anchor_stages[1].min_floor = 3;
        let err = policy
            .validate_contract()
            .expect_err("overlapping Endless anchor stages must fail");
        assert!(err.contains("invalid start"));

        let mut policy = RunPolicyData::builtin();
        policy.combat_difficulty.profiles[0].ordinary_bands[0]
            .tuning
            .attack_multiplier_percent = 0;
        let err = policy
            .validate_contract()
            .expect_err("zero difficulty tuning multipliers must fail");
        assert!(err.contains("attack_multiplier_percent"));

        let mut policy = RunPolicyData::builtin();
        policy.combat_difficulty.power_weights.level_flat = 0;
        let err = policy
            .validate_contract()
            .expect_err("zero certified power weights must fail");
        assert!(err.contains("power_weights.level_flat"));

        let mut policy = RunPolicyData::builtin();
        policy.combat_difficulty.profiles[0].ordinary_bands[0].quota_percent = 26;
        let err = policy
            .validate_contract()
            .expect_err("ordinary difficulty quotas must total 100");
        assert!(err.contains("totaling 100"));

        let mut policy = RunPolicyData::builtin();
        let standard_reward = policy.combat_difficulty.profiles[0]
            .ordinary_bands
            .iter()
            .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Standard)
            .unwrap()
            .tuning
            .reward_multiplier_percent;
        policy.combat_difficulty.profiles[0]
            .ordinary_bands
            .iter_mut()
            .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Hard)
            .unwrap()
            .tuning
            .reward_multiplier_percent = standard_reward;
        let err = policy
            .validate_contract()
            .expect_err("Hard rewards must be stronger than Standard");
        assert!(err.contains("Hard reward multiplier"));

        let mut policy = RunPolicyData::builtin();
        policy.combat_difficulty.profiles[0].role_tuning[0]
            .tuning
            .reward_multiplier_percent = standard_reward;
        let err = policy
            .validate_contract()
            .expect_err("special-role rewards must be stronger than Standard");
        assert!(err.contains("special-role reward multipliers"));
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
                ),
                save_point_trauma_recovery: CurrentPercent(percent: 15),
                gate_trauma_recovery: CurrentPercent(percent: 10),
                headquarters: (emergency_enkephalin: 120),
                legacy_fallback: true,
            )"#,
        )
        .expect_err("unknown fields must fail validation");

        assert!(err.contains("legacy_fallback"));
    }

    #[test]
    fn trauma_recovery_supports_flat_and_ceil_current_percent() {
        assert_eq!(TraumaRecovery::CurrentPercent { percent: 15 }.apply(0), 0);
        assert_eq!(TraumaRecovery::Flat { amount: 15 }.apply(10), 0);
        assert_eq!(TraumaRecovery::Flat { amount: 15 }.apply(100), 85);
        assert_eq!(TraumaRecovery::CurrentPercent { percent: 15 }.apply(1), 0);
        assert_eq!(TraumaRecovery::CurrentPercent { percent: 15 }.apply(99), 84);
        assert_eq!(
            TraumaRecovery::CurrentPercent { percent: 15 }.apply(100),
            85
        );
        assert_eq!(TraumaRecovery::CurrentPercent { percent: 10 }.apply(95), 85);
    }

    #[test]
    fn trauma_recovery_rejects_zero_or_out_of_range_values() {
        for invalid in [
            TraumaRecovery::Flat { amount: 0 },
            TraumaRecovery::Flat { amount: 101 },
            TraumaRecovery::CurrentPercent { percent: 0 },
            TraumaRecovery::CurrentPercent { percent: 101 },
        ] {
            assert!(invalid.validate("recovery").is_err());
        }
    }

    #[test]
    fn trauma_recovery_parses_exact_tagged_ron_variants() {
        assert_eq!(
            ron::de::from_str::<TraumaRecovery>("Flat(amount: 12)").unwrap(),
            TraumaRecovery::Flat { amount: 12 }
        );
        assert_eq!(
            ron::de::from_str::<TraumaRecovery>("CurrentPercent(percent: 15)").unwrap(),
            TraumaRecovery::CurrentPercent { percent: 15 }
        );
        assert!(
            ron::de::from_str::<TraumaRecovery>("CurrentPercent(percent: 15, amount: 12)").is_err()
        );
    }

    #[test]
    fn run_policy_rejects_legacy_support_group() {
        let current = include_str!("../../../../game_resources/data/run/policy.ron");
        let legacy = current.replace(
            "    save_point_trauma_recovery: CurrentPercent(percent: 15),\n    gate_trauma_recovery: CurrentPercent(percent: 10),\n",
            "    support: (\n        save_point_trauma_reduction_percent: 15,\n        rest_trauma_heal: 10,\n        gate_transition_trauma_recovery_percent: 10,\n    ),\n",
        );
        assert_ne!(legacy, current);

        let error = RunPolicyData::from_ron_str(&legacy).unwrap_err();
        assert!(error.contains("support") || error.contains("save_point_trauma_recovery"));
    }

    #[test]
    fn run_policy_rejects_removed_floor_combat_scaling_authority() {
        let current = include_str!("../../../../game_resources/data/run/policy.ron");
        let legacy = current.replacen("    combat_difficulty: (", "    floor_combat_scaling: (", 1);
        assert_ne!(legacy, current);

        let error = RunPolicyData::from_ron_str(&legacy).unwrap_err();
        assert!(error.contains("floor_combat_scaling") || error.contains("combat_difficulty"));
    }
}
