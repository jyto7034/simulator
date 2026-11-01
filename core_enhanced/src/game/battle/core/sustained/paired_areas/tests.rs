use super::*;
use crate::authoring::composition::{canonical_pair_tests as source, compile_json, Artifact};
use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::{
    assert_fragment_trace_origin, enable_complete_trace, hp_trace_rows, primary_fragment_input,
};
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{fixture_content_with_data, place},
    sim::{BattleExecutionState, BattleLiveCommand},
};
use serde_json::json;
const RELICS: [&str; 6] = [
    "long-breath",
    "widened-horizon",
    "healing-increase",
    "vivid-blessing",
    "lingering-warmth",
    "yin-yang-dragon",
];
fn fixture(
    relics: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut document = source::resolved();
    document["skills"][0]["id"] = json!("r11_aura");
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    let catalog = source::relics();
    let equipped: Vec<_> = relics
        .iter()
        .map(|i| {
            catalog
                .relics
                .iter()
                .position(|r| r.id == RELICS[*i])
                .unwrap()
        })
        .collect();
    let (mut c, s, a, e, b) = fixture_content_with_data(
        &RELICS,
        &equipped,
        native,
        |_| {},
        crate::game::resource::ResourceCatalog { resources: vec![] },
        |data| data.with_relics(catalog),
        |_| {},
    );
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for t in [a, e, b] {
        let u = c.units.get_mut(&t).unwrap();
        u.threat_class = BattleUnitThreatClass::Normal;
        u.stats.max_health = 1000;
        u.stats.current_health = 1000;
    }
    place(&mut c, a, 2, 1);
    place(&mut c, b, 4, 1);
    place(&mut c, e, 7, 3);
    (c, s, a, e, b)
}
fn cast(c: &mut BattleCore, a: UnitInstanceId, first: Position, second: Position, now: u64) {
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        target: Some(SkillCastTarget::TilePair { first, second }),
        time_ms: now,
    })
    .unwrap();
}
fn step(c: &mut BattleCore, s: &mut BattleExecutionState, now: u64) {
    c.step_battle_execution_until(s, now).unwrap();
}

fn provenance_fixture(
    burst: bool,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let (mut c, s, a, e, b) = fixture(if burst { &[5] } else { &[] });
    c.units.get_mut(&b).unwrap().stats.current_health = 500;
    place(&mut c, e, 4, 1);
    if burst {
        c.units.get_mut(&e).unwrap().threat_class = BattleUnitThreatClass::Boss;
        c.units.get_mut(&e).unwrap().stats.max_health = 1001;
        c.units.get_mut(&e).unwrap().stats.current_health = 1001;
        place(&mut c, b, 7, 3);
    } else {
        place(&mut c, b, 4, 1);
        place(&mut c, e, 7, 3);
    }
    (c, s, a, e, b)
}

#[test]
fn canonical_pair_heal_and_dragon_provenance_preserve_observer_parity() {
    for burst in [false, true] {
        let (mut plain, mut ps, a, e, b) = provenance_fixture(burst);
        let copy = primary_fragment_input(&plain, a);
        let (mut observed, mut os) = (plain.clone(), ps.clone());
        let (_trace, trace_from) = enable_complete_trace(&mut observed);
        for (core, state) in [(&mut plain, &mut ps), (&mut observed, &mut os)] {
            cast(
                core,
                a,
                Position::new(if burst { 4 } else { 2 }, 1),
                Position::new(4, 1),
                0,
            );
            step(core, state, 1000);
            assert_eq!(
                core.units[&e].stats.current_health,
                if burst { 750 } else { 1000 }
            );
            assert_eq!(
                core.units[&b].stats.current_health,
                if burst { 500 } else { 650 }
            );
        }
        assert_eq!(
            plain.event_log.to_json_string().unwrap(),
            observed.event_log.to_json_string().unwrap()
        );
        assert_eq!(
            plain.event_queue_debug_snapshot(),
            observed.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(plain.runtime_fact_summary()).unwrap(),
            serde_json::to_value(observed.runtime_fact_summary()).unwrap()
        );
        assert_eq!(
            plain.paired_areas_snapshot().unwrap(),
            observed.paired_areas_snapshot().unwrap()
        );
        let obs = observed.authoring_observation().unwrap().unwrap();
        let hp = obs.latest_nonzero_sourced_hp.as_ref().unwrap();
        assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, copy);
        assert_eq!(hp.source.as_ref().unwrap().cast.skill_id, "r11_aura");
        assert!(!hp.source.as_ref().unwrap().authored.source_file.is_empty());
        for row in hp_trace_rows(&observed, trace_from) {
            assert_fragment_trace_origin(&row, a, Some(("r11_aura", 0)));
        }
        let events = observed
            .event_log
            .entries
            .iter()
            .filter(|entry| {
                matches!(entry.event,
            crate::game::battle::event_log::BattleLogEvent::HpChanged { delta, .. } if delta != 0)
            })
            .count();
        assert_eq!(obs.nonzero_hp_applications, events as u64);
    }
}

