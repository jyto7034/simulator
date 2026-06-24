//! Current source of truth for combat mission type, variant, objective, and reward policy.
//!
//! Keep this as explicit table/function policy until there are enough live
//! mission variants to justify a richer data-driven system.

use crate::game::{
    battle::scenario::{
        BattleObjective, EnemyMovementPlan, PlayerMovementPlan, ScenarioUnitRef, TacticalPlan,
        WinCondition,
    },
    behavior::GameError,
    combat_preview::{
        BattlefieldArchetype, CombatMissionRisk, CombatMissionVariant, CombatNodeType,
        CombatPreview,
    },
    data::reward_data::RewardGrantKind,
    enums::RiskLevel,
    map::MapNodeCategory,
    resources::Position,
};

pub const DEFAULT_DEFENSE_OBJECT_REF: &str = "black_box_device";

/// Source of truth for currently implemented combat mission policy.
///
/// Keep this module as simple table/function lookups. Do not turn it into a
/// trait framework until real mission variants require runtime pluggability.
pub struct CombatMissionPolicy;

impl CombatMissionPolicy {
    pub const SUPPORTED_ENCOUNTER_NODE_TYPES: &'static [CombatNodeType] =
        &[CombatNodeType::Defense, CombatNodeType::Boss];

    pub const ALLOWED_COMBAT_REWARD_KINDS: &'static [RewardGrantKind] = &[
        RewardGrantKind::Currency,
        RewardGrantKind::Experience,
        RewardGrantKind::Equipment,
        RewardGrantKind::Artifact,
        RewardGrantKind::SkillFragment,
        RewardGrantKind::ResearchProgress,
        RewardGrantKind::Narrative,
    ];

    pub fn is_supported_encounter_node_type(node_type: CombatNodeType) -> bool {
        Self::SUPPORTED_ENCOUNTER_NODE_TYPES.contains(&node_type)
    }

    pub fn starts_as_live_battle(
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
    ) -> bool {
        matches!(
            (node_type, mission_variant),
            (CombatNodeType::Defense, CombatMissionVariant::Defense)
        )
    }

    pub fn preferred_node_types_for_map_node(
        category: MapNodeCategory,
        kind_id: &str,
    ) -> &'static [CombatNodeType] {
        if category == MapNodeCategory::Boss {
            return &[CombatNodeType::Boss];
        }

        if category != MapNodeCategory::Combat {
            return &[];
        }

        if kind_id.contains("elite") {
            return &[CombatNodeType::Defense];
        }

        &[CombatNodeType::Defense]
    }

    pub fn try_fallback_node_type_for_archetype(
        category: MapNodeCategory,
        archetype: BattlefieldArchetype,
    ) -> Result<CombatNodeType, GameError> {
        if category == MapNodeCategory::Boss {
            return Ok(CombatNodeType::Boss);
        }

        let node_type = match archetype {
            BattlefieldArchetype::OpenHall | BattlefieldArchetype::ObstacleRoom => {
                CombatNodeType::Defense
            }
            BattlefieldArchetype::Corridor => CombatNodeType::Defense,
            BattlefieldArchetype::ChokePoint => CombatNodeType::Defense,
            BattlefieldArchetype::Ambush => CombatNodeType::Defense,
            BattlefieldArchetype::Surrounded => CombatNodeType::Defense,
            BattlefieldArchetype::SplitRoom => {
                return Err(GameError::InvalidStaticData(
                    "SplitRoom battlefield archetype requires an explicit DefenseRoute/SplitRoom encounter; fallback is disabled until SplitRoom is implemented"
                        .to_string(),
                ));
            }
            BattlefieldArchetype::BossArena => CombatNodeType::Boss,
        };
        Ok(node_type)
    }

    pub fn fallback_mission_variant_for_archetype(
        node_type: CombatNodeType,
        _archetype: BattlefieldArchetype,
    ) -> CombatMissionVariant {
        CombatMissionVariant::default_for_node_type(node_type)
    }

    pub fn featured_reward_kinds_for_mission(
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
    ) -> &'static [RewardGrantKind] {
        match (node_type, mission_variant) {
            (CombatNodeType::Defense, CombatMissionVariant::Defense) => &[
                RewardGrantKind::ResearchProgress,
                RewardGrantKind::Equipment,
                RewardGrantKind::Narrative,
            ],
            (CombatNodeType::Boss, CombatMissionVariant::Boss) => &[
                RewardGrantKind::SkillFragment,
                RewardGrantKind::Equipment,
                RewardGrantKind::ResearchProgress,
            ],
            _ => &[],
        }
    }

    pub fn mission_risk_from_risk_level(risk_level: RiskLevel) -> CombatMissionRisk {
        match risk_level {
            RiskLevel::ZAYIN | RiskLevel::TETH => CombatMissionRisk::Controlled,
            RiskLevel::HE | RiskLevel::WAW => CombatMissionRisk::Unstable,
            RiskLevel::ALEPH => CombatMissionRisk::Collapse,
        }
    }

    pub fn default_tactical_plan_for_preview(combat_preview: &CombatPreview) -> TacticalPlan {
        match (combat_preview.node_type, combat_preview.mission_variant) {
            (CombatNodeType::Defense, CombatMissionVariant::Defense) => {
                default_defense_tactical_plan(combat_preview)
            }
            _ => TacticalPlan::default(),
        }
    }

    pub fn default_win_condition_for_tactical_plan(
        mission_variant: CombatMissionVariant,
        _mission_risk: CombatMissionRisk,
        tactical_plan: &TacticalPlan,
        survive_timer_ms: Option<u64>,
    ) -> WinCondition {
        if mission_variant == CombatMissionVariant::Defense {
            if let BattleObjective::ProtectUnit { unit_ref } = &tactical_plan.objective {
                if let Some(time_ms) = survive_timer_ms {
                    return WinCondition::ProtectUnitUntil {
                        unit_ref: unit_ref.clone(),
                        time_ms,
                    };
                }
                return WinCondition::ProtectUnit {
                    unit_ref: unit_ref.clone(),
                };
            }
        }

        WinCondition::AllRequiredEnemyGroupsDefeated
    }
}

