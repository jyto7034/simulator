#[test]
fn u06_selection_deployment_focus_settings_and_disconnect_share_one_restore() {
    use crate::game::behavior::{ActionDisabledReasonCode as Reason, BattlePlaybackSpeed as Speed};
    let TacticalWorldMutationFixture { mut core, player_id, first_employee_uuid, second_employee_uuid, first_position, .. } = tactical_world_mutation_fixture(0xC00610);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::SetBattleSpeed { speed: Speed::X2 }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::BeginDeploymentSelection { employee_uuid: first_employee_uuid }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.playback.speed, Speed::X0_5);
    assert_eq!(active.interaction.restore_playback.unwrap().speed, Speed::X2);
    assert_eq!(active.live_deployment.as_ref().unwrap().deployed_units.len(), 0);
    assert_future_cursor_rejection_is_atomic(&mut core, player_id, PlayerBehavior::CancelDeploymentSelection);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::DeployUnit {
        employee_uuid: first_employee_uuid, position: first_position, facing: FacingDirection::Right,
    }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let unit_id = active.live_deployment.as_ref().unwrap().deployed_units[&first_employee_uuid].unit_instance_id;
    assert_eq!(active.playback.speed, Speed::X2);
    assert_eq!(active.interaction.deployment_employee_uuid, None);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: unit_id }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::BeginDeploymentSelection { employee_uuid: second_employee_uuid }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ClearBattleFocus).unwrap();
    assert_eq!(core.state.active_battle.as_ref().unwrap().playback.speed, Speed::X0_5);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: unit_id }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::CancelDeploymentSelection).unwrap();
    let before = battle_advance_atomic_snapshot(&core);
    assert!(matches!(execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::SetBattleSpeed { speed: Speed::X1 }),
        Err(GameError::ActionUnavailable { reason: Reason::SelectionActive, .. })));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    assert!(matches!(execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: Uuid::from_u128(0xBAD).into() }),
        Err(GameError::ActionUnavailable { reason: Reason::UnitUnavailable, .. })));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);

    let original = core.state.active_battle.as_ref().unwrap();
    let mut copied = crate::authoring::clone_shape::controlled_clone(original).unwrap();
    copied.interaction.focused_unit_instance_id = None;
    copied.sync_selection_playback();
    assert_eq!(copied.playback.speed, Speed::X2);
    assert_eq!(original.playback.speed, Speed::X0_5);
    assert_eq!(original.interaction.focused_unit_instance_id, Some(unit_id));

    assert_future_cursor_rejection_is_atomic(&mut core, player_id, PlayerBehavior::OpenBattleSettings);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::OpenBattleSettings).unwrap();
    let saved = battle_advance_atomic_snapshot(&core);
    for command in [
        PlayerBehavior::ResumeBattle,
        PlayerBehavior::SetBattleSpeed { speed: Speed::X2 },
        PlayerBehavior::ClearBattleFocus,
        PlayerBehavior::WithdrawUnit { employee_uuid: first_employee_uuid },
    ] {
        assert!(matches!(execute_current_battle_mutation(&mut core, player_id, command),
            Err(GameError::ActionUnavailable { reason: Reason::SettingsOpen, .. })));
        assert_eq!(battle_advance_atomic_snapshot(&core), saved);
    }
    assert!(matches!(execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::OpenBattleSettings),
        Err(GameError::ActionUnavailable { reason: Reason::AlreadyActive, .. })));
    assert_eq!(battle_advance_atomic_snapshot(&core), saved);
    assert!(core.advance_active_battle_for_server_tick(20_000).unwrap().is_none());
    assert_eq!(battle_advance_atomic_snapshot(&core), saved);
    let query = core.execute(player_id, PlayerBehavior::RequestBattleState { since_seq: None }).unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = query else { panic!("checkpoint") };
    assert_eq!(battle_update.checkpoint.interaction, core.state.active_battle.as_ref().unwrap().interaction_dto());
    assert!(battle_update.checkpoint.playback_eligibility.speeds.iter().all(|speed| speed.eligibility.disabled_reason_code == Some(Reason::SettingsOpen)));
    assert!(core.auto_pause_active_battle_for_disconnect().unwrap(), "disconnect must also mark the settings restore as paused");
    assert!(!core.auto_pause_active_battle_for_disconnect().unwrap());
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::CloseBattleSettings).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.playback.paused);
    assert_eq!(active.playback.speed, Speed::X0_5);
    assert!(!active.settings_open());
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ClearBattleFocus).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.playback.paused);
    assert_eq!(active.playback.speed, Speed::X2);
    assert_eq!(active.execution.last_event_time_ms(), 0);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: unit_id }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::WithdrawUnit { employee_uuid: first_employee_uuid }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.interaction.focused_unit_instance_id, None);
    assert_eq!(active.playback.speed, Speed::X2);
}

