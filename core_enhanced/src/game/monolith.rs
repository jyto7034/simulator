use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::game::map::MapNodeId;

pub const MONOLITH_FINAL_STAGE: u8 = 3;
pub const MONOLITH_REMANIFESTATION_MAX_ATTEMPTS: u8 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaturalEliteOutcome {
    #[default]
    Unresolved,
    Victory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithProgress {
    pub natural_outcome: NaturalEliteOutcome,
    pub victories: u8,
    pub purchased: bool,
    pub uses_remaining: u8,
    pub active_node_id: Option<MapNodeId>,
    pub remanifestation_failed: bool,
    pub residual_created: bool,
}

impl Default for MonolithProgress {
    fn default() -> Self {
        Self {
            natural_outcome: NaturalEliteOutcome::Unresolved,
            victories: 0,
            purchased: false,
            uses_remaining: 0,
            active_node_id: None,
            remanifestation_failed: false,
            residual_created: false,
        }
    }
}

impl MonolithProgress {
    pub fn is_naturally_excluded(&self) -> bool {
        self.natural_outcome != NaturalEliteOutcome::Unresolved
    }

    pub fn is_offer_eligible(&self) -> bool {
        self.natural_outcome == NaturalEliteOutcome::Victory
            && !self.purchased
            && !self.remanifestation_failed
            && self.victories < MONOLITH_FINAL_STAGE
    }

    pub fn next_stage(&self) -> Option<u8> {
        (self.purchased && !self.remanifestation_failed)
            .then_some(self.victories.saturating_add(1))
            .filter(|stage| (2..=MONOLITH_FINAL_STAGE).contains(stage))
    }

    pub fn is_usable(&self) -> bool {
        self.next_stage().is_some() && self.uses_remaining > 0 && self.active_node_id.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithRemanifestationNode {
    pub abnormality_id: String,
    pub encounter_id: String,
    pub stage: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithResidual {
    pub sequence: u64,
    pub source_abnormality_id: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonolithExchangeState {
    #[default]
    Unavailable,
    Available,
    Started,
    Accepted,
    Declined,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithRunState {
    pub progress_by_abnormality: BTreeMap<String, MonolithProgress>,
    pub remanifestation_nodes: HashMap<MapNodeId, MonolithRemanifestationNode>,
    pub residuals: Vec<MonolithResidual>,
    pub next_residual_sequence: u64,
    pub exchange_state: MonolithExchangeState,
    pub exchange_shop_guarantee_issued: bool,
    pub next_floor_shop_guarantee_pending: bool,
    pub corrupted_monolith_acquired: bool,
}

impl crate::authoring::clone_shape::CloneShape for MonolithRunState {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::authoring::control;
        let Self {
            progress_by_abnormality,
            remanifestation_nodes,
            residuals,
            next_residual_sequence,
            exchange_state,
            exchange_shop_guarantee_issued,
            next_floor_shop_guarantee_pending,
            corrupted_monolith_acquired,
        } = self;
        meter.scan_storage(Some(1))?;
        progress_by_abnormality.visit_clone_shape(meter)?;
        // The candidate-owned lifetime insertion/import credit survives native
        // copies, removals, drains and inner rollback. Each independent map
        // contributes the full conservative bound before its bucket scan.
        meter
            .scan_storage(control::monolith_clone_bucket_bound().and_then(|n| n.checked_add(1)))?;
        for (key, value) in remanifestation_nodes {
            key.visit_clone_shape(meter)?;
            value.visit_clone_shape(meter)?;
        }
        residuals.visit_clone_shape(meter)?;
        next_residual_sequence.visit_clone_shape(meter)?;
        exchange_state.visit_clone_shape(meter)?;
        exchange_shop_guarantee_issued.visit_clone_shape(meter)?;
        next_floor_shop_guarantee_pending.visit_clone_shape(meter)?;
        corrupted_monolith_acquired.visit_clone_shape(meter)
    }
}
crate::authoring::clone_shape::copy_shape!(NaturalEliteOutcome, MonolithExchangeState);
crate::authoring::clone_shape::struct_shape!(MonolithProgress {
    natural_outcome,
    victories,
    purchased,
    uses_remaining,
    active_node_id,
    remanifestation_failed,
    residual_created
});
crate::authoring::clone_shape::struct_shape!(MonolithRemanifestationNode {
    abnormality_id,
    encounter_id,
    stage
});
crate::authoring::clone_shape::struct_shape!(MonolithResidual {
    sequence,
    source_abnormality_id
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonolithStateError {
    InvalidAbnormality,
    NaturalEliteAlreadyResolved,
    NotOfferEligible,
    AlreadyPurchased,
    NotUsable,
    NodeAlreadyConverted,
    InvalidStage,
    ExchangeUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonolithVictoryResolution {
    pub stage: u8,
    pub residual_created: bool,
}

impl MonolithRunState {
    pub fn progress(&self, abnormality_id: &str) -> Option<&MonolithProgress> {
        self.progress_by_abnormality.get(abnormality_id)
    }

    pub fn remanifestation(&self, node_id: MapNodeId) -> Option<&MonolithRemanifestationNode> {
        self.remanifestation_nodes.get(&node_id)
    }

    pub fn record_natural_victory(
        &mut self,
        abnormality_id: &str,
    ) -> Result<(), MonolithStateError> {
        let progress = self.progress_mut(abnormality_id)?;
        if progress.natural_outcome != NaturalEliteOutcome::Unresolved {
            return Err(MonolithStateError::NaturalEliteAlreadyResolved);
        }
        progress.natural_outcome = NaturalEliteOutcome::Victory;
        progress.victories = 1;
        Ok(())
    }

    pub fn purchase(
        &mut self,
        abnormality_id: &str,
        initial_uses: u8,
    ) -> Result<(), MonolithStateError> {
        if initial_uses == 0 {
            return Err(MonolithStateError::NotOfferEligible);
        }
        let progress = self.progress_mut(abnormality_id)?;
        if progress.purchased {
            return Err(MonolithStateError::AlreadyPurchased);
        }
        if !progress.is_offer_eligible() {
            return Err(MonolithStateError::NotOfferEligible);
        }
        progress.purchased = true;
        progress.uses_remaining = initial_uses;
        Ok(())
    }

    pub fn begin_remanifestation(
        &mut self,
        node_id: MapNodeId,
        abnormality_id: &str,
        encounter_id: &str,
    ) -> Result<MonolithRemanifestationNode, MonolithStateError> {
        if encounter_id.trim().is_empty() || abnormality_id.trim().is_empty() {
            return Err(MonolithStateError::InvalidAbnormality);
        }
        if self.remanifestation_nodes.contains_key(&node_id) {
            return Err(MonolithStateError::NodeAlreadyConverted);
        }
        let progress = self.progress_mut(abnormality_id)?;
        if !progress.is_usable() {
            return Err(MonolithStateError::NotUsable);
        }
        let stage = progress
            .next_stage()
            .ok_or(MonolithStateError::InvalidStage)?;
        progress.uses_remaining = progress.uses_remaining.saturating_sub(1);
        progress.active_node_id = Some(node_id);
        let node = MonolithRemanifestationNode {
            abnormality_id: abnormality_id.to_string(),
            encounter_id: encounter_id.to_string(),
            stage,
        };
        crate::authoring::control::monolith_inserted();
        self.remanifestation_nodes.insert(node_id, node.clone());
        Ok(node)
    }

    pub fn resolve_remanifestation_victory(
        &mut self,
        node_id: MapNodeId,
        residual_threshold: usize,
    ) -> Result<MonolithVictoryResolution, MonolithStateError> {
        let node = self
            .remanifestation_nodes
            .get(&node_id)
            .cloned()
            .ok_or(MonolithStateError::NotUsable)?;
        let progress = self
            .progress_by_abnormality
            .get(&node.abnormality_id)
            .ok_or(MonolithStateError::InvalidAbnormality)?;
        if progress.active_node_id != Some(node_id)
            || node.stage != progress.victories.saturating_add(1)
        {
            return Err(MonolithStateError::InvalidStage);
        }
        self.remanifestation_nodes.remove(&node_id);
        let progress = self
            .progress_by_abnormality
            .get_mut(&node.abnormality_id)
            .ok_or(MonolithStateError::InvalidAbnormality)?;
        progress.active_node_id = None;
        progress.victories = node.stage;

        let residual_created = node.stage == MONOLITH_FINAL_STAGE
            && progress.uses_remaining == 1
            && !progress.residual_created;
        if residual_created {
            progress.residual_created = true;
            progress.uses_remaining = 0;
            let sequence = self.next_residual_sequence;
            self.next_residual_sequence = self.next_residual_sequence.saturating_add(1);
            self.residuals.push(MonolithResidual {
                sequence,
                source_abnormality_id: node.abnormality_id,
            });
            if residual_threshold > 0 && self.residuals.len() >= residual_threshold {
                if self.exchange_state == MonolithExchangeState::Unavailable {
                    self.exchange_state = MonolithExchangeState::Available;
                }
                if !self.exchange_shop_guarantee_issued {
                    self.exchange_shop_guarantee_issued = true;
                    self.next_floor_shop_guarantee_pending = true;
                }
            }
        }

        Ok(MonolithVictoryResolution {
            stage: node.stage,
            residual_created,
        })
    }

    pub fn resolve_remanifestation_failure(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<(), MonolithStateError> {
        let node = self
            .remanifestation_nodes
            .get(&node_id)
            .cloned()
            .ok_or(MonolithStateError::NotUsable)?;
        let progress = self
            .progress_by_abnormality
            .get(&node.abnormality_id)
            .ok_or(MonolithStateError::InvalidAbnormality)?;
        if progress.active_node_id != Some(node_id) {
            return Err(MonolithStateError::InvalidStage);
        }
        self.remanifestation_nodes.remove(&node_id);
        let progress = self
            .progress_by_abnormality
            .get_mut(&node.abnormality_id)
            .ok_or(MonolithStateError::InvalidAbnormality)?;
        progress.active_node_id = None;
        progress.uses_remaining = 0;
        progress.remanifestation_failed = true;
        Ok(())
    }

    pub fn consume_next_floor_shop_guarantee(&mut self) -> bool {
        std::mem::take(&mut self.next_floor_shop_guarantee_pending)
    }

    pub fn clear_floor_local_remanifestations(&mut self) {
        for (node_id, node) in self.remanifestation_nodes.drain() {
            if let Some(progress) = self.progress_by_abnormality.get_mut(&node.abnormality_id) {
                if progress.active_node_id == Some(node_id) {
                    progress.active_node_id = None;
                }
            }
        }
    }

    pub fn validate_checkpoint_state(
        &self,
        initial_uses: u8,
        residual_threshold: usize,
        node_exists: impl Fn(MapNodeId) -> bool,
        abnormality_exists: impl Fn(&str) -> bool,
        encounter_matches: impl Fn(&str, &str) -> bool,
    ) -> Result<(), String> {
        for (abnormality_id, progress) in &self.progress_by_abnormality {
            if abnormality_id.trim().is_empty() || !abnormality_exists(abnormality_id) {
                return Err(format!(
                    "monolith progress references missing abnormality '{abnormality_id}'"
                ));
            }
            if progress.victories > MONOLITH_FINAL_STAGE
                || progress.uses_remaining > initial_uses
                || (progress.natural_outcome == NaturalEliteOutcome::Unresolved
                    && (progress.victories != 0 || progress.purchased))
                || (progress.natural_outcome == NaturalEliteOutcome::Victory
                    && progress.victories == 0)
                || (progress.purchased && progress.natural_outcome != NaturalEliteOutcome::Victory)
                || (progress.victories == MONOLITH_FINAL_STAGE
                    && (progress.uses_remaining != 0 || progress.active_node_id.is_some()))
                || (progress.remanifestation_failed
                    && (!progress.purchased
                        || progress.natural_outcome != NaturalEliteOutcome::Victory
                        || progress.victories == 0
                        || progress.victories >= MONOLITH_FINAL_STAGE
                        || progress.uses_remaining != 0
                        || progress.active_node_id.is_some()
                        || progress.residual_created))
                || (progress.residual_created && progress.victories != MONOLITH_FINAL_STAGE)
            {
                return Err(format!(
                    "monolith progress for '{abnormality_id}' violates stage/use/outcome invariants"
                ));
            }
            match progress.active_node_id {
                Some(node_id)
                    if !self
                        .remanifestation_nodes
                        .get(&node_id)
                        .is_some_and(|node| {
                            node.abnormality_id == *abnormality_id
                                && node.stage == progress.victories.saturating_add(1)
                        }) =>
                {
                    return Err(format!(
                        "monolith progress for '{abnormality_id}' has no matching active node"
                    ));
                }
                _ => {}
            }
        }
        for (node_id, node) in &self.remanifestation_nodes {
            if !node_exists(*node_id)
                || !(2..=MONOLITH_FINAL_STAGE).contains(&node.stage)
                || !encounter_matches(&node.encounter_id, &node.abnormality_id)
                || !self
                    .progress_by_abnormality
                    .get(&node.abnormality_id)
                    .is_some_and(|progress| {
                        progress.active_node_id == Some(*node_id)
                            && progress.purchased
                            && node.stage == progress.victories.saturating_add(1)
                    })
            {
                return Err(format!(
                    "monolith remanifestation node '{}' is inconsistent",
                    node_id.0
                ));
            }
        }
        let mut previous_sequence = None;
        let mut residual_sources = std::collections::HashSet::new();
        for residual in &self.residuals {
            if previous_sequence.is_some_and(|previous| residual.sequence <= previous)
                || !residual_sources.insert(residual.source_abnormality_id.as_str())
                || !self
                    .progress_by_abnormality
                    .get(&residual.source_abnormality_id)
                    .is_some_and(|progress| progress.residual_created)
            {
                return Err("monolith residual ledger is inconsistent".to_string());
            }
            previous_sequence = Some(residual.sequence);
        }
        if previous_sequence.is_some_and(|sequence| self.next_residual_sequence <= sequence)
            || (self.next_floor_shop_guarantee_pending && !self.exchange_shop_guarantee_issued)
            || (self.exchange_shop_guarantee_issued
                != (self.exchange_state != MonolithExchangeState::Unavailable))
            || (self.exchange_state == MonolithExchangeState::Unavailable
                && self.residuals.len() >= residual_threshold)
            || (self.corrupted_monolith_acquired
                != (self.exchange_state == MonolithExchangeState::Accepted))
            || (matches!(
                self.exchange_state,
                MonolithExchangeState::Available
                    | MonolithExchangeState::Started
                    | MonolithExchangeState::Declined
            ) && self.residuals.len() < residual_threshold)
        {
            return Err("monolith exchange state is inconsistent".to_string());
        }
        Ok(())
    }

    pub fn start_exchange(&mut self, residual_threshold: usize) -> Result<(), MonolithStateError> {
        if self.exchange_state != MonolithExchangeState::Available
            || residual_threshold == 0
            || self.residuals.len() < residual_threshold
        {
            return Err(MonolithStateError::ExchangeUnavailable);
        }
        self.exchange_state = MonolithExchangeState::Started;
        Ok(())
    }

    pub fn decline_exchange(&mut self) -> Result<(), MonolithStateError> {
        if self.exchange_state != MonolithExchangeState::Started {
            return Err(MonolithStateError::ExchangeUnavailable);
        }
        self.exchange_state = MonolithExchangeState::Declined;
        Ok(())
    }

    pub fn accept_exchange(
        &mut self,
        residual_threshold: usize,
    ) -> Result<Vec<MonolithResidual>, MonolithStateError> {
        if self.exchange_state != MonolithExchangeState::Started
            || residual_threshold == 0
            || self.residuals.len() < residual_threshold
        {
            return Err(MonolithStateError::ExchangeUnavailable);
        }
        self.residuals.sort_by_key(|residual| residual.sequence);
        let consumed = self.residuals.drain(0..residual_threshold).collect();
        self.exchange_state = MonolithExchangeState::Accepted;
        self.corrupted_monolith_acquired = true;
        Ok(consumed)
    }

    fn progress_mut(
        &mut self,
        abnormality_id: &str,
    ) -> Result<&mut MonolithProgress, MonolithStateError> {
        if abnormality_id.trim().is_empty() {
            return Err(MonolithStateError::InvalidAbnormality);
        }
        Ok(self
            .progress_by_abnormality
            .entry(abnormality_id.to_string())
            .or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn node(value: u128) -> MapNodeId {
        MapNodeId::new(Uuid::from_u128(value))
    }

    #[test]
    fn only_natural_victory_excludes_unmastered_pool_and_unlocks_purchase() {
        let mut state = MonolithRunState::default();
        state.record_natural_victory("won").unwrap();

        assert!(state.progress("won").unwrap().is_offer_eligible());
        assert_eq!(
            state.record_natural_victory("won"),
            Err(MonolithStateError::NaturalEliteAlreadyResolved)
        );
    }

    #[test]
    fn exhausted_remanifestation_failure_is_run_local_permanent() {
        let mut state = MonolithRunState::default();
        state.record_natural_victory("a").unwrap();
        state.purchase("a", 3).unwrap();
        state
            .begin_remanifestation(node(1), "a", "elite_a")
            .unwrap();

        state.resolve_remanifestation_failure(node(1)).unwrap();

        let progress = state.progress("a").unwrap();
        assert!(progress.remanifestation_failed);
        assert_eq!(progress.uses_remaining, 0);
        assert!(!progress.is_usable());
        assert_eq!(
            state.begin_remanifestation(node(2), "a", "elite_a"),
            Err(MonolithStateError::NotUsable)
        );
    }

    #[test]
    fn conversion_consumes_on_commit_and_enforces_one_active_node_per_identity() {
        let mut state = MonolithRunState::default();
        state.record_natural_victory("a").unwrap();
        state.purchase("a", 3).unwrap();

        let converted = state
            .begin_remanifestation(node(1), "a", "elite_a")
            .unwrap();
        assert_eq!(converted.stage, 2);
        assert_eq!(state.progress("a").unwrap().uses_remaining, 2);
        assert_eq!(
            state.begin_remanifestation(node(2), "a", "elite_a"),
            Err(MonolithStateError::NotUsable)
        );
    }

    #[test]
    fn final_victory_with_one_use_left_creates_residual_and_shop_guarantee() {
        let mut state = MonolithRunState::default();
        for index in 0..4_u128 {
            let id = format!("a{index}");
            state.record_natural_victory(&id).unwrap();
            state.purchase(&id, 3).unwrap();
            state
                .begin_remanifestation(node(index * 10 + 1), &id, "elite")
                .unwrap();
            state
                .resolve_remanifestation_victory(node(index * 10 + 1), 4)
                .unwrap();
            state
                .begin_remanifestation(node(index * 10 + 2), &id, "elite")
                .unwrap();
            let resolution = state
                .resolve_remanifestation_victory(node(index * 10 + 2), 4)
                .unwrap();
            assert!(resolution.residual_created);
        }

        assert_eq!(state.residuals.len(), 4);
        assert_eq!(state.exchange_state, MonolithExchangeState::Available);
        assert!(state.consume_next_floor_shop_guarantee());
        assert!(!state.consume_next_floor_shop_guarantee());

        state.record_natural_victory("a4").unwrap();
        state.purchase("a4", 3).unwrap();
        state
            .begin_remanifestation(node(41), "a4", "elite")
            .unwrap();
        state.resolve_remanifestation_victory(node(41), 4).unwrap();
        state
            .begin_remanifestation(node(42), "a4", "elite")
            .unwrap();
        state.resolve_remanifestation_victory(node(42), 4).unwrap();
        assert_eq!(state.residuals.len(), 5);
        assert!(!state.next_floor_shop_guarantee_pending);
    }

    #[test]
    fn invalid_resolution_keeps_active_remanifestation_transaction_intact() {
        let mut state = MonolithRunState::default();
        state.record_natural_victory("abno").unwrap();
        state.purchase("abno", 3).unwrap();
        state
            .begin_remanifestation(node(1), "abno", "elite_abno")
            .unwrap();
        state
            .progress_by_abnormality
            .get_mut("abno")
            .unwrap()
            .victories = 0;
        let before = state.clone();

        assert_eq!(
            state.resolve_remanifestation_victory(node(1), 4),
            Err(MonolithStateError::InvalidStage)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn floor_transition_discards_active_node_without_refunding_committed_use() {
        let mut state = MonolithRunState::default();
        state.record_natural_victory("abno").unwrap();
        state.purchase("abno", 3).unwrap();
        state
            .begin_remanifestation(node(1), "abno", "elite_abno")
            .unwrap();

        state.clear_floor_local_remanifestations();

        let progress = state.progress("abno").unwrap();
        assert_eq!(progress.uses_remaining, 2);
        assert_eq!(progress.active_node_id, None);
        assert!(state.remanifestation_nodes.is_empty());
        assert!(progress.is_usable());
    }

    #[test]
    fn exchange_consumes_oldest_four_once_and_grants_dormant_key() {
        let mut state = MonolithRunState {
            residuals: (0..5)
                .rev()
                .map(|sequence| MonolithResidual {
                    sequence,
                    source_abnormality_id: format!("a{sequence}"),
                })
                .collect(),
            exchange_state: MonolithExchangeState::Available,
            ..MonolithRunState::default()
        };

        state.start_exchange(4).unwrap();
        let consumed = state.accept_exchange(4).unwrap();
        assert_eq!(
            consumed
                .iter()
                .map(|entry| entry.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(state.residuals[0].sequence, 4);
        assert!(state.corrupted_monolith_acquired);
        assert_eq!(state.exchange_state, MonolithExchangeState::Accepted);
        assert_eq!(
            state.start_exchange(4),
            Err(MonolithStateError::ExchangeUnavailable)
        );
    }
}
