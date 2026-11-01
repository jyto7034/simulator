use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::{
    primary_fragment_input, enable_complete_trace, hp_trace_rows, assert_fragment_trace_origin,
};
use super::*;
use crate::authoring::composition::{canonical_hatred_tests as source, compile_json, Artifact};
use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_content_with_data;

fn canonical(
    bloom: bool,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    canonical_fixture(bloom, &[], None)
}

// Existing runtime boundaries now execute the current complete menu and actual
// authored relic registrations. The r11 root is only this native fixture's identity.
pub(super) fn canonical_fixture(
    bloom: bool,
    equipped: &[usize],
    period: Option<f64>,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut document = source::resolved(bloom);
    source::rename_root(&mut document, "r11_aura");
    if let Some(seconds) = period {
        for v in document["values"].as_array_mut().unwrap() {
            v["value"]["Effects"][0]["op"]["CounterChannel"]["program"]["interval_ms"] =
                serde_json::json!((seconds * 1000.0) as u64);
        }
    }
    canonical_document(document, equipped)
}
fn canonical_document(
    document: serde_json::Value,
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    let mut relics = source::relics();
    // The draft leaves duplicate policy unresolved. Preserve the old boundary
    // case's explicit permission to equip two copies; other cases use the
    // canonical technical completion unchanged. Runtime deduplication is tested.
    for index in equipped {
        if equipped
            .iter()
            .filter(|candidate| *candidate == index)
            .count()
            > 1
        {
            relics
                .relics
                .iter_mut()
                .find(|r| r.id == RELICS[*index])
                .unwrap()
                .allow_duplicates = true;
        }
    }
    // Callers use RELICS indices; the actual catalog includes other providers.
    let indices: Vec<_> = equipped
        .iter()
        .map(|i| {
            relics
                .relics
                .iter()
                .position(|r| r.id == RELICS[*i])
                .unwrap()
        })
        .collect();
    let (c, s, a, e, b) = fixture_content_with_data(
        &RELICS,
        &indices,
        native,
        |_| {},
        crate::game::resource::ResourceCatalog { resources: vec![] },
        |data| data.with_relics(relics),
        |_| {},
    );
    configure_counter_fixture(c, s, a, e, b)
}

#[test]
fn canonical_hatred_menu_damage_and_heal_origin_preserve_observer_parity() {
    for bloom in [false, true] {
        for manual in [false, true] {
            let (mut plain, mut plain_state, a, e, b) = canonical(bloom);
            if manual && !bloom {
                let key = key(&plain, a);
                plain.sustained.counters.owners.get_mut(&key).unwrap().count = 3;
            }
            plain.units.get_mut(&b).unwrap().stats.current_health = 9000;
            let copy = primary_fragment_input(&plain, a);
            let mut observed = plain.clone();
            let mut observed_state = plain_state.clone();
            let (_trace, trace_from) = enable_complete_trace(&mut observed);
            let expected_skill = if !manual { "r11_aura" }
                else if bloom { "r11_aura__entry_0" } else { "r11_aura__entry_1" };
            for (core, state) in [
                (&mut plain, &mut plain_state),
                (&mut observed, &mut observed_state),
            ] {
                cast(core, a, manual, 0, Position::new(3, 1));
                core.step_battle_execution_until(state, if bloom && manual { 5001 } else { 2001 })
                    .unwrap();
                assert!(owner(core, a).active.is_none());
                assert_eq!(owner(core, a).count, if manual { 0 } else { 2 });
                assert!(core.units[&b].stats.current_health > 9000);
                assert_eq!(
                    core.units[&e].stats.current_health < 10000,
                    !manual || bloom
                );
            }
            for id in [a, e, b] {
                assert_eq!(
                    plain.units[&id].stats.current_health,
                    observed.units[&id].stats.current_health
                );
            }
            assert_eq!(
                plain.counter_channels_snapshot().unwrap(),
                observed.counter_channels_snapshot().unwrap()
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
                plain.event_log.to_json_string().unwrap(),
                observed.event_log.to_json_string().unwrap()
            );
            let obs = observed.authoring_observation().unwrap().unwrap();
            let hp = obs.latest_nonzero_sourced_hp.as_ref().unwrap();
            assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, copy);
            assert_eq!(hp.fragment.as_ref().unwrap().skill_id, "r11_aura");
            assert_eq!(hp.source.as_ref().unwrap().cast.skill_id.as_str(), expected_skill);
            assert!(!hp.source.as_ref().unwrap().authored.source_file.is_empty());
            for row in hp_trace_rows(&observed, trace_from) {
                assert_fragment_trace_origin(&row, a, Some((expected_skill, 0)));
            }
            let events = observed
                .event_log
                .entries
                .iter()
                .filter(|entry| {
                    matches!(entry.event,
                crate::game::battle::event_log::BattleLogEvent::HpChanged {delta, ..} if delta != 0)
                })
                .count();
            assert_eq!(
                obs.nonzero_hp_applications, events as u64,
                "each HP application recorded once"
            );
        }
    }
}

