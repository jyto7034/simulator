// Included inside composition_equipment_state: uses the real authored equipment path.
const DOT_RECOVERY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/equipment_dot_recovery.skill.yaml"
));
fn dot_fixture() -> Fixture {
    dot_fixture_source(&crate::authoring::composition::canonical_presence_tests::equipment_fixture("armor"))
}
fn dot_fixture_source(source: &str) -> Fixture {
    let mut f = setup_source(source, |_| {}, |_| {});
    let artifact = compile_json(&parse_yaml(source).unwrap().to_string()).unwrap();
    let buffs = ron::de::from_str(r#"(buffs:[
      (name:"dot_one",kind:PeriodicDamage(damage_per_tick:1,damage_type:True),tick_interval_ms:1000,max_stacks:1,reapply_policy:RefreshDuration),
      (name:"dot_two",kind:PeriodicDamage(damage_per_tick:2,damage_type:True),tick_interval_ms:700,max_stacks:1,reapply_policy:RefreshDuration),
      (name:"kill_stun",kind:Stun,tick_interval_ms:0,max_stacks:1,reapply_policy:RefreshDuration),
      (name:"zero_dot",kind:PeriodicDamage(damage_per_tick:0,damage_type:True),tick_interval_ms:700,max_stacks:1,reapply_policy:RefreshDuration)
    ])"#).unwrap();
    f.core.game_data = GameDataBuilder::empty()
        .with_skills((*f.core.game_data.skill_data).clone())
        .with_equipment(f.core.game_data.equipment_data.items.clone())
        .with_equipment_states(artifact.equipment_states().to_vec())
        .with_buffs(buffs)
        .build_arc();
    f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1000;
    f.core.units.get_mut(&f.b).unwrap().stats.current_health = 1000;
    f
}
fn dot_apply(
    f: &mut Fixture,
    owner: UnitInstanceId,
    target: UnitInstanceId,
    name: &str,
    now: u64,
    duration: u64,
) {
    f.core
        .process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: now,
                caster_instance_id: owner,
                target_instance_id: target,
                buff_id: crate::game::battle::buffs::BuffId::from_name(name),
                duration_ms: duration,
                cause: Default::default(),
            },
            now,
        )
        .unwrap();
}
fn periodic_id(f: &Fixture, owner: UnitInstanceId) -> u64 {
    states(f)
        .into_iter()
        .find(|s| s.owner == owner)
        .expect("equipment active on deployment")
        .activation
}
#[test]
fn composition_dot_recovery_deduplicates_sources_caps_and_observes_exact_expiry() {
    let mut f = dot_fixture();
    let (a, b, e, s) = (f.a, f.b, f.enemy, f.secondary);
    let id = periodic_id(&f, a);
    dot_apply(&mut f, a, e, "dot_one", 0, 3000);
    dot_apply(&mut f, a, e, "dot_two", 0, 3000);
    dot_apply(&mut f, a, s, "dot_one", 0, 2000);
    dot_apply(&mut f, b, s, "dot_two", 0, 5000);
    assert_eq!(f.core.owned_dot_targets(a, 0, 4).unwrap().len(), 2);
    assert!(f.core.owned_dot_targets(a, 0, 1).is_err());
    f.core
        .step_battle_execution_until(&mut f.state, 999)
        .unwrap();
    assert_eq!(f.core.units[&a].stats.current_health, 1000);
    f.core
        .step_battle_execution_until(&mut f.state, 1000)
        .unwrap();
    assert_eq!(f.core.units[&a].stats.current_health, 1015);
    f.core
        .process_event_for_test(
            BattleEvent::EquipmentStateTick {
                time_ms: 1000,
                activation: id,
            },
            1000,
        )
        .unwrap();
    assert_eq!(
        f.core.units[&a].stats.current_health, 1015,
        "duplicate tick"
    );
    f.core
        .step_battle_execution_until(&mut f.state, 2000)
        .unwrap();
    assert_eq!(
        f.core.units[&a].stats.current_health, 1025,
        "exact expiry leaves only first target"
    );
    f.core
        .step_battle_execution_until(&mut f.state, 3000)
        .unwrap();
    assert_eq!(
        f.core.units[&a].stats.current_health, 1025,
        "all own DOT expired; other owner's DOT remains"
    );
}
#[test]
fn composition_dot_recovery_cleanup_discards_timer_and_stale_callbacks() {
    for boundary in ["unequip", "corrosion", "death", "terminal"] {
        let mut f = dot_fixture();
        let (a, e) = (f.a, f.enemy);
        let id = periodic_id(&f, a);
        dot_apply(&mut f, a, e, "dot_one", 0, 5000);
        f.core
            .runtime_transaction(|c| {
                match boundary {
                    "unequip" => {
                        let id = c
                            .items
                            .values()
                            .find(|i| i.owner_unit_instance == a)
                            .unwrap()
                            .instance_id;
                        c.items.remove(&id);
                    }
                    "corrosion" => c.units.get_mut(&a).unwrap().corroded = true,
                    "death" => c.units.get_mut(&a).unwrap().stats.current_health = 0,
                    _ => c.clear_sustained(100)?,
                }
                c.reconcile_sustained(100)
            })
            .unwrap();
        assert!(!states(&f).iter().any(|s| s.activation == id), "{boundary}");
        assert!(!f.core.event_queue.iter().any(|e|matches!(e.as_ref(),BattleEvent::EquipmentStateTick {activation,..} if *activation==id)),"{boundary}");
        let hp = f.core.units[&a].stats.current_health;
        f.core
            .process_event_for_test(
                BattleEvent::EquipmentStateTick {
                    time_ms: 1000,
                    activation: id,
                },
                1000,
            )
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, hp, "{boundary}");
    }
}
#[test]
fn composition_dot_recovery_invalid_clocks_and_event_payload_are_rejected() {
    for source in [
        DOT_RECOVERY.replace("interval_ms: 1000", "interval_ms: 0"),
        DOT_RECOVERY.replace("source: StateOwnerDotTargets", "source: ResolvedTargets"),
        DOT_RECOVERY.replace("moment: Live", "moment: Snapshot"),
        DOT_RECOVERY.replace(
            "{ Const: { value: 10, unit: Magnitude } }",
            "{ Read: EventHpLost }",
        ),
    ] {
        assert!(
            compile_json(&parse_yaml(&source).unwrap().to_string()).is_err(),
            "{source}"
        );
    }
    let artifact = compile_json(&parse_yaml(DOT_RECOVERY).unwrap().to_string()).unwrap();
    let roundtrip = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert_eq!(roundtrip.equipment_states().len(), 1);
}

