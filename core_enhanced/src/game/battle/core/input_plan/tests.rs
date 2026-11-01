use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::{
    primary_fragment_input, enable_complete_trace, hp_trace_rows, assert_fragment_trace_origin,
};
use super::*;
use crate::authoring::composition::{
    canonical_input_plan_tests as canonical, compile_json, Artifact,
};
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{fixture_content_with_data, hit, place},
    sim::{BattleExecutionState, BattleLiveCommand, BattleLiveCommandOutcome},
};
const RELICS: [&str; 6] = [
    "double-harvest",
    "little-red-puncture-wound",
    "deep-imprint",
    "little-red-slug-round",
    "broken-balance",
    "little-red-incendiary-round",
];
fn fixture(
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    fixture_custom(equipped, |_| {})
}
fn fixture_custom(
    equipped: &[usize],
    edit: impl FnOnce(&mut serde_json::Value),
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    fixture_custom_weapon(equipped, edit, None)
}
fn fixture_custom_weapon(
    equipped: &[usize],
    edit: impl FnOnce(&mut serde_json::Value),
    family: Option<&str>,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let (mut c, s, a, e, b) = canonical_fixture(equipped, edit, family);
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
    }
    c.runtime_transaction(|c| {
        c.clear_input_plans(0)?;
        c.clear_shared_cohorts()?;
        c.sustained.input_plans = Default::default();
        c.reconcile_input_plans(0)
    })
    .unwrap();
    place(&mut c, a, 2, 1);
    place(&mut c, e, 4, 1);
    place(&mut c, b, 7, 3);
    (c, s, a, e, b)
}
// Use the current source and actual provider registrations. Callers change only
// explicitly technical values for independent boundary cases.
fn canonical_fixture(
    equipped: &[usize],
    edit: impl FnOnce(&mut serde_json::Value),
    family: Option<&str>,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut document = canonical::resolved();
    document["skills"][0]["id"] = serde_json::json!("r11_aura");
    edit(canonical::program(&mut document));
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    let relics = canonical::relics();
    let indices: Vec<_> = equipped
        .iter()
        .map(|index| {
            relics
                .relics
                .iter()
                .position(|r| r.id == RELICS[*index])
                .unwrap()
        })
        .collect();
    // Use the catalog origin, independent of the program's expected family.
    // The shared legacy graph helper's Concept slug must not mask registration.
    let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
        ron::de::from_str(
            &include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/fragments.ron"
            ))
            .replace("r08_strike", "r11_aura"),
        )
        .unwrap();
    fragments.fragments[0].origin = canonical::catalog().origin;
    let mut unrelated = fragments.fragments[0].clone();
    unrelated.id = "unrelated_fragment".into();
    unrelated.uuid = Uuid::from_u128(0xf116);
    unrelated.origin = Some(
        crate::game::data::skill_fragment_data::SkillFragmentOrigin::Concept {
            concept_id: "unrelated_family".into(),
        },
    );
    fragments.fragments.push(unrelated);
    let abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../game_resources/data/abnormalities/base.ron"
        )))
        .unwrap();
    let mut origin = abnormalities
        .get_by_id("f-01-57_little_red")
        .unwrap()
        .clone();
    // This isolated combat fixture loads the real origin identity, without the
    // encounter's reward/boss-skill dependencies. World tests load the full pack.
    origin.response_complete_skill_fragment_id = None;
    origin.omen_chain_id = None;
    origin.skill_id = None;

    fixture_content_with_data(
        &RELICS,
        &indices,
        native,
        |weapon| weapon.content_family = family.map(str::to_owned),
        crate::game::resource::ResourceCatalog { resources: vec![] },
        |builder| {
            builder
                .with_relics(relics)
                .with_skill_fragments(fragments)
                .with_abnormalities(vec![origin])
        },
        |_| {},
    )
}
fn key(c: &BattleCore, a: UnitInstanceId) -> Uuid {
    c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
}
fn prepare(c: &mut BattleCore, a: UnitInstanceId, now: u64) -> u64 {
    let outcome = c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: now,
        })
        .unwrap();
    let BattleLiveCommandOutcome::InputPlanPrepared { preparation, .. } = outcome else {
        panic!("{outcome:?}")
    };
    preparation
}
fn input(id: u64, tiles: &[(ShotKind, i32, i32)]) -> ShotCommitInput {
    ShotCommitInput {
        preparation: id,
        request: 77,
        layout: ShotLayoutInput::Fresh(
            tiles
                .iter()
                .map(|(kind, x, y)| ShotPlacement {
                    kind: *kind,
                    tile: Position::new(*x, *y),
                })
                .collect(),
        ),
        aim: ShotAimInput::Precise(tiles.iter().map(|_| ShotDirection { x: 1, y: 0 }).collect()),
    }
}
#[test]
fn input_plan_runtime_prepare_freezes_time_and_commit_releases_once_from_selected_origin() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let copy = key(&c, a);
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: 0
        })
        .is_err());
    c.step_battle_execution_until(&mut s, 1000).unwrap();
    assert_eq!(c.sustained.input_plans.owners[&copy].normal, 1);
    let id = prepare(&mut c, a, 1000);
    let position = c.units[&a].body.position;
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    assert_eq!(s.last_event_time_ms(), 1000);
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert_eq!(c.units[&a].resonance_current, 100);
    let plan = input(id, &[(ShotKind::Normal, 1, 1)]);
    let outcome = c
        .apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: plan.clone(),
            time_ms: 1000,
        })
        .unwrap();
    assert!(matches!(
        outcome,
        BattleLiveCommandOutcome::InputPlanCommitted { .. }
    ));
    assert!(!c.input_plan_paused());
    assert_eq!(c.sustained.input_plans.flights.len(), 1);
    assert_eq!(c.units[&a].resonance_current, 0);
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert_eq!(c.units[&a].body.position, position);
    let snapshot = format!("{:?}", c.input_plans_snapshot().unwrap());
    let retry = c
        .apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: plan,
            time_ms: 1000,
        })
        .unwrap();
    assert_eq!(retry, outcome);
    assert_eq!(snapshot, format!("{:?}", c.input_plans_snapshot().unwrap()));
    c.step_battle_execution_until(&mut s, 1600).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 9900);
    c.step_battle_execution_until(&mut s, 3200).unwrap();
    let view = c.input_plans_snapshot().unwrap();
    assert!(view.active_projectiles.is_empty());
    assert_eq!(view.settled_projectiles.len(), 1);
    assert_eq!(view.settled_projectiles[0].hits.len(), 1);
}
#[test]
fn input_plan_runtime_invalid_whole_plan_preserves_stock_resonance_memory_and_pause() {
    let (mut c, mut s, a, _, _) = fixture(&[]);
    let copy = key(&c, a);
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    let id = prepare(&mut c, a, 2000);
    let before = format!(
        "{:?}{:?}",
        c.input_plans_snapshot().unwrap(),
        c.runtime_fact_snapshot().unwrap()
    );
    for mut plan in [
        input(id, &[(ShotKind::Normal, 1, 1)]),
        input(id, &[(ShotKind::Normal, 1, 1), (ShotKind::Normal, 1, 1)]),
        input(id, &[(ShotKind::Enhanced, 1, 1), (ShotKind::Normal, 2, 1)]),
        input(id, &[(ShotKind::Normal, 1, 1), (ShotKind::Normal, 9, 1)]),
    ] {
        assert!(c
            .apply_live_command(BattleLiveCommand::CommitInputPlan {
                unit_id: a,
                input: plan.clone(),
                time_ms: 2000
            })
            .is_err());
        plan.layout = ShotLayoutInput::Remembered;
        assert!(c
            .apply_live_command(BattleLiveCommand::CommitInputPlan {
                unit_id: a,
                input: plan,
                time_ms: 2000
            })
            .is_err());
        assert_eq!(
            before,
            format!(
                "{:?}{:?}",
                c.input_plans_snapshot().unwrap(),
                c.runtime_fact_snapshot().unwrap()
            )
        );
    }
    let mut zero = input(id, &[(ShotKind::Normal, 1, 1), (ShotKind::Normal, 2, 1)]);
    zero.aim = ShotAimInput::Focused(Position::new(1, 1));
    assert!(c
        .apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: zero,
            time_ms: 2000
        })
        .is_err());
    assert_eq!(c.sustained.input_plans.owners[&copy].normal, 2);
    assert_eq!(c.units[&a].resonance_current, 100);
    c.apply_live_command(BattleLiveCommand::CancelInputPlan {
        unit_id: a,
        preparation: id,
        disconnected: false,
        time_ms: 2000,
    })
    .unwrap();
    assert!(!c.input_plan_paused());
    let id = prepare(&mut c, a, 2000);
    c.apply_live_command(BattleLiveCommand::CancelInputPlan {
        unit_id: a,
        preparation: id,
        disconnected: true,
        time_ms: 2000,
    })
    .unwrap();
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    assert_eq!(s.last_event_time_ms(), 2000);
    assert!(c.input_plan_paused());
    c.resume_disconnected_input_plan();
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    assert_eq!(c.sustained.input_plans.owners[&copy].normal, 3);
}

