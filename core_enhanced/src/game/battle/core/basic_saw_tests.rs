mod saw_contract {
    use super::*;
    use crate::game::battle::{
        core::movement::types::{ActiveMovementSegment, WorldVec2},
        damage::DamageType,
        tile_range::FacingDirection,
    };

    fn setup(extra: u32) -> Fixture {
        let equipment: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let profile = equipment
            .get_by_id("placeholder_schadenfreude_weapon")
            .unwrap()
            .weapon_profile
            .clone()
            .unwrap();
        assert!(matches!(
            profile.action,
            BasicAttackAction::DamageOccupiedTiles
        ));
        assert_eq!(profile.block_capacity, 2);
        assert!(!profile.air_capable);
        assert_eq!(profile.damage_type, DamageType::Physical);
        let interval = profile.interval_ms;
        let mut f = fixture_profile(extra, profile);
        let actor = f.core.units.get_mut(&f.actor).unwrap();
        actor.stats.attack_interval_ms = interval;
        actor.facing_direction = Some(FacingDirection::Right);
        f
    }

    fn start_saw(f: &mut Fixture, now: u64) {
        f.core.units.get_mut(&f.actor).unwrap().next_basic_attack_ms = now;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: f.actor,
                    target_instance_id: Some(f.primary),
                    schedule_next: true,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
        let target = f.core.units[&f.actor]
            .current_target
            .expect("eligible front occupant");
        assert_eq!(f.core.units[&target].owner, Side::Opponent);
    }

    #[test]
    fn saw_catalog_all_five_occupants_one_front_tile_four_facings() {
        for (direction, target) in [
            (FacingDirection::Up, Position::new(3, 1)),
            (FacingDirection::Right, Position::new(4, 2)),
            (FacingDirection::Down, Position::new(3, 3)),
            (FacingDirection::Left, Position::new(2, 2)),
        ] {
            let mut f = setup(3);
            place(&mut f.core, f.actor, 3, 2);
            place(&mut f.core, f.ally, target.x, target.y);
            f.core.units.get_mut(&f.actor).unwrap().facing_direction = Some(direction);
            let enemies: Vec<_> = f
                .core
                .units
                .values()
                .filter(|u| u.owner == Side::Opponent)
                .map(|u| u.instance_id)
                .collect();
            assert_eq!(enemies.len(), 5);
            for id in &enemies {
                place(&mut f.core, *id, target.x, target.y);
            }
            start_saw(&mut f, 10);
            run(&mut f, 129);
            assert!(enemies.iter().all(|id| hp(&f, *id) == 1000));
            run(&mut f, 130);
            assert!(enemies.iter().all(|id| hp(&f, *id) == 900));
            assert_eq!(hp(&f, f.ally), 1000);
            let releases: Vec<_> = f
                .core
                .event_log
                .entries
                .iter()
                .filter_map(|entry| match &entry.event {
                    BattleLogEvent::BasicAttackTilesReleased {
                        attacker_instance_id,
                        affected_tiles,
                    } if *attacker_instance_id == f.actor => {
                        Some((entry.time_ms, affected_tiles.clone()))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(releases, vec![(130, vec![target])]);
        }
    }

    #[test]
    fn saw_catalog_single_tile_boundary_air_and_repeated_cadence() {
        for (x, y, air, expected) in [
            (3.0, 1.5, false, 800),
            (3.9999, 1.5, false, 800),
            (4.0, 1.5, false, 1000),
            (3.5, 2.0, false, 1000),
            (2.5, 1.5, false, 1000),
            (3.5, 1.5, true, 1000),
        ] {
            let mut f = setup(0);
            let other = f.core.units.get_mut(&f.other).unwrap();
            other.set_world_position(WorldVec2::new(x, y));
            if air {
                other.mobility_kind = MobilityKind::Airborne;
            }
            start_saw(&mut f, 10);
            run(&mut f, 880);
            assert_eq!(hp(&f, f.primary), 800);
            assert_eq!(hp(&f, f.other), expected, "({x},{y}), air={air}");
            let releases: Vec<_> = f.core.event_log.entries.iter().filter_map(|entry| {
                matches!(entry.event, BattleLogEvent::BasicAttackTilesReleased { attacker_instance_id, .. }
                    if attacker_instance_id == f.actor).then_some(entry.time_ms)
            }).collect();
            assert_eq!(releases, vec![130, 880]);
        }
    }

    #[test]
    fn saw_catalog_samples_current_movement_at_release_and_requires_live_primary() {
        for primary in [false, true] {
            let mut f = setup(0);
            place(&mut f.core, f.other, 3, 1);
            let moving = if primary { f.primary } else { f.other };
            f.core.active_movement_segments.insert(
                moving,
                ActiveMovementSegment {
                    start: WorldVec2::new(3.5, 1.5),
                    target: WorldVec2::new(4.5, 1.5),
                    started_at_ms: 10,
                    ends_at_ms: 130,
                },
            );
            assert_eq!(
                f.core.live_unit_projected_tile(moving),
                Some(Position::new(3, 1))
            );
            // Observe the exact release-time sample before a scheduler movement
            // policy can legitimately replace this explicitly supplied segment.
            assert_eq!(
                f.core.resolve_basic_attack(f.actor, f.primary, 130),
                !primary
            );
            assert_eq!(hp(&f, moving), 1000);
            assert_eq!(
                hp(&f, if primary { f.other } else { f.primary }),
                if primary { 1000 } else { 900 }
            );
        }
    }
}