#[test]
fn canonical_pair_observed_heal_and_dragon_cancel_restore_and_retry() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for burst in [false, true] {
        for site in [
            "sustained_observation_source",
            if burst {
                "pair_burst_effect"
            } else {
                "direct_heal_applied"
            },
        ] {
            let (mut c, mut s, a, e, b) = provenance_fixture(burst);
            c.enable_authoring_observation();
            cast(
                &mut c,
                a,
                Position::new(if burst { 4 } else { 2 }, 1),
                Position::new(4, 1),
                0,
            );
            step(&mut c, &mut s, 999);
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                format!("{s:?}"),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || c.step_battle_execution_until(&mut s, 1000),
                )
            });
            assert!(token.is_cancelled(), "{burst}/{site}: {result:?}");
            assert!(result.is_err());
            assert_eq!(
                before,
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                    format!("{s:?}")
                )
            );
            step(&mut c, &mut s, 1000);
            assert_eq!(
                c.units[&e].stats.current_health,
                if burst { 750 } else { 1000 }
            );
            assert_eq!(
                c.units[&b].stats.current_health,
                if burst { 500 } else { 650 }
            );
        }
    }
}

#[test]
fn paired_areas_runtime_union_once_and_intersection_membership_independent_of_healing() {
    let (mut c, mut s, a, _, b) = fixture(&[]);
    place(&mut c, b, 3, 1);
    c.units.get_mut(&b).unwrap().stats.current_health = 500;
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.pairs.pairs.len(), 1);
    assert_eq!(c.sustained.target_state_speed(b), 20);
    assert!(!c.resonance_use_has_obligations(a));
    step(&mut c, &mut s, 999);
    assert_eq!(c.units[&b].stats.current_health, 500);
    step(&mut c, &mut s, 1000);
    assert_eq!(c.units[&b].stats.current_health, 600);
    place(&mut c, b, 4, 1);
    c.runtime_transaction(|c| c.reconcile_sustained(1000))
        .unwrap();
    assert_eq!(c.sustained.target_state_speed(b), 0);
    place(&mut c, b, 3, 1);
    c.units.get_mut(&b).unwrap().stats.current_health = 1000;
    c.runtime_transaction(|c| c.reconcile_sustained(1000))
        .unwrap();
    assert_eq!(c.sustained.target_state_speed(b), 20);
    step(&mut c, &mut s, 3000);
    assert!(c.sustained.pairs.pairs.is_empty());
    assert_eq!(c.sustained.target_state_speed(b), 0);
}
#[test]
fn paired_areas_runtime_base_before_transfer_no_reboost_or_retransfer() {
    let (mut c, mut s, a, _, b) = fixture(&[2, 4]);
    c.units.get_mut(&b).unwrap().stats.current_health = 500;
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 1000);
    // 150 base; full-health first side contributes floor(150*50%)=75 exactly once.
    assert_eq!(c.units[&b].stats.current_health, 725);
    assert_eq!(c.units[&a].stats.current_health, 1000);
    assert!(c.sustained.records.values().any(|r| r.target == b
        && r.target_state
            .as_ref()
            .is_some_and(|p| p.attack_percent == 10)));
    assert!(!c.sustained.records.values().any(|r| r.target == a
        && r.target_state
            .as_ref()
            .is_some_and(|p| p.attack_percent == 10)));
    step(&mut c, &mut s, 3000);
    assert_eq!(c.units[&b].stats.current_health, 950);
    assert!(c.sustained.pairs.pairs.is_empty());
}
#[test]
fn paired_areas_runtime_rejects_partial_invalid_second_and_preserves_old_pair() {
    let (mut c, mut s, a, _, _) = fixture(&[]);
    for target in [
        None,
        Some(SkillCastTarget::Tile {
            position: Position::new(2, 1),
        }),
        Some(SkillCastTarget::TilePair {
            first: Position::new(2, 1),
            second: Position::new(99, 99),
        }),
    ] {
        let before = c.units[&a].resonance_current;
        assert!(c
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: "r11_aura".into(),
                target,
                time_ms: 0
            })
            .is_err());
        assert_eq!(c.units[&a].resonance_current, before);
        assert!(c.sustained.pairs.pairs.is_empty());
    }
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 1);
    let old = *c.sustained.pairs.pairs.keys().next().unwrap();
    c.units.get_mut(&a).unwrap().resonance_current = 100;
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: Some(SkillCastTarget::TilePair {
                first: Position::new(2, 1),
                second: Position::new(99, 99)
            }),
            time_ms: 1
        })
        .is_err());
    assert!(c.sustained.pairs.pairs.contains_key(&old));
    cast(&mut c, a, Position::new(2, 1), Position::new(2, 1), 1);
    step(&mut c, &mut s, 2);
    assert_eq!(c.sustained.pairs.pairs.len(), 1);
    assert!(!c.sustained.pairs.pairs.contains_key(&old));
}
#[test]
fn paired_areas_runtime_relics_change_only_declared_lifetime_range_and_buff() {
    let (mut c, mut s, a, _, b) = fixture(&[0, 1, 3]);
    place(&mut c, b, 4, 1);
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 0);
    let p = c.sustained.pairs.pairs.values().next().unwrap();
    assert_eq!(p.expires, 3750);
    assert_eq!(p.program.interval_ms, 1000);
    assert_eq!(p.program.healing, 100);
    assert_eq!(c.sustained.target_state_speed(b), 30);
    step(&mut c, &mut s, 3750);
    assert_eq!(c.sustained.target_state_speed(b), 0);
}
#[test]
fn paired_areas_runtime_coincident_burst_is_paid_one_shot_without_support() {
    let (mut c, mut s, a, e, b) = fixture(&[5]);
    place(&mut c, e, 3, 1);
    place(&mut c, b, 3, 2);
    c.units.get_mut(&e).unwrap().threat_class = BattleUnitThreatClass::Boss;
    c.units.get_mut(&e).unwrap().stats.max_health = 1001;
    c.units.get_mut(&e).unwrap().stats.current_health = 1001;
    cast(&mut c, a, Position::new(3, 1), Position::new(3, 1), 0);
    step(&mut c, &mut s, 999);
    assert!(c.sustained.records.is_empty());
    assert!(c.units[&a].is_active());
    assert_eq!(c.units[&a].resonance_current, 0);
    step(&mut c, &mut s, 1000);
    assert!(!c.units[&a].is_active());
    assert!(!c.units[&b].is_active());
    assert_eq!(c.units[&e].stats.current_health, 750);
    assert!(c.sustained.pairs.pairs.is_empty());
}
#[test]
fn paired_areas_runtime_proportional_residue_uses_stable_id_and_caps_to_deficit() {
    let (mut c, mut s, a, _, b) = fixture(&[]);
    let extra = UnitInstanceId(Uuid::from_u128(1));
    let mut u = controlled_clone(&c.units[&b]).unwrap();
    u.instance_id = extra;
    u.fragment_source = None;
    u.skill_id = None;
    u.stats.current_health = 500;
    c.units.insert(extra, u);
    place(&mut c, extra, 4, 2);
    c.units.get_mut(&b).unwrap().stats.current_health = 700;
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 1000);
    // Deficits 400:200, budget50 => floors33:16; smallest ID gets remaining1.
    assert_eq!(c.units[&extra].stats.current_health, 634);
    assert_eq!(c.units[&b].stats.current_health, 816);
}
#[test]
fn paired_areas_runtime_empty_rejected_and_airborne_members() {
    for case in 0..4 {
        let (mut c, mut s, a, _, b) = fixture(&[]);
        c.units.get_mut(&b).unwrap().stats.current_health = 500;
        c.units.get_mut(&b).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
        step(&mut c, &mut s, 0);
        match case {
            0 => place(&mut c, a, 0, 3),
            1 => c.units.get_mut(&a).unwrap().role = BattleUnitRole::IndependentWall,
            2 => {
                c.configure_stress_rounds(vec![vec!["boss_group".into()]]);
                c.units.get_mut(&a).unwrap().role = BattleUnitRole::DefenseObject;
            }
            _ => (),
        }
        step(&mut c, &mut s, 1000);
        assert_eq!(
            c.units[&b].stats.current_health,
            if case == 3 { 650 } else { 600 },
            "{case}"
        );
    }
}
#[test]
fn paired_areas_runtime_copies_heal_separately_and_keep_intersection_contributions_separate() {
    let (mut c, mut s, a, _, b) = fixture(&[3]);
    place(&mut c, b, 3, 1);
    cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
    cast(&mut c, b, Position::new(2, 1), Position::new(4, 1), 0);
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.pairs.pairs.len(), 2);
    assert_eq!(c.sustained.target_state_speed(b), 30);
    c.units.get_mut(&b).unwrap().stats.current_health = 500;
    step(&mut c, &mut s, 1000);
    assert_eq!(c.units[&b].stats.current_health, 700);
    c.runtime_transaction(|c| c.withdraw_unit(a, 1000)).unwrap();
    assert_eq!(c.sustained.pairs.pairs.len(), 1);
    assert_eq!(c.sustained.target_state_speed(b), 20);
    c.runtime_transaction(|c| c.clear_sustained(1000)).unwrap();
    assert!(c.sustained.pairs.pairs.is_empty());
    assert_eq!(c.sustained.target_state_speed(b), 0);
}
#[test]
fn paired_areas_runtime_hard_cc_silence_preserve_installation_but_exit_cancels() {
    use crate::game::ability::sustained::{HitStatusKind, HitStatusProfile};
    for case in 0..3 {
        let (mut c, mut s, a, e, b) = fixture(&[]);
        c.units.get_mut(&b).unwrap().stats.current_health = 500;
        cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
        step(&mut c, &mut s, 0);
        c.runtime_transaction(|c| {
            c.begin_sustained(
                e,
                a,
                SustainedEffectDef::HitStatus {
                    duration_ms: 2000,
                    profile: HitStatusProfile {
                        definition: "stun".into(),
                        kind: HitStatusKind::Stun,
                        modifier_source: None,
                        vulnerability: None,
                    },
                },
                1,
            )
        })
        .unwrap();
        c.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: e,
                target_instance_id: a,
                buff_id: crate::game::battle::buffs::BuffId::from_name("r11_silence"),
                duration_ms: 2000,
                cause: Default::default(),
            },
            1,
        )
        .unwrap();
        assert!(c.unit_control_state(a, 1).unwrap().silenced);
        if case == 1 {
            c.runtime_transaction(|c| c.withdraw_unit(a, 1)).unwrap();
        }
        if case == 2 {
            c.runtime_transaction(|c| {
                c.finalize_unit_death(Some(e), a, Side::Player, 1);
                control::poll("pair_source_death")
            })
            .unwrap();
        }
        step(&mut c, &mut s, 1000);
        assert_eq!(
            c.units[&b].stats.current_health,
            if case == 0 { 650 } else { 500 },
            "{case}"
        );
        assert_eq!(c.sustained.pairs.pairs.len(), usize::from(case == 0));
    }
}
#[test]
fn paired_areas_runtime_burst_requires_equipped_same_center_and_cancel_before_impact() {
    for case in 0..4 {
        let (mut c, mut s, a, e, b) = fixture(if case == 0 { &[] } else { &[5] });
        place(&mut c, b, 3, 1);
        place(&mut c, e, 3, 2);
        cast(
            &mut c,
            a,
            Position::new(3, 1),
            if case == 1 {
                Position::new(4, 1)
            } else {
                Position::new(3, 1)
            },
            0,
        );
        step(&mut c, &mut s, 0);
        if case == 2 {
            c.runtime_transaction(|c| c.withdraw_unit(a, 1)).unwrap();
        }
        if case == 3 {
            c.runtime_transaction(|c| c.clear_sustained(1)).unwrap();
        }
        step(&mut c, &mut s, 1000);
        assert!(c.units[&b].is_active());
        assert_eq!(c.units[&e].stats.current_health, 1000);
    }
}
#[test]
fn paired_areas_runtime_burst_classification_and_normal_damage_protection() {
    use crate::game::ability::sustained::SurvivalPolicy;
    for case in 0..4 {
        let (mut c, mut s, a, e, b) = fixture(&[5]);
        place(&mut c, a, 0, 0);
        place(&mut c, b, 3, 1);
        place(&mut c, e, 3, 2);
        c.units.get_mut(&e).unwrap().source_identity = BattleUnitSourceIdentity::CorrodedEmployee {
            profile_id: "fixture".into(),
            base_uuid: Uuid::from_u128(2),
        };
        c.units.get_mut(&e).unwrap().threat_class = if case == 0 {
            BattleUnitThreatClass::Normal
        } else if case == 1 {
            BattleUnitThreatClass::Elite
        } else {
            BattleUnitThreatClass::Boss
        };
        c.units.get_mut(&e).unwrap().stats.current_health = 100;
        c.runtime_transaction(|c| {
            c.begin_sustained(
                b,
                b,
                SustainedEffectDef::Survival {
                    duration_ms: 2000,
                    policy: SurvivalPolicy::FloorOne,
                },
                0,
            )?;
            c.begin_sustained(
                e,
                e,
                SustainedEffectDef::Survival {
                    duration_ms: 2000,
                    policy: SurvivalPolicy::FloorOne,
                },
                0,
            )?;
            if case == 3 {
                c.begin_sustained(
                    e,
                    e,
                    SustainedEffectDef::Shield {
                        duration_ms: 2000,
                        amount: 200,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )?;
            }
            Ok(())
        })
        .unwrap();
        cast(&mut c, a, Position::new(3, 1), Position::new(3, 1), 0);
        step(&mut c, &mut s, 1000);
        assert!(!c.units[&b].is_active());
        assert_eq!(
            c.units[&e].stats.current_health,
            if case < 2 {
                0
            } else if case == 2 {
                1
            } else {
                50
            },
            "{case}"
        );
    }
}
#[test]
fn paired_areas_runtime_install_and_heal_cancellation_restore_state_queue_and_retry() {
    use crate::authoring::control::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["paired_membership_query", "paired_heal_segment"] {
        let (mut c, mut s, a, _, b) = fixture(&[]);
        c.units.get_mut(&b).unwrap().stats.current_health = 500;
        cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
        let now = if site == "paired_heal_segment" {
            step(&mut c, &mut s, 999);
            1000
        } else {
            0
        };
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.pairs),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, now),
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
                format!("{:?}", c.sustained.pairs),
                format!("{s:?}")
            )
        );
        step(&mut c, &mut s, now);
        if now == 1000 {
            assert_eq!(c.units[&b].stats.current_health, 650);
        } else {
            assert_eq!(c.sustained.pairs.pairs.len(), 1);
        }
    }
}
#[test]
fn paired_areas_runtime_retained_boundary_replacement_restores_old_pair_and_retries() {
    use crate::game::battle::runtime_control as native;
    for replacement in [false, true] {
        let (mut c, mut s, a, _, b) = fixture(&[]);
        place(&mut c, b, 3, 1);
        if replacement {
            cast(&mut c, a, Position::new(2, 1), Position::new(4, 1), 0);
            step(&mut c, &mut s, 1);
            c.units.get_mut(&a).unwrap().resonance_current = 100;
        }
        let now = if replacement { 1 } else { 0 };
        cast(&mut c, a, Position::new(3, 1), Position::new(3, 1), now);
        // Match owned allocation shape for both measurements; cloning Vecs may
        // shrink spare capacity, so an original and a clone are different inputs.
        let mut probe = controlled_clone(&c).unwrap();
        c = controlled_clone(&c).unwrap();
        let mut probe_s = s.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut probe_s, now)
        });
        result.unwrap();
        let (site, _, boundary) = *admissions.iter().max_by_key(|(_, _, n)| *n).unwrap();
        assert!(boundary > 1);
        c.runtime_facts.limits.max_retained_bytes = boundary - 1;
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.pairs),
            format!("{s:?}"),
        );
        let failure = c.step_battle_execution_until(&mut s, now).unwrap_err();
        assert!(
            matches!(
                failure,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    ..
                })
            ),
            "{failure:?}"
        );
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{:?}", c.sustained.pairs),
                format!("{s:?}")
            )
        );
        c.runtime_facts.limits.max_retained_bytes = boundary;
        step(&mut c, &mut s, now);
        assert_eq!(c.sustained.pairs.pairs.len(), 1);
        assert_eq!(c.sustained.target_state_speed(b), 20);
        c.runtime_facts.limits = Default::default();
        c.runtime_transaction(|c| c.clear_paired_areas(now))
            .unwrap();
        assert_eq!(c.sustained.pairs.retained_extent(), Some(0));
        println!("PAIR retained replacement={replacement} boundary={boundary} site={site} one_short_rollback=true exact_retry=true pair_owner_released=true");
    }
}
