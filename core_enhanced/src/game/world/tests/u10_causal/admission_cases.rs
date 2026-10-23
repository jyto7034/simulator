use super::*;
use crate::game::battle::{core::relic_causal_tests as causal_native, runtime_control as native};

#[test]
fn u10_h_actual_fact_subscriber_admission_has_exact_limits_and_atomic_retry() {
    let fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[1])]),
        &relic_input(&damage_subscriber("admitted_echo", 7, 30)),
        &["u10_relic"],
        true,
    );
    let base = fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .clone();
    let before = causal_roots(&base);
    let apply = |core: &mut crate::game::battle::core::BattleCore| {
        causal_native::actual_damage_transaction(core, fixture.actor, fixture.targets[0])
    };
    let mut clean = base.clone();
    let (result, retained) = native::observe_retained_admissions(|| apply(&mut clean));
    result.unwrap();
    assert_eq!(
        base.units[&fixture.targets[0]].stats.current_health
            - clean.units[&fixture.targets[0]].stats.current_health,
        1
    );
    let expected = causal_roots(&clean);
    assert_ne!(
        before, expected,
        "the actual damage consumer committed a fact and child reservation"
    );
    assert!(
        !retained.is_empty(),
        "real producer admission must price retained ownership"
    );
    for resource in ["work", "clone"] {
        let defaults = native::RuntimeFactLimits::default();
        let mut low = 1;
        let mut high = if resource == "work" {
            defaults.max_work
        } else {
            defaults.max_clone_slots as u64
        };
        while low < high {
            let middle = low + (high - low) / 2;
            let mut limits = defaults;
            if resource == "work" {
                limits.max_work = middle;
            } else {
                limits.max_clone_slots = middle as usize;
            }
            let mut trial = base.clone();
            match native::run(limits, || apply(&mut trial)) {
                Ok(()) => {
                    assert_eq!(causal_roots(&trial), expected);
                    high = middle;
                }
                Err(GameError::BattleRuntimeFault(fault))
                    if fault.code
                        == if resource == "work" {
                            native::RuntimeFaultCode::WorkLimit
                        } else {
                            native::RuntimeFaultCode::CloneLimit
                        } =>
                {
                    assert_eq!(
                        causal_roots(&trial),
                        before,
                        "{resource} refusal must not publish HP, ancestry, IDs or queue changes"
                    );
                    low = middle + 1;
                }
                other => panic!("unexpected {resource} boundary result: {other:?}"),
            }
        }
        assert!(high > 1);
        for (limit, accept) in [(high - 1, false), (high, true)] {
            let mut limits = defaults;
            if resource == "work" {
                limits.max_work = limit;
            } else {
                limits.max_clone_slots = limit as usize;
            }
            let mut retry = base.clone();
            let outcome = native::run(limits, || apply(&mut retry));
            if accept {
                outcome.unwrap();
            } else {
                assert!(matches!(outcome, Err(GameError::BattleRuntimeFault(_))));
                assert_eq!(causal_roots(&retry), before);
                apply(&mut retry).unwrap();
            }
            assert_eq!(
                causal_roots(&retry),
                expected,
                "{resource} exact success and failed-then-clean retry must agree"
            );
        }
        println!(
            "U10_H fact_subscriber {resource}_exact={high} one_short={} rollback=true retry=true",
            high - 1
        );
    }
    let (site, _, boundary) = retained.iter().max_by_key(|row| row.2).unwrap();
    let mut retry = base.clone();
    let refused = native::run(
        native::RuntimeFactLimits {
            max_retained_bytes: boundary - 1,
            ..Default::default()
        },
        || apply(&mut retry),
    )
    .unwrap_err();
    assert!(matches!(
        refused,
        GameError::BattleRuntimeFault(native::BattleRuntimeFault {
            code: native::RuntimeFaultCode::RetainedByteLimit,
            ..
        })
    ));
    assert_eq!(causal_roots(&retry), before);
    native::run(
        native::RuntimeFactLimits {
            max_retained_bytes: *boundary,
            ..Default::default()
        },
        || apply(&mut retry),
    )
    .unwrap();
    assert_eq!(causal_roots(&retry), expected);
    println!("U10_H fact_subscriber retained_exact={boundary} site={site} rollback_retry=true");
}

