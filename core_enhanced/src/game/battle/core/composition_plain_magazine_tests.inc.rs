const PLAIN_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/plain_magazine.skill.yaml"
));
fn area_setup() -> GunFixture {
    setup_source_profile(PLAIN_SOURCE, true)
}
fn area_enemy(f: &mut GunFixture, x: i32, y: i32) {
    f.core.units.get_mut(&f.b).unwrap().owner = Side::Opponent;
    super::super::compiled_action_fixture::place(&mut f.core, f.b, x, y);
}
#[test]
fn composition_plain_magazine_optional_reward_roundtrip_and_partial_rejection() {
    let a = crate::authoring::composition::compile_yaml(PLAIN_SOURCE).unwrap();
    let b = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
    let m = b.equipment_attacks()[0].program.magazine.as_ref().unwrap();
    assert_eq!(m.reload_ms, 2000);
    assert!(m.reward_family.is_none());
    let old = "(capacity:6,reload_ms:3000,normal_percent:100,enhanced_primary_percent:150,enhanced_secondary_percent:100,enhanced_max_hits:8,reward_family:\"little-red\")";
    let old: crate::game::ability::equipment_attack::EquipmentMagazine =
        ron::de::from_str(old).unwrap();
    assert_eq!(old.enhanced_primary_percent, Some(150));
    let legacy = ron::ser::to_string(&old).unwrap();
    assert!(!legacy.contains("Some("), "{legacy}");
    for field in [
        "enhanced_primary_percent",
        "enhanced_secondary_percent",
        "enhanced_max_hits",
        "reward_family",
    ] {
        let mut d = parse_yaml(SOURCE).unwrap();
        d["values"][0]["value"]["AttackProgram"]["magazine"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        let error = compile_json(&d.to_string()).unwrap_err();
        assert!(format!("{error:?}").contains("magazine"));
    }
}
#[test]
fn composition_plain_magazine_projectile_explodes_at_impact_primary_once_secondary_ratio() {
    let mut f = area_setup();
    area_enemy(&mut f, 4, 2);
    start(&mut f, 1, false);
    advance(&mut f, 201);
    assert_eq!(rounds(&f, f.a), 5);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
    assert_eq!(f.core.units[&f.b].stats.current_health, 10000);
    assert_eq!(f.core.projectiles.len(), 1);
    assert!(f.core.projectiles.values().next().unwrap().area.is_some());
    advance(&mut f, 801);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
    assert_eq!(f.core.units[&f.b].stats.current_health, 9950);
    assert_eq!(rounds(&f, f.a), 5);
    assert!(f.core.projectiles.is_empty());
}
#[test]
fn composition_plain_magazine_six_releases_reload_2000_and_no_cohort_reward() {
    let mut f = area_setup();
    for i in 0..6 {
        shot(&mut f, 1 + i * 1000);
        assert_eq!(rounds(&f, f.a), 5 - i as u32);
    }
    let key = copy(&f, f.a);
    assert_eq!(
        f.core.equipment_attack_state.magazines[&key].reload_at,
        Some(7201)
    );
    reward(&mut f, 6000);
    assert_eq!(rounds(&f, f.a), 0);
    start(&mut f, 6001, false);
    advance(&mut f, 6801);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9400);
    advance(&mut f, 7200);
    assert_eq!(rounds(&f, f.a), 0);
    advance(&mut f, 7201);
    assert_eq!(rounds(&f, f.a), 6);
    assert!(!witness(&f).enhanced);
}
#[test]
fn composition_plain_magazine_live_impact_membership_and_frozen_release_geometry() {
    let mut f = area_setup();
    start(&mut f, 1, false);
    advance(&mut f, 201);
    f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
        crate::game::data::abnormality_data::BasicAttackAction::Damage;
    area_enemy(&mut f, 4, 2);
    advance(&mut f, 801);
    assert_eq!(f.core.units[&f.b].stats.current_health, 9950);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
}
#[test]
fn composition_plain_magazine_lost_primary_does_not_explode_or_refund_round() {
    let mut f = area_setup();
    area_enemy(&mut f, 4, 2);
    start(&mut f, 1, false);
    advance(&mut f, 201);
    f.core.units.get_mut(&f.enemy).unwrap().lifecycle =
        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
    advance(&mut f, 801);
    assert_eq!(f.core.units[&f.b].stats.current_health, 10000);
    assert_eq!(rounds(&f, f.a), 5);
    assert!(f.core.projectiles.is_empty());
}
#[test]
fn composition_plain_magazine_air_filter_and_flight_survives_withdrawal() {
    for air in [false, true] {
        let mut f = area_setup();
        f.core.units.get_mut(&f.a).unwrap().basic_attack.air_capable = air;
        start(&mut f, 1, false);
        advance(&mut f, 201);
        assert_eq!(
            f.core
                .projectiles
                .values()
                .next()
                .unwrap()
                .target_instance_id,
            f.enemy
        );
        area_enemy(&mut f, 4, 2);
        f.core.units.get_mut(&f.b).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 202,
            })
            .unwrap();
        advance(&mut f, 801);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        assert_eq!(
            f.core.units[&f.b].stats.current_health,
            if air { 9950 } else { 10000 }
        );
        assert!(f.core.projectiles.is_empty());
    }
}
#[test]
fn composition_plain_magazine_cancel_at_explosion_rolls_back_damage_facts_and_queue() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut f = area_setup();
    area_enemy(&mut f, 4, 2);
    start(&mut f, 1, false);
    advance(&mut f, 201);
    // Contact is at 366ms: 201ms release + (2 tiles - 0.35 radius) / 0.01.
    // The public runner commits earlier events individually; test this event's transaction.
    advance(&mut f, 365);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
    let snap = |f: &GunFixture| {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
        )
    };
    let before = snap(&f);
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("basic_projectile_area_resolved", 1, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || f.core.step_battle_execution_until(&mut f.state, 366),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!(before, snap(&f));
    advance(&mut f, 801);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
    assert_eq!(f.core.units[&f.b].stats.current_health, 9950);
}
#[test]
fn composition_plain_magazine_retained_capture_failure_rolls_back_ammo_and_retries() {
    use crate::game::battle::runtime_control as native;
    let mut f = area_setup();
    start(&mut f, 1, false);
    advance(&mut f, 200);
    let mut probe = f.core.clone();
    let mut state = f.state.clone();
    let (result, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut state, 201));
    result.unwrap();
    let required = admissions
        .iter()
        .filter(|(site, _, _)| *site == "basic_projectile_area_capture")
        .map(|(_, _, n)| *n)
        .max()
        .unwrap();
    let limit = f.core.runtime_facts.limits.max_retained_bytes;
    f.core.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = (
        f.core.transaction_debug_snapshot(),
        f.core.event_queue_debug_snapshot(),
        format!("{:?}", f.state),
    );
    let error = f
        .core
        .step_battle_execution_until(&mut f.state, 201)
        .unwrap_err();
    assert!(
        matches!(
            error,
            crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                site: "basic_projectile_area_capture",
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(
        before,
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state)
        )
    );
    f.core.runtime_facts.limits.max_retained_bytes = limit;
    advance(&mut f, 801);
    assert_eq!(rounds(&f, f.a), 5);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
}
#[test]
fn composition_plain_magazine_area_lifesteal_once_and_observer_free_clone() {
    let source = PLAIN_SOURCE.replace(
        "reactions: []",
        "reactions: []\n        basic_lifesteal: { percent: 50, max_heal: 1000 }",
    );
    let mut f = setup_source_profile(&source, true);
    area_enemy(&mut f, 4, 2);
    f.core.units.get_mut(&f.a).unwrap().stats.current_health = 5000;
    start(&mut f, 1, true);
    advance(&mut f, 201);
    let mut clone = f.core.clone();
    let mut state = f.state.clone();
    clone.authoring_observation = None;
    advance(&mut f, 801);
    clone.step_battle_execution_until(&mut state, 801).unwrap();
    assert_eq!(f.core.units[&f.a].stats.current_health, 5075);
    assert_eq!(
        serde_json::to_value(&f.core.event_log.entries).unwrap(),
        serde_json::to_value(&clone.event_log.entries).unwrap()
    );
    assert_eq!(
        format!("{:?}", f.core.runtime_facts),
        format!("{:?}", clone.runtime_facts)
    );
    for id in [f.a, f.b, f.enemy] {
        assert_eq!(
            f.core.units[&id].stats.current_health,
            clone.units[&id].stats.current_health
        );
    }
}
#[test]
fn composition_plain_magazine_impact_anchor_samples_current_motion() {
    use crate::game::{
        battle::core::movement::types::{ActiveMovementSegment, WorldVec2},
        resources::Position,
    };
    let mut f = area_setup();
    start(&mut f, 1, false);
    advance(&mut f, 201);
    let (id, projectile) = f
        .core
        .projectiles
        .iter()
        .next()
        .map(|(id, p)| (*id, p.clone()))
        .unwrap();
    assert_eq!(projectile.target_instance_id, f.enemy);
    f.core.active_movement_segments.insert(
        f.enemy,
        ActiveMovementSegment {
            start: WorldVec2::from_tile_center(Position::new(4, 1)),
            target: WorldVec2::from_tile_center(Position::new(3, 2)),
            started_at_ms: 201,
            ends_at_ms: 301,
        },
    );
    area_enemy(&mut f, 2, 3);
    assert_eq!(
        f.core.live_unit_projected_tile(f.enemy),
        Some(Position::new(4, 1))
    );
    let actual = f
        .core
        .runtime_transaction(|core| {
            core.resolve_basic_projectile_area(
                &projectile,
                projectile.area.as_ref().unwrap(),
                301,
                id,
                f.enemy,
                projectile.source_snapshot.clone(),
                WorldVec2::from_tile_center(Position::new(3, 2)),
            )
        })
        .unwrap();
    assert_eq!(actual, 150);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
    assert_eq!(f.core.units[&f.b].stats.current_health, 9950);
}
#[test]
fn composition_plain_magazine_target_locked_single_hit_uses_normal_ratio_once() {
    let mut f = setup_source(&PLAIN_SOURCE.replace("normal_percent: 100", "normal_percent: 50"));
    f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
        ron::de::from_str("Projectile(speed_units_per_ms:10000)").unwrap();
    shot(&mut f, 1);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 9950);
    assert_eq!(rounds(&f, f.a), 5);
}
#[test]
fn composition_plain_magazine_rudolta_bombardment_keeps_running_through_reload_and_stun() {
    use crate::game::battle::buffs::BuffId;
    let mut f = area_setup();
    for i in 0..6 {
        shot(&mut f, 1 + i * 1000);
    }
    assert_eq!(rounds(&f, f.a), 0);
    let due = f.core.equipment_attack_state.magazines[&copy(&f, f.a)].reload_at;
    assert_eq!(due, Some(7201));
    advance(&mut f, 6000);
    f.core.units.get_mut(&f.a).unwrap().resonance_current = 100;
    let action = f
        .core
        .request_skill_action(6000, f.a, "r11_aura".into(), None)
        .unwrap();
    advance(&mut f, 6001);
    f.core
        .units
        .get_mut(&f.a)
        .unwrap()
        .basic_attack_speed_percent = 900;
    f.core
        .process_event(
            BattleEvent::ApplyBuff {
                time_ms: 6001,
                caster_instance_id: f.enemy,
                target_instance_id: f.a,
                buff_id: BuffId::from_name("r11_stun"),
                duration_ms: 3000,
                cause: Default::default(),
            },
            6001,
        )
        .unwrap();
    assert!(f
        .core
        .unit_control_state(f.a, 6001)
        .unwrap()
        .hard_cc_active());
    advance(&mut f, 6402);
    assert_eq!(
        f.core.query_action(&action).unwrap().facts.damage_hp_lost,
        450
    );
    assert_eq!(rounds(&f, f.a), 0);
    assert_eq!(f.core.units[&f.enemy].stats.current_health, 8950);
    advance(&mut f, 7200);
    assert_eq!(rounds(&f, f.a), 0);
    advance(&mut f, 7201);
    assert_eq!(rounds(&f, f.a), 6);
    assert!(f
        .core
        .unit_control_state(f.a, 7201)
        .unwrap()
        .hard_cc_active());
}

