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
    #[serde(deserialize_with = "crate::game::data::deserialize_required_option")]
    pub event_id: Option<EventId>,
    #[serde(deserialize_with = "crate::game::data::deserialize_required_option")]
    pub encounter_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (index, chain) in self.chains.iter().enumerate() {
            validate_chain_authoring_contract(chain, index)?;
        }
        Ok(())
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid BossOmen data: {error:?}"));
        let _ = self.by_id();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&BossOmenChain> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.chains.get(index))
    }
}

fn validate_chain_authoring_contract(
    chain: &BossOmenChain,
    chain_index: usize,
) -> Result<(), crate::game::data::DataValidationError> {
    let chain_path = format!("chains[{chain_index}]");
    let error = |path: String, cause: &'static str| {
        crate::game::data::DataValidationError::at(chain.id.as_str(), path, cause)
    };
    if chain.id.as_str().trim().is_empty() {
        return Err(error(
            format!("{chain_path}.id"),
            "chain id must not be empty",
        ));
    }
    if chain.boss_abnormality_id.trim().is_empty() {
        return Err(error(
            format!("{chain_path}.boss_abnormality_id"),
            "boss_abnormality_id must not be empty",
        ));
    }
    if chain.min_floor == 0 {
        return Err(error(
            format!("{chain_path}.min_floor"),
            "min_floor must be greater than zero",
        ));
    }
    if chain.required_steps == 0 || chain.required_steps as usize > chain.steps.len() {
        return Err(error(
            format!("{chain_path}.required_steps"),
            "required_steps must be positive and not exceed authored steps",
        ));
    }
    if chain.boss_encounter_id.trim().is_empty() {
        return Err(error(
            format!("{chain_path}.boss_encounter_id"),
            "boss_encounter_id must not be empty",
        ));
    }
    if chain.steps.is_empty() {
        return Err(error(
            format!("{chain_path}.steps"),
            "must define at least one step",
        ));
    }

    let mut seen_step_ids = HashSet::new();
    for (step_index, step) in chain.steps.iter().enumerate() {
        let step_path = format!("{chain_path}.steps[{step_index}]");
        if step.id.as_str().trim().is_empty() {
            return Err(error(
                format!("{step_path}.id"),
                "step id must not be empty",
            ));
        }
        if !seen_step_ids.insert(step.id.as_str()) {
            return Err(error(format!("{step_path}.id"), "duplicate step id"));
        }
        for (field, value) in [
            ("hint_id", step.hint_id.as_str()),
            ("title_id", step.title_id.as_str()),
            ("description_id", step.description_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(error(
                    format!("{step_path}.{field}"),
                    "presentation id must not be empty",
                ));
            }
        }
        match step.source_kind {
            BossOmenSourceKind::Event => {
                if !step
                    .event_id
                    .as_ref()
                    .is_some_and(|event_id| !event_id.as_str().trim().is_empty())
                {
                    return Err(error(
                        format!("{step_path}.event_id"),
                        "Event step must define event_id",
                    ));
                }
                if step.encounter_id.is_some() {
                    return Err(error(
                        format!("{step_path}.encounter_id"),
                        "Event step must not define encounter_id",
                    ));
                }
            }
            BossOmenSourceKind::Combat => {
                if !step
                    .encounter_id
                    .as_ref()
                    .is_some_and(|encounter_id| !encounter_id.trim().is_empty())
                {
                    return Err(error(
                        format!("{step_path}.encounter_id"),
                        "Combat step must define encounter_id",
                    ));
                }
                if step.event_id.is_some() {
                    return Err(error(
                        format!("{step_path}.event_id"),
                        "Combat step must not define event_id",
                    ));
                }
            }
        }
    }
    Ok(())
}
