//! External RON is loaded through the production pack parser. Test numbers are
//! synthetic and do not change the official resource catalog or economy.
// Leave the authoritative fixture immediately before the real terminal bucket.
// The lookahead is discarded; fault/retry assertions exercise that same bucket.
fn advance_to_resource_terminal_boundary(core: &mut GameCore) {
    for _ in 0..6000 {
        let mut next = core.clone();
        next.advance_active_battle_to_next_event_bucket().unwrap();
        if next.state.active_battle.is_none() {
            return;
        }
        *core = next;
    }
    panic!("resource fixture did not reach its terminal bucket");
}

use super::*;
use crate::game::data::content_pack::tests::ExternalFixture;
use crate::game::resource::{ResourceLifetime, ResourceOwner};

#[test]
fn cmrp_external_world_active_terminal_and_pending_keep_effective_owner_limits() {
    use crate::game::data::content_pack::{self, InputLimits};
    use crate::runtime_safety::RuntimeSafetySettings;
    let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, true, true);
    for cap in [16 * 1024 * 1024, 32 * 1024 * 1024] {
        let mut raw =
            String::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON.to_vec())
                .unwrap()
                .replacen(
                    "max_retained_bytes: 16777216",
                    "max_retained_bytes: 8388608",
                    1,
                );
        let end = raw.rfind(')').unwrap();
        raw.insert_str(
            end,
            &format!(
                "development_measurement: Some((revision: 1, emergency_retained_bytes: {cap})),\n"
            ),
        );
        let settings = Arc::new(
            RuntimeSafetySettings::from_ron("trusted world profile", raw.as_bytes()).unwrap(),
        );
        let generation = content_pack::load_with_runtime_safety(
            fixture.root(),
            &fixture.selection(),
            &InputLimits::default(),
            settings.clone(),
        )
        .unwrap();
        let (mut core, _, _, _) = start_resource_battle(generation.into_fixture_database_arc());
        assert_eq!(
            core.runtime_safety().effective_sha256(),
            settings.effective_sha256()
        );
        assert_eq!(
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .runtime_fact_limits()
                .max_retained_bytes,
            cap
        );
        let cloned = core.clone();
        assert_eq!(
            cloned.runtime_safety().effective_sha256(),
            settings.effective_sha256()
        );
        advance_to_resource_cost(&mut core, 2);
        let result = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .capture_run_resources()
            .unwrap()
            .unwrap();
        assert_eq!(result.limits.max_retained_bytes, cap);
        assert!(result.retained_extent().unwrap() > 0);
        advance_to_resource_terminal_boundary(&mut core);
        #[cfg(feature = "runtime-observation")]
        let observed = {
            let mut application = crate::application::CoreApplication::from_core(core.clone());
            let (sender, receiver) = crate::runtime_observation::channel();
            let (observer, reader) = sender.session(104);
            application.set_runtime_observer(Some(observer));
            application
                .advance_active_battle_to_next_event_bucket()
                .unwrap();
            (application, receiver, reader)
        };
        core.advance_active_battle_to_next_event_bucket().unwrap();
        assert!(core.state.active_battle.is_none());
        assert_eq!(
            core.pending_resource_limits().unwrap().max_retained_bytes,
            cap
        );
        let pending_extent = core.pending_resource_extent().unwrap().unwrap();
        assert!(pending_extent > 0);
        #[cfg(feature = "runtime-observation")]
        {
            use crate::runtime_observation::NativeOwnerExtent;
            let (mut application, receiver, reader) = observed;
            let terminal = receiver.try_recv().unwrap();
            let expected_owner = NativeOwnerExtent::PendingResources {
                bytes: pending_extent as u64,
            };
            assert_eq!(terminal.committed.unwrap().owner_extent, expected_owner);
            assert!(terminal.committed.unwrap().battle.is_none());
            assert_eq!(
                terminal.committed.unwrap().usage.0[0],
                0,
                "legacy Usage keeps battle-only extent"
            );
            assert!(terminal.max_pending_owner_extent_bytes.unwrap() >= pending_extent as u64);
            assert_eq!(
                reader.snapshot(false).unwrap().current.owner_extent,
                expected_owner
            );
            application
                .execute_admin(
                    crate::application::ApplicationAdminCommand::from_document(
                        json!({"type":"admin_set_enkephalin","amount":1}).into(),
                    )
                    .unwrap(),
                )
                .unwrap();
            let pending_operation = receiver.try_recv().unwrap();
            let allowance = pending_operation.retained.unwrap();
            assert_eq!(allowance.seed_attempted_bytes, Some(pending_extent as u64));
            assert_eq!(allowance.seed_accepted_bytes, Some(pending_extent as u64));
            assert_eq!(allowance.owner_hard_bytes, cap as u64);
            assert_eq!(
                pending_operation.committed.unwrap().owner_extent,
                expected_owner
            );
            assert_eq!(
                pending_operation
                    .started_identity
                    .unwrap()
                    .selected_profile_hard_bytes,
                cap as u64
            );
            drop(application);
            assert!(!reader.is_alive());
        }
        assert!(!crate::game::battle::runtime_control::has_operation_scope());
    }
}

fn external_resource_data(lifetime: ResourceLifetime, insufficient: bool) -> Arc<GameDataBase> {
    external_resource_data_options(lifetime, insufficient, false, false)
}

fn external_resource_data_options(
    lifetime: ResourceLifetime,
    insufficient: bool,
    consume_only: bool,
    timed_result: bool,
) -> Arc<GameDataBase> {
    external_resource_fixture_options(lifetime, insufficient, consume_only, timed_result)
        .load()
        .into_fixture_database_arc()
}

fn external_resource_fixture_options(
    lifetime: ResourceLifetime,
    insufficient: bool,
    consume_only: bool,
    timed_result: bool,
) -> ExternalFixture {
    let fixture = ExternalFixture::official();
    // Preserve this resource/cost oracle's pre-U06 equipment and passive input.
    // The normal six active starter sets have separate live grant/use coverage.
    let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
        ron::de::from_str(&fixture.read("official/data/employees/definitions.ron")).unwrap();
    for employee in &mut employees.definitions {
        employee.initial_loadout.active_skill_fragment_id = None;
        if employee.starter_eligible {
            employee.initial_loadout.equipment_ids =
                vec!["standard_training_sword".into(), "standard_armor".into()];
            if employee.employee_definition_id == "maintenance_tech" {
                employee
                    .initial_loadout
                    .equipment_ids
                    .push("archive_scanner".into());
            }
            employee.initial_loadout.baseline_skill_fragment_ids =
                vec!["starter_basic_attack_enhancement".into()];
        } else {
            employee.initial_loadout.equipment_ids.clear();
            employee.initial_loadout.baseline_skill_fragment_ids.clear();
        }
    }
    fixture.write(
        "official/data/employees/definitions.ron",
        ron::ser::to_string(&employees).unwrap(),
    );
    fixture.write("official/data/resources/base.ron", format!(
        "(resources: [(id: \"employee_meter\", owner: Employee, lifetime: {lifetime:?}, initial: 7, cap: 15),
        (id: \"copy_meter\", owner: EquipmentCopy, lifetime: {lifetime:?}, initial: 7, cap: 15)])"));
    let mut equipment: EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let sword = equipment
        .items
        .iter_mut()
        .find(|item| item.id == "standard_training_sword")
        .unwrap();
    sword.ability_activations.push(ron::de::from_str("(ability_id: \"r05_resource_probe\", activation: TriggerProc(trigger: OnAttack, max_triggers_per_battle: Some(1)))").unwrap());
    // Keep this resource probe independent of the melee spacing contract.
    sword.weapon_profile.as_mut().unwrap().defense_tile_range = ron::de::from_str(
        "(include_anchor_tile: false, rows: [\"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXX@XXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\"])").unwrap();
    if timed_result {
        sword.price = 100; // synthetic sale quote; official training sword remains unpriced
        let mut dismantle = equipment.dismantle_recipes[0].clone();
        dismantle.equipment_id = "standard_training_sword".into();
        equipment.dismantle_recipes.push(dismantle);
        let mut enhance = equipment.enhancement_recipes[0].clone();
        enhance.equipment_id = "standard_training_sword".into();
        equipment.enhancement_recipes.push(enhance);
    }
    fixture.write(
        "official/data/equipments/base.ron",
        ron::ser::to_string(&equipment).unwrap(),
    );
    let mut skills = crate::game::data::skill_data::RawSkillDatabase::from_ron(
        "fixture",
        &fixture.read("official/data/skills/base.ron"),
    )
    .unwrap();
    let consume = if insufficient { 99 } else { 2 };
    let input = format!(
        r#"SkillDef(id: "r05_resource_probe", name: "R05 synthetic resource probe", kind: Untargeted, focus_time_ms: 0,
        use_lifetime: Action,
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false),
        steps: [(id: "resource_batch", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [
            (id: "resource_effect", op: Resources(operations: [
                (resource: "employee_meter", owner: SourceEmployee, operation: Grant(20)),
                (resource: "employee_meter", owner: SourceEmployee, operation: Decay(3)),
                (resource: "employee_meter", owner: SourceEmployee, operation: Consume(2)),
                (resource: "copy_meter", owner: SourceEquipment, operation: Grant(20)),
                (resource: "copy_meter", owner: SourceEquipment, operation: Decay(3)),
                (resource: "copy_meter", owner: SourceEquipment, operation: Consume({consume})),
            ]))])])"#
    );
    let input = if consume_only {
        input
            .lines()
            .filter(|line| !line.contains("operation: Grant") && !line.contains("operation: Decay"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        input
    };
    skills.skills.push(ron::de::from_str(&input).unwrap());
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&skills).unwrap(),
    );
    if timed_result {
        let mut encounters: PveEncounterDatabase =
            ron::de::from_str(&fixture.read("official/data/pve/encounters.ron")).unwrap();
        let encounter = encounters
            .encounters
            .iter_mut()
            .find(|encounter| encounter.id == "suppress_warm_hearted_woodsman")
            .unwrap();
        encounter
            .terminal_outcomes
            .iter_mut()
            .find(|outcome| outcome.id == "victory")
            .unwrap()
            .condition =
            crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 30_000 };
        for wave in &mut encounter.waves {
            wave.required_for_victory = false;
        }
        fixture.write(
            "official/data/pve/encounters.ron",
            ron::ser::to_string(&encounters).unwrap(),
        );
    }
    fixture
}

