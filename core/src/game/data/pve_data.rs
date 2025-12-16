use serde::{Deserialize, Serialize};

use crate::{
    ecs::resources::Position,
    game::enums::{RiskLevel, Tier},
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
    pub units: Vec<PveUnitData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PveEncounterDatabase {
    pub encounters: Vec<PveEncounter>,
}

impl PveEncounterDatabase {
    pub fn new(encounters: Vec<PveEncounter>) -> Self {
        Self { encounters }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&PveEncounter> {
        self.encounters.iter().find(|e| e.id == id)
    }

    pub fn get_by_abnormality_id(&self, abnormality_id: &str) -> Option<&PveEncounter> {
        self.encounters
            .iter()
            .find(|e| e.abnormality_id == abnormality_id)
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
