// Included beside the shared published-content fixture.
const PREPARED_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/prepared_state.skill.yaml"
));
fn prepared_program(doc: &mut Value) -> &mut Value {
    &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["PreparedState"]
        ["program"]
}
fn prepared_fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
    prepared_fixture_weapon(edit, |_| {})
}
fn prepared_fixture_weapon(
    edit: impl FnOnce(&mut Value),
    weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
) -> Fixture {
    let (mut c, s, a, e, b) = fixture_weapon(
        |doc| {
            *doc = parse_yaml(PREPARED_YAML).unwrap();
            doc["skills"][0]["id"] = json!("r11_aura");
            edit(doc);
        },
        weapon,
    );
    c.units.get_mut(&a).unwrap().basic_attack.range_policy =
        crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
    (c, s, a, e, b)
}
fn preparations(c: &BattleCore) -> Vec<crate::game::ability::sustained::PreparedBasicView> {
    c.sustained_effect_snapshot().unwrap().prepared_basics
}
fn free_preparation(doc: &mut Value) {
    doc["skills"][0]["admission"]["requirement"] = json!("None");
    doc["skills"][0]["admission"]["cost"] = json!("Zero");
}
fn prepared_projectile(c: &mut BattleCore, a: UnitInstanceId) {
    use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
    c.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
        speed_units_per_ms: 2000,
        hit_policy: ProjectileHitPolicy::TargetLocked,
        allow_targetless_cast: false,
        max_range_tiles: None,
        max_lifetime_ms: None,
        max_kills: None,
        max_pierces: None,
        collision: Default::default(),
    };
}
#[test]
fn composition_prepared_state_rejects_invalid_lifetimes_context_and_targets() {
    let doc = parse_yaml(PREPARED_YAML).unwrap();
    let artifact = compile_json(&doc.to_string()).unwrap();
    Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    for case in 0..8 {
        let mut d = doc.clone();
        match case {
            0 => prepared_program(&mut d)["owns_use"] = json!(true),
            1 => prepared_program(&mut d)["end_on_source_exit"] = json!(false),
            2 => prepared_program(&mut d)["replacement"] = json!("Stack"),
            3 => prepared_program(&mut d)["handlers"][0]["event"] = json!("Expired"),
            4 => {
                prepared_program(&mut d)["handlers"][0]["body"][1]["ApplyOwnerState"]["duration"]
                    ["root"] = json!({"Read":{"Time":"RemainingTime"}})
            }
            5 => {
                d["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                    ["PreparedState"]["denominator"] = json!(0)
            }
            6 => {
                d["definitions"][0]["body"]["Action"]["target"]["Literal"]["Target"] =
                    json!("CastTarget")
            }
            _ => {
                prepared_program(&mut d)["handlers"][0]["body"] = json!([{"Resources":{"operations":[{"resource":"missing","owner":"SourceFragment","operation":{"Grant":1}}]}}])
            }
        }
        let err = compile_json(&d.to_string()).unwrap_err();
        assert_ne!(
            err.code,
            crate::authoring::composition::DiagnosticCode::Parse,
            "case {case}: {err}"
        );
        if case == 7 {
            assert!(
                err.source.definition.contains(
                    "PreparedState.program.handlers[0].body[0].Resources.operations[0].resource"
                ),
                "{err:?}"
            );
        }
    }
}
#[test]
fn composition_prepared_state_waits_consumes_once_and_reward_changes_future_attack() {
    let mut outcomes = Vec::new();
    for observe in [false, true] {
        let (mut c, mut s, a, e, b) = prepared_fixture(|_| {});
        if observe {
            c.enable_authoring_observation_with_options(Default::default(), true);
        }
        let handle = c
            .request_skill_action(0, a, "r11_aura".into(), None)
            .unwrap();
        c.step_battle_execution_until(&mut s, 5001).unwrap();
        assert_eq!(preparations(&c).len(), 1);
        assert!(!c.resonance_use_has_obligations(a));
        assert_eq!(c.query_action(&handle).unwrap().facts.resource_spent, 100);
        assert_eq!(c.units[&a].stats.attack, 100);
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        start(&mut c, a, e, 5100, true);
        c.step_battle_execution_until(&mut s, 5101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert!(preparations(&c).is_empty());
        assert_eq!(c.units[&a].stats.attack, 125);
        assert_eq!(c.units[&a].stats.current_health, 500);
        assert!(c
            .query_action(&handle)
            .unwrap()
            .effects_settled_at_ms
            .is_some());
        let damage = c.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(damage.modifier_sources.len(), 1);
        assert!(damage.modifier_sources[0].fragment.is_some());
        assert_eq!(damage.modifier_sources[0].action.as_ref(), Some(&handle));
        start(&mut c, a, b, 5200, true);
        c.step_battle_execution_until(&mut s, 5201).unwrap();
        assert_eq!(c.units[&b].stats.current_health, 875);
        c.step_battle_execution_until(&mut s, 7102).unwrap();
        assert!(rewards(&c).is_empty());
        assert_eq!(c.units[&a].stats.attack, 100);
        outcomes.push(serde_json::to_value(c.runtime_fact_view()).unwrap());
    }
    assert_eq!(outcomes[0], outcomes[1]);
}
#[test]
fn composition_prepared_state_existing_windup_and_triggered_attacks_do_not_consume() {
    for old_windup in [false, true] {
        let (mut c, mut s, a, e, _) = prepared_fixture(|_| {});
        c.units.get_mut(&a).unwrap().basic_attack.windup_ms = 200;
        if old_windup {
            start(&mut c, a, e, 0, true);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            cast(&mut c, a, 50);
        } else {
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            start(&mut c, a, e, 50, false);
        }
        c.step_battle_execution_until(&mut s, 251).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 900);
        assert!(preparations(&c)[0].bound_action.is_none());
        start(&mut c, a, e, 300, true);
        c.step_battle_execution_until(&mut s, 501).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 650);
        assert!(preparations(&c).is_empty());
    }
}
#[test]
fn composition_prepared_state_cancel_restores_unless_superseded() {
    use crate::game::battle::buffs::BuffId;
    for recast in [false, true] {
        let (mut c, mut s, a, e, _) = prepared_fixture(free_preparation);
        c.units.get_mut(&a).unwrap().basic_attack.windup_ms = 200;
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let old = preparations(&c)[0].activation;
        start(&mut c, a, e, 10, true);
        c.step_battle_execution_until(&mut s, 11).unwrap();
        if recast {
            cast(&mut c, a, 30);
            c.step_battle_execution_until(&mut s, 31).unwrap();
        }
        c.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 50,
                caster_instance_id: e,
                target_instance_id: a,
                buff_id: BuffId::from_name("r11_stun"),
                duration_ms: 100,
                cause: Default::default(),
            },
            50,
        )
        .unwrap();
        let view = preparations(&c);
        assert_eq!(view.len(), 1);
        assert_eq!(view[0].activation == old, !recast);
        assert!(view[0].bound_action.is_none());
        c.step_battle_execution_until(&mut s, 211).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 1000);
        start(&mut c, a, e, 250, true);
        c.step_battle_execution_until(&mut s, 451).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 750);
    }
}
#[test]
fn composition_prepared_state_projectile_keeps_bound_program_after_recast_and_source_exit() {
    use crate::game::battle::core::sim::BattleLiveCommand;
    for exit in [false, true] {
        let (mut c, mut s, a, e, _) = prepared_fixture(free_preparation);
        prepared_projectile(&mut c, a);
        c.units.get_mut(&a).unwrap().basic_attack.windup_ms = 200;
        c.units.get_mut(&e).unwrap().stats.current_health = 400;
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        start(&mut c, a, e, 10, true);
        c.step_battle_execution_until(&mut s, 11).unwrap();
        c.units.get_mut(&a).unwrap().stats.attack = 200;
        c.step_battle_execution_until(&mut s, 211).unwrap();
        assert!(preparations(&c)[0].released);
        cast(&mut c, a, 250);
        c.step_battle_execution_until(&mut s, 251).unwrap();
        assert_eq!(preparations(&c).len(), 2);
        if exit {
            c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 300,
            })
            .unwrap();
        }
        c.step_battle_execution_until(&mut s, 1500).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(rewards(&c).len(), if exit { 0 } else { 1 });
        assert_eq!(preparations(&c).len(), if exit { 0 } else { 1 });
        if !exit {
            assert_eq!(preparations(&c)[0].state.as_ref().unwrap().events, 0);
        }
    }
}
#[test]
fn composition_prepared_state_base_kill_only_and_data_only_second_program() {
    use crate::game::stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect};
    for (hp, reward) in [(240, true), (300, false)] {
        let (mut c, mut s, a, e, _) = prepared_fixture_weapon(
            |doc| {
                let p = prepared_program(doc);
                p["handlers"][0]["body"].as_array_mut().unwrap().push(json!({"If":{
                "condition":{"Compare":{"left":{"moment":"Live","root":{"Read":{"StateField":{"name":"kills","unit":"Count"}}}},"comparison":"AtLeast","right":{"moment":"Live","root":{"Const":{"value":1,"unit":"Count"}}}}},
                "then_ops":[{"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":17,"unit":"Magnitude"}}}}}],"else_ops":[]}}));
            },
            |w| {
                w.triggered_effects.insert(
                    TriggerType::OnAttack,
                    vec![TriggeredEffect::targeted(
                        TriggerEffectTarget::CounterpartUnit,
                        Effect::BonusDamage {
                            flat: 100,
                            percent: 0,
                            damage_type: DamageType::True,
                        },
                    )],
                );
            },
        );
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = hp;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(
            c.units[&a].stats.current_health,
            if reward { 517 } else { 500 }
        );
        assert_eq!(c.units[&a].stats.attack, if reward { 125 } else { 100 });
    }
}
#[test]
fn composition_prepared_state_cancellation_restores_binding_damage_and_rewards() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for checkpoint in [
        "prepared_basic_bound",
        "prepared_basic_released",
        "state_reaction_applied",
        "prepared_basic_ended",
    ] {
        let (mut c, mut s, a, e, _) = prepared_fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(
            &mut s,
            if checkpoint == "prepared_basic_bound" {
                99
            } else {
                100
            },
        )
        .unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(checkpoint, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 101),
            )
        });
        assert!(
            result.is_err() && token.is_cancelled(),
            "{checkpoint}: {result:?}"
        );
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}")
            ),
            "{checkpoint}"
        );
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(rewards(&c).len(), 1);
        assert!(preparations(&c).is_empty());
    }
}
#[test]
fn composition_prepared_state_area_primary_dedup_and_capture_are_authored() {
    use crate::game::{
        ability::{DeliveryDef, SkillAreaAnchorSource},
        battle::tile_range::TileRangePattern,
    };
    for (primary, once, primary_hp, secondary_hp, count, end) in [
        (false, true, 240, 240, 1, false),
        (false, false, 240, 240, 2, false),
        (true, false, 240, 240, 1, false),
        (true, false, 1000, 240, 0, false),
        (false, false, 240, 240, 1, true),
        (false, true, 240, 1000, 1, false),
    ] {
        let (mut c, mut s, a, e, b) = prepared_fixture(|doc| {
            let p = prepared_program(doc);
            p["handlers"][0]["event"] =
                json!({"BasicAttackKill":{"primary_only":primary,"once_per_action":once}});
            p["handlers"][0]["body"].as_array_mut().unwrap().push(json!({"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":17,"unit":"Magnitude"}}}}}));
            if end {
                p["handlers"][0]["body"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("End"));
            }
        });
        place(&mut c, b, 4, 2);
        c.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
            footprint: TileRangePattern {
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                include_anchor_tile: true,
            },
            area: crate::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                tile_origin: crate::game::ability::SkillTileAreaOrigin::Anchor,
                ..Default::default()
            },
        };
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = primary_hp;
        c.units.get_mut(&b).unwrap().stats.current_health = secondary_hp;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        if secondary_hp == 240 {
            assert!(c.units[&b].lifecycle.is_dead());
        } else {
            assert_eq!(c.units[&b].stats.current_health, 750);
        }
        assert_eq!(c.units[&a].stats.current_health, 500 + 17 * count);
        assert_eq!(rewards(&c).len(), usize::from(count > 0));
        assert!(preparations(&c).is_empty());
        if primary_hp == 1000 {
            assert_eq!(c.units[&e].stats.current_health, 750);
        }
    }
}
#[test]
fn composition_prepared_state_different_families_queue_one_per_attack() {
    let (mut c, mut s, a, e, _) = prepared_fixture(|doc| {
        let mut second = doc["values"][0]["value"]["Effects"][0].clone();
        second["id"] = json!("second");
        second["op"]["Sustained"]["definition"]["PreparedState"]["numerator"] = json!(6);
        second["op"]["Sustained"]["definition"]["PreparedState"]["program"]["id"] =
            json!("second-family");
        doc["values"][0]["value"]["Effects"]
            .as_array_mut()
            .unwrap()
            .push(second);
    });
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    assert_eq!(preparations(&c).len(), 2);
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 101).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 750);
    assert_eq!(
        preparations(&c)[0].state.as_ref().unwrap().definition,
        "second-family"
    );
    start(&mut c, a, e, 200, true);
    c.step_battle_execution_until(&mut s, 201).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 450);
    assert!(preparations(&c).is_empty());
}
#[test]
fn composition_prepared_state_invalid_reward_rolls_back_death_and_latches_fault() {
    let (mut c, mut s, a, e, _) = prepared_fixture(|doc| {
        prepared_program(doc)["handlers"][0]["body"][1]["ApplyOwnerState"]["duration"]["root"]
            ["Const"]["value"] = json!(0);
    });
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 100).unwrap();
    let before = preparations(&c);
    let result = c.step_battle_execution_until(&mut s, 101);
    assert!(
        matches!(result, Err(GameError::ValueExpressionFault(_))),
        "{result:?}"
    );
    assert_eq!(preparations(&c), before);
    assert_eq!(c.units[&e].stats.current_health, 240);
    assert!(c.units[&e].is_active());
    assert!(rewards(&c).is_empty());
    assert!(c.step_battle_execution_until(&mut s, 201).is_err());
}
#[test]
fn composition_prepared_state_resources_use_canonical_scope_and_atomic_reaction() {
    use crate::game::resource::ResourceOwner;
    for consume in [3, 9] {
        let (mut c, mut s, a, e, _) = prepared_fixture(|doc| {
            doc["resources"] = json!([{"id":"kill_meter","owner":"FragmentCopy","lifetime":"Attempt","initial":0,"cap":20}]);
            prepared_program(doc)["handlers"][0]["body"].as_array_mut().unwrap().push(json!({"Resources":{"operations":[
                {"resource":"kill_meter","owner":"SourceFragment","operation":{"Grant":7}},
                {"resource":"kill_meter","owner":"SourceFragment","operation":{"Consume":consume}}
            ]}}));
        });
        let owners = c
            .units
            .values()
            .filter_map(|u| {
                u.fragment_source
                    .as_ref()
                    .map(|f| ResourceOwner::FragmentCopy(f.copy_uuid))
            })
            .collect();
        c.mount_run_resources(&Default::default(), &owners).unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        let before = preparations(&c);
        let result = c.step_battle_execution_until(&mut s, 101);
        if consume == 3 {
            result.unwrap();
            assert_eq!(resource_balances(&c), vec![4]);
            assert!(c.units[&e].lifecycle.is_dead());
            assert_eq!(rewards(&c).len(), 1);
        } else {
            assert!(result.is_err());
            assert_eq!(preparations(&c), before);
            assert!(resource_balances(&c).is_empty());
            assert_eq!(c.units[&e].stats.current_health, 240);
            assert!(rewards(&c).is_empty());
        }
    }
}
#[test]
fn composition_prepared_state_released_miss_and_terminal_cleanup_do_not_refund() {
    use crate::game::battle::scenario::BattleScenario;
    let (mut c, mut s, a, e, b) = prepared_fixture(|_| {});
    prepared_projectile(&mut c, a);
    let grant = c
        .request_skill_action(0, a, "r11_aura".into(), None)
        .unwrap();
    c.step_battle_execution_until(&mut s, 1).unwrap();
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 101).unwrap();
    assert!(preparations(&c)[0].released);
    hit(&mut c, b, e, 1000, 200);
    c.step_battle_execution_until(&mut s, 1500).unwrap();
    assert!(preparations(&c).is_empty() && rewards(&c).is_empty());
    assert_eq!(c.query_action(&grant).unwrap().facts.resource_spent, 100);
    assert_eq!(c.units[&a].stats.attack, 100);
    let (mut c, mut s, a, _, _) = prepared_fixture(|_| {});
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        BattleScenario::fixture_survival_outcomes(500);
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 501).unwrap();
    assert!(s.is_finished());
    let result = c.finalize_battle_execution(&mut s).unwrap();
    assert!(preparations(&c).is_empty());
    assert_eq!(
        result.runtime_facts.view().usage().outstanding_deliveries,
        0
    );
}
