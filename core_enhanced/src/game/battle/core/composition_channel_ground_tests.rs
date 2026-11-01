mod composition_channel_ground {
    use super::compiled_action_fixture::{cast, fixture_content_with_weapon, hit, place};
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact, DiagnosticCode};
    use crate::game::battle::core::sim::BattleExecutionState;
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/channel_ground.skill.yaml"
    ));
    fn area(d: &mut Value) -> &mut Value {
        &mut d["values"][0]["value"]["Delivery"]["TileArea"]["area"]
    }
    fn fixture(
        equipped: &[usize],
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut d = parse_yaml(YAML).unwrap();
        d["skills"][0]["id"] = json!("r11_aura");
        edit(&mut d);
        let artifact = compile_json(&d.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = crate::authoring::node_graph::NativeGraphContent {
            skills: artifact.load_native().unwrap(),
            equipment: vec![],
            mark_weapons: vec![],
        };
        let (mut c, s, a, e, b) = fixture_content_with_weapon(
            &["long-breath", "widened-horizon", "unbroken-trance", "tightening-shackles", "broken-balance"],
            equipped,
            native,
            |_| {},
        );
        place(&mut c, e, 3, 1);
        c.units.get_mut(&e).unwrap().stats.move_speed_units_per_ms = 1000;
        (c, s, a, e, b)
    }
    fn elapsed(c: &mut BattleCore, s: &mut BattleExecutionState, t: u64) {
        c.step_battle_execution_until(s, t).unwrap();
    }
    fn stunned(c: &BattleCore, e: UnitInstanceId, t: u64) -> bool {
        c.unit_control_state(e, t).unwrap().hard_cc_active()
    }
    #[test]
    fn composition_channel_ground_validates_unused_delivery_and_tracks_lifetime_input() {
        let d = parse_yaml(YAML).unwrap();
        let a = compile_json(&d.to_string()).unwrap();
        let trace = &a.sources()
            ["skills[0].action_policy.modifiers[0].change.Area.change.Duration.percent"];
        assert_eq!(trace.definition, "values[1].value.Count");
        assert!(a.sources()["skills[0].steps[0].delivery"]
            .inputs
            .iter()
            .any(|p| p == "values[1].value"));
        for unused in [false, true] {
            for mode in ["lifetime", "status", "rule", "mask"] {
                let mut invalid = d.clone();
                match mode {
                    "lifetime" => area(&mut invalid)["duration_ms"] = json!(0),
                    "status" => {
                        area(&mut invalid)["ground_status"]["statuses"][0]["kind"]["RampingSlow"]
                            ["duration_ms"] = json!(0)
                    }
                    "rule" => area(&mut invalid)["ground_status"]["rules"][0]["status"] = json!(99),
                    _ => {
                        invalid["values"][0]["value"]["Delivery"]["TileArea"]["footprint"]["rows"] =
                            json!(["XX"])
                    }
                }
                if unused {
                    invalid["skills"][0]["body"]["Call"]["arguments"][0]["value"] =
                        json!({"Literal":d["values"][0]["value"]});
                }
                let e = compile_json(&invalid.to_string()).unwrap_err();
                assert_eq!(e.code, DiagnosticCode::Native, "{mode}/{unused}: {e:?}");
                assert!(
                    e.source
                        .definition
                        .starts_with("values[0].value.Delivery.TileArea"),
                    "{e:?}"
                );
            }
        }
        for value in [
            json!({"Count":0}),
            json!({"Count":2147483648u32}),
            json!({"Milliseconds":25}),
        ] {
            let mut invalid = d.clone();
            invalid["values"][1]["value"] = value;
            assert!(compile_json(&invalid.to_string()).is_err());
        }
        let mut invalid = d;
        area(&mut invalid)["mode"] = json!("SelectedGround");
        let e = compile_json(&invalid.to_string()).unwrap_err();
        assert!(
            e.source.definition.contains("AreaDuration") && e.reason.contains("channel-owned"),
            "{e:?}"
        );
    }
    #[test]
    fn composition_channel_ground_membership_locks_fixed_origin_and_extended_lifetime() {
        for equipped in [vec![], vec![0], vec![0, 0], vec![0, 1]] {
            let (mut c, mut s, a, e, b) = fixture(&equipped, |_| {});
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1);
            assert!(
                c.channel_active(a)
                    && c.channel_pauses_basic(a)
                    && c.resonance_use_has_obligations(a)
            );
            assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
            let hp = c.units[&e].stats.current_health;
            hit(&mut c, b, e, 100, 1);
            assert_eq!(c.units[&e].stats.current_health, hp - 120);
            elapsed(&mut c, &mut s, 501);
            assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 600);
            assert!(!stunned(&c, e, 501));
            place(&mut c, a, 6, 2);
            elapsed(&mut c, &mut s, 1001);
            assert!(stunned(&c, e, 1001));
            assert_eq!(
                c.active_areas.values().next().unwrap().tile_anchor,
                Position::new(2, 1)
            );
            let end = if equipped.is_empty() { 2002 } else { 2503 };
            assert_eq!(c.active_areas.values().next().unwrap().expires_at_ms, end);
            elapsed(&mut c, &mut s, end - 1);
            assert!(stunned(&c, e, end - 1));
            elapsed(&mut c, &mut s, end);
            assert!(
                !stunned(&c, e, end) && !c.channel_active(a) && !c.resonance_use_has_obligations(a)
            );
            assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
            assert!(c.active_areas.is_empty());
            assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries, 0);
        }
    }
    #[test]
    fn composition_channel_ground_exit_reentry_and_same_time_expiry() {
        let (mut c, mut s, a, e, b) = fixture(&[], |_| {});
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 501);
        place(&mut c, e, 7, 2);
        c.runtime_transaction(|c| c.reconcile_sustained(501))
            .unwrap();
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
        place(&mut c, e, 3, 1);
        c.runtime_transaction(|c| c.reconcile_sustained(601))
            .unwrap();
        elapsed(&mut c, &mut s, 1600);
        assert!(!stunned(&c, e, 1600));
        elapsed(&mut c, &mut s, 1601);
        assert!(stunned(&c, e, 1601));
        place(&mut c, e, 7, 2);
        c.runtime_transaction(|c| c.reconcile_sustained(1601))
            .unwrap();
        assert!(stunned(&c, e, 1601));
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
        let hp = c.units[&e].stats.current_health;
        hit(&mut c, b, e, 100, 1601);
        assert_eq!(c.units[&e].stats.current_health, hp - 100);
        cast(&mut c, a, 1601);
        elapsed(&mut c, &mut s, 1601);
        assert!(c.active_areas.is_empty());
        assert!(!stunned(&c, e, 1601));
        let (mut c, mut s, a, e, _) = fixture(&[], |d| area(d)["duration_ms"] = json!(1000));
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1001);
        assert!(c.active_areas.is_empty() && !stunned(&c, e, 1001));
    }
    #[test]
    fn composition_channel_ground_immunity_and_other_subscription_definition() {
        use crate::game::battle::types::UnitTargetTrait;
        for immune in [UnitTargetTrait::SlowImmune, UnitTargetTrait::StunImmune] {
            let (mut c, mut s, a, e, b) = fixture(&[], |_| {});
            c.units.get_mut(&e).unwrap().target_traits.push(immune);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1001);
            assert_eq!(stunned(&c, e, 1001), immune != UnitTargetTrait::StunImmune);
            let hp = c.units[&e].stats.current_health;
            hit(&mut c, b, e, 100, 1001);
            assert_eq!(c.units[&e].stats.current_health, hp - 120);
            if immune == UnitTargetTrait::SlowImmune {
                assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
            }
        }
        // Same shared Action with a different authored entry/exit program.
        let (mut c, mut s, a, e, _) = fixture(&[0], |d| {
            area(d)["ground_status"] = json!({"statuses":[{"definition":"entry-only","kind":{"Slow":{"percent":25}},"overlap":"Strongest","lifetime":"UntilExit"}],"rules":[{"trigger":"Enter","status":0,"action":"Apply"}]});
        });
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1001);
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 750);
        assert!(!stunned(&c, e, 1001));
        elapsed(&mut c, &mut s, 2503);
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
    }
    #[test]
    fn composition_channel_ground_cancel_restores_transaction_and_retry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, time) in [
            ("ground_reconciled", 1),
            ("ground_reconciled", 501),
            ("ground_reconciled", 1001),
            ("area_expired", 2503),
            ("area_observation_committed", 1),
            ("area_observation_committed", 501),
            ("area_observation_committed", 2503),
        ] {
            let (mut c, mut s, a, _, _) = fixture(&[0], |_| {});
            observe_area(&mut c);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, time - 1);
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}"),
                serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
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
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap()
                )
            );
            elapsed(&mut c, &mut s, 2503);
            assert!(c.active_areas.is_empty());
        }
    }
    #[test]
    fn composition_channel_ground_observation_does_not_change_authoritative_state() {
        let mut outcomes = Vec::new();
        // Reuse the exact initial owners/hashers. Debug output from separately
        // constructed HashMaps has unrelated iteration order.
        let (initial, initial_state, a, e, _) = fixture(&[0], |_| {});
        for observe in [false, true] {
            let (mut c, mut s) = (initial.clone(), initial_state.clone());
            if observe {
                c.enable_authoring_observation_with_options(Default::default(), true);
                c.bind_authoring_observation(
                    crate::game::battle::core::authoring_observation::ObservationKey {
                        entry_ordinal: 1,
                        battle_uuid: Uuid::from_u128(0xb30b),
                    },
                )
                .unwrap();
            }
            cast(&mut c, a, 0);
            let mut snapshots = Vec::new();
            for t in [1, 501, 1001, 2503] {
                elapsed(&mut c, &mut s, t);
                snapshots.push((
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.runtime_fact_view()).unwrap(),
                    format!("{s:?}"),
                ));
            }
            assert!(c.active_areas.is_empty() && !stunned(&c, e, 2503));
            outcomes.push(snapshots);
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }
    fn observe_area(c: &mut BattleCore) {
        c.enable_authoring_observation();
        c.bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1,
                battle_uuid: Uuid::from_u128(0xb31),
            },
        )
        .unwrap();
    }
    fn latest_area(c: &BattleCore) -> &crate::authoring::AreaWitness {
        c.authoring_observation()
            .unwrap()
            .unwrap()
            .latest_area
            .as_ref()
            .unwrap()
    }
    #[test]
    fn composition_channel_ground_observed_membership_and_own_lifetime() {
        use crate::authoring::{AreaChange, AreaMembership};
        let (mut c, mut s, a, e, _) = fixture(&[0], |_| {});
        observe_area(&mut c);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1);
        let first = latest_area(&c).clone();
        assert_eq!(first.change, AreaChange::Reconciled);
        assert_eq!(
            first.before,
            Some(AreaMembership {
                members: 0,
                contributions: 0
            })
        );
        assert_eq!(
            first.after,
            Some(AreaMembership {
                members: 1,
                contributions: 2
            })
        );
        assert_eq!(
            (first.started_at_ms, first.expires_at_ms, first.time_ms),
            (1, 2503, 1)
        );
        assert_eq!(first.caster, a);
        assert_eq!(c.authoring_observation().unwrap().unwrap().area_changes, 2);
        elapsed(&mut c, &mut s, 1001);
        assert_eq!(latest_area(&c).after.unwrap().contributions, 3);
        place(&mut c, e, 7, 2);
        c.runtime_transaction(|c| c.reconcile_sustained(1001))
            .unwrap();
        assert_eq!(
            latest_area(&c).after,
            Some(AreaMembership {
                members: 0,
                contributions: 1
            })
        );
        elapsed(&mut c, &mut s, 2503);
        let last = latest_area(&c);
        assert_eq!(last.area_id, first.area_id);
        assert_eq!(last.cast_sequence, first.cast_sequence);
        assert_eq!(
            last.change,
            AreaChange::Ended {
                reason: crate::game::ability::sustained::SustainedEndReason::Expired
            }
        );
        assert_eq!(
            last.before,
            Some(AreaMembership {
                members: 0,
                contributions: 1
            })
        );
        assert!(last.after.is_none());
        assert_eq!(last.time_ms, 2503);
    }
    #[test]
    fn composition_channel_ground_observed_real_stop_cc_withdraw_death_and_terminal() {
        use crate::game::{
            ability::sustained::SustainedEndReason as End,
            battle::{core::sim::BattleLiveCommand, damage::BattleCommand},
        };
        for end in [
            End::Stopped,
            End::HardCrowdControl,
            End::SourceUnavailable,
            End::Terminal,
        ] {
            let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
            observe_area(&mut c);
            cast(&mut c, a, 0);
            let now = if end == End::HardCrowdControl {
                501
            } else {
                1001
            };
            elapsed(&mut c, &mut s, now);
            match end {
                End::Stopped => {
                    cast(&mut c, a, now);
                    elapsed(&mut c, &mut s, now);
                }
                End::HardCrowdControl => {
                    c.runtime_transaction(|c| {
                        c.process_commands(
                            vec![BattleCommand::ApplyBuff {
                                caster_id: e,
                                target_id: a,
                                buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                                duration_ms: 300,
                            }],
                            now,
                        );
                        crate::game::battle::runtime_control::poll()
                    })
                    .unwrap();
                }
                End::SourceUnavailable => {
                    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: now,
                    })
                    .unwrap();
                }
                _ => {
                    std::sync::Arc::make_mut(&mut c.scenario).terminal_outcomes =
                        BattleScenario::fixture_survival_outcomes(1100);
                    elapsed(&mut c, &mut s, 1101);
                    c.finalize_battle_execution(&mut s).unwrap();
                }
            }
            if end == End::HardCrowdControl {
                elapsed(&mut c, &mut s, now + 100);
            }
            assert!(c.active_areas.is_empty(), "{end:?}");
            assert_eq!(
                latest_area(&c).change,
                crate::authoring::AreaChange::Ended { reason: end }
            );
            assert_eq!(latest_area(&c).caster, a);
            assert!(!stunned(&c, e, 1101));
        }
        let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
        observe_area(&mut c);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1001);
        hit(&mut c, e, a, 2000, 1001);
        assert_eq!(
            latest_area(&c).change,
            crate::authoring::AreaChange::Ended {
                reason: End::SourceUnavailable
            }
        );
    }
    #[test]
    fn composition_channel_ground_observed_overlapping_casters_remain_distinct() {
        let (mut c, mut s, a, e, b) = fixture(&[], |_| {});
        observe_area(&mut c);
        place(&mut c, b, 4, 1);
        cast(&mut c, a, 0);
        elapsed(&mut c, &mut s, 1);
        let first = latest_area(&c).clone();
        c.units.get_mut(&b).unwrap().resonance_current = 100;
        cast(&mut c, b, 1);
        elapsed(&mut c, &mut s, 2);
        let second = latest_area(&c).clone();
        assert_ne!(first.area_id, second.area_id);
        assert_ne!(first.cast_sequence, second.cast_sequence);
        assert_eq!(second.caster, b);
        elapsed(&mut c, &mut s, 1002);
        cast(&mut c, a, 1002);
        elapsed(&mut c, &mut s, 1002);
        assert_eq!(latest_area(&c).area_id, first.area_id);
        assert_eq!(c.active_areas.len(), 1);
        assert!(stunned(&c, e, 1002));
        cast(&mut c, b, 1002);
        elapsed(&mut c, &mut s, 1002);
        assert_eq!(latest_area(&c).area_id, second.area_id);
        assert!(c.active_areas.is_empty());
        assert!(!stunned(&c, e, 1002));
    }
    fn with_modifiers(d: &mut Value) {
        let id = d["skills"][0]["id"].clone();
        *d = parse_yaml(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/skill_composition/channel_modifiers.skill.yaml"))).unwrap();
        d["skills"][0]["id"] = id;
    }
    #[test]
    fn composition_channel_modifiers_progress_ratio_changes_slope_arrival_not_lifetime() {
        for equipped in [vec![], vec![3], vec![0,3], vec![3,3]] {
            let (mut c, mut s, a, e, _) = fixture(&equipped, with_modifiers);
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 501);
            assert_eq!(c.units[&e].stats.move_speed_units_per_ms, if equipped.is_empty() { 600 } else { 400 });
            let arrival = if equipped.is_empty() { 1001 } else { 668 };
            elapsed(&mut c, &mut s, arrival - 1);
            assert!(!stunned(&c, e, arrival - 1));
            elapsed(&mut c, &mut s, arrival);
            assert!(stunned(&c, e, arrival));
            let expiry = if equipped.contains(&0) { 2503 } else { 2002 };
            assert_eq!(c.active_areas.values().next().unwrap().expires_at_ms, expiry);
            elapsed(&mut c, &mut s, expiry);
            assert!(c.active_areas.is_empty() && !stunned(&c, e, expiry));
        }
    }
    #[test]
    fn composition_channel_modifiers_protection_keeps_cc_state_and_normal_end_conditions() {
        use crate::game::battle::{damage::BattleCommand, core::sim::BattleLiveCommand};
        for equipped in [vec![], vec![2]] {
            for end in ["stop", "withdraw", "death", "expiry", "terminal"] {
                let (mut c, mut s, a, e, _) = fixture(&equipped, with_modifiers);
                cast(&mut c, a, 0);
                elapsed(&mut c, &mut s, 101);
                c.runtime_transaction(|c| {
                    c.process_commands(vec![BattleCommand::ApplyBuff { caster_id: e, target_id: a, buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"), duration_ms: 300 }], 101);
                    crate::game::battle::runtime_control::poll()
                }).unwrap();
                elapsed(&mut c, &mut s, 201);
                assert!(stunned(&c, a, 201));
                assert_eq!(c.channel_active(a), !equipped.is_empty());
                if equipped.is_empty() { assert!(c.active_areas.is_empty()); continue; }
                match end {
                    "stop" => { cast(&mut c, a, 201); elapsed(&mut c, &mut s, 201); }
                    "withdraw" => { c.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: a, time_ms: 201 }).unwrap(); }
                    "death" => { hit(&mut c, e, a, 2000, 201); }
                    "expiry" => elapsed(&mut c, &mut s, 2002),
                    _ => { std::sync::Arc::make_mut(&mut c.scenario).terminal_outcomes = BattleScenario::fixture_survival_outcomes(300); elapsed(&mut c, &mut s, 301); c.finalize_battle_execution(&mut s).unwrap(); }
                }
                assert!(c.active_areas.is_empty() && !c.channel_active(a), "{end}");
                assert!(!c.resonance_use_has_obligations(a));
            }
        }
    }

    #[test]
    fn composition_channel_modifiers_silence_keeps_current_channel_but_blocks_new_cast() {
        use crate::game::battle::damage::BattleCommand;
        for active in [false, true] {
            let (mut c, mut s, a, e, _) = fixture(&[2], with_modifiers);
            if active { cast(&mut c, a, 0); }
            elapsed(&mut c, &mut s, 101);
            c.runtime_transaction(|c| {
                c.process_commands(vec![BattleCommand::ApplyBuff { caster_id: e, target_id: a, buff_id: crate::game::battle::buffs::BuffId::from_name("r11_silence"), duration_ms: 300 }], 101);
                crate::game::battle::runtime_control::poll()
            }).unwrap();
            elapsed(&mut c, &mut s, 201);
            assert_eq!(c.channel_active(a), active);
            if active { cast(&mut c, a, 201); }
            else {
                let error = c.apply_live_command(crate::game::battle::core::sim::BattleLiveCommand::ActivateSkill {
                    unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 201,
                }).unwrap_err();
                assert!(format!("{error:?}").contains("Silenced"), "{error:?}");
            }
            elapsed(&mut c, &mut s, 201);
            assert!(!c.channel_active(a));
            if !active { assert_eq!(c.units[&a].resonance_current, 100); }
        }
    }

    fn with_vulnerability(d: &mut Value, direct: bool) {
        let id = d["skills"][0]["id"].clone();
        *d = if direct {
            parse_yaml(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/skill_composition/freeze_vulnerability.skill.yaml"))).unwrap()
        } else {
            crate::authoring::composition::canonical_channel_tests::resolved("fragment")
        };
        d["skills"][0]["id"] = id;
    }
    fn weaknesses(c: &BattleCore, target: UnitInstanceId) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
        c.sustained_effect_snapshot().unwrap().active.into_iter().filter(|r| r.target == target && r.hit_status.as_ref().is_some_and(|p| p.modifier_source.as_ref().is_some_and(|s| s.definition == "broken-balance"))).collect()
    }
    #[test]
    fn composition_control_vulnerability_ground_success_is_once_and_outlives_area() {
        for equipped in [vec![], vec![4], vec![0,4], vec![4,4]] {
            let (mut c, mut s, a, e, b) = fixture(&equipped, |d| with_vulnerability(d, false));
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1000);
            assert!(weaknesses(&c, e).is_empty());
            elapsed(&mut c, &mut s, 1001);
            assert!(stunned(&c, e, 1001));
            let initial = weaknesses(&c, e);
            assert_eq!(initial.len(), usize::from(!equipped.is_empty()));
            if let Some(effect) = initial.first() {
                assert_eq!(effect.expires_at_ms, 6001);
                let p = effect.hit_status.as_ref().unwrap();
                assert_eq!(p.kind, crate::game::ability::sustained::HitStatusKind::DamageTaken { percent: 15 });
                let source = p.modifier_source.as_ref().unwrap();
                assert!(!source.copy_uuid.is_nil() && source.rule.ends_with(":broken-balance"));
                assert_eq!(effect.caster, a);
            }
            for t in [1501,2001,3000,6000] {
                elapsed(&mut c, &mut s, t);
                assert_eq!(weaknesses(&c, e), initial, "maintenance must not reapply/refresh");
            }
            assert!(c.active_areas.is_empty() && !stunned(&c, e, 6000));
            let hp = c.units[&e].stats.current_health;
            hit(&mut c, b, e, 100, 6000);
            assert_eq!(c.units[&e].stats.current_health, hp - if equipped.is_empty() {100} else {115});
            elapsed(&mut c, &mut s, 6001);
            assert!(weaknesses(&c, e).is_empty());
            let hp = c.units[&e].stats.current_health;
            hit(&mut c, b, e, 100, 6001);
            assert_eq!(c.units[&e].stats.current_health, hp - 100);
            assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries, 0);
        }
    }
    #[test]
    fn composition_control_vulnerability_success_required_for_ground_and_direct_freeze() {
        for direct in [false,true] {
            for immune in [false,true] {
                let (mut c, mut s, a, e, b) = fixture(&[4], |d| with_vulnerability(d, direct));
                if immune { c.units.get_mut(&e).unwrap().target_traits.push(crate::game::battle::types::UnitTargetTrait::StunImmune); }
                cast(&mut c, a, 0);
                let start = if direct {1} else {1001};
                elapsed(&mut c, &mut s, start);
                assert_eq!(stunned(&c,e,start), !immune);
                let effects = weaknesses(&c,e);
                assert_eq!(effects.len(), usize::from(!immune));
                if let Some(effect) = effects.first() { assert_eq!(effect.expires_at_ms, start+5000); }
                elapsed(&mut c, &mut s, 3000);
                assert!(!stunned(&c,e,3000));
                let hp = c.units[&e].stats.current_health;
                hit(&mut c,b,e,100,3000);
                assert_eq!(c.units[&e].stats.current_health,hp-if immune {100} else {115});
                elapsed(&mut c, &mut s, start+5000);
                assert!(weaknesses(&c,e).is_empty());
                assert_eq!(c.runtime_fact_view().usage().outstanding_deliveries,0);
            }
        }
    }
    #[test]
    fn composition_control_vulnerability_survives_source_stop_exit_and_death_but_not_target_end() {
        for cause in ["stop","withdraw","death","target-death","terminal"] {
            let (mut c, mut s, a, e, b) = fixture(&[4], |d| with_vulnerability(d,false));
            cast(&mut c,a,0);
            elapsed(&mut c,&mut s,1001);
            let initial = weaknesses(&c,e);
            assert_eq!(initial.len(),1);
            match cause {
                "stop" => { cast(&mut c,a,1001); elapsed(&mut c,&mut s,1001); }
                "withdraw" => { c.apply_live_command(crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit { unit_id:a,time_ms:1001 }).unwrap(); }
                "death" => hit(&mut c,e,a,5000,1001),
                "target-death" => hit(&mut c,b,e,5000,1001),
                _ => {
                    std::sync::Arc::make_mut(&mut c.scenario).terminal_outcomes=BattleScenario::fixture_survival_outcomes(1100);
                    elapsed(&mut c,&mut s,1101);
                    c.finalize_battle_execution(&mut s).unwrap();
                }
            }
            if cause == "target-death" || cause == "terminal" { assert!(weaknesses(&c,e).is_empty()); }
            else {
                assert!(c.active_areas.is_empty());
                assert_eq!(weaknesses(&c,e),initial);
                elapsed(&mut c,&mut s,6001);
                assert!(weaknesses(&c,e).is_empty());
            }
        }
    }

    #[test]
    fn composition_area_effect_source_keeps_creation_generation_and_end_after_source_exit() {
        use crate::game::data::content_pack::ContentIdentity;
        for end in ["expiry", "stop", "withdraw", "target-death"] {
            let (mut c, mut s, a, e, b) = fixture(&[4], |d| with_vulnerability(d, false));
            observe_area(&mut c);
            let old = c.content_identity.clone();
            cast(&mut c, a, 0);
            elapsed(&mut c, &mut s, 1);
            // A producer may itself have an upstream authored callback. It must
            // not be mistaken for this generated effect's own EffectSite.
            c.active_areas.values_mut().next().unwrap().source_snapshot.fact_source.authored = Some(std::sync::Arc::new(crate::game::ability::authoring::EffectSource {
                source_file: "upstream".into(), field_path: "upstream.effect".into(),
                site: crate::game::ability::authoring::EffectSite {
                    action: crate::game::ability::authoring::ActionSite { definition_id: "upstream".into(), action_id: "action".into() }, effect_id: "effect".into(),
                }, origin_definition_id: "upstream".into(), origin: crate::game::ability::authoring::EffectOrigin::Original,
            }));
            c.content_identity = Some(std::sync::Arc::new(ContentIdentity { semantic_sha256: "different-generation".into(), ..old.as_ref().unwrap().as_ref().clone() }));
            elapsed(&mut c, &mut s, 1001);
            let started = c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.as_ref().unwrap().clone();
            let producer = started.area_source.as_ref().unwrap();
            assert_eq!(producer.generation, old);
            assert_eq!(producer.status_index, 1);
            assert_eq!(producer.skill_id.as_str(), "r11_aura");
            assert!(started.source.is_none() && started.authored.is_none());
            assert_eq!(started.time_ms, 1001);
            match end {
                "stop" => { cast(&mut c, a, 1001); elapsed(&mut c, &mut s, 1001); }
                "withdraw" => { c.apply_live_command(crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit { unit_id: a, time_ms: 1001 }).unwrap(); }
                "target-death" => hit(&mut c, b, e, 5000, 1001),
                _ => {}
            }
            elapsed(&mut c, &mut s, 6001);
            assert!(c.active_areas.is_empty() && weaknesses(&c,e).is_empty());
            let ended = c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.as_ref().unwrap();
            assert_eq!(serde_json::to_value(&ended.area_source).unwrap(), serde_json::to_value(&started.area_source).unwrap());
            assert_eq!(ended.area_source.as_ref().unwrap().generation, old);
            assert!(matches!(ended.change, crate::game::battle::core::authoring_observation::AppliedChange::Sustained { ended: Some(_), .. }));
            assert_eq!(ended.time_ms, if end == "target-death" {1001} else {6001});
        }
    }
    #[test]
    fn composition_area_effect_source_immunity_observation_invariance_and_cancel() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (initial, initial_state, a, e, _) = fixture(&[4], |d| with_vulnerability(d,false));
        let mut outcomes = Vec::new();
        for observe in [false,true] {
            let (mut c, mut s) = (initial.clone(), initial_state.clone());
            if observe { observe_area(&mut c); }
            cast(&mut c,a,0);
            let mut snapshots = Vec::new();
            for t in [1,1001,2002,6001] {
                elapsed(&mut c,&mut s,t);
                snapshots.push((c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(), serde_json::to_value(c.runtime_fact_view()).unwrap(), format!("{s:?}")));
            }
            outcomes.push(snapshots);
        }
        assert_eq!(outcomes[0], outcomes[1]);
        let (mut c, mut s) = (initial.clone(), initial_state.clone());
        observe_area(&mut c);
        c.units.get_mut(&e).unwrap().target_traits.push(crate::game::battle::types::UnitTargetTrait::StunImmune);
        cast(&mut c,a,0); elapsed(&mut c,&mut s,6001);
        assert!(c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.is_none());
        for time in [1001,6001] {
            let (mut c, mut s) = (initial.clone(), initial_state.clone());
            observe_area(&mut c); cast(&mut c,a,0); elapsed(&mut c,&mut s,time-1);
            let before=(c.transaction_debug_snapshot(),c.event_queue_debug_snapshot(),serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),format!("{s:?}"));
            let token=CancellationToken::default();
            let result=control::with_test_cancellation("sustained_observation_committed",1,||control::run(&ExecutionLimits::default(),&token,ExecutionPhase::TrialExecution,||c.step_battle_execution_until(&mut s,time)));
            assert!(token.is_cancelled() && result.is_err(), "{result:?}");
            assert_eq!(before,(c.transaction_debug_snapshot(),c.event_queue_debug_snapshot(),serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),format!("{s:?}")));
            elapsed(&mut c,&mut s,6001);
            assert!(weaknesses(&c,e).is_empty());
        }
    }

    #[test]
    fn composition_area_effect_two_casts_keep_each_fragment_after_caster_exit() {
        use crate::authoring::trace::{TraceAttempt, TraceLimits, TraceOptions};
        for delay in [0, 50] {
            let (mut initial, initial_state, _, enemy, _) = fixture(&[], |d| {
                area(d)["ground_status"]["statuses"][1]["vulnerability"] = json!({
                    "definition":"captured-ground-weakness", "percent":15, "duration_ms":5000
                });
            });
            let owned = |id| initial.units.values().find(|u| u.source_owned_uuid == Uuid::from_u128(id)).unwrap().instance_id;
            let a = owned(2);
            let b = owned(3);
            // These copy IDs are fixture inputs set before native admission.
            assert_eq!(initial.units[&a].fragment_source.as_ref().unwrap().copy_uuid, Uuid::from_u128(0xd200));
            assert_eq!(initial.units[&b].fragment_source.as_ref().unwrap().copy_uuid, Uuid::from_u128(0xd201));
            // The shared fixture's actor/ally selection does not assign these
            // owned IDs to fixed positions. Place both controlled inputs here.
            place(&mut initial, a, 2, 1);
            place(&mut initial, b, 4, 1);
            initial.units.get_mut(&a).unwrap().resonance_current = 100;
            initial.units.get_mut(&b).unwrap().resonance_current = 100;
            // Same-time ManualCastEnd events use unit-ID order, not request
            // insertion order. Determine this before either cast is admitted.
            assert!(b.as_bytes() < a.as_bytes());
            let a_sequence = if delay == 0 { 1 } else { 0 };
            let b_sequence = 1 - a_sequence;
            let mut native_outcome = None;
            for observe in [false, true] {
                let (mut c, mut s) = (initial.clone(), initial_state.clone());
                let options = TraceOptions { limits: Some(TraceLimits::default()) };
                if observe { c.enable_authoring_observation_with_trace(options); }
                let _trace = observe.then(|| TraceAttempt::enter(options).unwrap());
                assert_eq!(c.skill_cast_seq, 0);
                cast(&mut c, a, 0);
                elapsed(&mut c, &mut s, 0);
                cast(&mut c, b, delay);
                elapsed(&mut c, &mut s, 1001 + delay);
                let effects: Vec<_> = c.sustained_effect_snapshot().unwrap().active.into_iter()
                    .filter(|r| r.target == enemy && r.hit_status.as_ref().is_some_and(|p| p.definition == "captured-ground-weakness"))
                    .collect();
                if effects.len() != 2 {
                    eprintln!("area_binding_diagnostic {}", json!({
                        "delay":delay, "a":a, "b":b, "target":enemy, "effects":effects,
                        "areas":c.active_areas.values().map(|area| json!({
                            "caster":area.caster_instance_id,"cast":area.cast_seq,
                            "spawned_at":area.spawned_at_ms,"center":area.center,
                            "members":area.ground_status.as_ref().map(|ground| format!("{ground:?}"))
                        })).collect::<Vec<_>>(),
                        "events":serde_json::from_str::<Value>(&c.event_log.to_json_string().unwrap()).unwrap()
                    }));
                }
                assert_eq!(effects.len(), 2);
                for (actor, expiry) in [(a, 6001), (b, 6001 + delay)] {
                    let matching: Vec<_> = effects.iter().filter(|effect| effect.caster == actor).collect();
                    assert_eq!(matching.len(), 1);
                    assert_eq!(matching[0].expires_at_ms, expiry);
                    assert_eq!(matching[0].hit_status.as_ref().unwrap().kind,
                        crate::game::ability::sustained::HitStatusKind::DamageTaken { percent: 15 });
                }
                c.apply_live_command(crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit {
                    unit_id: a, time_ms: 1101,
                }).unwrap();
                elapsed(&mut c, &mut s, 6001 + delay);
                assert!(c.active_areas.is_empty());
                assert!(c.sustained_effect_snapshot().unwrap().active.iter()
                    .all(|r| r.hit_status.as_ref().is_none_or(|p| p.definition != "captured-ground-weakness")));
                if observe {
                    let trace = c.authoring_trace.view();
                    assert!(trace.gap.is_none());
                    let rows: Vec<Value> = std::str::from_utf8(trace.ndjson).unwrap().lines()
                        .map(|line| serde_json::from_str::<Value>(line).unwrap())
                        .filter(|r| r["area_source"]["definition"] == "captured-ground-weakness")
                        .collect();
                    assert_eq!(rows.len(), 4, "inspect every start and end: {rows:?}");
                    let mut area_ids = Vec::new();
                    for (actor, copy, sequence, start) in [(a, 0xd200, a_sequence, 1001), (b, 0xd201, b_sequence, 1001 + delay)] {
                        let matching: Vec<_> = rows.iter().filter(|r| r["source_unit"] == json!(actor)).collect();
                        assert_eq!(matching.len(), 2, "{rows:?}");
                        for (index, row) in matching.iter().enumerate() {
                            assert_eq!(row["time_ms"], start + if index == 0 { 0 } else { 5000 });
                            assert_eq!(row["target_unit"], json!(enemy));
                            assert_eq!(row["fragment"]["copy_uuid"], Uuid::from_u128(copy).to_string());
                            assert_eq!(row["fragment"]["skill_id"], "r11_aura");
                            assert_eq!(row["area_source"]["skill_id"], "r11_aura");
                            assert_eq!(row["area_source"]["cast_sequence"], sequence);
                            assert_eq!(row["area_source"]["status_index"], 1);
                            assert!(row["source"].is_null() && row["authored"].is_null());
                            assert_eq!(row["change"]["damage_taken_increase_percent"], 15);
                            assert_eq!(row["change"]["ended"].is_null(), index == 0);
                        }
                        assert_eq!(matching[0]["area_source"]["area_id"], matching[1]["area_source"]["area_id"]);
                        area_ids.push(matching[0]["area_source"]["area_id"].clone());
                    }
                    assert_ne!(area_ids[0], area_ids[1]);
                }
                let outcome = (c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(),
                    c.event_log.to_json_string().unwrap(), format!("{s:?}"));
                if let Some(expected) = &native_outcome { assert_eq!(expected, &outcome); }
                else { native_outcome = Some(outcome); }
            }
        }
    }

    #[test]
    fn conditional_damage_ground_query_uses_admitted_ramp_exit_stun_and_expiry() {
        let (mut c,mut s,a,e,_)=fixture(&[],|_|{});
        cast(&mut c,a,0);
        elapsed(&mut c,&mut s,1);
        assert!(!c.target_movement_impaired(e,1).unwrap());
        elapsed(&mut c,&mut s,501);
        assert!(c.target_movement_impaired(e,501).unwrap());
        place(&mut c,e,7,2);
        c.runtime_transaction(|c|c.reconcile_sustained(501)).unwrap();
        assert!(!c.target_movement_impaired(e,501).unwrap());
        place(&mut c,e,3,1);
        c.runtime_transaction(|c|c.reconcile_sustained(601)).unwrap();
        elapsed(&mut c,&mut s,1601);
        place(&mut c,e,7,2);
        c.runtime_transaction(|c|c.reconcile_sustained(1601)).unwrap();
        assert!(c.target_movement_impaired(e,1601).unwrap());
        elapsed(&mut c,&mut s,2002);
        assert!(!c.target_movement_impaired(e,2002).unwrap());
        let (mut c,mut s,a,e,_)=fixture(&[],|d| {
            area(d)["ground_status"]=json!({"statuses":[{"definition":"entry-slow","kind":{"Slow":{"percent":25}},"overlap":"Strongest","lifetime":"UntilExit"}],"rules":[{"trigger":"Enter","status":0,"action":"Apply"}]});
        });
        c.units.get_mut(&e).unwrap().target_traits.push(crate::game::battle::types::UnitTargetTrait::SlowImmune);
        cast(&mut c,a,0);
        elapsed(&mut c,&mut s,501);
        assert!(!c.target_movement_impaired(e,501).unwrap());
    }
}
