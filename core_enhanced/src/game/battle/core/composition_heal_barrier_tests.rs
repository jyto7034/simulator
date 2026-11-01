mod composition_heal_barrier {
    use super::*;
    use crate::game::battle::{
        core::equipment_support::HealBarrierWitness, runtime_facts::RuntimeFactSource,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/heal_barrier.skill.yaml"
    ));
    fn setup_with(source: &str) -> Fixture {
        fixture_extra_with_defense(
            "Heal(amount:20)",
            false,
            None,
            false,
            source,
            |_| {},
            |_| {},
        )
    }
    fn setup() -> Fixture {
        setup_with(SOURCE)
    }
    fn heal(f: &mut Fixture, actor: UnitInstanceId, target: UnitInstanceId, amount: i32, now: u64) {
        f.core
            .runtime_transaction(|c| {
                c.reconcile_sustained(now)?;
                c.apply_direct_heal_and_record(Some(actor), target, amount, now)
            })
            .unwrap();
    }
    fn pools(f: &Fixture, target: UnitInstanceId) -> Vec<(u32, u32, u64)> {
        f.core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|e| e.target == target)
            .filter_map(|e| e.shield.map(|p| (p.capacity, p.remaining, e.expires_at_ms)))
            .collect()
    }
    fn witness(f: &Fixture) -> &HealBarrierWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_heal_barrier
            .as_ref()
            .unwrap()
    }
    fn action(f: &mut Fixture, actor: UnitInstanceId, now: u64) {
        let source = RuntimeFactSource {
            actor: Some(actor),
            ..Default::default()
        };
        let handle = f
            .core
            .runtime_facts
            .accept(actor, None, source.clone(), now, false)
            .unwrap();
        f.core.runtime_fact_source = RuntimeFactSource {
            action: Some(handle),
            ..source
        };
    }
    #[test]
    fn composition_heal_barrier_roundtrip_origin_validation_and_old_support() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(
            restored.equipment_supports()[0]
                .program
                .direct_heal_barrier
                .unwrap()
                .cooldown_ms,
            8000
        );
        assert!(restored
            .sources()
            .get("equipment_supports[0].program")
            .is_some());
        for (old, new) in [
            ("hp_threshold_percent: 50", "hp_threshold_percent: 101"),
            ("actual_heal_percent: 25", "actual_heal_percent: 0"),
            ("actual_heal_percent: 25", "actual_heal_percent: 10001"),
            ("duration_ms: 5000", "duration_ms: 0"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(old, new)).is_err()
            );
        }
        let old = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/equipment_support.skill.yaml"
        ));
        assert!(crate::authoring::composition::compile_yaml(old)
            .unwrap()
            .equipment_supports()[0]
            .program
            .direct_heal_barrier
            .is_none());
    }
    #[test]
    fn composition_heal_barrier_actual_heal_cap_then_shield_boost_once() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 500;
        heal(&mut f, a, t, 20, 1);
        assert_eq!(f.core.units[&t].stats.current_health, 525);
        assert_eq!(pools(&f, t), vec![(7, 7, 5001)]);
        let w = witness(&f);
        assert_eq!(
            (
                w.hp_before,
                w.actual_healed,
                w.base_capacity,
                w.increased_capacity
            ),
            (500, 25, 6, 7)
        );
        assert_eq!(w.ready_at_ms, 8001);
        assert_eq!(
            w.equipment,
            f.core
                .items
                .values()
                .find(|i| i.owner_unit_instance == a)
                .unwrap()
                .source_snapshot()
        );
        let mut f =
            setup_with(&SOURCE.replace("hp_threshold_percent: 50", "hp_threshold_percent: 100"));
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 990;
        heal(&mut f, a, t, 100, 1);
        assert_eq!(f.core.units[&t].stats.current_health, 1000);
        assert_eq!(
            (witness(&f).actual_healed, witness(&f).base_capacity),
            (10, 2)
        );
        assert_eq!(pools(&f, t), vec![(2, 2, 5001)]);
    }
    #[test]
    fn composition_heal_barrier_threshold_self_hostile_and_inactive_exclusions() {
        for case in 0..7 {
            let mut f = setup();
            let a = f.caster;
            let mut t = f.ally;
            if case == 1 {
                t = a;
            }
            if case == 2 {
                t = f.target;
            }
            f.core.units.get_mut(&t).unwrap().stats.current_health =
                if case == 0 { 501 } else { 400 };
            match case {
                3 => f.core.units.get_mut(&a).unwrap().corroded = true,
                4 => f.core.units.get_mut(&t).unwrap().corroded = true,
                5 => {
                    f.core.units.get_mut(&t).unwrap().role =
                        crate::game::battle::types::BattleUnitRole::IndependentWall
                }
                6 => {
                    f.core.units.get_mut(&a).unwrap().lifecycle =
                        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                }
                _ => (),
            }
            heal(&mut f, a, t, 20, 1);
            assert!(pools(&f, t).is_empty(), "case {case}");
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_heal_barrier
                .is_none());
        }
    }
    #[test]
    fn composition_heal_barrier_per_target_cooldown_exact_boundary_and_expiry() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        let other = f.target;
        f.core.units.get_mut(&other).unwrap().owner = f.core.units[&a].owner;
        for id in [t, other] {
            f.core.units.get_mut(&id).unwrap().stats.current_health = 400;
            heal(&mut f, a, id, 40, 1);
            assert_eq!(pools(&f, id), vec![(15, 15, 5001)]);
        }
        heal(&mut f, a, t, 40, 100);
        assert_eq!(pools(&f, t), vec![(15, 15, 5001)]);
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(5001))
            .unwrap();
        assert!(pools(&f, t).is_empty());
        f.core.units.get_mut(&t).unwrap().stats.current_health = 400;
        heal(&mut f, a, t, 40, 8000);
        assert!(pools(&f, t).is_empty());
        heal(&mut f, a, t, 40, 8001);
        assert_eq!(pools(&f, t), vec![(15, 15, 13001)]);
        assert_eq!(witness(&f).ready_at_ms, 16001);
    }
    #[test]
    fn composition_heal_barrier_same_action_sums_and_preserves_absorbed_damage() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 500;
        action(&mut f, a, 1);
        heal(&mut f, a, t, 20, 1);
        f.core
            .runtime_transaction(|c| c.absorb_sustained_shields(t, 6, 1))
            .unwrap();
        assert_eq!(pools(&f, t), vec![(7, 1, 5001)]);
        heal(&mut f, a, t, 20, 1);
        assert_eq!(pools(&f, t), vec![(15, 9, 5001)]);
        assert_eq!(
            (witness(&f).actual_healed, witness(&f).ready_at_ms),
            (50, 8001)
        );
        heal(&mut f, a, t, 20, 2);
        assert_eq!(pools(&f, t), vec![(15, 9, 5001)]);
    }
    #[test]
    fn composition_heal_barrier_depleted_pool_tiny_same_action_heal_does_not_refill_or_claim_existing(
    ) {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 400;
        action(&mut f, a, 1);
        heal(&mut f, a, t, 20, 1);
        f.core
            .runtime_transaction(|c| c.absorb_sustained_shields(t, 7, 1))
            .unwrap();
        assert!(pools(&f, t).is_empty());
        heal(&mut f, a, t, 1, 1);
        assert!(pools(&f, t).is_empty());
        assert_eq!((witness(&f).actual_healed, witness(&f).remaining), (26, 0));
        assert!(!witness(&f).kept_existing);
        heal(&mut f, a, t, 4, 1);
        assert_eq!(pools(&f, t), vec![(1, 1, 5001)]);
        assert_eq!(
            (witness(&f).actual_healed, witness(&f).increased_capacity),
            (31, 8)
        );
    }
    #[test]
    fn composition_heal_barrier_rounding_zero_does_not_spend_cooldown() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 400;
        heal(&mut f, a, t, 1, 1);
        assert!(pools(&f, t).is_empty());
        heal(&mut f, a, t, 20, 2);
        assert_eq!(pools(&f, t), vec![(7, 7, 5002)]);
    }
    #[test]
    fn composition_heal_barrier_other_provider_keeps_larger_remaining_and_consumes_gate() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        let b = f.target;
        f.core.units.get_mut(&b).unwrap().owner = f.core.units[&a].owner;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 80, 1);
        assert_eq!(pools(&f, t), vec![(31, 31, 5001)]);
        heal(&mut f, b, t, 20, 2);
        assert_eq!(pools(&f, t), vec![(31, 31, 5001)]);
        assert!(witness(&f).kept_existing);
        heal(&mut f, b, t, 160, 3);
        assert_eq!(pools(&f, t), vec![(31, 31, 5001)]);
    }
    #[test]
    fn composition_heal_barrier_source_exit_keeps_pool_and_releases_gate() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 20, 1);
        let copy = f
            .core
            .items
            .iter()
            .find(|(_, i)| i.owner_unit_instance == a)
            .map(|(id, _)| *id)
            .unwrap();
        let item = f.core.items.remove(&copy).unwrap();
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(2))
            .unwrap();
        assert_eq!(pools(&f, t), vec![(7, 7, 5001)]);
        f.core.items.insert(copy, item);
        heal(&mut f, a, t, 40, 3);
        assert_eq!(pools(&f, t), vec![(15, 15, 5003)]);
        f.core
            .runtime_transaction(|c| c.clear_sustained(4))
            .unwrap();
        assert!(pools(&f, t).is_empty());
    }
    #[test]
    fn composition_heal_barrier_cancellation_restores_hp_pool_gate_and_observation() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 500;
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("heal_barrier_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core
                        .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 20, 1))
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
        assert_eq!(pools(&f, t), vec![(7, 7, 5001)]);
    }
    #[test]
    fn composition_heal_barrier_retained_failure_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 500;
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 20, 1))
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "heal_barrier_observation")
            .map(|(_, _, bytes)| *bytes)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let error = f
            .core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), t, 20, 1))
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits = Default::default();
        heal(&mut f, a, t, 20, 1);
        assert_eq!(pools(&f, t), vec![(7, 7, 5001)]);
    }
    #[test]
    fn composition_heal_barrier_stronger_provider_replaces_and_target_exit_clears() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        let b = f.target;
        f.core.units.get_mut(&b).unwrap().owner = f.core.units[&a].owner;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 100;
        heal(&mut f, a, t, 20, 1);
        heal(&mut f, b, t, 80, 2);
        assert_eq!(pools(&f, t), vec![(31, 31, 5002)]);
        assert!(!witness(&f).kept_existing);
        assert_eq!(witness(&f).actor, b);
        f.core.units.get_mut(&b).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(3))
            .unwrap();
        assert_eq!(pools(&f, t), vec![(31, 31, 5002)]);
        f.core.units.get_mut(&t).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(4))
            .unwrap();
        assert!(pools(&f, t).is_empty());
    }
    #[test]
    fn composition_heal_barrier_clone_and_observer_preserve_authority() {
        let mut f = setup();
        let a = f.caster;
        let t = f.ally;
        f.core.units.get_mut(&t).unwrap().stats.current_health = 500;
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| {
                c.reconcile_sustained(1)?;
                c.apply_direct_heal_and_record(Some(a), t, 20, 1)
            })
            .unwrap();
        heal(&mut f, a, t, 20, 1);
        assert_eq!(
            plain.transaction_debug_snapshot(),
            f.core.transaction_debug_snapshot()
        );
        let mut branch = f.core.clone();
        branch
            .runtime_transaction(|c| c.absorb_sustained_shields(t, 7, 2))
            .unwrap();
        assert!(branch
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .all(|e| e.target != t || e.shield.is_none()));
        assert_eq!(pools(&f, t), vec![(7, 7, 5001)]);
    }
}