#[test]
fn u06_forced_defense_uses_paused_and_selected_host_battle_clock() {
    use crate::game::{battle::types::EngagementStance, behavior::BattlePlaybackSpeed as Speed};
    let TacticalWorldMutationFixture { mut core, player_id, first_employee_uuid, first_position, second_position, .. } = tactical_world_mutation_fixture(0xC00611);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::DeployUnit {
        employee_uuid: first_employee_uuid, position: first_position, facing: FacingDirection::Right,
    }).unwrap();
    let unit_id = core.state.active_battle.as_ref().unwrap().live_deployment.as_ref().unwrap().deployed_units[&first_employee_uuid].unit_instance_id;
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: unit_id }).unwrap();
    let command = crate::game::behavior::TacticalCommandPreviewInput::Relocate {
        employee_uuid: first_employee_uuid, destination: second_position, facing: FacingDirection::Left,
    };
    let before_preview = battle_advance_atomic_snapshot(&core);
    for _ in 0..2 {
        let preview = core.execute(player_id, PlayerBehavior::PreviewTacticalCommand { command: command.clone() }).unwrap();
        let BehaviorResult::TacticalCommandPreview { result } = preview else { panic!("tactical preview") };
        assert_eq!(result.cost, 5);
        assert!(result.eligibility.enabled);
        assert_eq!(result.units[0].destination, second_position);
        assert_eq!(result.units[0].facing, FacingDirection::Left);
        assert_eq!(result.units[0].forced_defense_ms, 1500);
        assert_eq!(battle_advance_atomic_snapshot(&core), before_preview);
    }
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::TacticalRelocateUnit {
        employee_uuid: first_employee_uuid, destination: second_position, facing: FacingDirection::Left,
    }).unwrap();
    let before_stale_preview = battle_advance_atomic_snapshot(&core);
    let preview = core.execute(player_id, PlayerBehavior::PreviewTacticalCommand { command }).unwrap();
    let BehaviorResult::TacticalCommandPreview { result } = preview else { panic!("tactical preview") };
    assert_eq!(result.eligibility.disabled_reason_code, Some(crate::game::behavior::ActionDisabledReasonCode::RelocationCooldown));
    assert_eq!(battle_advance_atomic_snapshot(&core), before_stale_preview);
    assert_eq!(core.state.active_battle.as_ref().unwrap().battle.unit_tactical_snapshot(unit_id, 0).unwrap().forced_defense_remaining_ms, 1500);
    assert!(core.advance_active_battle_for_server_tick(20_000).unwrap().is_none());
    assert_eq!(core.state.active_battle.as_ref().unwrap().execution.last_event_time_ms(), 0);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::OpenBattleSettings).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::CloseBattleSettings).unwrap();
    assert!(core.state.active_battle.as_ref().unwrap().playback.paused, "settings closing preserves prior manual Pause");
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle).unwrap();
    assert_eq!(core.state.active_battle.as_ref().unwrap().playback.speed, Speed::X0_5);
    core.advance_active_battle_for_server_tick(2998).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.execution.last_event_time_ms(), 1499);
    let before = active.battle.unit_tactical_snapshot(unit_id, 1499).unwrap();
    assert_eq!(before.stance, EngagementStance::Defense);
    assert_eq!(before.forced_defense_remaining_ms, 1);
    core.advance_active_battle_for_server_tick(2).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    let after = active.battle.unit_tactical_snapshot(unit_id, 1500).unwrap();
    assert_eq!(after.stance, EngagementStance::Attack);
    assert_eq!(after.forced_defense_until_ms, None);
    assert_eq!(after.stance_cooldown_until_ms, 1500);
}

