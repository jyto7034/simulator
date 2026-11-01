mod composition_multi_target {
    use super::*;
    use crate::game::{
        ability::sustained::HitStatusKind, data::abnormality_data::BasicAttackAction,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/multi_target_basic.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture(0, false, |doc| {
            *doc = parse_yaml(SOURCE).unwrap();
            edit(doc);
        });
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.basic_attack.action = BasicAttackAction::DamageTargets { max_targets: 3 };
        u.basic_attack.delivery = ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
        u.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.damage_type = crate::game::battle::damage::DamageType::Magic;
        u.basic_attack.windup_ms = 80;
        u.stats.attack = 100;
        for id in [f.enemy, f.secondary] {
            f.core.units.get_mut(&id).unwrap().stats.magic_resist = 0;
        }
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
        f
    }
    fn step(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    #[test]
    fn basic_multi_target_authored_slow_each_victim_and_copy_source() {
        let mut f = setup(|_| {});
        for id in [f.enemy, f.secondary] {
            f.core
                .units
                .get_mut(&id)
                .unwrap()
                .stats
                .move_speed_units_per_ms = 1000;
        }
        attack(&mut f, 10, true);
        step(&mut f, 800);
        let states = f.core.sustained_effect_snapshot().unwrap().active;
        for id in [f.enemy, f.secondary] {
            assert_eq!(f.core.units[&id].stats.current_health, 900);
            assert_eq!(f.core.units[&id].stats.move_speed_units_per_ms, 800);
            assert!(states.iter().any(|s| s.caster == f.a
                && s.target == id
                && s.hit_status
                    .as_ref()
                    .is_some_and(|h| matches!(h.kind, HitStatusKind::Slow { percent: 20 }))));
        }
        assert_eq!(states.iter().filter(|s| s.hit_status.is_some()).count(), 2);
        assert_eq!(f.core.units[&f.b].stats.current_health, 1000);
        step(&mut f, 2300);
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .all(|s| s.hit_status.is_none()));
        for id in [f.enemy, f.secondary] {
            assert_eq!(f.core.units[&id].stats.move_speed_units_per_ms, 1000);
        }
    }
    #[test]
    fn basic_multi_target_lifesteal_sums_once_before_rounding_and_cap() {
        for (hp, cap, expected) in [(3, 100, 1), (1000, 17, 17), (1000, 100, 50)] {
            let mut f = setup(|doc| {
                program(doc)["basic_lifesteal"] = json!({"percent":25,"max_heal":cap});
            });
            for id in [f.enemy, f.secondary] {
                f.core.units.get_mut(&id).unwrap().stats.current_health = hp;
            }
            attack(&mut f, 10, true);
            step(&mut f, 800);
            assert_eq!(f.core.units[&f.a].stats.current_health, 500 + expected);
            assert_eq!(
                f.core.runtime_fact_summary().healing_delta,
                i64::from(expected)
            );
            assert!(f.core.basic_projectile_batches.is_empty());
        }
    }
    #[test]
    fn basic_multi_target_lifesteal_waits_for_last_miss_and_source_exit_cancels_heal() {
        for withdraw in [false, true] {
            let mut f = setup(|doc| {
                program(doc)["basic_lifesteal"] = json!({"percent":25,"max_heal":100});
            });
            attack(&mut f, 10, true);
            step(&mut f, 250);
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
            assert_eq!(f.core.units[&f.secondary].stats.current_health, 1000);
            assert_eq!(f.core.units[&f.a].stats.current_health, 500);
            f.core
                .units
                .get_mut(&f.secondary)
                .unwrap()
                .stats
                .current_health = 0;
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 251,
                    })
                    .unwrap();
            }
            step(&mut f, 800);
            assert_eq!(
                f.core.units[&f.a].stats.current_health,
                if withdraw { 500 } else { 525 }
            );
            assert!(f.core.basic_projectile_batches.is_empty());
        }
    }
    #[test]
    fn basic_multi_target_does_not_become_an_area_damage_judgment() {
        let area = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/area_batch.skill.yaml"
        )))
        .unwrap();
        let mut f = setup(|doc| {
            program(doc)["area_damage_percent"] =
                area["values"][0]["value"]["AttackProgram"]["area_damage_percent"].clone()
        });
        attack(&mut f, 10, true);
        step(&mut f, 800);
        for id in [f.enemy, f.secondary] {
            assert_eq!(f.core.units[&id].stats.current_health, 900);
        }
    }
    #[test]
    fn basic_multi_target_slow_reapplication_strongest_and_released_source_exit() {
        for withdraw in [false, true] {
            let mut f = setup(|_| {});
            for id in [f.enemy, f.secondary] {
                f.core
                    .units
                    .get_mut(&id)
                    .unwrap()
                    .stats
                    .move_speed_units_per_ms = 1000;
            }
            attack(&mut f, 10, true);
            step(&mut f, 500);
            attack(&mut f, 600, true);
            step(&mut f, 690);
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 690,
                    })
                    .unwrap();
            }
            step(&mut f, 1100);
            for id in [f.enemy, f.secondary] {
                assert_eq!(f.core.units[&id].stats.current_health, 800);
                assert_eq!(f.core.units[&id].stats.move_speed_units_per_ms, 800);
            }
            step(&mut f, 2100);
            assert_eq!(f.core.units[&f.enemy].stats.move_speed_units_per_ms, 800);
            step(&mut f, 2600);
            for id in [f.enemy, f.secondary] {
                assert_eq!(f.core.units[&id].stats.move_speed_units_per_ms, 1000);
            }
        }
    }
    #[test]
    fn basic_multi_target_slow_immunity_preserves_damage() {
        let mut f = setup(|_| {});
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::SlowImmune);
        attack(&mut f, 10, true);
        step(&mut f, 800);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
        let effects = f.core.sustained_effect_snapshot().unwrap().active;
        assert!(effects
            .iter()
            .all(|e| e.target != f.enemy || e.hit_status.is_none()));
        assert!(effects
            .iter()
            .any(|e| e.target == f.secondary && e.hit_status.is_some()));
    }
}
