use super::*;
use crate::game::{
    ability::SkillDef,
    battle::{
        buffs::BuffId,
        core::{sim::BattleExecutionState, tests::native_action_fixture},
        enums::BattleEvent,
        event_log::{BattleEventCause, SkillCastTarget},
        types::BattleUnitSourceIdentity,
    },
    behavior::{ResonanceGainBlockReasonDto as Block, ResonanceUsePhaseDto as Phase},
    data::skill_data::SkillDatabase,
    resources::Position,
};

const BASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/r08/skills.ron"
));
fn parse(text: &str) -> SkillDef {
    SkillDatabase::from_ron("u05/use.ron", text)
        .unwrap()
        .get_by_id("r08_strike")
        .unwrap()
        .clone()
}
fn fixture(
    text: &str,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
) {
    let (mut core, state, actor, target, _) =
        native_action_fixture(parse(text), Default::default());
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    unit.resonance_lock_ms = 0;
    core.schedule_resonance_pulse(actor, 0);
    (core, state, actor, target)
}
fn base() -> String {
    BASE.replace("amount: 100", "amount: 1")
}
fn buff(core: &mut BattleCore, actor: UnitInstanceId, now: u64, id: &str, duration: u64) {
    core.process_event(
        BattleEvent::ApplyBuff {
            time_ms: now,
            caster_instance_id: actor,
            target_instance_id: actor,
            buff_id: BuffId::from_name(id),
            duration_ms: duration,
            cause: BattleEventCause::default(),
        },
        now,
    )
    .unwrap();
}

#[test]
fn u05_admission_consumes_once_zero_starts_and_last_action_releases_gain() {
    let (mut core, mut state, actor, target) = fixture(&base());
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    assert_eq!(core.units[&actor].resonance_current, 0);
    assert_eq!(
        core.query_action(&action).unwrap().facts.resource_spent,
        100
    );
    assert_eq!(
        core.live_skill_readiness(0, actor)
            .unwrap()
            .resonance_use
            .unwrap()
            .phase,
        Phase::Accepted
    );
    let before = core.transaction_debug_snapshot();
    assert!(core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    core.step_battle_execution_until(&mut state, 299).unwrap();
    core.add_resonance(actor, 10, 299, false);
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.step_battle_execution_until(&mut state, 300).unwrap();
    assert_eq!(core.units[&target].stats.current_health, 99);
    assert!(core
        .live_skill_readiness(300, actor)
        .unwrap()
        .resonance_use
        .is_none());
    core.add_resonance(actor, 10, 300, false);
    assert_eq!(core.units[&actor].resonance_current, 10);
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 11);
    assert_eq!(core.query_action(&action).unwrap().facts.cost_mutations, 1);
}

#[test]
fn u05_last_projectile_effects_settlement_is_not_a_use_obligation() {
    let text = base()
        .replace(
            "delivery: Instant",
            "delivery: Projectile(speed_units_per_ms: 1000)",
        )
        .replace(
            "steps: [",
            "flow: Some(Action(id: \"launch\", step_id: \"hit\", wait: EffectsSettled)), steps: [",
        );
    let (mut core, mut state, actor, target) = fixture(&text);
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 300).unwrap();
    assert_eq!(core.units[&target].stats.current_health, 100);
    assert!(core
        .query_action(&action)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    assert!(core
        .live_skill_readiness(300, actor)
        .unwrap()
        .resonance_use
        .is_none());
    core.add_resonance(actor, 7, 300, false);
    assert_eq!(core.units[&actor].resonance_current, 7);
}

#[test]
fn u05_timed_shield_and_hard_cc_have_independent_endings_without_banked_pulses() {
    let text = base().replace("automatic_unit", "self").replace("EnemySingle(rule: Nearest)", "SelfUnit")
        .replace("reach: global", "reach: pattern")
        .replace("target: CastTarget", "target: SelfUnit").replace("delay_ms: 200", "delay_ms: 0")
        .replace("use_lifetime: Action", "use_lifetime: ActionAndEffects(effects: [(step: \"hit\", effect: \"damage\")])")
        .replace("Damage(amount: 1, damage_type: True)", "Sustained(definition: Shield(duration_ms: 1000, amount: 20, priority: 0, shared_radius_units: None))");
    let (mut core, mut state, actor, _) = fixture(&text);
    core.request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 300).unwrap();
    buff(&mut core, actor, 300, "stun", 2000);
    assert_eq!(
        core.live_skill_readiness(300, actor)
            .unwrap()
            .resonance_gain_block_reasons,
        vec![Block::HardCrowdControl, Block::SkillUse]
    );
    core.step_battle_execution_until(&mut state, 1100).unwrap();
    assert_eq!(
        core.live_skill_readiness(1100, actor)
            .unwrap()
            .resonance_gain_block_reasons,
        vec![Block::HardCrowdControl]
    );
    core.step_battle_execution_until(&mut state, 3000).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 1);
}