fn initial(program: &mut serde_json::Value, normal: u32, enhanced: u32) {
    program["normal"]["initial"] = serde_json::json!(normal);
    program["enhanced"]["initial"] = serde_json::json!(enhanced);
}
fn commit(c: &mut BattleCore, a: UnitInstanceId, now: u64, tiles: &[(ShotKind, i32, i32)]) {
    let id = prepare(c, a, now);
    c.apply_live_command(BattleLiveCommand::CommitInputPlan {
        unit_id: a,
        input: input(id, tiles),
        time_ms: now,
    })
    .unwrap();
}
#[test]
fn input_plan_runtime_wounds_use_old_stack_and_same_time_stable_order_with_explicit_prerequisite() {
    for (equipped, expected_loss, stacks) in
        [(vec![1], 230, 2), (vec![1, 2], 260, 4), (vec![2], 200, 0)]
    {
        let (mut c, mut s, a, e, _) = fixture_custom(&equipped, |d| initial(d, 2, 0));
        let id = prepare(&mut c, a, 0);
        let mut plan = input(id, &[(ShotKind::Normal, 1, 0), (ShotKind::Normal, 1, 2)]);
        plan.aim = ShotAimInput::Focused(Position::new(4, 1));
        c.apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: plan,
            time_ms: 0,
        })
        .unwrap();
        c.step_battle_execution_until(&mut s, 700).unwrap();
        assert_eq!(10000 - c.units[&e].stats.current_health, expected_loss);
        let view = c.input_plans_snapshot().unwrap();
        assert_eq!(view.wounds.iter().map(|w| w.3).sum::<u32>(), stacks);
        let hits: Vec<_> = c
            .sustained
            .input_plans
            .flights
            .values()
            .flat_map(|f| &f.hit_results)
            .collect();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].time_ms, hits[1].time_ms);
        c.step_battle_execution_until(&mut s, 1800).unwrap();
        assert!(c.input_plans_snapshot().unwrap().wounds.is_empty());
    }
}
#[test]
fn input_plan_runtime_zero_hp_damage_is_contact_and_stun_immunity_does_not_create_vulnerability() {
    use crate::game::battle::types::UnitTargetTrait;
    let (mut c, mut s, a, e, _) = fixture_custom(&[1, 3, 4], |d| initial(d, 1, 0));
    c.units
        .get_mut(&e)
        .unwrap()
        .incoming_damage_modifiers
        .damage_reduction_percent = 100;
    c.units
        .get_mut(&e)
        .unwrap()
        .target_traits
        .push(UnitTargetTrait::StunImmune);
    commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
    c.step_battle_execution_until(&mut s, 700).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert_eq!(c.input_plans_snapshot().unwrap().wounds[0].3, 1);
    assert_eq!(c.hit_stun_until(e, Some(700)), None);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
}
#[test]
fn input_plan_runtime_released_enhanced_hit_survives_caster_withdrawal_and_keeps_root_and_slug_order(
) {
    let (mut c, mut s, a, e, _) = fixture_custom(&[3, 4], |d| initial(d, 0, 1));
    let from = c.units[&e].body.position;
    commit(&mut c, a, 0, &[(ShotKind::Enhanced, 1, 1)]);
    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: a,
        time_ms: 0,
    })
    .unwrap();
    c.step_battle_execution_until(&mut s, 700).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 9850);
    assert!(c.rooted(e));
    assert!(c.hit_stun_until(e, Some(700)).is_some());
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 10);
    let statuses = &c
        .sustained
        .input_plans
        .flights
        .values()
        .next()
        .unwrap()
        .hit_results[0]
        .statuses;
    assert_eq!(statuses.len(), 2);
    assert!(statuses.iter().all(|s| s.applied));
    assert!(c.units[&e].body.position.x > from.x);
    c.step_battle_execution_until(&mut s, 2700).unwrap();
    assert!(!c.rooted(e));
    assert_eq!(c.hit_stun_until(e, Some(2700)), None);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 10);
}
#[test]
fn input_plan_runtime_disconnect_allows_fresh_prepare_and_memory_reuse_requires_exact_stock() {
    let (mut c, mut s, a, _, _) = fixture_custom(&[], |d| initial(d, 1, 0));
    let id = prepare(&mut c, a, 0);
    c.apply_live_command(BattleLiveCommand::CancelInputPlan {
        unit_id: a,
        preparation: id,
        disconnected: true,
        time_ms: 0,
    })
    .unwrap();
    let next = prepare(&mut c, a, 0);
    assert_ne!(id, next);
    assert!(c.input_plan_paused());
    let first = input(next, &[(ShotKind::Normal, 1, 1)]);
    c.apply_live_command(BattleLiveCommand::CommitInputPlan {
        unit_id: a,
        input: first.clone(),
        time_ms: 0,
    })
    .unwrap();
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    c.units.get_mut(&a).unwrap().resonance_current = 100;
    let id = prepare(&mut c, a, 3000);
    let mut remembered = input(id, &[(ShotKind::Normal, 1, 1)]);
    remembered.layout = ShotLayoutInput::Remembered;
    assert!(c
        .apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: remembered,
            time_ms: 3000
        })
        .is_err());
    assert!(
        !c.input_plans_snapshot()
            .unwrap()
            .preparation
            .unwrap()
            .remembered_layout_valid
    );
    // An old receipt at the current clock must not end this newer preparation.
    let replay = c
        .apply_live_command(BattleLiveCommand::CommitInputPlan {
            unit_id: a,
            input: first,
            time_ms: 3000,
        })
        .unwrap();
    assert!(matches!(
        replay,
        BattleLiveCommandOutcome::InputPlanCommitted { .. }
    ));
    assert_eq!(c.input_plan_preparation_id(), Some((a, id)));
    assert!(c.input_plan_paused());
}
#[test]
fn input_plan_runtime_cohort_is_hostile_and_kill_credit_belongs_to_original_copy_after_withdrawal()
{
    for equipped in [vec![], vec![0]] {
        let (mut c, mut s, a, _, b) = fixture(&equipped);
        let copy = key(&c, a);
        c.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 0,
        })
        .unwrap();
        c.step_battle_execution_until(&mut s, 5000).unwrap();
        assert_eq!(c.sustained.input_plans.owners[&copy].normal, 5);
        let wolf = *c.sustained.shared_cohorts.members.keys().next().unwrap();
        assert_eq!(c.units[&wolf].owner, Side::Opponent);
        assert_eq!(c.units[&wolf].stats.current_health, 300);
        place(&mut c, b, 1, 1);
        c.units.get_mut(&b).unwrap().block_capacity = 1;
        c.step_battle_execution_until(&mut s, 7000).unwrap();
        assert!(c.units[&b].stats.current_health < 10000);
        hit(&mut c, b, wolf, 1000, 7000);
        assert!(c.units[&wolf].lifecycle.is_dead());
        assert_eq!(
            c.sustained.input_plans.owners[&copy].enhanced,
            if equipped.is_empty() { 1 } else { 2 }
        );
        assert!(!c.sustained.shared_cohorts.members.contains_key(&wolf));
    }
}
#[test]
fn input_plan_runtime_cohort_escape_and_nonallied_death_do_not_grant_stock() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let copy = key(&c, a);
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    let first = *c.sustained.shared_cohorts.members.keys().next().unwrap();
    hit(&mut c, e, first, 1000, 5000);
    assert_eq!(c.sustained.input_plans.owners[&copy].enhanced, 0);
    c.step_battle_execution_until(&mut s, 10000).unwrap();
    let second = *c.sustained.shared_cohorts.members.keys().next().unwrap();
    let route = c.units[&second].enemy_movement_plan.clone();
    place(&mut c, second, 7, 1);
    c.units.get_mut(&second).unwrap().enemy_movement_plan = route;
    c.runtime_transaction(|c| c.settle_cohort_escapes(10000))
        .unwrap();
    assert!(!c.units[&second].is_active());
    assert!(!c.units[&second].lifecycle.is_dead());
    assert!(c.sustained.shared_cohorts.members.is_empty());
    assert_eq!(c.sustained.input_plans.owners[&copy].enhanced, 0);
}
#[test]
fn input_plan_runtime_explosion_has_separate_relic_source_and_last_hit_does_not_explode() {
    for hits in [1, 3] {
        let (mut c, mut s, a, e, _) = fixture_custom(&[5], |d| {
            initial(d, 1, 0);
            for kind in ["normal_projectile", "enhanced_projectile"] {
                d[kind]["max_hits"] = serde_json::json!(hits);
            }
        });
        commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
        c.step_battle_execution_until(&mut s, 700).unwrap();
        assert_eq!(
            c.units[&e].stats.current_health,
            if hits == 1 { 9900 } else { 9800 }
        );
        let facts = c.runtime_fact_snapshot().unwrap();
        let text = format!("{facts:?}");
        assert_eq!(text.contains("little-red-incendiary-round"), hits > 1);
        assert!(c.input_plans_snapshot().unwrap().wounds.is_empty());
    }
}

