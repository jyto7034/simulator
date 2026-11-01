// Technical authored inputs exercise the public World bridge; these are not
// the canonical Knight catalog or its unresolved production balance values.
fn relation_world_fixture(bloom: bool) -> (GameCore, Uuid, Uuid) {
    let original = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::relation_action_fixture(bloom).to_string(),
        "r08_strike".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let document = crate::authoring::composition::owned_action_tests::module(&original);
    let authored = crate::authoring::composition::compile_json(&document.to_string())
        .unwrap()
        .load_native()
        .unwrap();
    r12_live_cross_deployment_fixture_with_data(
        false,
        false,
        0,
        Some(&ron::ser::to_string(&authored).unwrap()),
        false,
        true,
        false,
        true,
        |builder| builder.with_relics(crate::authoring::composition::canonical_knight_tests::relics()),
    )
}

fn relation_world_owner(
    core: &GameCore,
) -> crate::game::ability::relation_action::RelationOwnerSnapshot {
    core.state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .relation_actions_snapshot()
        .unwrap()
        .owners
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn relation_world_public_selection_authority_toggle_and_failure_rollback() {
    use crate::application::{ApplicationCommand, ApplicationCommandKind, ApplicationDocument};
    use crate::game::behavior::BattleStateAuthorityDto;
    for bloom in [false, true] {
        let (mut core, player, employee) = relation_world_fixture(bloom);
        let initial = relation_world_owner(&core);
        assert!(initial.menu_available);
        assert!(initial.eligible_targets.contains(&initial.caster));
        assert_eq!(initial.stock, 0);
        let select = PlayerBehavior::SelectSkillRelation {
            employee_uuid: employee,
            entry: initial.menu_entry.clone(),
            target: initial.caster,
        };
        let command = ApplicationCommand::from_document(ApplicationDocument::new(
            serde_json::to_value(&select).unwrap(),
        ))
        .unwrap();
        assert_eq!(command.kind(), ApplicationCommandKind::SelectSkillRelation);
        let via_kind = ApplicationCommand::try_new(
            ApplicationCommandKind::SelectSkillRelation,
            ApplicationDocument::new(json!({"employee_uuid": employee,
                "entry": initial.menu_entry, "target": initial.caster})),
        )
        .unwrap();
        assert_eq!(
            command.document().as_value(),
            via_kind.document().as_value()
        );
        let active = core.state.active_battle.as_ref().unwrap();
        let authority = BattleStateAuthorityDto {
            battle_uuid: active.battle_uuid,
            battle_state_revision: active.battle_state_revision,
        };
        let before = battle_advance_atomic_snapshot(&core);
        assert!(matches!(
            core.execute(player, select.clone()),
            Err(GameError::MissingBattleStateAuthority { .. })
        ));
        assert_eq!(before, battle_advance_atomic_snapshot(&core));
        for invalid in [
            PlayerBehavior::SelectSkillRelation {
                employee_uuid: employee,
                entry: "missing".into(),
                target: initial.caster,
            },
            PlayerBehavior::SelectSkillRelation {
                employee_uuid: Uuid::nil(),
                entry: initial.menu_entry.clone(),
                target: initial.caster,
            },
            PlayerBehavior::SelectSkillRelation {
                employee_uuid: employee,
                entry: initial.menu_entry.clone(),
                target: Uuid::nil().into(),
            },
        ] {
            assert!(execute_current_battle_mutation(&mut core, player, invalid).is_err());
            assert_eq!(before, battle_advance_atomic_snapshot(&core));
        }
        // The future cursor rejects before mutation. Separately cancel at the
        // existing post-selection seam to prove rollback of the new relation
        // generation and its scheduled gain inside the World candidate.
        let token = crate::authoring::control::CancellationToken::default();
        let cancelled =
            crate::authoring::control::with_test_cancellation("relation_target_changed", 1, || {
                crate::authoring::control::run(
                    &crate::authoring::control::ExecutionLimits::default(),
                    &token,
                    crate::authoring::control::ExecutionPhase::TrialExecution,
                    || execute_current_battle_mutation(&mut core, player, select.clone()),
                )
            });
        assert!(
            token.is_cancelled(),
            "post-selection seam was not reached: {cancelled:?}"
        );
        assert!(cancelled.is_err());
        assert_eq!(before, battle_advance_atomic_snapshot(&core));
        assert_future_cursor_rejection_is_atomic(&mut core, player, select.clone());
        let result = execute_current_battle_mutation(&mut core, player, select.clone()).unwrap();
        let BehaviorResult::BattleRelationSelected {
            battle_update,
            employee_uuid,
            unit_instance_id,
            entry,
            ..
        } = result
        else {
            panic!("relation result missing")
        };
        assert_eq!(employee_uuid, employee);
        assert_eq!(unit_instance_id, initial.caster);
        assert_eq!(entry, initial.menu_entry);
        assert!(battle_update
            .action_capabilities
            .iter()
            .any(
                |capability| capability.action == ActionKind::SelectSkillRelation
                    && capability.enabled
            ));
        let selected = relation_world_owner(&core);
        assert_eq!(selected.target, Some(initial.caster));
        assert_eq!(selected.stock, 0);
        assert_eq!(selected.next_gain_ms, Some(1000));
        assert_eq!(
            battle_update.checkpoint.relation_actions.owners,
            vec![selected.clone()]
        );
        assert_eq!(
            battle_update.checkpoint.battle_time_ms,
            before.battle_time_ms
        );
        assert_eq!(
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle_state_revision,
            authority.battle_state_revision + 1
        );
        let committed = battle_advance_atomic_snapshot(&core);
        assert!(matches!(
            core.execute_with_transport_context(player, select.clone(), None, 0, Some(authority)),
            Err(GameError::StaleBattleStateAuthority { .. })
        ));
        assert_eq!(committed, battle_advance_atomic_snapshot(&core));
        // A deliberate second click uses fresh authority and detaches once.
        execute_current_battle_mutation(&mut core, player, select.clone()).unwrap();
        assert_eq!(relation_world_owner(&core).target, None);
        core.advance_active_battle_for_server_tick(1000).unwrap();
        assert_eq!(relation_world_owner(&core).stock, 0);
        execute_current_battle_mutation(&mut core, player, select).unwrap();
        core.advance_active_battle_for_server_tick(1000).unwrap();
        let gained = relation_world_owner(&core);
        assert_eq!(gained.stock, 1);
        assert_ne!(gained.generation, selected.generation);
    }
}

#[test]
fn relation_world_replacement_settings_and_withdraw_preserve_owner_lifetime() {
    for bloom in [false, true] {
        let (mut core, player, employee) = relation_world_fixture(bloom);
        let initial = relation_world_owner(&core);
        let other = core
            .roster()
            .unwrap()
            .combat_eligible_employee_ids()
            .into_iter()
            .find(|id| *id != employee)
            .unwrap();
        let active = core.state.active_battle.as_ref().unwrap();
        let occupied = active.battle.units[&initial.caster].body.projected_tile();
        let position = active
            .combat_preview
            .deployment_zones
            .iter()
            .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
            .flat_map(|zone| zone.cells.iter().copied())
            .find(|cell| *cell != occupied)
            .unwrap();
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: other,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let target = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .live_deployment
            .as_ref()
            .unwrap()
            .deployed_units[&other]
            .unit_instance_id;
        let choose = |target| PlayerBehavior::SelectSkillRelation {
            employee_uuid: employee,
            entry: initial.menu_entry.clone(),
            target,
        };
        execute_current_battle_mutation(&mut core, player, choose(initial.caster)).unwrap();
        let first = relation_world_owner(&core);
        execute_current_battle_mutation(&mut core, player, choose(target)).unwrap();
        let replaced = relation_world_owner(&core);
        assert_eq!(replaced.target, Some(target));
        assert_ne!(replaced.generation, first.generation);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::OpenBattleSettings)
            .unwrap();
        let before = battle_advance_atomic_snapshot(&core);
        assert!(matches!(
            execute_current_battle_mutation(&mut core, player, choose(initial.caster)),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::SettingsOpen,
                ..
            })
        ));
        assert_eq!(before, battle_advance_atomic_snapshot(&core));
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::CloseBattleSettings)
            .unwrap();
        core.advance_active_battle_for_server_tick(1000).unwrap();
        assert_eq!(relation_world_owner(&core).stock, 1);
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: other,
            },
        )
        .unwrap();
        let detached = relation_world_owner(&core);
        assert_eq!(detached.target, None);
        let retained_stock = detached.stock;
        core.advance_active_battle_for_server_tick(1000).unwrap();
        assert_eq!(relation_world_owner(&core).stock, retained_stock);
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
        .unwrap();
        let withdrawn = relation_world_owner(&core);
        assert_eq!(withdrawn.source_copy, initial.source_copy);
        assert_eq!(withdrawn.stock, retained_stock);
        assert_eq!(withdrawn.target, None);
        assert!(!withdrawn.menu_available);
        let before = battle_advance_atomic_snapshot(&core);
        assert!(
            execute_current_battle_mutation(&mut core, player, choose(initial.caster)).is_err()
        );
        assert_eq!(before, battle_advance_atomic_snapshot(&core));
    }
}
