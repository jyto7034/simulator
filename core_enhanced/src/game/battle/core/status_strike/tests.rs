use super::*;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{fixture_graph, place},
    sim::{BattleExecutionState, BattleLiveCommand},
};
use crate::game::{battle::tile_range::FacingDirection, resources::Position};
const RELICS: [&str; 6] = [
    "long-breath",
    "broken-balance",
    "widened-horizon",
    "explosion-vulnerability",
    "summon-vulnerability",
    "snow-queen-remnant",
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
    fixture_edit(relics, |_| {})
}
fn fixture_edit(
    relics: &[usize],
    edit: impl FnOnce(&mut serde_json::Value),
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut doc = crate::authoring::node_graph::tests::status_strike_fixture();
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-resonance-cost" {
            n["values"]["mode"] = serde_json::json!("zero");
        }
        if n["kind"] == "skill-resonance-requirement" {
            n["values"]["mode"] = serde_json::json!("none");
        }
    }
    edit(&mut doc);
    let (mut c, s, a, e, b) = fixture_graph(&RELICS, relics, doc);
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.threat_class = crate::game::battle::types::BattleUnitThreatClass::Normal;
        u.stats.max_health = 1000;
        u.stats.current_health = 1000;
        u.stats.attack = 100;
        u.stats.defense = 0;
        u.stats.magic_resist = 0;
    }
    place(&mut c, a, 3, 3);
    place(&mut c, e, 3, 2);
    place(&mut c, b, 7, 3);
    (c, s, a, e, b)
}
fn cast(c: &mut BattleCore, a: UnitInstanceId, now: u64) -> Result<(), GameError> {
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        target: Some(SkillCastTarget::Direction {
            facing: FacingDirection::Up,
        }),
        time_ms: now,
    })
    .map(|_| ())
}
fn step(c: &mut BattleCore, s: &mut BattleExecutionState, now: u64) {
    c.step_battle_execution_until(s, now).unwrap();
}
fn defs(c: &BattleCore, a: UnitInstanceId) -> Arc<StatusStrikeDef> {
    c.game_data
        .skill_data
        .get_by_id(&c.units[&a].fragment_source.as_ref().unwrap().skill_id)
        .and_then(BattleCore::status_strike_declaration)
        .unwrap()
        .clone()
}
#[test]
fn status_strike_runtime_first_hit_slows_second_converts_and_preserves_original_source() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let p = defs(&c, a);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    assert_eq!(c.units[&e].stats.current_health, 900);
    let cold = c.status_predecessors(e, a, &p.from.definition, 0).unwrap();
    assert_eq!(cold.len(), 1);
    assert!(!c.has_status_definition(e, &p.to.definition, 0).unwrap());
    assert_eq!(c.status_mobile_views().unwrap().len(), 1);
    cast(&mut c, a, 1).unwrap();
    step(&mut c, &mut s, 1);
    assert!(c
        .status_predecessors(e, a, &p.from.definition, 1)
        .unwrap()
        .is_empty());
    assert!(c.has_status_definition(e, &p.to.definition, 1).unwrap());
    let snap = c.sustained_effect_snapshot().unwrap();
    let frozen = snap
        .active
        .iter()
        .find(|r| {
            r.hit_status
                .as_ref()
                .is_some_and(|p| p.kind == HitStatusKind::Freeze)
        })
        .unwrap();
    assert_eq!(frozen.expires_at_ms, 2001);
    assert_eq!(
        frozen.status_transition.as_ref().unwrap().previous[0],
        cold[0]
    );
    cast(&mut c, a, 2).unwrap();
    step(&mut c, &mut s, 2);
    assert_eq!(
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .find(|r| r
                .hit_status
                .as_ref()
                .is_some_and(|p| p.kind == HitStatusKind::Freeze))
            .unwrap()
            .expires_at_ms,
        2001
    );
}
#[test]
fn status_strike_runtime_chase_explodes_after_delay_and_cold_refreshes() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let p = defs(&c, a);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    place(&mut c, e, 4, 2);
    step(&mut c, &mut s, 199);
    assert_eq!(c.units[&e].stats.current_health, 900);
    step(&mut c, &mut s, 200);
    assert!(c.status_mobile_views().unwrap().is_empty());
    assert_eq!(c.units[&e].stats.current_health, 800);
    let statuses = c.sustained_effect_snapshot().unwrap();
    assert!(statuses.active.iter().any(|r| r
        .hit_status
        .as_ref()
        .is_some_and(|s| s.definition == p.from.definition)
        && r.expires_at_ms == 3200));
}
fn damage(
    c: &mut BattleCore,
    owner: UnitInstanceId,
    target: UnitInstanceId,
    amount: u32,
    now: u64,
) {
    let snapshot = c
        .damage_source_snapshot_template_for_unit(
            owner,
            DamageSource::Ability,
            DamageType::Magic,
            amount,
            DamageModifiers::default(),
            0,
            now,
            false,
        )
        .unwrap();
    c.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
        target_id: target,
        source_snapshot: snapshot,
        committed_at_ms: now,
        policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
    })
    .unwrap();
}
#[test]
fn status_strike_runtime_immunity_preserves_cold_and_only_successful_freeze_weakens() {
    let (mut c, mut s, a, e, _) = fixture(&[0, 1]);
    let p = defs(&c, a);
    damage(&mut c, a, e, 10, 0);
    let original = c.status_predecessors(e, a, &p.from.definition, 0).unwrap();
    c.units
        .get_mut(&e)
        .unwrap()
        .target_traits
        .push(crate::game::battle::types::UnitTargetTrait::StunImmune);
    cast(&mut c, a, 1).unwrap();
    step(&mut c, &mut s, 1);
    assert_eq!(
        c.status_predecessors(e, a, &p.from.definition, 1).unwrap(),
        original
    );
    assert!(!c.has_status_definition(e, &p.to.definition, 1).unwrap());
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
    c.units.get_mut(&e).unwrap().target_traits.clear();
    cast(&mut c, a, 2).unwrap();
    step(&mut c, &mut s, 2);
    assert_eq!(c.units[&e].stats.current_health, 790);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 50);
    let statuses = c.sustained_effect_snapshot().unwrap();
    assert!(statuses.active.iter().any(|r| r
        .hit_status
        .as_ref()
        .is_some_and(|s| s.kind == HitStatusKind::Freeze)
        && r.expires_at_ms == 2502));
    assert!(statuses.active.iter().any(|r| r
        .hit_status
        .as_ref()
        .is_some_and(|s| matches!(s.kind, HitStatusKind::DamageTaken { .. }))
        && r.expires_at_ms == 5002));
    assert_eq!(
        c.status_mobile_views().unwrap()[0].waiting_expires_at_ms,
        201
    );
}
#[test]
fn status_strike_runtime_zero_shield_lethal_and_slow_immunity_do_not_grant_cold() {
    for mode in 0..4 {
        let (mut c, _, a, e, _) = fixture(&[5]);
        let p = defs(&c, a);
        if mode == 1 {
            c.begin_sustained(
                e,
                e,
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
        if mode == 3 {
            c.units
                .get_mut(&e)
                .unwrap()
                .target_traits
                .push(crate::game::battle::types::UnitTargetTrait::SlowImmune);
        }
        damage(
            &mut c,
            a,
            e,
            match mode {
                0 => 0,
                2 => 2000,
                _ => 100,
            },
            0,
        );
        assert!(c
            .status_predecessors(e, a, &p.from.definition, 0)
            .unwrap()
            .is_empty());
        assert!(c.status_mobile_views().unwrap().is_empty());
    }
}
#[test]
fn status_strike_runtime_other_owner_cold_does_not_qualify_relic_and_ally_kill_does() {
    for owner_cold in [false, true] {
        let (mut c, _, a, e, b) = fixture(&[5]);
        damage(&mut c, if owner_cold { a } else { b }, e, 10, 0);
        damage(&mut c, b, e, 2000, 1);
        let views = c.status_mobile_views().unwrap();
        assert_eq!(views.len(), usize::from(owner_cold));
        if owner_cold {
            let origin = views[0].death_origin.as_ref().unwrap();
            assert_eq!(origin.target, e);
            assert_eq!(origin.states[0].owner, a);
            assert_eq!(origin.relic.definition, "snow-queen-remnant");
            assert_eq!(
                origin.relic.copy_uuid,
                c.equipped_normal_relic_copy(a, "snow-queen-remnant")
                    .unwrap()
            );
        }
    }
}
#[test]
fn status_strike_runtime_direct_lethal_spawns_relic_then_basic_and_remnant_kills_do_not_recurse() {
    let (mut c, mut s, a, e, b) = fixture(&[5]);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    c.units.get_mut(&e).unwrap().stats.current_health = 100;
    cast(&mut c, a, 1).unwrap();
    step(&mut c, &mut s, 1);
    let views = c.status_mobile_views().unwrap();
    assert_eq!(views.len(), 3);
    assert!(views[1].death_origin.is_some());
    assert!(views[2].death_origin.is_none());
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    place(&mut c, b, 4, 2);
    damage(&mut c, a, b, 10, 2);
    c.units.get_mut(&b).unwrap().stats.current_health = 100;
    step(&mut c, &mut s, 201);
    assert!(!c.units[&b].is_active());
    assert!(c.status_mobile_views().unwrap().is_empty());
}
#[test]
fn status_strike_runtime_expired_state_does_not_seed_and_empty_direction_does_not_spawn() {
    let (mut c, mut s, a, e, b) = fixture(&[5]);
    damage(&mut c, a, e, 10, 0);
    step(&mut c, &mut s, 3000);
    damage(&mut c, b, e, 2000, 3000);
    assert!(c.status_mobile_views().unwrap().is_empty());
    cast(&mut c, a, 3001).unwrap();
    step(&mut c, &mut s, 3001);
    assert!(c.status_mobile_views().unwrap().is_empty());
}
#[test]
fn status_strike_runtime_mobile_release_survives_exit_without_new_passive_subscription() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let p = defs(&c, a);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    let old = c.status_predecessors(e, a, &p.from.definition, 0).unwrap();
    c.withdraw_unit(a, 10).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 800);
    assert_eq!(
        c.status_predecessors(e, a, &p.from.definition, 200)
            .unwrap(),
        old
    );
    assert!(c.status_mobile_views().unwrap().is_empty());
}
#[test]
fn status_strike_runtime_shared_cap_prioritizes_death_generation_without_duplicate_states() {
    let (mut c, mut s, a, e, b) = fixture_edit(&[5], |doc| {
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-count-positive" {
                n["values"]["value"] = serde_json::json!(1);
            }
        }
    });
    damage(&mut c, a, e, 10, 0);
    c.units.get_mut(&e).unwrap().stats.current_health = 100;
    cast(&mut c, a, 1).unwrap();
    step(&mut c, &mut s, 1);
    let v = c.status_mobile_views().unwrap();
    assert_eq!(v.len(), 1);
    assert!(v[0].death_origin.is_some());
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    place(&mut c, b, 4, 2);
    damage(&mut c, a, b, 10, 2);
    damage(&mut c, a, b, 2000, 3);
    assert_eq!(c.status_mobile_views().unwrap().len(), 1);
}
#[test]
fn status_strike_runtime_retargets_and_keeps_deadline_without_target() {
    let (mut c, mut s, a, e, b) = fixture(&[]);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    assert_eq!(
        c.status_mobile_views().unwrap()[0].tile,
        Position::new(3, 2)
    );
    damage(&mut c, b, e, 2000, 1);
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    place(&mut c, b, 4, 2);
    step(&mut c, &mut s, 100);
    let v = c.status_mobile_views().unwrap();
    assert!(
        matches!(v[0].phase,MobilePhaseView::Chasing{target:Some(id),explodes_at_ms:200,..} if id==b)
    );
    damage(&mut c, a, b, 2000, 101);
    step(&mut c, &mut s, 199);
    assert_eq!(c.status_mobile_views().unwrap().len(), 1);
    step(&mut c, &mut s, 200);
    assert!(c.status_mobile_views().unwrap().is_empty());
}
#[test]
fn status_strike_runtime_range_power_and_summon_vulnerability_stay_on_mobile_effect() {
    let (mut c, mut s, a, e, b) = fixture(&[2, 3, 4]);
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    place(&mut c, b, 4, 2);
    cast(&mut c, a, 0).unwrap();
    step(&mut c, &mut s, 0);
    assert_eq!(c.units[&e].stats.current_health, 900);
    assert_eq!(c.units[&b].stats.current_health, 900);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 750);
    assert_eq!(c.units[&b].stats.current_health, 750);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 50);
    assert_eq!(c.hit_damage_taken_percent(b).unwrap(), 50);
}
#[test]
fn status_strike_runtime_slow_is_nonstacking_and_freeze_stops_actions_until_deadline() {
    let (mut c, mut s, a, e, b) = fixture(&[]);
    c.units.get_mut(&e).unwrap().stats.move_speed_units_per_ms = 1000;
    damage(&mut c, a, e, 1, 0);
    assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 700);
    damage(&mut c, b, e, 1, 0);
    assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 700);
    cast(&mut c, a, 1).unwrap();
    step(&mut c, &mut s, 1);
    assert_eq!(c.hit_stun_until(e, Some(1)), Some(2000));
    step(&mut c, &mut s, 2001);
    assert_eq!(c.hit_stun_until(e, Some(2001)), None);
}
#[test]
fn status_strike_runtime_cancellation_restores_transition_and_chase_damage() {
    use crate::authoring::control::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "status_strike_damage",
        "chase_move",
        "mobile_explosion_damage",
    ] {
        let (mut c, mut s, a, _, _) = fixture(&[5]);
        cast(&mut c, a, 0).unwrap();
        let now = if site == "mobile_explosion_damage" {
            step(&mut c, &mut s, 199);
            200
        } else {
            0
        };
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.chases),
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
                format!("{:?}", c.sustained.chases),
                format!("{s:?}")
            )
        );
        step(&mut c, &mut s, now);
    }
}
#[test]
fn status_strike_runtime_retained_exact_boundary_one_below_restores_and_retries() {
    use crate::game::battle::runtime_control as native;
    let (mut original, s, a, _, _) = fixture(&[0, 1, 5]);
    cast(&mut original, a, 0).unwrap();
    let mut probe = controlled_clone(&original).unwrap();
    let mut c = controlled_clone(&original).unwrap();
    let mut probe_s = s.clone();
    let mut s = s;
    let (result, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut probe_s, 0));
    result.unwrap();
    let (site, _, boundary) = *admissions.iter().max_by_key(|(_, _, n)| *n).unwrap();
    assert!(boundary > 1);
    c.runtime_facts.limits.max_retained_bytes = boundary - 1;
    let before = (
        c.transaction_debug_snapshot(),
        c.event_queue_debug_snapshot(),
        format!("{:?}", c.sustained.chases),
        format!("{s:?}"),
    );
    let error = c.step_battle_execution_until(&mut s, 0).unwrap_err();
    assert!(matches!(
        error,
        GameError::BattleRuntimeFault(native::BattleRuntimeFault {
            code: native::RuntimeFaultCode::RetainedByteLimit,
            ..
        })
    ));
    assert_eq!(
        before,
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.chases),
            format!("{s:?}")
        )
    );
    c.runtime_facts.limits.max_retained_bytes = boundary;
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.chases.entity_count(), 1);
    c.runtime_transaction(|c| c.clear_status_mobile()).unwrap();
    assert_eq!(c.sustained.chases.entity_count(), 0);
    assert_eq!(c.sustained.chases.retained_extent(), Some(0));
    println!("status strike boundary {boundary} at {site}");
}