fn world_roots(core: &GameCore) -> serde_json::Value {
    let active = core.state.active_battle.as_ref().unwrap();
    serde_json::json!({
        "battle":causal_roots(&active.battle),
        "execution":format!("{:?}",active.execution),
        "revision":active.battle_state_revision,
        "run":core.get_run_snapshot_json().unwrap(),
        "resources":format!("{:?}",core.battle_typed_resources()),
    })
}

#[test]
fn u10_h_join_final_receipt_and_wedge_commit_preserve_prior_prefix_on_refusal() {
    for case in ["join", "wedge"] {
        let (skills, relics, prefix, finished) = if case == "join" {
            (
                skill_input(&[(0, &[1])]),
                relic_input(&super::join_cases::branch_pair(true)),
                26,
                44,
            )
        } else {
            (
                skill_input(&[(0, &[100]), (50, &[100])]),
                super::wedge_cases::wedge_input(""),
                110,
                230,
            )
        };
        let mut fixture = CausalWorld::prepare(&skills, &relics, &["u10_relic"], true);
        let hp_before = fixture.target_hp();
        fixture.cast();
        fixture.advance(50);
        assert_eq!(hp_before - fixture.target_hp(), prefix);
        let base = fixture.core;
        let before = world_roots(&base);
        let advance = |core: &mut GameCore| core.advance_active_battle_for_server_tick(1);
        let mut clean = base.clone();
        let (result, retained) = native::observe_retained_admissions(|| advance(&mut clean));
        result.unwrap();
        let final_hp: u32 = fixture
            .targets
            .iter()
            .map(|target| {
                clean.state.active_battle.as_ref().unwrap().battle.units[target]
                    .stats
                    .current_health
            })
            .sum();
        assert_eq!(hp_before - final_hp, finished);
        let expected = world_roots(&clean);
        for resource in ["work", "clone"] {
            let defaults = native::RuntimeFactLimits::default();
            let mut low = 1;
            let mut high = if resource == "work" {
                defaults.max_work
            } else {
                defaults.max_clone_slots as u64
            };
            while low < high {
                let middle = low + (high - low) / 2;
                let mut limits = defaults;
                if resource == "work" {
                    limits.max_work = middle;
                } else {
                    limits.max_clone_slots = middle as usize;
                }
                let mut trial = base.clone();
                match native::run(limits, || advance(&mut trial)) {
                    Ok(_) => {
                        assert_eq!(world_roots(&trial), expected);
                        high = middle;
                    }
                    Err(GameError::BattleRuntimeFault(fault))
                        if fault.code
                            == if resource == "work" {
                                native::RuntimeFaultCode::WorkLimit
                            } else {
                                native::RuntimeFaultCode::CloneLimit
                            } =>
                    {
                        assert_eq!(world_roots(&trial),before,
                            "{case} {resource}: failed timestamp must preserve its earlier prefix and all ownership");
                        low = middle + 1;
                    }
                    other => panic!("{case} unexpected {resource} result {other:?}"),
                }
            }
            assert!(high > 1);
            for (limit, accept) in [(high - 1, false), (high, true)] {
                let mut limits = defaults;
                if resource == "work" {
                    limits.max_work = limit;
                } else {
                    limits.max_clone_slots = limit as usize;
                }
                let mut trial = base.clone();
                let result = native::run(limits, || advance(&mut trial));
                if accept {
                    result.unwrap();
                } else {
                    assert!(matches!(result, Err(GameError::BattleRuntimeFault(_))));
                    assert_eq!(world_roots(&trial), before);
                    advance(&mut trial).unwrap();
                }
                assert_eq!(
                    world_roots(&trial),
                    expected,
                    "{case}: retry must neither duplicate a receipt nor advance the strike twice"
                );
            }
            println!("U10_H {case} {resource}_exact={high} prefix={prefix} finished={finished}");
        }
        let (site, _, boundary) = retained.iter().max_by_key(|row| row.2).unwrap();
        let mut retry = base.clone();
        let refused = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: boundary - 1,
                ..Default::default()
            },
            || advance(&mut retry),
        )
        .unwrap_err();
        assert!(matches!(
            refused,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ));
        assert_eq!(world_roots(&retry), before);
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: *boundary,
                ..Default::default()
            },
            || advance(&mut retry),
        )
        .unwrap();
        assert_eq!(world_roots(&retry), expected);
        println!("U10_H {case} retained_exact={boundary} site={site} prefix={prefix} finished={finished}");
    }
}