#[test]
fn input_plan_runtime_cancellation_restores_preparation_cost_memory_damage_and_stock() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "input_plan_prepared",
        "shot_volley_cost_and_memory",
        "input_plan_commit_complete",
        "shot_direct_damage",
        "input_plan_stock_grant",
        "input_plan_hostile_spawn",
    ] {
        let (mut c, mut s, a, _, _) = fixture_custom(&[], |d| initial(d, 1, 0));
        let prepare_only = site == "input_plan_prepared";
        let committing = matches!(
            site,
            "shot_volley_cost_and_memory" | "input_plan_commit_complete"
        );
        let mut plan = None;
        let now = if prepare_only || committing {
            0
        } else if site == "input_plan_stock_grant" {
            1000
        } else if site == "input_plan_hostile_spawn" {
            5000
        } else {
            480
        };
        if committing {
            let id = prepare(&mut c, a, 0);
            plan = Some(input(id, &[(ShotKind::Normal, 1, 1)]));
        } else if site == "shot_direct_damage" {
            commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
        }
        if now > 0 {
            c.step_battle_execution_until(&mut s, now - 1).unwrap();
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            c.input_plans_snapshot().unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    if prepare_only {
                        c.apply_live_command(BattleLiveCommand::ActivateSkill {
                            unit_id: a,
                            skill_id: "r11_aura".into(),
                            target: None,
                            time_ms: now,
                        })
                        .map(|_| ())
                    } else if committing {
                        c.apply_live_command(BattleLiveCommand::CommitInputPlan {
                            unit_id: a,
                            input: plan.clone().unwrap(),
                            time_ms: now,
                        })
                        .map(|_| ())
                    } else {
                        c.step_battle_execution_until(&mut s, now).map(|_| ())
                    }
                },
            )
        });
        assert!(
            token.is_cancelled() && result.is_err(),
            "{site}: {result:?}"
        );
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                c.input_plans_snapshot().unwrap(),
                format!("{s:?}")
            ),
            "{site}"
        );
        if prepare_only {
            prepare(&mut c, a, now);
        } else if committing {
            c.apply_live_command(BattleLiveCommand::CommitInputPlan {
                unit_id: a,
                input: plan.unwrap(),
                time_ms: now,
            })
            .unwrap();
        } else {
            c.step_battle_execution_until(&mut s, now).unwrap();
        }
    }
}
#[test]
fn input_plan_runtime_obstacle_air_and_battle_end_have_independent_finite_results() {
    for obstacle in [false, true] {
        let (mut c, mut s, a, e, _) = fixture_custom(&[], |d| initial(d, 1, 0));
        c.units.get_mut(&e).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        if obstacle {
            Arc::make_mut(&mut c.scenario)
                .battlefield
                .obstacles
                .push(Position::new(3, 1));
        }
        commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
        c.step_battle_execution_until(&mut s, 700).unwrap();
        assert_eq!(
            c.units[&e].stats.current_health,
            if obstacle { 10000 } else { 9900 }
        );
        let copied = c.clone();
        assert_eq!(
            c.input_plans_snapshot().unwrap(),
            copied.input_plans_snapshot().unwrap()
        );
        let snapshot = c.input_plans_snapshot().unwrap();
        assert_eq!(
            serde_json::from_str::<InputPlansSnapshot>(&serde_json::to_string(&snapshot).unwrap())
                .unwrap(),
            snapshot
        );
        if obstacle {
            assert_eq!(snapshot.settled_projectiles[0].end, ShotEnd::Obstacle);
        } else {
            c.runtime_transaction(|c| c.clear_input_plans(700)).unwrap();
            assert_eq!(
                c.input_plans_snapshot().unwrap().settled_projectiles[0].end,
                ShotEnd::BattleEnded
            );
        }
    }
}

