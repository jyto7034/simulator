use super::*;
use crate::game::employee::Employee;

fn start_selected(ids: &[&str]) -> (GameCore, Uuid) {
    start_selected_with_data(ids, live_game_data_from_ron())
}

pub(super) fn start_selected_with_data(ids: &[&str], data: Arc<GameDataBase>) -> (GameCore, Uuid) {
    let mut core = GameCore::new(data, 0xC006);
    let player = Uuid::from_u128(1);
    let result = core
        .execute(
            player,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap();
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(vn),
        ..
    } = result
    else {
        panic!("briefing")
    };
    core.execute(
        player,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: vn.session_id,
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                vn.current_scene_id,
            ),
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids: ids.iter().map(|id| id.to_string()).collect(),
            loan_copy_ids: vec![],
        },
    )
    .unwrap();
    (core, player)
}

fn assert_initial_copy(core: &GameCore, employee: &Employee) {
    let definition = core
        .game_data
        .employee_definition_data
        .get_by_id(&employee.employee_definition_id)
        .unwrap();
    assert!(definition
        .initial_loadout
        .baseline_skill_fragment_ids
        .is_empty());
    let active = employee
        .skill_fragments
        .active_copy_uuid()
        .expect("active training copy");
    let (copy, metadata, form) = core
        .state
        .skill_fragments
        .resolve(&active, &core.game_data.skill_fragment_data)
        .unwrap();
    assert_eq!(copy.equipped_to, Some(employee.uuid));
    assert_eq!(
        Some(&metadata.id),
        definition.initial_loadout.active_skill_fragment_id.as_ref()
    );
    assert_eq!(copy.selected_form, "base");
    assert_eq!(form.resonance_max, 100);
    assert!(
        core.skill_fragment_compatibility_report_for_employee(employee.uuid, active)
            .unwrap()
            .is_compatible
    );
    let equipped: Vec<_> = employee
        .loadout
        .item_slot
        .iter()
        .map(|item| {
            let copy = core
                .state
                .inventory
                .equipments
                .get_item(&item.instance_uuid)
                .unwrap();
            assert_eq!(copy.equipped_to, Some(employee.uuid));
            assert_eq!(copy.meta.uuid, item.base_uuid);
            copy.meta.id.clone()
        })
        .collect();
    assert_eq!(equipped, definition.initial_loadout.equipment_ids);
}

#[test]
fn u12_display_aptitude_alone_preserves_weapon_and_effective_battle_inputs() {
    use crate::game::employee::CombatAptitude;
    let (mut core, _) = start_selected(&["field_medic", "breach_guard", "records_clerk"]);
    let employee = core.state.roster.iter().next().unwrap().uuid;
    let baseline = core.state.roster.get(&employee).unwrap().clone();
    let profile = |core: &GameCore| serde_json::to_value(
        crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
            &core.state.roster, &core.state.inventory, &core.state.skill_fragments,
            core.game_data.as_ref(), employee,
        ).expect("same authored weapon remains eligible")
    ).unwrap();
    let expected = profile(&core);
    assert!(expected["weapon_profile"].is_object());
    for aptitude in [CombatAptitude::Defensive, CombatAptitude::Offensive] {
        core.state.roster.get_mut(&employee).unwrap().profile.combat_aptitude = aptitude;
        let changed = core.state.roster.get(&employee).unwrap();
        assert_eq!(changed.profile.base_combat_stats, baseline.profile.base_combat_stats);
        assert_eq!(serde_json::to_value(&changed.loadout).unwrap(), serde_json::to_value(&baseline.loadout).unwrap());
        assert_eq!(profile(&core), expected, "display aptitude must not alter any battle input");
    }
}