#[test]
fn u05_accepted_use_ignores_later_silence_but_cc_cancels_without_refund() {
    for status in ["silence", "stun", "freeze"] {
        let (mut core, mut state, actor, target) = fixture(&base());
        core.request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        buff(&mut core, actor, 0, status, 500);
        core.step_battle_execution_until(&mut state, 700).unwrap();
        assert_eq!(core.units[&actor].resonance_current, 0);
        assert_eq!(
            core.units[&target].stats.current_health,
            if status == "silence" { 99 } else { 100 }
        );
        assert_eq!(core.runtime_fact_summary().resource_spent, 100);
        assert!(core
            .live_skill_readiness(700, actor)
            .unwrap()
            .resonance_use
            .is_none());
    }
}

#[test]
fn u05_employee_attack_gains_removed_opponent_gains_retained() {
    let (mut core, _, actor, target) = fixture(&base());
    core.units.get_mut(&actor).unwrap().resonance_current = 0;
    core.units.get_mut(&target).unwrap().resonance_current = 0;
    for unit in [actor, target] {
        core.grant_basic_attack_release_resonance(unit, 0);
        core.grant_basic_attack_damage_dealt_resonance(unit, 50, 0);
        core.grant_basic_attack_damage_received_resonance(unit, 50, 0, true);
    }
    assert_eq!(core.units[&actor].resonance_current, 0);
    assert_eq!(core.units[&target].resonance_current, 20);
}

#[test]
fn u05_empty_tile_is_valid_and_failed_wrong_target_does_not_spend() {
    let text = base()
        .replace("automatic_unit", "player_tile")
        .replace("EnemySingle(rule: Nearest)", "CastTarget");
    let (mut core, _, actor, target) = fixture(&text);
    assert!(core
        .request_skill_action(
            0,
            actor,
            "r08_strike".into(),
            Some(SkillCastTarget::Unit {
                unit_instance_id: target
            })
        )
        .is_err());
    assert_eq!(core.units[&actor].resonance_current, 100);
    core.request_skill_action(
        0,
        actor,
        "r08_strike".into(),
        Some(SkillCastTarget::Tile {
            position: Position::new(3, 3),
        }),
    )
    .unwrap();
    assert_eq!(core.units[&actor].resonance_current, 0);
}

#[test]
fn u05_extra_attacks_hold_use_until_last_release_and_cc_does_not_resume_them() {
    for interrupted in [false, true] {
        let text = base().replace("delay_ms: 200", "delay_ms: 0").replace(
            "Damage(amount: 1, damage_type: True)",
            "ExtraAttack(count: 3)",
        );
        let (mut core, mut state, actor, target) = fixture(&text);
        core.units.get_mut(&actor).unwrap().stats.attack = 1;
        core.request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 299).unwrap();
        assert!(core.resonance_use_has_obligations(actor));
        if interrupted {
            buff(&mut core, actor, 299, "stun", 100);
        }
        core.step_battle_execution_until(&mut state, 699).unwrap();
        assert_eq!(core.resonance_use_has_obligations(actor), !interrupted);
        core.step_battle_execution_until(&mut state, 700).unwrap();
        assert!(!core.resonance_use_has_obligations(actor));
        assert_eq!(
            core.units[&target].stats.current_health,
            if interrupted { 100 } else { 97 }
        );
        core.step_battle_execution_until(&mut state, 1000).unwrap();
        assert_eq!(core.units[&actor].resonance_current, 1);
    }
}

