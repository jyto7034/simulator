mod composition_nearby_defense {
    use super::*;
    use crate::game::{
        ability::sustained::SustainedEffectDef,
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            tile_range::{FacingDirection, TileRangePattern},
        },
        resources::Position,
        stats::{StatId, StatModifier, StatModifierKind},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/nearby_defense.skill.yaml"
    ));
    fn setup(per_enemy: u32, cap: u32) -> Fixture {
        let mut f = fixture(0, false, |doc| {
            *doc = parse_yaml(SOURCE).unwrap();
            doc["values"][0]["value"]["DefenseProgram"]["NearbyHostileDefense"]["per_enemy"] =
                json!(per_enemy);
            doc["values"][0]["value"]["DefenseProgram"]["NearbyHostileDefense"]["max_bonus"] =
                json!(cap);
        });
        // The shared Native fixture installs explicit stats after startup. Reset
        // its old baseline along with those stats before testing composition.
        f.core.sustained = Default::default();
        for unit in f.core.units.values_mut() {
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
        }
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0x7100),
                },
            )
            .unwrap();
        refresh(&mut f.core, 0).unwrap();
        f
    }
    fn refresh(core: &mut BattleCore, now: u64) -> Result<(), GameError> {
        core.runtime_transaction(|c| c.reconcile_sustained(now))
    }
    fn place(core: &mut BattleCore, id: UnitInstanceId, x: i32, y: i32) {
        super::super::compiled_action_fixture::place(core, id, x, y);
    }
    fn defense(f: &Fixture, id: UnitInstanceId) -> i32 {
        f.core.units[&id].stats.defense
    }
    fn snap(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    fn damage(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageType,
        amount: u32,
        now: u64,
    ) {
        core.runtime_transaction(|c| {
            let source = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    DamageSource::Ability,
                    kind,
                    amount,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: source,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
    }
    #[test]
    fn composition_nearby_defense_roundtrip_and_precise_diagnostics() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(restored.equipment_defenses()[0].program, crate::game::ability::equipment_defense::EquipmentDefenseProgram::NearbyHostileDefense { per_enemy: 10, max_bonus: 25, .. }));
        for (field, value) in [
            ("per_enemy", json!(0)),
            ("per_enemy", json!(2147483648u64)),
            ("max_bonus", json!(0)),
            ("max_bonus", json!(2147483648u64)),
            ("range", json!({"rows":["@".repeat(34)]})),
        ] {
            let mut doc = parse_yaml(SOURCE).unwrap();
            doc["values"][0]["value"]["DefenseProgram"]["NearbyHostileDefense"][field] = value;
            let e = compile_json(&doc.to_string()).unwrap_err();
            assert!(e.source.definition.ends_with(field), "{e:?}");
        }
    }
    #[test]
    fn composition_nearby_defense_current_count_cap_and_independent_copies() {
        let mut f = setup(20, 25);
        assert_eq!((defense(&f, f.a), defense(&f, f.b)), (20, 20));
        place(&mut f.core, f.secondary, 3, 1); // Same tile still two distinct opponents.
        refresh(&mut f.core, 1).unwrap();
        assert_eq!((defense(&f, f.a), defense(&f, f.b)), (25, 0));
        place(&mut f.core, f.enemy, 7, 3);
        place(&mut f.core, f.secondary, 7, 2);
        refresh(&mut f.core, 2).unwrap();
        assert_eq!((defense(&f, f.a), defense(&f, f.b)), (0, 25));
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_nearby_defense
            .as_ref()
            .unwrap();
        assert_eq!(
            w.bonus,
            w.contributors.iter().map(|c| c.bonus as i32).sum::<i32>()
        );
        assert!(w.contributors[0]
            .field_path
            .ends_with("NearbyHostileDefense"));
        assert_eq!(
            Some(w.contributors[0].equipment.persistent_copy_uuid.unwrap()),
            match owner(&f.core, w.actor) {
                ResourceOwner::EquipmentCopy(id) => Some(id),
                _ => None,
            }
        );
    }
    #[test]
    fn composition_nearby_defense_alive_air_friends_corrosion_and_withdrawal() {
        let mut f = setup(10, 25);
        f.core.units.get_mut(&f.enemy).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        refresh(&mut f.core, 1).unwrap();
        assert_eq!(defense(&f, f.a), 10);
        f.core.units.get_mut(&f.enemy).unwrap().owner = Side::Player;
        refresh(&mut f.core, 2).unwrap();
        assert_eq!(defense(&f, f.a), 0);
        f.core.units.get_mut(&f.enemy).unwrap().owner = Side::Opponent;
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        refresh(&mut f.core, 3).unwrap();
        assert_eq!(defense(&f, f.a), 0);
        f.core.units.get_mut(&f.a).unwrap().corroded = false;
        refresh(&mut f.core, 4).unwrap();
        assert_eq!(defense(&f, f.a), 10);
        damage(&mut f.core, f.a, f.enemy, DamageType::True, 10000, 5);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        refresh(&mut f.core, 5).unwrap();
        assert_eq!(defense(&f, f.a), 0);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.b,
                time_ms: 6,
            })
            .unwrap();
        assert_eq!(defense(&f, f.b), 0);
    }
    #[test]
    fn composition_nearby_defense_samples_both_moving_centers_and_rotated_authored_shape() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        let mut f = setup(10, 25);
        // The simulation stores the segment endpoint on the body. Current
        // observation at 10ms must instead see its origin beside the wearer.
        place(&mut f.core, f.enemy, 7, 1);
        f.core.active_movement_segments.insert(
            f.enemy,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(3, 1)),
                target: WorldVec2::from_tile_center(Position::new(7, 1)),
                started_at_ms: 10,
                ends_at_ms: 110,
            },
        );
        refresh(&mut f.core, 10).unwrap();
        assert_eq!(defense(&f, f.a), 10);
        refresh(&mut f.core, 110).unwrap();
        assert_eq!(defense(&f, f.a), 0);
        place(&mut f.core, f.a, 7, 1);
        f.core.active_movement_segments.insert(
            f.a,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(2, 1)),
                target: WorldVec2::from_tile_center(Position::new(7, 1)),
                started_at_ms: 110,
                ends_at_ms: 210,
            },
        );
        refresh(&mut f.core, 110).unwrap();
        assert_eq!(defense(&f, f.a), 0);
        refresh(&mut f.core, 210).unwrap();
        assert_eq!(defense(&f, f.a), 10);
        let mut doc = parse_yaml(SOURCE).unwrap();
        doc["values"][0]["value"]["DefenseProgram"]["NearbyHostileDefense"]["range"] =
            json!({"include_anchor_tile":false,"rows":["X","@"]});
        let mut g = fixture(0, false, |d| *d = doc);
        g.core.sustained = Default::default();
        for u in g.core.units.values_mut() {
            u.stats.defense = 0;
        }
        g.core.units.get_mut(&g.a).unwrap().facing_direction = Some(FacingDirection::Right);
        refresh(&mut g.core, 0).unwrap();
        assert_eq!(defense(&g, g.a), 10);
        g.core.units.get_mut(&g.a).unwrap().facing_direction = Some(FacingDirection::Left);
        refresh(&mut g.core, 1).unwrap();
        assert_eq!(defense(&g, g.a), 0);
    }
    #[test]
    fn composition_nearby_defense_permanent_percent_negative_baseline_no_compounding_and_terminal_clear(
    ) {
        let mut f = setup(20, 25);
        let current = f.core.units[&f.a].stats;
        let stats = f.core.apply_permanent_with_sustained(
            f.a,
            current,
            StatModifier {
                stat: StatId::Defense,
                kind: StatModifierKind::Flat,
                value: -10,
            },
        );
        f.core.units.get_mut(&f.a).unwrap().stats = stats;
        refresh(&mut f.core, 0).unwrap();
        assert_eq!(defense(&f, f.a), 10);
        let a = f.a;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::Modifiers {
                        duration_ms: 200,
                        modifiers: vec![StatModifier {
                            stat: StatId::Defense,
                            kind: StatModifierKind::Percent,
                            value: 50,
                        }],
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(defense(&f, a), 15);
        let now = f.core.units[&a].stats;
        let stats = f.core.apply_permanent_with_sustained(
            a,
            now,
            StatModifier {
                stat: StatId::Defense,
                kind: StatModifierKind::Flat,
                value: 20,
            },
        );
        f.core.units.get_mut(&a).unwrap().stats = stats;
        assert_eq!(defense(&f, a), 45); // (-10 + 20 permanent + 20 nearby) * 1.5
        for t in [1, 10, 100] {
            refresh(&mut f.core, t).unwrap();
            assert_eq!(defense(&f, a), 45);
        }
        refresh(&mut f.core, 200).unwrap();
        assert_eq!(defense(&f, a), 30);
        f.core
            .runtime_transaction(|c| c.clear_sustained(201))
            .unwrap();
        assert_eq!(defense(&f, a), 10);
        assert!(!f.core.has_sustained_stat_base(a));
    }
    #[test]
    fn composition_nearby_defense_actual_physical_damage_and_low_defense_target_consume_stat() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup(20, 25);
            let mut off = f.core.clone();
            off.units.get_mut(&f.a).unwrap().corroded = true;
            refresh(&mut off, 1).unwrap();
            damage(&mut f.core, f.enemy, f.a, kind, 100, 1);
            damage(&mut off, f.enemy, f.a, kind, 100, 1);
            if kind == DamageType::Physical {
                assert!(
                    f.core.units[&f.a].stats.current_health > off.units[&f.a].stats.current_health
                );
            } else {
                assert_eq!(
                    f.core.units[&f.a].stats.current_health,
                    off.units[&f.a].stats.current_health
                );
            }
        }
        let mut f = setup(20, 25);
        place(&mut f.core, f.b, 5, 1);
        place(&mut f.core, f.secondary, 7, 3);
        let u = f.core.units.get_mut(&f.enemy).unwrap();
        u.basic_attack.targeting_profile =
            crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
        u.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXXXXXXXX".into(), "XXXX@XXXX".into(), "XXXXXXXXX".into()],
        });
        u.facing_direction = Some(FacingDirection::Up);
        refresh(&mut f.core, 1).unwrap();
        assert_eq!((defense(&f, f.a), defense(&f, f.b)), (20, 0));
        assert_eq!(f.core.choose_attack_target_in_range(f.enemy), Some(f.b));
    }
    #[test]
    fn composition_nearby_defense_cancellation_and_observation_budget_restore_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(10, 25);
        place(&mut f.core, f.secondary, 3, 1);
        let before = snap(&f.core);
        let token = CancellationToken::default();
        let outcome = control::with_test_cancellation("nearby_defense_reconciled", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || refresh(&mut f.core, 1),
            )
        });
        assert!(outcome.is_err() && token.is_cancelled());
        assert_eq!(snap(&f.core), before);
        let mut probe = f.core.clone();
        let (result, admissions) =
            crate::game::battle::runtime_control::observe_retained_admissions(|| {
                refresh(&mut probe, 1)
            });
        result.unwrap();
        let needed = admissions
            .iter()
            .find(|(site, _, _)| *site == "nearby_defense_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = needed - 1;
        let before = snap(&f.core);
        let e = refresh(&mut f.core, 1).unwrap_err();
        assert!(matches!(e,GameError::BattleRuntimeFault(crate::game::battle::runtime_control::BattleRuntimeFault { code:crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,site:"nearby_defense_observation",.. })),"{e:?}");
        assert_eq!(snap(&f.core), before);
        f.core.runtime_facts.limits = Default::default();
        let mut off = f.core.clone();
        off.authoring_observation = None;
        refresh(&mut f.core, 1).unwrap();
        refresh(&mut off, 1).unwrap();
        assert_eq!(snap(&f.core), snap(&off));
        assert_eq!(defense(&f, f.a), 20);
    }
}
