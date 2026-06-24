use super::*;

#[test]
fn use_consumable_item_is_safezone_action_and_exposes_active_modifier_snapshot() {
    let consumable = consumable_meta(
        0xC001,
        "stabilizing_ampoule",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 25 },
    );
    let owned_uuid = Uuid::from_u128(0xC0FFEE);
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    core.inventory_mut()
        .unwrap()
        .consumables
        .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_rest",
        MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::UseConsumableItem));

    let result = core
        .execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: owned_uuid,
                target_employee_uuid: employee_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::ConsumableItemUsed {
        item_uuid,
        target_employee_uuid,
        replaced_modifier,
        applied_modifier,
        inventory_diff,
    } = result
    else {
        panic!("expected consumable use result");
    };
    assert_eq!(item_uuid, owned_uuid);
    assert_eq!(target_employee_uuid, employee_uuid);
    assert!(replaced_modifier.is_none());
    assert_eq!(applied_modifier.definition_id, "stabilizing_ampoule");
    assert_eq!(inventory_diff.removed, vec![owned_uuid]);
    assert!(core
        .inventory()
        .unwrap()
        .consumables
        .get_item(&owned_uuid)
        .is_none());

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert!(snapshot["inventory"]["consumables"]
        .as_array()
        .unwrap()
        .is_empty());
    let employees = snapshot["roster"]["employees"].as_array().unwrap();
    let employee = employees
        .iter()
        .find(|value| value["uuid"] == json!(employee_uuid))
        .unwrap();
    assert_eq!(
        employee["active_consumable_modifier"]["definition_id"],
        "stabilizing_ampoule"
    );
    assert_eq!(
        employee["active_consumable_modifier"]["remaining_combat_nodes"],
        1
    );
}

#[test]
fn use_consumable_item_allows_alive_but_combat_unavailable_target() {
    let consumable = consumable_meta(
        0xC020,
        "field_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 15 },
    );
    let owned_uuid = Uuid::from_u128(0xC020_0001);
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.availability = EmployeeAvailability::Unavailable;
    }
    core.inventory_mut()
        .unwrap()
        .consumables
        .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_rest",
        MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

    let result = core.execute(
        player_id,
        PlayerBehavior::UseConsumableItem {
            item_uuid: owned_uuid,
            target_employee_uuid: employee_uuid,
        },
    );

    assert!(matches!(
        result,
        Ok(BehaviorResult::ConsumableItemUsed { .. })
    ));
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .active_consumable_modifier
            .as_ref()
            .unwrap()
            .definition_id,
        "field_tonic"
    );
}

#[test]
fn use_consumable_item_rejects_dead_target() {
    let consumable = consumable_meta(
        0xC021,
        "dead_target_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 15 },
    );
    let owned_uuid = Uuid::from_u128(0xC021_0001);
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.life_state = EmployeeLifeState::Dead;
    }
    core.inventory_mut()
        .unwrap()
        .consumables
        .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_rest",
        MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

    let result = core.execute(
        player_id,
        PlayerBehavior::UseConsumableItem {
            item_uuid: owned_uuid,
            target_employee_uuid: employee_uuid,
        },
    );

    assert!(matches!(result, Err(GameError::InvalidAction)));
    assert!(core
        .inventory()
        .unwrap()
        .consumables
        .get_item(&owned_uuid)
        .is_some());
}

