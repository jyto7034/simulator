//! The three basic relic-node activities. Authored supply is separate from fixed rules.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::game::{data::relic_data::{RelicDatabase, RelicDefinition}, map::{MapNodeId, RewardActivityKind}, relic::OwnedRelic};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelicRewardGrade { Common, Rare, Special }
impl RelicRewardGrade {
    pub const ALL: [Self; 3] = [Self::Common, Self::Rare, Self::Special];
    pub fn from_rarity(value: &str) -> Option<Self> {
        match value { "common" => Some(Self::Common), "rare" => Some(Self::Rare), "special" => Some(Self::Special), _ => None }
    }
    pub fn index(self) -> usize { match self { Self::Common => 0, Self::Rare => 1, Self::Special => 2 } }
    pub fn next(current: Option<Self>) -> Option<Self> {
        match current { None => Some(Self::Common), Some(Self::Common) => Some(Self::Rare), Some(Self::Rare) => Some(Self::Special), Some(Self::Special) => None }
    }
}
/// Array order is Common, Rare, Special; input order cannot affect the row.
pub fn fusion_probabilities(a: RelicRewardGrade, b: RelicRewardGrade) -> [u32; 3] {
    use RelicRewardGrade::*;
    match (a.min(b), a.max(b)) {
        (Common, Common) => [80,19,1], (Common, Rare) => [40,55,5],
        (Common, Special) => [0,70,30], (Rare, Rare) => [0,80,20],
        (Rare, Special) => [0,40,60], (Special, Special) => [0,0,100],
        _ => unreachable!("ordered grade pair"),
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicRewardCandidate { pub relic_id: String, pub weight: u32 }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicRelicRewardPolicy {
    pub candidates: Vec<RelicRewardCandidate>,
    /// Cumulative costs, not the amount paid at each step.
    pub dust_thresholds: [u32; 3],
}
impl Default for BasicRelicRewardPolicy {
    fn default() -> Self { Self { candidates: vec![], dust_thresholds: [3,9,18] } }
}
impl BasicRelicRewardPolicy {
    pub fn validate(&self, database: &RelicDatabase) -> Result<(), String> {
        if self.dust_thresholds[0] == 0 || !self.dust_thresholds.windows(2).all(|w| w[0] < w[1]) {
            return Err("relic reward dust thresholds must be positive and strictly increasing".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut total = 0u32;
        for candidate in &self.candidates {
            crate::authoring::control::charge(database.relics.len() as u64 + 1, "relic_reward_authoring")
                .map_err(|e| format!("{e:?}"))?;
            if candidate.weight == 0 || !seen.insert(&candidate.relic_id) { return Err("zero weight or duplicate relic reward candidate".into()); }
            total = total.checked_add(candidate.weight).ok_or("relic reward weight sum overflow")?;
            let definition = database.get_by_id(&candidate.relic_id).ok_or_else(|| format!("unknown relic reward candidate '{}'", candidate.relic_id))?;
            if RelicRewardGrade::from_rarity(&definition.rarity).is_none() { return Err(format!("unsupported relic reward grade '{}'", definition.rarity)); }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicRewardSession {
    pub kind: RewardActivityKind,
    pub investment_grade: Option<RelicRewardGrade>,
    pub paid_dust: u32,
    pub fusion_inputs: Option<[Uuid; 2]>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicRewardBlockedReason { NoCandidates, InsufficientDust, MaximumGrade, SelectTwoCopies, MissingCopy, EquippedCopy, SameCopy, UnsupportedGrade, MissingOutcomeGrade }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelicRewardEligibility {
    pub enabled: bool,
    pub disabled_reason: Option<RelicRewardBlockedReason>,
}
impl RelicRewardEligibility {
    pub fn new(reason: Option<RelicRewardBlockedReason>) -> Self { Self { enabled: reason.is_none(), disabled_reason: reason } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicInvestmentPreview {
    pub next_grade: Option<RelicRewardGrade>,
    pub additional_dust: Option<u32>,
    pub eligibility: RelicRewardEligibility,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicFusionPreview {
    pub inputs: Option<[Uuid; 2]>,
    /// Common/Rare/Special percentages. Absent until two valid inputs are selected.
    pub probabilities: Option<[u32; 3]>,
    pub eligibility: RelicRewardEligibility,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicFusionInputView {
    pub copy: OwnedRelic,
    pub grade: Option<RelicRewardGrade>,
    pub eligibility: RelicRewardEligibility,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicRewardView {
    pub kind: RewardActivityKind,
    pub investment_grade: Option<RelicRewardGrade>,
    pub paid_dust: u32,
    pub equipment_dust: u32,
    /// Only currently offered definitions/effects; never unpaid future choices.
    pub candidates: Vec<RelicDefinition>,
    pub no_candidates_reason: Option<RelicRewardBlockedReason>,
    pub investment: Option<RelicInvestmentPreview>,
    pub fusion: Option<RelicFusionPreview>,
    pub fusion_inputs: Vec<RelicFusionInputView>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicRewardEntryView {
    pub kind: RewardActivityKind,
    pub dust_thresholds: Option<[u32; 3]>,
    pub requires_two_distinct_unequipped_copies: bool,
    pub corrupt_inputs_allowed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicRewardReceipt {
    pub node_id: MapNodeId,
    pub kind: RewardActivityKind,
    pub granted: Vec<OwnedRelic>,
    pub consumed_copies: Vec<Uuid>,
    pub paid_dust: u32,
    pub equipment_dust: u32,
}
crate::authoring::clone_shape::copy_shape!(RelicRewardGrade);
crate::authoring::clone_shape::copy_shape!(RelicRewardSession);