#[test]
fn u06_starter_six_normal_sets_grant_exact_copies_and_continue_bindings() {
    for ids in [
        ["field_medic", "breach_guard", "records_clerk"],
        ["scout_runner", "containment_trainee", "maintenance_tech"],
    ] {
        let (core, player) = start_selected(&ids);
        assert_eq!(core.state.roster.len(), 3);
        assert_eq!(core.state.skill_fragments.copies().count(), 3);
        for employee in core.state.roster.iter() {
            assert_initial_copy(&core, employee);
        }
        core.state
            .skill_fragments
            .validate_bindings(&core.state.roster, &core.game_data.skill_fragment_data)
            .unwrap();
        let export = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut restored = core_with_profile_for_continue(&core, 0xDEAD);
        restored
            .continue_from_active_run_checkpoint(
                player,
                &export.core_payload_json,
                export.loads_used,
                export.run_peak_certified_power,
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(&core.state.roster).unwrap(),
            serde_json::to_value(&restored.state.roster).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&core.state.skill_fragments).unwrap(),
            serde_json::to_value(&restored.state.skill_fragments).unwrap()
        );
    }
}

#[test]
fn u06_starter_recruit_normal_loadout_and_post_grant_failure_rollback() {
    let (mut core, player) = start_selected(&["field_medic", "breach_guard", "records_clerk"]);
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        MapNodePayload::HeadquartersContact {
            shop_pool_id: Some("headquarters_basic_supplies".into()),
            candidate_count: 9,
        },
    );
    let result = select_and_confirm_map_node(&mut core, player, node);
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = result
    else {
        panic!("recruitment")
    };
    let id = recruitment_candidates
        .iter()
        .find(|c| c.employee_definition_id.starts_with("hq_"))
        .unwrap()
        .employee_definition_id
        .clone();
    let before = non_battle_state_without_profile_fingerprint(&core);
    let result: Result<(), GameError> = core.with_state_transaction(|candidate| {
        candidate.handle_recruit_employee(&id)?;
        Err(GameError::InvalidAction)
    });
    assert!(result.is_err());
    assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
    let result = core
        .execute(
            player,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id: id,
            },
        )
        .unwrap();
    let BehaviorResult::EmployeeRecruited { employee_uuid, .. } = result else {
        panic!("recruited")
    };
    assert_eq!(core.state.roster.len(), 4);
    assert_initial_copy(&core, core.state.roster.get(&employee_uuid).unwrap());
    assert_eq!(core.state.skill_fragments.copies().count(), 4);
}

