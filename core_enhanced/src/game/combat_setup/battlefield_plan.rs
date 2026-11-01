//! Converts a generated combat preview into the static battlefield plan used to start battle.
//!
//! This module intentionally owns only field dimensions, valid tiles, and static
//! obstacles. Spawn groups, mission policy, and rewards are assembled by sibling
//! combat setup modules.

use crate::game::{
    battle::scenario::{BattleFieldSpec, BattleScenario, IndependentPlacementDomain},
    behavior::GameError,
    combat_preview::CombatPreview,
    resources::Position,
};

#[cfg(test)]
use crate::game::data::GameDataBase;

#[derive(Debug, Clone)]
pub(crate) struct BattleStartPlan {
    pub(crate) field_size: (u8, u8),
    valid_tiles: Vec<Position>,
    walkable_paths: Vec<Position>,
    static_obstacles: Vec<Position>,
    object_placement: IndependentPlacementDomain,
}

impl BattleStartPlan {
    #[cfg(test)]
    pub(crate) fn from_preview(
        game_data: &GameDataBase,
        encounter_id: &str,
        combat_preview: &CombatPreview,
    ) -> Result<Self, GameError> {
        game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;
        Self::from_resolved_preview(combat_preview)
    }

    /// Builds the field portion after the encounter/preview authority has already
    /// been resolved by the aggregate combat-setup planner.
    pub(crate) fn from_resolved_preview(combat_preview: &CombatPreview) -> Result<Self, GameError> {
        let field_size = (
            checked_preview_dimension(combat_preview.width)?,
            checked_preview_dimension(combat_preview.height)?,
        );

        // Count borrowed input before allocating either new placement domain.
        let scan = combat_preview
            .tiles
            .len()
            .saturating_add(combat_preview.spawn_zones.len())
            .saturating_add(combat_preview.routes.len());
        crate::authoring::control::charge(scan as u64, "object_preview_domain_count")?;
        let ground_count = combat_preview
            .tiles
            .iter()
            .filter(|tile| tile.kind == crate::game::combat_preview::BattlefieldTileKind::Ground)
            .count();
        let reserved_count = combat_preview
            .spawn_zones
            .iter()
            .try_fold(0_usize, |sum, zone| sum.checked_add(zone.cells.len()))
            .and_then(|n| {
                combat_preview
                    .routes
                    .len()
                    .checked_mul(2)
                    .and_then(|routes| n.checked_add(routes))
            });
        let cells = reserved_count.and_then(|n| n.checked_add(ground_count));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            cells.and_then(|n| n.checked_mul(std::mem::size_of::<Position>())),
            "object_preview_domain",
        )?;
        let reserved_count = reserved_count.ok_or(GameError::InvalidAction)?;
        let mut meter = crate::authoring::control::CloneMeter::default();
        // The borrowed count pass above already proved this flat Copy extent.
        // There is no recursive clone-shape traversal here: reserve its slots
        // and charge the one materialization pass without billing a second scan.
        meter.add_storage(cells);
        meter.finish("object_preview_domain", "object_preview_domain_copy")?;
        let sort_work = (reserved_count as u64)
            .saturating_mul(u64::from(usize::BITS - reserved_count.leading_zeros()) + 1);
        crate::authoring::control::charge(sort_work, "object_preview_domain_sort")?;
        // The shared sort debit already forwards to native. Preserve only the
        // separate ground-copy visit here, before materializing either vector.
        crate::game::battle::runtime_control::charge(
            ground_count as u64,
            "object_preview_domain_copy_sort",
        )?;
        let mut ground = Vec::with_capacity(ground_count);
        ground.extend(
            combat_preview
                .tiles
                .iter()
                .filter(|tile| {
                    tile.kind == crate::game::combat_preview::BattlefieldTileKind::Ground
                })
                .map(|tile| tile.position),
        );
        let mut reserved = Vec::with_capacity(reserved_count);
        reserved.extend(
            combat_preview
                .spawn_zones
                .iter()
                .flat_map(|zone| zone.cells.iter().copied())
                .chain(
                    combat_preview
                        .routes
                        .iter()
                        .flat_map(|route| [route.start, route.end]),
                ),
        );
        reserved.sort_unstable_by_key(|position| (position.y, position.x));
        reserved.dedup();
        let path_count = combat_preview.routes.iter().try_fold(0usize, |n,r|n.checked_add(r.cells.len())).ok_or(GameError::InvalidAction)?;
        crate::authoring::control::charge((path_count as u64).saturating_mul(u64::from(usize::BITS-path_count.leading_zeros())+1), "burst_path_union")?;
        crate::game::battle::runtime_control::admit_retained_growth(0, path_count.checked_mul(std::mem::size_of::<Position>()), "burst_path_union")?;
        let mut walkable_paths = Vec::with_capacity(path_count);
        walkable_paths.extend(combat_preview.routes.iter().flat_map(|r|r.cells.iter().copied()));
        walkable_paths.sort_unstable_by_key(|p|(p.y,p.x));
        walkable_paths.dedup();
        Ok(Self {
            walkable_paths,
            field_size,
            valid_tiles: combat_preview.valid_tiles.clone(),
            static_obstacles: combat_preview.obstacles.clone(),
            object_placement: IndependentPlacementDomain { ground, reserved },
        })
    }

    pub(crate) fn into_battlefield_spec(self) -> BattleFieldSpec {
        BattleFieldSpec {
            walkable_paths: self.walkable_paths,
            object_placement: Some(self.object_placement),
            width: self.field_size.0,
            height: self.field_size.1,
            valid_tiles: self.valid_tiles,
            obstacles: self.static_obstacles,
        }
    }
}

