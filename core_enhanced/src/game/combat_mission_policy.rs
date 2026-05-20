use std::collections::HashSet;

use crate::game::{
    battle::scenario::{
        BattleObjective, EnemyMovementPlan, FormationKind, GroupObjective, PlayerMovementPlan,
        ScenarioUnitRef, TacticalGroupMembers, TacticalGroupPlan, TacticalGroupPlanId,
        TacticalPlan, TacticalPoint, TacticalPointId, WinCondition,
    },
    combat_preview::{BattlefieldArchetype, CombatMissionRisk, CombatNodeType, CombatPreview},
    data::reward_data::RewardTag,
    enums::RiskLevel,
    map::MapNodeCategory,
    resources::Position,
};

pub const DEFAULT_DEFENSE_OBJECT_REF: &str = "black_box_recovery_device";
pub const DEFAULT_DEFENSE_POINT_ID: &str = "black_box_recovery";
pub const DEFAULT_RECOVERY_TARGET_POINT_ID: &str = "recovery_target";
pub const DEFAULT_RECOVERY_EXTRACTION_POINT_ID: &str = "extraction_point";
pub const DEFAULT_SURVIVAL_ANCHOR_POINT_ID: &str = "survival_anchor";
pub const DEFAULT_ENCIRCLEMENT_SURVIVE_MS: u64 = 45_000;

/// Source of truth for currently implemented combat mission policy.
///
/// Keep this module as simple table/function lookups. Do not turn it into a
/// trait framework until real mission variants require runtime pluggability.
pub struct CombatMissionPolicy;

