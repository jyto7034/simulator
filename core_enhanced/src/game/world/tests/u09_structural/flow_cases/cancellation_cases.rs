use super::*;

#[test]
fn rv001_original_repeat_reset_retains_both_released_child_owners_on_withdrawal() {
    let (mut core, player, employee, _, targets, before) = released_child_fixture_options(true);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(before - hp(&core, &targets), 776);
    assert_eq!(step_count(&core), 6);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(before - hp(&core, &targets), 1568);
    core.advance_active_battle_for_server_tick(10).unwrap();
    assert_eq!(before - hp(&core, &targets), 2096);
    core.advance_active_battle_for_server_tick(40).unwrap();
    assert_eq!(before - hp(&core, &targets), 2096);
    assert_eq!(step_count(&core), 6);
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    println!("RV001 original_repeat reset=true child_owners=2 cumulative_loss=250,776,1568,2096 deferred_children=0 final_cleanup=true");
}

#[test]
fn rv001_staged_withdrawal_native_work_resource_clone_and_retained_boundaries() {
    use crate::game::battle::runtime_control as native;
    let (base, player, employee, _, _, _) = released_child_fixture();
    let withdraw = |core: &mut GameCore| {
        execute_current_battle_mutation(
            core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
    };
    let mut clean = base.clone();
    let (result, admissions) = native::observe_retained_admissions(|| withdraw(&mut clean));
    result.unwrap();
    let expected = withdrawal_roots(&clean);
    let before = withdrawal_roots(&base);
    // Find the actual minimum accepted native budget; no configured maximum
    // or workload is increased, and every rejected candidate must be identical.
    for kind in ["work", "clone"] {
        let defaults = native::RuntimeFactLimits::default();
        let mut low = 1;
        let mut high = if kind == "work" {
            defaults.max_work
        } else {
            defaults.max_clone_slots as u64
        };
        let mut last_site = String::new();
        while low < high {
            let middle = low + (high - low) / 2;
            let mut limits = defaults;
            if kind == "work" {
                limits.max_work = middle;
            } else {
                limits.max_clone_slots = middle as usize;
            }
            let mut trial = base.clone();
            match native::run(limits, || withdraw(&mut trial)) {
                Ok(_) => {
                    assert_eq!(withdrawal_roots(&trial), expected);
                    high = middle;
                }
                Err(GameError::BattleRuntimeFault(fault))
                    if fault.code
                        == if kind == "work" {
                            native::RuntimeFaultCode::WorkLimit
                        } else {
                            native::RuntimeFaultCode::CloneLimit
                        } =>
                {
                    assert_eq!(withdrawal_roots(&trial), before);
                    last_site = fault.site.into();
                    low = middle + 1;
                }
                other => panic!("unexpected {kind} boundary: {other:?}"),
            }
        }
        let mut limits = defaults;
        if kind == "work" {
            limits.max_work = high - 1;
        } else {
            limits.max_clone_slots = (high - 1) as usize;
        }
        let mut retry = base.clone();
        let error = native::run(limits, || withdraw(&mut retry)).unwrap_err();
        assert_eq!(withdrawal_roots(&retry), before);
        withdraw(&mut retry).unwrap();
        assert_eq!(withdrawal_roots(&retry), expected);
        println!("RV001 withdrawal {kind}_minimum={high} one_short_fault={error:?} last_binary_site={last_site} rollback_and_retry_exact=true");
    }
    if let Some((site, _, boundary)) = admissions.iter().max_by_key(|row| row.2) {
        let mut trial = base.clone();
        let error = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: boundary - 1,
                ..Default::default()
            },
            || withdraw(&mut trial),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ));
        assert_eq!(withdrawal_roots(&trial), before);
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: *boundary,
                ..Default::default()
            },
            || withdraw(&mut trial),
        )
        .unwrap();
        assert_eq!(withdrawal_roots(&trial), expected);
        println!("RV001 withdrawal retained_minimum={boundary} site={site} one_short_fault={error:?} rollback_retry_exact=true");
    } else {
        panic!("withdrawal fixture must reach retained growth to test late admission");
    }
}