#[test]
fn input_plan_runtime_family_weakness_uses_actual_fragment_or_weapon_source_and_not_wearer() {
    use super::super::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy};
    use crate::game::battle::damage::{DamageSource, DamageType};
    for family in [None, Some("little-red"), Some("f-01-57_little_red")] {
        let (mut c, mut s, a, _, _) = fixture_custom_weapon(&[], |_| {}, family);
        c.step_battle_execution_until(&mut s, 5000).unwrap();
        let wolf = *c.sustained.shared_cohorts.members.keys().next().unwrap();
        let equipment = c
            .items
            .values()
            .find(|item| item.owner_unit_instance == a)
            .unwrap()
            .source_snapshot();
        let fragment = c.units[&a].fragment_source.clone();
        for mode in 0..4 {
            let before = c.units[&wolf].stats.current_health;
            c.runtime_transaction(|c| {
                let mut source = c
                    .damage_source_snapshot_for_unit(
                        a,
                        wolf,
                        DamageSource::Ability,
                        DamageType::True,
                        20,
                        Default::default(),
                        0,
                        5000,
                        false,
                    )
                    .unwrap();
                source.fact_source = RuntimeFactSource::default();
                source.fact_source.actor = Some(a);
                if mode == 1 {
                    source.fact_source.fragment = fragment.clone();
                }
                if mode == 2 {
                    source.fact_source.equipment = Some(equipment);
                }
                if mode == 3 {
                    let mut unrelated = fragment.as_ref().unwrap().as_ref().clone();
                    unrelated.definition_id = "unrelated_fragment".into();
                    source.fact_source.fragment = Some(Arc::new(unrelated));
                }
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: wolf,
                    source_snapshot: source,
                    committed_at_ms: 5000,
                    policy: CommittedDamagePolicy::GeneralCommand,
                });
                control::poll("family_damage_test")
            })
            .unwrap();
            assert_eq!(
                before - c.units[&wolf].stats.current_health,
                if mode == 1 || mode == 2 && family == Some("f-01-57_little_red") {
                    30
                } else {
                    20
                }
            );
        }
        let copy = key(&c, a);
        c.runtime_transaction(|c| {
            let source = c
                .damage_source_snapshot_for_unit(
                    a,
                    wolf,
                    DamageSource::Ability,
                    DamageType::True,
                    1000,
                    Default::default(),
                    0,
                    5000,
                    false,
                )
                .unwrap();
            c.units.get_mut(&a).unwrap().owner = Side::Opponent;
            c.execute_committed_damage(CommittedDamageInput {
                target_id: wolf,
                source_snapshot: source,
                committed_at_ms: 5000,
                policy: CommittedDamagePolicy::GeneralCommand,
            });
            control::poll("cohort_released_side_test")
        })
        .unwrap();
        assert_eq!(c.sustained.input_plans.owners[&copy].enhanced, 1);
    }
}

