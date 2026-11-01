const HEADSHOT_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/scarecrow_headshot.skill.yaml"
));
fn headshot_fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
    headshot_fixture_weapon(edit, |_| {})
}
fn headshot_fixture_weapon(
    edit: impl FnOnce(&mut Value),
    weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
) -> Fixture {
    headshot_fixture_weapon_setup(edit, weapon, |_| {})
}
fn headshot_fixture_weapon_setup(
    edit: impl FnOnce(&mut Value),
    weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
    configure: impl FnOnce(&mut BattleScenario),
) -> Fixture {
    let (mut core, state, a, e, b) = fixture_weapon_with_setup(
        |doc| {
            *doc = parse_yaml(HEADSHOT_YAML).unwrap();
            doc["skills"][0]["id"] = json!("r11_aura");
            edit(doc);
        },
        |w| {
            w.weapon_profile = Some(
                ron::de::from_str(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/skill_composition/scarecrow.weapon.ron"
                )))
                .unwrap(),
            );
            weapon(w);
        },
        configure,
    );
    // The shared fixture starts battle at t=0. Discard its already-admitted
    // initial windup so this scenario owns the next ordinary attack.
    for u in core.units.values_mut() {
        u.stats.attack_interval_ms = 100_000;
        u.next_basic_attack_ms = 100_000;
        u.basic_attack_epoch += 1;
        u.action_locks = Default::default();
        u.basic_attack_windup_until_ms = None;
        u.pending_basic_attack = false;
    }
    core.units.get_mut(&a).unwrap().stats.attack = 100;
    (core, state, a, e, b)
}
fn headshot_magazine(c: &BattleCore, actor: UnitInstanceId) -> (u32, Option<u64>) {
    c.equipment_attack_state
        .magazines
        .values()
        .find(|m| m.owner == actor)
        .map_or((1, None), |m| (m.remaining, m.reload_at))
}
#[test]
fn composition_headshot_resonance_schema_roundtrip_and_source_diagnostics() {
    let doc = parse_yaml(HEADSHOT_YAML).unwrap();
    let artifact = compile_json(&doc.to_string()).unwrap();
    let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    loaded.load_native().unwrap();
    let m = loaded.equipment_attacks()[0]
        .program
        .magazine
        .as_ref()
        .unwrap();
    assert_eq!((m.capacity, m.reload_ms, m.normal_percent), (1, 1500, 100));
    assert!(m.reward_family.is_none());
    let mut invalid = doc;
    prepared_program(&mut invalid)["handlers"][0]["body"][0]["GrantOwnerResonance"]["amount"]
        ["root"]["Const"]["unit"] = json!("Count");
    let error = compile_json(&invalid.to_string()).unwrap_err();
    assert!(format!("{error:?}").contains("Magnitude"), "{error:?}");
    assert!(
        error.source.definition.contains("PreparedState")
            && error.reason.contains("handlers[0].body[0]"),
        "{error:?}"
    );
}
#[test]
fn composition_headshot_resonance_single_round_release_reload_and_normal_followup() {
    let (mut c, mut s, a, e, b) = headshot_fixture(|_| {});
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    assert_eq!(c.units[&a].resonance_current, 0);
    assert_eq!(headshot_magazine(&c, a), (1, None));
    assert!(c.projectiles.is_empty());
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 299).unwrap();
    assert_eq!(headshot_magazine(&c, a), (1, None));
    c.step_battle_execution_until(&mut s, 300).unwrap();
    assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
    assert_eq!(c.units[&e].stats.current_health, 240);
    c.step_battle_execution_until(&mut s, 500).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(c.units[&a].resonance_current, 20);
    assert_eq!(c.units[&a].stats.attack, 100);
    assert_eq!(c.units[&a].stats.current_health, 500);
    assert!(preparations(&c).is_empty() && rewards(&c).is_empty());
    start(&mut c, a, b, 600, true);
    c.step_battle_execution_until(&mut s, 1799).unwrap();
    assert_eq!(c.units[&b].stats.current_health, 1000);
    assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
    c.step_battle_execution_until(&mut s, 1800).unwrap();
    assert_eq!(headshot_magazine(&c, a), (1, None));
    // The blocked ordinary attack resumes automatically at reload completion.
    c.step_battle_execution_until(&mut s, 2600).unwrap();
    assert_eq!(c.units[&b].stats.current_health, 900);
    assert_eq!(headshot_magazine(&c, a), (0, Some(3500)));
    assert_eq!(c.units[&a].resonance_current, 22); // two ordinary battle-time pulses
}
#[test]
fn composition_headshot_resonance_normal_and_nonlethal_shots_never_reward() {
    for prepared in [false, true] {
        let (mut c, mut s, a, e, _) = headshot_fixture(|_| {});
        if prepared {
            cast(&mut c, a, 0);
        }
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&a).unwrap().resonance_current = 0;
        c.units.get_mut(&e).unwrap().stats.current_health = if prepared { 1000 } else { 90 };
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert_eq!(c.units[&a].resonance_current, 0);
        assert_eq!(
            c.units[&e].stats.current_health,
            if prepared { 750 } else { 0 }
        );
        assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
        assert!(preparations(&c).is_empty());
    }
}
#[test]
fn composition_headshot_resonance_bonus_kill_excluded_and_cap_does_not_autocast() {
    use crate::game::stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect};
    for hp in [240, 300] {
        let (mut c, mut s, a, e, _) = headshot_fixture_weapon(
            |_| {},
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
        c.units.get_mut(&a).unwrap().resonance_current = 90;
        c.units.get_mut(&a).unwrap().skill_activation_mode = SkillActivationMode::Auto;
        c.units.get_mut(&e).unwrap().stats.current_health = hp;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(
            c.units[&a].resonance_current,
            if hp == 240 { 100 } else { 90 }
        );
        assert!(!c.units[&a].pending_cast);
        assert!(preparations(&c).is_empty());
    }
}
#[test]
fn composition_headshot_resonance_area_once_per_action_uses_existing_weapon_shape() {
    use crate::game::ability::{DeliveryDef, SkillAreaAnchorSource};
    let (mut c, mut s, a, e, b) = headshot_fixture(|doc| {
        doc["equipment_attacks"] = json!([]);
    });
    c.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
        footprint: crate::game::battle::tile_range::TileRangePattern {
            rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            include_anchor_tile: true,
        },
        area: crate::game::ability::SkillTileAreaDeliveryDef {
            anchor: SkillAreaAnchorSource::CastTarget,
            tile_origin: crate::game::ability::SkillTileAreaOrigin::Anchor,
            ..Default::default()
        },
    };
    place(&mut c, b, 4, 2);
    for target in [e, b] {
        c.units.get_mut(&target).unwrap().stats.current_health = 240;
    }
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 500).unwrap();
    assert!(c.units[&e].lifecycle.is_dead() && c.units[&b].lifecycle.is_dead());
    assert_eq!(c.units[&a].resonance_current, 20);
    assert_eq!(c.units[&a].stats.attack, 100);
}
#[test]
fn composition_headshot_resonance_gain_lock_discards_reward_and_withdrawal_keeps_projectile() {
    use crate::game::battle::{buffs::BuffId, core::sim::BattleLiveCommand};
    for withdraw in [false, true] {
        let (mut c, mut s, a, e, b) = headshot_fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 300).unwrap();
        if withdraw {
            c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 350,
            })
            .unwrap();
        } else {
            c.process_event(
                BattleEvent::ApplyBuff {
                    time_ms: 350,
                    caster_instance_id: b,
                    target_instance_id: a,
                    buff_id: BuffId::from_name("r11_stun"),
                    duration_ms: 300,
                    cause: Default::default(),
                },
                350,
            )
            .unwrap();
        }
        c.step_battle_execution_until(&mut s, 464).unwrap();
        if !withdraw {
            assert!(c
                .unit_control_state(a, 464)
                .is_some_and(|v| v.hard_cc_release_time_ms.is_some()));
        }
        c.step_battle_execution_until(&mut s, 700).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(c.units[&a].resonance_current, 0);
        assert!(preparations(&c).is_empty());
    }
}
#[test]
fn composition_headshot_resonance_cancel_before_release_preserves_round_and_preparation() {
    use crate::game::battle::buffs::BuffId;
    let (mut c, mut s, a, e, _) = headshot_fixture(|_| {});
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 101).unwrap();
    c.process_event(
        BattleEvent::ApplyBuff {
            time_ms: 150,
            caster_instance_id: e,
            target_instance_id: a,
            buff_id: BuffId::from_name("r11_stun"),
            duration_ms: 100,
            cause: Default::default(),
        },
        150,
    )
    .unwrap();
    c.step_battle_execution_until(&mut s, 350).unwrap();
    assert_eq!(headshot_magazine(&c, a), (1, None));
    assert!(preparations(&c)[0].bound_action.is_none());
    assert_eq!(c.units[&e].stats.current_health, 1000);
    start(&mut c, a, e, 400, true);
    c.step_battle_execution_until(&mut s, 800).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 750);
    assert_eq!(headshot_magazine(&c, a), (0, Some(2100)));
}
#[test]
fn composition_headshot_resonance_clone_without_observation_preserves_ordered_results() {
    let (mut c, mut s, a, e, _) = headshot_fixture(|_| {});
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    let mut other = c.clone();
    let mut other_s = s.clone();
    c.enable_authoring_observation();
    for (core, state) in [(&mut c, &mut s), (&mut other, &mut other_s)] {
        start(core, a, e, 100, true);
        core.step_battle_execution_until(state, 500).unwrap();
    }
    assert_eq!(c.units[&a].resonance_current, 20);
    assert_eq!(
        c.transaction_debug_snapshot(),
        other.transaction_debug_snapshot()
    );
    assert_eq!(
        c.event_queue_debug_snapshot(),
        other.event_queue_debug_snapshot()
    );
    assert_eq!(
        serde_json::to_value(c.runtime_fact_view()).unwrap(),
        serde_json::to_value(other.runtime_fact_view()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&c.event_log).unwrap(),
        serde_json::to_value(&other.event_log).unwrap()
    );
}
#[test]
fn composition_headshot_resonance_cancellation_rolls_back_release_and_lethal_reward() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (checkpoint, before_time, after_time) in [
        ("prepared_basic_bound", 99, 101),
        ("prepared_basic_released", 299, 300),
        ("state_reaction_applied", 464, 465),
    ] {
        let (mut c, mut s, a, e, _) = headshot_fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, before_time).unwrap();
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
                || c.step_battle_execution_until(&mut s, after_time),
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
            )
        );
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(c.units[&a].resonance_current, 20);
        assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
    }
}