fn start_resource_battle(data: Arc<GameDataBase>) -> (GameCore, Uuid, Uuid, Position) {
    let mut core = GameCore::new(data, 0xCA05_101);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let (employee, position) = enter_resource_battle(&mut core, player);
    (core, player, employee, position)
}

fn enter_resource_battle(core: &mut GameCore, player: Uuid) -> (Uuid, Position) {
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let node = force_map_combat_node(
        core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_warm_hearted_woodsman",
    );
    let preview = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat),
        ..
    } = &preview
    else {
        panic!("combat preview")
    };
    let end = combat
        .routes
        .iter()
        .find(|route| route.id == "defense_main")
        .unwrap()
        .end;
    let cells: Vec<_> = combat
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .collect();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let position = cells
        .into_iter()
        .filter(|cell| {
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .player_tactical_slot_occupant(*cell, None)
                .is_none()
        })
        .min_by_key(|cell| (cell.manhattan(&end), cell.y, cell.x))
        .unwrap();
    execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    (employee, position)
}

fn resource_spent(core: &GameCore) -> i64 {
    core.state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap()
        .providers
        .iter()
        .filter_map(|provider| provider.summary.latest_cost.as_ref())
        .filter(|cost| {
            matches!(
                cost.resource,
                crate::game::battle::runtime_facts::FactResource::Typed { .. }
            )
        })
        .map(|cost| cost.actual_spent())
        .sum()
}

#[test]
fn external_ron_executes_each_lifetime_with_actual_employee_and_copy_sources() {
    for lifetime in [
        ResourceLifetime::Run,
        ResourceLifetime::Attempt,
        ResourceLifetime::Deployment,
        ResourceLifetime::Activation,
        ResourceLifetime::Delivery,
        ResourceLifetime::Chain,
    ] {
        let (mut core, _player, employee, _) =
            start_resource_battle(external_resource_data(lifetime, false));
        for _ in 0..2000 {
            core.advance_active_battle_to_next_event_bucket()
                .unwrap_or_else(|error| panic!("{lifetime:?}: {error:?}"));
            if resource_spent(&core) >= 2 {
                break;
            }
        }
        assert_eq!(
            resource_spent(&core),
            2,
            "{lifetime:?}: time={} resources={:?}",
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .execution
                .last_event_time_ms(),
            serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap()
        );
        let facts = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .runtime_fact_snapshot()
            .unwrap();
        let witness = facts
            .providers
            .iter()
            .find(|provider| {
                provider.summary.latest_cost.as_ref().is_some_and(|cost| {
                    matches!(
                        cost.resource,
                        crate::game::battle::runtime_facts::FactResource::Typed { .. }
                    )
                })
            })
            .unwrap();
        let cost = witness.summary.latest_cost.as_ref().unwrap();
        let copy = witness.provider.persistent_copy_uuid.unwrap();
        assert_eq!(
            cost.owner,
            crate::game::battle::runtime_facts::ResourceOwner::EquipmentCopy(copy)
        );
        assert_eq!(
            (cost.before, cost.after, cost.requested_delta),
            (12, 10, -2)
        );
        assert!(
            matches!(&cost.resource, crate::game::battle::runtime_facts::FactResource::Typed { lifetime: actual, .. } if *actual == lifetime)
        );
        assert!(cost.source.authored.is_some());
        assert_eq!(
            cost.source.equipment.unwrap().persistent_copy_uuid,
            Some(copy)
        );
        let view = core.battle_typed_resources().unwrap();
        assert_eq!(view.reservation_count(), 0);
        let entries: Vec<_> = view.entries().collect();
        if matches!(
            lifetime,
            ResourceLifetime::Run | ResourceLifetime::Attempt | ResourceLifetime::Deployment
        ) {
            assert_eq!(entries.len(), 2, "{lifetime:?}");
            assert!(entries
                .iter()
                .all(|(_, value)| value.balance == 10 && value.reserved == 0));
            assert!(entries
                .iter()
                .any(|(key, _)| key.owner == ResourceOwner::Employee(employee)));
            assert!(entries
                .iter()
                .any(|(key, _)| key.owner == ResourceOwner::EquipmentCopy(copy)));
        } else {
            assert!(entries.is_empty(), "ended {lifetime:?}: {entries:?}");
            assert!(
                !view.scopes().any(|(_, live, _, _)| live == lifetime),
                "ended {lifetime:?}"
            );
        }
        assert!(
            core.run_resources().unwrap().entries.is_empty(),
            "overlay is not a committed Run result"
        );
    }
}

#[test]
fn external_ron_second_cost_shortage_preserves_actual_timestamp_candidate() {
    let (mut core, _, _, _) =
        start_resource_battle(external_resource_data(ResourceLifetime::Run, true));
    let mut rejected = false;
    for _ in 0..2000 {
        let before = battle_advance_atomic_snapshot(&core);
        let resources = serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap();
        match core.advance_active_battle_to_next_event_bucket() {
            Ok(_) => {}
            Err(error) => {
                assert_eq!(error.code(), "resource_insufficient");
                assert_eq!(battle_advance_atomic_snapshot(&core), before);
                assert_eq!(
                    serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap(),
                    resources
                );
                assert_eq!(resource_spent(&core), 0);
                rejected = true;
                break;
            }
        }
    }
    assert!(
        rejected,
        "authored shortage must reach the production consumer"
    );
}

fn advance_to_resource_cost(core: &mut GameCore, spent: i64) {
    for _ in 0..3000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        if resource_spent(core) >= spent {
            return;
        }
    }
    panic!("actual typed cost did not occur");
}

#[test]
fn actual_withdraw_redeploy_resets_deployment_but_preserves_attempt_and_run() {
    for lifetime in [
        ResourceLifetime::Run,
        ResourceLifetime::Attempt,
        ResourceLifetime::Deployment,
    ] {
        let (mut core, player, employee, position) =
            start_resource_battle(external_resource_data_options(lifetime, false, true, false));
        let view = core.battle_typed_resources().unwrap();
        let scope = view
            .scopes()
            .find(|(_, kind, _, _)| *kind == lifetime)
            .unwrap()
            .0;
        assert_eq!(
            view.value(
                &crate::game::resource::ResourceId::new("employee_meter".into()).unwrap(),
                ResourceOwner::Employee(employee),
                scope
            )
            .unwrap()
            .balance,
            7
        );
        assert!(
            view.entries().next().is_none(),
            "query must not materialize initial"
        );
        advance_to_resource_cost(&mut core, 2);
        assert!(core
            .battle_typed_resources()
            .unwrap()
            .entries()
            .all(|(_, value)| value.balance == 5));
        // The scenario also has NPC deployments. Use the actual employee's
        // consumed scope, rather than the first diagnostic Deployment ordinal.
        let scope = core
            .battle_typed_resources()
            .unwrap()
            .entries()
            .find(|(key, _)| key.owner == ResourceOwner::Employee(employee))
            .unwrap()
            .0
            .scope;
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
        .unwrap();
        let after = core.battle_typed_resources().unwrap();
        if lifetime == ResourceLifetime::Deployment {
            assert!(after.entries().next().is_none());
            assert_eq!(
                after.value(
                    &crate::game::resource::ResourceId::new("employee_meter".into()).unwrap(),
                    ResourceOwner::Employee(employee),
                    scope
                ),
                None
            );
        } else {
            assert_eq!(after.entries().count(), 2);
            assert!(after.entries().all(|(_, value)| value.balance == 5));
        }
        let mut deployed = false;
        for _ in 0..3000 {
            let before = serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap();
            match execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::DeployUnit {
                    employee_uuid: employee,
                    position,
                    facing: FacingDirection::Right,
                },
            ) {
                Ok(_) => {
                    deployed = true;
                    break;
                }
                Err(GameError::ActionUnavailable {
                    reason: ActionDisabledReasonCode::RedeployCooldown,
                    ..
                }) => {
                    assert_eq!(
                        serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap(),
                        before
                    );
                    core.advance_active_battle_to_next_event_bucket().unwrap();
                }
                Err(error) => panic!("redeploy {lifetime:?}: {error:?}"),
            }
        }
        assert!(deployed);
        advance_to_resource_cost(&mut core, 4);
        let expected = if lifetime == ResourceLifetime::Deployment {
            5
        } else {
            3
        };
        assert_eq!(core.battle_typed_resources().unwrap().entries().count(), 2);
        assert!(core
            .battle_typed_resources()
            .unwrap()
            .entries()
            .all(|(_, value)| value.balance == expected));
    }
}

