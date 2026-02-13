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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abnormality_is_component_and_is_stored_in_world() {
        use bevy_ecs::world::World;

        let mut world = World::new();
        let entity = world
            .spawn(Abnormality::new(
                "F-01-02",
                "Scorched Girl",
                RiskLevel::TETH,
                Tier::I,
            ))
            .id();

        let abnormality = world.get::<Abnormality>(entity).unwrap();
        assert_eq!(abnormality.id, "F-01-02");
        assert_eq!(abnormality.risk_level, RiskLevel::TETH);
    }

    #[test]
    fn player_serialization_roundtrip() {
        let player_id = Uuid::new_v4();
        let player = Player {
            id: player_id,
            name: "Hero".to_string(),
        };

        // When: JSON 직렬화/역직렬화
        let json = serde_json::to_string(&player).unwrap();
        let deserialized: Player = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, player.id);
        assert_eq!(deserialized.name, player.name);
    }

    #[test]
    fn player_bundle_spawns_queryable_player_component() {
        use bevy_ecs::world::World;

        let mut world = World::new();
        let player_id = Uuid::new_v4();

        let entity = world
            .spawn(PlayerBundle {
                player: Player {
                    id: player_id,
                    name: "Test Hero".to_string(),
                },
            })
            .id();

        // Then: Entity가 생성되어 Player를 조회할 수 있음
        let player = world.get::<Player>(entity).unwrap();
        assert_eq!(player.id, player_id);
        assert_eq!(player.name, "Test Hero");
    }
}