#[test]
fn input_plan_runtime_current_cohort_blocks_victory_but_equal_future_spawn_cannot_delay_a_win() {
    let (mut c, mut s, a, e, b) = fixture(&[]);
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_required_group_outcomes();
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    let wolf = *c.sustained.shared_cohorts.members.keys().next().unwrap();
    hit(&mut c, a, e, 20000, 5000);
    c.step_battle_execution_until(&mut s, 5001).unwrap();
    assert!(!s.is_finished());
    hit(&mut c, b, wolf, 1000, 5001);
    c.step_battle_execution_until(&mut s, 5002).unwrap();
    assert!(s.is_finished());
    let (mut c, mut s, a, e, _) = fixture_custom(&[], |d| {
        initial(d, 1, 0);
        d["cohort"]["interval_ms"] = serde_json::json!(480);
    });
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_required_group_outcomes();
    c.units.get_mut(&e).unwrap().stats.current_health = 100;
    commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
    c.step_battle_execution_until(&mut s, 480).unwrap();
    assert!(s.is_finished());
    assert!(c.sustained.shared_cohorts.members.is_empty());
}
#[test]
fn input_plan_runtime_authored_block_demand_and_escape_damage_are_real_combat_effects() {
    let (mut c, mut s, a, _, b) = fixture_custom(&[], |d| {
        d["cohort"]["block"] = serde_json::json!(2);
        d["cohort"]["leak"] = serde_json::json!(7);
    });
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    let wolf = *c.sustained.shared_cohorts.members.keys().next().unwrap();
    let route = c.units[&wolf].enemy_movement_plan.clone();
    place(&mut c, wolf, 1, 1);
    c.units.get_mut(&wolf).unwrap().enemy_movement_plan = route.clone();
    place(&mut c, b, 1, 1);
    c.units.get_mut(&b).unwrap().block_capacity = 1;
    c.refresh_block_state();
    assert_eq!(c.blocked_by(wolf), None);
    c.units.get_mut(&b).unwrap().block_capacity = 2;
    c.refresh_block_state();
    assert_eq!(c.blocked_by(wolf), Some(b));
    c.units.get_mut(&b).unwrap().role = crate::game::battle::types::BattleUnitRole::DefenseObject;
    place(&mut c, b, 7, 3);
    place(&mut c, wolf, 7, 1);
    c.units.get_mut(&wolf).unwrap().enemy_movement_plan = route;
    c.refresh_block_state();
    c.runtime_transaction(|c| c.settle_cohort_escapes(5000))
        .unwrap();
    assert_eq!(c.units[&b].stats.current_health, 9993);
    assert_eq!(c.sustained.input_plans.owners[&key(&c, a)].enhanced, 0);
}

