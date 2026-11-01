// Current authored program in a technical World fixture. Actual canonical boss
// acquisition/publication is verified separately; this isolates menu admission.
fn counter_world_fixture(bloom: bool) -> (GameCore, Uuid, Uuid) {
    use crate::authoring::composition::{canonical_hatred_tests as source, compile_json};
    let mut document = source::resolved(bloom);
    source::rename_root(&mut document, "r08_strike");
    // Shorten only this technical fixture's periodic gain to reach base menu
    // admission before its short battle ends. Every menu copy must agree.
    for v in document["values"].as_array_mut().unwrap() {
        v["value"]["Effects"][0]["op"]["CounterChannel"]["program"]["interval_ms"] = json!(1000);
    }
    let native = compile_json(&document.to_string()).unwrap().load_native().unwrap();
    r12_live_cross_deployment_fixture_with_data(false, false, 0,
        Some(&ron::ser::to_string(&native).unwrap()), false, true, false, true,
        |builder| builder.with_relics(source::relics()))
}
fn counter_world_owner(core: &GameCore) -> crate::game::ability::counter_channel::CounterOwnerSnapshot {
    core.state.active_battle.as_ref().unwrap().battle.counter_channels_snapshot().unwrap().owners.into_iter().next().unwrap()
}

#[test]
fn counter_world_authored_nondefault_menu_focus_and_activation_match_capability() {
    for bloom in [false, true] {
        let (mut core, player, employee) = counter_world_fixture(bloom);
        if !bloom { core.advance_active_battle_for_server_tick(3000).unwrap(); }
        let owner = counter_world_owner(&core);
        assert_eq!(owner.count, if bloom {0} else {3});
        let menu = owner.menu.iter().find(|m| m.key == "black-hole").unwrap();
        assert!(menu.counter_eligible && menu.resonance_eligible);
        let skill = menu.skill_id.clone();
        let position = core.state.active_battle.as_ref().unwrap().battle.unit_projected_tile(owner.caster).unwrap();
        let result = execute_current_battle_mutation(&mut core, player,
            PlayerBehavior::BeginSkillTargeting { employee_uuid: employee, skill_id: skill.clone() }).unwrap();
        let BehaviorResult::BattleSkillTargetingFocusChanged {battle_update, ..} = result else {panic!()};
        assert!(battle_update.action_capabilities.iter().any(|c| c.action == ActionKind::ActivateSkill && c.enabled),
            "a valid nondefault focus must enable its activation");
        let before = battle_advance_atomic_snapshot(&core);
        assert!(execute_current_battle_mutation(&mut core, player,
            PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: "r08_strike".into(),
                target: Some(crate::game::battle::event_log::SkillCastTarget::Tile {position}) }).is_err());
        assert_eq!(before, battle_advance_atomic_snapshot(&core));
        execute_current_battle_mutation(&mut core, player,
            PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: skill.clone(),
                target: Some(crate::game::battle::event_log::SkillCastTarget::Tile {position}) }).unwrap();
        assert_eq!(counter_world_owner(&core).count, 0);
        core.advance_active_battle_for_server_tick(1).unwrap();
        let active = counter_world_owner(&core).active.unwrap();
        assert_eq!(active.skill_id, skill);
        assert_eq!(active.consumed, if bloom {0} else {3});
        assert!(!active.automatic);
    }
}

#[test]
fn counter_world_authored_menu_admission_cancel_retry_and_stale_authority_are_atomic() {
    use crate::game::behavior::BattleStateAuthorityDto;
    let (mut core, player, employee) = counter_world_fixture(true);
    let owner = counter_world_owner(&core);
    let menu = owner.menu.iter().find(|m| m.key == "black-hole").unwrap();
    let skill = menu.skill_id.clone();
    let position = core.state.active_battle.as_ref().unwrap().battle.unit_projected_tile(owner.caster).unwrap();
    let before = battle_advance_atomic_snapshot(&core);
    // The automatic menu is never a player selection, even with a known ID.
    assert!(execute_current_battle_mutation(&mut core, player,
        PlayerBehavior::BeginSkillTargeting { employee_uuid: employee, skill_id: "r08_strike__entry_2".into() }).is_err());
    assert_eq!(before, battle_advance_atomic_snapshot(&core));
    execute_current_battle_mutation(&mut core, player,
        PlayerBehavior::BeginSkillTargeting { employee_uuid: employee, skill_id: skill.clone() }).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::CancelSkillTargeting).unwrap();
    assert_eq!(counter_world_owner(&core), owner);
    execute_current_battle_mutation(&mut core, player,
        PlayerBehavior::BeginSkillTargeting { employee_uuid: employee, skill_id: skill.clone() }).unwrap();
    let command = PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: skill.clone(),
        target: Some(crate::game::battle::event_log::SkillCastTarget::Tile {position}) };
    let before = battle_advance_atomic_snapshot(&core);
    let token = crate::authoring::control::CancellationToken::default();
    let result = crate::authoring::control::with_test_cancellation("counter_channel_admitted", 1, || {
        crate::authoring::control::run(&crate::authoring::control::ExecutionLimits::default(), &token,
            crate::authoring::control::ExecutionPhase::TrialExecution,
            || execute_current_battle_mutation(&mut core, player, command.clone()))
    });
    assert!(token.is_cancelled(), "admission cancellation seam missed: {result:?}");
    assert!(result.is_err());
    assert_eq!(before, battle_advance_atomic_snapshot(&core));
    let active = core.state.active_battle.as_ref().unwrap();
    let authority = BattleStateAuthorityDto {battle_uuid: active.battle_uuid, battle_state_revision: active.battle_state_revision};
    execute_current_battle_mutation(&mut core, player, command.clone()).unwrap();
    let committed = battle_advance_atomic_snapshot(&core);
    assert!(matches!(core.execute_with_transport_context(player, command, None, 0, Some(authority)),
        Err(GameError::StaleBattleStateAuthority {..})));
    assert_eq!(committed, battle_advance_atomic_snapshot(&core));
}

#[test]
fn counter_world_authored_nondefault_heal_preserves_fragment_and_cast_origin() {
    let (mut core, player, employee) = counter_world_fixture(true);
    let owner = counter_world_owner(&core);
    let skill = owner.menu.iter().find(|m| m.key == "black-hole").unwrap().skill_id.clone();
    let active = core.state.active_battle.as_mut().unwrap();
    let position = active.battle.unit_projected_tile(owner.caster).unwrap();
    let actor = active.battle.units.get_mut(&owner.caster).unwrap();
    actor.stats.current_health -= 100;
    active.battle.enable_authoring_observation();
    execute_current_battle_mutation(&mut core, player,
        PlayerBehavior::BeginSkillTargeting { employee_uuid: employee, skill_id: skill.clone() }).unwrap();
    execute_current_battle_mutation(&mut core, player,
        PlayerBehavior::ActivateSkill { employee_uuid: employee, skill_id: skill.clone(),
            target: Some(crate::game::battle::event_log::SkillCastTarget::Tile {position}) }).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let observation = core.state.active_battle.as_ref().unwrap().battle.authoring_observation().unwrap().unwrap();
    let hp = observation.latest_nonzero_sourced_hp.as_ref().unwrap();
    assert_eq!(hp.fragment.as_ref().expect("menu effect must retain its owning fragment").copy_uuid, owner.source_copy);
    assert_eq!(hp.source.as_ref().expect("menu effect must retain its cast").cast.skill_id, skill);
}
