mod compiled_overlay {
    use super::compiled_action_fixture::{cast, fixture_content_with_resources, place};
    use super::*;
    use crate::game::battle::core::sim::{BattleExecutionState, BattleLiveCommand};
    use crate::game::battle::event_log::BattleEventRootCause;
    fn fixture(
        equipped: bool,
        cap: u32,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let native = crate::authoring::composition::canonical_overlay_tests::fixture();
        let (mut c, s, a, e, b) = fixture_content_with_resources(
            &["piercing-damage-increase"],
            if equipped { &[0] } else { &[] },
            native,
            |w| r12_lifesteal_weapon(w, cap),
            serde_json::from_value(serde_json::json!({"resources":[{
                "id":"overlay_resource_probe","owner":"Employee","lifetime":"Attempt","initial":0,"cap":1
            }]})).unwrap(),
        );
        c.units.get_mut(&b).unwrap().owner = Side::Opponent;
        c.units.get_mut(&b).unwrap().fragment_source = None;
        c.units.get_mut(&b).unwrap().skill_id = None;
        place(&mut c, e, 4, 1);
        place(&mut c, b, 6, 1);
        for id in [a, e, b] {
            let u = c.units.get_mut(&id).unwrap();
            u.stats.max_health = 1000;
            u.stats.current_health = 1000;
            u.stats.defense = 0;
        }
        c.units.get_mut(&a).unwrap().stats.current_health = 500;
        (c, s, a, e, b)
    }
    fn attack(c: &mut BattleCore, a: UnitInstanceId, e: UnitInstanceId, now: u64) {
        c.units.get_mut(&a).unwrap().next_basic_attack_ms = now;
        c.event_queue.push(BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: a,
            target_instance_id: Some(e),
            schedule_next: true,
            cause: BattleEventCause::Root {
                kind: BattleEventRootCause::Period,
            },
        });
    }
    fn reinput(
        c: &mut BattleCore,
        a: UnitInstanceId,
        now: u64,
    ) -> Result<crate::game::battle::core::sim::BattleLiveCommandOutcome, GameError> {
        let activation = c
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .find(|s| s.caster == a)
            .unwrap()
            .activation;
        c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            activation,
            target: None,
            time_ms: now,
        })
    }
    #[test]
    fn compiled_overlay_orders_actual_contacts_resets_each_attack_and_caps_aggregate_weapon_heal() {
        for equipped in [false, true] {
            let (mut c, mut s, a, e, b) = fixture(equipped, 17);
            let original = (
                c.units[&a].basic_attack.interval_ms,
                c.units[&a].basic_attack.windup_ms,
                c.units[&a].block_capacity,
                c.units[&a].body.position,
            );
            let next = c.units[&a].next_basic_attack_ms;
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(c.units[&a].next_basic_attack_ms, next);
            assert_eq!(
                (
                    c.units[&a].basic_attack.interval_ms,
                    c.units[&a].basic_attack.windup_ms,
                    c.units[&a].block_capacity,
                    c.units[&a].body.position
                ),
                original
            );
            assert!(c.resonance_use_has_obligations(a));
            attack(&mut c, a, e, 100);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 960);
            assert_eq!(
                c.units[&b].stats.current_health,
                if equipped { 950 } else { 960 }
            );
            assert_eq!(c.units[&a].stats.current_health, 517);
            assert_eq!(c.runtime_fact_summary().heal_mutations, 1);
            attack(&mut c, a, e, 200);
            c.step_battle_execution_until(&mut s, 201).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 920);
            assert_eq!(
                c.units[&b].stats.current_health,
                if equipped { 900 } else { 920 }
            );
            assert_eq!(c.units[&a].stats.current_health, 534);
            assert_eq!(c.runtime_fact_summary().heal_mutations, 2);
            assert!(c.projectiles.is_empty());
        }
    }
    #[test]
    fn compiled_overlay_counts_fully_shielded_contact_and_excludes_off_line_target() {
        let (mut c, mut s, a, e, b) = fixture(true, 100);
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.begin_sustained(
            e,
            e,
            crate::game::ability::sustained::SustainedEffectDef::Shield {
                duration_ms: 1000,
                amount: 1000,
                priority: 0,
                shared_radius_units: None,
            },
            1,
        )
        .unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(c.units[&b].stats.current_health, 950);
        assert_eq!(c.units[&a].stats.current_health, 512);
        place(&mut c, b, 6, 3);
        attack(&mut c, a, e, 200);
        c.step_battle_execution_until(&mut s, 201).unwrap();
        assert_eq!(c.units[&b].stats.current_health, 950);
        assert_eq!(c.units[&a].stats.current_health, 512);
    }
    #[test]
    fn compiled_overlay_reinput_has_millisecond_power_no_cost_or_basic_lifesteal_and_is_once() {
        for (now, expected) in [(501, 40), (502, 40), (1002, 30), (1999, 11)] {
            let (mut c, mut s, a, e, b) = fixture(true, 100);
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, now).unwrap();
            let cost = c.runtime_fact_summary().cost_mutations;
            let activation = c.sustained_effect_snapshot().unwrap().active[0].activation;
            reinput(&mut c, a, now).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 1000 - expected);
            assert_eq!(c.units[&b].stats.current_health, 1000 - expected);
            assert_eq!(c.units[&a].stats.current_health, 500);
            assert_eq!(c.runtime_fact_summary().cost_mutations, cost);
            assert_eq!(c.units[&a].resonance_current, 0);
            assert!(!c.resonance_use_has_obligations(a));
            assert!(c.basic_line_profile(a).is_none());
            assert!(c
                .typed_resources()
                .unwrap()
                .scopes()
                .all(|(_, lifetime, _, _)| !matches!(
                    lifetime,
                    crate::game::resource::ResourceLifetime::Activation
                        | crate::game::resource::ResourceLifetime::Chain
                        | crate::game::resource::ResourceLifetime::Delivery
                )));
            assert!(c
                .apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                    unit_id: a,
                    skill_id: "r11_aura".into(),
                    activation,
                    target: None,
                    time_ms: now
                })
                .is_err());
        }
    }
    #[test]
    fn compiled_overlay_natural_expiry_has_no_finish_and_restores_original_profile() {
        let (mut c, mut s, a, e, b) = fixture(true, 100);
        let original = serde_json::to_value(&c.units[&a].basic_attack).unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert!(c.basic_line_profile(a).is_none());
        assert_eq!(
            serde_json::to_value(&c.units[&a].basic_attack).unwrap(),
            original
        );
        assert!(!c.resonance_use_has_obligations(a));
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(c.units[&b].stats.current_health, 1000);
    }
    #[test]
    fn compiled_overlay_line_settlement_fault_restores_health_queue_and_owner() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "basic_line_before_hit",
            "basic_lifesteal_after_heal",
            "basic_line_after_settlement",
        ] {
            let (mut c, mut s, a, e, _) = fixture(true, 100);
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            attack(&mut c, a, e, 100);
            c.step_battle_execution_until(&mut s, 100).unwrap();
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{s:?}"),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || c.step_battle_execution_until(&mut s, 101),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    format!("{s:?}")
                ),
                before,
                "{site}"
            );
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(c.runtime_fact_summary().damage_hp_lost, 90);
        }
    }
    #[test]
    fn compiled_overlay_same_input_routes_to_finish_and_common_control_gate_preserves_the_buff() {
        for buff in ["r11_silence", "r11_stun"] {
            let (mut c, mut s, a, e, _) = fixture(false, 100);
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let initial = c.sustained_effect_snapshot().unwrap().active[0].clone();
            c.process_event_for_test(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: e,
                    target_instance_id: a,
                    buff_id: crate::game::battle::buffs::BuffId::from_name(buff),
                    duration_ms: 500,
                    cause: Default::default(),
                },
                1,
            )
            .unwrap();
            c.step_battle_execution_until(&mut s, 250).unwrap();
            let before = c.transaction_debug_snapshot();
            assert!(c
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: a,
                    skill_id: "r11_aura".into(),
                    target: None,
                    time_ms: 250
                })
                .is_err());
            assert!(reinput(&mut c, a, 250).is_err());
            assert_eq!(c.transaction_debug_snapshot(), before);
            assert_eq!(
                c.sustained_effect_snapshot().unwrap().active[0].expires_at_ms,
                initial.expires_at_ms
            );
            c.step_battle_execution_until(&mut s, 502).unwrap();
            cast(&mut c, a, 502);
            assert!(c.basic_line_profile(a).is_none());
            assert_eq!(c.units[&e].stats.current_health, 960);
        }
    }
    #[test]
    fn compiled_overlay_rejects_retroactive_reinput_and_wrong_source_without_consuming_right() {
        let (mut c, mut s, a, _, b) = fixture(false, 100);
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 501).unwrap();
        let activation = c.sustained_effect_snapshot().unwrap().active[0].activation;
        let before = c.transaction_debug_snapshot();
        assert!(reinput(&mut c, a, 400).is_err());
        for (actor, skill) in [(b, "r11_aura"), (a, "wrong")] {
            assert!(c
                .apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                    unit_id: actor,
                    skill_id: skill.into(),
                    activation,
                    target: None,
                    time_ms: 501
                })
                .is_err());
        }
        assert_eq!(c.transaction_debug_snapshot(), before);
        reinput(&mut c, a, 501).unwrap();
    }

    #[test]
    fn compiled_overlay_air_capability_and_friendly_exclusion_follow_the_equipped_weapon() {
        for (air, friendly, hit) in [
            (false, false, false),
            (true, false, true),
            (true, true, false),
        ] {
            let (mut c, mut s, a, e, b) = fixture(false, 100);
            c.units.get_mut(&a).unwrap().basic_attack.air_capable = air;
            c.units.get_mut(&b).unwrap().mobility_kind =
                crate::game::battle::types::MobilityKind::Airborne;
            if friendly {
                c.units.get_mut(&b).unwrap().owner = Side::Player;
            }
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            attack(&mut c, a, e, 100);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(
                c.units[&b].stats.current_health,
                if hit { 960 } else { 1000 }
            );
        }
    }
    #[test]
    fn compiled_overlay_caps_third_contact_and_excludes_overkill_from_one_heal() {
        let (mut c, mut s, a, e, b) = fixture(true, 100);
        let mut third = c.units[&e].clone();
        let id = UnitInstanceId(Uuid::from_u128(0xbaba));
        third.instance_id = id;
        c.units.insert(id, third);
        place(&mut c, id, 7, 1);
        c.units.get_mut(&e).unwrap().stats.current_health = 3;
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 0);
        assert_eq!(c.units[&b].stats.current_health, 950);
        assert_eq!(c.units[&id].stats.current_health, 940);
        assert_eq!(c.units[&a].stats.current_health, 528); // floor((3+50+60)*25/100)
        assert_eq!(c.runtime_fact_summary().heal_mutations, 1);
    }
    #[test]
    fn compiled_overlay_defense_stance_and_exit_do_not_create_or_store_a_finish() {
        let (mut c, mut s, a, e, _) = fixture(false, 100);
        c.initialize_deployed_tactical_state(
            a,
            crate::game::battle::types::TacticalTileKind::Ground,
            Default::default(),
            0,
        )
        .unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1501).unwrap();
        c.apply_live_command(BattleLiveCommand::SetEngagementStance {
            unit_id: a,
            stance: crate::game::battle::types::EngagementStance::Defense,
            time_ms: 1501,
        })
        .unwrap();
        assert!(reinput(&mut c, a, 1501).is_err());
        assert!(c.basic_line_profile(a).is_some());
        c.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 1501,
        })
        .unwrap();
        assert!(c.basic_line_profile(a).is_none());
        assert_eq!(c.units[&e].stats.current_health, 1000);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 1000);
    }
    #[test]
    fn compiled_overlay_resource_handoff_cancel_restores_and_retry_releases_scopes() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "reinput_after_consume",
            "reinput_after_invoke",
            "reinput_after_execution",
        ] {
            let (mut c, mut state, actor, _, _) = fixture(false, 17);
            cast(&mut c, actor, 0);
            c.step_battle_execution_until(&mut state, 501).unwrap();
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                c.typed_resources().unwrap().scopes().collect::<Vec<_>>(),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || reinput(&mut c, actor, 501),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    c.typed_resources().unwrap().scopes().collect::<Vec<_>>()
                ),
                before
            );
            reinput(&mut c, actor, 501).unwrap();
            assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
            assert!(c
                .typed_resources()
                .unwrap()
                .scopes()
                .all(|(_, lifetime, _, _)| !matches!(
                    lifetime,
                    crate::game::resource::ResourceLifetime::Activation
                        | crate::game::resource::ResourceLifetime::Chain
                        | crate::game::resource::ResourceLifetime::Delivery
                )));
        }
    }
}
