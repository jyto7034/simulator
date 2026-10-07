#[test]
fn u03_independent_internal_reconcile_must_fault_application() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xB055_C007);
    let player_id = Uuid::from_u128(0xB055_C007);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected deployed employee");
    };
    {
        let active = core.state.active_battle.as_mut().unwrap();
        let unit = active.battle.units.get_mut(&unit_instance_id).unwrap();
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        active.employee_attempt.roster = crate::game::employee::EmployeeRoster::new();
    }

    let mut app = crate::application::CoreApplication::from_core(core);
    let first = app.advance_active_battle_by(50, 0).expect_err("internal reconciliation must fail");
    println!("first_error={} fault_present={}", first.code(), app.battle_fault().is_some());
    let second = app.advance_active_battle_by(50, 0).expect_err("second must be gated");
    println!("second_error={} fault_present={}", second.code(), app.battle_fault().is_some());
    assert!(app.battle_fault().is_some(), "trusted internal roster inconsistency must terminally fault");
    assert_eq!(second.code(), "battle_faulted");
}