#[test]
fn input_plan_runtime_successful_memory_reuses_origins_with_new_aim() {
    let (mut c, mut s, a, _, _) = fixture_custom(&[], |d| {
        initial(d, 1, 0);
        for kind in ["normal_projectile", "enhanced_projectile"] {
            d[kind]["range"] = serde_json::json!({"numerator":2,"denominator":1});
        }
    });
    let copy = key(&c, a);
    commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
    c.step_battle_execution_until(&mut s, 1000).unwrap();
    assert!(c.sustained.input_plans.flights.is_empty());
    assert_eq!(c.sustained.input_plans.owners[&copy].normal, 1);
    c.units.get_mut(&a).unwrap().resonance_current = 100;
    let id = prepare(&mut c, a, 1000);
    let mut plan = input(id, &[(ShotKind::Normal, 1, 1)]);
    plan.layout = ShotLayoutInput::Remembered;
    plan.aim = ShotAimInput::Precise(vec![ShotDirection { x: 0, y: 1 }]);
    c.apply_live_command(BattleLiveCommand::CommitInputPlan {
        unit_id: a,
        input: plan,
        time_ms: 1000,
    })
    .unwrap();
    let flight = c.sustained.input_plans.flights.values().next().unwrap();
    assert_eq!(
        flight.start,
        WorldVec2::from_tile_center(Position::new(1, 1))
    );
    assert_eq!(flight.direction, WorldVec2::new(0.0, 1.0));
    assert_eq!(c.sustained.input_plans.owners[&copy].normal, 0);
    c.step_battle_execution_until(&mut s, 1500).unwrap();
    assert!(c.sustained.input_plans.flights.is_empty());
    assert_eq!(c.sustained.input_plans.settled.len(), 2);
}