#[test]
fn u05_extra_attack_projectiles_unlock_at_last_launch_before_any_impact() {
    let text = base().replace("delay_ms: 200", "delay_ms: 0").replace(
        "Damage(amount: 1, damage_type: True)",
        "ExtraAttack(count: 3)",
    );
    let (mut core, mut state, actor, target) = fixture(&text);
    let unit = core.units.get_mut(&actor).unwrap();
    unit.stats.attack = 1;
    unit.basic_attack.delivery = ron::de::from_str("Projectile(speed_units_per_ms: 100)").unwrap();
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 699).unwrap();
    assert!(core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 700).unwrap();
    assert!(!core.resonance_use_has_obligations(actor));
    assert!(core
        .query_action(&action)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    assert_eq!(core.units[&target].stats.current_health, 100);
    core.add_resonance(actor, 7, 700, false);
    assert_eq!(core.units[&actor].resonance_current, 7);
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 8);
    assert_eq!(core.units[&target].stats.current_health, 100);
}

#[test]
fn u05_direct_target_withdrawal_cancels_pending_use_immediately_without_refund() {
    let (mut core, _, actor, target) = fixture(&base());
    core.request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.withdraw_unit(target, 50).unwrap();
    assert!(core.units[&actor].pending_skill_cast.is_none());
    assert!(!core.resonance_use_has_obligations(actor));
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.add_resonance(actor, 7, 50, false);
    assert_eq!(core.units[&actor].resonance_current, 7);
}

#[test]
fn u05_public_descriptor_rejects_missing_ambiguous_or_independent_effect_ownership() {
    let finite = base().replace("Damage(amount: 1, damage_type: True)",
        "Sustained(definition: Modifiers(duration_ms: 1000, modifiers: [(stat: Attack, kind: Flat, value: 1)]))");
    let reference =
        "use_lifetime: ActionAndEffects(effects: [(step: \"hit\", effect: \"damage\")])";
    let valid = finite.replace("use_lifetime: Action", reference);
    parse(&valid);
    for invalid in [
        finite,
        valid.replace(reference, ""),
        valid.replace(reference, "use_lifetime: ActionAndEffects(effects: [])"),
        valid.replace("effect: \"damage\"", "effect: \"missing\""),
        valid.replace(
            "effects: [(step: \"hit\", effect: \"damage\")]",
            "effects: [(step: \"hit\", effect: \"damage\"), (step: \"hit\", effect: \"damage\")]",
        ),
        base().replace("use_lifetime: Action", reference),
        base().replace("use_lifetime: Action", reference).replace(
            "Damage(amount: 1, damage_type: True)",
            "ApplyBuff(buff_id: \"poison\", duration_ms: 1000)",
        ),
    ] {
        assert!(
            SkillDatabase::from_ron("u05/invalid-use.ron", &invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn u05_actor_death_and_withdrawal_clear_pending_use_and_stale_pulse_without_refund() {
    for dead in [false, true] {
        let (mut core, mut state, actor, target) = fixture(&base());
        let action = core
            .request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 50).unwrap();
        if dead {
            core.finalize_unit_death(Some(target), actor, crate::game::enums::Side::Player, 50);
        } else {
            core.withdraw_unit(actor, 50).unwrap();
        }
        assert!(core.units[&actor].resonance_use.is_none());
        assert!(core.units[&actor].pending_skill_cast.is_none());
        assert!(core.units[&actor].next_resonance_pulse_ms.is_none());
        core.step_battle_execution_until(&mut state, 1200).unwrap();
        assert_eq!(core.units[&actor].resonance_current, 0);
        assert_eq!(core.units[&target].stats.current_health, 100);
        assert_eq!(
            core.query_action(&action).unwrap().facts.resource_spent,
            100
        );
        assert!(core.query_action(&action).unwrap().terminal.is_some());
    }
}

#[test]
fn u05_repeat_branch_and_parallel_keep_future_control_until_last_release() {
    let flows = [
        ("Sequence(id: \"root\", children: [Action(id: \"first\", step_id: \"hit\", wait: Released), Wait(id: \"wait\", duration_ms: 400), Action(id: \"last\", step_id: \"hit\", wait: EffectsSettled)])", 500),
        ("Repeat(id: \"root\", count: 3, interval_ms: 200, body: Action(id: \"leaf\", step_id: \"hit\", wait: Released))", 500),
        ("Parallel(id: \"root\", branches: [Action(id: \"first\", step_id: \"hit\", wait: Released), Sequence(id: \"later\", children: [Wait(id: \"wait\", duration_ms: 400), Action(id: \"last\", step_id: \"hit\", wait: EffectsSettled)])])", 500),
        ("Sequence(id: \"root\", children: [Action(id: \"first\", step_id: \"hit\", wait: EffectsSettled), Branch(id: \"branch\", condition: Compare(node_id: \"first\", metric: HpLost, comparison: AtLeast, value: Hp(1)), then_node: Sequence(id: \"chosen\", children: [Wait(id: \"wait\", duration_ms: 400), Action(id: \"last\", step_id: \"hit\", wait: EffectsSettled)]), else_node: Wait(id: \"miss\", duration_ms: 0))])", 500),
    ];
    for (flow, end) in flows {
        let flow = flow.replace(
            "id: \"last\", step_id: \"hit\"",
            "id: \"last\", step_id: \"hit2\"",
        );
        let mut text = base()
            .replace("delay_ms: 200", "delay_ms: 0")
            .replace("steps: [", &format!("flow: Some({flow}), steps: ["));
        if flow.contains("hit2") {
            text = text.replace(
                "])]\n",
                r#"]), SkillStepDef(id: "hit2", delay_ms: 0, target: CastTarget, delivery: Instant,
                effects: [(id: "damage", op: Damage(amount: 1, damage_type: True))])]
"#,
            );
        }
        let (mut core, mut state, actor, _) = fixture(&text);
        core.request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, end - 1)
            .unwrap();
        assert!(core.resonance_use_has_obligations(actor), "{flow}");
        core.add_resonance(actor, 7, end - 1, false);
        assert_eq!(core.units[&actor].resonance_current, 0);
        core.step_battle_execution_until(&mut state, end).unwrap();
        assert!(!core.resonance_use_has_obligations(actor), "{flow}");
        core.add_resonance(actor, 7, end, false);
        assert_eq!(core.units[&actor].resonance_current, 7);
    }
}