#[test]
fn composition_headshot_resonance_existing_windup_and_recast_keep_original_capture() {
    for recast in [false, true] {
        let (mut c, mut s, a, e, _) = headshot_fixture(free_preparation);
        if recast {
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, if recast { 300 } else { 101 })
            .unwrap();
        cast(&mut c, a, if recast { 350 } else { 150 });
        c.step_battle_execution_until(&mut s, if recast { 351 } else { 151 })
            .unwrap();
        c.units.get_mut(&a).unwrap().resonance_current = 0;
        if recast {
            c.units.get_mut(&e).unwrap().stats.current_health = 240;
        }
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert_eq!(
            c.units[&e].stats.current_health,
            if recast { 0 } else { 900 }
        );
        assert_eq!(c.units[&a].resonance_current, if recast { 20 } else { 0 });
        assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
        let remaining = preparations(&c);
        assert_eq!(remaining.len(), 1);
        assert!(!remaining[0].released && remaining[0].bound_action.is_none());
        assert_eq!(remaining[0].state.as_ref().unwrap().events, 0);
    }
}
#[test]
fn composition_headshot_resonance_released_miss_and_foreign_kill_refund_nothing() {
    let (mut c, mut s, a, e, b) = headshot_fixture(|_| {});
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 300).unwrap();
    hit(&mut c, b, e, 1000, 350);
    c.step_battle_execution_until(&mut s, 700).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(c.units[&a].resonance_current, 0);
    assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
    assert!(preparations(&c).is_empty());
}
#[test]
fn composition_headshot_resonance_invalid_amount_rolls_back_lethal_result() {
    let (mut c, mut s, a, e, _) = headshot_fixture(|doc| {
        prepared_program(doc)["handlers"][0]["body"][0]["GrantOwnerResonance"]["amount"]["root"]
            ["Const"]["value"] = json!(-1);
    });
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 464).unwrap();
    let prepared = preparations(&c);
    let result = c.step_battle_execution_until(&mut s, 465);
    assert!(
        matches!(result, Err(GameError::ValueExpressionFault(_))),
        "{result:?}"
    );
    assert!(c.units[&e].is_active());
    assert_eq!(c.units[&e].stats.current_health, 240);
    assert_eq!(c.units[&a].resonance_current, 0);
    assert_eq!(preparations(&c), prepared);
    assert_eq!(headshot_magazine(&c, a), (0, Some(1800)));
    assert!(c.step_battle_execution_until(&mut s, 500).is_err());
}
#[test]
fn composition_headshot_resonance_authored_gain_modifier_applies_without_stat_buff() {
    let (mut c, mut s, a, e, _) = headshot_fixture(|doc| {
        doc["values"].as_array_mut().unwrap().push(json!({"name":"gain","value":{"SupportProgram":{
            "heal_percent":0,"shield_percent":0,"resonance_gain":{"percent":50,"channels":"Authored"}
        }}}));
        doc["equipment_supports"] = json!([{"equipment":"r11_weapon","program":{"Shared":"gain"}}]);
    });
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 500).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(c.units[&a].resonance_current, 30);
    assert_eq!(c.units[&a].stats.attack, 100);
}
#[test]
fn composition_headshot_resonance_event_history_retained_limit_rolls_back_and_retries() {
    use crate::game::battle::runtime_control as native;
    let (mut c, mut s, a, e, _) = headshot_fixture(|_| {});
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 240;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 464).unwrap();
    let mut probe = c.clone();
    let mut execution = s.clone();
    let (result, admissions) = native::observe_retained_admissions(|| {
        probe.step_battle_execution_until(&mut execution, 465)
    });
    result.unwrap();
    let required = admissions
        .iter()
        .find(|(site, _, _)| *site == "state_event_history")
        .unwrap()
        .2;
    let original = c.runtime_facts.limits.max_retained_bytes;
    c.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = (
        c.transaction_debug_snapshot(),
        c.event_queue_debug_snapshot(),
        serde_json::to_value(c.runtime_fact_view()).unwrap(),
        format!("{s:?}"),
    );
    let error = c.step_battle_execution_until(&mut s, 465).unwrap_err();
    assert!(
        matches!(
            &error,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                site: "state_event_history",
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(
        before,
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            format!("{s:?}")
        )
    );
    c.runtime_facts.limits.max_retained_bytes = original;
    c.step_battle_execution_until(&mut s, 500).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(c.units[&a].resonance_current, 20);
}