#[test]
fn input_plan_runtime_bounded_stock_caps_and_unbounded_overflow_rolls_back() {
    for bounded in [true, false] {
        let (mut c, _, a, _, _) = fixture_custom(&[], |d| {
            for kind in ["normal", "enhanced"] {
                d[kind]["initial"] = serde_json::json!(u32::MAX);
                d[kind]["capacity"] = if bounded {
                    serde_json::json!(u32::MAX)
                } else {
                    serde_json::Value::Null
                };
            }
        });
        let copy = key(&c, a);
        let before = format!("{:?}", c.sustained.input_plans);
        let result = c.runtime_transaction(|c| c.shot_stock_tick(copy, 1000));
        if bounded {
            assert!(result.is_ok());
            assert_eq!(c.sustained.input_plans.owners[&copy].normal, u32::MAX);
        } else {
            assert!(result.is_err());
            assert_eq!(format!("{:?}", c.sustained.input_plans), before);
        }
    }
}

#[test]
fn input_plan_runtime_two_equipped_copies_keep_stock_memory_and_hostile_reward_separate() {
    let (mut c, mut s, a, e, b) = canonical_fixture(&[0], |p| initial(p, 1, 0), None);
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
    }
    c.runtime_transaction(|c| {
        c.clear_input_plans(0)?;
        c.clear_shared_cohorts()?;
        c.sustained.input_plans = Default::default();
        c.reconcile_input_plans(0)
    })
    .unwrap();
    let ka = key(&c, a);
    let kb = key(&c, b);
    assert_ne!(ka, kb);
    commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
    assert_eq!(c.sustained.input_plans.owners[&ka].normal, 0);
    assert_eq!(c.sustained.input_plans.owners[&kb].normal, 1);
    assert_eq!(c.sustained.input_plans.owners[&ka].memory.len(), 1);
    assert!(c.sustained.input_plans.owners[&kb].memory.is_empty());
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    let ca = *c
        .sustained
        .shared_cohorts
        .members
        .iter()
        .find(|(_, copy)| copy.fragment == Some(ka))
        .unwrap()
        .0;
    let cb = *c
        .sustained
        .shared_cohorts
        .members
        .iter()
        .find(|(_, copy)| copy.fragment == Some(kb))
        .unwrap()
        .0;
    hit(&mut c, b, ca, 1000, 5000);
    assert_eq!(c.sustained.input_plans.owners[&ka].enhanced, 2);
    assert_eq!(c.sustained.input_plans.owners[&kb].enhanced, 0);
    hit(&mut c, a, cb, 1000, 5000);
    assert_eq!(c.sustained.input_plans.owners[&ka].enhanced, 2);
    assert_eq!(c.sustained.input_plans.owners[&kb].enhanced, 1);
}

#[test]
fn input_plan_runtime_ground_only_projectile_passes_airborne_without_damage() {
    let (mut c, mut s, a, e, _) = fixture_custom(&[], |d| {
        initial(d, 1, 0);
        for kind in ["normal_projectile", "enhanced_projectile"] {
            d[kind]["air"] = serde_json::json!(false);
        }
    });
    c.units.get_mut(&e).unwrap().mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
    commit(&mut c, a, 0, &[(ShotKind::Normal, 1, 1)]);
    c.step_battle_execution_until(&mut s, 700).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert!(c
        .input_plans_snapshot()
        .unwrap()
        .settled_projectiles
        .iter()
        .all(|f| f.hits.is_empty()));
}

