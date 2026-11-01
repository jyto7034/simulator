// Included in composition_equipment_state, using real authored equipment copies.
const STUN_END: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/equipment_stun_end.skill.yaml"
));
fn stun_fixture() -> Fixture {
    dot_fixture_source(STUN_END)
}
fn stun_elapsed(f: &mut Fixture, now: u64) {
    f.core
        .step_battle_execution_until(&mut f.state, now)
        .unwrap();
}
fn stun_hit(
    f: &mut Fixture,
    owner: UnitInstanceId,
    target: UnitInstanceId,
    kind: crate::game::ability::sustained::HitStatusKind,
    now: u64,
    duration_ms: u32,
) {
    f.core
        .runtime_transaction(|c| {
            c.begin_sustained(
                owner,
                target,
                SustainedEffectDef::HitStatus {
                    duration_ms,
                    profile: crate::game::ability::sustained::HitStatusProfile {
                        definition: "test-stun".into(),
                        kind,
                        modifier_source: None,
                        vulnerability: None,
                    },
                },
                now,
            )
        })
        .unwrap();
}
#[test]
fn composition_stun_end_native_expiry_root_deadline_and_copy_source() {
    let mut f = stun_fixture();
    let (a, e) = (f.a, f.enemy);
    f.core.enable_authoring_observation();
    dot_apply(&mut f, a, e, "kill_stun", 0, 100);
    stun_elapsed(&mut f, 99);
    assert!(f.core.unit_control_state(e, 99).unwrap().hard_cc_active());
    assert!(!f.core.rooted(e));
    stun_elapsed(&mut f, 100);
    assert!(!f.core.unit_control_state(e, 100).unwrap().hard_cc_active());
    assert!(f.core.rooted(e));
    let state = states(&f);
    assert_eq!(state.len(), 1);
    assert_eq!(state[0].owner, a);
    assert_eq!(state[0].state.events, 1);
    let snapshot = f.core.sustained_effect_snapshot().unwrap();
    let root = snapshot.active.iter().find(|r| r.target == e).unwrap();
    assert_eq!(root.expires_at_ms, 1100);
    let witness = f
        .core
        .authoring_observation
        .as_ref()
        .unwrap()
        .latest_sourced_sustained
        .as_ref()
        .unwrap();
    assert_eq!(witness.target_unit, e);
    assert!(witness
        .authored
        .as_ref()
        .unwrap()
        .field_path
        .contains("handlers[0].body[0]"));
    assert_eq!(
        witness.equipment.unwrap().runtime_item_instance_id,
        state[0].runtime_item_instance_id
    );
    // Root permits a real basic attack while movement is locked.
    let hp = f.core.units[&a].stats.current_health;
    launch(&mut f, e, a, 101);
    stun_elapsed(&mut f, 103);
    assert!(f.core.units[&a].stats.current_health < hp);
    stun_elapsed(&mut f, 1099);
    assert!(f.core.rooted(e));
    stun_elapsed(&mut f, 1100);
    assert!(!f.core.rooted(e));
    assert_eq!(
        states(&f)[0].state.events,
        1,
        "Root ending does not emit Stun ended"
    );
}
#[test]
fn composition_stun_end_refresh_stale_expiry_and_same_owner_overlap() {
    use crate::game::ability::sustained::HitStatusKind;
    for same_time in [false, true] {
        let mut f = stun_fixture();
        let (a, e) = (f.a, f.enemy);
        dot_apply(&mut f, a, e, "kill_stun", 0, 100);
        dot_apply(&mut f, a, e, "kill_stun", 50, 150);
        stun_hit(
            &mut f,
            a,
            e,
            HitStatusKind::Stun,
            0,
            if same_time { 200 } else { 300 },
        );
        stun_elapsed(&mut f, 100);
        assert!(states(&f).is_empty());
        stun_elapsed(&mut f, 199);
        assert!(states(&f).is_empty());
        stun_elapsed(&mut f, 200);
        if !same_time {
            assert!(states(&f).is_empty());
            stun_elapsed(&mut f, 300);
        }
        assert_eq!(states(&f)[0].state.events, 1);
        assert!(f.core.rooted(e));
    }
}
#[test]
fn composition_stun_end_other_owner_and_targets_remain_independent() {
    let mut f = stun_fixture();
    let (a, b, e, s) = (f.a, f.b, f.enemy, f.secondary);
    dot_apply(&mut f, a, e, "kill_stun", 0, 100);
    dot_apply(&mut f, b, e, "kill_stun", 0, 200);
    dot_apply(&mut f, a, s, "kill_stun", 0, 150);
    stun_elapsed(&mut f, 100);
    assert!(f.core.rooted(e) && f.core.unit_control_state(e, 100).unwrap().hard_cc_active());
    assert_eq!(states(&f)[0].state.events, 1);
    stun_elapsed(&mut f, 150);
    assert!(f.core.rooted(s));
    assert_eq!(
        states(&f)
            .iter()
            .find(|s| s.owner == a)
            .unwrap()
            .state
            .events,
        2
    );
    stun_elapsed(&mut f, 200);
    assert_eq!(
        states(&f)
            .iter()
            .find(|s| s.owner == b)
            .unwrap()
            .state
            .events,
        1
    );
    assert_eq!(
        f.core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .filter(|r| r.target == e)
            .count(),
        2
    );
}
#[test]
fn composition_stun_end_non_stun_and_immunity_do_not_fake_removal() {
    use crate::game::{ability::sustained::HitStatusKind, battle::types::UnitTargetTrait};
    for kind in [
        HitStatusKind::Freeze,
        HitStatusKind::Slow { percent: 20 },
        HitStatusKind::Stun,
    ] {
        let mut f = stun_fixture();
        let (a, e) = (f.a, f.enemy);
        if kind == HitStatusKind::Stun {
            f.core
                .units
                .get_mut(&e)
                .unwrap()
                .target_traits
                .push(UnitTargetTrait::StunImmune);
        }
        stun_hit(&mut f, a, e, kind, 0, 100);
        stun_elapsed(&mut f, 100);
        assert!(states(&f).is_empty());
        assert!(!f.core.rooted(e));
    }
    let mut f = stun_fixture();
    let (a, e) = (f.a, f.enemy);
    f.core
        .units
        .get_mut(&e)
        .unwrap()
        .target_traits
        .push(UnitTargetTrait::RootImmune);
    stun_hit(&mut f, a, e, HitStatusKind::Stun, 0, 100);
    assert!(f.core.unit_control_state(e, 0).unwrap().hard_cc_active());
    stun_elapsed(&mut f, 100);
    assert_eq!(states(&f)[0].state.events, 1);
    assert!(!f.core.rooted(e));
}
#[test]
fn composition_stun_end_source_exit_and_equipment_eligibility() {
    for boundary in [
        "withdraw",
        "death",
        "unequip",
        "corrosion",
        "target-death",
        "terminal",
    ] {
        let mut f = stun_fixture();
        let (a, e) = (f.a, f.enemy);
        dot_apply(&mut f, a, e, "kill_stun", 0, 100);
        match boundary {
            "withdraw" => {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 50,
                    })
                    .unwrap();
            }
            "death" => lethal(&mut f.core, e, a, DamageSource::Ability, 50).unwrap(),
            "target-death" => lethal(&mut f.core, a, e, DamageSource::Ability, 50).unwrap(),
            "unequip" => f.core.items.retain(|_, i| i.owner_unit_instance != a),
            "corrosion" => f.core.units.get_mut(&a).unwrap().corroded = true,
            _ => {
                f.core
                    .runtime_transaction(|c| c.clear_sustained(50))
                    .unwrap();
            }
        }
        stun_elapsed(&mut f, 100);
        assert!(states(&f).is_empty(), "{boundary}");
        assert!(!f.core.rooted(e), "{boundary}");
    }
    let mut f = stun_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "kill_stun", 0, 100);
    stun_elapsed(&mut f, 100);
    f.core
        .apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 150,
        })
        .unwrap();
    assert!(f.core.rooted(e));
    stun_elapsed(&mut f, 1100);
    assert!(!f.core.rooted(e));
}
#[test]
fn composition_stun_end_same_copy_refresh_and_data_defined_followup() {
    for source in [
        STUN_END.to_owned(),
        STUN_END
            .replace("value: 1000", "value: 1500")
            .replace("kind: Root", "kind: { Slow: { percent: 25 } }"),
    ] {
        let mut f = dot_fixture_source(&source);
        let (a, e) = (f.a, f.enemy);
        dot_apply(&mut f, a, e, "kill_stun", 0, 100);
        stun_elapsed(&mut f, 100);
        dot_apply(&mut f, a, e, "kill_stun", 150, 50);
        stun_elapsed(&mut f, 200);
        assert_eq!(states(&f)[0].state.events, 2);
        let snapshot = f.core.sustained_effect_snapshot().unwrap();
        let records = snapshot
            .active
            .iter()
            .filter(|r| r.target == e)
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].expires_at_ms,
            if source == STUN_END { 1200 } else { 1700 }
        );
    }
}
#[test]
fn composition_stun_end_compile_rejects_wrong_context_unit_and_profile() {
    let artifact = crate::authoring::composition::compile_yaml(STUN_END).unwrap();
    assert_eq!(
        Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .equipment_states()
            .len(),
        1
    );
    for source in [
        STUN_END.replace("OwnerStunEnded", "OwnerKill"),
        STUN_END.replace("unit: Milliseconds", "unit: Count"),
        STUN_END.replace("definition: lingering-root", "definition: ''"),
    ] {
        let error = crate::authoring::composition::compile_yaml(&source).unwrap_err();
        assert!(
            error.source.definition.contains("StateProgram"),
            "{error:?}"
        );
    }
}
#[test]
fn composition_stun_end_cancellation_and_retained_refusal_restore_then_retry() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::game::battle::runtime_control as native;
    let snapshot = |f: &Fixture| {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
        )
    };
    for site in [
        "event_target_status_applied",
        "equipment_stun_end_dispatched",
    ] {
        let mut f = stun_fixture();
        let (a, e) = (f.a, f.enemy);
        f.core.enable_authoring_observation();
        dot_apply(&mut f, a, e, "kill_stun", 0, 100);
        stun_elapsed(&mut f, 99);
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 100),
            )
        });
        assert!(
            result.is_err() && token.is_cancelled(),
            "{site}: {result:?}"
        );
        assert_eq!(snapshot(&f), before, "{site}");
        stun_elapsed(&mut f, 100);
        assert_eq!(states(&f)[0].state.events, 1);
        assert!(f.core.rooted(e));
    }
    let mut f = stun_fixture();
    let (a, e) = (f.a, f.enemy);
    f.core.enable_authoring_observation();
    dot_apply(&mut f, a, e, "kill_stun", 0, 100);
    stun_elapsed(&mut f, 99);
    let mut probe = f.core.clone();
    let mut state = f.state.clone();
    let (result, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut state, 100));
    result.unwrap();
    let required = admissions
        .iter()
        .find(|(site, _, _)| *site == "equipment_state_record")
        .unwrap()
        .2;
    f.core.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = snapshot(&f);
    let error = f
        .core
        .step_battle_execution_until(&mut f.state, 100)
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
    stun_elapsed(&mut f, 100);
    assert_eq!(states(&f)[0].state.events, 1);
    assert!(f.core.rooted(e));
}
#[test]
fn composition_stun_end_observer_and_clone_continuation_are_identical() {
    let mut f = stun_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "kill_stun", 0, 100);
    let mut observed = Fixture {
        core: f.core.clone(),
        state: f.state.clone(),
        a: f.a,
        b: f.b,
        enemy: f.enemy,
        secondary: f.secondary,
        run: f.run.clone(),
    };
    observed.core.enable_authoring_observation();
    for time in [100, 1100] {
        stun_elapsed(&mut f, time);
        stun_elapsed(&mut observed, time);
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            observed.core.transaction_debug_snapshot()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            observed.core.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_summary()).unwrap(),
            serde_json::to_value(observed.core.runtime_fact_summary()).unwrap()
        );
    }
    assert!(observed
        .core
        .authoring_observation
        .as_ref()
        .unwrap()
        .latest_sourced_sustained
        .is_some());
}
