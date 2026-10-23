fn input_plan_world_fixture(deploy: bool) -> (GameCore, Uuid, Uuid) {
    let mut doc = crate::authoring::node_graph::tests::input_plan_fixture();
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-stock" && n["title"].as_str().unwrap().contains("일반") {
            n["values"]["initial"] = json!(1);
        }
    }
    let native = crate::authoring::node_graph::compile(&doc.to_string(), "r08_strike".into())
        .unwrap()
        .load_content()
        .unwrap();
    let ron = ron::ser::to_string(&native.skills).unwrap();
    r12_live_cross_deployment_fixture(false, false, 0, Some(&ron), false, true, false, deploy)
}
#[test]
fn input_plan_world_equipped_undeployed_copy_generates_stock_and_hostile_without_reset_on_deploy() {
    let (mut c, player, employee) = input_plan_world_fixture(false);
    let initial = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .input_plans_snapshot()
        .unwrap();
    assert_eq!(initial.owners.len(), 1);
    assert_eq!(initial.owners[0].caster, None);
    assert_eq!(initial.owners[0].normal, 1);
    c.advance_active_battle_for_server_tick(5000).unwrap();
    let active = c.state.active_battle.as_ref().unwrap();
    let view = active.battle.input_plans_snapshot().unwrap();
    assert_eq!(view.owners[0].normal, 6);
    assert_eq!(view.cohort_units.len(), 1);
    let position = active
        .combat_preview
        .deployment_zones
        .iter()
        .find(|z| z.kind == DeploymentZoneKind::Ground)
        .unwrap()
        .cells[0];
    execute_current_battle_mutation(
        &mut c,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let view = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .input_plans_snapshot()
        .unwrap();
    assert!(view.owners[0].caster.is_some());
    assert_eq!(view.owners[0].normal, 6);
    assert_eq!(view.owners[0].next_stock_ms, 6000);
}
#[test]
fn input_plan_world_pause_cancel_disconnect_commit_replay_are_atomic_across_playback_speeds() {
    use crate::game::ability::input_plan::*;
    use crate::game::behavior::{BattlePlaybackSpeed, BattleStateAuthorityDto};
    for (speed, paused) in [
        (BattlePlaybackSpeed::X2, false),
        (BattlePlaybackSpeed::X0_5, false),
        (BattlePlaybackSpeed::X2, true),
    ] {
        let (mut c, player, employee) = input_plan_world_fixture(true);
        execute_current_battle_mutation(&mut c, player, PlayerBehavior::SetBattleSpeed { speed })
            .unwrap();
        if paused {
            execute_current_battle_mutation(&mut c, player, PlayerBehavior::PauseBattle).unwrap();
        }
        let activate = PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "r08_strike".into(),
            target: None,
        };
        execute_current_battle_mutation(&mut c, player, activate.clone()).unwrap();
        let active = c.state.active_battle.as_ref().unwrap();
        let battle_uuid = active.battle_uuid;
        let view = active.battle.input_plans_snapshot().unwrap();
        let preparation = view.preparation.unwrap();
        let revision = active.battle_state_revision;
        assert!(active.playback.paused);
        let clock = active.execution.last_event_time_ms();
        let before = battle_advance_atomic_snapshot(&c);
        assert!(c
            .advance_active_battle_for_server_tick(2000)
            .unwrap()
            .is_none());
        assert_eq!(before, battle_advance_atomic_snapshot(&c));
        assert!(
            execute_current_battle_mutation(&mut c, player, PlayerBehavior::ResumeBattle).is_err()
        );
        assert!(execute_current_battle_mutation(
            &mut c,
            player,
            PlayerBehavior::SetBattleSpeed {
                speed: BattlePlaybackSpeed::X1
            }
        )
        .is_err());
        execute_current_battle_mutation(
            &mut c,
            player,
            PlayerBehavior::CancelInputPlan {
                battle_uuid,
                revision,
                employee_uuid: employee,
                preparation: preparation.id,
            },
        )
        .unwrap();
        assert_eq!(
            c.state.active_battle.as_ref().unwrap().playback,
            crate::game::behavior::BattlePlaybackState { paused, speed }
        );
        execute_current_battle_mutation(&mut c, player, activate.clone()).unwrap();
        assert!(c.auto_pause_active_battle_for_disconnect().unwrap());
        assert!(
            c.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .input_plans_snapshot()
                .unwrap()
                .disconnected
        );
        execute_current_battle_mutation(&mut c, player, activate).unwrap();
        let active = c.state.active_battle.as_ref().unwrap();
        let preparation = active
            .battle
            .input_plans_snapshot()
            .unwrap()
            .preparation
            .unwrap();
        let revision = active.battle_state_revision;
        let input = ShotCommitInput {
            preparation: preparation.id,
            request: 25,
            layout: ShotLayoutInput::Fresh(vec![ShotPlacement {
                kind: ShotKind::Normal,
                tile: preparation.placement_tiles[0],
            }]),
            aim: ShotAimInput::Precise(vec![ShotDirection { x: 1, y: 0 }]),
        };
        let commit = PlayerBehavior::CommitInputPlan {
            battle_uuid,
            revision,
            employee_uuid: employee,
            input,
        };
        execute_current_battle_mutation(&mut c, player, commit.clone()).unwrap();
        let active = c.state.active_battle.as_ref().unwrap();
        assert!(!active.playback.paused);
        assert_eq!(active.playback.speed, BattlePlaybackSpeed::X1);
        assert_eq!(active.execution.last_event_time_ms(), clock);
        let view = active.battle.input_plans_snapshot().unwrap();
        assert_eq!(view.receipts.len(), 1);
        assert_eq!(view.active_projectiles.len(), 1);
        assert_eq!(view.owners[0].normal, 0);
        let before = battle_advance_atomic_snapshot(&c);
        c.execute_with_transport_context(
            player,
            commit,
            None,
            0,
            Some(BattleStateAuthorityDto {
                battle_uuid,
                battle_state_revision: revision,
            }),
        )
        .unwrap();
        assert_eq!(before, battle_advance_atomic_snapshot(&c));
    }
}
