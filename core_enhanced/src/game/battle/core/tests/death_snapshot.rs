use super::*;

fn fixture(
    blocked: bool,
) -> (
    BattleCore,
    sim::BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let skill = single_step_skill(
        "predeath_blocker_fixture",
        SkillTarget::EnemySingle {
            rule: UnitTargetRule::Nearest,
        },
        3,
        10,
        DeliveryDef::Instant,
        vec![SkillEffectDef::Damage {
            amount: 7,
            damage_type: DamageType::True,
        }],
    );
    let (mut core, state, killer, victim, blocker) =
        native_action_fixture(skill, Default::default());
    Arc::make_mut(&mut core.scenario).tactical_plan.player_plan =
        crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;
    for unit in core.units.values_mut() {
        unit.stats.move_speed_units_per_ms = 0;
    }
    core.units.get_mut(&victim).unwrap().stats.current_health = 1;
    if blocked {
        let unit = core.units.get_mut(&blocker).unwrap();
        unit.block_capacity = 1;
        unit.block_radius_units = 1.5;
        place_unit(&mut core, blocker, Position::new(2, 2));
        sync_unit_to_battlefield_tile_center(&mut core, blocker);
    }
    core.refresh_block_state();
    assert_eq!(core.blocked_by(victim), blocked.then_some(blocker));
    (core, state, killer, victim, blocker)
}

#[test]
fn predeath_blocker_actual_lethal_attack_preserves_identity_and_serializes() {
    for blocked in [false, true] {
        for observed in [false, true] {
            let (mut core, mut state, killer, victim, blocker) = fixture(blocked);
            if observed {
                core.enable_authoring_observation_with_options(Default::default(), true);
                core.bind_authoring_observation(
                    crate::game::battle::core::authoring_observation::ObservationKey {
                        entry_ordinal: 1,
                        battle_uuid: uuid::Uuid::from_u128(0xb17a),
                    },
                )
                .unwrap();
            }
            let action = core
                .request_skill_action(0, killer, "predeath_blocker_fixture".into(), None)
                .unwrap();
            core.step_battle_execution_until(&mut state, 20).unwrap();
            assert!(core.units[&victim].lifecycle.is_dead());
            let fact = core
                .runtime_fact_summary()
                .latest_death
                .as_ref()
                .unwrap()
                .clone();
            assert_eq!(fact.source.actor, Some(killer));
            assert_eq!(fact.predeath.victim, victim);
            assert_eq!(fact.predeath.blocker, blocked.then_some(blocker));
            assert_ne!(killer, blocker);
            assert!(fact.lethal_damage.is_some());
            assert_eq!(
                core.query_action(&action)
                    .unwrap()
                    .facts
                    .latest_death
                    .as_ref()
                    .unwrap()
                    .predeath
                    .blocker,
                fact.predeath.blocker
            );
            let json = serde_json::to_value(&fact).unwrap();
            assert_eq!(
                json["predeath"]["blocker"],
                serde_json::to_value(blocked.then_some(blocker)).unwrap()
            );
            core.refresh_block_state();
            assert_eq!(core.blocked_by(victim), None);
            core.finalize_unit_death(Some(killer), victim, Side::Opponent, 21);
            assert_eq!(core.runtime_fact_summary().deaths, 1);
            assert_eq!(
                core.runtime_fact_summary()
                    .latest_death
                    .as_ref()
                    .unwrap()
                    .predeath
                    .blocker,
                blocked.then_some(blocker)
            );
        }
    }
}

#[test]
fn predeath_blocker_prepared_snapshot_survives_engagement_cleanup() {
    let (mut core, _, killer, victim, blocker) = fixture(true);
    let prepared = core
        .prepare_death_fact(Some(killer), victim, 10)
        .unwrap()
        .unwrap();
    core.units.get_mut(&blocker).unwrap().block_capacity = 0;
    core.refresh_block_state();
    assert_eq!(core.blocked_by(victim), None);
    core.finalize_prepared_death(Some(killer), victim, Side::Opponent, 10, prepared, None);
    assert_eq!(
        core.runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .predeath
            .blocker,
        Some(blocker)
    );
}

#[test]
fn predeath_blocker_cancelled_lethal_timestamp_restores_fact_and_engagement() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    let (mut core, mut state, killer, victim, blocker) = fixture(true);
    core.request_skill_action(0, killer, "predeath_blocker_fixture".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 9).unwrap();
    let before = native_action_probe(&core, &state);
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("runtime_timestamp_commit", 1, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || core.step_battle_execution_until(&mut state, 10),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!(native_action_probe(&core, &state), before);
    assert_eq!(core.blocked_by(victim), Some(blocker));
    core.step_battle_execution_until(&mut state, 20).unwrap();
    assert_eq!(core.runtime_fact_summary().deaths, 1);
    assert_eq!(
        core.runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .predeath
            .blocker,
        Some(blocker)
    );
}