#[test]
fn composition_kill_speed_rudolta_actual_grenade_and_bombardment_trigger_without_ammo_reward() {
    let kill_source=include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/skill_composition/kill_speed.skill.yaml"));
    let mut document=parse_yaml(PLAIN_SOURCE).unwrap();
    let kill=parse_yaml(kill_source).unwrap();
    document["values"].as_array_mut().unwrap().extend(kill["values"].as_array().unwrap().iter().cloned());
    document["equipment_states"]=kill["equipment_states"].clone();
    // JSON is also valid YAML; both programs use their authored common contracts.
    for bombardment in [false,true] {
        let mut f=setup_source_profile(&document.to_string(),true);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health=50;
        if bombardment {
            advance(&mut f,10);
            f.core.units.get_mut(&f.a).unwrap().resonance_current=100;
            f.core.request_skill_action(10,f.a,"r11_aura".into(),None).unwrap();
            advance(&mut f,412);
            assert_eq!(rounds(&f,f.a),6);
        } else {
            shot(&mut f,1);
            assert_eq!(rounds(&f,f.a),5);
        }
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!(f.core.sustained_basic_interval(f.a,1000),667);
        let snapshot=f.core.sustained_effect_snapshot().unwrap();
        assert_eq!(snapshot.equipment_states.len(),1);
        assert_eq!(snapshot.equipment_states[0].state.events,1);
        assert_eq!(snapshot.active.iter().filter(|r|r.target_state.as_ref().is_some_and(|p|p.definition=="delivery-coat-speed")).count(),1);
    }
}