#[test]
fn u06_starter_public_recruitment_stops_at_ten_with_exact_ownership() {
    let (mut core, player) = start_selected(&["field_medic", "breach_guard", "records_clerk"]);
    // Boundary fixture funds all seven paid hires without relying on battle rewards.
    core.state.enkephalin.amount = 1000;
    for expected in 4..=11 {
        let node = force_first_available_node(
            &mut core,
            MapNodeCategory::HeadquartersContact,
            "headquarters_contact",
            MapNodePayload::HeadquartersContact {
                shop_pool_id: Some("headquarters_basic_supplies".into()),
                candidate_count: 9,
            },
        );
        let result = select_and_confirm_map_node(&mut core, player, node);
        let BehaviorResult::HeadquartersContactState {
            recruitment_candidates,
            ..
        } = result
        else {
            panic!("headquarters at roster size {}", expected - 1)
        };
        let id = if expected == 11 {
            assert!(recruitment_candidates.is_empty(), "full roster hides paid offers");
            core.state.run.as_ref().unwrap().recruitment.active_session().unwrap()
                .employee_definition_ids[0].clone()
        } else {
            recruitment_candidates[0].employee_definition_id.clone()
        };
        let before = non_battle_state_without_profile_fingerprint(&core);
        let result = core.execute(
            player,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id: id,
            },
        );
        if expected == 11 {
            assert!(result.is_err());
            assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
            assert_eq!(core.state.roster.len(), 10);
        } else {
            result.unwrap();
            assert_eq!(core.state.roster.len(), expected);
            assert_eq!(core.state.skill_fragments.copies().count(), expected);
            for employee in core.state.roster.iter() {
                assert_initial_copy(&core, employee);
            }
            core.state
                .run
                .as_ref()
                .unwrap()
                .recruitment
                .validate_exact_ownership(
                    &core.game_data.employee_definition_data.definitions,
                    &core.state.roster,
                )
                .unwrap();
            core.execute(player, PlayerBehavior::RequestEmergencySupplies).unwrap();
        }
    }

    core.execute(player, PlayerBehavior::RequestEmergencySupplies)
        .unwrap();
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        MapNodePayload::Encounter {
            encounter_id: Some("suppress_burrowing_heaven".into()),
        },
    );
    select_and_confirm_map_node(&mut core, player, node);
    // This boundary fixture supplies TP without advancing into enemy waves.
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .apply_delta(99);
    let employees: Vec<_> = core.state.roster.iter().map(|e| e.uuid).collect();
    let destination = |core: &GameCore, employee| {
        let active = core.state.active_battle.as_ref().unwrap();
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                core.game_data.as_ref(),
                employee,
            )
            .unwrap();
        active
            .combat_preview
            .deployment_zones
            .iter()
            .filter(|zone| zone.kind.supports_affinity(profile.deployment_affinity))
            .flat_map(|zone| zone.cells.iter().copied())
            .find(|position| {
                active
                    .battle
                    .validate_player_tactical_destination(*position, None)
                    .is_ok()
            })
            .expect("an unoccupied compatible authored deployment cell")
    };
    for employee in employees.iter().take(8) {
        let position = destination(&core, *employee);
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: *employee,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    }
    let position = destination(&core, employees[8]);
    let active = core.state.active_battle.as_ref().unwrap();
    let before = serde_json::to_value(
        active
            .battle_update_dto_after_or_resync(0, active.employee_roster())
            .unwrap(),
    )
    .unwrap();
    let result = execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employees[8],
            position,
            facing: FacingDirection::Right,
        },
    );
    assert!(
        matches!(
            result,
            Err(GameError::ActionUnavailable {
                reason: crate::game::behavior::ActionDisabledReasonCode::DeploymentLimit,
                ..
            })
        ),
        "{result:?}"
    );
    assert_eq!(
        serde_json::to_value({
            let active = core.state.active_battle.as_ref().unwrap();
            active
                .battle_update_dto_after_or_resync(0, active.employee_roster())
                .unwrap()
        })
        .unwrap(),
        before
    );
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employees[0],
        },
    )
    .unwrap();
    let position = destination(&core, employees[8]);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employees[8],
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .live_deployment
            .as_ref()
            .unwrap()
            .deployed_units
            .len(),
        8
    );
}

