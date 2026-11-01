use super::*;
use game_core::application::ApplicationCommand;
use game_core::game::behavior::{
    BattlePlaybackSpeed, BattleStateAuthorityDto, LiveBattleUpdateDto,
};

async fn mutation(
    actor: &actix::Addr<PlayerGameActor>,
    session_id: Uuid,
    before: &LiveBattleUpdateDto,
    behavior: ApplicationCommand,
) -> LiveBattleUpdateDto {
    let reply = actor
        .send(ExecutePlayerBehavior {
            session_id,
            request_id: format!("u06-{}", before.battle_state_revision),
            behavior,
            battle_authority: Some(BattleStateAuthorityDto {
                battle_uuid: before.battle_uuid,
                battle_state_revision: before.battle_state_revision,
            }),
            battle_side_message: BattleSideMessageKind::BattleUpdate,
        })
        .await
        .unwrap()
        .unwrap();
    assert!(reply.state_snapshot.is_none());
    let update = reply
        .side_messages
        .into_iter()
        .find_map(|m| {
            if let PlayerGameServerMessage::BattleUpdate(u) = m {
                Some(u)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        update.battle_state_revision,
        before.battle_state_revision + 1
    );
    update
}

#[actix_web::test]
async fn u06_actor_selection_settings_disconnect_and_reconnect_restore_priority() {
    let load_balance = LoadBalanceActor::new().start();
    let player = Uuid::from_u128(0xC006_510C);
    let actor = PlayerGameActor::new(
        player,
        game_core_waiting_on_live_defense_node(player),
        load_balance,
    )
    .with_live_battle_tick_interval(Duration::from_secs(60))
    .start();
    let (probe, _, _) = spawn_probe();
    let session = Uuid::from_u128(0xC006_510D);
    actor
        .send(AttachSession {
            session_id: session,
            socket: probe.clone().recipient(),
            control: probe.recipient(),
        })
        .await
        .unwrap()
        .unwrap();
    let entered = actor
        .send(ExecutePlayerBehavior {
            session_id: session,
            request_id: "u06-enter".into(),
            behavior: command!(ConfirmEnterNode),
            battle_authority: None,
            battle_side_message: BattleSideMessageKind::BattleUpdate,
        })
        .await
        .unwrap()
        .unwrap();
    let mut update = entered
        .side_messages
        .into_iter()
        .find_map(|m| {
            if let PlayerGameServerMessage::BattleUpdate(u) = m {
                Some(u)
            } else {
                None
            }
        })
        .unwrap();
    let employee = update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .unit_deploy_costs
        .iter()
        .find(|u| u.eligibility.enabled)
        .unwrap()
        .employee_uuid;
    update = mutation(
        &actor,
        session,
        &update,
        command!(SetBattleSpeed {
            speed: BattlePlaybackSpeed::X2
        }),
    )
    .await;
    update = mutation(
        &actor,
        session,
        &update,
        command!(BeginDeploymentSelection {
            employee_uuid: employee
        }),
    )
    .await;
    assert_eq!(update.checkpoint.playback.speed, BattlePlaybackSpeed::X0_5);
    update = mutation(&actor, session, &update, command!(OpenBattleSettings)).await;
    assert!(update.checkpoint.playback.paused);
    let rejected = actor
        .send(ExecutePlayerBehavior {
            session_id: session,
            request_id: "u06-menu-resume-rejected".into(),
            behavior: command!(ResumeBattle),
            battle_authority: Some(BattleStateAuthorityDto {
                battle_uuid: update.battle_uuid,
                battle_state_revision: update.battle_state_revision,
            }),
            battle_side_message: BattleSideMessageKind::BattleUpdate,
        })
        .await
        .unwrap();
    assert!(rejected.is_err());
    actor
        .send(DetachSession {
            session_id: session,
        })
        .await
        .unwrap();
    let (probe, _, _) = spawn_probe();
    let next_session = Uuid::from_u128(0xC006_510E);
    let attached = actor
        .send(AttachSession {
            session_id: next_session,
            socket: probe.clone().recipient(),
            control: probe.recipient(),
        })
        .await
        .unwrap()
        .unwrap();
    update = attached
        .side_messages
        .into_iter()
        .find_map(|m| {
            if let PlayerGameServerMessage::BattleUpdate(u) = m {
                Some(u)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        update.checkpoint.interaction.deployment_employee_uuid,
        Some(employee)
    );
    assert!(
        update
            .checkpoint
            .interaction
            .settings_restore_playback
            .unwrap()
            .paused
    );
    assert!(
        update
            .checkpoint
            .interaction
            .restore_playback
            .unwrap()
            .paused
    );
    update = mutation(&actor, next_session, &update, command!(CloseBattleSettings)).await;
    assert!(update.checkpoint.playback.paused);
    assert_eq!(update.checkpoint.playback.speed, BattlePlaybackSpeed::X0_5);
    update = mutation(
        &actor,
        next_session,
        &update,
        command!(CancelDeploymentSelection),
    )
    .await;
    assert!(update.checkpoint.playback.paused);
    assert_eq!(update.checkpoint.playback.speed, BattlePlaybackSpeed::X2);
    update = mutation(&actor, next_session, &update, command!(ResumeBattle)).await;
    assert!(!update.checkpoint.playback.paused);
    assert_eq!(update.checkpoint.playback.speed, BattlePlaybackSpeed::X2);
    assert_eq!(update.checkpoint.battle_time_ms, 0);
}