#[test]
fn status_strike_runtime_finite_cold_seeds_after_owner_exit_but_corrosion_disables_relic() {
    for corroded in [false, true] {
        let (mut c, _, a, e, b) = fixture(&[5]);
        damage(&mut c, a, e, 10, 0);
        c.withdraw_unit(a, 1).unwrap();
        c.units.get_mut(&a).unwrap().corroded = corroded;
        damage(&mut c, b, e, 2000, 2);
        let views = c.status_mobile_views().unwrap();
        assert_eq!(views.len(), usize::from(!corroded));
        if !corroded {
            assert_eq!(views[0].death_origin.as_ref().unwrap().states[0].owner, a);
        }
    }
}
#[test]
fn status_strike_runtime_mobile_descendant_and_reaction_lethals_do_not_seed() {
    for reaction in [false, true] {
        let (mut c, mut s, a, e, b) = fixture(&[5]);
        cast(&mut c, a, 0).unwrap();
        step(&mut c, &mut s, 200);
        assert!(c.status_mobile_views().unwrap().is_empty());
        let origin = c.runtime_fact_summary().latest_damage.as_ref().unwrap().source.clone();
        assert_eq!(origin.mobile_effect().unwrap().motion, MobileMotion::ChaseDelay);
        let mut inherited = origin.independent_origin().unwrap();
        if reaction {
            inherited = RuntimeFactSource { reaction: true, ..Default::default() };
        }
        let previous = std::mem::replace(&mut c.runtime_fact_source, inherited);
        damage(&mut c, b, e, 2000, 201);
        c.runtime_fact_source = previous;
        assert!(!c.units[&e].is_active());
        assert!(c.status_mobile_views().unwrap().is_empty());
        let fact = c.runtime_fact_summary();
        let lethal = &fact.latest_damage.as_ref().unwrap().source;
        assert!(if reaction { lethal.reaction } else { lethal.mobile_effect().is_some() });
    }
}