#[test]
fn u06_live_six_starter_actives_execute_through_core_application() {
    use crate::application::{
        ApplicationBattleAuthority, ApplicationCommand, ApplicationDocument,
        ApplicationExecutionContext, CoreApplication,
    };
    use crate::game::behavior::LiveBattleStateCheckpointDto;
    let cases = [
        ("scout_runner", "training_adrenaline"),
        ("records_clerk", "training_berserker"),
        ("breach_guard", "training_last_line"),
        ("containment_trainee", "training_last_line"),
        ("field_medic", "training_single_heal"),
        ("maintenance_tech", "training_area_heal"),
    ];
    for (definition, skill) in cases {
        let ids: Vec<_> = std::iter::once(definition)
            .chain(
                cases
                    .iter()
                    .map(|case| case.0)
                    .filter(|id| *id != definition)
                    .take(2),
            )
            .collect();
        let (mut core, player) = start_selected(&ids);
        let employee = core
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == definition)
            .unwrap()
            .uuid;
        let node = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            MapNodePayload::Encounter {
                encounter_id: Some("suppress_burrowing_heaven".into()),
            },
        );
        select_and_confirm_map_node(&mut core, player, node);
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                core.game_data.as_ref(),
                employee,
            )
            .unwrap();
        let position = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .combat_preview
            .deployment_zones
            .iter()
            .find(|zone| zone.kind.supports_affinity(profile.deployment_affinity))
            .unwrap()
            .cells[0];
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
        let active = core.state.active_battle.as_mut().unwrap();
        let unit_id =
            active.live_deployment.as_ref().unwrap().deployed_units[&employee].unit_instance_id;
        // Explicit readiness/injury fixture isolates live authored effects from the 80s charge wait.
        // The employee, equipment, active copy, compiled skill and deployment all use normal paths.
        let unit = active.battle.units.get_mut(&unit_id).unwrap();
        unit.resonance_current = unit.resonance_max;
        unit.stats.current_health = 10;
        let before = unit.stats.clone();
        if skill == "training_single_heal" || skill == "training_area_heal" {
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::BeginSkillTargeting {
                    employee_uuid: employee,
                    skill_id: skill.into(),
                },
            )
            .unwrap();
        }
        let active = core.state.active_battle.as_ref().unwrap();
        let authority = ApplicationBattleAuthority {
            battle_uuid: active.battle_uuid,
            battle_state_revision: active.battle_state_revision,
        };
        let target = match skill {
            "training_single_heal" => Some(SkillCastTarget::Unit {
                unit_instance_id: unit_id,
            }),
            "training_area_heal" => Some(SkillCastTarget::Tile { position }),
            _ => None,
        };
        let mut app = CoreApplication::from_core(core);
        let command = ApplicationCommand::from_document(ApplicationDocument::new(
            serde_json::to_value(PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: skill.into(),
                target,
            })
            .unwrap(),
        ))
        .unwrap();
        app.execute(
            player,
            command,
            ApplicationExecutionContext {
                expected_battle_authority: Some(authority),
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| panic!("{skill}: {error:?}"));
        app.advance_active_battle_by(1, 0).unwrap();
        let state = app.active_battle_reconnect(0).unwrap();
        let checkpoint: LiveBattleStateCheckpointDto = serde_json::from_value(
            state.payload().as_value()["battle_update"]["checkpoint"].clone(),
        )
        .unwrap();
        let unit = checkpoint
            .units
            .iter()
            .find(|u| u.unit_instance_id == unit_id)
            .unwrap();
        assert_eq!(unit.hud.resonance_max, Some(100));
        assert_eq!(
            unit.hud.resonance_current,
            Some(0),
            "{skill}: accepted once"
        );
        let effects = &checkpoint.sustained_effects.active;
        match skill {
            "training_adrenaline" | "training_berserker" => {
                assert!(unit.stats.attack > before.attack, "{skill}");
                assert!(
                    unit.stats.attack_interval_ms < before.attack_interval_ms,
                    "{skill}"
                );
                if skill == "training_berserker" {
                    assert!(effects.iter().any(|e| e
                        .combat_modifiers
                        .is_some_and(|m| m.basic_lifesteal_percent == 30)));
                }
            }
            "training_last_line" => assert!(effects.iter().any(|e| e
                .combat_modifiers
                .is_some_and(|m| m.damage_reduction_percent == 50))),
            "training_single_heal" => assert_eq!(unit.stats.current_health, 70),
            "training_area_heal" => assert_eq!(unit.stats.current_health, 35),
            _ => unreachable!(),
        }
        if skill == "training_single_heal" || skill == "training_area_heal" {
            assert_eq!(
                unit.basic_action.kind,
                crate::game::behavior::LiveBattleBasicActionKind::Heal
            );
            let after_active_hp = unit.stats.current_health;
            app.advance_active_battle_by(2500, 0).unwrap();
            let next = app.active_battle_reconnect(0).unwrap();
            let next: LiveBattleStateCheckpointDto = serde_json::from_value(
                next.payload().as_value()["battle_update"]["checkpoint"].clone(),
            )
            .unwrap();
            let healed = next
                .units
                .iter()
                .find(|u| u.unit_instance_id == unit_id)
                .unwrap();
            assert!(
                healed.stats.current_health > after_active_hp,
                "{skill}: normal public clock must also resolve the authored basic Heal"
            );
            assert!(healed.stats.current_health <= healed.stats.max_health);
        } else {
            assert_eq!(
                unit.basic_action.kind,
                crate::game::behavior::LiveBattleBasicActionKind::Damage
            );
        }
    }
}