fn withdrawal_roots(core: &GameCore) -> serde_json::Value {
    let active = core.state.active_battle.as_ref().unwrap();
    serde_json::json!({
        "run": core.get_run_snapshot_json().unwrap(),
        "grants": serialized_grant_roots(core),
        "revision": active.battle_state_revision,
        "battle": native_probe::flow_roots(&active.battle),
        "resources": format!("{:?}", core.battle_typed_resources()),
    })
}

#[test]
fn rv001_staged_withdrawal_late_cancellation_restores_owned_flow_and_retries() {
    use crate::authoring::control::{
        self, CancellationToken, ExecutionLimits, ExecutionPhase, NativeStorageProvenance,
    };
    let ((base, player, employee, _, targets, before_hp), _, provenance) =
        control::run_with_provenance(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            NativeStorageProvenance::fresh(),
            released_child_fixture,
        )
        .expect("fixture construction records native storage provenance");
    let mut clean = base.clone();
    execute_current_battle_mutation(
        &mut clean,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    let clean_roots = withdrawal_roots(&clean);
    // These sites cover the taken Flow owner, the post-resource-close child
    // settlement scan, and the final cleanup poll after cancellation mutation.
    for site in [
        "flow_side_cancel",
        "flow_cancelled_child_settlement",
        "flow_withdraw_cleanup",
    ] {
        let mut core = base.clone();
        let before = withdrawal_roots(&core);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run_with_provenance(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                provenance,
                || {
                    execute_current_battle_mutation(
                        &mut core,
                        player,
                        PlayerBehavior::WithdrawUnit {
                            employee_uuid: employee,
                        },
                    )
                },
            )
        });
        let failure = result
            .err()
            .expect("late cancellation must reach real charged site");
        assert!(token.is_cancelled(), "{failure:?}");
        assert_eq!(failure.code, "authoring_cancelled");
        assert_eq!(failure.site, site);
        assert_eq!(
            withdrawal_roots(&core),
            before,
            "{site} failed candidate preserves every owner"
        );
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
        .unwrap();
        assert_eq!(
            withdrawal_roots(&core),
            clean_roots,
            "{site} retry matches clean operation"
        );
        core.advance_active_battle_for_server_tick(60).unwrap();
        assert_eq!(before_hp - hp(&core, &targets), 1059);
        assert_eq!(
            native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
            0
        );
        println!("RV001 staged_withdraw site={site} failed_prestate=true owned_flow_restored=true retry_exact=true released_final_loss=1059");
    }
}

fn released_child_fixture() -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    u32,
) {
    released_child_fixture_options(false)
}