#[test]
fn use_consumable_item_replaces_existing_modifier_without_refund() {
    let first = consumable_meta(
        0xC010,
        "first_ampoule",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 10 },
    );
    let second = consumable_meta(
        0xC011,
        "second_ampoule",
        ConsumableTier::Uncommon,
        ConsumableEffect::BattleHpSetup { bonus_percent: 20 },
    );
    let first_owned = Uuid::from_u128(0xC010_0001);
    let second_owned = Uuid::from_u128(0xC011_0001);
    let game_data = game_data_with_consumables(vec![first.clone(), second.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .consumables
            .add_item(OwnedConsumable::new(first_owned, Arc::new(first)))
            .unwrap();
        inventory
            .consumables
            .add_item(OwnedConsumable::new(second_owned, Arc::new(second)))
            .unwrap();
    }

    core.execute(
        player_id,
        PlayerBehavior::UseConsumableItem {
            item_uuid: first_owned,
            target_employee_uuid: employee_uuid,
        },
    )
    .unwrap();
    let result = core
        .execute(
            player_id,
            PlayerBehavior::UseConsumableItem {
                item_uuid: second_owned,
                target_employee_uuid: employee_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::ConsumableItemUsed {
        replaced_modifier,
        inventory_diff,
        ..
    } = result
    else {
        panic!("expected consumable use result");
    };
    assert_eq!(replaced_modifier.unwrap().definition_id, "first_ampoule");
    assert_eq!(inventory_diff.removed, vec![second_owned]);
    assert!(core
        .inventory()
        .unwrap()
        .consumables
        .get_item(&first_owned)
        .is_none());
    assert!(core
        .inventory()
        .unwrap()
        .consumables
        .get_item(&second_owned)
        .is_none());
}

#[test]
fn equip_item_targets_employee_loadout_after_roster_initialization() {
    let unit_meta = abnormality_meta(1);
    let weapon = equipment_meta(10, "employee_weapon", EquipmentType::Weapon);
    let weapon_owned_uuid = Uuid::from_u128(200);
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");

    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(
                weapon_owned_uuid,
                Arc::new(weapon.clone()),
            ))
            .unwrap();
    }

    let result = core
        .handle_equip_item(weapon_owned_uuid, employee_uuid)
        .unwrap();

    let BehaviorResult::EquipItem { result } = result else {
        panic!("expected equip item result");
    };
    assert_eq!(result.target_unit, employee_uuid);
    assert!(result
        .equipped_items
        .iter()
        .any(|item| item.base_uuid == weapon.uuid));

    let roster = core.roster().unwrap();
    let employee = roster.get(&employee_uuid).unwrap();
    let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
    assert_eq!(equipped.len(), 2);
    assert!(equipped
        .iter()
        .any(|item| item.instance_uuid == weapon_owned_uuid));

    let inventory = core.inventory().unwrap();
    let equipped_item = inventory.equipments.get_item(&weapon_owned_uuid).unwrap();
    assert_eq!(equipped_item.equipped_to, Some(employee_uuid));

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employees = snapshot["employees"].as_array().unwrap();
    let employee_snapshot = employees
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_uuid))
        .expect("equipped employee is exposed in roster snapshot");
    assert_eq!(
        employee_snapshot["combat_profile"]["effective_weapon_profile"]["weapon_archetype"],
        "Sword"
    );
}