#[test]
fn u06_tactical_swap_preview_and_ineligible_roster_rows_are_read_only() {
    use crate::game::behavior::{TacticalCommandPreviewInput, ActionDisabledReasonCode as Reason};
    let TacticalWorldMutationFixture { mut core, player_id, first_employee_uuid, second_employee_uuid, first_position, second_position } = tactical_world_mutation_fixture(0xC00612);
    for (employee_uuid, position) in [(first_employee_uuid, first_position), (second_employee_uuid, second_position)] {
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::DeployUnit {
            employee_uuid, position, facing: FacingDirection::Right,
        }).unwrap();
    }
    let command = TacticalCommandPreviewInput::Swap { first_employee_uuid, second_employee_uuid,
        first_facing: FacingDirection::Left, second_facing: FacingDirection::Down };
    let before = battle_advance_atomic_snapshot(&core);
    let result = core.execute(player_id, PlayerBehavior::PreviewTacticalCommand { command: command.clone() }).unwrap();
    let BehaviorResult::TacticalCommandPreview { result } = result else { panic!("swap preview") };
    assert_eq!(result.eligibility.disabled_reason_code, Some(Reason::InsufficientResources));
    assert_eq!(result.units.len(), 2);
    assert_eq!(result.units[0].destination, second_position);
    assert_eq!(result.units[1].destination, first_position);
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    // Isolate the shared final-payload validation from elapsed-time recovery.
    core.state.active_battle.as_mut().unwrap().tactical_resource.apply_delta(5);
    let before = battle_advance_atomic_snapshot(&core);
    let result = core.execute(player_id, PlayerBehavior::PreviewTacticalCommand { command }).unwrap();
    let BehaviorResult::TacticalCommandPreview { result } = result else { panic!("swap preview") };
    assert!(result.eligibility.enabled);
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::TacticalSwapUnits {
        first_employee_uuid, second_employee_uuid, first_facing: FacingDirection::Left, second_facing: FacingDirection::Down,
    }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    for row in result.units {
        assert_eq!(active.battle.unit_projected_tile(row.unit_instance_id), Some(row.destination));
        let state = active.battle.unit_tactical_snapshot(row.unit_instance_id, 0).unwrap();
        assert_eq!(state.forced_defense_remaining_ms, row.forced_defense_ms);
    }
    assert_eq!(active.tactical_resource.current(), 0);
    let undeployed = active.employee_roster().iter().find(|e| e.uuid != first_employee_uuid && e.uuid != second_employee_uuid).unwrap().uuid;
    core.state.active_battle.as_mut().unwrap().employee_attempt.roster.get_mut(&undeployed).unwrap().condition.physical_damage = crate::game::employee::MAX_PHYSICAL_DAMAGE - 1;
    let active = core.state.active_battle.as_ref().unwrap();
    let deployment = active.live_deployment_dto(active.employee_roster()).unwrap();
    let row = deployment.unit_deploy_costs.iter().find(|e| e.employee_uuid == undeployed).expect("ineligible roster members remain visible");
    assert_eq!(row.eligibility.disabled_reason_code, Some(Reason::CriticalPhysicalDamage), "condition wins over zero TP");
}

#[test]
fn u06_general_and_skill_focus_cancel_success_and_failure_share_restore() {
    use crate::game::behavior::BattlePlaybackSpeed as Speed;
    let (mut core, player_id, employee_uuid, skill_id, caster_id, target_id) = player_unit_skill_focus_fixture();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::SetBattleSpeed { speed: Speed::X2 }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: target_id }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::BeginSkillTargeting { employee_uuid, skill_id: skill_id.clone() }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.tactical_focus_dto().unwrap().restore_playback.speed, Speed::X2);
    assert_eq!(active.interaction.restore_playback.unwrap().speed, Speed::X2);
    let before = battle_advance_atomic_snapshot(&core);
    assert!(execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ActivateSkill {
        employee_uuid, skill_id: skill_id.clone(), target: None,
    }).is_err());
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::FocusBattleUnit { unit_instance_id: caster_id }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::CancelSkillTargeting).unwrap();
    assert_eq!(core.state.active_battle.as_ref().unwrap().playback.speed, Speed::X0_5);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::BeginSkillTargeting { employee_uuid, skill_id: skill_id.clone() }).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ActivateSkill {
        employee_uuid, skill_id, target: Some(SkillCastTarget::Unit { unit_instance_id: target_id }),
    }).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.tactical_focus.is_none());
    assert_eq!(active.playback.speed, Speed::X0_5);
    assert_eq!(active.interaction.focused_unit_instance_id, Some(caster_id));
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ClearBattleFocus).unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.playback.paused);
    assert_eq!(active.playback.speed, Speed::X2);
    assert_eq!(active.interaction.restore_playback, None);
}