fn released_child_fixture_options(
    repeat_original: bool,
) -> (
    GameCore,
    Uuid,
    Uuid,
    UnitInstanceId,
    Vec<UnitInstanceId>,
    u32,
) {
    let skills = skill_text(100)
        .replace(
            r#"Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)"#,
            r#"Sequence(id: "original", children: [
            Action(id: "prior_node", step_id: "prior", wait: EffectsSettled),
            Action(id: "original_bite", step_id: "bite", wait: EffectsSettled),
            Action(id: "later_node", step_id: "later", wait: EffectsSettled)])"#,
        )
        .replace(
            "steps: [SkillStepDef",
            r#"steps: [
        (id: "prior", delay_ms: 0, target: CastTarget, delivery: Instant,
            effects: [(id: "prior_damage", op: Damage(amount: 5, damage_type: True))]),
        (id: "later", delay_ms: 0, target: CastTarget, delivery: Instant,
            effects: [(id: "later_damage", op: Damage(amount: 17, damage_type: True))]),
        SkillStepDef"#,
        );
    let skills = if repeat_original {
        let action = r#"Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)"#;
        skills.replace(
            action,
            &format!(r#"Repeat(id: "original_twice", count: 2, interval_ms: 10, body: {action})"#),
        )
    } else {
        skills
    };
    let relics = relic_text(None)
        .replace(r#"Action(id: "heal_action", step_id: "heal_once", wait: EffectsSettled)"#,
            r#"Repeat(id: "released_twice", count: 2, interval_ms: 50,
                body: Action(id: "heal_action", step_id: "heal_once", wait: Released))"#)
        .replace("target: SelfUnit, delivery: Instant", r#"target: CastTarget, delivery: TileArea(
            footprint: (include_anchor_tile: true, rows: ["@"]),
            area: (anchor: CastTarget, tile_origin: Anchor, tracking: GroundFixed,
                hit_targets: Enemies, include_caster: false, tick_policy: EveryTick,
                duration_ms: 30, tick_interval_ms: Some(10)))"#)
        .replace("(id: \"heal\", op: Heal(amount: 37))", r#"
            (id: "own_damage", op: Damage(amount: 7, damage_type: True)),
            (id: "prefix_damage", op: DamageExpr(amount: (moment: Live,
                root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost))), damage_type: True))"#);
    let resources = crate::game::resource::ResourceCatalog {
        resources: vec![crate::game::resource::ResourceDefinition {
            id: crate::game::resource::ResourceId::new("rv_delivery".into()).unwrap(),
            owner: crate::game::resource::ResourceOwnerKind::Employee,
            lifetime: crate::game::resource::ResourceLifetime::Delivery,
            initial: 7,
            cap: 7,
        }],
    };
    let (mut base, player, employee, actor, targets) =
        prepare_with_generated_count(&skills, &relics, Some(resources), None, 2);
    // Keep the selected target stationary through all ground-fixed ticks.
    let battle = &mut base.state.active_battle.as_mut().unwrap().battle;
    native_probe::place_stationary(battle, actor, 2, 2);
    native_probe::place_stationary(battle, targets[0], 3, 2);
    if repeat_original {
        let target = battle.units.get_mut(&targets[0]).unwrap();
        target.stats.max_health = 10000;
        target.stats.current_health = 10000;
    }
    let before = hp(&base, &targets);
    cast(&mut base, player, employee);
    // Child scope starts at prior5 + anchor113 = 118, excludes later17.
    // Each tick applies 7, then its prefix: 125, 257, 521.
    assert_eq!(
        before - hp(&base, &targets),
        if repeat_original { 250 } else { 267 }
    );
    assert_eq!(step_count(&base), if repeat_original { 3 } else { 4 });
    (base, player, employee, actor, targets, before)
}

#[test]
fn rv001_released_child_repeat_keeps_scoped_prefix_after_withdrawal_and_cc() {
    let (base, player, employee, actor, targets, before) = released_child_fixture();
    for mode in ["withdraw", "hard_cc_callback"] {
        let mut core = base.clone();
        if mode == "withdraw" {
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::WithdrawUnit {
                    employee_uuid: employee,
                },
            )
            .unwrap();
        } else {
            native_probe::hard_cc_cancel(
                &mut core.state.active_battle.as_mut().unwrap().battle,
                actor,
                1,
            );
        }
        // Repeat reset has already removed the first child's node occurrence.
        // A second cancellation must not double-close its old reservation.
        native_probe::hard_cc_cancel(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            actor,
            1,
        );
        assert_eq!(
            native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
            1
        );
        core.advance_active_battle_for_server_tick(10).unwrap();
        assert_eq!(
            before - hp(&core, &targets),
            531,
            "{mode} second tick keeps exact own prefix"
        );
        core.advance_active_battle_for_server_tick(10).unwrap();
        assert_eq!(
            before - hp(&core, &targets),
            1059,
            "{mode} third tick keeps exact own prefix"
        );
        core.advance_active_battle_for_server_tick(40).unwrap();
        assert_eq!(
            before - hp(&core, &targets),
            1059,
            "{mode} no deferred second child"
        );
        assert_eq!(step_count(&core), 4);
        assert_eq!(
            native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
            0
        );
        assert!(!core
            .battle_typed_resources()
            .unwrap()
            .scopes()
            .any(|(_, kind, _, _)| matches!(
                kind,
                crate::game::resource::ResourceLifetime::Delivery
                    | crate::game::resource::ResourceLifetime::Activation
                    | crate::game::resource::ResourceLifetime::Chain
            )));
        println!("RV001 cancellation={mode} original=135 child_ticks=132,264,528 future_child=0 final_loss=1059 casts_scopes_released=true");
    }
}