#[test]
fn canonical_hatred_menu_observation_and_applied_heal_cancellation_restore_and_retry() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "sustained_observation_fragment_owner",
        "direct_heal_applied",
    ] {
        let (mut c, mut s, a, _, b) = canonical(true);
        c.units.get_mut(&b).unwrap().stats.current_health = 9000;
        c.enable_authoring_observation();
        cast(&mut c, a, true, 0, Position::new(3, 1));
        // Commit the first tick; fail the following timestamp as one transaction.
        c.step_battle_execution_until(&mut s, 499).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = crate::authoring::control::with_test_cancellation(site, 1, || {
            crate::authoring::control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 500),
            )
        });
        assert!(token.is_cancelled(), "{site}: {result:?}");
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
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert_eq!(c.units[&b].stats.current_health, 9016);
        assert!(owner(&c, a).active.is_some());
    }
}

#[test]
fn canonical_hatred_widened_range_changes_both_normal_recipients_and_automatic_friendly_damage() {
    for bloom in [false, true] {
        for automatic in [false, true] {
            let mut document = source::resolved(bloom);
            source::rename_root(&mut document, "r11_aura");
            // A separate explicit resolution of the unknown footprint: the
            // unmodified range is one tile, the authored modifier remains 5x5.
            // Update all menu copies, preserving their matching declaration.
            for value in document["values"].as_array_mut().unwrap() {
                let program = &mut value["value"]["Effects"][0]["op"]["CounterChannel"]["program"];
                for entry in program["entries"].as_array_mut().unwrap() {
                    if (automatic && entry["automatic"] == true)
                        || (!automatic && entry["key"] == "beam")
                    {
                        for phase in entry["phases"].as_array_mut().unwrap() {
                            phase["range"]["Fixed"]["pattern"] =
                                serde_json::json!({"rows":["@"],"include_anchor_tile":true});
                        }
                    }
                }
            }
            for widened in [false, true] {
                let (mut c, mut s, a, e, b) =
                    canonical_document(document.clone(), if widened { &[5] } else { &[] });
                let actor_tile = c.units[&a].body.projected_tile();
                let ally_tile = c.units[&b].body.projected_tile();
                if automatic {
                    c.step_battle_execution_until(&mut s, 90000).unwrap();
                    assert!(
                        c.counter_channels_snapshot().unwrap().owners[0]
                            .active
                            .as_ref()
                            .unwrap()
                            .automatic
                    );
                    assert_eq!(
                        c.units[&a].stats.current_health, 9990,
                        "automatic hits its own caster"
                    );
                    assert_eq!(
                        c.units[&e].stats.current_health,
                        if widened { 9990 } else { 10000 }
                    );
                    assert_eq!(
                        c.units[&b].stats.current_health,
                        if widened { 9990 } else { 10000 }
                    );
                    assert_eq!(c.units[&a].body.projected_tile(), actor_tile);
                    assert_eq!(
                        c.units[&b].body.projected_tile(),
                        ally_tile,
                        "friendlies are never pulled"
                    );
                } else {
                    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
                    cast(&mut c, a, false, 0, Position::new(3, 1));
                    c.step_battle_execution_until(&mut s, 2000).unwrap();
                    assert_eq!(
                        c.units[&e].stats.current_health,
                        if widened { 9960 } else { 10000 }
                    );
                    assert_eq!(
                        c.units[&b].stats.current_health,
                        if widened { 9032 } else { 9000 }
                    );
                    assert_eq!(owner(&c, a).count, if widened { 2 } else { 1 });
                }
            }
        }
    }
}
