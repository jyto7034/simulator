mod composition_actual_heal_reaction {
    use super::*;
    const SOURCE: &str = include_str!(
        "../../../../tests/fixtures/skill_composition/actual_heal_reaction.skill.yaml"
    );
    fn combined_source() -> String {
        let mut v: serde_json::Value = serde_json::to_value(parse_yaml(SOURCE).unwrap()).unwrap();
        v["values"][0]["value"]["SupportProgram"]["actual_heal_reaction"]["target_haste"] = v
            ["values"][1]["value"]["SupportProgram"]["actual_heal_reaction"]["target_haste"]
            .clone();
        v["equipment_supports"][0]["equipment"] = json!("r11_weapon");
        v["equipment_supports"].as_array_mut().unwrap().truncate(1);
        serde_json::to_string_pretty(&v).unwrap()
    }
    fn setup_with(s: &str) -> Fixture {
        fixture_extra_with_defense("Heal(amount:20)", false, None, false, s, |_| {}, |_| {})
    }
    fn setup() -> Fixture {
        setup_with(&combined_source())
    }
    #[test]
    fn shield_beneficiary_actual_heal_pool_uses_existing_balance_and_depletion() {
        let mut doc: serde_json::Value = serde_json::from_str(&combined_source()).unwrap();
        doc["values"][0]["value"]["SupportProgram"]["shield_beneficiary_damage_percent"] =
            json!(20);
        let mut f = setup_with(&doc.to_string());
        let (a, b, t) = (f.caster, f.ally, f.target);
        f.core.units.get_mut(&b).unwrap().stats.current_health = 900;
        heal(&mut f, a, b, 100, 1);
        assert_eq!(pool(&f.core, a), vec![(200, 25, 5001)]);
        let apply = |c: &mut BattleCore, actor, target, now| {
            let shot = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    DamageSource::Ability,
                    DamageType::True,
                    100,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            let before = c.units[&target].stats.current_health;
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: shot,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()?;
            Ok(before - c.units[&target].stats.current_health)
        };
        assert_eq!(
            f.core.runtime_transaction(|c| apply(c, a, t, 2)).unwrap(),
            120
        );
        assert_eq!(
            f.core.runtime_transaction(|c| apply(c, t, a, 3)).unwrap(),
            75
        );
        assert_eq!(
            f.core.runtime_transaction(|c| apply(c, a, t, 4)).unwrap(),
            100
        );
    }
    fn heal(f: &mut Fixture, a: UnitInstanceId, t: UnitInstanceId, amount: i32, now: u64) {
        f.core
            .runtime_transaction(|c| {
                c.reconcile_sustained(now)?;
                c.apply_direct_heal_and_record(Some(a), t, amount, now)
            })
            .unwrap();
    }
    fn pool(core: &BattleCore, who: UnitInstanceId) -> Vec<(u32, u32, u64)> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|r| r.target == who)
            .filter_map(|r| r.shield.map(|p| (p.capacity, p.remaining, r.expires_at_ms)))
            .collect()
    }
    fn w(f: &Fixture) -> &crate::game::battle::core::equipment_support::ActualHealReactionWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_actual_heal_reaction
            .as_ref()
            .unwrap()
    }
    #[test]
    fn actual_heal_reaction_roundtrip_strict_bounds_shared_origin_and_new_input() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(restored.equipment_supports().len(), 2);
        assert!(restored
            .sources()
            .get("equipment_supports[0].program")
            .is_some());
        for (from, to) in [
            ("actual_heal_percent: 25", "actual_heal_percent: 0"),
            ("actual_heal_percent: 25", "actual_heal_percent: 10001"),
            ("max_hp_cap_percent: 20", "max_hp_cap_percent: 101"),
            ("max_hp_cap_percent: 20", "max_hp_cap_percent: 0"),
            ("duration_ms: 5000", "duration_ms: 0"),
            ("attack_speed_points: 30", "attack_speed_points: 10001"),
            ("attack_speed_points: 30", "attack_speed_points: 0"),
            ("duration_ms: 3000", "duration_ms: 0"),
            ("priority: 0", "unknown: 0"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).is_err(),
                "{to}"
            );
        }
        let mut f = setup_with(
            &combined_source()
                .replace("\"actual_heal_percent\": 25", "\"actual_heal_percent\": 50")
                .replace("\"attack_speed_points\": 30", "\"attack_speed_points\": 60"),
        );
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 900;
        heal(&mut f, a, t, 20, 1);
        assert_eq!(pool(&f.core, a), vec![(200, 10, 5001)]);
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 625);
    }
    #[test]
    fn actual_heal_reaction_actual_amount_overheal_self_and_nonheal_exclusions() {
        for case in 0..9 {
            let mut f = setup();
            let a = f.caster;
            let t = if case == 1 { a } else { f.ally };
            f.core.units.get_mut(&t).unwrap().stats.current_health =
                if case == 0 { 1000 } else { 995 };
            match case {
                2 => f.core.units.get_mut(&a).unwrap().corroded = true,
                3 => f.core.units.get_mut(&a).unwrap().stats.current_health = 0,
                4 => {
                    f.core.units.get_mut(&a).unwrap().lifecycle =
                        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                }
                5 => f.core.units.get_mut(&t).unwrap().owner = Side::Opponent,
                6 => {
                    f.core.units.get_mut(&t).unwrap().role =
                        crate::game::battle::types::BattleUnitRole::IndependentWall
                }
                _ => (),
            }
            if case == 7 {
                f.core.apply_hp_delta_and_record(
                    Some(a),
                    t,
                    10,
                    1,
                    crate::game::battle::event_log::HpChangeReason::Command,
                );
            } else {
                heal(&mut f, a, t, 20, 1);
            }
            if case == 8 {
                assert_eq!(pool(&f.core, a), vec![(200, 1, 5001)]);
                assert_eq!(w(&f).actual_healed, 5);
                assert_eq!(
                    w(&f).contributors[0].self_shield.unwrap().remainder_after,
                    25
                );
            } else {
                assert!(pool(&f.core, a).is_empty(), "case{case}");
                assert!(
                    f.core
                        .authoring_observation
                        .as_ref()
                        .unwrap()
                        .latest_actual_heal_reaction
                        .is_none(),
                    "case{case}"
                );
            }
        }
    }
    #[test]
    fn actual_heal_reaction_multiple_targets_fraction_cap_and_no_free_regeneration() {
        let mut f = setup();
        let (a, t, b) = (f.caster, f.ally, f.target);
        f.core.units.get_mut(&b).unwrap().owner = f.core.units[&a].owner;
        for id in [t, b] {
            f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        }
        for (n, id) in [t, b, t, b].into_iter().enumerate() {
            heal(&mut f, a, id, 1, n as u64 + 1);
        }
        assert_eq!(pool(&f.core, a), vec![(200, 1, 5004)]);
        heal(&mut f, a, t, 899, 5);
        assert_eq!(pool(&f.core, a), vec![(200, 200, 5005)]);
        assert_eq!(
            w(&f).contributors[0].self_shield.unwrap().remainder_after,
            0
        );
        f.core
            .runtime_transaction(|c| c.absorb_sustained_shields(a, 199, 6))
            .unwrap();
        assert_eq!(pool(&f.core, a), vec![(200, 1, 5005)]);
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(7))
            .unwrap();
        assert_eq!(pool(&f.core, a), vec![(200, 1, 5005)]);
        heal(&mut f, a, b, 1, 8);
        assert_eq!(pool(&f.core, a), vec![(200, 1, 5008)]);
        assert_eq!(
            w(&f).contributors[0].self_shield.unwrap().remainder_after,
            25
        );
    }
    #[test]
    fn actual_heal_reaction_depleted_rebuild_and_live_max_hp_cap() {
        let mut f = setup();
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 100, 1);
        assert_eq!(pool(&f.core, a), vec![(200, 25, 5001)]);
        f.core.units.get_mut(&a).unwrap().stats.max_health = 50;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(2))
            .unwrap();
        assert_eq!(pool(&f.core, a), vec![(10, 10, 5001)]);
        f.core.units.get_mut(&a).unwrap().stats.max_health = 1000;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3))
            .unwrap();
        assert_eq!(pool(&f.core, a), vec![(200, 10, 5001)]);
        f.core
            .runtime_transaction(|c| c.absorb_sustained_shields(a, 10, 4))
            .unwrap();
        assert!(pool(&f.core, a).is_empty());
        heal(&mut f, a, t, 8, 5);
        assert_eq!(pool(&f.core, a), vec![(200, 2, 5005)]);
    }
    #[test]
    fn actual_heal_reaction_haste_refresh_one_record_expiry_and_source_exit() {
        let mut f = setup();
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 4, 1);
        let count = f.core.sustained_effect_snapshot().unwrap().active.len();
        let queue_count = f.core.event_queue.len();
        for at in 2..102 {
            heal(&mut f, a, t, 1, at);
        }
        assert_eq!(f.core.event_queue.len(), queue_count);
        assert_eq!(
            f.core.sustained_effect_snapshot().unwrap().active.len(),
            count
        );
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 770);
        assert!(w(&f).contributors[0].target_haste.unwrap().refreshed);
        f.core.units.get_mut(&a).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(102))
            .unwrap();
        assert!(pool(&f.core, a).is_empty());
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 770);
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3101))
            .unwrap();
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 1000);
    }
    #[test]
    fn actual_heal_reaction_copy_transfer_clears_shield_and_fraction() {
        let mut f = setup();
        let (a, t, b) = (f.caster, f.ally, f.target);
        f.core.units.get_mut(&b).unwrap().owner = f.core.units[&a].owner;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 7, 1);
        let id = w(&f).contributors[0].equipment.runtime_item_instance_id;
        f.core.items.get_mut(&id).unwrap().owner_unit_instance = b;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(2))
            .unwrap();
        assert!(pool(&f.core, a).is_empty());
        heal(&mut f, b, t, 1, 3);
        assert!(pool(&f.core, b).is_empty());
        assert_eq!(
            w(&f)
                .contributors
                .iter()
                .find(|c| c.equipment.runtime_item_instance_id == id)
                .unwrap()
                .self_shield
                .unwrap()
                .remainder_before,
            0
        );
    }
    #[test]
    fn actual_heal_reaction_fraction_expires_terminal_stops_and_clones_isolate() {
        let mut f = setup();
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 3, 1);
        let mut branch = f.core.clone();
        branch
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 1, 2))
            .unwrap();
        assert_eq!(pool(&branch, a), vec![(200, 1, 5002)]);
        assert!(pool(&f.core, a).is_empty());
        heal(&mut f, a, t, 1, 5001);
        assert!(pool(&f.core, a).is_empty());
        assert_eq!(
            w(&f).contributors[0].self_shield.unwrap().remainder_before,
            0
        );
        f.core
            .runtime_transaction(|c| c.clear_sustained(5002))
            .unwrap();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        heal(&mut f, a, t, 100, 5003);
        assert!(pool(&f.core, a).is_empty());
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap()
                ["latest_actual_heal_reaction"],
            obs["latest_actual_heal_reaction"]
        );
    }
    #[test]
    fn actual_heal_reaction_cancellation_restores_hp_pool_speed_fraction_and_observer() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result =
            control::with_test_cancellation("actual_heal_reaction_provider_applied", 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            c.apply_direct_heal_and_record(Some(a), t, 20, 1)
                        })
                    },
                )
            });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        heal(&mut f, a, t, 20, 1);
        assert_eq!(pool(&f.core, a), vec![(200, 5, 5001)]);
    }
    #[test]
    fn actual_heal_reaction_observer_and_retained_failure_preserve_authority() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let (a, t) = (f.caster, f.ally);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| {
                c.reconcile_sustained(1)?;
                c.apply_direct_heal_and_record(Some(a), t, 20, 1)
            })
            .unwrap();
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 20, 1))
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(s, _, _)| *s == "actual_heal_shield_state")
            .map(|(_, _, b)| *b)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        assert!(f
            .core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 20, 1))
            .is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        f.core.runtime_facts.limits = Default::default();
        heal(&mut f, a, t, 20, 1);
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            plain.transaction_debug_snapshot()
        );
    }
    #[test]
    fn actual_heal_reaction_typed_damage_uses_existing_priority_and_hp_absorption() {
        use crate::game::ability::sustained::SustainedEffectDef;
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup();
            let (a, t, e) = (f.caster, f.ally, f.target);
            f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
            heal(&mut f, a, t, 20, 1);
            f.core
                .runtime_transaction(|c| {
                    c.begin_sustained(
                        a,
                        a,
                        SustainedEffectDef::IndependentShield {
                            duration_ms: 5000,
                            amount: 3,
                            priority: 100,
                        },
                        1,
                    )
                })
                .unwrap();
            for (amount, at) in [(7, 2), (3, 3)] {
                f.core
                    .runtime_transaction(|c| {
                        let snapshot = c
                            .damage_source_snapshot_for_unit(
                                e,
                                a,
                                DamageSource::Ability,
                                kind,
                                amount,
                                Default::default(),
                                0,
                                at,
                                false,
                            )
                            .unwrap();
                        c.process_commands(
                            vec![BattleCommand::ApplyDamage {
                                target_id: a,
                                source_snapshot: snapshot,
                            }],
                            at,
                        );
                        crate::game::battle::runtime_control::poll()
                    })
                    .unwrap();
                if at == 2 {
                    assert_eq!(f.core.units[&a].stats.current_health, 1000);
                    assert_eq!(pool(&f.core, a), vec![(200, 1, 5001)]);
                } else {
                    assert_eq!(f.core.units[&a].stats.current_health, 998);
                    assert!(pool(&f.core, a).is_empty());
                }
            }
        }
    }
    #[test]
    fn actual_heal_reaction_two_authored_providers_strongest_fallback_and_isolated_pools() {
        let mut f = setup();
        let (a, t, b) = (f.caster, f.ally, f.target);
        f.core.units.get_mut(&b).unwrap().owner = f.core.units[&a].owner;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let mut second = equipment.get_by_id("r11_weapon").unwrap().clone();
        second.id = "second_heal_provider".into();
        second.uuid = Uuid::from_u128(0xe800);
        equipment.items.push(second);
        let mut program = equipment.equipment_supports[0].clone();
        program.equipment = "second_heal_provider".into();
        let haste = program
            .program
            .actual_heal_reaction
            .as_mut()
            .unwrap()
            .target_haste
            .as_mut()
            .unwrap();
        haste.attack_speed_points = 60;
        haste.duration_ms = 1000;
        equipment.equipment_supports.push(program);
        let equipment: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(&ron::ser::to_string(&equipment).unwrap()).unwrap();
        f.core.game_data = crate::game::data::GameDataBuilder::from_validated(
            &crate::game::data::ValidatedGameData::from(std::sync::Arc::clone(&f.core.game_data)),
        )
        .with_equipment_data(std::sync::Arc::new(equipment))
        .build_arc();
        for item in f
            .core
            .items
            .values_mut()
            .filter(|i| i.owner_unit_instance == b)
        {
            item.base_uuid = Uuid::from_u128(0xe800);
        }
        heal(&mut f, a, t, 20, 1);
        heal(&mut f, b, t, 40, 2);
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 625);
        assert_eq!(pool(&f.core, a), vec![(200, 5, 5001)]);
        assert_eq!(pool(&f.core, b), vec![(200, 10, 5002)]);
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(1002))
            .unwrap();
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 770);
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3001))
            .unwrap();
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 1000);
    }
    #[test]
    fn actual_heal_reaction_reactive_bee_charge_other_heal_and_withdrawn_source() {
        use crate::game::ability::sustained::{ReactiveHealProfile, SustainedEffectDef};
        use crate::game::battle::runtime_facts::RuntimeFactSource;
        let mut f = setup();
        let (a, t, e) = (f.caster, f.ally, f.target);
        f.core.units.get_mut(&t).unwrap().stats.current_health = 600;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    a,
                    t,
                    SustainedEffectDef::ReactiveHeal {
                        duration_ms: 10000,
                        profile: ReactiveHealProfile {
                            definition: "finite-healing-bees".into(),
                            charges: 2,
                            max_health_percent: 20,
                        },
                    },
                    0,
                )
            })
            .unwrap();
        for at in [1, 2] {
            if at == 2 {
                f.core.units.get_mut(&a).unwrap().lifecycle =
                    crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
            }
            let source = RuntimeFactSource {
                actor: Some(e),
                ..Default::default()
            };
            let action = f
                .core
                .runtime_facts
                .accept(e, None, source.clone(), at, false)
                .unwrap();
            f.core.runtime_fact_source = RuntimeFactSource {
                action: Some(action),
                ..source
            };
            crate::game::battle::core::relic_causal_tests::joint_committed_damage(
                &mut f.core,
                e,
                t,
                100,
                at,
            );
            if at == 1 {
                assert_eq!(f.core.units[&t].stats.current_health, 700);
                assert_eq!(pool(&f.core, a), vec![(200, 50, 5001)]);
                assert_eq!(w(&f).actual_healed, 200);
            } else {
                assert_eq!(f.core.units[&t].stats.current_health, 800);
                assert!(pool(&f.core, a).is_empty());
                assert_eq!(w(&f).time_ms, 1);
            }
        }
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .all(|r| r.reactive_heal.is_none()));
        assert_eq!(f.core.sustained_basic_interval(t, 1000), 770);
    }
}
