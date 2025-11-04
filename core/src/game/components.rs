use bevy_ecs::component::Component;

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
