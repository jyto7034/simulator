mod composition_target_state {
    use super::compiled_action_fixture::{cast, fixture_content_with_weapon, hit};
    use super::*;
    use crate::{
        authoring::{
            composition::{compile_json, parse_yaml, Artifact},
            node_graph::NativeGraphContent,
        },
        game::{
            ability::sustained::SustainedEffectView,
            battle::core::sim::{BattleExecutionState, BattleLiveCommand},
        },
    };
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/target_state.skill.yaml"
    ));
    fn grant(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["TargetState"]
    }
    fn target_ally(doc: &mut Value) {
        doc["skills"][0]["kind"] = json!("Targeted");
        doc["skills"][0]["cast_targeting"]["Explicit"] = json!({"selection":"player_unit","target":"AllySingle","reach":"global","air_capable":true});
        doc["skills"][0]["body"]["Call"]["arguments"][0]["value"]["Literal"]["Target"] =
            json!("CastTarget");
    }
    fn fixture(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let a = compile_json(&doc.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let (mut c, s, a, e, b) = fixture_content_with_weapon(
            &[],
            &[],
            NativeGraphContent {
                skills: a.load_native().unwrap(),
                equipment: vec![],
                mark_weapons: vec![],
            },
            |_| {},
        );
        for id in [a, b] {
            c.units.get_mut(&id).unwrap().stats.attack = 100;
            c.units.get_mut(&id).unwrap().stats.defense = 100;
        }
        c.units.get_mut(&e).unwrap().stats.current_health = 1000;
        c.units.get_mut(&e).unwrap().stats.max_health = 1000;
        (c, s, a, e, b)
    }
    fn active(c: &BattleCore) -> Vec<SustainedEffectView> {
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|r| r.target_state.is_some())
            .collect()
    }
    fn cast_at(c: &mut BattleCore, a: UnitInstanceId, b: UnitInstanceId, now: u64) {
        c.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: b,
            }),
            time_ms: now,
        })
        .unwrap();
    }
    #[test]
    fn composition_target_state_validates_used_and_unused_values_and_preserves_source() {
        let doc = parse_yaml(YAML).unwrap();
        let a = compile_json(&doc.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let skill = &a.load_native().unwrap().skills[0];
        assert!(
            matches!(&skill.use_lifetime,crate::game::ability::SkillUseLifetime::ActionAndEffects{effects} if effects.len()==1)
        );
        for unused in [false, true] {
            for (case, expected) in [(0, "positive duration"), (1, "nonempty"), (2, "nonempty")] {
                let mut d = doc.clone();
                match case {
                    0 => grant(&mut d)["duration_ms"] = json!(0),
                    1 => grant(&mut d)["profile"]["definition"] = json!(""),
                    _ => grant(&mut d)["profile"]["attack_percent"] = json!(10001),
                }
                if unused {
                    d["skills"][0]["body"]["Call"]["arguments"][1]["value"] =
                        json!({"Literal":{"Effects":[]}});
                }
                let diagnostic = compile_json(&d.to_string()).unwrap_err();
                let error = diagnostic.to_string();
                assert!(
                    error.contains(expected) && error.contains("TargetState"),
                    "{unused}/{case}: {error}"
                );
                assert!(
                    diagnostic.source.definition.starts_with(
                        "values[0].value.Effects[0].op.Sustained.definition.TargetState"
                    ),
                    "{diagnostic:?}"
                );
                if !unused {
                    assert!(
                        diagnostic
                            .source
                            .calls
                            .iter()
                            .any(|p| p.starts_with("skills[0]")),
                        "{diagnostic:?}"
                    );
                }
            }
        }
        for direct_action in [false, true] {
            let mut d = doc.clone();
            grant(&mut d)["duration_ms"] = json!(0);
            let literal = json!({"Literal": d["values"][0]["value"]});
            d["values"] = json!([]);
            let prefix = if direct_action {
                d["skills"][0]["body"] = d["definitions"][0]["body"].clone();
                d["skills"][0]["body"]["Action"]["target"] =
                    json!({"Literal":{"Target":"SelfUnit"}});
                d["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"] = literal;
                "skills[0].body.effects[0].effects.Literal"
            } else {
                d["skills"][0]["body"]["Call"]["arguments"][1]["value"] = literal;
                "skills[0].body.arguments[1].value.Literal"
            };
            let error = compile_json(&d.to_string()).unwrap_err();
            assert_eq!(error.source.definition, format!("{prefix}.Effects[0].op.Sustained.definition.TargetState"));
            assert!(error.reason.contains("positive duration"));
        }
    }
    #[test]
    fn composition_target_state_self_grant_preserves_weapon_block_and_expires_without_heal() {
        for (observed, trace) in [(false, false), (true, false), (true, true)] {
            let (mut c, mut s, a, e, _) = fixture(|_| {});
            if observed {
                c.enable_authoring_observation_with_options(crate::authoring::trace::TraceOptions { limits: trace.then(Default::default) }, true);
                c.bind_authoring_observation(
                    crate::game::battle::core::authoring_observation::ObservationKey {
                        entry_ordinal: 1,
                        battle_uuid: Uuid::from_u128(0x20a),
                    },
                )
                .unwrap();
            }
            let block = c.units[&a].block_capacity;
            c.units.get_mut(&a).unwrap().basic_attack.range_policy =
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
            c.units.get_mut(&a).unwrap().basic_attack.windup_ms = 1;
            let profile = format!("{:?}", c.units[&a].basic_attack);
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(
                (c.units[&a].stats.attack, c.units[&a].stats.defense),
                (125, 150)
            );
            assert_eq!(c.sustained_basic_interval(a, 1000), 667);
            assert_eq!(c.units[&a].block_capacity, block);
            assert_eq!(format!("{:?}", c.units[&a].basic_attack), profile);
            assert!(c.resonance_use_has_obligations(a));
            assert!(!c.channel_active(a));
            let r = &active(&c)[0];
            assert!(r.authored_site.is_some() && r.fragment_copy_uuid.is_some());
            if observed {
                let observation = c.authoring_observation().unwrap().unwrap();
                let latest = observation.latest_sourced_sustained.as_ref().unwrap();
                let value = serde_json::to_value(latest).unwrap();
                assert_eq!(value["change"]["before"]["attack"], 100);
                assert_eq!(value["change"]["after"]["attack"], 125);
                assert!(value["change"]["ended"].is_null());
                assert_eq!(value["fragment"]["skill_id"], "r11_aura");
                assert_eq!(latest.source.as_ref().unwrap().authored.site, r.authored_site.clone().unwrap());
                assert!(observation.latest_nonzero_sourced_hp.is_none());
            }
            let observed_source = c.authoring_observation().unwrap()
                .and_then(|o| o.latest_sourced_sustained.as_ref()).and_then(|w| w.source.clone());
            c.units.get_mut(&a).unwrap().next_basic_attack_ms = 100;
            c.event_queue.push(BattleEvent::AttackStart {
                time_ms: 100,
                attacker_instance_id: a,
                target_instance_id: Some(e),
                schedule_next: true,
                cause: Default::default(),
            });
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 875);
            hit(&mut c, e, a, 5000, 200);
            assert_eq!(c.units[&a].stats.current_health, 1);
            c.step_battle_execution_until(&mut s, 1501).unwrap();
            assert!(active(&c).is_empty());
            assert_eq!(
                (
                    c.units[&a].stats.attack,
                    c.units[&a].stats.defense,
                    c.units[&a].stats.current_health
                ),
                (100, 100, 1)
            );
            if observed {
                let observation = c.authoring_observation().unwrap().unwrap();
                let latest = observation.latest_sourced_sustained.as_ref().unwrap();
                let value = serde_json::to_value(latest).unwrap();
                assert_eq!(value["change"]["before"]["attack"], 125);
                assert_eq!(value["change"]["after"]["attack"], 100);
                assert_eq!(value["change"]["ended"], "Expired");
                assert_eq!(latest.target_unit, a);
                assert_eq!(latest.time_ms, 1501);
                assert_eq!(latest.source, observed_source);
                assert_eq!(latest.change.hp_delta(), 0);
            }
            assert!(!c.resonance_use_has_obligations(a));
            hit(&mut c, e, a, 1, 1502);
            assert!(c.units[&a].lifecycle.is_dead());
        }
    }
    #[test]
    fn composition_target_state_damage_floor_does_not_block_health_cost() {
        let (mut c, mut s, a, e, _) = fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        hit(&mut c, e, a, 5000, 2);
        c.runtime_transaction(|c| {
            c.process_commands(
                vec![crate::game::battle::damage::BattleCommand::PayHealthCost {
                    caster_id: a,
                    amount: 1,
                    allow_lethal: true,
                }],
                3,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
        assert!(c.units[&a].lifecycle.is_dead());
        assert!(active(&c).is_empty());
    }
    #[test]
    fn composition_target_state_ally_grant_composes_with_independent_reactive_heal() {
        for end in [false, true] {
            let (mut c, mut s, a, e, b) = fixture(|doc| {
                target_ally(doc);
                grant(doc)["profile"]["end_on_source_exit"] = json!(end);
                let aid = parse_yaml(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/skill_composition/state_events.skill.yaml"
                )))
                .unwrap();
                let mut effect = aid["values"][0]["value"]["Effects"][0].clone();
                effect["op"]["Sustained"]["definition"]["State"]["program"]["owns_use"] =
                    json!(false);
                doc["values"][0]["value"]["Effects"]
                    .as_array_mut()
                    .unwrap()
                    .push(effect);
            });
            cast_at(&mut c, a, b, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(c.units[&a].stats.attack, 100);
            assert_eq!(c.units[&b].stats.attack, 125);
            c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 100,
            })
            .unwrap();
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(c.units[&b].stats.attack, if end { 100 } else { 125 });
            // HpLost subscriptions require a real action identity. A helper's
            // direct damage command is not an authored enemy attack.
            let enemy = c.units.get_mut(&e).unwrap();
            enemy.stats.attack = 5000;
            enemy.basic_attack.range_policy =
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
            enemy.basic_attack.windup_ms = 1;
            enemy.basic_attack.range_role =
                crate::game::data::equipment_data::WeaponRangeRole::Ranged;
            enemy.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
            enemy.next_basic_attack_ms = 200;
            c.event_queue.push(BattleEvent::AttackStart {
                time_ms: 200,
                attacker_instance_id: e,
                target_instance_id: Some(b),
                schedule_next: true,
                cause: Default::default(),
            });
            c.step_battle_execution_until(&mut s, 201).unwrap();
            assert_eq!(c.units[&b].stats.current_health, if end { 0 } else { 201 });
            if !end {
                assert_eq!(active(&c)[0].caster, a);
            }
            c.step_battle_execution_until(&mut s, 1501).unwrap();
            assert!(active(&c).is_empty());
            assert_eq!(c.units[&b].stats.attack, 100);
        }
    }
    #[test]
    fn composition_target_state_observation_is_bounded_and_shared_with_enemy_status() {
        for status in [false, true] {
            let (mut c, mut execution, actor, enemy, _) = fixture(|d| {
                grant(d)["duration_ms"] = json!(500);
                if status {
                    d["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"] = json!({
                        "HitStatus": {"duration_ms": 500, "profile": {"definition":"test-stun", "kind":"Stun"}}
                    });
                    d["skills"][0]["cast_targeting"]["Explicit"] = json!({
                        "selection":"automatic_unit", "target":{"EnemySingle":{"rule":"RouteProgress"}}, "reach":"global", "air_capable":true
                    });
                    d["skills"][0]["body"]["Call"]["arguments"][0]["value"] = json!({"Literal":{"Target":"CastTarget"}});
                }
            });
            c.enable_authoring_observation();
            c.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1, battle_uuid: Uuid::from_u128(0x21b),
            }).unwrap();
            let mut extent = None;
            let mut now = 0;
            for _ in 0..8 {
                c.units.get_mut(&actor).unwrap().resonance_current = 100;
                cast(&mut c, actor, now);
                c.step_battle_execution_until(&mut execution, now + 10).unwrap();
                let start = c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.as_ref().unwrap().clone();
                assert_eq!(start.target_unit, if status {enemy} else {actor});
                assert!(matches!(start.change, crate::game::battle::core::authoring_observation::AppliedChange::Sustained {ended:None,..}));
                let deadline = match start.change {
                    crate::game::battle::core::authoring_observation::AppliedChange::Sustained {expires_at_ms,..} => expires_at_ms,
                    _ => unreachable!(),
                };
                assert!(start.time_ms >= now && start.time_ms <= now + 10);
                assert_eq!(deadline, start.time_ms + 500);
                c.step_battle_execution_until(&mut execution, deadline).unwrap();
                now = deadline + 1;
                let observation = c.authoring_observation().unwrap().unwrap();
                let end = observation.latest_sourced_sustained.as_ref().unwrap();
                assert_eq!(start.source, end.source);
                assert!(end.ordinal > start.ordinal, "status={status}, now={now}, start={start:?}, end={end:?}, sustained={:?}, execution={execution:?}", c.sustained_effect_snapshot().unwrap());
                assert!(matches!(end.change, crate::game::battle::core::authoring_observation::AppliedChange::Sustained {ended:Some(crate::game::ability::sustained::SustainedEndReason::Expired),..}));
                let actual = observation.retained_extent().unwrap();
                if let Some(expected) = extent { assert_eq!(actual, expected); } else { extent = Some(actual); }
                assert!(observation.latest_nonzero_sourced_hp.is_none());
            }
            assert_eq!(c.sustained_effect_snapshot().unwrap().ended, 8);
        }
    }

    #[test]
    fn composition_target_state_observation_retained_refusal_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let (mut core, mut execution, actor, _, _) = fixture(|_| {});
        core.enable_authoring_observation();
        core.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey {
            entry_ordinal: 1, battle_uuid: Uuid::from_u128(0x21c),
        }).unwrap();
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut execution, 1500).unwrap();
        let mut probe = core.clone();
        let mut probe_execution = execution.clone();
        let (result, admissions) = native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut probe_execution, 1501));
        result.unwrap();
        let required = admissions.iter().find(|(site,_,_)| *site=="sustained_observation_source").unwrap().2;
        core.runtime_facts.limits.max_retained_bytes = required - 1;
        let snapshot = |c: &BattleCore, s: &BattleExecutionState| (
            c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
            serde_json::to_value(c.authoring_observation().unwrap()).unwrap(), format!("{s:?}"),
        );
        let before = snapshot(&core, &execution);
        let error = core.step_battle_execution_until(&mut execution, 1501).unwrap_err();
        assert!(matches!(error, GameError::BattleRuntimeFault(native::BattleRuntimeFault {
            code:native::RuntimeFaultCode::RetainedByteLimit, site:"sustained_observation_source",..
        })),"{error:?}");
        assert_eq!(snapshot(&core,&execution),before);
        core.runtime_facts.limits = Default::default();
        core.step_battle_execution_until(&mut execution,1501).unwrap();
        assert_eq!(core.units[&actor].stats.attack,100);
        assert!(core.authoring_observation().unwrap().unwrap().latest_sourced_sustained.is_some());
    }

    #[test]
    fn composition_target_state_apply_and_expiry_cancel_restore_real_authored_flow() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, expiry) in [
            ("target_state_definition", false),
            ("sustained_started", false),
            ("sustained_ended", true),
            ("sustained_observation_committed", false),
            ("sustained_observation_committed", true),
        ] {
            let (mut c, mut s, a, _, _) = fixture(|_| {});
            c.enable_authoring_observation();
            c.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1, battle_uuid: Uuid::from_u128(0x21a),
            }).unwrap();
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 0).unwrap();
            let time = if expiry {
                c.step_battle_execution_until(&mut s, 1500).unwrap();
                1501
            } else {
                1
            };
            let probe = |c: &BattleCore, s: &BattleExecutionState| {
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.runtime_fact_view()).unwrap(),
                    format!("{s:?}"),
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                )
            };
            let before = probe(&c, &s);
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
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(probe(&c, &s), before);
            c.step_battle_execution_until(&mut s, time).unwrap();
            assert_eq!(active(&c).len(), usize::from(time == 1));
        }
    }
}
