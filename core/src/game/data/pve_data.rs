use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        data::{build_string_index, once_lock_with},
        enums::{RewardMode, RiskLevel, Tier},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveUnitData {
    pub abnormality_id: String,
    pub position: PvePosition,
    #[serde(default = "default_tier")]
    pub tier: Tier,
}

fn default_tier() -> Tier {
    Tier::I
}

fn default_reward_mode() -> RewardMode {
    RewardMode::ClaimAll
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
pub struct PveEncounter {
    pub id: String,
    pub abnormality_id: String,
    pub difficulty: u8,
    pub risk_level: RiskLevel,
    #[serde(default = "default_reward_mode")]
    pub reward_mode: RewardMode,
    #[serde(default)]
    pub reward_bonus_uuids: Vec<Uuid>,
    pub units: Vec<PveUnitData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveEncounterDatabase {
    pub encounters: Vec<PveEncounter>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_abnormality_id: OnceLock<HashMap<String, usize>>,
}

impl PveEncounterDatabase {
    pub fn new(encounters: Vec<PveEncounter>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter id",
            |encounter| &encounter.id,
        ));
        let by_abnormality_id = once_lock_with(build_string_index(
            &encounters,
            "pve encounter abnormality_id",
            |encounter| &encounter.abnormality_id,
        ));

        Self {
            encounters,
            by_id,
            by_abnormality_id,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.encounters, "pve encounter id", |encounter| {
                &encounter.id
            })
        })
    }

    fn by_abnormality_id(&self) -> &HashMap<String, usize> {
        self.by_abnormality_id.get_or_init(|| {
            build_string_index(
                &self.encounters,
                "pve encounter abnormality_id",
                |encounter| &encounter.abnormality_id,
            )
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_abnormality_id();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&PveEncounter> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_by_abnormality_id(&self, abnormality_id: &str) -> Option<&PveEncounter> {
        self.by_abnormality_id()
            .get(abnormality_id)
            .and_then(|&index| self.encounters.get(index))
    }

    pub fn get_by_risk_level(&self, level: RiskLevel) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.risk_level == level)
            .collect()
    }

    pub fn get_by_difficulty_range(&self, min: u8, max: u8) -> Vec<&PveEncounter> {
        self.encounters
            .iter()
            .filter(|e| e.difficulty >= min && e.difficulty <= max)
            .collect()
    }
}
