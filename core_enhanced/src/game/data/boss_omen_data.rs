use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::data::{build_string_index, event_data::EventId, once_lock_with};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BossOmenChainId(pub String);

impl BossOmenChainId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BossOmenStepId(pub String);

impl BossOmenStepId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossOmenSourceKind {
    Event,
    Combat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossOmenRevealLevel {
    HintOnly,
    Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossOmenChain {
    pub id: BossOmenChainId,
    pub boss_abnormality_id: String,
    pub min_floor: u32,
    pub required_steps: u32,
    pub steps: Vec<BossOmenStep>,
    pub boss_encounter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossOmenStep {
    pub id: BossOmenStepId,
    pub source_kind: BossOmenSourceKind,
    pub hint_id: String,
    pub title_id: String,
    pub description_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossOmenChainDatabase {
    pub chains: Vec<BossOmenChain>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl BossOmenChainDatabase {
    pub fn new(chains: Vec<BossOmenChain>) -> Self {
        let by_id = once_lock_with(build_string_index(&chains, "boss omen chain id", |chain| {
            chain.id.as_str()
        }));
        Self { chains, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.chains, "boss omen chain id", |chain| {
                chain.id.as_str()
            })
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        for chain in &self.chains {
            validate_chain_authoring_contract(chain);
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&BossOmenChain> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.chains.get(index))
    }
}

fn validate_chain_authoring_contract(chain: &BossOmenChain) {
    assert!(
        !chain.id.as_str().trim().is_empty(),
        "boss omen chain id must not be empty"
    );
    assert!(
        !chain.boss_abnormality_id.trim().is_empty(),
        "boss omen chain '{}' boss_abnormality_id must not be empty",
        chain.id.as_str()
    );
    assert!(
        chain.min_floor > 0,
        "boss omen chain '{}' min_floor must be greater than zero",
        chain.id.as_str()
    );
    assert!(
        chain.required_steps > 0,
        "boss omen chain '{}' required_steps must be greater than zero",
        chain.id.as_str()
    );
    assert!(
        !chain.boss_encounter_id.trim().is_empty(),
        "boss omen chain '{}' boss_encounter_id must not be empty",
        chain.id.as_str()
    );
    assert!(
        !chain.steps.is_empty(),
        "boss omen chain '{}' must define at least one step",
        chain.id.as_str()
    );
    assert!(
        chain.required_steps as usize <= chain.steps.len(),
        "boss omen chain '{}' required_steps must not exceed authored steps",
        chain.id.as_str()
    );

    let mut seen_step_ids = HashSet::new();
    for step in &chain.steps {
        assert!(
            !step.id.as_str().trim().is_empty(),
            "boss omen chain '{}' has empty step id",
            chain.id.as_str()
        );
        assert!(
            seen_step_ids.insert(step.id.as_str()),
            "boss omen chain '{}' has duplicate step id '{}'",
            chain.id.as_str(),
            step.id.as_str()
        );
        assert!(
            !step.hint_id.trim().is_empty(),
            "boss omen chain '{}' step '{}' hint_id must not be empty",
            chain.id.as_str(),
            step.id.as_str()
        );
        assert!(
            !step.title_id.trim().is_empty(),
            "boss omen chain '{}' step '{}' title_id must not be empty",
            chain.id.as_str(),
            step.id.as_str()
        );
        assert!(
            !step.description_id.trim().is_empty(),
            "boss omen chain '{}' step '{}' description_id must not be empty",
            chain.id.as_str(),
            step.id.as_str()
        );
        match step.source_kind {
            BossOmenSourceKind::Event => {
                assert!(
                    step.event_id
                        .as_ref()
                        .is_some_and(|event_id| !event_id.as_str().trim().is_empty()),
                    "boss omen chain '{}' event step '{}' must define event_id",
                    chain.id.as_str(),
                    step.id.as_str()
                );
                assert!(
                    step.encounter_id.is_none(),
                    "boss omen chain '{}' event step '{}' must not define encounter_id",
                    chain.id.as_str(),
                    step.id.as_str()
                );
            }
            BossOmenSourceKind::Combat => {
                assert!(
                    step.encounter_id
                        .as_ref()
                        .is_some_and(|encounter_id| !encounter_id.trim().is_empty()),
                    "boss omen chain '{}' combat step '{}' must define encounter_id",
                    chain.id.as_str(),
                    step.id.as_str()
                );
                assert!(
                    step.event_id.is_none(),
                    "boss omen chain '{}' combat step '{}' must not define event_id",
                    chain.id.as_str(),
                    step.id.as_str()
                );
            }
        }
    }
}
