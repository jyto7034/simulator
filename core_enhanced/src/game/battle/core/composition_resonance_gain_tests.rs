mod composition_resonance_gain {
    use super::*;
    use crate::game::battle::core::equipment_resonance_gain::{
        ResonanceGainChannel, ResonanceGainWitness,
    };
    use crate::game::battle::{runtime_control as native, types::BattleUnitSourceIdentity};
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/resonance_gain.skill.yaml"
    ));
    fn setup(percent: u32, channels: &str) -> Fixture {
        let text = SOURCE
            .replace("percent: 20", &format!("percent: {percent}"))
            .replace("AutomaticAndAuthored", channels);
        let mut f =
            fixture_extra_with_defense("Heal(amount:1)", false, None, false, &text, |_| {}, |_| {});
        for actor in [f.caster, f.ally] {
            let u = f.core.units.get_mut(&actor).unwrap();
            assert!(matches!(
                u.source_identity,
                BattleUnitSourceIdentity::Employee { .. }
            ));
            u.resonance_current = 0;
            u.resonance_max = 100;
        }
        f
    }
    fn witness(core: &BattleCore) -> &ResonanceGainWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_resonance_gain
            .as_ref()
            .unwrap()
    }
    fn gain(core: &mut BattleCore, actor: UnitInstanceId, amount: i32, now: u64) {
        core.runtime_transaction(|c| {
            c.modify_resonance(actor, amount, now, false);
            native::poll()
        })
        .unwrap();
    }
    fn automatic(core: &mut BattleCore, actor: UnitInstanceId, amount: u32, now: u64) {
        core.runtime_transaction(|c| {
            c.add_resonance_from(
                actor,
                amount,
                now,
                false,
                Some(ResonanceGainChannel::Automatic),
            );
            native::poll()
        })
        .unwrap();
    }
    fn snapshot(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(&core.authoring_observation).unwrap(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_resonance_gain_schema_shared_roundtrip_bounds_and_any_equipment_slot() {
        let d = parse_yaml(SOURCE).unwrap();
        let artifact = compile_json(&d.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(
            artifact.equipment_supports()[0]
                .program
                .resonance_gain
                .unwrap()
                .percent,
            20
        );
        assert!(artifact
            .sources()
            .get("equipment_supports[0].program")
            .unwrap()
            .definition
            .ends_with("SupportProgram"));
        for value in [0, 10000] {
            crate::authoring::composition::compile_yaml(
                &SOURCE.replace("percent: 20", &format!("percent: {value}")),
            )
            .unwrap();
        }
        let error = crate::authoring::composition::compile_yaml(
            &SOURCE.replace("percent: 20", "percent: 10001"),
        )
        .unwrap_err();
        assert!(error.source.definition.ends_with("resonance_gain.percent"));
        assert!(crate::authoring::composition::compile_yaml(
            &SOURCE.replace("AutomaticAndAuthored", "BasicHit")
        )
        .is_err());
        let f = setup(20, "AutomaticAndAuthored");
        let mut data = (*f.core.game_data.equipment_data).clone();
        let item = data
            .items
            .iter_mut()
            .find(|i| i.id == "r11_weapon")
            .unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        item.weapon_profile = None;
        crate::game::ability::equipment_support::EquipmentSupportCatalog::compile(&data, &Default::default()).unwrap();
    }
    #[test]
    fn composition_resonance_gain_real_automatic_pulses_fraction_and_new_authored_rate() {
        for (percent, seconds, expected) in [(20, 5, 6), (50, 2, 3), (100, 3, 6), (0, 5, 5)] {
            let mut f = setup(percent, "AutomaticAndAuthored");
            f.core
                .step_battle_execution_until(&mut f.state, seconds * 1000)
                .unwrap();
            assert_eq!(
                f.core.units[&f.caster].resonance_current, expected,
                "rate {percent}"
            );
            if percent > 0 {
                let w = witness(&f.core);
                assert_eq!(w.channel, ResonanceGainChannel::Automatic);
                assert_eq!(w.percent, u64::from(percent));
                assert_eq!(w.remainder_after, 0);
                assert!(w.contributors[0].field_path.ends_with("resonance_gain"));
            }
        }
    }
    #[test]
    fn composition_resonance_gain_authored_packet_negative_and_neutral_boundary() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        gain(&mut f.core, a, 9, 1);
        assert_eq!(
            (
                f.core.units[&a].resonance_current,
                witness(&f.core).remainder_after
            ),
            (10, 80)
        );
        gain(&mut f.core, a, -5, 2);
        assert_eq!(f.core.units[&a].resonance_current, 5);
        f.core.add_resonance(a, 1, 3, false);
        assert_eq!(f.core.units[&a].resonance_current, 6);
        gain(&mut f.core, a, 1, 4);
        let w = witness(&f.core);
        assert_eq!(
            (
                w.base_amount,
                w.increased_amount,
                w.actual_gain,
                w.before,
                w.after,
                w.remainder_before,
                w.remainder_after
            ),
            (1, 2, 2, 6, 8, 80, 0)
        );
    }
    #[test]
    fn composition_resonance_gain_channel_filter_and_alternating_sources_share_fraction() {
        for (channels, auto, authored) in [("Automatic", 6, 5), ("Authored", 5, 6)] {
            let mut f = setup(20, channels);
            let a = f.caster;
            automatic(&mut f.core, a, 5, 1);
            assert_eq!(f.core.units[&a].resonance_current, auto);
            gain(&mut f.core, a, 5, 2);
            assert_eq!(f.core.units[&a].resonance_current, auto + authored);
        }
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        for t in 1..=5 {
            if t % 2 == 0 {
                automatic(&mut f.core, a, 1, t)
            } else {
                gain(&mut f.core, a, 1, t)
            }
        }
        assert_eq!(f.core.units[&a].resonance_current, 6);
        assert_eq!(witness(&f.core).remainder_after, 0);
        let mut f = setup(20, "Automatic");
        let a = f.caster;
        automatic(&mut f.core, a, 4, 1);
        gain(&mut f.core, a, 1, 2);
        automatic(&mut f.core, a, 1, 3);
        assert_eq!(
            f.core.units[&a].resonance_current, 7,
            "ineligible channel preserves earned80/100 fraction"
        );
    }
    #[test]
    fn composition_resonance_gain_full_bar_discards_fraction_and_no_overflow_bank() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        f.core.units.get_mut(&a).unwrap().resonance_current = 98;
        gain(&mut f.core, a, 1, 1);
        assert_eq!(witness(&f.core).remainder_after, 20);
        gain(&mut f.core, a, 10, 2);
        assert_eq!(
            (
                witness(&f.core).actual_gain,
                witness(&f.core).after,
                witness(&f.core).remainder_after
            ),
            (1, 100, 0)
        );
        assert!(f.core.units[&a].equipment_resonance_gain.is_none());
        gain(&mut f.core, a, 100, 3);
        gain(&mut f.core, a, -10, 4);
        gain(&mut f.core, a, 4, 5);
        assert_eq!(f.core.units[&a].resonance_current, 94);
        f.core.add_resonance(a, 6, 6, false);
        assert!(f.core.units[&a].equipment_resonance_gain.is_none());
        gain(&mut f.core, a, -10, 7);
        gain(&mut f.core, a, 1, 8);
        assert_eq!(
            (
                f.core.units[&a].resonance_current,
                witness(&f.core).remainder_before
            ),
            (91, 0)
        );
        let mut wide = setup(10000, "AutomaticAndAuthored");
        let a = wide.caster;
        wide.core.units.get_mut(&a).unwrap().resonance_max = u32::MAX;
        automatic(&mut wide.core, a, u32::MAX, 1);
        assert_eq!(
            (
                witness(&wide.core).increased_amount,
                witness(&wide.core).actual_gain,
                witness(&wide.core).remainder_after
            ),
            (u32::MAX, u32::MAX, 0)
        );
    }
    #[test]
    fn composition_resonance_gain_stun_blocks_pulses_and_extra_gain_without_backlog() {
        use crate::game::battle::{buffs::BuffId, enums::BattleEvent};
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        gain(&mut f.core, a, 4, 1);
        f.core
            .process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: f.target,
                    target_instance_id: a,
                    buff_id: BuffId::from_name("r11_stun"),
                    duration_ms: 2100,
                    cause: Default::default(),
                },
                1,
            )
            .unwrap();
        gain(&mut f.core, a, 5, 2);
        assert_eq!(f.core.units[&a].resonance_current, 4);
        f.core
            .step_battle_execution_until(&mut f.state, 2999)
            .unwrap();
        assert_eq!(f.core.units[&a].resonance_current, 4);
        f.core
            .step_battle_execution_until(&mut f.state, 3000)
            .unwrap();
        assert_eq!(
            f.core.units[&a].resonance_current, 6,
            "one resumed pulse spends previous80/100, no blocked-time accrual"
        );
    }
    #[test]
    fn composition_resonance_gain_skill_use_blocks_and_release_resumes() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        f.core.units.get_mut(&a).unwrap().resonance_current = 100;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 0,
            })
            .unwrap();
        assert!(f.core.resonance_use_has_obligations(a));
        gain(&mut f.core, a, 10, 0);
        assert_eq!(f.core.units[&a].resonance_current, 0);
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert!(!f.core.resonance_use_has_obligations(a));
        gain(&mut f.core, a, 5, 2);
        assert_eq!(f.core.units[&a].resonance_current, 6);
    }
    #[test]
    fn composition_resonance_gain_copy_isolation_additive_providers_and_changed_identity() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        let b = f.ally;
        gain(&mut f.core, a, 4, 1);
        gain(&mut f.core, b, 1, 2);
        gain(&mut f.core, a, 1, 3);
        assert_eq!(
            (
                f.core.units[&a].resonance_current,
                f.core.units[&b].resonance_current
            ),
            (6, 1)
        );
        let mut item = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == a)
            .unwrap()
            .clone();
        item.instance_id = Uuid::from_u128(0x780001);
        item.persistent_copy_uuid = Some(Uuid::from_u128(0x780002));
        f.core.items.insert(item.instance_id, item.clone());
        gain(&mut f.core, a, 10, 4);
        assert_eq!(f.core.units[&a].resonance_current, 20);
        assert_eq!(witness(&f.core).percent, 40);
        assert_eq!(witness(&f.core).contributors.len(), 2);
        gain(&mut f.core, a, 1, 5);
        assert_eq!(witness(&f.core).remainder_after, 40);
        f.core.items.remove(&item.instance_id);
        gain(&mut f.core, a, 3, 6);
        assert_eq!(
            (
                witness(&f.core).remainder_before,
                witness(&f.core).remainder_after
            ),
            (0, 60)
        );
    }
    #[test]
    fn composition_resonance_gain_ineligible_identity_withdraw_and_terminal_cleanup() {
        for case in 0..3 {
            let mut f = setup(20, "AutomaticAndAuthored");
            let a = f.caster;
            match case {
                0 => f.core.units.get_mut(&a).unwrap().corroded = true,
                1 => f.core.units.get_mut(&a).unwrap().owner = Side::Opponent,
                _ => f.core.units.get_mut(&a).unwrap().stats.current_health = 0,
            }
            gain(&mut f.core, a, 5, 1);
            assert_eq!(f.core.units[&a].resonance_current, 5);
            assert!(f.core.units[&a].equipment_resonance_gain.is_none());
        }
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        gain(&mut f.core, a, 4, 1);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 2,
            })
            .unwrap();
        assert!(f.core.units[&a].equipment_resonance_gain.is_none());
        let before = f.core.units[&a].resonance_current;
        gain(&mut f.core, a, 5, 3);
        assert_eq!(f.core.units[&a].resonance_current, before);
        let b = f.ally;
        gain(&mut f.core, b, 4, 4);
        f.core
            .runtime_transaction(|c| c.clear_sustained(5))
            .unwrap();
        assert!(f
            .core
            .units
            .values()
            .all(|u| u.equipment_resonance_gain.is_none()));
        // Equipment shutdown must not change the old neutral gain boundary.
        gain(&mut f.core, b, 5, 6);
        assert_eq!(f.core.units[&b].resonance_current, 9);
        assert!(f.core.units[&b].equipment_resonance_gain.is_none());
        f.core.reset_runtime_state_for_battle();
        assert!(f.core.units.is_empty());
    }
    #[test]
    fn composition_resonance_gain_public_pulse_cancellation_restores_fraction_queue_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(20, "AutomaticAndAuthored");
        f.core
            .step_battle_execution_until(&mut f.state, 4999)
            .unwrap();
        let before = (snapshot(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("resonance_gain_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 5000),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (snapshot(&f.core), format!("{:?}", f.state)));
        f.core
            .step_battle_execution_until(&mut f.state, 5000)
            .unwrap();
        assert_eq!(f.core.units[&f.caster].resonance_current, 6);
    }
    #[test]
    fn composition_resonance_gain_retained_refusal_preserves_state_and_finite_retry() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        let limits = f.core.runtime_facts.limits;
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| {
                c.modify_resonance(a, 4, 1, false);
                native::poll()
            })
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "resonance_gain_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f.core);
        let error = f
            .core
            .runtime_transaction(|c| {
                c.modify_resonance(a, 4, 1, false);
                native::poll()
            })
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
        assert_eq!(snapshot(&f.core), before);
        f.core.runtime_facts.limits = limits;
        gain(&mut f.core, a, 4, 1);
        assert_eq!(
            (
                f.core.units[&a].resonance_current,
                witness(&f.core).remainder_after
            ),
            (4, 80)
        );
    }
    #[test]
    fn composition_resonance_gain_clone_observation_and_native_timestamp_reuse_preserve_fraction() {
        let mut f = setup(20, "AutomaticAndAuthored");
        let a = f.caster;
        f.core
            .step_battle_execution_until(&mut f.state, 4000)
            .unwrap();
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut state = f.state.clone();
        f.core
            .step_battle_execution_until(&mut f.state, 5000)
            .unwrap();
        plain.step_battle_execution_until(&mut state, 5000).unwrap();
        assert_eq!(f.core.units[&a].resonance_current, 6);
        assert_eq!(plain.units[&a].resonance_current, 6);
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            plain.transaction_debug_snapshot()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            plain.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            serde_json::to_value(plain.runtime_fact_view()).unwrap()
        );
    }
}