#[test]
fn unequip_item_allows_unbound_equipment_in_safezone() {
    let unit_meta = abnormality_meta(1);
    let weapon = equipment_meta(10, "employee_weapon", EquipmentType::Weapon);
    let weapon_owned_uuid = Uuid::from_u128(200);
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let mut owned = OwnedEquipment::new(weapon_owned_uuid, Arc::new(weapon.clone()));
        owned.equipped_to = Some(employee_uuid);
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(owned)
            .unwrap();
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee
            .loadout
            .item_slot
            .equip(
                EquippedRef {
                    instance_uuid: weapon_owned_uuid,
                    base_uuid: weapon.uuid,
                    equipment_type: weapon.equipment_type,
                },
                weapon.allow_duplicate_equip,
            )
            .unwrap();
    }

    let result = core
        .execute(
            player_id,
            PlayerBehavior::UnEquipItem {
                item_uuid: weapon_owned_uuid,
                target_unit: employee_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::UnEquipItem { result } = result else {
        panic!("expected unequip item result");
    };
    assert_eq!(result.item_uuid, weapon_owned_uuid);
    assert_eq!(result.equipped_items.len(), 1);
    assert_eq!(
        result.equipped_items[0].base_uuid,
        starter_equipment_meta().uuid
    );
    let inventory = core.inventory().unwrap();
    let equipment = inventory.equipments.get_item(&weapon_owned_uuid).unwrap();
    assert_eq!(equipment.equipped_to, None);
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
    assert_eq!(equipped.len(), 1);
    assert_eq!(equipped[0].base_uuid, starter_equipment_meta().uuid);
}

#[test]
fn unequip_item_rejects_bound_equipment_and_snapshot_exposes_reason() {
    let unit_meta = abnormality_meta(1);
    let mut weapon = equipment_meta(10, "bound_weapon", EquipmentType::Weapon);
    weapon.bound = true;
    weapon.cannot_unequip_reason = "story_bound".to_string();
    let weapon_owned_uuid = Uuid::from_u128(201);
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()], vec![]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let mut owned = OwnedEquipment::new(weapon_owned_uuid, Arc::new(weapon.clone()));
        owned.equipped_to = Some(employee_uuid);
        core.inventory_mut()
            .unwrap()
            .equipments
            .add_item(owned)
            .unwrap();
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee
            .loadout
            .item_slot
            .equip(
                EquippedRef {
                    instance_uuid: weapon_owned_uuid,
                    base_uuid: weapon.uuid,
                    equipment_type: weapon.equipment_type,
                },
                weapon.allow_duplicate_equip,
            )
            .unwrap();
    }

    let err = core
        .execute(
            player_id,
            PlayerBehavior::UnEquipItem {
                item_uuid: weapon_owned_uuid,
                target_unit: employee_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["inventory"]["equipments"][0]["item"]["can_unequip"],
        false
    );
    assert_eq!(
        snapshot["inventory"]["equipments"][0]["item"]["cannot_unequip_reason"],
        "story_bound"
    );
    let employees = snapshot["roster"]["employees"].as_array().unwrap();
    let employee = employees
        .iter()
        .find(|value| value["uuid"] == json!(employee_uuid))
        .unwrap();
    assert_eq!(employee["equipped_items"][0]["can_unequip"], false);
    assert_eq!(
        employee["equipped_items"][0]["cannot_unequip_reason"],
        "story_bound"
    );
}

#[test]
fn bound_equipment_cannot_be_dismantled_or_enhanced_in_maintenance() {
    let unit_meta = abnormality_meta(1);
    let mut equipment = equipment_meta(30, "bound_maintenance_weapon", EquipmentType::Weapon);
    equipment.bound = true;
    equipment.cannot_unequip_reason = "story_bound".to_string();
    let material = EquipmentMaterialMetadata {
        id: "bound_maintenance_fragment".to_string(),
        uuid: Uuid::from_u128(31),
        name: "Bound Maintenance Fragment".to_string(),
        description: "A test material for bound equipment operations".to_string(),
        material_type: EquipmentMaterialType::Fragment,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        equipment_type: Some(EquipmentType::Weapon),
    };
    let dismantle_recipe = EquipmentDismantleRecipeMetadata {
        equipment_id: equipment.id.clone(),
        yields: vec![EquipmentMaterialCost {
            material_id: material.id.clone(),
            amount: 1,
        }],
    };
    let enhancement_recipe = EquipmentEnhancementRecipeMetadata {
        equipment_id: equipment.id.clone(),
        max_level: 2,
        costs_per_level: vec![EquipmentMaterialCost {
            material_id: material.id.clone(),
            amount: 1,
        }],
        modifiers_per_level: vec![StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 1,
        }],
    };
    let game_data = test_game_data_builder()
        .with_abnormalities(vec![(*unit_meta).clone()])
        .with_equipment_data(Arc::new(EquipmentDatabase::with_all(
            equipment_with_starter_assets(vec![equipment.clone()]),
            vec![material.clone()],
            vec![],
            vec![dismantle_recipe],
            vec![enhancement_recipe],
        )))
        .build_arc();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let owned_uuid = Uuid::from_u128(301);
    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
            .unwrap();
        inventory.equipment_materials.add(&material.id, 1).unwrap();
    }

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::MaintenanceState {
        maintenance_options,
        ..
    } = entered
    else {
        panic!("expected maintenance state with options");
    };
    let preview = maintenance_options
        .items
        .iter()
        .find(|item| item.target_id == owned_uuid.to_string())
        .expect("bound equipment should appear in maintenance preview");
    assert!(!preview.operations.dismantle.can_execute);
    assert!(!preview.operations.enhance.can_execute);

    let dismantle_err = core
        .execute(
            player_id,
            PlayerBehavior::DismantleEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(dismantle_err, GameError::InvalidAction));
    let enhance_err = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(enhance_err, GameError::InvalidAction));
    let owned = core
        .inventory()
        .unwrap()
        .equipments
        .get_item(&owned_uuid)
        .expect("bound equipment remains owned");
    assert_eq!(owned.enhancement_level, 0);
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipment_materials
            .amount(&material.id),
        1
    );
}