impl CombatMissionPolicy {
    pub const LIVE_SUPPORTED_NODE_TYPES: &'static [CombatNodeType] = &[
        CombatNodeType::Suppression,
        CombatNodeType::Defense,
        CombatNodeType::Frontline,
        CombatNodeType::Encirclement,
        CombatNodeType::Recovery,
        CombatNodeType::Boss,
    ];

    pub const ALLOWED_COMBAT_REWARD_TAGS: &'static [RewardTag] = &[
        RewardTag::Currency,
        RewardTag::Experience,
        RewardTag::Equipment,
        RewardTag::Artifact,
        RewardTag::SkillFragment,
        RewardTag::ResearchProgress,
        RewardTag::Narrative,
    ];

    pub fn is_live_supported_node_type(node_type: CombatNodeType) -> bool {
        Self::LIVE_SUPPORTED_NODE_TYPES.contains(&node_type)
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
            return &[
                CombatNodeType::Encirclement,
                CombatNodeType::Frontline,
                CombatNodeType::Suppression,
                CombatNodeType::Defense,
                CombatNodeType::Recovery,
            ];
        }

        &[
            CombatNodeType::Suppression,
            CombatNodeType::Frontline,
            CombatNodeType::Defense,
            CombatNodeType::Encirclement,
            CombatNodeType::Recovery,
        ]
    }

    pub fn fallback_node_type_for_archetype(
        category: MapNodeCategory,
        archetype: BattlefieldArchetype,
    ) -> CombatNodeType {
        if category == MapNodeCategory::Boss {
            return CombatNodeType::Boss;
        }

        match archetype {
            BattlefieldArchetype::OpenHall | BattlefieldArchetype::ObstacleRoom => {
                CombatNodeType::Suppression
            }
            BattlefieldArchetype::Corridor => CombatNodeType::Frontline,
            BattlefieldArchetype::ChokePoint => CombatNodeType::Defense,
            BattlefieldArchetype::Ambush => CombatNodeType::Suppression,
            BattlefieldArchetype::Surrounded => CombatNodeType::Encirclement,
            BattlefieldArchetype::SplitRoom => CombatNodeType::Suppression,
            BattlefieldArchetype::BossArena => CombatNodeType::Boss,
        }
    }

    pub fn featured_reward_tags_for_node_type(node_type: CombatNodeType) -> &'static [RewardTag] {
        match node_type {
            CombatNodeType::Suppression => &[
                RewardTag::SkillFragment,
                RewardTag::Equipment,
                RewardTag::ResearchProgress,
                RewardTag::Experience,
            ],
            CombatNodeType::Defense => &[
                RewardTag::ResearchProgress,
                RewardTag::Equipment,
                RewardTag::Narrative,
            ],
            CombatNodeType::Frontline => &[RewardTag::Equipment, RewardTag::Experience],
            CombatNodeType::Encirclement => &[
                RewardTag::Equipment,
                RewardTag::ResearchProgress,
                RewardTag::Experience,
            ],
            CombatNodeType::SplitOperation => &[],
            CombatNodeType::Recovery => &[
                RewardTag::Equipment,
                RewardTag::Narrative,
                RewardTag::ResearchProgress,
            ],
            CombatNodeType::Boss => &[
                RewardTag::SkillFragment,
                RewardTag::Equipment,
                RewardTag::ResearchProgress,
            ],
        }
    }

    pub fn mission_risk_from_risk_level(risk_level: RiskLevel) -> CombatMissionRisk {
        match risk_level {
            RiskLevel::ZAYIN | RiskLevel::TETH => CombatMissionRisk::Controlled,
            RiskLevel::HE | RiskLevel::WAW => CombatMissionRisk::Unstable,
            RiskLevel::ALEPH => CombatMissionRisk::Collapse,
        }
    }

    pub fn defense_cleanup_required(risk: CombatMissionRisk) -> bool {
        !matches!(risk, CombatMissionRisk::Controlled)
    }

    pub fn default_defense_duration_ms(risk: CombatMissionRisk) -> u64 {
        match risk {
            CombatMissionRisk::Controlled => 30_000,
            CombatMissionRisk::Unstable => 45_000,
            CombatMissionRisk::Collapse => 60_000,
        }
    }

    pub fn default_recovery_hold_duration_ms(risk: CombatMissionRisk) -> u64 {
        match risk {
            CombatMissionRisk::Controlled => 5_000,
            CombatMissionRisk::Unstable => 8_000,
            CombatMissionRisk::Collapse => 12_000,
        }
    }

    pub fn default_tactical_plan_for_preview(
        node_type: CombatNodeType,
        combat_preview: &CombatPreview,
    ) -> TacticalPlan {
        let mut plan = TacticalPlan::for_archetype(combat_preview.archetype);
        match node_type {
            CombatNodeType::Defense => {
                let point_id = TacticalPointId::new(DEFAULT_DEFENSE_POINT_ID);
                let point = TacticalPoint {
                    id: point_id.clone(),
                    position: default_defense_point_position(combat_preview),
                };
                plan.objective = BattleObjective::ProtectUnitForDuration {
                    unit_ref: ScenarioUnitRef::new(DEFAULT_DEFENSE_OBJECT_REF),
                    time_ms: combat_preview.mission_risk.default_defense_duration_ms(),
                    cleanup_required: combat_preview.mission_risk.defense_cleanup_required(),
                };
                plan.player_plan = PlayerMovementPlan::HoldDeployment {
                    guard_radius: 1.5,
                    leash_radius: 2.5,
                    chase_radius: 0.75,
                    return_to_anchor: true,
                };
                plan.enemy_plan = EnemyMovementPlan::PathToPoint { point_id };
                plan.points = vec![point];
                plan
            }
            CombatNodeType::Recovery => default_recovery_tactical_plan(combat_preview, plan),
            CombatNodeType::Frontline => default_frontline_tactical_plan(plan),
            CombatNodeType::Encirclement => {
                default_encirclement_tactical_plan(combat_preview, plan)
            }
            _ => plan,
        }
    }

    pub fn default_win_condition_for_tactical_plan(
        node_type: CombatNodeType,
        mission_risk: CombatMissionRisk,
        tactical_plan: &TacticalPlan,
    ) -> WinCondition {
        if node_type == CombatNodeType::Defense {
            if let BattleObjective::ProtectUnitForDuration {
                unit_ref,
                time_ms,
                cleanup_required,
            } = &tactical_plan.objective
            {
                return WinCondition::ProtectUnitForDuration {
                    unit_ref: unit_ref.clone(),
                    time_ms: *time_ms,
                    cleanup_required: *cleanup_required,
                };
            }
            if let BattleObjective::ProtectUnit { unit_ref } = &tactical_plan.objective {
                return WinCondition::ProtectUnitForDuration {
                    unit_ref: unit_ref.clone(),
                    time_ms: mission_risk.default_defense_duration_ms(),
                    cleanup_required: mission_risk.defense_cleanup_required(),
                };
            }
        }
        if node_type == CombatNodeType::Recovery {
            if let BattleObjective::RecoverHoldAndExtract {
                target_point_id,
                extraction_point_id,
                hold_duration_ms,
            } = &tactical_plan.objective
            {
                return WinCondition::RecoverHoldAndExtract {
                    target_point_id: target_point_id.clone(),
                    extraction_point_id: extraction_point_id.clone(),
                    target_radius: 0.75,
                    extraction_radius: 0.75,
                    hold_duration_ms: *hold_duration_ms,
                };
            }
        }
        if node_type == CombatNodeType::Encirclement {
            if let BattleObjective::Survive { time_ms } = &tactical_plan.objective {
                return WinCondition::SurviveUntil { time_ms: *time_ms };
            }
        }

        WinCondition::AllRequiredEnemyGroupsDefeated
    }
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