#[test]
fn canonical_input_plan_direct_hit_origin_preserves_observer_parity_after_withdrawal() {
    for enhanced in [false, true] {
        for withdrawn in [false, true] {
            let (mut plain, mut plain_state, a, e, _) = fixture_custom(&[], |p| {
                initial(p, u32::from(!enhanced), u32::from(enhanced));
            });
            let copy = primary_fragment_input(&plain, a);
            let mut observed = plain.clone();
            let mut observed_state = plain_state.clone();
            let (_trace, trace_from) = enable_complete_trace(&mut observed);
            for (core, state) in [
                (&mut plain, &mut plain_state),
                (&mut observed, &mut observed_state),
            ] {
                commit(
                    core,
                    a,
                    0,
                    &[(
                        if enhanced {
                            ShotKind::Enhanced
                        } else {
                            ShotKind::Normal
                        },
                        1,
                        1,
                    )],
                );
                if withdrawn {
                    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 0,
                    })
                    .unwrap();
                }
                core.step_battle_execution_until(state, 700).unwrap();
                assert_eq!(
                    core.units[&e].stats.current_health,
                    if enhanced { 9850 } else { 9900 }
                );
            }
            assert_eq!(
                plain.units[&e].stats.current_health,
                observed.units[&e].stats.current_health
            );
            assert_eq!(
                plain.input_plans_snapshot().unwrap(),
                observed.input_plans_snapshot().unwrap()
            );
            assert_eq!(
                plain.event_queue_debug_snapshot(),
                observed.event_queue_debug_snapshot()
            );
            assert_eq!(
                plain.event_log.to_json_string().unwrap(),
                observed.event_log.to_json_string().unwrap()
            );
            assert_eq!(
                serde_json::to_value(plain.runtime_fact_summary()).unwrap(),
                serde_json::to_value(observed.runtime_fact_summary()).unwrap()
            );
            let observation = observed.authoring_observation().unwrap().unwrap();
            let hp = observation.latest_nonzero_sourced_hp.as_ref().unwrap();
            assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, copy);
            assert_eq!(hp.source.as_ref().unwrap().cast.skill_id, "r11_aura");
            assert!(!hp.source.as_ref().unwrap().authored.source_file.is_empty());
            let rows = hp_trace_rows(&observed, trace_from);
            assert_eq!(rows.len(), 1);
            assert_fragment_trace_origin(&rows[0], a, Some(("r11_aura", 0)));
            assert_eq!(observation.nonzero_hp_applications, observed.event_log.entries.iter().filter(|entry| matches!(entry.event, crate::game::battle::event_log::BattleLogEvent::HpChanged{delta,..} if delta != 0)).count() as u64);
        }
    }
}

#[test]
fn canonical_input_plan_hit_observation_cancellation_restores_and_retries() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["sustained_observation_source", "shot_direct_damage"] {
        let (mut core, mut state, a, e, _) = fixture_custom(&[], |p| initial(p, 1, 0));
        let copy = key(&core, a);
        core.enable_authoring_observation();
        commit(&mut core, a, 0, &[(ShotKind::Normal, 1, 1)]);
        core.step_battle_execution_until(&mut state, 479).unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, 480),
            )
        });
        assert!(token.is_cancelled(), "{site}: {result:?}");
        assert!(result.is_err());
        assert_eq!(
            before,
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                format!("{state:?}")
            )
        );
        core.step_battle_execution_until(&mut state, 700).unwrap();
        assert_eq!(core.units[&e].stats.current_health, 9900);
        assert_eq!(
            core.authoring_observation()
                .unwrap()
                .unwrap()
                .latest_nonzero_sourced_hp
                .as_ref()
                .unwrap()
                .fragment
                .as_ref()
                .unwrap()
                .copy_uuid,
            copy
        );
    }
}

#[test]
fn canonical_input_plan_registered_fragment_shot_hits_real_cohort_with_family_bonus() {
    let (mut core, mut state, actor, enemy, _) = fixture_custom(&[], |p| {
        p["normal"]["capacity"] = serde_json::json!(1);
    });
    place(&mut core, enemy, 7, 3);
    core.step_battle_execution_until(&mut state, 5000).unwrap();
    let wolf = *core.sustained.shared_cohorts.members.keys().next().unwrap();
    assert_eq!(core.units[&wolf].owner, Side::Opponent);
    assert_eq!(core.units[&wolf].stats.current_health, 300);
    place(&mut core, wolf, 4, 1);
    core.enable_authoring_observation();
    commit(&mut core, actor, 5000, &[(ShotKind::Normal, 1, 1)]);
    core.step_battle_execution_until(&mut state, 6000).unwrap();
    assert_eq!(core.units[&wolf].stats.current_health, 150);
    let observation = core.authoring_observation().unwrap().unwrap();
    let hp = observation.latest_nonzero_sourced_hp.as_ref().unwrap();
    assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, key(&core, actor));
    assert_eq!(hp.source.as_ref().unwrap().cast.skill_id, "r11_aura");
}

#[path = "canonical_equipment_tests.rs"]
mod canonical_equipment;