#[test]
fn u05_all_positive_commands_share_use_gate_negative_changes_and_cap_discard_do_not() {
    use crate::game::battle::damage::BattleCommand;
    let (mut core, mut state, actor, _) = fixture(&base());
    core.request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.process_commands(
        vec![BattleCommand::ModifyResonance {
            target_id: actor,
            amount: 100,
            allow_autocast_when_full: true,
        }],
        0,
    );
    assert_eq!(core.units[&actor].resonance_current, 0);
    // Negative changes remain legal even while the same positive gate is closed.
    core.units.get_mut(&actor).unwrap().resonance_current = 10;
    core.process_commands(
        vec![BattleCommand::ModifyResonance {
            target_id: actor,
            amount: -4,
            allow_autocast_when_full: false,
        }],
        0,
    );
    assert_eq!(core.units[&actor].resonance_current, 6);
    core.step_battle_execution_until(&mut state, 300).unwrap();
    assert_eq!(
        core.units[&actor].resonance_current, 6,
        "cast end does not charge twice"
    );
    core.add_resonance(actor, 1000, 300, false);
    core.step_battle_execution_until(&mut state, 2000).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 100);
    core.process_commands(
        vec![BattleCommand::ModifyResonance {
            target_id: actor,
            amount: -10,
            allow_autocast_when_full: false,
        }],
        2000,
    );
    core.step_battle_execution_until(&mut state, 2999).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 90);
    core.step_battle_execution_until(&mut state, 3000).unwrap();
    assert_eq!(
        core.units[&actor].resonance_current, 91,
        "capped pulses were not banked"
    );
}