#[test]
fn actual_retreat_restores_run_resource_entry_timeline_and_attempt_identity() {
    let (mut core, player, _, _) = start_resource_battle(external_resource_data_options(
        ResourceLifetime::Run,
        false,
        true,
        false,
    ));
    let before = core.run_resources().unwrap().clone();
    advance_to_resource_cost(&mut core, 2);
    assert_eq!(core.battle_typed_resources().unwrap().entries().count(), 2);
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(core.run_resources(), Some(&before));
    assert!(core.battle_typed_resources().is_none());
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .battle_typed_resources()
        .unwrap()
        .entries()
        .next()
        .is_none());
    assert_eq!(core.run_resources(), Some(&before));
}

#[test]
fn actual_run_result_failure_retry_and_fresh_public_checkpoint_preserve_nonempty_values() {
    let data = external_resource_data_options(ResourceLifetime::Run, false, true, true);
    let (mut core, player, _, _) = start_resource_battle(data.clone());
    advance_to_resource_cost(&mut core, 2);
    advance_to_resource_terminal_boundary(&mut core);
    assert!(core.run_resources().unwrap().entries.is_empty());
    let outcome = core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .outcome_policies
        .iter_mut()
        .find(|policy| policy.verdict == BattleVerdict::Victory)
        .unwrap();
    let original_rewards = outcome.rewards.clone();
    outcome.rewards[0]
        .effects
        .push(RewardEffect::GrantEquipment {
            equipment_id: "r05_missing_reward".into(),
        });
    let before = serialized_grant_roots(&core);
    let root = core.run_resources().unwrap().clone();
    for _ in 0..2 {
        let error = core
            .advance_active_battle_to_next_event_bucket()
            .unwrap_err();
        assert!(
            matches!(error, GameError::InvalidStaticData(ref message) if message.contains("r05_missing_reward")),
            "{error:?}"
        );
        assert_eq!(serialized_grant_roots(&core), before);
        assert_eq!(core.run_resources(), Some(&root));
        assert!(core.state.active_battle.is_some());
        assert!(core.state.confirmed_combat_result.is_none());
    }
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .outcome_policies
        .iter_mut()
        .find(|policy| policy.verdict == BattleVerdict::Victory)
        .unwrap()
        .rewards = original_rewards;
    core.advance_active_battle_to_next_event_bucket().unwrap();
    assert!(core.state.active_battle.is_none());
    let confirmed = core.run_resources().unwrap().clone();
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(core.run_resources(), Some(&confirmed));
    let committed = core.run_resources().unwrap().clone();
    assert_eq!(committed.entries.len(), 2);
    assert!(committed.entries.iter().all(|entry| entry.balance == 5));
    assert_eq!(committed.revision, root.revision + 1);
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(core.run_resources(), Some(&committed));

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
    let document: Value = serde_json::from_slice(checkpoint.core_payload_json()).unwrap();
    assert_eq!(document["version"], 12);
    assert_eq!(
        document["continuation"]["current"]["typed_resources"],
        serde_json::to_value(&committed).unwrap()
    );
    let mut profile: crate::game::encounter_compendium::PlayerProfileState = serde_json::from_value(
        source.snapshot(crate::application::ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().as_value().clone()
    ).unwrap();
    profile.highest_floor_reached = 99;
    profile.longest_deathless_floor_streak = 17;
    let seed_profile = |app: &mut crate::application::CoreApplication| {
        app.install_authoritative_player_profile(crate::application::ApplicationDocument::new(
            serde_json::to_value(&profile).unwrap(),
        ))
        .unwrap();
    };
    let profile_snapshot = |app: &crate::application::CoreApplication| {
        serde_json::to_value(
            app.snapshot(crate::application::ApplicationSnapshotKind::PlayerProfile)
                .unwrap(),
        )
        .unwrap()
    };
    let mut destination = crate::application::CoreApplication::new(data.clone(), 0xCA05_999);
    seed_profile(&mut destination);
    let preserved_profile = profile_snapshot(&destination);
    let pristine_destination = serde_json::to_value(
        destination
            .snapshot(crate::application::ApplicationSnapshotKind::Run)
            .unwrap(),
    )
    .unwrap();
    for fault in [
        "unknown_owner",
        "unknown_resource",
        "cap",
        "duplicate",
        "old_contract",
        "missing_root",
        "bad_checksum",
    ] {
        let mut bad = document.clone();
        let entries = &mut bad["continuation"]["current"]["typed_resources"]["entries"];
        match fault {
            "unknown_owner" => {
                entries[0]["owner"] = json!({"EquipmentCopy": Uuid::from_u128(0xBAD)})
            }
            "unknown_resource" => entries[0]["resource"] = json!("missing_resource"),
            "cap" => entries[0]["balance"] = json!(16),
            "duplicate" => {
                let entry = entries[0].clone();
                entries.as_array_mut().unwrap().insert(0, entry);
            }
            "old_contract" => {
                bad["continuation"]["current"]["content_identity"]["contract"]["execution"] =
                    json!(1)
            }
            "missing_root" => {
                bad["continuation"]["current"]
                    .as_object_mut()
                    .unwrap()
                    .remove("typed_resources");
            }
            "bad_checksum" => {}
            _ => unreachable!(),
        }
        // Missing required fields are rejected by the strict codec before its
        // typed checksum helper can construct a payload.
        if fault != "missing_root" {
            refresh_active_run_checksum(&mut bad);
        }
        if fault == "bad_checksum" {
            bad["checkpoint_sha256"] = json!("0".repeat(64));
        }
        let invalid = crate::application::ApplicationCheckpoint::new(
            serde_json::to_vec(&bad).unwrap(),
            checkpoint.loads_used(),
            checkpoint.run_peak_certified_power(),
        );
        assert!(
            destination
                .continue_from_active_run_checkpoint(player, &invalid)
                .is_err(),
            "{fault}"
        );
        assert!(
            destination.run_resources().is_none(),
            "{fault} must not publish a Run"
        );
        assert_eq!(source.run_resources(), Some(&committed));
        assert_eq!(profile_snapshot(&destination), preserved_profile);
        assert_eq!(
            serde_json::to_value(
                destination
                    .snapshot(crate::application::ApplicationSnapshotKind::Run)
                    .unwrap()
            )
            .unwrap(),
            pristine_destination
        );
    }
    destination
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    assert_eq!(destination.run_resources(), Some(&committed));
    assert_eq!(profile_snapshot(&destination), preserved_profile);
    assert!(destination.battle_typed_resources().is_none());
    let existing = serde_json::to_value(
        destination
            .snapshot(crate::application::ApplicationSnapshotKind::Run)
            .unwrap(),
    )
    .unwrap();
    assert!(destination
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .is_err());
    assert_eq!(destination.run_resources(), Some(&committed));
    assert_eq!(profile_snapshot(&destination), preserved_profile);
    assert_eq!(
        serde_json::to_value(
            destination
                .snapshot(crate::application::ApplicationSnapshotKind::Run)
                .unwrap()
        )
        .unwrap(),
        existing
    );

    let high_water = checkpoint.run_peak_certified_power() + 100;
    let counted = crate::application::ApplicationCheckpoint::new(
        checkpoint.core_payload_json().to_vec(),
        2,
        high_water,
    );
    let mut counted_destination = crate::application::CoreApplication::new(data, 0xCA05_998);
    seed_profile(&mut counted_destination);
    counted_destination
        .continue_from_active_run_checkpoint(player, &counted)
        .unwrap();
    let reexport = counted_destination
        .export_active_run_checkpoint()
        .unwrap()
        .unwrap();
    assert_eq!(
        reexport.loads_used(),
        2,
        "continue does not consume a checkpoint load"
    );
    assert!(reexport.run_peak_certified_power() >= high_water);
    let load_command = || {
        crate::application::ApplicationCommand::from_document(
            crate::application::ApplicationDocument::new(json!({"type":"load_run_checkpoint"})),
        )
        .unwrap()
    };
    counted_destination
        .execute(player, load_command(), Default::default())
        .unwrap();
    assert_eq!(counted_destination.run_resources(), Some(&committed));
    assert_eq!(profile_snapshot(&counted_destination), preserved_profile);
    let loaded = counted_destination
        .export_active_run_checkpoint()
        .unwrap()
        .unwrap();
    assert_eq!(loaded.loads_used(), 3);
    assert!(loaded.run_peak_certified_power() >= high_water);
    assert!(counted_destination
        .execute(player, load_command(), Default::default())
        .is_err());
    assert_eq!(counted_destination.run_resources(), Some(&committed));
    assert_eq!(profile_snapshot(&counted_destination), preserved_profile);
    assert_eq!(
        counted_destination
            .export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .loads_used(),
        3
    );

    let changed = external_resource_fixture_options(ResourceLifetime::Run, false, true, true);
    let resources = changed
        .read("official/data/resources/base.ron")
        .replace("cap: 15", "cap: 14");
    changed.write("official/data/resources/base.ron", resources);
    let mut mismatch = crate::application::CoreApplication::new(
        changed.load().into_fixture_database_arc(),
        0xCA05_997,
    );
    seed_profile(&mut mismatch);
    assert!(
        mismatch
            .continue_from_active_run_checkpoint(player, &checkpoint)
            .is_err(),
        "same resource IDs with changed caps must not accept an older content fingerprint"
    );
    assert!(mismatch.run_resources().is_none());
    assert_eq!(profile_snapshot(&mismatch), preserved_profile);
    assert_eq!(source.run_resources(), Some(&committed));
}

#[test]
fn actual_same_copy_transfer_enhance_sale_and_dismantle_preserve_or_discard_only_that_run_key() {
    let (mut base, player, employee, _) = start_resource_battle(external_resource_data_options(
        ResourceLifetime::Run,
        false,
        true,
        true,
    ));
    advance_to_resource_cost(&mut base, 2);
    for _ in 0..6000 {
        base.advance_active_battle_to_next_event_bucket().unwrap();
        if base.state.active_battle.is_none() {
            break;
        }
    }
    base.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let original = base.run_resources().unwrap().clone();
    let copy = original
        .entries
        .iter()
        .find_map(|entry| match entry.owner {
            ResourceOwner::EquipmentCopy(id) => Some(id),
            _ => None,
        })
        .unwrap();
    let other = base
        .state
        .roster
        .iter()
        .find(|other| other.uuid != employee)
        .unwrap()
        .uuid;
    base.execute(
        player,
        PlayerBehavior::UnEquipItem {
            item_uuid: copy,
            target_unit: employee,
        },
    )
    .unwrap();
    assert_eq!(base.run_resources(), Some(&original));
    base.execute(
        player,
        PlayerBehavior::EquipItem {
            item_uuid: copy,
            target_unit: other,
        },
    )
    .unwrap();
    assert_eq!(base.run_resources(), Some(&original));
    assert_eq!(
        base.state
            .inventory
            .equipments
            .get_item(&copy)
            .unwrap()
            .equipped_to,
        Some(other)
    );

    let mut sale = base.clone();
    sale.execute(
        player,
        PlayerBehavior::UnEquipItem {
            item_uuid: copy,
            target_unit: other,
        },
    )
    .unwrap();
    let node = force_first_available_node(
        &mut sale,
        MapNodeCategory::Shop,
        "shop_general",
        MapNodePayload::Shop {
            shop_id: Some("unknown_exploration_shop".into()),
            shop_pool_id: None,
        },
    );
    sale.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    sale.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let before = serialized_grant_roots(&sale);
    assert!(sale
        .execute(
            player,
            PlayerBehavior::SellItem {
                item_uuid: copy,
                quantity: 2
            }
        )
        .is_err());
    assert_eq!(serialized_grant_roots(&sale), before);
    assert_eq!(sale.run_resources(), Some(&original));
    let currency = sale.state.enkephalin.amount;
    let quote = sale
        .state
        .inventory
        .equipments
        .get_item(&copy)
        .unwrap()
        .meta
        .price
        * 35
        / 100;
    sale.execute(
        player,
        PlayerBehavior::SellItem {
            item_uuid: copy,
            quantity: 1,
        },
    )
    .unwrap();
    assert_eq!(sale.state.enkephalin.amount, currency + quote);
    assert!(sale.state.inventory.equipments.get_item(&copy).is_none());
    assert_eq!(sale.run_resources().unwrap().entries.len(), 1);
    assert_eq!(
        sale.run_resources().unwrap().entries[0].owner,
        ResourceOwner::Employee(employee)
    );
    assert_eq!(sale.run_resources().unwrap().entries[0].balance, 5);
    let after_sale = sale.run_resources().unwrap().clone();
    assert!(sale
        .execute(
            player,
            PlayerBehavior::SellItem {
                item_uuid: copy,
                quantity: 1
            }
        )
        .is_err());
    assert_eq!(sale.run_resources(), Some(&after_sale));

    let mut maintenance = base;
    let node = force_first_available_node(
        &mut maintenance,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    maintenance
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    maintenance
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let before = serialized_grant_roots(&maintenance);
    assert!(maintenance
        .execute(
            player,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: Uuid::nil()
            }
        )
        .is_err());
    assert_eq!(serialized_grant_roots(&maintenance), before);
    let armors: Vec<_> = maintenance
        .state
        .inventory
        .equipments
        .iter()
        .filter(|item| item.meta.id == "standard_armor")
        .take(2)
        .map(|item| item.instance_uuid)
        .collect();
    assert_eq!(armors.len(), 2);
    for item_uuid in armors {
        maintenance
            .execute(player, PlayerBehavior::DismantleEquipment { item_uuid })
            .unwrap();
        assert_eq!(maintenance.run_resources(), Some(&original));
    }
    maintenance
        .execute(player, PlayerBehavior::EnhanceEquipment { item_uuid: copy })
        .unwrap();
    assert_eq!(maintenance.run_resources(), Some(&original));
    assert_eq!(
        maintenance
            .state
            .inventory
            .equipments
            .get_item(&copy)
            .unwrap()
            .enhancement_level,
        1
    );
    let yields = maintenance
        .game_data
        .equipment_data
        .get_dismantle_recipe_by_equipment_id(
            &maintenance
                .state
                .inventory
                .equipments
                .get_item(&copy)
                .unwrap()
                .meta
                .id,
        )
        .unwrap()
        .yields
        .clone();
    assert!(!yields.is_empty());
    let mut rejected_dismantle = maintenance.clone();
    let material = &yields[0];
    let current = rejected_dismantle
        .state
        .inventory
        .equipment_materials
        .amount(&material.material_id);
    rejected_dismantle
        .state
        .inventory
        .equipment_materials
        .add(&material.material_id, u32::MAX - current)
        .unwrap();
    let rejected_before = serialized_grant_roots(&rejected_dismantle);
    assert!(rejected_dismantle
        .execute(
            player,
            PlayerBehavior::DismantleEquipment { item_uuid: copy }
        )
        .is_err());
    assert_eq!(serialized_grant_roots(&rejected_dismantle), rejected_before);
    assert_eq!(rejected_dismantle.run_resources(), Some(&original));
    let expected_yields: Vec<_> = yields
        .iter()
        .map(|material| {
            (
                material.material_id.clone(),
                maintenance
                    .state
                    .inventory
                    .equipment_materials
                    .amount(&material.material_id)
                    + material.amount,
            )
        })
        .collect();
    maintenance
        .execute(
            player,
            PlayerBehavior::DismantleEquipment { item_uuid: copy },
        )
        .unwrap();
    for (id, expected) in expected_yields {
        assert_eq!(
            maintenance.state.inventory.equipment_materials.amount(&id),
            expected
        );
    }
    assert!(maintenance
        .state
        .inventory
        .equipments
        .get_item(&copy)
        .is_none());
    assert_eq!(maintenance.run_resources().unwrap().entries.len(), 1);
    assert_eq!(
        maintenance.run_resources().unwrap().entries[0].owner,
        ResourceOwner::Employee(employee)
    );
    assert_eq!(maintenance.run_resources().unwrap().entries[0].balance, 5);
    assert_eq!(
        maintenance.run_resources().unwrap().revision,
        original.revision + 1
    );
}

#[test]
fn external_resource_reference_and_selector_negatives_keep_exact_authored_source() {
    use crate::game::data::content_pack::{load, CatalogKind, InputLimits};
    for (needle, replacement, expected_field) in [
        (
            "resource:\"employee_meter\"",
            "resource:\"missing_resource\"",
            ".resource",
        ),
        ("owner:SourceEmployee", "owner:SourceEquipment", ".owner"),
    ] {
        let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, true, false);
        let before = fixture.read("official/data/skills/base.ron");
        assert!(before.contains(needle));
        fixture.write(
            "official/data/skills/base.ron",
            before.replace(needle, replacement),
        );
        let changed = fixture.read("official/data/skills/base.ron");
        let error = load(
            fixture.root(),
            &fixture.selection(),
            &InputLimits::default(),
        )
        .err()
        .expect("invalid resource reference");
        assert_eq!(error.source_file, "official/data/skills/base.ron");
        assert_eq!(error.catalog_kind, Some(CatalogKind::Skills));
        assert_eq!(error.object_id.as_deref(), Some("r05_resource_probe"));
        assert!(error.field_path.as_ref().unwrap().ends_with(expected_field));
        assert_eq!(fixture.read("official/data/skills/base.ron"), changed);
    }
}

