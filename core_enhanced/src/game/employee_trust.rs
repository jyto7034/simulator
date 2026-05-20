use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeTrait {
    Brave,
    Cautious,
    Obedient,
    Defiant,
    Comradely,
    SurvivalInstinct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBand {
    Distrust,
    Uneasy,
    Neutral,
    Trusting,
    Devoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustMemoryKind {
    TreatedAfterIncapacitation,
    RestedBeforeDanger,
    PromiseKept,
    ConsistentInvestment,
    DeployedWhileInjured,
    IncapacitatedNeglected,
    ForcedEarlyAwakening,
    ForcedRiskFragmentUse,
    AllyDeathThenDanger,
    RestedAtRecoveryNode,
    NeglectedAtRecoveryNode,
    AllyDeathWitnessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustMemory {
    pub kind: TrustMemoryKind,
    pub intensity: u8,
    pub remaining_nodes: Option<u8>,
}

impl TrustMemory {
    pub fn new(kind: TrustMemoryKind, intensity: u8) -> Self {
        Self {
            kind,
            intensity,
            remaining_nodes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmployeeTrustState {
    pub score: i16,
    pub traits: Vec<EmployeeTrait>,
    pub memories: Vec<TrustMemory>,
    pub recent_reactions: Vec<TrustReactionSummary>,
}

impl Default for EmployeeTrustState {
    fn default() -> Self {
        Self {
            score: 0,
            traits: vec![EmployeeTrait::Cautious],
            memories: Vec::new(),
            recent_reactions: Vec::new(),
        }
    }
}

impl EmployeeTrustState {
    pub const MIN_SCORE: i16 = -100;
    pub const MAX_SCORE: i16 = 100;
    pub const RECENT_REACTION_LIMIT: usize = 8;

    pub fn new_for_employee(employee_id: Uuid) -> Self {
        let primary = match employee_id.as_u128() % 6 {
            0 => EmployeeTrait::Brave,
            1 => EmployeeTrait::Cautious,
            2 => EmployeeTrait::Obedient,
            3 => EmployeeTrait::Defiant,
            4 => EmployeeTrait::Comradely,
            _ => EmployeeTrait::SurvivalInstinct,
        };
        Self {
            traits: vec![primary],
            ..Self::default()
        }
    }

    pub fn band(&self) -> TrustBand {
        match self.score {
            i16::MIN..=-51 => TrustBand::Distrust,
            -50..=-11 => TrustBand::Uneasy,
            -10..=25 => TrustBand::Neutral,
            26..=70 => TrustBand::Trusting,
            71..=i16::MAX => TrustBand::Devoted,
        }
    }

    pub fn apply_reaction(&mut self, reaction: &TrustReaction) {
        self.score = (self.score + reaction.trust_delta).clamp(Self::MIN_SCORE, Self::MAX_SCORE);
        self.memories
            .extend(reaction.memory_updates.iter().cloned());
        if let Some(summary) = TrustReactionSummary::from_reaction(reaction) {
            self.recent_reactions.push(summary);
            let overflow = self
                .recent_reactions
                .len()
                .saturating_sub(Self::RECENT_REACTION_LIMIT);
            if overflow > 0 {
                self.recent_reactions.drain(0..overflow);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustReactionSummary {
    pub event: TrustEventKind,
    pub trust_delta: i16,
    pub cue_count: usize,
    pub combat_modifier_count: usize,
    pub trauma_modifier_count: usize,
}

impl TrustReactionSummary {
    fn from_reaction(reaction: &TrustReaction) -> Option<Self> {
        if reaction.is_empty() {
            return None;
        }
        Some(Self {
            event: reaction.event,
            trust_delta: reaction.trust_delta,
            cue_count: reaction.cues.len(),
            combat_modifier_count: reaction.combat_modifiers.len(),
            trauma_modifier_count: reaction.trauma_modifiers.len(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustEventKind {
    TreatedAfterIncapacitation,
    RestedBeforeDanger,
    PromiseKept,
    ConsistentInvestment,
    DeployedWhileInjured,
    IncapacitatedNeglected,
    ForcedEarlyAwakening,
    ForcedRiskFragmentUse,
    AllyDeathThenDanger,
    RestedAtRecoveryNode,
    NeglectedAtRecoveryNode,
    AllyDied,
    BeforeDangerNode,
    BeforeFinalNode,
    IncurredTrauma,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustEvent {
    pub employee_id: Uuid,
    pub kind: TrustEventKind,
    pub intensity: u8,
}

impl TrustEvent {
    pub fn new(employee_id: Uuid, kind: TrustEventKind) -> Self {
        Self {
            employee_id,
            kind,
            intensity: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighRiskChoiceKind {
    EarlyAwakening,
    RiskFragmentUse,
    InjuredRedeployment,
    TraumaRedeployment,
    AllyDeathThenDangerNode,
    FinalNodeDeployment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskChoice {
    pub employee_id: Uuid,
    pub kind: HighRiskChoiceKind,
    pub risk_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighRiskDecision {
    Accepted,
    AcceptedWithWarning,
    RequiresAdditionalCost,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustCueKind {
    Dialogue,
    Caption,
    FacialExpression,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustCue {
    pub kind: TrustCueKind,
    pub message_key: String,
}

impl TrustCue {
    pub fn new(kind: TrustCueKind, message_key: impl Into<String>) -> Self {
        Self {
            kind,
            message_key: message_key.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustCombatModifierKind {
    Resolve,
    Revenge,
    Focus,
    BreakdownResistance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustCombatModifier {
    pub kind: TrustCombatModifierKind,
    pub magnitude: i16,
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustTraumaModifier {
    FlatReduction(u32),
    PercentReduction(u8),
    ResistBreakdownChance { percent: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustReaction {
    pub employee_id: Uuid,
    pub event: TrustEventKind,
    pub trust_delta: i16,
    pub memory_updates: Vec<TrustMemory>,
    pub cues: Vec<TrustCue>,
    pub high_risk_decision: Option<HighRiskDecision>,
    pub trauma_modifiers: Vec<TrustTraumaModifier>,
    pub combat_modifiers: Vec<TrustCombatModifier>,
}

impl TrustReaction {
    pub fn empty(employee_id: Uuid, event: TrustEventKind) -> Self {
        Self {
            employee_id,
            event,
            trust_delta: 0,
            memory_updates: Vec::new(),
            cues: Vec::new(),
            high_risk_decision: None,
            trauma_modifiers: Vec::new(),
            combat_modifiers: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.trust_delta == 0
            && self.memory_updates.is_empty()
            && self.cues.is_empty()
            && self.high_risk_decision.is_none()
            && self.trauma_modifiers.is_empty()
            && self.combat_modifiers.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustFeatureFlags {
    pub memory_enabled: bool,
    pub dialogue_enabled: bool,
    pub high_risk_acceptance_enabled: bool,
    pub trauma_modifier_enabled: bool,
    pub pre_battle_modifier_enabled: bool,
    pub ally_death_reaction_enabled: bool,
    pub combat_modifier_enabled: bool,
}

impl TrustFeatureFlags {
    pub fn all_disabled() -> Self {
        Self {
            memory_enabled: false,
            dialogue_enabled: false,
            high_risk_acceptance_enabled: false,
            trauma_modifier_enabled: false,
            pre_battle_modifier_enabled: false,
            ally_death_reaction_enabled: false,
            combat_modifier_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustThresholds {
    pub uneasy_max: i16,
    pub trusting_min: i16,
    pub devoted_min: i16,
    pub high_risk_warning_max: i16,
    pub high_risk_refusal_max: i16,
}

impl Default for TrustThresholds {
    fn default() -> Self {
        Self {
            uneasy_max: -11,
            trusting_min: 26,
            devoted_min: 71,
            high_risk_warning_max: 25,
            high_risk_refusal_max: -51,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmployeeTrustPolicy {
    pub enabled: bool,
    pub features: TrustFeatureFlags,
    pub thresholds: TrustThresholds,
    pub high_trust_trauma_reduction_percent: u8,
    pub devoted_trauma_reduction_percent: u8,
}

impl EmployeeTrustPolicy {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn narrative_only() -> Self {
        Self {
            enabled: true,
            features: TrustFeatureFlags {
                memory_enabled: true,
                dialogue_enabled: true,
                high_risk_acceptance_enabled: false,
                trauma_modifier_enabled: false,
                pre_battle_modifier_enabled: false,
                ally_death_reaction_enabled: false,
                combat_modifier_enabled: false,
            },
            ..Self::default()
        }
    }
}

impl Default for EmployeeTrustPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            features: TrustFeatureFlags::all_disabled(),
            thresholds: TrustThresholds::default(),
            high_trust_trauma_reduction_percent: 10,
            devoted_trauma_reduction_percent: 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustTraumaOutcome {
    pub base_amount: u32,
    pub final_amount: u32,
    pub reaction: TrustReaction,
}

pub struct EmployeeTrustResolver;

impl EmployeeTrustResolver {
    pub fn apply_event(event: TrustEvent, policy: &EmployeeTrustPolicy) -> TrustReaction {
        if !policy.enabled {
            return TrustReaction::empty(event.employee_id, event.kind);
        }

        let mut reaction = TrustReaction::empty(event.employee_id, event.kind);
        if policy.features.memory_enabled {
            reaction.memory_updates.push(TrustMemory::new(
                memory_kind_for_event(event.kind),
                event.intensity,
            ));
            reaction.trust_delta = trust_delta_for_event(event.kind) * i16::from(event.intensity);
        }
        if policy.features.dialogue_enabled {
            if let Some(cue_key) = cue_key_for_event(event.kind) {
                reaction
                    .cues
                    .push(TrustCue::new(TrustCueKind::Dialogue, cue_key));
            }
        }
        reaction
    }

    pub fn evaluate_high_risk_choice(
        state: &EmployeeTrustState,
        choice: HighRiskChoice,
        policy: &EmployeeTrustPolicy,
    ) -> TrustReaction {
        let mut reaction =
            TrustReaction::empty(choice.employee_id, event_kind_for_choice(choice.kind));
        if !policy.enabled || !policy.features.high_risk_acceptance_enabled {
            reaction.high_risk_decision = Some(HighRiskDecision::Accepted);
            return reaction;
        }

        reaction.high_risk_decision = Some(
            if state.score <= policy.thresholds.high_risk_refusal_max && choice.risk_level >= 2 {
                HighRiskDecision::Refused
            } else if state.score <= policy.thresholds.high_risk_warning_max {
                HighRiskDecision::AcceptedWithWarning
            } else {
                HighRiskDecision::Accepted
            },
        );

        if policy.features.dialogue_enabled {
            let cue = match reaction.high_risk_decision {
                Some(HighRiskDecision::Refused) => "trust.high_risk.refused",
                Some(HighRiskDecision::AcceptedWithWarning) => "trust.high_risk.warning",
                Some(HighRiskDecision::RequiresAdditionalCost) => "trust.high_risk.cost",
                Some(HighRiskDecision::Accepted) | None => "trust.high_risk.accepted",
            };
            reaction
                .cues
                .push(TrustCue::new(TrustCueKind::Warning, cue));
        }
        reaction
    }

    pub fn modify_trauma(
        state: &EmployeeTrustState,
        employee_id: Uuid,
        base_amount: u32,
        policy: &EmployeeTrustPolicy,
    ) -> TrustTraumaOutcome {
        let mut reaction = TrustReaction::empty(employee_id, TrustEventKind::IncurredTrauma);
        if !policy.enabled || !policy.features.trauma_modifier_enabled {
            return TrustTraumaOutcome {
                base_amount,
                final_amount: base_amount,
                reaction,
            };
        }

        let reduction_percent = match state.band() {
            TrustBand::Devoted => policy.devoted_trauma_reduction_percent,
            TrustBand::Trusting => policy.high_trust_trauma_reduction_percent,
            TrustBand::Distrust | TrustBand::Uneasy | TrustBand::Neutral => 0,
        };
        if reduction_percent == 0 {
            return TrustTraumaOutcome {
                base_amount,
                final_amount: base_amount,
                reaction,
            };
        }

        reaction
            .trauma_modifiers
            .push(TrustTraumaModifier::PercentReduction(reduction_percent));
        let reduction = base_amount.saturating_mul(u32::from(reduction_percent)) / 100;
        TrustTraumaOutcome {
            base_amount,
            final_amount: base_amount.saturating_sub(reduction),
            reaction,
        }
    }
}

fn trust_delta_for_event(kind: TrustEventKind) -> i16 {
    match kind {
        TrustEventKind::TreatedAfterIncapacitation => 2,
        TrustEventKind::RestedBeforeDanger
        | TrustEventKind::PromiseKept
        | TrustEventKind::ConsistentInvestment
        | TrustEventKind::RestedAtRecoveryNode => 8,
        TrustEventKind::DeployedWhileInjured
        | TrustEventKind::IncapacitatedNeglected
        | TrustEventKind::ForcedEarlyAwakening
        | TrustEventKind::ForcedRiskFragmentUse
        | TrustEventKind::AllyDeathThenDanger
        | TrustEventKind::NeglectedAtRecoveryNode => -10,
        TrustEventKind::AllyDied
        | TrustEventKind::BeforeDangerNode
        | TrustEventKind::BeforeFinalNode
        | TrustEventKind::IncurredTrauma => 0,
    }
}

fn memory_kind_for_event(kind: TrustEventKind) -> TrustMemoryKind {
    match kind {
        TrustEventKind::TreatedAfterIncapacitation => TrustMemoryKind::TreatedAfterIncapacitation,
        TrustEventKind::RestedBeforeDanger => TrustMemoryKind::RestedBeforeDanger,
        TrustEventKind::PromiseKept => TrustMemoryKind::PromiseKept,
        TrustEventKind::ConsistentInvestment => TrustMemoryKind::ConsistentInvestment,
        TrustEventKind::DeployedWhileInjured => TrustMemoryKind::DeployedWhileInjured,
        TrustEventKind::IncapacitatedNeglected => TrustMemoryKind::IncapacitatedNeglected,
        TrustEventKind::ForcedEarlyAwakening => TrustMemoryKind::ForcedEarlyAwakening,
        TrustEventKind::ForcedRiskFragmentUse => TrustMemoryKind::ForcedRiskFragmentUse,
        TrustEventKind::AllyDeathThenDanger => TrustMemoryKind::AllyDeathThenDanger,
        TrustEventKind::RestedAtRecoveryNode => TrustMemoryKind::RestedAtRecoveryNode,
        TrustEventKind::NeglectedAtRecoveryNode => TrustMemoryKind::NeglectedAtRecoveryNode,
        TrustEventKind::AllyDied => TrustMemoryKind::AllyDeathWitnessed,
        TrustEventKind::BeforeDangerNode
        | TrustEventKind::BeforeFinalNode
        | TrustEventKind::IncurredTrauma => TrustMemoryKind::ConsistentInvestment,
    }
}

fn cue_key_for_event(kind: TrustEventKind) -> Option<&'static str> {
    match kind {
        TrustEventKind::TreatedAfterIncapacitation => {
            Some("trust.memory.treated_after_incapacitation")
        }
        TrustEventKind::RestedBeforeDanger => Some("trust.memory.rested_before_danger"),
        TrustEventKind::PromiseKept => Some("trust.memory.promise_kept"),
        TrustEventKind::DeployedWhileInjured => Some("trust.memory.deployed_while_injured"),
        TrustEventKind::ForcedEarlyAwakening => Some("trust.memory.forced_early_awakening"),
        TrustEventKind::ForcedRiskFragmentUse => Some("trust.memory.forced_risk_fragment"),
        TrustEventKind::AllyDeathThenDanger => Some("trust.memory.ally_death_then_danger"),
        TrustEventKind::NeglectedAtRecoveryNode => Some("trust.memory.neglected_at_recovery"),
        _ => None,
    }
}

fn event_kind_for_choice(kind: HighRiskChoiceKind) -> TrustEventKind {
    match kind {
        HighRiskChoiceKind::EarlyAwakening => TrustEventKind::ForcedEarlyAwakening,
        HighRiskChoiceKind::RiskFragmentUse => TrustEventKind::ForcedRiskFragmentUse,
        HighRiskChoiceKind::InjuredRedeployment => TrustEventKind::DeployedWhileInjured,
        HighRiskChoiceKind::TraumaRedeployment => TrustEventKind::IncurredTrauma,
        HighRiskChoiceKind::AllyDeathThenDangerNode => TrustEventKind::AllyDeathThenDanger,
        HighRiskChoiceKind::FinalNodeDeployment => TrustEventKind::BeforeFinalNode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_policy_does_not_change_event_state() {
        let employee_id = Uuid::from_u128(1);
        let mut state = EmployeeTrustState::default();
        let reaction = EmployeeTrustResolver::apply_event(
            TrustEvent::new(employee_id, TrustEventKind::TreatedAfterIncapacitation),
            &EmployeeTrustPolicy::disabled(),
        );

        state.apply_reaction(&reaction);

        assert_eq!(state.score, 0);
        assert!(state.memories.is_empty());
        assert!(state.recent_reactions.is_empty());
    }

    #[test]
    fn narrative_policy_records_memory_and_reaction() {
        let employee_id = Uuid::from_u128(2);
        let mut state = EmployeeTrustState::default();
        let reaction = EmployeeTrustResolver::apply_event(
            TrustEvent::new(employee_id, TrustEventKind::TreatedAfterIncapacitation),
            &EmployeeTrustPolicy::narrative_only(),
        );

        state.apply_reaction(&reaction);

        assert!(state.score > 0);
        assert_eq!(state.memories.len(), 1);
        assert_eq!(state.recent_reactions.len(), 1);
        assert_eq!(reaction.cues.len(), 1);
    }

    #[test]
    fn employee_seed_assigns_stable_trait() {
        let first = EmployeeTrustState::new_for_employee(Uuid::from_u128(1));
        let second = EmployeeTrustState::new_for_employee(Uuid::from_u128(2));
        let first_again = EmployeeTrustState::new_for_employee(Uuid::from_u128(1));

        assert_ne!(first.traits, second.traits);
        assert_eq!(first.traits, first_again.traits);
    }

    #[test]
    fn disabled_high_risk_policy_always_accepts() {
        let employee_id = Uuid::from_u128(3);
        let state = EmployeeTrustState {
            score: -100,
            ..EmployeeTrustState::default()
        };
        let reaction = EmployeeTrustResolver::evaluate_high_risk_choice(
            &state,
            HighRiskChoice {
                employee_id,
                kind: HighRiskChoiceKind::EarlyAwakening,
                risk_level: 3,
            },
            &EmployeeTrustPolicy::disabled(),
        );

        assert_eq!(
            reaction.high_risk_decision,
            Some(HighRiskDecision::Accepted)
        );
        assert!(reaction.cues.is_empty());
    }

    #[test]
    fn enabled_trauma_modifier_reduces_trauma_for_high_trust() {
        let employee_id = Uuid::from_u128(4);
        let state = EmployeeTrustState {
            score: 80,
            ..EmployeeTrustState::default()
        };
        let mut policy = EmployeeTrustPolicy::default();
        policy.enabled = true;
        policy.features.trauma_modifier_enabled = true;

        let outcome = EmployeeTrustResolver::modify_trauma(&state, employee_id, 50, &policy);

        assert_eq!(outcome.final_amount, 40);
        assert_eq!(outcome.reaction.trauma_modifiers.len(), 1);
    }
}