fn default_defense_tactical_plan(combat_preview: &CombatPreview) -> TacticalPlan {
    TacticalPlan {
        objective: BattleObjective::ProtectUnit {
            unit_ref: ScenarioUnitRef::new(DEFAULT_DEFENSE_OBJECT_REF),
        },
        player_plan: PlayerMovementPlan::FixedDefense,
        enemy_plan: EnemyMovementPlan::PathAlongCells {
            cells: default_defense_route_cells(combat_preview),
        },
        points: Vec::new(),
    }
}

fn default_defense_route_cells(combat_preview: &CombatPreview) -> Vec<Position> {
    combat_preview
        .routes
        .first()
        .map(|route| {
            if route.cells.is_empty() {
                vec![route.end]
            } else {
                route.cells.clone()
            }
        })
        .unwrap_or_else(|| vec![default_defense_point_position(combat_preview)])
}

fn default_defense_point_position(combat_preview: &CombatPreview) -> Position {
    let cells = combat_preview
        .deployment_zones
        .first()
        .map(|zone| zone.cells.as_slice())
        .unwrap_or(&[]);
    if cells.is_empty() {
        return Position::new(combat_preview.width / 2, combat_preview.height / 2);
    }

    let sum_x = cells.iter().map(|cell| cell.x).sum::<i32>();
    let sum_y = cells.iter().map(|cell| cell.y).sum::<i32>();
    let center = Position::new(
        sum_x / i32::try_from(cells.len()).unwrap_or(1),
        sum_y / i32::try_from(cells.len()).unwrap_or(1),
    );
    cells
        .iter()
        .copied()
        .min_by_key(|cell| (cell.x - center.x).abs() + (cell.y - center.y).abs())
        .unwrap_or(center)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_current_encounter_node_types_are_supported() {
        assert!(CombatMissionPolicy::is_supported_encounter_node_type(
            CombatNodeType::Defense
        ));
        assert!(CombatMissionPolicy::is_supported_encounter_node_type(
            CombatNodeType::Boss
        ));
    }

    #[test]
    fn elite_map_nodes_prefer_high_pressure_missions() {
        let preferred = CombatMissionPolicy::preferred_node_types_for_map_node(
            MapNodeCategory::Combat,
            "combat_elite",
        );

        assert_eq!(preferred[0], CombatNodeType::Defense);
    }

    #[test]
    fn split_room_fallback_is_disabled_until_policy_is_implemented() {
        let result = CombatMissionPolicy::try_fallback_node_type_for_archetype(
            MapNodeCategory::Combat,
            BattlefieldArchetype::SplitRoom,
        );

        assert!(result.is_err());
    }

    #[test]
    fn live_execution_is_explicit_per_mission_variant() {
        assert!(CombatMissionPolicy::starts_as_live_battle(
            CombatNodeType::Defense,
            CombatMissionVariant::Defense
        ));
        assert!(!CombatMissionPolicy::starts_as_live_battle(
            CombatNodeType::Boss,
            CombatMissionVariant::Boss
        ));
    }

    #[test]
    fn battlefield_archetype_does_not_change_mission_variant() {
        assert_eq!(
            CombatMissionPolicy::fallback_mission_variant_for_archetype(
                CombatNodeType::Defense,
                BattlefieldArchetype::Surrounded,
            ),
            CombatMissionVariant::Defense
        );
    }
}