#[test]
fn u05_pulse_and_use_counter_overflows_restore_native_boundary_and_retry() {
    use crate::game::{battle::runtime_control::RuntimeFaultCode, behavior::GameError};
    let text = base().replace("delay_ms: 200", "delay_ms: 0").replace(
        "Damage(amount: 1, damage_type: True)",
        "ExtraAttack(count: 3)",
    );
    let (mut core, mut state, actor, _) = fixture(&text);
    core.units.get_mut(&actor).unwrap().stats.attack = 1;
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
    );
    let error = core
        .runtime_transaction(|candidate| {
            candidate
                .units
                .get_mut(&actor)
                .unwrap()
                .next_resonance_pulse_ms = None;
            candidate.schedule_resonance_pulse(actor, u64::MAX);
            Ok(())
        })
        .unwrap_err();
    assert!(
        matches!(error, GameError::BattleRuntimeFault(fault) if fault.code == RuntimeFaultCode::CounterOverflow && fault.site == "resonance_pulse_deadline")
    );
    assert_eq!(
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot()
        ),
        before
    );
    core.request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 99).unwrap();
    core.units
        .get_mut(&actor)
        .unwrap()
        .resonance_use
        .as_mut()
        .unwrap()
        .unreleased_attacks = u32::MAX;
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{state:?}"),
    );
    let error = core
        .step_battle_execution_until(&mut state, 100)
        .unwrap_err();
    assert!(
        matches!(error, GameError::BattleRuntimeFault(fault) if fault.code == RuntimeFaultCode::CounterOverflow && fault.site == "resonance_use_attacks")
    );
    assert_eq!(
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{state:?}")
        ),
        before
    );
    core.units
        .get_mut(&actor)
        .unwrap()
        .resonance_use
        .as_mut()
        .unwrap()
        .unreleased_attacks = 0;
    core.step_battle_execution_until(&mut state, 100).unwrap();
    assert!(
        core.units[&actor]
            .resonance_use
            .as_ref()
            .unwrap()
            .unreleased_attacks
            > 0
    );
}

#[test]
fn u05_use_state_survives_controlled_clone_and_native_admission_failure() {
    use crate::game::{battle::runtime_control::RuntimeFactLimits, behavior::GameError};
    let (mut core, _, actor, _) = fixture(&base());
    let limits = core.runtime_facts.limits;
    core.runtime_facts.limits = RuntimeFactLimits {
        max_clone_slots: 1,
        ..limits
    };
    let before = core.transaction_debug_snapshot();
    assert!(matches!(
        core.request_skill_action(0, actor, "r08_strike".into(), None),
        Err(GameError::BattleRuntimeFault(_))
    ));
    assert_eq!(core.transaction_debug_snapshot(), before);
    core.runtime_facts.limits = limits;
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    let cloned = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
    assert_eq!(
        cloned.transaction_debug_snapshot(),
        core.transaction_debug_snapshot()
    );
    assert!(cloned.has_resonance_use_credential(actor, &"r08_strike".into(), &action));
    assert_eq!(
        serde_json::to_value(cloned.live_skill_readiness(0, actor)).unwrap(),
        serde_json::to_value(core.live_skill_readiness(0, actor)).unwrap()
    );
    assert_eq!(
        cloned.query_action(&action).unwrap().facts.resource_spent,
        100
    );
}

#[test]
fn u05_native_use_scans_refuse_at_their_budget_boundary_without_mutation() {
    use crate::game::battle::runtime_control::{
        self as native, RuntimeFactLimits, RuntimeFaultCode,
    };
    use crate::game::{battle::runtime_facts::RuntimeFactSource, behavior::GameError};
    let text = base().replace("automatic_unit", "self").replace("EnemySingle(rule: Nearest)", "SelfUnit")
        .replace("reach: global", "reach: pattern").replace("target: CastTarget", "target: SelfUnit")
        .replace("delay_ms: 200", "delay_ms: 0")
        .replace("use_lifetime: Action", "use_lifetime: ActionAndEffects(effects: [(step: \"hit\", effect: \"damage\")])")
        .replace("Damage(amount: 1, damage_type: True)", "Sustained(definition: Shield(duration_ms: 1000, amount: 20, priority: 0, shared_radius_units: None))");
    let (mut core, mut state, actor, _) = fixture(&text);
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 300).unwrap();
    let source = RuntimeFactSource {
        actor: Some(actor),
        action: Some(action),
        authored: core
            .game_data
            .skill_data
            .get_by_id("r08_strike")
            .unwrap()
            .steps[0]
            .effects[0]
            .source
            .clone(),
        ..Default::default()
    };
    assert!(source.authored.is_some());
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
    );
    for (case, site) in [
        (0, "resonance_use_obligations"),
        (1, "resonance_effect_reference"),
        (2, "resonance_target_lifecycle"),
    ] {
        let error = native::run(
            RuntimeFactLimits {
                max_work: 1,
                ..Default::default()
            },
            || {
                match case {
                    0 => {
                        let _ = core.resonance_use_has_obligations(actor);
                    }
                    1 => {
                        let _ = core.source_owns_use_effect(&source);
                    }
                    _ => core.cancel_uses_with_unavailable_targets(300)?,
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert!(
            matches!(error, GameError::BattleRuntimeFault(fault) if fault.code == RuntimeFaultCode::WorkLimit && fault.site == site),
            "{error:?}"
        );
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot()
            ),
            before
        );
    }
    assert!(native::run(Default::default(), || Ok(
        core.resonance_use_has_obligations(actor)
    ))
    .unwrap());
    core.step_battle_execution_until(&mut state, 1100).unwrap();
    assert!(!core.resonance_use_has_obligations(actor));
}