#[test]
fn u06_basic_heal_public_query_and_clock_agree_on_current_recovery() {
    use crate::application::{
        ApplicationBattleAuthority, ApplicationCommand, ApplicationDocument,
        ApplicationExecutionContext, CoreApplication,
    };
    use crate::game::{
        behavior::{LiveBattleBasicActionKind, LiveBattleStateCheckpointDto},
        data::{abnormality_data::BasicAttackAction, ValidatedGameData},
    };
    for (definition, weapon) in [
        ("field_medic", "standard_training_healing_staff"),
        ("maintenance_tech", "standard_training_area_healing_staff"),
    ] {
        for (flat, percent, spell_power, useful) in [
            (0, 0, 0, false),
            (0, 100, 0, false),
            (0, 1, 99, false),
            (0, 50, 40, true),
        ] {
            let live = ValidatedGameData::load_live_embedded().unwrap();
            let mut equipment = live.equipment_data.as_ref().clone();
            let profile = equipment
                .items
                .iter_mut()
                .find(|item| item.id == weapon)
                .unwrap()
                .weapon_profile
                .as_mut()
                .unwrap();
            let BasicAttackAction::Heal {
                flat: authored_flat,
                spell_power_percent,
                ..
            } = &mut profile.action
            else {
                panic!("authored healing staff")
            };
            *authored_flat = flat;
            *spell_power_percent = percent;
            // Preserve the live area footprint and use normal grants/equipment/deployment.
            let data = GameDataBuilder::from_validated(&live)
                .with_equipment_data(Arc::new(equipment))
                .build_arc();
            let (mut core, player) =
                start_selected_with_data(&[definition, "breach_guard", "records_clerk"], data);
            let employee = core
                .state
                .roster
                .iter()
                .find(|e| e.employee_definition_id == definition)
                .unwrap()
                .uuid;
            let node = force_first_available_node(
                &mut core,
                MapNodeCategory::Combat,
                "combat_monster",
                MapNodePayload::Encounter {
                    encounter_id: Some("suppress_burrowing_heaven".into()),
                },
            );
            select_and_confirm_map_node(&mut core, player, node);
            let profile =
                crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                    core.roster().unwrap(),
                    core.inventory().unwrap(),
                    &core.state.skill_fragments,
                    core.game_data.as_ref(),
                    employee,
                )
                .unwrap();
            let position = core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .combat_preview
                .deployment_zones
                .iter()
                .find(|zone| zone.kind.supports_affinity(profile.deployment_affinity))
                .unwrap()
                .cells[0];
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
            let active = core.state.active_battle.as_mut().unwrap();
            let unit_id =
                active.live_deployment.as_ref().unwrap().deployed_units[&employee].unit_instance_id;
            // Explicit injury/current-stat fixture; the authored action is not replaced in memory.
            let unit = active.battle.units.get_mut(&unit_id).unwrap();
            unit.stats.current_health = 50;
            unit.stats.spell_power = spell_power;
            let authority = ApplicationBattleAuthority {
                battle_uuid: active.battle_uuid,
                battle_state_revision: active.battle_state_revision,
            };
            let mut app = CoreApplication::from_core(core);
            let command = ApplicationCommand::from_document(ApplicationDocument::new(
                serde_json::to_value(PlayerBehavior::FocusBattleUnit {
                    unit_instance_id: unit_id,
                })
                .unwrap(),
            ))
            .unwrap();
            app.execute(
                player,
                command,
                ApplicationExecutionContext {
                    expected_battle_authority: Some(authority),
                    ..Default::default()
                },
            )
            .unwrap();
            let checkpoint = |app: &mut CoreApplication| {
                let result = app.active_battle_reconnect(0).unwrap();
                serde_json::from_value::<LiveBattleStateCheckpointDto>(
                    result.payload().as_value()["battle_update"]["checkpoint"].clone(),
                )
                .unwrap()
            };
            let before = checkpoint(&mut app);
            let before = before
                .units
                .iter()
                .find(|unit| unit.unit_instance_id == unit_id)
                .unwrap();
            assert_eq!(before.basic_action.kind, LiveBattleBasicActionKind::Heal);
            assert!(before.basic_action.preview_available);
            assert_eq!(
                before.basic_action.eligible_target_ids,
                if useful { vec![unit_id] } else { vec![] }
            );
            assert_eq!(
                before.basic_action.selected_target_id,
                useful.then_some(unit_id)
            );
            app.advance_active_battle_by(2500, 0).unwrap();
            let after = checkpoint(&mut app);
            let after = after
                .units
                .iter()
                .find(|unit| unit.unit_instance_id == unit_id)
                .unwrap();
            if useful {
                assert!(after.stats.current_health > 50);
            } else {
                assert_eq!(after.stats.current_health, 50);
                assert!(after.basic_action.eligible_target_ids.is_empty());
                assert_eq!(after.basic_action.selected_target_id, None);
            }
            println!("F01 public {definition}: flat={flat}, percent={percent}, SP={spell_power}, useful={useful}, final_hp={}", after.stats.current_health);
        }
    }
}