#[test]
fn composition_dot_recovery_same_timestamp_death_precedes_recovery() {
    let mut f = dot_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    f.core.units.get_mut(&e).unwrap().stats.current_health = 1;
    f.core
        .step_battle_execution_until(&mut f.state, 1000)
        .unwrap();
    assert!(f.core.units[&e].lifecycle.is_dead());
    assert_eq!(f.core.units[&a].stats.current_health, 1000);
    assert!(f.core.owned_dot_targets(a, 1000, 5).unwrap().is_empty());
}
#[test]
fn composition_dot_recovery_zero_friendly_and_other_source_do_not_qualify() {
    let mut f = dot_fixture();
    let (a, b, e) = (f.a, f.b, f.enemy);
    for (owner, target, name) in [
        (a, a, "dot_one"),
        (a, b, "dot_one"),
        (b, e, "dot_one"),
        (a, e, "zero_dot"),
    ] {
        dot_apply(&mut f, owner, target, name, 0, 5000);
    }
    assert!(f.core.owned_dot_targets(a, 0, 5).unwrap().is_empty());
    assert_eq!(
        f.core.owned_dot_targets(b, 0, 5).unwrap(),
        [e].into_iter().collect()
    );
}
#[test]
fn composition_dot_recovery_withdraw_redeploy_uses_new_identity_and_clock() {
    let mut f = dot_fixture();
    let (old, e) = (f.a, f.enemy);
    let old_id = periodic_id(&f, old);
    dot_apply(&mut f, old, e, "dot_one", 0, 7000);
    let owned = f.core.units[&old].source_owned_uuid;
    let draft = f
        .core
        .scenario
        .groups
        .iter()
        .flat_map(|g| &g.spawns)
        .find(|s| s.draft.owned_uuid == owned)
        .unwrap()
        .draft
        .clone();
    f.core
        .apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: old,
            time_ms: 30,
        })
        .unwrap();
    let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: fresh } =
        f.core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: crate::game::resources::Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe511,
                time_ms: 1001,
                current_hp_policy: None,
            })
            .unwrap()
    else {
        unreachable!()
    };
    assert_ne!(old, fresh);
    assert_ne!(old_id, periodic_id(&f, fresh));
    let unit = f.core.units.get_mut(&fresh).unwrap();
    unit.stats.max_health = 10000;
    unit.stats.current_health = 1000;
    unit.stats.attack_interval_ms = 100000;
    unit.next_basic_attack_ms = 100000;
    assert!(
        f.core.owned_dot_targets(fresh, 1001, 5).unwrap().is_empty(),
        "old instance's DOT is not reassigned"
    );
    dot_apply(&mut f, fresh, e, "dot_two", 1001, 5000);
    f.core
        .step_battle_execution_until(&mut f.state, 2000)
        .unwrap();
    assert_eq!(f.core.units[&fresh].stats.current_health, 1000);
    f.core
        .step_battle_execution_until(&mut f.state, 2001)
        .unwrap();
    assert_eq!(f.core.units[&fresh].stats.current_health, 1010);
    f.core
        .process_event_for_test(
            BattleEvent::EquipmentStateTick {
                time_ms: 3000,
                activation: old_id,
            },
            3000,
        )
        .unwrap();
    assert_eq!(f.core.units[&fresh].stats.current_health, 1010);
}
#[test]
fn composition_dot_recovery_cancel_and_member_storage_failure_restore_hp_and_clock() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::game::battle::runtime_control as native;
    let mut f = dot_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    f.core.enable_authoring_observation();
    f.core
        .step_battle_execution_until(&mut f.state, 999)
        .unwrap();
    let snapshot = |f: &Fixture| {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
        )
    };
    let before = snapshot(&f);
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("state_reaction_applied", 1, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || f.core.step_battle_execution_until(&mut f.state, 1000),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!(snapshot(&f), before);
    let mut probe = f.core.clone();
    let mut state = f.state.clone();
    let (result, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut state, 1000));
    result.unwrap();
    let required = admissions
        .iter()
        .find(|(site, _, _)| *site == "owned_dot_member_storage")
        .unwrap()
        .2;
    f.core.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = snapshot(&f);
    let error = f
        .core
        .step_battle_execution_until(&mut f.state, 1000)
        .unwrap_err();
    assert!(
        matches!(
            error,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(snapshot(&f), before);
    f.core.runtime_facts.limits = Default::default();
    f.core
        .step_battle_execution_until(&mut f.state, 1000)
        .unwrap();
    assert_eq!(f.core.units[&a].stats.current_health, 1010);
}

#[test]
fn composition_dot_recovery_native_and_unused_programs_keep_lifetime_validation() {
    use crate::game::{
        ability::state_program::{StateEvent, StateProgram},
        value_expr::ExprLimits,
    };
    let artifact = compile_json(&parse_yaml(DOT_RECOVERY).unwrap().to_string()).unwrap();
    let mut program = artifact.equipment_states()[0].program.clone();
    let limits = ExprLimits::default();
    assert!(program.compile_equipped(&crate::game::resource::ResourceCatalog { resources: vec![] }, None, &limits).is_ok());
    assert!(
        program.compile_bound(None, &limits).is_err(),
        "a finite skill state has no equipment clock"
    );
    assert!(program.compile_prepared(None, &limits).is_err());
    let mut second = program.handlers[0].clone();
    second.id = "second-clock".into();
    program.handlers.push(second);
    assert!(program
        .compile_equipped(&crate::game::resource::ResourceCatalog { resources: vec![] }, None, &limits)
        .unwrap_err()
        .contains("one handler"));
    program.handlers.pop();
    program.handlers[0].event = StateEvent::Periodic { interval_ms: 0 };
    let native: StateProgram = ron::de::from_str(&ron::ser::to_string(&program).unwrap()).unwrap();
    assert!(native.compile_equipped(&crate::game::resource::ResourceCatalog { resources: vec![] }, None, &limits).is_err());
    let mut unused = parse_yaml(DOT_RECOVERY).unwrap();
    unused["equipment_states"] = json!([]);
    unused["values"][0]["value"]["StateProgram"]["handlers"][0]["event"]["Periodic"]
        ["interval_ms"] = json!(0);
    assert!(
        compile_json(&unused.to_string()).is_err(),
        "unused native declarations are validated"
    );
    // Native payload additions retain the existing v1 envelope compatibility;
    // v2 is required only for the separate predicate/boolean composition grammar.
    assert!(compile_json(
        &parse_yaml(&DOT_RECOVERY.replace("version: 2", "version: 1"))
            .unwrap()
            .to_string()
    )
    .is_ok());
}