#[test]
fn two_real_same_definition_copies_and_target_employee_selector_do_not_alias() {
    let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, true, false);
    let skills = fixture.read("official/data/skills/base.ron");
    assert!(skills.contains("owner:SourceEmployee"));
    fixture.write(
        "official/data/skills/base.ron",
        skills.replace("owner:SourceEmployee", "owner:TargetEmployee"),
    );
    let (mut core, player, first, first_position) =
        start_resource_battle(fixture.load().into_fixture_database_arc());
    advance_to_resource_cost(&mut core, 2);
    let second = core
        .state
        .roster
        .iter()
        .find(|employee| employee.uuid != first)
        .unwrap()
        .uuid;
    let active = core.state.active_battle.as_ref().unwrap();
    let position = active
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .filter(|position| {
            active
                .battle
                .player_tactical_slot_occupant(*position, None)
                .is_none()
        })
        .min_by_key(|position| (position.manhattan(&first_position), position.y, position.x))
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: second,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 4);
    let entries: Vec<_> = core.battle_typed_resources().unwrap().entries().collect();
    assert_eq!(entries.len(), 4);
    assert!(entries.iter().all(|(_, value)| value.balance == 5));
    for employee in [first, second] {
        assert_eq!(
            entries
                .iter()
                .filter(|(key, _)| key.owner == ResourceOwner::Employee(employee))
                .count(),
            1
        );
    }
    let facts = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap();
    let providers: Vec<_> = facts
        .providers
        .iter()
        .filter(|provider| {
            provider.summary.latest_cost.as_ref().is_some_and(|cost| {
                matches!(
                    cost.resource,
                    crate::game::battle::runtime_facts::FactResource::Typed { .. }
                )
            })
        })
        .collect();
    assert_eq!(providers.len(), 2);
    assert_eq!(
        providers[0].provider.definition_uuid,
        providers[1].provider.definition_uuid
    );
    assert_ne!(
        providers[0].provider.persistent_copy_uuid,
        providers[1].provider.persistent_copy_uuid
    );
    for provider in providers {
        let copy = provider.provider.persistent_copy_uuid.unwrap();
        assert_eq!(
            entries
                .iter()
                .filter(|(key, _)| key.owner == ResourceOwner::EquipmentCopy(copy))
                .count(),
            1
        );
    }
}