fn default_recovery_tactical_plan(
    combat_preview: &CombatPreview,
    mut base_plan: TacticalPlan,
) -> TacticalPlan {
    let target_point_id = TacticalPointId::new(DEFAULT_RECOVERY_TARGET_POINT_ID);
    let extraction_point_id = TacticalPointId::new(DEFAULT_RECOVERY_EXTRACTION_POINT_ID);
    let extraction_position = default_defense_point_position(combat_preview);
    let target_position = default_recovery_target_position(combat_preview, extraction_position);

    base_plan.objective = BattleObjective::RecoverHoldAndExtract {
        target_point_id: target_point_id.clone(),
        extraction_point_id: extraction_point_id.clone(),
        hold_duration_ms: combat_preview
            .mission_risk
            .default_recovery_hold_duration_ms(),
    };
    base_plan.player_plan = PlayerMovementPlan::CautiousEngage {
        leash_radius: 3.0,
        chase_radius: 1.0,
    };
    base_plan.enemy_plan = EnemyMovementPlan::AssaultPlayer;
    base_plan.points = vec![
        TacticalPoint {
            id: target_point_id.clone(),
            position: target_position,
        },
        TacticalPoint {
            id: extraction_point_id.clone(),
            position: extraction_position,
        },
    ];
    base_plan.group_plans = vec![TacticalGroupPlan {
        id: TacticalGroupPlanId::new("player_main"),
        side: crate::game::enums::Side::Player,
        members: TacticalGroupMembers::SideAll(crate::game::enums::Side::Player),
        objective: GroupObjective::AdvanceAlongPath {
            point_ids: vec![target_point_id, extraction_point_id],
        },
        formation: FormationKind::Loose,
        cohesion_radius: 3.0,
        engage_radius: 2.5,
    }];
    base_plan
}

fn default_frontline_tactical_plan(mut base_plan: TacticalPlan) -> TacticalPlan {
    base_plan.objective = BattleObjective::SuppressAll;
    base_plan.player_plan = PlayerMovementPlan::CautiousEngage {
        leash_radius: 5.0,
        chase_radius: 1.5,
    };
    base_plan.enemy_plan = EnemyMovementPlan::AssaultPlayer;
    base_plan.group_plans = vec![TacticalGroupPlan {
        id: TacticalPlan::default_player_main_group_id(),
        side: crate::game::enums::Side::Player,
        members: TacticalGroupMembers::SideAll(crate::game::enums::Side::Player),
        objective: GroupObjective::FollowBattleObjective,
        formation: FormationKind::Loose,
        cohesion_radius: 4.0,
        engage_radius: 3.0,
    }];
    base_plan
}

fn default_encirclement_tactical_plan(
    combat_preview: &CombatPreview,
    mut base_plan: TacticalPlan,
) -> TacticalPlan {
    let point_id = TacticalPointId::new(DEFAULT_SURVIVAL_ANCHOR_POINT_ID);
    let point = TacticalPoint {
        id: point_id.clone(),
        position: default_defense_point_position(combat_preview),
    };

    base_plan.objective = BattleObjective::Survive {
        time_ms: DEFAULT_ENCIRCLEMENT_SURVIVE_MS,
    };
    base_plan.player_plan = PlayerMovementPlan::HoldDeployment {
        guard_radius: 2.0,
        leash_radius: 3.0,
        chase_radius: 1.0,
        return_to_anchor: true,
    };
    base_plan.enemy_plan = EnemyMovementPlan::AssaultPlayer;
    base_plan.points = vec![point];
    base_plan.group_plans = vec![TacticalGroupPlan {
        id: TacticalPlan::default_player_main_group_id(),
        side: crate::game::enums::Side::Player,
        members: TacticalGroupMembers::SideAll(crate::game::enums::Side::Player),
        objective: GroupObjective::HoldArea { point_id },
        formation: FormationKind::Loose,
        cohesion_radius: 3.0,
        engage_radius: 2.5,
    }];
    base_plan
}

fn default_recovery_target_position(
    combat_preview: &CombatPreview,
    extraction_position: Position,
) -> Position {
    let obstacle_positions = combat_preview
        .obstacles
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let mut candidates = if combat_preview.valid_tiles.is_empty() {
        (0..combat_preview.height)
            .flat_map(|y| (0..combat_preview.width).map(move |x| Position::new(x, y)))
            .collect::<Vec<_>>()
    } else {
        combat_preview.valid_tiles.clone()
    };
    candidates.retain(|position| !obstacle_positions.contains(position));
    candidates
        .into_iter()
        .max_by_key(|position| {
            (position.x - extraction_position.x).abs() + (position.y - extraction_position.y).abs()
        })
        .unwrap_or(Position::new(
            combat_preview.width / 2,
            combat_preview.height / 2,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_operation_is_kept_out_of_live_supported_missions() {
        assert!(CombatMissionPolicy::is_live_supported_node_type(
            CombatNodeType::Defense
        ));
        assert!(!CombatMissionPolicy::is_live_supported_node_type(
            CombatNodeType::SplitOperation
        ));
    }

    #[test]
    fn elite_map_nodes_prefer_high_pressure_missions() {
        let preferred = CombatMissionPolicy::preferred_node_types_for_map_node(
            MapNodeCategory::Combat,
            "combat_elite",
        );

        assert_eq!(preferred[0], CombatNodeType::Encirclement);
        assert_eq!(preferred[1], CombatNodeType::Frontline);
        assert!(!preferred.contains(&CombatNodeType::SplitOperation));
    }

    #[test]
    fn defense_risk_controls_cleanup_requirement() {
        assert!(!CombatMissionPolicy::defense_cleanup_required(
            CombatMissionRisk::Controlled
        ));
        assert!(CombatMissionPolicy::defense_cleanup_required(
            CombatMissionRisk::Unstable
        ));
        assert!(CombatMissionPolicy::defense_cleanup_required(
            CombatMissionRisk::Collapse
        ));
    }
}