#[test]
fn u06_starter_all_six_copies_transfer_by_weapon_compatibility() {
    for definition in [
        "scout_runner",
        "records_clerk",
        "breach_guard",
        "containment_trainee",
        "field_medic",
        "maintenance_tech",
    ] {
        let target_definition = if definition == "scout_runner" {
            "records_clerk"
        } else {
            "scout_runner"
        };
        let third = if definition == "breach_guard" {
            "field_medic"
        } else {
            "breach_guard"
        };
        let (mut core, player) = start_selected(&[definition, target_definition, third]);
        let source = core
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == definition)
            .unwrap()
            .uuid;
        let target = core
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == target_definition)
            .unwrap()
            .uuid;
        let copy = core
            .state
            .roster
            .get(&source)
            .unwrap()
            .skill_fragments
            .active_copy_uuid()
            .unwrap();
        let displaced = core
            .state
            .roster
            .get(&target)
            .unwrap()
            .skill_fragments
            .active_copy_uuid()
            .unwrap();
        core.execute(
            player,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid: target,
                copy_uuid: displaced,
            },
        )
        .unwrap();
        let before = non_battle_state_without_profile_fingerprint(&core);
        let result = core.execute(
            player,
            PlayerBehavior::TransferSkillFragment {
                copy_uuid: copy,
                target_employee_uuid: target,
            },
        );
        assert!(
            matches!(result, Err(GameError::SkillFragmentIncompatible { .. })),
            "{definition}: {result:?}"
        );
        assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
        let weapon_id = &core
            .game_data
            .employee_definition_data
            .get_by_id(definition)
            .unwrap()
            .initial_loadout
            .equipment_ids[0];
        let weapon = core
            .game_data
            .equipment_data
            .get_by_id(weapon_id)
            .unwrap()
            .clone();
        grant_and_equip_weapon(&mut core, target, weapon, Uuid::from_u128(0xC006FFFF));
        core.execute(
            player,
            PlayerBehavior::TransferSkillFragment {
                copy_uuid: copy,
                target_employee_uuid: target,
            },
        )
        .unwrap();
        assert_eq!(
            core.state
                .roster
                .get(&source)
                .unwrap()
                .skill_fragments
                .active_copy_uuid(),
            None
        );
        assert_eq!(
            core.state
                .roster
                .get(&target)
                .unwrap()
                .skill_fragments
                .active_copy_uuid(),
            Some(copy)
        );
        assert_eq!(
            core.state.skill_fragments.get(&copy).unwrap().equipped_to,
            Some(target)
        );
        assert_eq!(
            core.state
                .skill_fragments
                .get(&displaced)
                .unwrap()
                .equipped_to,
            None
        );
        assert_eq!(core.state.skill_fragments.copies().count(), 3);
        core.state
            .skill_fragments
            .validate_bindings(&core.state.roster, &core.game_data.skill_fragment_data)
            .unwrap();
    }
}
