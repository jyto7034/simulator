fn employee_revival_world_fixture() -> ExternalFixture {
    employee_revival_world_fixture_with_transform(false)
}
fn employee_revival_world_fixture_with_transform(transform: bool) -> ExternalFixture {
    use crate::game::data::content_pack::{CatalogFile, CatalogKind, PackManifest};
    let fixture = external_resource_fixture_options(ResourceLifetime::Attempt, false, false, true);
    let path = "official/data/equipments/base.ron";
    let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    equipment
        .items
        .iter_mut()
        .find(|i| i.id == "standard_training_sword")
        .unwrap()
        .ability_activations
        .clear();
    fixture.write(path, ron::ser::to_string(&equipment).unwrap());
    let path = "official/data/employees/definitions.ron";
    let mut employees: crate::game::data::employee_data::EmployeeDefinitionDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let employee = employees
        .definitions
        .iter_mut()
        .find(|e| e.employee_definition_id == "records_clerk")
        .unwrap();
    employee
        .initial_loadout
        .equipment_ids
        .push("placeholder_nothing_there_accessory".into());
    employee.profile.base_combat_stats.max_hp = 1000;
    fixture.write(path, ron::ser::to_string(&employees).unwrap());
    let path = "official/data/abnormalities/base.ron";
    let mut abnormalities: crate::game::data::abnormality_data::AbnormalityDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    let enemy = abnormalities
        .items
        .iter_mut()
        .find(|e| e.id == "f-05-32_warm_hearted_woodsman")
        .unwrap();
    enemy.attack = 10000;
    enemy.max_health = 100000;
    enemy.skill_id = None;
    enemy.basic_attack.interval_ms = 1500;
    enemy.basic_attack.windup_ms = 200;
    enemy.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
    enemy.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
    let mut rows = vec!["XXXXXXXXXXXXXXXXX".to_string(); 17];
    rows[8] = "XXXXXXXX@XXXXXXXX".into();
    enemy.basic_attack.defense_tile_range =
        Some(crate::game::battle::tile_range::TileRangePattern {
            include_anchor_tile: false,
            rows,
        });
    if transform {
        let form = abnormalities.items.iter_mut().find(|e| e.id == "o-06-20_nothing_there").unwrap();
        form.max_health = 333; form.attack = 17; form.defense = 0; form.magic_resist = 0;
        form.skill_id = None; form.movement.speed_units_per_ms = 0;
    }
    fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
    let path = "official/data/pve/encounters.ron";
    let mut encounters: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let encounter = encounters
        .encounters
        .iter_mut()
        .find(|e| e.id == "suppress_warm_hearted_woodsman")
        .unwrap();
    encounter.waves.retain(|w| w.id == "primary_abnormality");
    fixture.write(path, ron::ser::to_string(&encounters).unwrap());
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_revival.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: placeholder_nothing_there_accessory",
            );
    let source = if transform {
        let mut source = crate::authoring::composition::canonical_overlay_tests::equipment_fixture("accessory");
        source["equipment_defenses"][0]["equipment"] = serde_json::json!("placeholder_nothing_there_accessory");
        source.to_string()
    } else { source };
    let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
    let path = "data/skills/employee-revival.skill-composition.ron";
    fixture.write(&format!("official/{path}"), artifact.to_ron().unwrap());
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.files.push(CatalogFile {
        kind: CatalogKind::Skills,
        path: path.into(),
    });
    fixture.manifest("official", manifest);
    fixture
}
fn revival_world_count(core: &GameCore, employee: Uuid, battle: bool) -> u32 {
    if battle {
        core.battle_typed_resources()
            .unwrap()
            .entries()
            .find(|(key, _)| {
                key.resource.as_str() == "revival_total"
                    && key.owner == ResourceOwner::Employee(employee)
            })
            .map_or(0, |(_, v)| v.balance)
    } else {
        core.run_resources()
            .unwrap()
            .entries
            .iter()
            .find(|e| {
                e.resource.as_str() == "revival_total"
                    && e.owner == ResourceOwner::Employee(employee)
            })
            .map_or(0, |e| e.balance)
    }
}
fn reach_employee_revival(core: &mut GameCore, employee: Uuid, expected: u32) {
    for _ in 0..299 {
        core.advance_active_battle_by(100).unwrap();
        assert!(
            core.state.active_battle.is_some(),
            "revival {expected} must occur before result; run count {}", revival_world_count(core, employee, false)
        );
        if revival_world_count(core, employee, true) == expected {
            return;
        }
    }
    panic!(
        "real enemy attack did not trigger revival: {:?}",
        core.state
            .active_battle
            .as_ref()
            .map(|a| (&a.battle.units, a.battle.runtime_fact_summary()))
    );
}
fn finish_revival_battle(core: &mut GameCore, player: Uuid) {
    for _ in 0..300 {
        if core.state.active_battle.is_none() {
            break;
        }
        core.advance_active_battle_by(100).unwrap();
    }
    assert!(core.state.active_battle.is_none());
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
}
#[test]
fn composition_revival_world_real_attack_retreat_two_results_transfer_checkpoint_and_continue() {
    let fixture = employee_revival_world_fixture();
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
    let entry = core.run_resources().unwrap().clone();
    let original_condition = core
        .state
        .roster
        .iter()
        .find(|e| e.uuid == employee)
        .unwrap()
        .condition;
    let runtime = core
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
    reach_employee_revival(&mut core, employee, 1);
    assert_eq!(core.run_resources(), Some(&entry));
    assert_eq!(
        core.state
            .roster
            .iter()
            .find(|e| e.uuid == employee)
            .unwrap()
            .condition,
        original_condition
    );
    let unit = &core.state.active_battle.as_ref().unwrap().battle.units[&runtime];
    assert!(unit.is_active());
    assert_eq!(unit.stats.current_health, unit.stats.max_health / 2);
    let mut retreat = core.clone();
    execute_current_battle_mutation(&mut retreat, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(retreat.run_resources(), Some(&entry));
    assert!(retreat.battle_typed_resources().is_none());
    retreat
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert_eq!(revival_world_count(&retreat, employee, true), 0);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    finish_revival_battle(&mut core, player);
    assert_eq!(revival_world_count(&core, employee, false), 1);
    let first = core.run_resources().unwrap().clone();
    let other = core
        .state
        .roster
        .iter()
        .find(|e| e.uuid != employee)
        .unwrap()
        .uuid;
    core.execute(
        player,
        PlayerBehavior::UnEquipItem {
            item_uuid: copy,
            target_unit: employee,
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::EquipItem {
            item_uuid: copy,
            target_unit: other,
        },
    )
    .unwrap();
    assert_eq!(core.run_resources(), Some(&first));
    assert_eq!(revival_world_count(&core, other, false), 0);
    core.execute(
        player,
        PlayerBehavior::UnEquipItem {
            item_uuid: copy,
            target_unit: other,
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
    assert_eq!(revival_world_count(&core, employee, true), 1);
    reach_employee_revival(&mut core, employee, 2);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    finish_revival_battle(&mut core, player);
    assert_eq!(revival_world_count(&core, employee, false), 2);
    assert_eq!(
        core.state
            .roster
            .iter()
            .find(|e| e.uuid == employee)
            .unwrap()
            .condition,
        original_condition
    );
    let confirmed = core.run_resources().unwrap().clone();
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
    let mut restored = crate::application::CoreApplication::new(data, 0xe680);
    install_application_profile_for_continue(&mut restored, &source);
    restored
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    assert_eq!(restored.run_resources(), Some(&confirmed));
    let command = crate::application::ApplicationCommand::from_document(
        crate::application::ApplicationDocument::new(
            serde_json::to_value(PlayerBehavior::UnEquipItem {
                item_uuid: copy,
                target_unit: employee,
            })
            .unwrap(),
        ),
    )
    .unwrap();
    restored
        .execute(player, command, Default::default())
        .unwrap();
    assert_eq!(restored.run_resources(), Some(&confirmed));
}