#[test]
fn composition_chance_stun_rudolta_grenade_applies_but_independent_bombardment_does_not() {
    let hit=crate::authoring::composition::parse_yaml(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/skill_composition/chance_stun.skill.yaml"))).unwrap();
    let mut doc=parse_yaml(PLAIN_SOURCE).unwrap();
    doc["values"][0]["value"]["AttackProgram"]["on_hit_status"]=hit["values"][0]["value"]["AttackProgram"]["on_hit_status"].clone();
    for bombardment in [false,true] {
        let mut f=setup_source_profile(&doc.to_string(),true);
        if bombardment {
            advance(&mut f,10);f.core.units.get_mut(&f.a).unwrap().resonance_current=100;
            f.core.request_skill_action(10,f.a,"r11_aura".into(),None).unwrap();advance(&mut f,2011);
            assert_eq!(rounds(&f,f.a),6);assert_eq!(f.core.units[&f.enemy].stats.current_health,9550);
        } else {
            start(&mut f,1,true);advance(&mut f,801);
            assert_eq!(rounds(&f,f.a),5);assert_eq!(f.core.units[&f.enemy].stats.current_health,9900);
        }
        let count=f.core.sustained_effect_snapshot().unwrap().active.into_iter().filter(|s|s.hit_status.as_ref().is_some_and(|p|p.kind==crate::game::ability::sustained::HitStatusKind::Stun)).count();
        assert_eq!(count,usize::from(!bombardment));
    }
}
