fn cohort_bounty_world_fixture() -> ExternalFixture {
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
    accessory.id = "cohort_bounty_fixture".into();
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
        .push("cohort_bounty_fixture".into());
    employee.profile.base_combat_stats.max_hp = 1000;
    fixture.write(path, ron::ser::to_string(&employees).unwrap());
    let source =
        include_str!("../../../../tests/fixtures/skill_composition/cohort_bounty.skill.yaml")
            .replace(
                "equipment: r11_accessory",
                "equipment: cohort_bounty_fixture",
            )
            .replace("hp: 300", "hp: 1")
            .replace("power: 20", "power: 1");
    let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
    let path = "data/skills/cohort-bounty.skill-composition.ron";
    fixture.write(&format!("official/{path}"), artifact.to_ron().unwrap());
    let mut manifest: PackManifest = ron::de::from_str(&fixture.read("official/pack.ron")).unwrap();
    manifest.files.push(CatalogFile {
        kind: CatalogKind::Skills,
        path: path.into(),
    });
    fixture.manifest("official", manifest);
    fixture
}
fn cohort_bounty_world_pending(core: &GameCore, copy: Uuid) -> u32 {
    core.battle_typed_resources()
        .unwrap()
        .entries()
        .find(|(key, _)| {
            key.resource.as_str() == "bounty_pending"
                && key.owner == ResourceOwner::EquipmentCopy(copy)
        })
        .map_or(0, |(_, value)| value.balance)
}
#[test]
fn composition_cohort_bounty_world_real_kills_retreat_settlement_copy_transfer_and_restore() {
    let fixture = cohort_bounty_world_fixture();
    let data = fixture.load();
    let (mut core, player, employee, _) = start_resource_battle(data.clone());
    let copy = core
        .state
        .inventory
        .equipments
        .iter()
        .find(|i| i.meta.uuid == Uuid::from_u128(0xe500) && i.equipped_to == Some(employee))
        .unwrap()
        .instance_uuid;
    let entry = core.run_resources().unwrap().clone();
    assert!(entry.entries.is_empty());
    let base_attack = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units
        .values()
        .find(|u| u.source_owned_uuid == employee && u.is_active())
        .unwrap()
        .stats
        .attack;
    for _ in 0..290 {
        core.advance_active_battle_by(100).unwrap();
        assert!(
            core.state.active_battle.is_some(),
            "actual kill must be reached before terminal"
        );
        if cohort_bounty_world_pending(&core, copy) > 0 {
            break;
        }
    }
    assert!(
        cohort_bounty_world_pending(&core, copy) > 0,
        "ordinary queued attacks must kill a real cohort"
    );
    assert_eq!(core.run_resources(), Some(&entry));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(
        active
            .battle
            .units
            .values()
            .find(|u| u.source_owned_uuid == employee && u.is_active())
            .unwrap()
            .stats
            .attack,
        base_attack
    );
    let mut retreat = core.clone();
    execute_current_battle_mutation(&mut retreat, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(retreat.run_resources(), Some(&entry));
    assert!(retreat.battle_typed_resources().is_none());
    retreat
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert_eq!(cohort_bounty_world_pending(&retreat, copy), 0);
    for _ in 0..300 {
        if core.state.active_battle.is_none() {
            break;
        }
        core.advance_active_battle_by(100).unwrap();
    }
    assert!(core.state.active_battle.is_none());
    let confirmed = core.run_resources().unwrap().clone();
    let bounty = confirmed
        .entries
        .iter()
        .find(|e| e.resource.as_str() == "bounty_confirmed")
        .unwrap();
    assert_eq!(
        (bounty.owner, bounty.balance),
        (ResourceOwner::EquipmentCopy(copy), 2)
    );
    assert!(confirmed
        .entries
        .iter()
        .all(|e| e.resource.as_str() != "bounty_pending"));
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(core.run_resources(), Some(&confirmed));
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
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
    assert_eq!(core.run_resources(), Some(&confirmed));
    assert_eq!(
        core.state
            .inventory
            .equipments
            .get_item(&copy)
            .unwrap()
            .equipped_to,
        Some(other)
    );
    // Return it to the original wearer, then consume the confirmed tier in a new battle.
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
    let mut next = core.clone();
    enter_resource_battle(&mut next, player);
    let actual = next
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units
        .values()
        .find(|u| u.source_owned_uuid == employee && u.is_active())
        .unwrap()
        .stats
        .attack;
    assert_eq!(actual, base_attack + 10);
    assert_eq!(cohort_bounty_world_pending(&next, copy), 0);
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
    let mut restored = crate::application::CoreApplication::new(data, 0xb051);
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
    assert_eq!(
        restored.run_resources(),
        Some(&confirmed),
        "fresh application continues after restore without losing the copy balance"
    );
}
