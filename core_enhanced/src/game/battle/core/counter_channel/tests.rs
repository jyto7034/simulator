use super::*;
use crate::game::battle::{
    core::{
        boss_runtime::tests::compiled_action_fixture::{fixture_graph, place},
        sim::{BattleExecutionState, BattleLiveCommand},
    },
    event_log::SkillCastTarget,
};
const RELICS: [&str; 6] = [
    "double-harvest",
    "hasty-second-hand",
    "healing-increase",
    "lingering-warmth",
    "unbroken-trance",
    "widened-horizon",
];
fn fixture(
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
    let mut doc = crate::authoring::node_graph::tests::counter_channel_fixture(bloom);
    if let Some(seconds) = period {
        let timer = doc["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["kind"] == "skill-seconds" && n["values"]["value"] == 15)
            .unwrap();
        timer["values"]["value"] = serde_json::json!(seconds);
    }
    graph_fixture(doc, equipped)
}
fn graph_fixture(
    doc: serde_json::Value,
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let (mut c, s, a, e, b) = fixture_graph(&RELICS, equipped, doc);
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    place(&mut c, e, 3, 1);
    place(&mut c, b, 2, 2);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
    }
    // Shared fixture binds inventory after deployment at t=0. Finish that setup
    // before measuring any clock or accepting any cast.
    c.runtime_transaction(|c| {
        c.clear_counter_channels(0)?;
        c.reconcile_counter_channels(0)
    })
    .unwrap();
    (c, s, a, e, b)
}
fn key(c: &BattleCore, a: UnitInstanceId) -> Uuid {
    c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
}
fn owner(c: &BattleCore, a: UnitInstanceId) -> &Owner {
    &c.sustained.counters.owners[&key(c, a)]
}
fn skill(c: &BattleCore, a: UnitInstanceId, manual: bool) -> SkillId {
    let o = owner(c, a);
    let i = if manual {
        o.program
            .entries
            .iter()
            .position(|e| e.spend == CounterSpend::AdmissionAll)
            .unwrap()
    } else {
        o.program.default_entry
    };
    o.program.entry_skill_id(&o.fragment.skill_id, i)
}
fn cast(c: &mut BattleCore, a: UnitInstanceId, manual: bool, now: u64, tile: Position) {
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: skill(c, a, manual),
        target: Some(SkillCastTarget::Tile { position: tile }),
        time_ms: now,
    })
    .unwrap();
}
#[test]
fn counter_channel_runtime_normal_ticks_half_open_and_awards_actual_cast_outcomes_once() {
    for (enemy, injured, reward) in [
        (true, true, 2),
        (true, false, 1),
        (false, true, 1),
        (false, false, 1),
    ] {
        let (mut c, mut s, a, e, b) = fixture(false, &[], None);
        if !enemy {
            place(&mut c, e, 7, 3);
        }
        if injured {
            c.units.get_mut(&b).unwrap().stats.current_health = 9000;
        }
        let hp = c.units[&e].stats.current_health;
        cast(&mut c, a, false, 0, Position::new(3, 1));
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert!(
            c.channel_active(a) && c.channel_pauses_basic(a) && c.resonance_use_has_obligations(a)
        );
        assert!(owner(&c, a)
            .active
            .as_ref()
            .unwrap()
            .snapshot
            .fact_source
            .authored
            .is_some());
        assert_eq!(owner(&c, a).count, 0);
        assert_eq!(
            hp - c.units[&e].stats.current_health,
            if enemy { 10 } else { 0 }
        );
        c.step_battle_execution_until(&mut s, 1999).unwrap();
        assert_eq!(owner(&c, a).count, 0);
        c.step_battle_execution_until(&mut s, 2000).unwrap();
        assert_eq!(owner(&c, a).count, reward);
        assert_eq!(
            hp - c.units[&e].stats.current_health,
            if enemy { 40 } else { 0 }
        );
        assert_eq!(
            c.units[&b].stats.current_health,
            if injured { 9032 } else { 10000 }
        );
        assert!(!c.channel_active(a) && !c.resonance_use_has_obligations(a));
        assert_eq!(owner(&c, a).next_gain, Some(15000));
    }
}
#[test]
fn counter_channel_runtime_manual_cancel_keeps_committed_hits_and_awards_nothing() {
    let (mut c, mut s, a, e, _) = fixture(false, &[], None);
    cast(&mut c, a, false, 0, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 100).unwrap();
    let hp = c.units[&e].stats.current_health;
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: skill(&c, a, false),
        target: None,
        time_ms: 100,
    })
    .unwrap();
    assert!(!c.channel_active(a));
    assert_eq!(owner(&c, a).count, 0);
    assert_eq!(c.units[&e].stats.current_health, hp);
    assert_eq!(owner(&c, a).aim, Some(Position::new(3, 1)));
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    assert_eq!(owner(&c, a).count, 0);
}
#[test]
fn counter_channel_runtime_bloom_zero_spend_resets_timer_and_pillar_uses_selected_center() {
    let (mut c, mut s, a, e, b) = fixture(true, &[], None);
    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
    cast(&mut c, a, true, 100, Position::new(4, 1));
    c.step_battle_execution_until(&mut s, 2099).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert_eq!(owner(&c, a).next_gain, Some(15100));
    assert_eq!(owner(&c, a).aim, None);
    c.step_battle_execution_until(&mut s, 2100).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 9990);
    c.step_battle_execution_until(&mut s, 5100).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 9940);
    assert_eq!(c.units[&b].stats.current_health, 9080);
    assert_eq!(owner(&c, a).count, 0);
    assert!(!c.channel_active(a));
}
#[test]
fn counter_channel_runtime_periodic_cap_waits_for_current_cast_and_auto_resets_without_resonance() {
    let (mut c, mut s, a, _, b) = fixture(false, &[], Some(1.0));
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    assert_eq!(owner(&c, a).count, 5);
    cast(&mut c, a, false, 5000, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 6000).unwrap();
    assert_eq!(owner(&c, a).count, 6);
    assert!(
        !owner(&c, a).program.entries[owner(&c, a).active.as_ref().unwrap().admitted.entry]
            .automatic
    );
    assert_eq!(c.units[&b].stats.current_health, 10000);
    c.step_battle_execution_until(&mut s, 7000).unwrap();
    assert!(
        owner(&c, a).program.entries[owner(&c, a).active.as_ref().unwrap().admitted.entry]
            .automatic
    );
    assert_eq!(c.units[&a].resonance_current, 0);
    assert_eq!(c.units[&b].stats.current_health, 9990);
    assert_eq!(owner(&c, a).next_gain, None);
    c.step_battle_execution_until(&mut s, 10000).unwrap();
    assert_eq!(owner(&c, a).count, 0);
    assert_eq!(owner(&c, a).next_gain, Some(11000));
    assert!(!c.channel_active(a));
}
#[test]
fn counter_channel_runtime_actual_normal_relics_separate_clock_gain_healing_and_independent_buff() {
    let (mut c, mut s, a, _, b) = fixture(false, &[2, 3], None);
    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
    let attack = c.units[&b].stats.attack;
    cast(&mut c, a, false, 0, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 1).unwrap();
    assert_eq!(c.units[&b].stats.current_health, 9010);
    assert_eq!(c.units[&b].stats.attack, attack + attack / 10);
    assert_eq!(c.units[&a].stats.attack, 100, "overheal grants no buff");
    assert_eq!(owner(&c, a).next_gain, Some(15000));
    assert_eq!(owner(&c, a).active.as_ref().unwrap().next_at, 500);
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    assert_eq!(owner(&c, a).count, 2);
    assert_eq!(c.units[&b].stats.current_health, 9040);
    assert_eq!(c.units[&b].stats.attack, attack);
    assert!(!c.resonance_use_has_obligations(a));
    let (mut c, mut s, a, _, b) = fixture(false, &[0, 0, 1], None);
    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
    cast(&mut c, a, false, 0, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    assert_eq!(
        owner(&c, a).count,
        4,
        "one authored doubling, not once per equipped copy"
    );
    assert_eq!(owner(&c, a).next_gain, Some(12000));
    c.step_battle_execution_until(&mut s, 12000).unwrap();
    assert_eq!(owner(&c, a).count, 6);
    assert!(owner(&c, a).active.is_some());
    c.step_battle_execution_until(&mut s, 15000).unwrap();
    assert_eq!(owner(&c, a).count, 0);
    assert_eq!(owner(&c, a).next_gain, Some(27000));
}
fn buff(
    c: &mut BattleCore,
    source: UnitInstanceId,
    target: UnitInstanceId,
    name: &str,
    duration_ms: u64,
    now: u64,
) {
    c.runtime_transaction(|c| {
        c.process_commands(
            vec![crate::game::battle::damage::BattleCommand::ApplyBuff {
                caster_id: source,
                target_id: target,
                buff_id: crate::game::battle::buffs::BuffId::from_name(name),
                duration_ms,
            }],
            now,
        );
        control::poll("counter_fixture_buff")
    })
    .unwrap();
}
#[test]
fn counter_channel_runtime_silence_preserves_channel_and_protection_only_changes_running_interrupts(
) {
    for protected in [false, true] {
        for name in ["r11_stun", "r11_silence"] {
            let (mut c, mut s, a, e, _) = fixture(false, if protected { &[4] } else { &[] }, None);
            cast(&mut c, a, false, 0, Position::new(3, 1));
            c.step_battle_execution_until(&mut s, 100).unwrap();
            buff(&mut c, e, a, name, 300, 100);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            let survives = protected || name == "r11_silence";
            assert_eq!(c.channel_active(a), survives);
            c.step_battle_execution_until(&mut s, 2000).unwrap();
            assert_eq!(owner(&c, a).count, u32::from(survives));
        }
        let (mut c, mut s, a, e, _) = fixture(false, if protected { &[4] } else { &[] }, None);
        buff(&mut c, e, a, "r11_stun", 300, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let before = (
            c.units[&a].resonance_current,
            c.counter_channels_snapshot().unwrap(),
        );
        assert!(c
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: skill(&c, a, false),
                target: Some(SkillCastTarget::Tile {
                    position: Position::new(3, 1)
                }),
                time_ms: 1
            })
            .is_err());
        assert_eq!(
            (
                c.units[&a].resonance_current,
                c.counter_channels_snapshot().unwrap()
            ),
            before
        );
    }
}
#[test]
fn counter_channel_runtime_reserved_auto_obeys_silence_cannot_be_stopped_manually_and_does_not_resume_after_cc(
) {
    let (mut c, mut s, a, e, _) = fixture(false, &[], Some(1.0));
    c.step_battle_execution_until(&mut s, 5500).unwrap();
    buff(&mut c, e, a, "r11_silence", 2000, 5500);
    c.step_battle_execution_until(&mut s, 6000).unwrap();
    assert_eq!(owner(&c, a).count, 6);
    assert!(owner(&c, a).active.is_none());
    assert_eq!(owner(&c, a).next_gain, None);
    let snapshot = c.counter_channels_snapshot().unwrap();
    assert!(snapshot.owners[0].automatic_reserved);
    assert_eq!(
        snapshot.owners[0].automatic_tile,
        c.units[&a].body.projected_tile()
    );
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: skill(&c, a, false),
            target: Some(SkillCastTarget::Tile {
                position: Position::new(3, 1)
            }),
            time_ms: 6000
        })
        .is_err());
    c.step_battle_execution_until(&mut s, 7500).unwrap();
    let active = owner(&c, a).active.as_ref().unwrap();
    let automatic = owner(&c, a)
        .program
        .entry_skill_id(&owner(&c, a).fragment.skill_id, active.admitted.entry);
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: automatic,
            target: None,
            time_ms: 7500
        })
        .is_err());
    c.step_battle_execution_until(&mut s, 7600).unwrap();
    buff(&mut c, e, a, "r11_stun", 300, 7600);
    c.step_battle_execution_until(&mut s, 7601).unwrap();
    assert!(!c.channel_active(a));
    assert_eq!(owner(&c, a).count, 0);
    assert_eq!(owner(&c, a).next_gain, Some(8600));
    c.step_battle_execution_until(&mut s, 8000).unwrap();
    assert!(!c.channel_active(a));
}
#[test]
fn counter_channel_runtime_checkpoint_preview_clone_and_terminal_keep_exact_ownership() {
    let (mut c, mut s, a, _, _) = fixture(false, &[], Some(1.0));
    c.step_battle_execution_until(&mut s, 4000).unwrap();
    let menu = skill(&c, a, false);
    let predicted = c
        .counter_channel_preview(a, &menu, Position::new(3, 1), 4000)
        .unwrap()
        .unwrap();
    assert_eq!(predicted.possible_automatic_followup.len(), 2);
    cast(&mut c, a, false, 4000, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 4100).unwrap();
    let snapshot = c.counter_channels_snapshot().unwrap();
    assert_eq!(
        serde_json::from_str::<CounterChannelsSnapshot>(&serde_json::to_string(&snapshot).unwrap())
            .unwrap(),
        snapshot
    );
    assert_eq!(snapshot.owners[0].menu.len(), 2);
    assert_eq!(
        snapshot.owners[0].remembered_tile,
        Some(Position::new(3, 1))
    );
    let mut clone = c.clone();
    let mut clone_s = s.clone();
    c.step_battle_execution_until(&mut s, 6100).unwrap();
    clone
        .step_battle_execution_until(&mut clone_s, 6100)
        .unwrap();
    assert_eq!(
        c.counter_channels_snapshot().unwrap(),
        clone.counter_channels_snapshot().unwrap()
    );
    assert_eq!(
        c.event_queue_debug_snapshot(),
        clone.event_queue_debug_snapshot()
    );
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(6200);
    c.step_battle_execution_until(&mut s, 6201).unwrap();
    c.finalize_battle_execution(&mut s).unwrap();
    assert!(c.counter_channels_snapshot().unwrap().owners.is_empty());
    assert!(!c.event_queue.iter().any(|e| matches!(
        e.as_ref(),
        BattleEvent::CounterGain { .. } | BattleEvent::CounterChannelTick { .. }
    )));
}
#[test]
fn counter_channel_runtime_manual_consumes_exact_count_and_pulls_only_corrupted_enemies_by_fractional_distance(
) {
    use crate::game::battle::types::BattleUnitSourceIdentity;
    let (mut c, mut s, a, e, b) = fixture(false, &[], Some(1.0));
    place(&mut c, e, 6, 1);
    place(&mut c, b, 4, 2);
    c.units.get_mut(&e).unwrap().source_identity = BattleUnitSourceIdentity::CorrodedEmployee {
        profile_id: "fixture_corrupted".into(),
        base_uuid: Uuid::from_u128(0xdc01),
    };
    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    let before = c.units[&e].body.position;
    cast(&mut c, a, true, 3000, Position::new(4, 1));
    assert_eq!(owner(&c, a).count, 0);
    c.step_battle_execution_until(&mut s, 3001).unwrap();
    assert_eq!(before.x - c.units[&e].body.position.x, 0.5);
    assert_eq!(c.units[&b].stats.current_health, 9014);
    assert_eq!(owner(&c, a).active.as_ref().unwrap().admitted.consumed, 3);
    c.step_battle_execution_until(&mut s, 5000).unwrap();
    assert_eq!(before.x - c.units[&e].body.position.x, 2.0);
    assert_eq!(c.units[&e].stats.current_health, 10000);
    assert_eq!(c.units[&b].stats.current_health, 9056);
    assert_eq!(
        owner(&c, a).count,
        2,
        "clock gains continue; no completion award"
    );
    assert_eq!(c.units[&b].body.projected_tile(), Position::new(4, 2));
}
#[test]
fn counter_channel_runtime_pull_requeries_targets_and_respects_body_collision() {
    use crate::game::battle::types::BattleUnitSourceIdentity;
    let mut doc = crate::authoring::node_graph::tests::counter_channel_fixture(false);
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-seconds" && n["values"]["value"] == 15 {
            n["values"]["value"] = serde_json::json!(1);
        }
        if n["kind"] == "skill-ground-pull" {
            n["values"]["tiles"] = serde_json::json!(1);
        }
        if n["kind"] == "skill-range"
            && n["title"].as_str().unwrap().starts_with("기억한 고정 타일")
        {
            n["values"]["pattern"] = serde_json::json!("@X");
            n["values"]["includeAnchor"] = serde_json::json!(false);
        }
    }
    assert_eq!(
        doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["values"]["pattern"] == "@X")
            .count(),
        1
    );
    for blocked in [false, true] {
        let (mut c, mut s, a, e, b) = graph_fixture(doc.clone(), &[]);
        c.units.get_mut(&e).unwrap().source_identity = BattleUnitSourceIdentity::CorrodedEmployee {
            profile_id: "fixture_corrupted".into(),
            base_uuid: Uuid::from_u128(0xdc02),
        };
        // A normal beam supplies accepted aim; no counter field is seeded.
        cast(&mut c, a, false, 0, Position::new(4, 1));
        c.step_battle_execution_until(&mut s, 2000).unwrap();
        place(&mut c, e, 5, 1);
        place(
            &mut c,
            b,
            if blocked { 4 } else { 7 },
            if blocked { 1 } else { 3 },
        );
        c.step_battle_execution_until(&mut s, 4999).unwrap();
        let hp = c.units[&e].stats.current_health;
        // Clock 1,2 plus the completion award 1 reaches six at t=5000.
        c.step_battle_execution_until(&mut s, 5000).unwrap();
        assert_eq!(
            c.units[&e].body.projected_tile(),
            Position::new(if blocked { 5 } else { 4 }, 1)
        );
        assert_eq!(
            hp - c.units[&e].stats.current_health,
            if blocked { 10 } else { 0 },
            "damage query must run after displacement"
        );
    }
}
#[test]
fn counter_channel_runtime_withdrawal_self_lethal_auto_and_stale_generation_do_not_recreate_state()
{
    for lethal in [false, true] {
        let (mut c, mut s, a, _, _) = fixture(false, &[], Some(1.0));
        let stale_generation = owner(&c, a).generation;
        let copy = key(&c, a);
        if lethal {
            c.units.get_mut(&a).unwrap().stats.current_health = 5;
            c.step_battle_execution_until(&mut s, 6000).unwrap();
            assert!(c.units[&a].lifecycle.is_dead());
            assert!(c.event_log_entries_after_seq(0).iter().any(|entry| entry.time_ms==6000 && matches!(entry.event,
                crate::game::battle::event_log::BattleLogEvent::HpChanged { source_instance_id:Some(source),target_instance_id,hp_after:0,.. } if source==a && target_instance_id==a)));
        } else {
            cast(&mut c, a, false, 0, Position::new(3, 1));
            c.step_battle_execution_until(&mut s, 100).unwrap();
            c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 100,
            })
            .unwrap();
        }
        assert!(c.counter_channels_snapshot().unwrap().owners.is_empty());
        c.process_event(
            BattleEvent::CounterGain {
                time_ms: 7000,
                source: copy,
                generation: stale_generation,
            },
            7000,
        )
        .unwrap();
        assert!(c.counter_channels_snapshot().unwrap().owners.is_empty());
        assert!(!c.event_queue.iter().any(|e| matches!(
            e.as_ref(),
            BattleEvent::CounterGain { .. } | BattleEvent::CounterChannelTick { .. }
        )));
    }
}
#[test]
fn counter_channel_runtime_invalid_manual_count_and_unrepresentable_healing_preserve_cost_timer_and_aim(
) {
    for overflow in [false, true] {
        let mut doc = crate::authoring::node_graph::tests::counter_channel_fixture(false);
        if overflow {
            for n in doc["nodes"].as_array_mut().unwrap() {
                if n["kind"] == "skill-healing-amount" {
                    n["values"]["value"] = serde_json::json!(i32::MAX);
                }
                if n["kind"] == "skill-seconds" && n["values"]["value"] == 15 {
                    n["values"]["value"] = serde_json::json!(1);
                }
            }
        }
        let (mut c, mut s, a, _, _) = graph_fixture(doc, &[]);
        let now = if overflow { 3000 } else { 1 };
        c.step_battle_execution_until(&mut s, now).unwrap();
        let before = (
            c.counter_channels_snapshot().unwrap(),
            c.units[&a].resonance_current,
            c.event_queue_debug_snapshot(),
        );
        assert!(c
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: skill(&c, a, true),
                target: Some(SkillCastTarget::Tile {
                    position: Position::new(3, 1)
                }),
                time_ms: now
            })
            .is_err());
        assert_eq!(
            (
                c.counter_channels_snapshot().unwrap(),
                c.units[&a].resonance_current,
                c.event_queue_debug_snapshot()
            ),
            before
        );
    }
}
#[test]
fn counter_channel_runtime_independent_heal_buff_outlives_channel_without_keeping_resonance_locked()
{
    let mut doc = crate::authoring::node_graph::tests::counter_channel_fixture(false);
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-seconds" && n["values"]["value"] == serde_json::json!(0.3) {
            n["values"]["value"] = serde_json::json!(3);
        }
    }
    let (mut c, mut s, a, _, b) = graph_fixture(doc, &[3]);
    c.units.get_mut(&b).unwrap().stats.current_health = 9000;
    cast(&mut c, a, false, 0, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    assert_eq!(c.units[&b].stats.attack, 110);
    assert!(!c.resonance_use_has_obligations(a));
    c.step_battle_execution_until(&mut s, 4500).unwrap();
    assert_eq!(c.units[&b].stats.attack, 100);
}
#[test]
fn counter_channel_runtime_tactical_move_keeps_counter_clock_and_aim_and_manual_child_uses_corrosion_gate(
) {
    for corrosion in [false, true] {
        let (mut c, mut s, a, _, _) = fixture(false, &[], Some(1.0));
        if !corrosion {
            let weapon = c
                .game_data
                .equipment_data
                .get_by_id("r11_weapon")
                .unwrap()
                .weapon_profile
                .clone()
                .unwrap();
            c.initialize_deployed_tactical_state(
                a,
                crate::game::battle::types::TacticalTileKind::Ground,
                weapon,
                0,
            )
            .unwrap();
        }
        c.step_battle_execution_until(&mut s, 3000).unwrap();
        if corrosion {
            c.units.get_mut(&a).unwrap().employee_trauma =
                Some(c.game_data.run_policy.corrosion.trauma_threshold);
            assert!(c.corrosion_ready(a, &skill(&c, a, true), 3000));
            cast(&mut c, a, true, 3000, Position::new(3, 1));
            c.step_battle_execution_until(&mut s, 3001).unwrap();
            assert!(c.units[&a].corroded);
            assert!(c.forced_pause_remaining_ms().is_some());
            assert!(c.counter_channels_snapshot().unwrap().owners.is_empty());
        } else {
            let before = (owner(&c, a).count, owner(&c, a).next_gain, owner(&c, a).aim);
            c.apply_live_command(BattleLiveCommand::TacticalRelocateUnit {
                unit_id: a,
                destination: Position::new(1, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: FacingDirection::Left,
                time_ms: 3000,
            })
            .unwrap();
            assert_eq!(
                (owner(&c, a).count, owner(&c, a).next_gain, owner(&c, a).aim),
                before
            );
        }
    }
}
#[test]
fn counter_channel_runtime_admission_tick_and_completion_cancellation_restore_exact_state_and_retry(
) {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "counter_channel_admitted",
        "counter_phase_heal",
        "counter_channel_finished",
    ] {
        let admission = site == "counter_channel_admitted";
        let (mut c, mut s, a, _, b) = fixture(false, &[], if admission { Some(1.0) } else { None });
        c.units.get_mut(&b).unwrap().stats.current_health = 9000;
        let now = if admission {
            3010
        } else if site == "counter_phase_heal" {
            510
        } else {
            2010
        };
        if admission {
            c.step_battle_execution_until(&mut s, now).unwrap();
        } else {
            cast(&mut c, a, false, 10, Position::new(3, 1));
            c.step_battle_execution_until(&mut s, now - 1).unwrap();
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            c.counter_channels_snapshot().unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let requested = skill(&c, a, admission);
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    if admission {
                        c.apply_live_command(BattleLiveCommand::ActivateSkill {
                            unit_id: a,
                            skill_id: requested.clone(),
                            target: Some(SkillCastTarget::Tile {
                                position: Position::new(3, 1),
                            }),
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
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                c.counter_channels_snapshot().unwrap(),
                format!("{s:?}")
            ),
            before,
            "{site}"
        );
        if admission {
            cast(&mut c, a, true, now, Position::new(3, 1));
            assert_eq!(owner(&c, a).count, 0);
        } else {
            c.step_battle_execution_until(&mut s, now).unwrap();
            assert_eq!(
                owner(&c, a).count,
                if site == "counter_channel_finished" {
                    2
                } else {
                    0
                }
            );
        }
    }
}
#[test]
fn counter_channel_runtime_real_redeployment_resets_same_copy_and_rejects_old_clock_generation() {
    let (mut c, mut s, a, _, _) = fixture(false, &[], Some(1.0));
    cast(&mut c, a, false, 0, Position::new(3, 1));
    c.step_battle_execution_until(&mut s, 3000).unwrap();
    let old = owner(&c, a);
    assert_eq!(old.count, 4);
    assert!(old.aim.is_some());
    let source = key(&c, a);
    let generation = old.generation;
    let owned = c.units[&a].source_owned_uuid;
    let draft = c
        .scenario
        .groups
        .iter()
        .flat_map(|g| &g.spawns)
        .find(|spawn| spawn.draft.owned_uuid == owned)
        .unwrap()
        .draft
        .clone();
    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: a,
        time_ms: 3000,
    })
    .unwrap();
    let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: new } = c
        .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
            draft,
            position: Position::new(2, 1),
            tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
            facing: FacingDirection::Right,
            instance_salt: 4242,
            time_ms: 3000,
            current_hp_policy: None,
        })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(key(&c, new), source);
    assert_ne!(owner(&c, new).generation, generation);
    assert_eq!(
        (
            owner(&c, new).count,
            owner(&c, new).aim,
            owner(&c, new).next_gain
        ),
        (0, None, Some(4000))
    );
    // Both genuine clocks had the same next timestamp; only their generation differs.
    c.process_event(
        BattleEvent::CounterGain {
            time_ms: 4000,
            source,
            generation,
        },
        4000,
    )
    .unwrap();
    assert_eq!(owner(&c, new).count, 0);
    c.step_battle_execution_until(&mut s, 4000).unwrap();
    assert_eq!(owner(&c, new).count, 1);
}