#[test]
fn equip_item_combines_equipped_component_before_slot_rejection() {
    let unit_meta = abnormality_meta(1);
    let component_a = equipment_meta(10, "weapon_component_a", EquipmentType::Weapon);
    let component_b = equipment_meta(11, "weapon_component_b", EquipmentType::Weapon);
    let completed = equipment_meta(12, "completed_weapon", EquipmentType::Weapon);
    let game_data = game_data_with_equipment(
        Arc::clone(&unit_meta),
        vec![component_a.clone(), component_b.clone(), completed.clone()],
        vec![EquipmentRecipeMetadata {
            ingredients: vec![component_a.uuid, component_b.uuid],
            result: completed.uuid,
        }],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    let component_a_owned_uuid = Uuid::from_u128(200);
    let component_b_owned_uuid = Uuid::from_u128(201);

    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(
                component_a_owned_uuid,
                Arc::new(component_a),
            ))
            .unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(
                component_b_owned_uuid,
                Arc::new(component_b),
            ))
            .unwrap();
    }

    core.handle_equip_item(component_a_owned_uuid, employee_uuid)
        .unwrap();
    let result = core
        .handle_equip_item(component_b_owned_uuid, employee_uuid)
        .unwrap();

    let BehaviorResult::EquipItem { result } = result else {
        panic!("expected equip item result");
    };
    assert_eq!(result.requested_item_uuid, component_b_owned_uuid);
    assert_eq!(result.target_unit, employee_uuid);
    assert_eq!(result.inventory_diff.removed.len(), 2);
    assert!(result
        .inventory_diff
        .removed
        .contains(&component_a_owned_uuid));
    assert!(result
        .inventory_diff
        .removed
        .contains(&component_b_owned_uuid));
    assert_eq!(result.inventory_diff.added.len(), 1);
    assert!(matches!(
        result.outcome,
        EquipItemOutcomeDto::Combined {
            result_base_uuid,
            ..
        } if result_base_uuid == completed.uuid
    ));
    assert!(result
        .equipped_items
        .iter()
        .any(|item| item.base_uuid == completed.uuid));

    let inventory = core.inventory().unwrap();
    assert!(inventory
        .equipments
        .get_item(&component_a_owned_uuid)
        .is_none());
    assert!(inventory
        .equipments
        .get_item(&component_b_owned_uuid)
        .is_none());

    let roster = core.roster().unwrap();
    let employee = roster.get(&employee_uuid).unwrap();
    let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
    assert_eq!(equipped.len(), 2);
    assert!(equipped.iter().any(|item| item.base_uuid == completed.uuid));

    let completed_item = inventory
        .equipments
        .iter()
        .find(|item| item.meta.uuid == completed.uuid)
        .expect("completed item should be created");
    assert_eq!(completed_item.equipped_to, Some(employee_uuid));
}

