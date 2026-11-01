mod compiled_channel {
    use super::compiled_action_fixture::{cast, fixture_graph, hit, place};
    use super::*;
    use crate::{
        authoring::node_graph::tests::channel_ground_fixture,
        game::battle::{
            core::sim::{BattleExecutionState, BattleLiveCommand},
            damage::BattleCommand,
            types::UnitTargetTrait,
        },
    };
    const RELICS: [&str; 5] = [
        "widened-horizon",
        "long-breath",
        "unbroken-trance",
        "broken-balance",
        "tightening-shackles",
    ];
    fn fixture(
        equipped: &[usize],
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        graph_fixture(equipped, channel_ground_fixture())
    }
    fn graph_fixture(
        equipped: &[usize],
        doc: serde_json::Value,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let (mut core, state, a, e, b) = fixture_graph(&RELICS, equipped, doc);
        place(&mut core, e, 3, 1);
        core.units
            .get_mut(&e)
            .unwrap()
            .stats
            .move_speed_units_per_ms = 1000;
        (core, state, a, e, b)
    }
    fn elapsed(core: &mut BattleCore, state: &mut BattleExecutionState, now: u64) {
        core.step_battle_execution_until(state, now).unwrap();
    }
    fn slow(core: &BattleCore, e: UnitInstanceId) -> u32 {
        core.units[&e].stats.move_speed_units_per_ms
    }
    fn stunned(core: &BattleCore, e: UnitInstanceId, now: u64) -> bool {
        core.unit_control_state(e, now).unwrap().hard_cc_active()
    }
    #[test]
    fn compiled_channel_ramp_half_open_lifetime_locks_and_independent_damage_taken() {
        let (mut c, mut s, a, e, b) = fixture(&[]);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1);
        assert!(
            c.channel_active(a) && c.resonance_use_has_obligations(a) && c.channel_pauses_basic(a)
        );
        assert_eq!(slow(&c, e), 1000);
        assert_eq!(c.units[&a].resonance_current, 0);
        let hp = c.units[&e].stats.current_health;
        hit(&mut c, b, e, 100, 1);
        assert_eq!(c.units[&e].stats.current_health, hp - 120);
        elapsed(&mut c, &mut s, 101);
        assert_eq!(slow(&c, e), 700);
        assert!(!stunned(&c, e, 101));
        elapsed(&mut c, &mut s, 201);
        assert_eq!(slow(&c, e), 400);
        assert!(stunned(&c, e, 201));
        elapsed(&mut c, &mut s, 601);
        assert_eq!(slow(&c, e), 1000);
        assert!(!stunned(&c, e, 601));
        assert!(!c.channel_active(a));
        assert!(!c.resonance_use_has_obligations(a));
        assert!(c.active_areas.is_empty());
        assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries, 0);
    }
    #[test]
    fn compiled_channel_exit_reentry_resets_ramp_but_arrived_stun_outlives_exit() {
        let (mut c, mut s, a, e, b) = fixture(&[3]);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 101);
        assert_eq!(slow(&c, e), 700);
        place(&mut c, e, 6, 2);
        c.runtime_transaction(|c| c.reconcile_sustained(101))
            .unwrap();
        assert_eq!(slow(&c, e), 1000);
        place(&mut c, e, 3, 1);
        c.runtime_transaction(|c| c.reconcile_sustained(151))
            .unwrap();
        assert_eq!(slow(&c, e), 1000);
        elapsed(&mut c, &mut s, 301);
        assert!(!stunned(&c, e, 301));
        assert_eq!(slow(&c, e), 550);
        elapsed(&mut c, &mut s, 351);
        assert!(stunned(&c, e, 351));
        let hp = c.units[&e].stats.current_health;
        hit(&mut c, b, e, 100, 351);
        assert_eq!(c.units[&e].stats.current_health, hp - 150);
        place(&mut c, e, 6, 2);
        c.runtime_transaction(|c| c.reconcile_sustained(351))
            .unwrap();
        assert!(stunned(&c, e, 351));
        assert_eq!(slow(&c, e), 1000);
        let hp = c.units[&e].stats.current_health;
        hit(&mut c, b, e, 100, 351);
        assert_eq!(c.units[&e].stats.current_health, hp - 130);
        elapsed(&mut c, &mut s, 601);
        assert!(!stunned(&c, e, 601));
        assert!(!c.sustained_effect_snapshot().unwrap().active.is_empty());
        elapsed(&mut c, &mut s, 5351);
        assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
        assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries, 0);
    }
    #[test]
    fn compiled_channel_slow_and_stun_immunity_leave_entry_amplification_and_progress_independent()
    {
        for immune in [UnitTargetTrait::SlowImmune, UnitTargetTrait::StunImmune] {
            let (mut c, mut s, a, e, b) = fixture(&[3]);
            c.units.get_mut(&e).unwrap().target_traits.push(immune);
            if immune == UnitTargetTrait::StunImmune {
                c.process_commands(
                    vec![BattleCommand::ApplyBuff {
                        caster_id: a,
                        target_id: e,
                        buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                        duration_ms: 500,
                    }],
                    0,
                );
                assert!(!stunned(&c, e, 0));
            }
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 101);
            assert_eq!(
                slow(&c, e),
                if immune == UnitTargetTrait::SlowImmune {
                    1000
                } else {
                    700
                }
            );
            elapsed(&mut c, &mut s, 201);
            assert_eq!(stunned(&c, e, 201), immune != UnitTargetTrait::StunImmune);
            let hp = c.units[&e].stats.current_health;
            hit(&mut c, b, e, 100, 201);
            assert_eq!(
                c.units[&e].stats.current_health,
                hp - if immune == UnitTargetTrait::StunImmune {
                    120
                } else {
                    150
                }
            );
        }
    }
    #[test]
    fn compiled_channel_real_equipped_range_duration_and_progress_modifiers_are_once_and_separate()
    {
        for (mods, end, stun_at) in [
            (vec![], 601, 201),
            (vec![1, 1], 751, 201),
            (vec![4, 4], 601, 101),
            (vec![1, 4], 751, 101),
        ] {
            let (mut c, mut s, a, e, _) = fixture(&mods);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1);
            assert_eq!(c.active_areas.values().next().unwrap().expires_at_ms, end);
            elapsed(&mut c, &mut s, stun_at - 1);
            assert!(!stunned(&c, e, stun_at - 1));
            elapsed(&mut c, &mut s, stun_at);
            assert!(stunned(&c, e, stun_at));
            elapsed(&mut c, &mut s, end);
            assert!(c.active_areas.is_empty());
        }
        for expanded in [false, true] {
            let (mut c, mut s, a, e, _) = fixture(if expanded { &[0] } else { &[] });
            place(&mut c, e, 4, 1);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 201);
            assert_eq!(stunned(&c, e, 201), expanded);
        }
    }
    #[test]
    fn compiled_channel_direct_stop_ignores_empty_resonance_and_recast_gets_a_new_clock() {
        let (mut c, mut s, a, e, _) = fixture(&[]);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 201);
        cast(&mut c, a, 201);
        elapsed(&mut c, &mut s, 201);
        assert!(!c.channel_active(a) && !stunned(&c, e, 201));
        assert_eq!(slow(&c, e), 1000);
        assert_eq!(c.units[&a].resonance_current, 0);
        c.units.get_mut(&a).unwrap().resonance_current = 100;
        cast(&mut c, a, 202);
        elapsed(&mut c, &mut s, 203);
        assert_eq!(slow(&c, e), 1000);
        elapsed(&mut c, &mut s, 403);
        assert!(stunned(&c, e, 403));
        elapsed(&mut c, &mut s, 803);
        assert!(!c.channel_active(a));
    }
    #[test]
    fn compiled_channel_protection_changes_hard_cc_cancellation_not_silence_or_voluntary_stop() {
        for protected in [false, true] {
            for buff in ["r11_stun", "r11_silence"] {
                let (mut c, mut s, a, e, _) = fixture(if protected { &[2] } else { &[] });
                cast(&mut c, a, 0);
                elapsed(&mut c, &mut s, 101);
                c.runtime_transaction(|c| {
                    c.process_commands(
                        vec![BattleCommand::ApplyBuff {
                            caster_id: e,
                            target_id: a,
                            buff_id: crate::game::battle::buffs::BuffId::from_name(buff),
                            duration_ms: 300,
                        }],
                        101,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
                elapsed(&mut c, &mut s, 201);
                assert_eq!(c.channel_active(a), protected || buff == "r11_silence");
                if c.channel_active(a) {
                    cast(&mut c, a, 201);
                    elapsed(&mut c, &mut s, 201);
                    assert!(!c.channel_active(a));
                }
            }
        }
    }
    #[test]
    fn compiled_channel_caster_withdrawal_death_and_terminal_remove_owned_stun() {
        for reason in [0, 1, 2] {
            let (mut c, mut s, a, e, b) = fixture(&[2]);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 201);
            match reason {
                0 => {
                    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 201,
                    })
                    .unwrap();
                }
                1 => {
                    hit(&mut c, e, a, 2000, 201);
                }
                _ => {
                    std::sync::Arc::make_mut(&mut c.scenario).terminal_outcomes =
                        BattleScenario::fixture_survival_outcomes(250);
                    elapsed(&mut c, &mut s, 251);
                    c.finalize_battle_execution(&mut s).unwrap();
                }
            }
            assert!(c.active_areas.is_empty(), "reason {reason}");
            assert!(!stunned(&c, e, 251));
            assert_eq!(slow(&c, e), 1000);
            let _ = b;
        }
    }
    #[test]
    fn compiled_channel_admission_geometry_is_fixed_before_release_and_later_movement() {
        let mut doc = channel_ground_fixture();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-range" {
                n["values"]["pattern"] = serde_json::json!("X\n@");
                n["values"]["includeAnchor"] = serde_json::json!(false);
            }
        }
        let (mut c, mut s, a, e, _) = graph_fixture(&[], doc);
        c.units.get_mut(&a).unwrap().facing_direction =
            Some(crate::game::battle::tile_range::FacingDirection::Right);
        cast(&mut c, a, 0);
        place(&mut c, a, 6, 2);
        c.units.get_mut(&a).unwrap().facing_direction =
            Some(crate::game::battle::tile_range::FacingDirection::Left);
        elapsed(&mut c, &mut s, 201);
        assert!(stunned(&c, e, 201));
        let area = c.active_areas.values().next().unwrap();
        assert_eq!(area.tile_anchor, Position::new(2, 1));
    }
    #[test]
    fn compiled_channel_expiry_wins_over_ramp_arrival_at_same_timestamp() {
        let mut doc = channel_ground_fixture();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-seconds" && n["values"]["value"] == serde_json::json!(0.2) {
                n["values"]["value"] = serde_json::json!(0.6);
            }
        }
        let (mut c, mut s, a, e, _) = graph_fixture(&[3], doc);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 601);
        assert!(!stunned(&c, e, 601));
        assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
    }
    #[test]
    fn compiled_channel_registration_ramp_stun_and_end_faults_restore_exact_state_then_retry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, time) in [
            ("ground_reconciled", 1),
            ("ground_reconciled", 101),
            ("sustained_started", 201),
            ("area_expired", 601),
        ] {
            let (mut c, mut s, a, e, _) = fixture(&[3]);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, time - 1);
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}"),
                format!("{:?}", c.units[&e].stats),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || c.step_battle_execution_until(&mut s, time),
                )
            });
            assert!(
                token.is_cancelled() && result.is_err(),
                "{site}@{time}: {result:?}"
            );
            assert_eq!(
                before,
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.runtime_fact_view()).unwrap(),
                    format!("{s:?}"),
                    format!("{:?}", c.units[&e].stats)
                )
            );
            elapsed(&mut c, &mut s, 5201);
            assert!(c.active_areas.is_empty());
            assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
            assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries, 0);
        }
    }
    #[test]
    fn compiled_channel_fractional_progress_preserves_slope_before_rounded_arrival() {
        let mut doc = channel_ground_fixture();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-modifier-ramp" {
                n["values"]["factor"] = serde_json::json!(1.5);
            }
        }
        let (mut c, mut s, a, e, _) = graph_fixture(&[4], doc);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 101);
        assert_eq!(slow(&c, e), 550);
        elapsed(&mut c, &mut s, 134);
        assert!(!stunned(&c, e, 134));
        elapsed(&mut c, &mut s, 135);
        assert!(stunned(&c, e, 135));
        assert_eq!(slow(&c, e), 400);
    }
    #[test]
    fn compiled_channel_overlapping_sources_keep_strongest_ramp_and_only_remove_their_own_stun() {
        let (mut c, mut s, a, e, b) = fixture(&[]);
        place(&mut c, b, 4, 1);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 101);
        c.units.get_mut(&b).unwrap().resonance_current = 100;
        cast(&mut c, b, 101);
        elapsed(&mut c, &mut s, 102);
        assert_eq!(slow(&c, e), 700);
        elapsed(&mut c, &mut s, 302);
        assert!(stunned(&c, e, 302));
        assert_eq!(slow(&c, e), 400);
        cast(&mut c, a, 302);
        elapsed(&mut c, &mut s, 302);
        assert_eq!(c.active_areas.len(), 1);
        assert!(stunned(&c, e, 302));
        assert_eq!(slow(&c, e), 400);
        cast(&mut c, b, 302);
        elapsed(&mut c, &mut s, 302);
        assert!(c.active_areas.is_empty());
        assert_eq!(slow(&c, e), 1000);
        assert!(!stunned(&c, e, 302));
    }
    #[test]
    fn compiled_channel_basic_policy_controls_real_periodic_attacks_and_restarts_after_stop() {
        for allow in [false, true] {
            let mut doc = channel_ground_fixture();
            for n in doc["nodes"].as_array_mut().unwrap() {
                if n["kind"] == "skill-channel-begin" {
                    n["values"]["basicAttack"] =
                        serde_json::json!(if allow { "allow" } else { "pause" });
                }
            }
            let (mut c, mut s, a, e, _) = graph_fixture(&[], doc);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1);
            {
                let u = c.units.get_mut(&a).unwrap();
                u.next_basic_attack_ms = 2;
                u.stats.attack_interval_ms = 100;
            }
            c.event_queue.push(BattleEvent::AttackStart {
                time_ms: 2,
                attacker_instance_id: a,
                target_instance_id: Some(e),
                schedule_next: true,
                cause: Default::default(),
            });
            let before = c.runtime_fact_summary().damage_hp_lost;
            elapsed(&mut c, &mut s, 150);
            assert_eq!(c.runtime_fact_summary().damage_hp_lost > before, allow);
            cast(&mut c, a, 150);
            elapsed(&mut c, &mut s, 350);
            assert!(c.runtime_fact_summary().damage_hp_lost > before);
        }
    }
}
