mod composition_overheal_reserve {
    use super::*;
    use crate::game::battle::{
        core::equipment_defense::OverhealReserveWitness, event_log::HpChangeReason,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/overheal_reserve.skill.yaml"
    ));
    fn setup_with(source: &str, op: &str) -> Fixture {
        fixture_extra_with_defense(op, false, None, false, source, |_| {}, |_| {})
    }
    fn setup() -> Fixture {
        setup_with(SOURCE, "Heal(amount:20)")
    }
    fn witness(f: &Fixture) -> &OverhealReserveWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_overheal_reserve
            .as_ref()
            .unwrap()
    }
    fn balance(f: &Fixture, owner: UnitInstanceId) -> u32 {
        f.core
            .equipment_defense
            .reserves
            .values()
            .filter(|r| r.owner == owner)
            .map(|r| r.amount)
            .sum()
    }
    fn heal(f: &mut Fixture, actor: UnitInstanceId, target: UnitInstanceId, amount: i32) {
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(actor), target, amount, 1))
            .unwrap();
    }
    fn damage(f: &mut Fixture, amount: u32, now: u64) {
        let target = f.target;
        let source = f.caster;
        f.core
            .runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        source,
                        target,
                        DamageSource::Ability,
                        DamageType::True,
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
                        source_snapshot: snapshot,
                    }],
                    now,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
    }
    #[test]
    fn composition_overheal_reserve_roundtrip_bounds_and_origin() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            restored.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::OverhealReserve {
                max_hp_cap_percent: 50,
                trigger_hp_percent: 50
            }
        ));
        assert!(restored
            .sources()
            .get("equipment_defenses[0].program")
            .is_some());
        for key in ["max_hp_cap_percent", "trigger_hp_percent"] {
            for value in [0, 101] {
                let mut doc = parse_yaml(SOURCE).unwrap();
                doc["values"][0]["value"]["DefenseProgram"]["OverhealReserve"][key] = json!(value);
                let errors = compile_json(&doc.to_string()).unwrap_err();
                assert!(format!("{errors:?}").contains(key));
            }
        }
    }
    #[test]
    fn composition_overheal_reserve_received_only_actual_overflow_and_cap() {
        let mut f = setup();
        let actor = f.caster;
        let target = f.target;
        f.core.units.get_mut(&target).unwrap().stats.current_health = 950;
        heal(&mut f, actor, target, 120);
        assert_eq!(f.core.units[&target].stats.current_health, 1000);
        assert_eq!(balance(&f, target), 70);
        assert_eq!(balance(&f, actor), 0);
        assert_eq!(witness(&f).incoming_overheal, 70);
        heal(&mut f, actor, target, 900);
        assert_eq!(balance(&f, target), 500);
        heal(&mut f, actor, target, 0);
        assert_eq!(balance(&f, target), 500);
    }
    #[test]
    fn composition_overheal_reserve_damage_threshold_release_and_leftover() {
        let mut f = setup();
        let actor = f.caster;
        let target = f.target;
        heal(&mut f, actor, target, 400);
        damage(&mut f, 499, 2);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (501, 400)
        );
        damage(&mut f, 1, 3);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (900, 0)
        );
        assert_eq!(
            (
                witness(&f).stored_before,
                witness(&f).stored_after,
                witness(&f).actual_healed
            ),
            (400, 0, 400)
        );
        let source = SOURCE
            .replace("max_hp_cap_percent: 50", "max_hp_cap_percent: 100")
            .replace("trigger_hp_percent: 50", "trigger_hp_percent: 100");
        let mut f = setup_with(&source, "Heal(amount:20)");
        let actor = f.caster;
        let target = f.target;
        heal(&mut f, actor, target, 800);
        damage(&mut f, 100, 2);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (1000, 700)
        );
        damage(&mut f, 100, 2);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (1000, 600)
        );
    }
    #[test]
    fn composition_overheal_reserve_reaction_heal_stores_but_own_release_never_does() {
        let mut f = setup();
        let target = f.target;
        f.core.runtime_fact_source.reaction = true;
        heal(&mut f, target, target, 500);
        assert_eq!(balance(&f, target), 500);
        assert!(witness(&f).incoming.reaction);
        damage(&mut f, 600, 2);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (900, 0)
        );
    }
    #[test]
    fn composition_overheal_reserve_lethal_and_hp_cost_do_not_release() {
        let mut f = setup();
        let actor = f.caster;
        let target = f.target;
        heal(&mut f, actor, target, 500);
        f.core
            .runtime_transaction(|c| {
                c.apply_hp_delta_and_record(
                    Some(target),
                    target,
                    -600,
                    2,
                    HpChangeReason::HealthCost,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (400, 500)
        );
        damage(&mut f, 400, 3);
        assert_eq!(f.core.units[&target].stats.current_health, 0);
        assert_eq!(balance(&f, target), 0);
    }
    #[test]
    fn composition_overheal_reserve_cap_change_and_lifetime_cleanup() {
        for boundary in 0..5 {
            let mut f = setup();
            let actor = f.caster;
            let target = f.target;
            heal(&mut f, actor, target, 500);
            match boundary {
                0 => {
                    f.core.units.get_mut(&target).unwrap().stats.max_health = 200;
                    f.core
                        .runtime_transaction(|c| {
                            c.record_runtime_stat_hp_change_and_finalize(
                                None, target, 1000, 1000, 2, None,
                            );
                            crate::game::battle::runtime_control::poll()
                        })
                        .unwrap();
                    assert_eq!(balance(&f, target), 100);
                    f.core.units.get_mut(&target).unwrap().stats.max_health = 2000;
                }
                1 => f.core.units.get_mut(&target).unwrap().corroded = true,
                2 => {
                    f.core.units.get_mut(&target).unwrap().lifecycle =
                        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                }
                3 => f
                    .core
                    .items
                    .retain(|_, item| item.owner_unit_instance != target),
                _ => {
                    f.core
                        .runtime_transaction(|c| c.clear_sustained(2))
                        .unwrap();
                }
            }
            f.core
                .runtime_transaction(|c| c.reconcile_equipment_defenses())
                .unwrap();
            assert_eq!(balance(&f, target), if boundary == 0 { 100 } else { 0 });
        }
    }
    #[test]
    fn composition_overheal_reserve_observer_independence_and_clone_balance() {
        let mut observed = setup();
        let actor = observed.caster;
        let target = observed.target;
        heal(&mut observed, actor, target, 500);
        let mut plain = setup();
        plain.core = observed.core.clone();
        plain.core.authoring_observation = None;
        damage(&mut observed, 600, 2);
        damage(&mut plain, 600, 2);
        assert_eq!(
            observed.core.transaction_debug_snapshot(),
            plain.core.transaction_debug_snapshot()
        );
        assert_eq!(balance(&plain, target), 0);
        assert_eq!(plain.core.units[&target].stats.current_health, 900);
    }
    #[test]
    fn composition_overheal_reserve_public_cancel_restores_heal_store_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        cast(&mut f, 0);
        f.core.step_battle_execution_until(&mut f.state, 0).unwrap();
        let before = f.core.transaction_debug_snapshot();
        let state = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("overheal_reserve_stored", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(balance(&f, f.target), 20);
    }
    #[test]
    fn composition_overheal_reserve_public_retained_failure_restores_and_retry() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        cast(&mut f, 0);
        f.core.step_battle_execution_until(&mut f.state, 0).unwrap();
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 1)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "overheal_reserve_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let state = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 1)
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "overheal_reserve_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits = Default::default();
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(balance(&f, f.target), 20);
    }
    #[test]
    fn composition_overheal_reserve_amplified_received_heal_and_real_stat_cap() {
        use crate::game::stats::{StatId, StatModifier, StatModifierKind};
        let mut doc = parse_yaml(SOURCE).unwrap();
        let support = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/equipment_support.skill.yaml"
        )))
        .unwrap();
        doc["values"]
            .as_array_mut()
            .unwrap()
            .extend(support["values"].as_array().unwrap().iter().cloned());
        doc["equipment_supports"] = support["equipment_supports"].clone();
        let mut f = setup_with(&doc.to_string(), "Heal(amount:20)");
        let target = f.ally;
        let actor = f.caster;
        // Both player copies carry the authored test binding; outgoing bonus is allied only.
        f.core.units.get_mut(&target).unwrap().stats.current_health = 990;
        heal(&mut f, actor, target, 100);
        assert_eq!(balance(&f, target), 115);
        f.core
            .runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::ApplyModifier {
                        target_id: target,
                        modifier: StatModifier {
                            stat: StatId::MaxHealth,
                            kind: StatModifierKind::Flat,
                            value: -900,
                        },
                    }],
                    2,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(f.core.units[&target].stats.max_health, 100);
        assert_eq!(balance(&f, target), 50);
        f.core
            .runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::ApplyModifier {
                        target_id: target,
                        modifier: StatModifier {
                            stat: StatId::MaxHealth,
                            kind: StatModifierKind::Flat,
                            value: 1900,
                        },
                    }],
                    3,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(
            balance(&f, target),
            50,
            "max HP gain grants no stored healing"
        );
    }
    #[test]
    fn composition_overheal_reserve_shield_only_hit_never_releases() {
        use crate::game::ability::sustained::SustainedEffectDef;
        let mut f = setup();
        let actor = f.caster;
        let target = f.target;
        heal(&mut f, actor, target, 500);
        f.core.units.get_mut(&target).unwrap().stats.current_health = 400;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    target,
                    target,
                    SustainedEffectDef::Shield {
                        duration_ms: 100,
                        amount: 100,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    1,
                )
            })
            .unwrap();
        damage(&mut f, 100, 2);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (400, 500)
        );
        damage(&mut f, 1, 3);
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (899, 0)
        );
    }
    #[test]
    fn composition_overheal_reserve_public_release_cancel_restores_damage_balance_and_source() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup_with(SOURCE, "Damage(amount:600,damage_type:True)");
        let actor = f.caster;
        let target = f.target;
        heal(&mut f, actor, target, 500);
        cast(&mut f, 0);
        f.core.step_battle_execution_until(&mut f.state, 0).unwrap();
        let before = f.core.transaction_debug_snapshot();
        let state = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("overheal_reserve_released", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(
            (
                f.core.units[&target].stats.current_health,
                balance(&f, target)
            ),
            (900, 0)
        );
        let fact = f.core.runtime_fact_summary().latest_heal.as_ref().unwrap();
        assert_eq!(fact.source.actor, Some(target));
        assert_eq!(fact.source.equipment, Some(witness(&f).equipment));
        assert!(fact
            .source
            .authored
            .as_ref()
            .unwrap()
            .field_path
            .ends_with("OverhealReserve"));
        assert!(
            fact.source.cast.is_none()
                && fact.source.fragment.is_none()
                && fact.source.action.is_none()
        );
        assert!(
            witness(&f).incoming.cast.is_some(),
            "incoming hostile cast stays in evidence"
        );
    }
}