fn external_resource_area_data(lifetime: ResourceLifetime) -> Arc<GameDataBase> {
    let delivery: DeliveryDef = ron::de::from_str(r#"TileArea(
        footprint: (include_anchor_tile: true, rows: ["XXXXXXX", "XXXXXXX", "XXXXXXX", "XXX@XXX", "XXXXXXX", "XXXXXXX", "XXXXXXX"]),
        area: (anchor: Caster, tile_origin: Caster, tracking: GroundFixed, hit_targets: Allies,
            include_caster: true, tick_policy: EveryTick, duration_ms: 2000, tick_interval_ms: Some(500)))"#).unwrap();
    external_resource_carrier_data(lifetime, delivery, false)
}

fn external_resource_carrier_data(
    lifetime: ResourceLifetime,
    delivery: DeliveryDef,
    enemy: bool,
) -> Arc<GameDataBase> {
    let fixture = external_resource_fixture_options(lifetime, false, true, false);
    let mut skills = crate::game::data::skill_data::RawSkillDatabase::from_ron(
        "fixture",
        &fixture.read("official/data/skills/base.ron"),
    )
    .unwrap();
    let mut skill = serde_json::to_value(skills.skills.last().unwrap()).unwrap();
    let mut pending = skill["steps"][0].clone();
    pending["id"] = json!("must_not_release");
    pending["delay_ms"] = json!(3000);
    for operation in pending["effects"][0]["op"]["Resources"]["operations"]
        .as_array_mut()
        .unwrap()
    {
        operation["operation"]["Consume"] = json!(99);
    }
    skill["steps"][0]["delivery"] = serde_json::to_value(delivery).unwrap();
    for operation in skill["steps"][0]["effects"][0]["op"]["Resources"]["operations"]
        .as_array_mut()
        .unwrap()
    {
        operation["operation"]["Consume"] = json!(1);
    }
    if enemy {
        let target = crate::game::ability::SkillTarget::EnemySingle {
            rule: crate::game::ability::UnitTargetRule::Nearest,
        };
        let range = ron::de::from_str("(include_anchor_tile: false, rows: [\"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXX@XXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\"])").unwrap();
        skill["kind"] = json!("Targeted");
        skill["cast_targeting"] =
            serde_json::to_value(crate::game::ability::SkillCastTargetingDef::explicit(
                target.clone(),
                crate::game::ability::SkillTargetReach::Pattern,
                Some(range),
                false,
            ))
            .unwrap();
        skill["steps"][0]["target"] = serde_json::to_value(target).unwrap();
        let damage: crate::game::ability::SkillEffectDef =
            ron::de::from_str("Damage(amount: 1, damage_type: True)").unwrap();
        skill["steps"][0]["effects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"synthetic_hit", "op": damage}));
    }
    skill["steps"].as_array_mut().unwrap().push(pending);
    *skills.skills.last_mut().unwrap() = serde_json::from_value(skill).unwrap();
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&skills).unwrap(),
    );
    fixture.load().into_fixture_database_arc()
}

#[test]
fn released_fixed_area_keeps_delivery_and_chain_resources_after_controller_withdrawal() {
    for lifetime in [ResourceLifetime::Delivery, ResourceLifetime::Chain] {
        let (mut core, player, employee, _) =
            start_resource_battle(external_resource_area_data(lifetime));
        advance_to_resource_cost(&mut core, 1);
        let active = core.state.active_battle.as_ref().unwrap();
        let first_time = active.execution.last_event_time_ms();
        let facts = active.battle.runtime_fact_snapshot().unwrap();
        assert!(facts.active_actions.iter().any(|action| action
            .skill_id
            .as_ref()
            .is_some_and(|id| id.as_str() == "r05_resource_probe")
            && action.terminal.is_none()));
        let spent = facts.summary.resource_spent;
        let view = core.battle_typed_resources().unwrap();
        let scope = view.entries().next().unwrap().0.scope;
        assert_eq!(
            view.entries().count(),
            2,
            "source operations run once despite multiple allied targets"
        );
        assert!(view.entries().all(|(_, value)| value.balance == 6));
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        )
        .unwrap();
        assert!(core
            .battle_typed_resources()
            .unwrap()
            .scopes()
            .any(|(id, _, _, _)| id == scope));
        for _ in 0..1500 {
            core.advance_active_battle_to_next_event_bucket().unwrap();
            if core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .execution
                .last_event_time_ms()
                > first_time + 3500
            {
                break;
            }
        }
        let facts = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .runtime_fact_snapshot()
            .unwrap();
        assert!(
            facts.summary.resource_spent > spent,
            "released payload must consume after cancellation"
        );
        assert!(!core
            .battle_typed_resources()
            .unwrap()
            .scopes()
            .any(|(id, _, _, _)| id == scope));
        assert!(core
            .battle_typed_resources()
            .unwrap()
            .entries()
            .next()
            .is_none());
        assert_eq!(
            core.battle_typed_resources().unwrap().reservation_count(),
            0
        );
    }
}

#[test]
fn released_payload_cannot_reuse_closed_activation_and_failure_keeps_committed_withdrawal() {
    let (mut core, player, employee, _) =
        start_resource_battle(external_resource_area_data(ResourceLifetime::Activation));
    advance_to_resource_cost(&mut core, 1);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    assert!(core
        .battle_typed_resources()
        .unwrap()
        .entries()
        .next()
        .is_none());
    let mut rejected = false;
    for _ in 0..1000 {
        let before = battle_advance_atomic_snapshot(&core);
        let resources = serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap();
        match core.advance_active_battle_to_next_event_bucket() {
            Ok(_) => {}
            Err(error) => {
                assert_eq!(error.code(), "resource_stale_scope");
                assert_eq!(battle_advance_atomic_snapshot(&core), before);
                assert_eq!(
                    serde_json::to_value(core.battle_typed_resources().unwrap()).unwrap(),
                    resources
                );
                rejected = true;
                break;
            }
        }
    }
    assert!(rejected);
}