#[test]
fn u05_independent_dot_permanent_and_area_do_not_extend_use() {
    for operation in [
        "ApplyBuff(buff_id: \"poison\", duration_ms: 3000)",
        "ModifyStats(modifier: (stat: Attack, kind: Flat, value: -1))",
        "Sustained(definition: Control(duration_ms: 3000, kind: Root))",
    ] {
        let text = base().replace("Damage(amount: 1, damage_type: True)", operation);
        let (mut core, mut state, actor, _) = fixture(&text);
        core.request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 300).unwrap();
        assert!(!core.resonance_use_has_obligations(actor), "{operation}");
        core.add_resonance(actor, 1, 300, false);
        assert_eq!(core.units[&actor].resonance_current, 1);
    }
    let text = base().replace("delivery: Instant", r#"delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]), area: (anchor: CastTarget, tile_origin: Anchor, duration_ms: 3000, tick_interval_ms: Some(1000)))"#);
    let (mut core, mut state, actor, _) = fixture(&text);
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 300).unwrap();
    assert!(core
        .query_action(&action)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    assert!(!core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.units[&actor].resonance_current, 1);
}

#[test]
fn u05_independent_area_attacks_never_reopen_the_creator_use() {
    let text = base()
        .replace("Damage(amount: 1, damage_type: True)", "ExtraAttack(count: 3)")
        .replace("delivery: Instant", r#"delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]), area: (anchor: CastTarget, tile_origin: Anchor, duration_ms: 3000, tick_interval_ms: Some(1000)))"#);
    let (mut core, mut state, actor, target) = fixture(&text);
    core.units.get_mut(&actor).unwrap().stats.attack = 1;
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert!(!core.active_areas.is_empty());
    assert!(core
        .query_action(&action)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    assert!(!core.resonance_use_has_obligations(actor));
    assert_eq!(core.units[&target].stats.current_health, 100);
    core.add_resonance(actor, 7, 301, false);
    assert_eq!(core.units[&actor].resonance_current, 7);
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.units[&target].stats.current_health, 97);
    assert_eq!(core.units[&actor].resonance_current, 8);
    core.step_battle_execution_until(&mut state, 1301).unwrap();
    assert!(!core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 2000).unwrap();
    assert_eq!(core.units[&target].stats.current_health, 94);
    assert_eq!(core.units[&actor].resonance_current, 9);
}

#[test]
fn u05_finite_modifiers_survival_and_overlay_expire_at_actual_lease_end() {
    for definition in [
        "Modifiers(duration_ms: 1000, modifiers: [(stat: Attack, kind: Flat, value: 1)])",
        "Survival(duration_ms: 1000, policy: FloorOne)",
        "BasicAttackOverlay(duration_ms: 1000, next_attack_only: false, profile: (range_policy: whole_field_valid_tiles, defense_tile_range: None, delivery: Instant))",
    ] {
        let text = base().replace("automatic_unit", "self").replace("EnemySingle(rule: Nearest)", "SelfUnit")
            .replace("reach: global", "reach: pattern").replace("target: CastTarget", "target: SelfUnit").replace("delay_ms: 200", "delay_ms: 0")
            .replace("use_lifetime: Action", "use_lifetime: ActionAndEffects(effects: [(step: \"hit\", effect: \"damage\")])")
            .replace("Damage(amount: 1, damage_type: True)", &format!("Sustained(definition: {definition})"));
        let (mut core, mut state, actor, _) = fixture(&text);
        core.request_skill_action(0, actor, "r08_strike".into(), None).unwrap();
        core.step_battle_execution_until(&mut state, 1099).unwrap();
        assert!(core.resonance_use_has_obligations(actor), "{definition}");
        assert_eq!(core.units[&actor].resonance_current, 0);
        core.step_battle_execution_until(&mut state, 1100).unwrap();
        assert!(!core.resonance_use_has_obligations(actor), "{definition}");
        core.add_resonance(actor, 1, 1100, false);
        assert_eq!(core.units[&actor].resonance_current, 1);
    }
}

#[test]
fn u05_mixed_uses_wait_for_last_selected_lease_but_not_independent_area() {
    let text = r#"SkillDatabase(skills: [SkillDef(
      id: "r08_strike", name: "Mixed obligations", kind: Targeted, focus_time_ms: 100,
      use_lifetime: ActionAndEffects(effects: [(step: "self", effect: "boost"), (step: "self", effect: "shield")]),
      cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
      steps: [SkillStepDef(id: "self", delay_ms: 0, target: SelfUnit, delivery: Instant,
        effects: [(id: "boost", op: Sustained(definition: Modifiers(duration_ms: 600, modifiers: [(stat: Attack, kind: Flat, value: 1)]))),
                  (id: "shield", op: Sustained(definition: Shield(duration_ms: 1000, amount: 20, priority: 0, shared_radius_units: None)))]),
       SkillStepDef(id: "area", delay_ms: 0, target: SelfUnit,
        delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]), area: (anchor: Caster, tile_origin: Caster, duration_ms: 3000, tick_interval_ms: Some(1000))),
        effects: [(id: "damage", op: Damage(amount: 1, damage_type: True))])]
    )])"#;
    let (mut core, mut state, actor, _) = fixture(text);
    let action = core
        .request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 699).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 2);
    core.step_battle_execution_until(&mut state, 700).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    assert!(core.resonance_use_has_obligations(actor));
    core.add_resonance(actor, 9, 700, false);
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.step_battle_execution_until(&mut state, 1100).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(core
        .query_action(&action)
        .unwrap()
        .effects_settled_at_ms
        .is_none());
    assert!(!core.resonance_use_has_obligations(actor));
    core.add_resonance(actor, 9, 1100, false);
    assert_eq!(core.units[&actor].resonance_current, 9);
}

