use bevy_ecs::{bundle::Bundle, component::Component};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::enums::{RiskLevel, Tier};

#[derive(Component, Clone)]
pub struct Abnormality {
    pub id: String,
    pub name: String,
    pub risk_level: RiskLevel,
    pub tier: Tier,
}

impl Abnormality {
    pub fn new(id: &str, name: &str, risk_level: RiskLevel, tier: Tier) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            risk_level,
            tier,
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
}

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub level: u32,
    pub exp: u32,
}

/// Player Entity Bundle
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
}