#[test]
fn released_projectile_completion_retains_delivery_after_motion_and_controller_end() {
    let delivery =
        ron::de::from_str("Projectile(speed_units_per_ms: 1000, hit_policy: TargetLocked)")
            .unwrap();
    let (mut core, player, employee, _) = start_resource_battle(external_resource_carrier_data(
        ResourceLifetime::Delivery,
        delivery,
        true,
    ));
    let mut launched = false;
    for _ in 0..3000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        if core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter().any(|entry|
            matches!(&entry.event, BattleLogEvent::SkillProjectileLaunched { skill_id, .. } if skill_id.as_str() == "r05_resource_probe")) {
            launched = true; break;
        }
    }
    assert!(launched);
    assert_eq!(resource_spent(&core), 0, "launch must precede consumption");
    let withdrawn_at = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 1);
    let facts = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap();
    let cost = facts
        .providers
        .iter()
        .filter_map(|provider| provider.summary.latest_cost.as_ref())
        .find(|cost| {
            matches!(
                cost.resource,
                crate::game::battle::runtime_facts::FactResource::Typed { .. }
            )
        })
        .unwrap();
    assert!(cost.time_ms > withdrawn_at);
    assert_eq!((cost.before, cost.after), (7, 6));
    let crate::game::battle::runtime_facts::FactResource::Typed {
        scope, lifetime, ..
    } = cost.resource
    else {
        unreachable!()
    };
    assert_eq!(lifetime, ResourceLifetime::Delivery);
    assert!(!core
        .battle_typed_resources()
        .unwrap()
        .scopes()
        .any(|(id, _, _, _)| id == scope));
    assert!(core
        .battle_typed_resources()
        .unwrap()
        .entries()
        .next()
        .is_none());
    for _ in 0..1500 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        if core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms()
            > withdrawn_at + 6000
        {
            break;
        }
    }
    assert_eq!(
        resource_spent(&core),
        1,
        "unreleased control step must not execute"
    );
}

#[test]
fn extra_attack_child_cast_preserves_its_actual_causal_chain() {
    let fixture = external_resource_fixture_options(ResourceLifetime::Chain, false, true, false);
    let mut equipment: EquipmentDatabase =
        ron::de::from_str(&fixture.read("official/data/equipments/base.ron")).unwrap();
    let sword = equipment
        .items
        .iter_mut()
        .find(|item| item.id == "standard_training_sword")
        .unwrap();
    let binding = sword.ability_activations.last_mut().unwrap();
    let crate::game::ability::AbilityActivationDef::TriggerProc {
        max_triggers_per_battle,
        ..
    } = &mut binding.activation;
    *max_triggers_per_battle = Some(2);
    fixture.write(
        "official/data/equipments/base.ron",
        ron::ser::to_string(&equipment).unwrap(),
    );
    let mut skills = crate::game::data::skill_data::RawSkillDatabase::from_ron(
        "fixture",
        &fixture.read("official/data/skills/base.ron"),
    )
    .unwrap();
    let mut skill = serde_json::to_value(skills.skills.last().unwrap()).unwrap();
    let target = crate::game::ability::SkillTarget::EnemySingle {
        rule: crate::game::ability::UnitTargetRule::Nearest,
    };
    let range = ron::de::from_str("(include_anchor_tile: false, rows: [\"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXX@XXX\", \"XXXXXXX\", \"XXXXXXX\", \"XXXXXXX\"])").unwrap();
    skill["kind"] = json!("Targeted");
    skill["cast_targeting"] =
        serde_json::to_value(crate::game::ability::SkillCastTargetingDef::explicit(
            target.clone(),
            crate::game::ability::SkillTargetReach::Pattern,
            Some(range),
            false,
        ))
        .unwrap();
    skill["steps"][0]["target"] = serde_json::to_value(target).unwrap();
    skill["steps"][0]["effects"].as_array_mut().unwrap().push(json!({
        "id": "causal_extra_attack", "op": crate::game::ability::SkillEffectDef::ExtraAttack { count: 1 }
    }));
    *skills.skills.last_mut().unwrap() = serde_json::from_value(skill).unwrap();
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&skills).unwrap(),
    );
    let (mut core, _, _, _) = start_resource_battle(fixture.load().into_fixture_database_arc());
    let mut child_spent = false;
    for _ in 0..3000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        let facts = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .runtime_fact_snapshot()
            .unwrap();
        if facts
            .providers
            .iter()
            .filter_map(|provider| provider.summary.latest_cost.as_ref())
            .any(|cost| {
                matches!(
                    cost.resource,
                    crate::game::battle::runtime_facts::FactResource::Typed {
                        lifetime: ResourceLifetime::Chain,
                        ..
                    }
                ) && cost.before == 5
                    && cost.after == 3
            })
        {
            child_spent = true;
            break;
        }
    }
    let facts = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap();
    let causal_events: Vec<_> = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .event_log
        .entries
        .iter()
        .map(|entry| format!("{:?}", entry))
        .filter(|entry| entry.contains("r05_resource_probe") || entry.contains("Some(Triggered)"))
        .collect();
    assert!(child_spent, "child cast must consume the parent's remaining 5, without reopening at initial 7; costs {:?}; causal events {:?}",
        facts.providers.iter().filter_map(|p| p.summary.latest_cost.as_ref()).collect::<Vec<_>>(), causal_events);
    for _ in 0..20 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
    }
    assert!(core
        .battle_typed_resources()
        .unwrap()
        .entries()
        .next()
        .is_none());
    assert_eq!(
        core.battle_typed_resources().unwrap().reservation_count(),
        0
    );
}

fn finish_actual_resource_battle(core: &mut GameCore, player: Uuid) {
    for _ in 0..6000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        if core.state.active_battle.is_none() {
            break;
        }
    }
    assert!(core.state.active_battle.is_none());
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
}

#[test]
fn nonempty_run_survives_repeat_retreat_second_result_floor_and_ends_at_new_run() {
    let (mut core, player, _, _) = start_resource_battle(external_resource_data_options(
        ResourceLifetime::Run,
        false,
        true,
        true,
    ));
    advance_to_resource_cost(&mut core, 2);
    finish_actual_resource_battle(&mut core, player);
    let first = core.run_resources().unwrap().clone();
    assert_eq!(first.entries.len(), 2);
    assert!(first.entries.iter().all(|entry| entry.balance == 5));

    let (employee, position) = enter_resource_battle(&mut core, player);
    advance_to_resource_cost(&mut core, 2);
    assert!(core
        .battle_typed_resources()
        .unwrap()
        .entries()
        .all(|(_, value)| value.balance == 3));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(core.run_resources(), Some(&first));
    let pre_entry = core.get_run_snapshot_json().unwrap();
    let first_entry = core
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: first_uuid,
        ..
    } = first_entry
    else {
        panic!("battle entry")
    };
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 2);
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(
        core.get_run_snapshot_json().unwrap(),
        projection_after_test_retreat(pre_entry, 2)
    );
    assert_eq!(core.run_resources(), Some(&first));
    let repeated_entry = core
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: repeated_uuid,
        ..
    } = repeated_entry
    else {
        panic!("battle entry")
    };
    assert_eq!(
        repeated_uuid, first_uuid,
        "retreat restores UUID/RNG continuation even with nonempty resources"
    );
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 2);
    finish_actual_resource_battle(&mut core, player);
    let second = core.run_resources().unwrap().clone();
    assert_eq!(second.entries.len(), 2);
    assert!(second.entries.iter().all(|entry| entry.balance == 3));
    assert_eq!(second.revision, first.revision + 1);

    let gate = force_first_available_node(
        &mut core,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: gate })
        .unwrap();
    assert!(matches!(
        core.execute(player, PlayerBehavior::ConfirmEnterNode)
            .unwrap(),
        BehaviorResult::FloorAdvanced { .. }
    ));
    assert_eq!(core.run_resources(), Some(&second));
    core.execute(player, PlayerBehavior::AbandonRun).unwrap();
    if let Some(closed) = core.run_resources() {
        assert!(closed.closed);
    }
    start_new_game_with_default_starters(&mut core, player);
    let fresh = core.run_resources().unwrap();
    assert!(fresh.entries.is_empty());
    assert!(!fresh.closed);
    assert!(core.battle_typed_resources().is_none());
}