#[test]
fn dismantle_equipment_is_maintenance_only_and_grants_material_stacks() {
    let unit_meta = abnormality_meta(1);
    let equipment = equipment_meta(40, "dismantle_weapon", EquipmentType::Weapon);
    let material = EquipmentMaterialMetadata {
        id: "dismantled_weapon_fragment".to_string(),
        uuid: Uuid::from_u128(41),
        name: "Dismantled Weapon Fragment".to_string(),
        description: "A test dismantle material".to_string(),
        material_type: EquipmentMaterialType::Fragment,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        equipment_type: Some(EquipmentType::Weapon),
    };
    let recipe = EquipmentDismantleRecipeMetadata {
        equipment_id: equipment.id.clone(),
        yields: vec![EquipmentMaterialCost {
            material_id: material.id.clone(),
            amount: 2,
        }],
    };
    let game_data = game_data_with_equipment_dismantle(
        Arc::clone(&unit_meta),
        vec![equipment.clone()],
        vec![material.clone()],
        vec![recipe],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let owned_uuid = Uuid::from_u128(401);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
        .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::DismantleEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::DismantleEquipment));

    let result = core
        .execute(
            player_id,
            PlayerBehavior::DismantleEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::EquipmentDismantled {
        item_uuid,
        equipment_id,
        inventory_diff,
    } = result
    else {
        panic!("expected equipment dismantle result");
    };
    assert_eq!(item_uuid, owned_uuid);
    assert_eq!(equipment_id, equipment.id);
    assert!(inventory_diff.added.is_empty());
    assert_eq!(inventory_diff.removed, vec![owned_uuid]);
    assert_eq!(inventory_diff.material_stacks.len(), 1);
    assert_eq!(inventory_diff.material_stacks[0].material_id, material.id);
    assert_eq!(inventory_diff.material_stacks[0].amount, 2);
    assert!(core
        .inventory()
        .unwrap()
        .equipments
        .get_item(&owned_uuid)
        .is_none());
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipment_materials
            .amount(&material.id),
        2
    );
}

#[test]
fn dismantle_equipment_automatically_unequips_equipped_item() {
    let unit_meta = abnormality_meta(1);
    let equipment = equipment_meta(45, "equipped_dismantle_weapon", EquipmentType::Weapon);
    let material = EquipmentMaterialMetadata {
        id: "equipment_dust".to_string(),
        uuid: Uuid::from_u128(46),
        name: "Equipment Dust".to_string(),
        description: "A test equipment dust material".to_string(),
        material_type: EquipmentMaterialType::Generic,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        equipment_type: None,
    };
    let recipe = EquipmentDismantleRecipeMetadata {
        equipment_id: equipment.id.clone(),
        yields: vec![EquipmentMaterialCost {
            material_id: material.id.clone(),
            amount: 1,
        }],
    };
    let game_data = game_data_with_equipment_dismantle(
        Arc::clone(&unit_meta),
        vec![equipment.clone()],
        vec![material.clone()],
        vec![recipe],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let owned_uuid = Uuid::from_u128(451);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipItem {
            item_uuid: owned_uuid,
            target_unit: employee_uuid,
        },
    )
    .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::MaintenanceState {
        maintenance_options,
        ..
    } = entered
    else {
        panic!("expected maintenance state with options");
    };
    let preview = maintenance_options
        .items
        .iter()
        .find(|item| item.target_id == owned_uuid.to_string())
        .expect("equipped equipment should appear in maintenance preview");
    assert!(preview.operations.dismantle.can_execute);
    assert!(preview.operations.dismantle.will_unequip);

    core.execute(
        player_id,
        PlayerBehavior::DismantleEquipment {
            item_uuid: owned_uuid,
        },
    )
    .unwrap();

    assert!(core
        .inventory()
        .unwrap()
        .equipments
        .get_item(&owned_uuid)
        .is_none());
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert!(!employee.loadout.item_slot.contains_instance(owned_uuid));
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipment_materials
            .amount(&material.id),
        1
    );
}