#[test]
fn composition_release_gap_prepared_headshot_kill_attribution_and_reload_interval() {
    use crate::game::stats::{Effect,TriggerEffectTarget,TriggerType,TriggeredEffect};
    for (hp,bonus,expected_reward) in [(260,0,20),(300,100,0)] {
        let (mut c,mut s,a,e,b)=headshot_fixture_weapon_setup(|doc| {
            doc["values"][1]["value"]["AttackProgram"]["release_gap"]=json!({"per_second_percent":20,"max_percent":100});
        }, |w| {
            if bonus>0 {
                w.triggered_effects.insert(TriggerType::OnAttack,vec![TriggeredEffect::targeted(
                    TriggerEffectTarget::CounterpartUnit,Effect::BonusDamage{flat:bonus,percent:0,damage_type:DamageType::True})]);
            }
        }, |scenario| {
            for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                spawn.draft.equipped_items[0].persistent_copy_uuid=Some(Uuid::from_u128(0xe10220+i as u128));
            }
        });
        c.enable_authoring_observation();
        cast(&mut c,a,0);c.step_battle_execution_until(&mut s,1).unwrap();
        assert_eq!(c.release_gap_snapshot(a,1).unwrap().unwrap().since_ms,0);
        c.units.get_mut(&e).unwrap().stats.current_health=hp;
        start(&mut c,a,e,100,true);c.step_battle_execution_until(&mut s,300).unwrap();
        let w=c.authoring_observation.as_ref().unwrap().latest_release_gap.as_ref().unwrap();
        assert_eq!((w.time_ms,w.percent,w.since_ms),(300,6,0));
        c.step_battle_execution_until(&mut s,500).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());assert_eq!(c.units[&a].resonance_current,expected_reward);
        assert!(preparations(&c).is_empty());assert_eq!(headshot_magazine(&c,a),(0,Some(1800)));
        if bonus==0 {
            start(&mut c,a,b,600,true);c.step_battle_execution_until(&mut s,1799).unwrap();
            assert_eq!(c.release_gap_snapshot(a,1799).unwrap().unwrap().since_ms,300);
            c.step_battle_execution_until(&mut s,2600).unwrap();
            assert_eq!(c.units[&b].stats.current_health,866);
            let w=c.authoring_observation.as_ref().unwrap().latest_release_gap.as_ref().unwrap();
            assert_eq!((w.time_ms,w.elapsed_ms,w.percent),(2000,1700,34));
        }
    }
}