pub(crate) fn validate_static_obstacles_do_not_overlap_scenario(
    scenario: &BattleScenario,
) -> Result<(), GameError> {
    for obstacle in &scenario.battlefield.obstacles {
        if scenario
            .groups
            .iter()
            .flat_map(|group| group.spawns.iter())
            .any(|spawn| spawn.position == *obstacle)
        {
            return Err(GameError::StaticObstacleBlocked);
        }
    }

    Ok(())
}

fn checked_preview_dimension(value: i32) -> Result<u8, GameError> {
    u8::try_from(value).map_err(|_| {
        GameError::InvalidStaticData(format!(
            "battlefield dimension must fit u8 and be non-negative: {}",
            value
        ))
    })
}

#[cfg(test)]
mod u07_preview_accounting_tests {
    use super::*;
    use crate::game::{
        battle::runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode},
        combat_preview::*,
        map::MapNodeId,
    };

    #[test]
    fn native_u07_preview_count_sort_and_copy_debit_once_before_materialization() {
        let ground = vec![
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(2, 0),
        ];
        let preview = CombatPreview {
            deterrence: Default::default(),
            node_id: MapNodeId(uuid::Uuid::from_u128(7)),
            encounter_id: None,
            battlefield_template_id: "accounting".into(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: CombatMissionRisk::Controlled,
            archetype: BattlefieldArchetype::OpenHall,
            size_class: BattlefieldSizeClass::Small,
            width: 3,
            height: 2,
            tiles: ground
                .iter()
                .copied()
                .map(|position| BattlefieldTile {
                    position,
                    kind: BattlefieldTileKind::Ground,
                })
                .chain([BattlefieldTile {
                    position: Position::new(1, 1),
                    kind: BattlefieldTileKind::Platform,
                }])
                .collect(),
            valid_tiles: ground.clone(),
            deployment_zones: vec![],
            spawn_zones: vec![SpawnZone {
                id: "entry".into(),
                label: "Entry".into(),
                kind: SpawnZoneKind::Entry,
                confidence: ZoneConfidence::Confirmed,
                cells: vec![Position::new(2, 0), Position::new(0, 0)],
                revealed_details: vec![],
            }],
            routes: vec![BattlefieldRoute {
                id: "route".into(),
                start: Position::new(0, 0),
                end: Position::new(2, 0),
                cells: ground.clone(),
            }],
            spawn_waves: vec![],
            obstacles: vec![],
            enemy_briefing: vec![],
            threat_warnings: vec![],
            enemy_stat_scale: Default::default(),
        };
        let before = serde_json::to_string(&preview).unwrap();
        // Six borrowed records; seven flat Copy cells admitted and copied;
        // four endpoints at sixteen sort units; three extra ground visits;
        // three path cells at three scan/sort units each, plus three storage
        // admission units (no active waves required).
        for (requested, site) in [
            (6, "object_preview_domain_count"),
            (13, "object_preview_domain"),
            (20, "object_preview_domain_copy"),
            (36, "object_preview_domain_sort"),
            (39, "object_preview_domain_copy_sort"),
            (48, "burst_path_union"),
            (51, "burst_path_union"),
        ] {
            let (short, admissions) = native::observe_retained_admissions(|| {
                native::run(
                    RuntimeFactLimits {
                        max_work: requested - 1,
                        ..Default::default()
                    },
                    || BattleStartPlan::from_resolved_preview(&preview),
                )
            });
            let error = short.unwrap_err();
            assert!(
                matches!(error, GameError::BattleRuntimeFault(fault)
                if fault.code == RuntimeFaultCode::WorkLimit && fault.site == site && fault.requested == Some(requested)),
                "{error:?}"
            );
            if requested == 6 {
                assert!(admissions.is_empty());
            }
            assert_eq!(serde_json::to_string(&preview).unwrap(), before);
        }
        let plan = native::run(
            RuntimeFactLimits {
                max_work: 51,
                ..Default::default()
            },
            || BattleStartPlan::from_resolved_preview(&preview),
        )
        .unwrap();
        assert_eq!(plan.object_placement.ground, ground);
        assert_eq!(plan.walkable_paths, ground);
        assert_eq!(
            plan.object_placement.reserved,
            vec![Position::new(0, 0), Position::new(2, 0)]
        );
        assert_eq!(serde_json::to_string(&preview).unwrap(), before);
        let mut multiple = preview.clone();
        multiple.routes[0].cells=vec![Position::new(0,0),Position::new(1,0)];
        multiple.routes.push(BattlefieldRoute {id:"other".into(),start:Position::new(1,0),end:Position::new(2,0),cells:vec![Position::new(1,0),Position::new(2,0)]});
        assert!(multiple.spawn_waves.is_empty());
        let all=native::run(RuntimeFactLimits::default(),||BattleStartPlan::from_resolved_preview(&multiple)).unwrap().into_battlefield_spec();
        assert_eq!(all.walkable_paths,ground,"all routes contribute once even without a wave");
    }
}