#[test]
fn enhance_equipment_is_maintenance_only_and_updates_owned_instance() {
    let unit_meta = abnormality_meta(1);
    let equipment = equipment_meta(50, "enhance_weapon", EquipmentType::Weapon);
    let material = EquipmentMaterialMetadata {
        id: "enhance_weapon_fragment".to_string(),
        uuid: Uuid::from_u128(51),
        name: "Enhance Weapon Fragment".to_string(),
        description: "A test enhancement material".to_string(),
        material_type: EquipmentMaterialType::Fragment,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        equipment_type: Some(EquipmentType::Weapon),
    };
    let recipe = EquipmentEnhancementRecipeMetadata {
        equipment_id: equipment.id.clone(),
        max_level: 2,
        costs_per_level: vec![EquipmentMaterialCost {
            material_id: material.id.clone(),
            amount: 2,
        }],
        modifiers_per_level: vec![StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 3,
        }],
    };
    let game_data = game_data_with_equipment_enhancement(
        Arc::clone(&unit_meta),
        vec![equipment.clone()],
        vec![material.clone()],
        vec![recipe],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let owned_uuid = Uuid::from_u128(501);
    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment.clone())))
            .unwrap();
        inventory.equipment_materials.add(&material.id, 2).unwrap();
    }

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::MaintenanceState {
        maintenance_options,
        ..
    } = entered
    else {
        panic!("expected maintenance state with options");
    };
    let maintenance_item = maintenance_options
        .items
        .iter()
        .find(|item| item.target_id == owned_uuid.to_string())
        .expect("equipment should be present in maintenance preview");
    assert!(maintenance_item.operations.enhance.can_execute);
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::EnhanceEquipment));

    let result = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::EquipmentEnhanced {
        item_uuid,
        equipment_id,
        enhancement_level,
        inventory_diff,
    } = result
    else {
        panic!("expected equipment enhanced result");
    };
    assert_eq!(item_uuid, owned_uuid);
    assert_eq!(equipment_id, equipment.id);
    assert_eq!(enhancement_level, 1);
    assert_eq!(inventory_diff.updated.len(), 1);
    assert_eq!(inventory_diff.material_stacks[0].amount, 0);
    let owned = core
        .inventory()
        .unwrap()
        .equipments
        .get_item(&owned_uuid)
        .unwrap();
    assert_eq!(owned.enhancement_level, 1);
}

#[test]
fn skill_fragment_actions_equip_and_unequip_employee_loadout() {
    let fragment = active_skill_fragment("test_active_fragment", 0xF00D, "test_active_skill");
    let weapon = weapon_equipment(0xE001, "starter_test_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![weapon.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE001_0001),
    );
    core.state.skill_fragments.add(&fragment).unwrap();

    let equip_result = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

    let BehaviorResult::SkillFragmentLoadoutUpdated {
        employee_uuid: updated_employee,
        equipped_fragment_ids,
    } = equip_result
    else {
        panic!("expected skill fragment loadout result");
    };
    assert_eq!(updated_employee, employee_uuid);
    assert!(equipped_fragment_ids.iter().any(|id| id == &fragment.id));

    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    let profile = employee
        .combat_profile_for_battle(
            &core.game_data.skill_fragment_data,
            &core.state.skill_fragments,
        )
        .unwrap();
    assert_eq!(profile.skill_id.as_deref(), Some("test_active_skill"));

    let unequip_result = core
        .execute(
            player_id,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();
    let BehaviorResult::SkillFragmentLoadoutUpdated {
        equipped_fragment_ids,
        ..
    } = unequip_result
    else {
        panic!("expected skill fragment loadout result");
    };
    assert!(!equipped_fragment_ids.iter().any(|id| id == &fragment.id));
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    let profile = employee
        .combat_profile_for_battle(
            &core.game_data.skill_fragment_data,
            &core.state.skill_fragments,
        )
        .unwrap();
    assert_eq!(profile.skill_id, None);
}

#[test]
fn skill_fragment_equip_rejects_unowned_fragment() {
    let fragment = active_skill_fragment("unowned_active_fragment", 0xF00E, "unowned_skill");
    let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id,
            },
        )
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn skill_fragment_equip_requires_one_owned_copy_per_employee() {
    let fragment = active_skill_fragment("copy_limited_fragment", 0xF020, "copy_limited_skill");
    let weapon = weapon_equipment(0xE020, "copy_limit_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![weapon.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    grant_and_equip_weapon(
        &mut core,
        employee_ids[0],
        weapon.clone(),
        Uuid::from_u128(0xE020_0001),
    );
    grant_and_equip_weapon(
        &mut core,
        employee_ids[1],
        weapon,
        Uuid::from_u128(0xE020_0002),
    );
    core.state.skill_fragments.add(&fragment).unwrap();

    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[0],
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee_ids[1],
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[1],
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();
}