#[test]
fn actual_event_defeat_commits_nonempty_resources_once_before_ack_and_restart() {
    let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, true, true);
    let mut encounters: PveEncounterDatabase =
        ron::de::from_str(&fixture.read("official/data/pve/encounters.ron")).unwrap();
    let mut encounter = encounters
        .encounters
        .iter()
        .find(|encounter| encounter.id == "suppress_warm_hearted_woodsman")
        .unwrap()
        .clone();
    encounter.id = "r05_event_defeat".into();
    encounter
        .terminal_outcomes
        .push(crate::game::data::pve_data::PveTerminalOutcomeData {
            id: "synthetic_timeout_defeat".into(),
            verdict: BattleVerdict::Defeat,
            condition: crate::game::data::pve_data::PveTerminalConditionData::TimeReached {
                time_ms: 10_000,
            },
            priority: 1,
        });
    encounter
        .outcome_policies
        .push(crate::game::data::pve_data::PveOutcomePolicyData {
            outcome_id: "synthetic_timeout_defeat".into(),
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: vec![],
            grant_node_bonus: false,
            advance_encounter_progression: false,
        });
    encounters.encounters.push(encounter);
    fixture.write(
        "official/data/pve/encounters.ron",
        ron::ser::to_string(&encounters).unwrap(),
    );
    let mut events: crate::game::data::event_data::EventDatabase =
        ron::de::from_str(&fixture.read("official/data/events/story/base.ron")).unwrap();
    events.events.push(combat_choice_event_definition(
        "r05_event",
        "fight",
        "r05_event_defeat",
        "f-05-32_warm_hearted_woodsman",
    ));
    fixture.write(
        "official/data/events/story/base.ron",
        ron::ser::to_string(&events).unwrap(),
    );
    // The existing event-combat fixture uses this deterministic successful
    // employee check seed; the check still executes through the public command.
    let mut core = GameCore::new(fixture.load().into_fixture_database_arc(), 123);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let (employee, _) = enter_resource_battle(&mut core, player);
    advance_to_resource_cost(&mut core, 2);
    finish_actual_resource_battle(&mut core, player);
    let before_event = core.run_resources().unwrap().clone();
    let node = force_single_event_node_run_for_combat_tests(&mut core, "r05_event");
    enter_event_combat_choice(&mut core, player, node, "r05_event", "fight");
    let active = core.state.active_battle.as_ref().unwrap();
    let end = active
        .combat_preview
        .routes
        .iter()
        .find(|route| route.id == "defense_main")
        .unwrap()
        .end;
    let position = active
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .filter(|cell| {
            active
                .battle
                .player_tactical_slot_occupant(*cell, None)
                .is_none()
        })
        .min_by_key(|cell| (cell.manhattan(&end), cell.y, cell.x))
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 2);
    for _ in 0..6000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        if core.state.active_battle.is_none() {
            break;
        }
    }
    assert!(matches!(
        &core.state.confirmed_combat_result.as_ref().unwrap().summary,
        crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
            verdict: BattleVerdict::Defeat,
            ..
        }
    ));
    let confirmed = core.run_resources().unwrap().clone();
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(core.run_resources(), Some(&confirmed));
    let committed = core.run_resources().unwrap().clone();
    assert_eq!(committed.entries.len(), 2);
    assert!(committed.entries.iter().all(|entry| entry.balance == 3));
    assert_eq!(committed.revision, before_event.revision + 1);
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(core.run_resources(), Some(&committed));
    assert!(core
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .is_err());
    let restored = u04_continue(&core, player);
    assert_eq!(restored.run_resources(), Some(&committed));
}

#[test]
fn actual_admin_acquisition_recruitment_and_employee_status_preserve_owner_identity() {
    use crate::game::world::AdminCommand;
    let (mut core, player, employee, _) = start_resource_battle(external_resource_data_options(
        ResourceLifetime::Run,
        false,
        true,
        true,
    ));
    advance_to_resource_cost(&mut core, 2);
    finish_actual_resource_battle(&mut core, player);
    let original = core.run_resources().unwrap().clone();
    for (physical_damage, trauma) in [(5, 100), (6, 100), (0, 0)] {
        core.execute_admin_command(AdminCommand::AdminSetEmployeeCondition {
            employee_uuid: employee,
            physical_damage,
            trauma,
        })
        .unwrap();
        assert_eq!(
            core.run_resources(),
            Some(&original),
            "same UUID remains the owner in every condition"
        );
    }
    let before = serialized_grant_roots(&core);
    assert!(core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "missing_resource_weapon".into(),
            count: 1,
        })
        .is_err());
    assert_eq!(serialized_grant_roots(&core), before);
    assert_eq!(core.run_resources(), Some(&original));
    let granted = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "standard_training_sword".into(),
            count: 1,
        })
        .unwrap();
    let copy: Uuid = serde_json::from_value(granted.payload["item_uuids"][0].clone()).unwrap();
    assert!(original
        .entries
        .iter()
        .all(|entry| entry.owner != ResourceOwner::EquipmentCopy(copy)));
    assert_eq!(
        core.run_resources(),
        Some(&original),
        "acquisition does not materialize an unused initial balance"
    );
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        MapNodePayload::HeadquartersContact {
            shop_pool_id: Some("headquarters_basic_supplies".into()),
            candidate_count: 3,
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let entered = core
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = entered
    else {
        panic!("headquarters candidates")
    };
    let recruited = core
        .execute(
            player,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id: recruitment_candidates[0].employee_definition_id.clone(),
            },
        )
        .unwrap();
    let BehaviorResult::EmployeeRecruited {
        employee_uuid: recruit,
        ..
    } = recruited
    else {
        panic!("recruit")
    };
    assert_ne!(recruit, employee);
    assert_eq!(core.run_resources(), Some(&original));
    core.execute(
        player,
        PlayerBehavior::EquipItem {
            item_uuid: copy,
            target_unit: recruit,
        },
    )
    .unwrap();
    let (first, position) = enter_resource_battle(&mut core, player);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: first,
        },
    )
    .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: recruit,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    advance_to_resource_cost(&mut core, 2);
    let entries: Vec<_> = core.battle_typed_resources().unwrap().entries().collect();
    assert_eq!(entries.len(), 4);
    for owner in [
        ResourceOwner::Employee(recruit),
        ResourceOwner::EquipmentCopy(copy),
    ] {
        assert!(entries
            .iter()
            .any(|(key, value)| key.owner == owner && value.balance == 5));
    }
    finish_actual_resource_battle(&mut core, player);
    assert_eq!(core.run_resources().unwrap().entries.len(), 4);
    for entry in &original.entries {
        assert!(core.run_resources().unwrap().entries.contains(entry));
    }
}

#[test]
fn actual_shop_and_reward_acquisitions_create_distinct_spendable_copy_owners() {
    use crate::game::data::shop_data::{ShopMerchandise, ShopMerchandiseEntry};
    let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, true, true);
    let mut shops: crate::game::data::shop_data::ShopDatabase =
        ron::de::from_str(&fixture.read("official/data/events/shops/base.ron")).unwrap();
    let shop = shops
        .shops
        .iter_mut()
        .find(|shop| shop.id == "unknown_exploration_shop")
        .unwrap();
    shop.offer_count = 1;
    shop.preset_refs.clear();
    shop.local_entries = vec![ShopMerchandiseEntry {
        merchandise: ShopMerchandise::Equipment {
            equipment_id: "standard_training_sword".into(),
        },
        quantity: 1,
        weight: 1,
        repeatable: false,
        buy_percent: None,
        tags: vec![],
    }];
    // RON 0.5 serializes internally tagged names as strings; the production
    // raw catalog requires the authored identifier spelling for these tags.
    let shop_ron = ron::ser::to_string(&shops)
        .unwrap()
        .replace("type:\"equipment\"", "type:equipment")
        .replace("type:\"consumable\"", "type:consumable")
        .replace("type:\"monolith\"", "type:monolith");
    fixture.write("official/data/events/shops/base.ron", shop_ron);
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read("official/data/events/rewards/base.ron")).unwrap();
    let mut reward = rewards.rewards[0].clone();
    reward.id = "r05_resource_weapon_reward".into();
    reward.uuid = Uuid::from_u128(0xCA05_600);
    reward.effects = vec![RewardEffect::GrantEquipment {
        equipment_id: "standard_training_sword".into(),
    }];
    rewards.rewards.push(reward);
    rewards
        .pools
        .push(crate::game::data::reward_data::RewardPoolMetadata {
            id: "r05_resource_reward_pool".into(),
            reward_ids: vec!["r05_resource_weapon_reward".into()],
        });
    fixture.write(
        "official/data/events/rewards/base.ron",
        ron::ser::to_string(&rewards).unwrap(),
    );
    let (mut base, player, employee, _) =
        start_resource_battle(fixture.load().into_fixture_database_arc());
    advance_to_resource_cost(&mut base, 2);
    finish_actual_resource_battle(&mut base, player);
    let original = base.run_resources().unwrap().clone();
    let old_copy = original
        .entries
        .iter()
        .find_map(|entry| match entry.owner {
            ResourceOwner::EquipmentCopy(copy) => Some(copy),
            _ => None,
        })
        .unwrap();
    for acquisition in ["shop", "reward"] {
        let mut core = base.clone();
        let before: std::collections::BTreeSet<_> = core
            .state
            .inventory
            .equipments
            .iter()
            .map(|item| item.instance_uuid)
            .collect();
        let node = if acquisition == "shop" {
            core.execute_admin_command(crate::game::world::AdminCommand::AdminSetEnkephalin {
                amount: 1000,
            })
            .unwrap();
            force_first_available_node(
                &mut core,
                MapNodeCategory::Shop,
                "shop_general",
                MapNodePayload::Shop {
                    shop_id: Some("unknown_exploration_shop".into()),
                    shop_pool_id: None,
                },
            )
        } else {
            force_first_available_node(
                &mut core,
                MapNodeCategory::Reward,
                "reward_treasure",
                MapNodePayload::Reward {
                    reward_pool_id: Some("r05_resource_reward_pool".into()),
                },
            )
        };
        core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
            .unwrap();
        let entered = core
            .execute(player, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        if acquisition == "shop" {
            let BehaviorResult::ShopState { shop, .. } = entered else {
                panic!("shop")
            };
            let offer = &shop.offers[0];
            let currency = core.get_enkephalin();
            core.execute(
                player,
                PlayerBehavior::PurchaseItem {
                    offer_id: offer.offer_id.clone(),
                },
            )
            .unwrap();
            assert_eq!(core.get_enkephalin(), currency - offer.total_price);
            core.execute(player, PlayerBehavior::ExitShop).unwrap();
        } else {
            core.execute(player, PlayerBehavior::ClaimReward).unwrap();
            core.execute(player, PlayerBehavior::ExitReward).unwrap();
        }
        assert_eq!(core.run_resources(), Some(&original));
        let copies: Vec<_> = core
            .state
            .inventory
            .equipments
            .iter()
            .filter(|item| !before.contains(&item.instance_uuid))
            .map(|item| item.instance_uuid)
            .collect();
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        assert_ne!(copy, old_copy);
        core.execute(
            player,
            PlayerBehavior::UnEquipItem {
                item_uuid: old_copy,
                target_unit: employee,
            },
        )
        .unwrap();
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: copy,
                target_unit: employee,
            },
        )
        .unwrap();
        enter_resource_battle(&mut core, player);
        advance_to_resource_cost(&mut core, 2);
        let entries: Vec<_> = core.battle_typed_resources().unwrap().entries().collect();
        assert!(entries.iter().any(
            |(key, value)| key.owner == ResourceOwner::EquipmentCopy(copy) && value.balance == 5
        ));
        assert!(entries.iter().any(|(key, value)| key.owner
            == ResourceOwner::EquipmentCopy(old_copy)
            && value.balance == 5));
        assert!(entries.iter().any(
            |(key, value)| key.owner == ResourceOwner::Employee(employee) && value.balance == 3
        ));
        let facts = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .runtime_fact_snapshot()
            .unwrap();
        assert!(facts
            .providers
            .iter()
            .any(
                |provider| provider.provider.persistent_copy_uuid == Some(copy)
                    && provider
                        .summary
                        .latest_cost
                        .as_ref()
                        .is_some_and(|cost| cost.after == 5)
            ));
        finish_actual_resource_battle(&mut core, player);
        assert_eq!(core.run_resources().unwrap().entries.len(), 3);
    }
}

