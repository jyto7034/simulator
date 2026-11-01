const LIFESTEAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/equipment_lifesteal.skill.yaml"
));
fn lifesteal_fixture(percent: u32, cap: u32) -> Fixture {
    let mut f = fixture(0, false, |doc| {
        *doc = crate::authoring::composition::canonical_overlay_tests::equipment_fixture("weapon");
        program(doc)["basic_lifesteal"] = json!({"percent":percent,"max_heal":cap});
    });
    f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 1;
    f
}
fn lifesteal_step(f: &mut Fixture, now: u64) {
    f.core
        .step_battle_execution_until(&mut f.state, now)
        .unwrap();
}
fn lifesteal_snapshot(f: &Fixture) -> impl PartialEq + std::fmt::Debug {
    (
        f.core.transaction_debug_snapshot(),
        f.core.event_queue_debug_snapshot(),
        serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
        format!("{:?}", f.state),
    )
}
#[test]
fn composition_lifesteal_roundtrip_and_invalid_caps_slots_duplicate_native() {
    use crate::game::ability::equipment_attack::EquipmentAttackCatalog;
    let artifact = crate::authoring::composition::compile_yaml(LIFESTEAL).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let authored = artifact.equipment_attacks()[0]
        .program
        .basic_lifesteal
        .unwrap();
    assert_eq!((authored.percent, authored.max_heal), (25, 17));
    for (old, new) in [
        ("percent: 25", "percent: 0"),
        ("percent: 25", "percent: 101"),
        ("max_heal: 17", "max_heal: 0"),
        ("max_heal: 17", "max_heal: 2147483648"),
    ] {
        let e =
            crate::authoring::composition::compile_yaml(&LIFESTEAL.replace(old, new)).unwrap_err();
        assert!(e.source.definition.contains("basic_lifesteal"), "{e:?}");
    }
    let f = lifesteal_fixture(25, 17);
    for duplicate in [false, true] {
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let weapon = equipment
            .items
            .iter_mut()
            .find(|w| w.id == "r11_weapon")
            .unwrap();
        if duplicate {
            r12_lifesteal_weapon(weapon, 17);
        } else {
            weapon.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
            weapon.weapon_profile = None;
        }
        let e = EquipmentAttackCatalog::compile(
            &equipment,
            &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(),
            &crate::game::value_expr::ExprLimits::default(),
        )
        .unwrap_err();
        assert!(
            e.to_string().contains(if duplicate {
                "duplicate authored"
            } else {
                "require a weapon"
            }),
            "{e:?}"
        );
    }
}
#[test]
fn composition_lifesteal_basic_actual_hp_cap_rounding_and_copy_origin() {
    for (percent, cap, enemy_hp, owner_hp, expected) in [
        (25, 17, 1000, 500, 17),
        (50, 100, 1000, 500, 50),
        (25, 100, 3, 500, 0),
        (25, 100, 5, 500, 1),
        (25, 100, 1000, 999, 1),
    ] {
        let mut f = lifesteal_fixture(percent, cap);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = enemy_hp;
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = owner_hp;
        let copy = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == f.a)
            .unwrap()
            .source_snapshot();
        attack(&mut f, 10, true);
        lifesteal_step(&mut f, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, owner_hp + expected);
        assert_eq!(f.core.units[&f.b].stats.current_health, 1000);
        let facts = f.core.runtime_fact_summary();
        assert_eq!(facts.healing_delta, i64::from(expected));
        if expected > 0 {
            let heal = facts.latest_heal.as_ref().unwrap();
            assert_eq!(heal.source.equipment, Some(copy));
            assert!(heal.source.cast.is_none());
            assert!(heal
                .source
                .authored
                .as_ref()
                .unwrap()
                .field_path
                .ends_with("program.basic_lifesteal"));
        }
    }
}
#[test]
fn composition_lifesteal_area_sums_before_rounding_and_caps_once() {
    let delivery = r#"TileArea(footprint: (include_anchor_tile: true, rows: ["XXXXXXXXX", "XXXX@XXXX", "XXXXXXXXX"]), area: (anchor: CastTarget, tile_origin: Anchor))"#;
    for (hp, cap, expected) in [(3, 100, 1), (1000, 17, 17), (1000, 100, 50)] {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(LIFESTEAL).unwrap();
                program(doc)["basic_lifesteal"]["max_heal"] = json!(cap);
            },
            |skill| *skill = r11_overlay_ron(delivery, 1500, true),
            |_| {},
            None,
        );
        for id in [f.enemy, f.secondary] {
            f.core.units.get_mut(&id).unwrap().stats.current_health = hp;
        }
        super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 1;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: f.a,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 0,
            })
            .unwrap();
        lifesteal_step(&mut f, 1);
        attack(&mut f, 10, true);
        lifesteal_step(&mut f, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 500 + expected);
        assert_eq!(f.core.runtime_fact_summary().heal_mutations, 1);
        assert_eq!(
            f.core.runtime_fact_summary().damage_hp_lost,
            u64::from(hp.min(100)) * 2
        );
    }
}
#[test]
fn composition_lifesteal_shield_and_triggered_attack_excluded() {
    for shield in [false, true] {
        let mut f = lifesteal_fixture(25, 100);
        if shield {
            f.core
                .begin_sustained(
                    f.enemy,
                    f.enemy,
                    crate::game::ability::sustained::SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 1000,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
                .unwrap();
        }
        attack(&mut f, 10, shield);
        lifesteal_step(&mut f, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 500);
        assert_eq!(f.core.runtime_fact_summary().heal_mutations, 0);
        assert_eq!(
            f.core.units[&f.enemy].stats.current_health,
            if shield { 1000 } else { 900 }
        );
    }
}
#[test]
fn composition_lifesteal_admission_settlement_cancel_and_retained_refusal_retry() {
    use crate::{
        authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase},
        game::battle::runtime_control as native,
    };
    for site in ["equipment_attack_bound", "basic_lifesteal_after_heal"] {
        let mut f = lifesteal_fixture(25, 17);
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
        f.core.event_queue.push(BattleEvent::AttackStart {
            time_ms: 10,
            attacker_instance_id: f.a,
            target_instance_id: Some(f.enemy),
            schedule_next: true,
            cause: Default::default(),
        });
        lifesteal_step(
            &mut f,
            if site == "equipment_attack_bound" {
                9
            } else {
                10
            },
        );
        let before = lifesteal_snapshot(&f);
        let token = CancellationToken::default();
        let r = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 11),
            )
        });
        assert!(r.is_err() && token.is_cancelled(), "{site}: {r:?}");
        assert_eq!(before, lifesteal_snapshot(&f));
        lifesteal_step(&mut f, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 517);
    }
    let mut f = lifesteal_fixture(25, 17);
    f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
    f.core.event_queue.push(BattleEvent::AttackStart {
        time_ms: 10,
        attacker_instance_id: f.a,
        target_instance_id: Some(f.enemy),
        schedule_next: true,
        cause: Default::default(),
    });
    lifesteal_step(&mut f, 10);
    let mut probe = f.core.clone();
    let mut state = f.state.clone();
    let (r, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut state, 11));
    r.unwrap();
    let required = admissions
        .iter()
        .find(|(site, _, _)| *site == "equipment_lifesteal_source")
        .unwrap()
        .2;
    f.core.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = lifesteal_snapshot(&f);
    let e = f
        .core
        .step_battle_execution_until(&mut f.state, 11)
        .unwrap_err();
    assert!(
        matches!(
            e,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                site: "equipment_lifesteal_source",
                ..
            })
        ),
        "{e:?}"
    );
    assert_eq!(before, lifesteal_snapshot(&f));
    f.core.runtime_facts.limits = Default::default();
    lifesteal_step(&mut f, 11);
    assert_eq!(f.core.units[&f.a].stats.current_health, 517);
}
#[test]
fn composition_lifesteal_observer_clone_continuation_matches() {
    let mut f = lifesteal_fixture(25, 17);
    attack(&mut f, 10, true);
    let mut observed = f.core.clone();
    let mut state = f.state.clone();
    observed.enable_authoring_observation_with_options(Default::default(), true);
    observed
        .bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1,
                battle_uuid: Uuid::from_u128(0x6767),
            },
        )
        .unwrap();
    lifesteal_step(&mut f, 11);
    observed
        .step_battle_execution_until(&mut state, 11)
        .unwrap();
    assert_eq!(
        f.core.transaction_debug_snapshot(),
        observed.transaction_debug_snapshot()
    );
    assert_eq!(
        f.core.event_queue_debug_snapshot(),
        observed.event_queue_debug_snapshot()
    );
    assert_eq!(
        serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
        serde_json::to_value(observed.runtime_fact_view()).unwrap()
    );
}
#[test]
fn composition_lifesteal_authored_hello_includes_aggregate_goodbye_excludes() {
    use super::compiled_action_fixture::{cast, fixture_content_with_data, place};
    let mut doc = parse_yaml(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/manual_state.skill.yaml"
    )))
    .unwrap();
    let gear = parse_yaml(LIFESTEAL).unwrap();
    doc["values"]
        .as_array_mut()
        .unwrap()
        .push(gear["values"][0].clone());
    doc["equipment_attacks"] = gear["equipment_attacks"].clone();
    doc["skills"][0]["id"] = json!("r11_aura");
    let artifact = compile_json(&doc.to_string()).unwrap();
    let (mut c, mut s, a, e, b) = fixture_content_with_data(
        &["piercing-damage-increase"],
        &[0],
        crate::authoring::node_graph::NativeGraphContent {
            skills: artifact.load_native().unwrap(),
            equipment: vec![],
            mark_weapons: vec![],
        },
        |_| {},
        ResourceCatalog { resources: vec![] },
        |builder| builder.with_equipment_attacks(artifact.equipment_attacks().to_vec()),
        |_| {},
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
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&a).unwrap().next_basic_attack_ms = 100;
    c.event_queue.push(BattleEvent::AttackStart {
        time_ms: 100,
        attacker_instance_id: a,
        target_instance_id: Some(e),
        schedule_next: true,
        cause: Default::default(),
    });
    c.step_battle_execution_until(&mut s, 101).unwrap();
    assert_eq!(
        (
            c.units[&e].stats.current_health,
            c.units[&b].stats.current_health,
            c.units[&a].stats.current_health
        ),
        (960, 950, 517)
    );
    assert_eq!(c.runtime_fact_summary().heal_mutations, 1);
    let activation = c
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .iter()
        .find(|r| r.caster == a)
        .unwrap()
        .activation;
    c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        activation,
        target: None,
        time_ms: 501,
    })
    .unwrap();
    assert!(c.units[&e].stats.current_health < 960 && c.units[&b].stats.current_health < 950);
    assert_eq!(c.units[&a].stats.current_health, 517);
    assert_eq!(c.runtime_fact_summary().heal_mutations, 1);
}
#[test]
fn composition_lifesteal_projectile_finishes_once_after_overlay_and_skips_withdrawn_owner() {
    let delivery="Projectile(speed_units_per_ms: 10000, hit_policy: DirectionalCollision, max_range_tiles: Some(7), max_lifetime_ms: Some(1000), max_pierces: Some(1), collision: (radius_units: 125000, piercing: true))";
    for withdrawn in [false, true] {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(LIFESTEAL).unwrap();
                program(doc)["basic_lifesteal"]["max_heal"] = json!(35);
            },
            |skill| *skill = r11_overlay_ron(delivery, 100, false),
            |_| {},
            None,
        );
        super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 1;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: f.a,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 0,
            })
            .unwrap();
        lifesteal_step(&mut f, 1);
        attack(&mut f, 10, true);
        lifesteal_step(&mut f, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 500);
        assert!(!f.core.projectiles.is_empty());
        if withdrawn {
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: f.a,
                    time_ms: 12,
                })
                .unwrap();
        }
        lifesteal_step(&mut f, 500);
        assert!(f.core.projectiles.is_empty());
        assert_eq!(f.core.runtime_fact_summary().damage_hp_lost, 200);
        assert_eq!(
            f.core.runtime_fact_summary().healing_delta,
            if withdrawn { 0 } else { 35 }
        );
        assert_eq!(
            f.core.runtime_fact_summary().heal_mutations,
            if withdrawn { 0 } else { 1 }
        );
    }
}
