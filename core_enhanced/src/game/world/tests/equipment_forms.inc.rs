fn equipment_forms_world_fixture() -> ExternalFixture {
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
    let mut accessory = equipment
        .items
        .iter()
        .find(|i| i.id == "crimson_goggles")
        .unwrap()
        .clone();
    accessory.id = "equipment_forms_fixture".into();
    accessory.uuid = Uuid::from_u128(0xe500);
    accessory.triggered_effects.clear();
    accessory.ability_activations.clear();
    equipment.items.push(accessory);
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
        .push("equipment_forms_fixture".into());
    employee.profile.base_combat_stats.max_hp = 1000;
    fixture.write(path, ron::ser::to_string(&employees).unwrap());
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/equipment_forms.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: equipment_forms_fixture",
            )
            .replace("hp: 300", "hp: 1")
            .replace("power: 20", "power: 1");
    let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
    let path = "data/skills/equipment-forms.skill-composition.ron";
    fixture.write(&format!("official/{path}"), artifact.to_ron().unwrap());
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.files.push(CatalogFile {
        kind: CatalogKind::Skills,
        path: path.into(),
    });
    fixture.manifest("official", manifest);
    let path = "official/data/enemies/corroded_employees.ron";
    let enemies = fixture.read(path);
    assert!(enemies.contains("max_health: 70,"));
    fixture.write(path, enemies.replacen("max_health: 70,", "max_health: 1,", 1));
    fixture
}
fn equipment_forms_world_pending(core: &GameCore, copy: Uuid) -> u32 {
    core.battle_typed_resources().unwrap().entries().find(|(key, _)| key.resource.as_str() == "mask_pending" && key.owner == ResourceOwner::EquipmentCopy(copy)).map_or(0, |(_, value)| value.balance)
}
fn equipment_forms_world_item(core: &GameCore, copy: Uuid) -> crate::game::resources::EquipmentItemDto {
    core.get_inventory_snapshot_dto().unwrap().equipments.into_iter().find(|item| item.instance_uuid == copy).unwrap().item
}
#[test]
fn composition_equipment_forms_world_actual_kill_retreat_display_transfer_and_checkpoint() {
    let fixture = equipment_forms_world_fixture();
    let data = fixture.load();
    let (mut core, player, employee, _) = start_resource_battle(data.clone());
    let copy = core.state.inventory.equipments.iter().find(|i| i.meta.uuid == Uuid::from_u128(0xe500) && i.equipped_to == Some(employee)).unwrap().instance_uuid;
    let entry = core.run_resources().unwrap().clone();
    let item = equipment_forms_world_item(&core, copy);
    assert_eq!(item.name, "양의 탈");
    assert_eq!((item.form.as_ref().unwrap().index, item.form.as_ref().unwrap().progress), (0, 0));
    let actor = core.state.active_battle.as_ref().unwrap().battle.units.values().find(|u| u.source_owned_uuid == employee && u.is_active()).unwrap();
    assert_eq!(actor.stats.max_health, 1180, "authored employee 1000 + starter armor 80 + mask 100");
    let base_attack = actor.stats.attack;
    for _ in 0..290 {
        core.advance_active_battle_by(100).unwrap();
        assert!(core.state.active_battle.is_some(), "direct kill must precede terminal settlement");
        if equipment_forms_world_pending(&core, copy) == 1 { break; }
    }
    assert_eq!(equipment_forms_world_pending(&core, copy), 1, "real queued attack must kill an encounter enemy");
    assert_eq!(core.run_resources(), Some(&entry));
    assert_eq!(equipment_forms_world_item(&core, copy).name, "양의 탈");
    let mut retreat = core.clone();
    execute_current_battle_mutation(&mut retreat, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(retreat.run_resources(), Some(&entry));
    retreat.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    assert_eq!(equipment_forms_world_pending(&retreat, copy), 0);
    assert_eq!(equipment_forms_world_item(&retreat, copy).name, "양의 탈");
    for _ in 0..300 {
        if core.state.active_battle.is_none() { break; }
        core.advance_active_battle_by(100).unwrap();
    }
    assert!(core.state.active_battle.is_none());
    let confirmed = core.run_resources().unwrap().clone();
    let item = equipment_forms_world_item(&core, copy);
    assert_eq!(item.name, "흐려진 미소");
    let form = item.form.unwrap();
    assert_eq!((form.index, form.progress, form.wins_required), (1, 0, 2));
    assert_eq!(form.appearance, "sheep-mask-clouded");
    assert!(form.description.contains("공격력"));
    let completion = core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    let BehaviorResult::CombatRewardsGranted { inventory_diff, .. } = completion else { panic!("expected victory acknowledgement: {completion:?}"); };
    let changed = inventory_diff.updated.iter().find_map(|item| match item {
        crate::game::resources::InventoryItemDto::Equipment(item) if item.uuid == copy => Some(item), _ => None
    }).expect("victory diff must publish the changed form without requiring a snapshot reload");
    assert_eq!(changed.name, "흐려진 미소");
    assert_eq!(changed.form.as_ref().unwrap().index, 1);

    let other = core.state.roster.iter().find(|e| e.uuid != employee).unwrap().uuid;
    for (from, to) in [(employee, other), (other, employee)] {
        core.execute(player, PlayerBehavior::UnEquipItem { item_uuid: copy, target_unit: from }).unwrap();
        assert_eq!(equipment_forms_world_item(&core, copy).name, "흐려진 미소");
        core.execute(player, PlayerBehavior::EquipItem { item_uuid: copy, target_unit: to }).unwrap();
        assert_eq!(core.run_resources(), Some(&confirmed));
        assert_eq!(equipment_forms_world_item(&core, copy).form.unwrap().index, 1);
    }
    let mut next = core.clone();
    enter_resource_battle(&mut next, player);
    let actor = next.state.active_battle.as_ref().unwrap().battle.units.values().find(|u| u.source_owned_uuid == employee && u.is_active()).unwrap();
    assert_eq!((actor.stats.max_health, actor.stats.attack), (1080, base_attack + 10));
    assert_eq!(equipment_forms_world_pending(&next, copy), 0);
    let node = force_first_available_node(&mut core, MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint);
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let source = crate::application::CoreApplication::from_core(core);
    let checkpoint = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = crate::application::CoreApplication::new(data, 0xb052);
    install_application_profile_for_continue(&mut restored, &source);
    restored.continue_from_active_run_checkpoint(player, &checkpoint).unwrap();
    assert_eq!(restored.run_resources(), Some(&confirmed));
    let command = crate::application::ApplicationCommand::from_document(crate::application::ApplicationDocument::new(
        serde_json::to_value(PlayerBehavior::UnEquipItem { item_uuid: copy, target_unit: employee }).unwrap())).unwrap();
    restored.execute(player, command, Default::default()).unwrap();
    assert_eq!(restored.run_resources(), Some(&confirmed));
}
