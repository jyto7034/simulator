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

    // ============================================================
    // Abnormality Tests
    // ============================================================

    #[test]
    fn test_abnormality_new() {
        let abnormality = Abnormality::new("F-01-02", "Scorched Girl", RiskLevel::TETH, Tier::I);

        assert_eq!(abnormality.id, "F-01-02");
        assert_eq!(abnormality.name, "Scorched Girl");
        assert_eq!(abnormality.risk_level, RiskLevel::TETH);
        assert_eq!(abnormality.tier, Tier::I);
    }

    #[test]
    fn test_abnormality_clone() {
        let abnormality = Abnormality::new(
            "O-01-04",
            "Opened Can of WellCheers",
            RiskLevel::ZAYIN,
            Tier::II,
        );

        let cloned = abnormality.clone();

        assert_eq!(cloned.id, abnormality.id);
        assert_eq!(cloned.name, abnormality.name);
        assert_eq!(cloned.risk_level, abnormality.risk_level);
        assert_eq!(cloned.tier, abnormality.tier);
    }

    #[test]
    fn test_abnormality_various_risk_levels() {
        let zayin = Abnormality::new("Z-01", "ZAYIN Level", RiskLevel::ZAYIN, Tier::I);
        let teth = Abnormality::new("T-01", "TETH Level", RiskLevel::TETH, Tier::I);
        let he = Abnormality::new("H-01", "HE Level", RiskLevel::HE, Tier::II);
        let waw = Abnormality::new("W-01", "WAW Level", RiskLevel::WAW, Tier::II);
        let aleph = Abnormality::new("A-01", "ALEPH Level", RiskLevel::ALEPH, Tier::III);

        assert_eq!(zayin.risk_level, RiskLevel::ZAYIN);
        assert_eq!(teth.risk_level, RiskLevel::TETH);
        assert_eq!(he.risk_level, RiskLevel::HE);
        assert_eq!(waw.risk_level, RiskLevel::WAW);
        assert_eq!(aleph.risk_level, RiskLevel::ALEPH);
    }

    // ============================================================
    // Player Tests
    // ============================================================

    #[test]
    fn test_player_creation() {
        let player_id = Uuid::new_v4();
        let player = Player {
            id: player_id,
            name: "Test Player".to_string(),
        };

        assert_eq!(player.id, player_id);
        assert_eq!(player.name, "Test Player");
    }

    #[test]
    fn test_player_clone() {
        let player_id = Uuid::new_v4();
        let player = Player {
            id: player_id,
            name: "Hero".to_string(),
        };

        let cloned = player.clone();

        assert_eq!(cloned.id, player.id);
        assert_eq!(cloned.name, player.name);
    }

    #[test]
    fn test_player_serialization() {
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

    // ============================================================
    // PlayerStats Tests
    // ============================================================

    #[test]
    fn test_player_stats_creation() {
        let stats = PlayerStats { level: 1, exp: 0 };

        assert_eq!(stats.level, 1);
        assert_eq!(stats.exp, 0);
    }

    #[test]
    fn test_player_stats_clone() {
        let stats = PlayerStats {
            level: 5,
            exp: 1000,
        };

        let cloned = stats.clone();

        assert_eq!(cloned.level, stats.level);
        assert_eq!(cloned.exp, stats.exp);
    }

    // ============================================================
    // PlayerBundle Tests
    // ============================================================

    #[test]
    fn test_player_bundle_creation() {
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
