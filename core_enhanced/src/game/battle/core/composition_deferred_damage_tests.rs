mod composition_deferred_damage {
    use super::*;
    use crate::game::{
        ability::sustained::{SurvivalPolicy, SustainedEffectDef},
        battle::{core::equipment_defense::DeferredOperation, event_log::HpChangeReason},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/deferred_damage.skill.yaml"
    ));
    fn setup_with(source: &str) -> Fixture {
        fixture_extra_with_defense(
            "Damage(amount:100, damage_type:True)",
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
    fn balance(f: &Fixture, owner: UnitInstanceId) -> u32 {
        f.core
            .equipment_defense
            .deferred
            .values()
            .filter(|r| r.owner == owner)
            .map(|r| r.remaining)
            .sum()
    }
    fn hit(
        f: &mut Fixture,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        amount: u32,
        kind: DamageSource,
        now: u64,
    ) {
        f.core
            .runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        actor,
                        target,
                        kind,
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
    fn damage(f: &mut Fixture, amount: u32, now: u64) {
        let (a, t) = (f.caster, f.target);
        hit(f, a, t, amount, DamageSource::Ability, now);
    }
    fn advance(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn effect(f: &mut Fixture, definition: SustainedEffectDef, now: u64) {
        let target = f.target;
        f.core
            .runtime_transaction(|c| c.begin_sustained(target, target, definition, now))
            .unwrap();
    }
    fn witness(
        f: &Fixture,
    ) -> &crate::game::battle::core::equipment_defense::DeferredDamageWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_deferred_damage
            .as_ref()
            .unwrap()
    }
    #[test]
    fn composition_deferred_damage_roundtrip_and_reject_invalid_schedule() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            restored.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::DeferredDamage {
                defer_percent: 30,
                duration_ms: 5000,
                tick_ms: 1000,
                kill_clear_percent: 50
            }
        ));
        assert!(restored
            .sources()
            .get("equipment_defenses[0].program")
            .is_some());
        for (key, value) in [
            ("defer_percent", 0),
            ("defer_percent", 101),
            ("tick_ms", 0),
            ("tick_ms", 3000),
            ("duration_ms", 0),
            ("duration_ms", 1001000),
            ("kill_clear_percent", 101),
        ] {
            let mut doc = parse_yaml(SOURCE).unwrap();
            doc["values"][0]["value"]["DefenseProgram"]["DeferredDamage"][key] = json!(value);
            assert!(compile_json(&doc.to_string()).is_err(), "{key}={value}");
        }
    }
    #[test]
    fn composition_deferred_damage_100_is_70_then_five_6_and_no_redeferral() {
        let mut f = setup();
        damage(&mut f, 100, 2);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (930, 30)
        );
        advance(&mut f, 1001);
        assert_eq!(f.core.units[&f.target].stats.current_health, 930);
        for (now, hp, left) in [
            (1002, 924, 24),
            (2002, 918, 18),
            (3002, 912, 12),
            (4002, 906, 6),
            (5002, 900, 0),
        ] {
            advance(&mut f, now);
            assert_eq!(
                (
                    f.core.units[&f.target].stats.current_health,
                    balance(&f, f.target)
                ),
                (hp, left)
            );
        }
        assert!(matches!(witness(&f).operation, DeferredOperation::Repay));
        assert_eq!(witness(&f).actual_hp_loss, 6);
        assert_eq!(witness(&f).incoming.actor, Some(f.caster));
        assert!(f.core.equipment_defense.deferred.is_empty());
    }
    #[test]
    fn composition_deferred_damage_integer_remainders_and_independent_expiry() {
        let mut f = setup();
        damage(&mut f, 10, 2);
        damage(&mut f, 20, 502);
        assert_eq!(balance(&f, f.target), 9);
        advance(&mut f, 5002);
        assert_eq!(balance(&f, f.target), 2);
        advance(&mut f, 5502);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (970, 0)
        );
    }
    #[test]
    fn composition_deferred_damage_overkill_split_before_survival() {
        let mut f = setup();
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .stats
            .current_health = 100;
        effect(
            &mut f,
            SustainedEffectDef::Survival {
                duration_ms: 10000,
                policy: SurvivalPolicy::PreventDeathOnce,
            },
            1,
        );
        damage(&mut f, 120, 2);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (16, 36)
        );
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .any(|r| r.survival == Some(SurvivalPolicy::PreventDeathOnce)));
        advance(&mut f, 3002);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1);
        assert!(!f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .any(|r| r.survival == Some(SurvivalPolicy::PreventDeathOnce)));
        advance(&mut f, 4002);
        assert!(f.core.units[&f.target].lifecycle.is_dead());
        assert_eq!(balance(&f, f.target), 0);
    }
    #[test]
    fn composition_deferred_damage_shield_before_store_repayment_bypasses_new_shield_and_amp() {
        let mut f = setup();
        effect(
            &mut f,
            SustainedEffectDef::Shield {
                duration_ms: 10000,
                amount: 40,
                priority: 0,
                shared_radius_units: None,
            },
            1,
        );
        damage(&mut f, 100, 2);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (958, 18)
        );
        effect(
            &mut f,
            SustainedEffectDef::Shield {
                duration_ms: 10000,
                amount: 100,
                priority: 0,
                shared_radius_units: None,
            },
            3,
        );
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .incoming_damage_modifiers
            .damage_amp_percent = 100;
        advance(&mut f, 5002);
        assert_eq!(f.core.units[&f.target].stats.current_health, 940);
        assert_eq!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .into_iter()
                .filter_map(|r| r.shield.map(|s| s.remaining))
                .sum::<u32>(),
            100
        );
    }
    #[test]
    fn composition_deferred_damage_kill_clears_fifo_without_heal_or_dead_credit() {
        let mut f = setup();
        damage(&mut f, 100, 2);
        damage(&mut f, 100, 3);
        let (killer, dead) = (f.target, f.caster);
        hit(&mut f, killer, dead, 2000, DamageSource::Ability, 4);
        assert!(f.core.units[&dead].lifecycle.is_dead());
        assert_eq!(
            (
                f.core.units[&killer].stats.current_health,
                balance(&f, killer)
            ),
            (860, 30)
        );
        assert_eq!(
            f.core
                .equipment_defense
                .deferred
                .values()
                .filter(|r| r.owner == killer)
                .count(),
            1
        );
        assert!(matches!(
            witness(&f).operation,
            DeferredOperation::ClearOnKill
        ));
        assert_eq!((witness(&f).amount, witness(&f).actual_hp_loss), (30, 0));
        advance(&mut f, 1002);
        assert_eq!(f.core.units[&killer].stats.current_health, 860);
        advance(&mut f, 1003);
        assert_eq!(
            f.core.units[&killer].stats.current_health, 854,
            "original attacker death does not cancel debt"
        );
    }
    #[test]
    fn composition_deferred_damage_hostile_ticks_and_reactions_include_self_ally_cost_exclude() {
        let mut f = setup();
        let (actor, target, ally) = (f.caster, f.target, f.ally);
        hit(&mut f, actor, target, 100, DamageSource::BuffTick, 2);
        hit(&mut f, actor, target, 100, DamageSource::Environment, 3);
        assert_eq!(balance(&f, target), 60);
        hit(&mut f, target, target, 100, DamageSource::Ability, 4);
        hit(&mut f, actor, ally, 100, DamageSource::Ability, 5);
        f.core
            .runtime_transaction(|c| {
                c.apply_hp_delta_and_record(Some(target), target, -100, 6, HpChangeReason::Command);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(balance(&f, target), 60);
        assert_eq!(balance(&f, ally), 0);
    }
    #[test]
    fn composition_deferred_damage_corrosion_keeps_debt_but_stops_new_credit() {
        let mut f = setup();
        damage(&mut f, 100, 2);
        f.core.units.get_mut(&f.target).unwrap().corroded = true;
        damage(&mut f, 100, 3);
        assert_eq!(balance(&f, f.target), 30);
        advance(&mut f, 1002);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (824, 24)
        );
    }
    #[test]
    fn composition_deferred_damage_unequip_terminal_and_stale_events() {
        let mut f = setup();
        damage(&mut f, 100, 2);
        let copy = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == f.target)
            .unwrap()
            .instance_id;
        f.core.items.remove(&copy);
        advance(&mut f, 1002);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (930, 0)
        );
        let mut f = setup();
        damage(&mut f, 100, 2);
        f.core
            .runtime_transaction(|c| c.clear_sustained(3))
            .unwrap();
        assert!(f.core.equipment_defense.deferred.is_empty());
        f.core
            .runtime_transaction(|c| c.deferred_damage_tick(1, 1002))
            .unwrap();
        assert_eq!(f.core.units[&f.target].stats.current_health, 930);
    }
    #[test]
    fn composition_deferred_damage_public_cancellation_rolls_back_tick_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        damage(&mut f, 100, 2);
        advance(&mut f, 1001);
        let before = f.core.transaction_debug_snapshot();
        let state = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("deferred_repaid", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1002),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        advance(&mut f, 1002);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (924, 24)
        );
    }
    #[test]
    fn composition_deferred_damage_actual_withdraw_cancels_debt_and_restores_on_cancel() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let (enemy, wearer) = (f.target, f.caster);
        hit(&mut f, enemy, wearer, 100, DamageSource::Ability, 2);
        let before = f.core.transaction_debug_snapshot();
        let command = BattleLiveCommand::WithdrawUnit {
            unit_id: wearer,
            time_ms: 3,
        };
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_defense_remove_owner", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.apply_live_command(command.clone()),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        f.core.apply_live_command(command).unwrap();
        assert_eq!(balance(&f, wearer), 0);
        assert!(!f.core.units[&wearer].is_active());
        advance(&mut f, 5002);
        assert_eq!(balance(&f, wearer), 0);
    }
    #[test]
    fn composition_deferred_damage_invulnerable_tick_consumes_schedule_without_hp_loss() {
        let mut f = setup();
        damage(&mut f, 100, 2);
        effect(
            &mut f,
            SustainedEffectDef::Parry {
                duration_ms: 5000,
                window_ms: 1000,
                invulnerable_ms: 1500,
                miss_recovery_ms: 100,
                reduction_percent: 0,
                counter_skill: "r11_aura".into(),
                end_on_resolution: false,
                fragment_guard: None,
                guard_charge: None,
            },
            3,
        );
        damage(&mut f, 100, 4); // Real direct hit arms the parry window, without adding debt.
        assert_eq!(balance(&f, f.target), 30);
        advance(&mut f, 1002);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (930, 24)
        );
        assert_eq!(witness(&f).actual_hp_loss, 0);
        advance(&mut f, 2002);
        assert_eq!(f.core.units[&f.target].stats.current_health, 924);
    }
    #[test]
    fn composition_deferred_damage_public_retained_failure_restores_creation_and_retry() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        cast(&mut f, 0);
        advance(&mut f, 0);
        let mut probe = f.core.clone();
        let mut probe_state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut probe_state, 1)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "deferred_observation")
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
                    site: "deferred_observation",
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
        advance(&mut f, 1);
        assert_eq!(
            (
                f.core.units[&f.target].stats.current_health,
                balance(&f, f.target)
            ),
            (930, 30)
        );
    }
}