#[test]
fn u05_selected_finite_projectile_preserves_obligation_from_launch_through_lease() {
    let text = base().replace("delay_ms: 200", "delay_ms: 0")
        .replace("use_lifetime: Action", "use_lifetime: ActionAndEffects(effects: [(step: \"hit\", effect: \"damage\")])")
        .replace("Damage(amount: 1, damage_type: True)", "Sustained(definition: Modifiers(duration_ms: 1000, modifiers: [(stat: Attack, kind: Flat, value: 1)]))")
        .replace("effects: [(id: \"damage\"", "effects: [(id: \"hostile\", op: Damage(amount: 1, damage_type: True)), (id: \"damage\"")
        .replace("delivery: Instant", "delivery: Projectile(speed_units_per_ms: 1000)");
    let (mut core, mut state, actor, _) = fixture(&text);
    core.request_skill_action(0, actor, "r08_strike".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 100).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(core.resonance_use_has_obligations(actor));
    core.add_resonance(actor, 7, 100, false);
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.step_battle_execution_until(&mut state, 1500).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    assert!(core.resonance_use_has_obligations(actor));
    let expiry = core.sustained_effect_snapshot().unwrap().active[0].expires_at_ms;
    core.step_battle_execution_until(&mut state, expiry - 1)
        .unwrap();
    assert!(core.resonance_use_has_obligations(actor));
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.step_battle_execution_until(&mut state, expiry)
        .unwrap();
    assert!(!core.resonance_use_has_obligations(actor));
    let at_expiry = core.units[&actor].resonance_current;
    core.add_resonance(actor, 7, expiry, false);
    assert_eq!(core.units[&actor].resonance_current, at_expiry + 7);
    core.step_battle_execution_until(&mut state, (expiry / 1000 + 1) * 1000)
        .unwrap();
    assert_eq!(core.units[&actor].resonance_current, at_expiry + 8);
}