#[test]
fn terminal_nonempty_result_enforces_native_and_authoring_budgets_before_commit() {
    use crate::authoring::control::{
        self, CancellationToken, ExecutionLimits, ExecutionPhase, NativeStorageProvenance,
    };
    use crate::game::battle::runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    let data = external_resource_data_options(ResourceLifetime::Run, false, true, true);
    let limits = ExecutionLimits {
        max_work: 1 << 30,
        ..Default::default()
    };
    let token = CancellationToken::default();
    let phase = ExecutionPhase::TrialPreparation;
    let ((mut source, player), _, provenance) = control::run_with_provenance(
        &limits,
        &token,
        phase,
        NativeStorageProvenance::fresh(),
        || {
            let (mut core, player, _, _) = start_resource_battle(data);
            advance_to_resource_cost(&mut core, 2);
            advance_to_resource_terminal_boundary(&mut core);
            assert!(core.state.active_battle.is_some());
            (core, player)
        },
    )
    .unwrap();
    let snapshot = |core: &GameCore| {
        json!({
            "run": core.get_run_snapshot_json().unwrap(),
            "roots": serialized_grant_roots(core),
            "resources": core.run_resources(),
            "pending": format!("{:?}", core.state.active_node_content),
            "profile": serde_json::to_value(&core.state.player_profile).unwrap(),
        })
    };
    let before = snapshot(&source);
    let extent = source
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .retained_runtime_fact_extent()
        .unwrap();
    for clone_fault in [true, false] {
        let mut candidate = source.clone();
        let native_limits = if clone_fault {
            RuntimeFactLimits {
                max_clone_slots: 1,
                ..Default::default()
            }
        } else {
            RuntimeFactLimits {
                max_retained_bytes: extent - 1,
                ..Default::default()
            }
        };
        let candidate_before = snapshot(&candidate);
        for _ in 0..2 {
            let error = crate::game::battle::runtime_control::run(native_limits, || {
                candidate.advance_active_battle_to_next_event_bucket()
            })
            .unwrap_err();
            assert!(
                matches!(error, GameError::BattleRuntimeFault(fault) if fault.code ==
                if clone_fault { RuntimeFaultCode::CloneLimit } else { RuntimeFaultCode::RetainedByteLimit }),
                "{error:?}"
            );
            assert_eq!(snapshot(&candidate), candidate_before);
        }
        candidate
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
        assert!(candidate
            .run_resources()
            .unwrap()
            .entries
            .iter()
            .all(|entry| entry.balance == 5));
        assert_eq!(candidate.run_resources().unwrap().entries.len(), 2);
    }
    let mut successful = source.clone();
    let (result, usage, _) =
        control::run_with_provenance(&limits, &token, phase, provenance, || {
            successful.advance_active_battle_to_next_event_bucket()
        })
        .unwrap();
    result.unwrap();
    assert!(usage.clone_slots_peak > 0 && usage.work > 0);
    for clone_fault in [true, false] {
        let mut candidate = source.clone();
        let mut exact = limits.clone();
        if clone_fault {
            exact.max_clone_slots = usage.clone_slots_peak - 1;
        } else {
            exact.max_work = usage.work - 1;
        }
        let failure = control::run_with_provenance(&exact, &token, phase, provenance, || {
            candidate.advance_active_battle_to_next_event_bucket()
        })
        .err()
        .expect("budget must reject");
        assert_eq!(
            failure.code,
            if clone_fault {
                "authoring_clone_limit"
            } else {
                "authoring_work_limit"
            }
        );
        assert_eq!(snapshot(&candidate), before);
    }
    assert_eq!(snapshot(&source), before);
    source.advance_active_battle_to_next_event_bucket().unwrap();
    source
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(source.run_resources(), successful.run_resources());
    let accepted = snapshot(&source);
    assert!(source
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(snapshot(&source), accepted);
    println!("R05 pending result retained_bound={extent} authoring_usage={usage:?}; native clone/retained and authoring clone/work refusal preserve pre-terminal state; retry once");
}

#[test]
fn runtime_safety_checkpoint_restore_uses_current_host_profile_and_preserves_original() {
    use crate::game::data::content_pack::{self, InputLimits};
    use crate::{
        application::{ApplicationErrorKind, ApplicationSnapshotKind, CoreApplication},
        runtime_safety::RuntimeSafetySettings,
    };
    let fixture = external_resource_fixture_options(ResourceLifetime::Run, false, false, false);
    let extra = (0..3500).map(|n| format!("(id: \"restore_meter_{n:04}\", owner: Employee, lifetime: Run, initial: 1, cap: 15)")).collect::<Vec<_>>().join(",");
    let resources = fixture.read("official/data/resources/base.ron").replacen(
        "resources: [",
        &format!("resources: [{extra},"),
        1,
    );
    fixture.write("official/data/resources/base.ron", resources);
    let generation = fixture.load();
    let player = Uuid::from_u128(0xC250);
    let mut source = GameCore::new(generation.clone(), 123);
    start_new_game_with_default_starters(&mut source, player);
    // Synthetic previously-used, valid Run resource ledger. Catalog size alone
    // does not allocate balances: the store materializes them lazily on use.
    let employee = source.roster().unwrap().combat_eligible_employee_ids()[0];
    source.state.run.as_mut().unwrap().typed_resources.entries = (0..3500)
        .map(|n| crate::game::resource::RunResourceEntry {
            resource: crate::game::resource::ResourceId::new(format!("restore_meter_{n:04}"))
                .unwrap(),
            owner: ResourceOwner::Employee(employee),
            balance: 1,
            revision: 1,
        })
        .collect();
    let node_id = force_first_available_node(
        &mut source,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    source
        .execute(player, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    source
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    source
        .execute(player, PlayerBehavior::CompleteNode)
        .unwrap();
    let source = CoreApplication::from_core(source);
    let checkpoint = source.export_active_run_checkpoint().unwrap().unwrap();
    let original = checkpoint.core_payload_json().to_vec();
    let raw = std::str::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
        .unwrap()
        .replace("max_retained_bytes: 16777216", "max_retained_bytes: 262144");
    let profile = Arc::new(RuntimeSafetySettings::from_ron("small-host", raw.as_bytes()).unwrap());
    let selected = content_pack::load_with_runtime_safety(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
        profile,
    )
    .unwrap();
    assert_eq!(generation.identity(), selected.identity());
    let mut rejected = CoreApplication::new(selected, 555);
    install_application_profile_for_continue(&mut rejected, &source);
    let before = rejected.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let fault = rejected
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap_err();
    assert_eq!(fault.kind(), ApplicationErrorKind::BattleRuntimeFault);
    assert_eq!(fault.code(), "battle_runtime_retained_byte_limit");
    assert_eq!(
        rejected.snapshot(ApplicationSnapshotKind::Run).unwrap(),
        before
    );
    assert_eq!(checkpoint.core_payload_json(), original);
    assert_eq!(
        source
            .export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .core_payload_json(),
        original
    );
    let mut accepted = CoreApplication::new(generation, 555);
    install_application_profile_for_continue(&mut accepted, &source);
    accepted
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    assert!(accepted.run_resources().is_some());
    assert_eq!(checkpoint.core_payload_json(), original);
}
