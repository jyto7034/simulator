#[test]
fn composition_metamorphosis_world_three_real_battles_retreat_permanent_death_asset_recovery_and_checkpoint(
) {
    let fixture = employee_revival_world_fixture_with_transform(true);
    let data = fixture.load();
    let (mut core, player, employee, _) = start_resource_battle(data.clone());
    let copy = core
        .state
        .inventory
        .equipments
        .iter()
        .find(|i| {
            i.meta.id == "placeholder_nothing_there_accessory" && i.equipped_to == Some(employee)
        })
        .unwrap()
        .instance_uuid;
    let fragment = core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .skill_fragments
        .active_copy_uuid()
        .unwrap();
    for expected in [1, 2] {
        reach_employee_revival(&mut core, employee, expected);
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
        .unwrap();
        finish_revival_battle(&mut core, player);
        assert_eq!(revival_world_count(&core, employee, false), expected);
        assert!(core.state.roster.get(&employee).unwrap().is_alive());
        enter_resource_battle(&mut core, player);
    }
    // Keep a second actual employee deployed in the third battle. With only
    // the wearer, its transformation correctly resolves AllPlayerCombatants-
    // Defeated immediately, leaving no active attempt to test Retreat against.
    let guard = core
        .state
        .roster
        .iter()
        .find(|e| e.employee_definition_id == "breach_guard" && e.is_alive())
        .unwrap()
        .uuid;
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    let wearer_position = battle
        .units
        .values()
        .find(|u| u.source_owned_uuid == employee && u.is_active())
        .unwrap()
        .body
        .projected_tile();
    let guard_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|z| z.kind == DeploymentZoneKind::Ground)
        .flat_map(|z| z.cells.iter().copied())
        .filter(|p| battle.player_tactical_slot_occupant(*p, None).is_none())
        .max_by_key(|p| (p.manhattan(&wearer_position), p.y, p.x))
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: guard,
            position: guard_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let entry = core.run_resources().unwrap().clone();
    let old = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units
        .values()
        .find(|u| u.source_owned_uuid == employee && u.is_active())
        .unwrap()
        .instance_id;
    let position = core.state.active_battle.as_ref().unwrap().battle.units[&old].world_position();
    reach_employee_revival(&mut core, employee, 3);
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active
        .employee_roster()
        .get(&employee)
        .unwrap()
        .condition
        .is_dead());
    assert!(
        core.state.roster.get(&employee).unwrap().is_alive(),
        "run death commits only with the result"
    );
    assert_eq!(
        active.battle.units[&old].lifecycle,
        crate::game::battle::core::types::RuntimeUnitLifecycle::PermanentlyDead
    );
    let transformed = active.battle.units.values().find(|u| matches!(
        &u.source_identity, crate::game::battle::types::BattleUnitSourceIdentity::SummonedUnit { definition_id, .. }
            if definition_id == "o-06-20_nothing_there"
    )).unwrap();
    assert_eq!(transformed.world_position(), position);
    assert_eq!(transformed.owner, crate::game::enums::Side::Opponent);
    assert_ne!(transformed.instance_id, old);
    assert!(active
        .battle
        .compendium_candidates()
        .iter()
        .all(|c| c.id != "o-06-20_nothing_there"));
    let mut retreat = core.clone();
    execute_current_battle_mutation(&mut retreat, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(retreat.run_resources(), Some(&entry));
    assert!(retreat.state.roster.get(&employee).unwrap().is_alive());
    assert_eq!(
        retreat
            .state
            .inventory
            .equipments
            .iter()
            .find(|i| i.instance_uuid == copy)
            .unwrap()
            .equipped_to,
        Some(employee)
    );
    assert_eq!(
        retreat
            .state
            .roster
            .get(&employee)
            .unwrap()
            .skill_fragments
            .active_copy_uuid(),
        Some(fragment)
    );
    // Actual public redeployment is denied while the terminal employee remains
    // in the attempt; failure preserves the run and active battle.
    let before = core.get_run_snapshot_json().unwrap();
    let error = execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position: crate::game::resources::Position::new(1, 1),
            facing: crate::game::battle::tile_range::FacingDirection::Right,
        },
    );
    assert!(error.is_err());
    assert_eq!(core.get_run_snapshot_json().unwrap(), before);
    // The normal enemy continues attacking the remaining employee. Defeat
    // must remain available even while the additional required enemy lives.
    finish_revival_battle(&mut core, player);
    assert!(core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .condition
        .is_dead());
    assert_eq!(revival_world_count(&core, employee, false), 3);
    assert_eq!(
        core.state
            .inventory
            .equipments
            .iter()
            .find(|i| i.instance_uuid == copy)
            .unwrap()
            .equipped_to,
        None
    );
    assert_eq!(
        core.state
            .skill_fragments
            .get(&fragment)
            .unwrap()
            .equipped_to,
        None
    );
    assert_eq!(
        core.state
            .roster
            .get(&employee)
            .unwrap()
            .skill_fragments
            .active_copy_uuid(),
        None
    );
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let source = crate::application::CoreApplication::from_core(core);
    let checkpoint = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = crate::application::CoreApplication::new(data, 0xe690);
    install_application_profile_for_continue(&mut restored, &source);
    restored
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    let snapshot = restored
        .snapshot(crate::application::ApplicationSnapshotKind::EmployeeRoster)
        .unwrap();
    assert_eq!(
        snapshot,
        source
            .snapshot(crate::application::ApplicationSnapshotKind::EmployeeRoster)
            .unwrap()
    );
    assert_eq!(restored.run_resources(), source.run_resources());
}