#[test]
fn global_exclusive_skill_fragment_rejects_second_employee_even_with_extra_copy() {
    let mut fragment = active_skill_fragment(
        "global_exclusive_fragment",
        0xF021,
        "global_exclusive_skill",
    );
    fragment.equip_limit = SkillFragmentEquipLimit::GlobalExclusive;
    let weapon = weapon_equipment(0xE021, "global_exclusive_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![weapon.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    grant_and_equip_weapon(
        &mut core,
        employee_ids[0],
        weapon.clone(),
        Uuid::from_u128(0xE021_0001),
    );
    grant_and_equip_weapon(
        &mut core,
        employee_ids[1],
        weapon,
        Uuid::from_u128(0xE021_0002),
    );
    core.state.skill_fragments.add(&fragment).unwrap();
    core.state.skill_fragments.add(&fragment).unwrap();

    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[0],
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee_ids[1],
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn skill_fragment_equip_rejects_incompatible_weapon_profile() {
    let mut fragment = active_skill_fragment("gun_locked_fragment", 0xF013, "gun_locked_skill");
    fragment.compatibility = SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
        ..SkillFragmentCompatibilityRequirements::default()
    };
    let sword = weapon_equipment(0xE002, "test_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![sword.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        sword,
        Uuid::from_u128(0xE002_0001),
    );
    core.state.skill_fragments.add(&fragment).unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap_err();

    assert!(matches!(
        err,
        GameError::SkillFragmentIncompatible {
            fragment_id,
            failure_codes,
        } if fragment_id == fragment.id
            && failure_codes == vec![
                SkillFragmentCompatibilityFailureCode::WeaponArchetypeMismatch
            ]
    ));

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee_snapshot = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_uuid))
        .unwrap();
    let compatibility = employee_snapshot["skill_fragments"]["compatibility"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == json!(fragment.id))
        .unwrap();
    assert_eq!(compatibility["is_compatible"], false);
    assert_eq!(
        compatibility["failure_codes"],
        json!(["weapon_archetype_mismatch"])
    );
}

#[test]
fn skill_fragment_equip_accepts_matching_weapon_profile() {
    let mut fragment = active_skill_fragment("matching_gun_fragment", 0xF014, "matching_gun_skill");
    fragment.compatibility = SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
        allowed_range_roles: vec![WeaponRangeRole::Ranged],
        ..SkillFragmentCompatibilityRequirements::default()
    };
    let gun = weapon_equipment(0xE003, "test_gun", WeaponArchetype::Gun);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![gun.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(&mut core, employee_uuid, gun, Uuid::from_u128(0xE003_0001));
    core.state.skill_fragments.add(&fragment).unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentLoadoutUpdated { .. }
    ));
}

#[test]
fn equipment_combination_rejects_result_that_invalidates_active_fragment() {
    let mut fragment =
        active_skill_fragment("gun_combo_locked_fragment", 0xF015, "gun_combo_skill");
    fragment.compatibility = SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
        ..SkillFragmentCompatibilityRequirements::default()
    };
    let gun_component = weapon_equipment(0xE008, "gun_component", WeaponArchetype::Gun);
    let catalyst = weapon_equipment(0xE009, "weapon_catalyst", WeaponArchetype::Gun);
    let sword_result = weapon_equipment(0xE00A, "sword_result", WeaponArchetype::Sword);
    let game_data = game_data_with_equipment_recipes_and_skill_fragments(
        vec![
            gun_component.clone(),
            catalyst.clone(),
            sword_result.clone(),
        ],
        vec![EquipmentRecipeMetadata {
            ingredients: vec![gun_component.uuid, catalyst.uuid],
            result: sword_result.uuid,
        }],
        vec![fragment.clone()],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        gun_component,
        Uuid::from_u128(0xE008_0001),
    );
    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();
    let catalyst_owned_uuid = Uuid::from_u128(0xE009_0001);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(catalyst_owned_uuid, Arc::new(catalyst)))
        .unwrap();

    let err = core
        .handle_equip_item(catalyst_owned_uuid, employee_uuid)
        .unwrap_err();

    assert!(matches!(
        err,
        GameError::SkillFragmentIncompatible {
            fragment_id,
            failure_codes,
        } if fragment_id == fragment.id
            && failure_codes == vec![
                SkillFragmentCompatibilityFailureCode::WeaponArchetypeMismatch
            ]
    ));
}

#[test]
fn skill_fragment_upgrade_consumes_fragment_dust_and_updates_progress() {
    let target = active_skill_fragment("upgrade_active_fragment", 0xF00F, "upgrade_skill");
    let game_data = game_data_with_skill_fragments(vec![target.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.state.skill_fragments.add(&target).unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::UpgradeSkillFragment {
                target_fragment_id: target.id.clone(),
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::UpgradeSkillFragment));
    core.state.skill_fragments.add_fragment_dust(4).unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::UpgradeSkillFragment {
                target_fragment_id: target.id.clone(),
            },
        )
        .unwrap();

    let BehaviorResult::SkillFragmentUpgraded {
        target_fragment_id,
        dust_spent,
        remaining_dust,
        progress,
    } = result
    else {
        panic!("expected skill fragment upgrade result");
    };
    assert_eq!(target_fragment_id, target.id);
    assert_eq!(dust_spent, 4);
    assert_eq!(remaining_dust, 0);
    assert_eq!(progress.upgrade_level, 1);
    assert_eq!(progress.awakening_progress, 1);
    assert_eq!(core.state.skill_fragments.count(&target.id), 1);
    assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
}

#[test]
fn skill_fragment_dismantle_is_maintenance_only_and_grants_dust() {
    let fragment = active_skill_fragment("dismantle_fragment", 0xF011, "dismantle_skill");
    let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.state.skill_fragments.add(&fragment).unwrap();
    core.state.skill_fragments.add(&fragment).unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::DismantleSkillFragment));

    let result = core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

    let BehaviorResult::SkillFragmentDismantled {
        fragment_id,
        remaining_count,
        dust_gained,
        total_dust,
    } = result
    else {
        panic!("expected skill fragment dismantle result");
    };
    assert_eq!(fragment_id, fragment.id);
    assert_eq!(remaining_count, 1);
    assert_eq!(dust_gained, 2);
    assert_eq!(total_dust, 2);
    assert_eq!(core.state.skill_fragments.count(&fragment.id), 1);
    assert_eq!(core.state.skill_fragments.fragment_dust(), 2);
}

#[test]
fn skill_fragment_dismantle_keeps_equipped_fragment_when_remaining_copy_allows_it() {
    let fragment = active_skill_fragment("equipped_dismantle_fragment", 0xF012, "equipped_skill");
    let weapon = weapon_equipment(0xE004, "dismantle_test_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![weapon.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE004_0001),
    );
    core.state.skill_fragments.add(&fragment).unwrap();
    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentDismantled {
            remaining_count: 1,
            ..
        }
    ));
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(
        employee.skill_fragments.active_fragment_id(),
        Some(&fragment.id)
    );
    assert_eq!(core.state.skill_fragments.fragment_dust(), 2);
}

#[test]
fn skill_fragment_dismantle_allows_last_copy() {
    let fragment = active_skill_fragment("last_copy_dismantle_fragment", 0xF013, "last_copy_skill");
    let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.state.skill_fragments.add(&fragment).unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                fragment_id: fragment.id.clone(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentDismantled {
            remaining_count: 0,
            ..
        }
    ));
    assert_eq!(core.state.skill_fragments.count(&fragment.id), 0);
}
