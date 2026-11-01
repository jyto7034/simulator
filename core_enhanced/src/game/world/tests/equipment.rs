use crate::game::behavior::ActionDisabledReasonCode;
use crate::game::data::{
    equipment_data::{EquipmentMaterialCost, EquipmentMaterialType},
    skill_fragment_data::{
        SkillFragmentCompatibilityFailureCode, SkillFragmentCompatibilityRequirements,
    },
};
use crate::game::resources::InventoryItemDto;

use super::*;

fn serialized_equipment_ledgers(core: &GameCore) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::to_value(&core.state.roster).expect("roster should serialize"),
        serde_json::to_value(&core.state.inventory).expect("inventory should serialize"),
    )
}

fn assert_equipment_loadout_graph_valid(core: &GameCore) {
    assert!(
        core.state
            .inventory
            .validate_checkpoint_state(&core.state.roster)
            .is_ok(),
        "equipment reverse bindings and employee slots must remain a valid graph"
    );
}

#[test]
fn use_consumable_item_decrements_definition_stack_and_exposes_active_modifier_snapshot() {
    let consumable = consumable_meta(
        0xC001,
        "stabilizing_ampoule",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 25 },
    );
    let owned_uuid = consumable.uuid;
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 2)
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));

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
        active_modifier,
        inventory_diff,
        ..
    } = result
    else {
        panic!("expected consumable use result");
    };
    assert_eq!(item_uuid, owned_uuid);
    assert_eq!(target_employee_uuid, employee_uuid);
    assert!(replaced_modifier.is_none());
    assert_eq!(
        active_modifier.unwrap().definition_id,
        "stabilizing_ampoule"
    );
    assert!(inventory_diff.removed.is_empty());
    let InventoryItemDto::Consumable(updated_stack) = &inventory_diff.updated[0] else {
        panic!("expected updated consumable stack");
    };
    assert_eq!(updated_stack.definition_id, "stabilizing_ampoule");
    assert_eq!(updated_stack.quantity, 1);
    assert_eq!(
        core.inventory()
            .unwrap()
            .consumables
            .get_item(&owned_uuid)
            .unwrap()
            .quantity,
        1
    );

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["inventory"]["reserved_slot_count"], 50);
    assert_eq!(snapshot["inventory"]["occupied_slot_count"], 7);
    assert_eq!(snapshot["inventory"]["consumables"][0]["quantity"], 1);
    assert_eq!(
        snapshot["inventory"]["consumables"][0]["uuid"],
        json!(owned_uuid)
    );
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
    let owned_uuid = consumable.uuid;
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
    }
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 1)
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
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
fn physical_treatment_consumable_reduces_exact_integer_damage_without_creating_modifier() {
    let consumable = consumable_meta(
        0xC022,
        "field_surgery_kit",
        ConsumableTier::Common,
        ConsumableEffect::ReducePhysicalDamage { amount: 2 },
    );
    let owned_uuid = consumable.uuid;
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .condition
        .physical_damage = 5;
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 1)
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

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
        active_modifier,
        condition,
        inventory_diff,
        ..
    } = result
    else {
        panic!("expected physical treatment result");
    };

    assert_eq!(condition.physical_damage, 3);
    assert!(active_modifier.is_none());
    assert_eq!(inventory_diff.removed, vec![owned_uuid]);
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(employee.condition.physical_damage, 3);
    assert!(employee.active_consumable_modifier.is_none());
}

#[test]
fn physical_treatment_can_be_spent_on_an_undamaged_living_employee() {
    let consumable = consumable_meta(
        0xC023,
        "surplus_field_surgery_kit",
        ConsumableTier::Common,
        ConsumableEffect::ReducePhysicalDamage { amount: 2 },
    );
    let owned_uuid = consumable.uuid;
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .condition
            .physical_damage,
        0
    );
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 1)
        .unwrap();

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
        condition,
        active_modifier,
        inventory_diff,
        ..
    } = result
    else {
        panic!("expected physical treatment result");
    };
    assert_eq!(condition.physical_damage, 0);
    assert!(active_modifier.is_none());
    assert_eq!(inventory_diff.removed, vec![owned_uuid]);
    assert!(core
        .inventory()
        .unwrap()
        .consumables
        .get_item(&owned_uuid)
        .is_none());
}

#[test]
fn use_consumable_item_rejects_dead_target() {
    let consumable = consumable_meta(
        0xC021,
        "dead_target_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 15 },
    );
    let owned_uuid = consumable.uuid;
    let game_data = game_data_with_consumables(vec![consumable.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE;
    }
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 1)
        .unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
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
    let first_owned = first.uuid;
    let second_owned = second.uuid;
    let game_data = game_data_with_consumables(vec![first.clone(), second.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    {
        let inventory = core.inventory_mut().unwrap();
        inventory.add_consumable(Arc::new(first), 1).unwrap();
        inventory.add_consumable(Arc::new(second), 1).unwrap();
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
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    let displaced_starter_uuid = force_unequip_employee_weapon(&mut core, employee_uuid);

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
    assert_eq!(result.requested_item_uuid, weapon_owned_uuid);
    assert_eq!(result.target_unit, employee_uuid);
    assert!(matches!(
        result.outcome,
        EquipItemOutcomeDto::Equipped { item_uuid } if item_uuid == weapon_owned_uuid
    ));
    assert_eq!(
        result
            .inventory_diff
            .updated
            .iter()
            .map(InventoryItemDto::uuid)
            .collect::<Vec<_>>(),
        vec![weapon_owned_uuid]
    );
    assert!(result.inventory_diff.added.is_empty());
    assert!(result.inventory_diff.removed.is_empty());
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
    assert_eq!(
        inventory
            .equipments
            .get_item(&displaced_starter_uuid)
            .unwrap()
            .equipped_to,
        None
    );
    assert_equipment_loadout_graph_valid(&core);

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
    assert_eq!(
        employee_snapshot["combat_profile"]["effective_weapon_profile"]["block_capacity"],
        0
    );
}

#[test]
fn equipped_weapon_authors_block_capacity_for_every_current_archetype() {
    let unit_meta = abnormality_meta(1);
    let specifications = [
        (WeaponArchetype::Bow, 0),
        (WeaponArchetype::Gun, 0),
        (WeaponArchetype::Staff, 0),
        (WeaponArchetype::Sword, 1),
        (WeaponArchetype::Spear, 1),
        (WeaponArchetype::Shield, 3),
    ];
    let mut weapons = specifications
        .iter()
        .enumerate()
        .map(|(index, (archetype, block_capacity))| {
            let mut weapon = weapon_equipment(
                0xB10C_1000 + index as u128,
                &format!("block_capacity_{archetype:?}"),
                *archetype,
            );
            weapon
                .weapon_profile
                .as_mut()
                .expect("test weapon profile")
                .block_capacity = *block_capacity;
            weapon
        })
        .collect::<Vec<_>>();
    let replacement_armor = equipment_meta(0xB10C_2000, "replacement_armor", EquipmentType::Armor);
    let mut equipment = weapons.clone();
    equipment.push(replacement_armor.clone());
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), equipment);
    let mut core = GameCore::new(game_data, 0xB10C);
    let player_id = Uuid::from_u128(0xB10C);
    start_new_game_with_default_starters(&mut core, player_id);

    for employee_uuid in core.roster().unwrap().combat_eligible_employee_ids() {
        let weapon = core
            .roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .loadout
            .item_slot
            .iter()
            .find(|equipped| equipped.equipment_type == EquipmentType::Weapon)
            .copied()
            .expect("starter fixture must be armed");
        core.roster_mut()
            .unwrap()
            .get_mut(&employee_uuid)
            .unwrap()
            .loadout
            .item_slot
            .remove_by_instance(weapon.instance_uuid)
            .unwrap();
        core.inventory_mut()
            .unwrap()
            .equipments
            .get_item_mut(&weapon.instance_uuid)
            .unwrap()
            .equipped_to = None;
        assert!(matches!(
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                &core.game_data,
                employee_uuid,
            ),
            Err(GameError::WeaponRequired)
        ));
    }

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    for (index, (weapon, (_, expected))) in weapons.drain(..).zip(specifications.iter()).enumerate()
    {
        grant_and_equip_weapon(
            &mut core,
            employee_uuid,
            weapon,
            Uuid::from_u128(0xB10C_3000 + index as u128),
        );
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                &core.game_data,
                employee_uuid,
            )
            .unwrap();
        assert_eq!(profile.block_capacity, *expected);
        assert_eq!(
            profile
                .weapon_profile
                .as_ref()
                .expect("equipped weapon profile")
                .block_capacity,
            *expected
        );
    }

    let armor_owned_uuid = Uuid::from_u128(0xB10C_4000);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(
            armor_owned_uuid,
            Arc::new(replacement_armor),
        ))
        .unwrap();
    core.handle_equip_item(armor_owned_uuid, employee_uuid)
        .unwrap();
    let profile = crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
        core.roster().unwrap(),
        core.inventory().unwrap(),
        &core.state.skill_fragments,
        &core.game_data,
        employee_uuid,
    )
    .unwrap();
    assert_eq!(
        profile.block_capacity, 3,
        "armor replacement must not modify the equipped Shield's authored capacity"
    );

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_uuid))
        .unwrap();
    assert_eq!(
        employee["combat_profile"]["effective_weapon_profile"]["block_capacity"],
        3
    );
}

#[test]
fn unequip_item_allows_unbound_equipment_in_safezone() {
    let unit_meta = abnormality_meta(1);
    let weapon = equipment_meta(10, "employee_weapon", EquipmentType::Weapon);
    let weapon_owned_uuid = Uuid::from_u128(200);
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    force_unequip_employee_weapon(&mut core, employee_uuid);
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
    assert_eq!(result.target_unit, employee_uuid);
    assert_eq!(
        result
            .inventory_diff
            .updated
            .iter()
            .map(InventoryItemDto::uuid)
            .collect::<Vec<_>>(),
        vec![weapon_owned_uuid]
    );
    assert!(result.inventory_diff.added.is_empty());
    assert!(result.inventory_diff.removed.is_empty());
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
    assert_equipment_loadout_graph_valid(&core);
}

#[test]
fn unequip_item_rejects_bound_equipment_and_snapshot_exposes_reason() {
    let unit_meta = abnormality_meta(1);
    let mut weapon = equipment_meta(10, "bound_weapon", EquipmentType::Weapon);
    weapon.bound = true;
    weapon.cannot_unequip_reason = "story_bound".to_string();
    let weapon_owned_uuid = Uuid::from_u128(201);
    let game_data = game_data_with_equipment(Arc::clone(&unit_meta), vec![weapon.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    force_unequip_employee_weapon(&mut core, employee_uuid);
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

    let before = serialized_equipment_ledgers(&core);

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
    assert_eq!(serialized_equipment_ledgers(&core), before);
    assert_equipment_loadout_graph_valid(&core);

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
        material_type: EquipmentMaterialType::EquipmentDust,
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
    assert!(matches!(
        dismantle_err,
        GameError::ActionUnavailable {
            action: ActionKind::DismantleEquipment,
            reason: ActionDisabledReasonCode::NoEligiblePayload,
        }
    ));
    let enhance_err = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(
        enhance_err,
        GameError::ActionUnavailable {
            action: ActionKind::EnhanceEquipment,
            reason: ActionDisabledReasonCode::NoEligiblePayload,
        }
    ));
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
fn equip_item_replaces_occupied_slot_without_consuming_either_item() {
    let unit_meta = abnormality_meta(1);
    let original = equipment_meta(10, "original_weapon", EquipmentType::Weapon);
    let replacement = equipment_meta(11, "replacement_weapon", EquipmentType::Weapon);
    let game_data = game_data_with_equipment(
        Arc::clone(&unit_meta),
        vec![original.clone(), replacement.clone()],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    let original_owned_uuid = Uuid::from_u128(200);
    let replacement_owned_uuid = Uuid::from_u128(201);

    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(original_owned_uuid, Arc::new(original)))
            .unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(
                replacement_owned_uuid,
                Arc::new(replacement),
            ))
            .unwrap();
    }

    core.handle_equip_item(original_owned_uuid, employee_uuid)
        .unwrap();
    let result = core
        .handle_equip_item(replacement_owned_uuid, employee_uuid)
        .unwrap();

    let BehaviorResult::EquipItem { result } = result else {
        panic!("expected equip item result");
    };
    assert_eq!(result.requested_item_uuid, replacement_owned_uuid);
    assert_eq!(result.target_unit, employee_uuid);
    assert!(result.inventory_diff.removed.is_empty());
    assert!(result.inventory_diff.added.is_empty());
    assert_eq!(
        result
            .inventory_diff
            .updated
            .iter()
            .map(InventoryItemDto::uuid)
            .collect::<Vec<_>>(),
        vec![replacement_owned_uuid, original_owned_uuid]
    );
    let serialized = serde_json::to_value(&result).expect("equip result should serialize");
    assert_eq!(
        serialized["outcome"]["Equipped"]["item_uuid"],
        replacement_owned_uuid.to_string()
    );
    assert!(!serialized.to_string().contains("Combined"));
    assert!(matches!(
        result.outcome,
        EquipItemOutcomeDto::Equipped { item_uuid } if item_uuid == replacement_owned_uuid
    ));
    assert_eq!(result.equipped_items.len(), 2);
    assert!(result
        .equipped_items
        .iter()
        .any(|item| { item.instance_uuid == replacement_owned_uuid }));

    let inventory = core.inventory().unwrap();
    assert_eq!(
        inventory
            .equipments
            .get_item(&original_owned_uuid)
            .expect("original item remains owned")
            .equipped_to,
        None
    );
    assert_eq!(
        inventory
            .equipments
            .get_item(&replacement_owned_uuid)
            .expect("replacement item remains owned")
            .equipped_to,
        Some(employee_uuid)
    );

    let roster = core.roster().unwrap();
    let employee = roster.get(&employee_uuid).unwrap();
    let equipped = employee.loadout.item_slot.iter().collect::<Vec<_>>();
    assert_eq!(equipped.len(), 2);
    assert!(equipped
        .iter()
        .any(|item| item.instance_uuid == replacement_owned_uuid));
    assert!(!equipped
        .iter()
        .any(|item| item.instance_uuid == original_owned_uuid));
    assert_equipment_loadout_graph_valid(&core);
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
        material_type: EquipmentMaterialType::EquipmentDust,
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
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::DismantleEquipment,
            reason: ActionDisabledReasonCode::ContextIrrelevant,
        }
    ));

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
        material_type: EquipmentMaterialType::EquipmentDust,
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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

    let result = core
        .execute(
            player_id,
            PlayerBehavior::DismantleEquipmentConfirmed { confirmed: true,
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
    assert_eq!(inventory_diff.removed, vec![owned_uuid]);
    assert!(inventory_diff.updated.is_empty());
    assert!(inventory_diff.added.is_empty());

    assert!(core
        .inventory()
        .unwrap()
        .equipments
        .get_item(&owned_uuid)
        .is_none());
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert!(!employee.loadout.item_slot.contains_instance(owned_uuid));
    assert_eq!(
        core.state.crafting_resources.equipment_dust,
        1
    );
    assert_equipment_loadout_graph_valid(&core);
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
        material_type: EquipmentMaterialType::EquipmentDust,
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
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::EnhanceEquipment,
            reason: ActionDisabledReasonCode::ContextIrrelevant,
        }
    ));

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

fn duplicate_enhancement_cost_fixture(material_balance: u32) -> (GameCore, Uuid, Uuid, String) {
    let unit_meta = abnormality_meta(1);
    let equipment = equipment_meta(60, "duplicate_enhance_weapon", EquipmentType::Weapon);
    let material = EquipmentMaterialMetadata {
        id: "duplicate_enhance_dust".to_string(),
        uuid: Uuid::from_u128(61),
        name: "Duplicate Enhance Dust".to_string(),
        description: "Aggregate enhancement fixture".to_string(),
        material_type: EquipmentMaterialType::EquipmentDust,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        equipment_type: Some(EquipmentType::Weapon),
    };
    let recipe = EquipmentEnhancementRecipeMetadata {
        equipment_id: equipment.id.clone(),
        max_level: 2,
        costs_per_level: vec![
            EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 2,
            },
            EquipmentMaterialCost {
                material_id: material.id.clone(),
                amount: 2,
            },
        ],
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
    let owned_uuid = Uuid::from_u128(601);
    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(owned_uuid, Arc::new(equipment)))
            .unwrap();
        inventory
            .equipment_materials
            .add(&material.id, material_balance)
            .unwrap();
    }
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
    (core, player_id, owned_uuid, material.id)
}

#[test]
fn duplicate_material_rows_with_insufficient_aggregate_leave_enhancement_state_unchanged() {
    let (mut core, player_id, owned_uuid, material_id) = duplicate_enhancement_cost_fixture(2);
    let inventory_before = serde_json::to_value(core.inventory().unwrap()).unwrap();

    let error = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .expect_err("aggregate cost four must reject balance two");

    assert_eq!(
        serde_json::to_value(core.inventory().unwrap()).unwrap(),
        inventory_before,
        "a failed enhancement must not consume an earlier duplicate row"
    );
    assert!(matches!(
        error,
        GameError::ActionUnavailable {
            action: ActionKind::EnhanceEquipment,
            reason: ActionDisabledReasonCode::NoEligiblePayload,
        }
    ));
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipment_materials
            .amount(&material_id),
        2
    );
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipments
            .get_item(&owned_uuid)
            .unwrap()
            .enhancement_level,
        0
    );
}

#[test]
fn duplicate_material_rows_with_exact_aggregate_consume_once_and_enhance_once() {
    let (mut core, player_id, owned_uuid, material_id) = duplicate_enhancement_cost_fixture(4);

    let result = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: owned_uuid,
            },
        )
        .unwrap();
    let BehaviorResult::EquipmentEnhanced {
        enhancement_level,
        inventory_diff,
        ..
    } = result
    else {
        panic!("expected one equipment enhancement result");
    };

    assert_eq!(enhancement_level, 1);
    assert_eq!(inventory_diff.updated.len(), 1);
    assert_eq!(inventory_diff.material_stacks.len(), 1);
    assert_eq!(inventory_diff.material_stacks[0].material_id, material_id);
    assert_eq!(inventory_diff.material_stacks[0].amount, 0);
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipments
            .get_item(&owned_uuid)
            .unwrap()
            .enhancement_level,
        1
    );
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipment_materials
            .amount(&material_id),
        0
    );
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
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE001_0001),
    );
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let equip_result = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                copy_uuid,
            },
        )
        .unwrap();

    let BehaviorResult::SkillFragmentLoadoutUpdated {
        employee_uuid: updated_employee,
        active_copy_uuid,
    } = equip_result
    else {
        panic!("expected skill fragment loadout result");
    };
    assert_eq!(updated_employee, employee_uuid);
    assert_eq!(active_copy_uuid, Some(copy_uuid));

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
                copy_uuid,
            },
        )
        .unwrap();
    let BehaviorResult::SkillFragmentLoadoutUpdated {
        active_copy_uuid, ..
    } = unequip_result
    else {
        panic!("expected skill fragment loadout result");
    };
    assert_eq!(active_copy_uuid, None);
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
fn newly_dead_employee_keeps_roster_record_and_atomically_returns_equipment_and_shared_fragment() {
    let fragment = active_skill_fragment("death_recovery_fragment", 0xF0D0, "death_recovery_skill");
    let weapon = weapon_equipment(0xE0D0, "death_recovery_sword", WeaponArchetype::Sword);
    let game_data =
        game_data_with_equipment_and_skill_fragments(vec![weapon.clone()], vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let weapon_instance_uuid = Uuid::from_u128(0xE0D0_0001);
    grant_and_equip_weapon(&mut core, employee_uuid, weapon, weapon_instance_uuid);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
        },
    )
    .unwrap();

    let mut staged_roster = core.state.roster.clone();
    {
        let employee = staged_roster.get_mut(&employee_uuid).unwrap();
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE;
    }
    let mut staged_inventory = core.state.inventory.clone();
    let mut staged_fragments = core.state.skill_fragments.clone();
    let expected_updated_ids = staged_roster
        .get(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .map(|equipped| equipped.instance_uuid)
        .collect::<Vec<_>>();

    let diff = core
        .recover_assets_for_new_deaths(
            &mut staged_roster,
            &mut staged_inventory,
            &mut staged_fragments,
        )
        .unwrap();

    let dead = staged_roster.get(&employee_uuid).unwrap();
    assert!(dead.condition.is_dead());
    assert!(dead.loadout.item_slot.is_empty());
    assert!(dead.skill_fragments.active_copy_uuid().is_none());
    assert_eq!(dead.profile, core.state.roster.get(&employee_uuid).unwrap().profile);
    assert!(serde_json::to_value(dead).unwrap().get("psychology").is_none());
    assert_eq!(staged_roster.len(), core.state.roster.len());
    assert_eq!(
        diff.updated
            .iter()
            .map(InventoryItemDto::uuid)
            .collect::<Vec<_>>(),
        expected_updated_ids
    );
    assert_eq!(
        staged_inventory
            .equipments
            .get_item(&weapon_instance_uuid)
            .unwrap()
            .equipped_to,
        None
    );
    assert!(diff.updated.iter().any(|item| matches!(
        item,
        InventoryItemDto::Equipment(equipment)
            if equipment.uuid == weapon_instance_uuid
    )));
    assert!(staged_inventory
        .validate_checkpoint_state(&staged_roster)
        .is_ok());

    // Recovery is staged: until the enclosing result transaction commits, live state is untouched.
    let persistent = core.state.roster.get(&employee_uuid).unwrap();
    assert!(persistent.is_alive());
    assert_eq!(
        persistent.skill_fragments.active_copy_uuid(),
        Some(copy_uuid)
    );
    assert_eq!(
        core.state
            .inventory
            .equipments
            .get_item(&weapon_instance_uuid)
            .unwrap()
            .equipped_to,
        Some(employee_uuid)
    );
}

#[test]
fn death_recovery_rejects_a_mismatched_reverse_binding_without_mutating_either_staged_ledger() {
    let mut core = GameCore::new(empty_game_data(), 0xD3A7_0001);
    let player_id = Uuid::from_u128(0xD3A7_0001);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let mismatched_item_uuid = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .next()
        .unwrap()
        .instance_uuid;

    let mut staged_roster = core.state.roster.clone();
    staged_roster
        .get_mut(&employee_uuid)
        .unwrap()
        .condition
        .physical_damage = MAX_PHYSICAL_DAMAGE;
    let mut staged_inventory = core.state.inventory.clone();
    let mut staged_fragments = core.state.skill_fragments.clone();
    staged_inventory
        .equipments
        .get_item_mut(&mismatched_item_uuid)
        .unwrap()
        .equipped_to = None;
    let roster_before = serde_json::to_value(&staged_roster).unwrap();
    let inventory_before = serde_json::to_value(&staged_inventory).unwrap();

    let error = core
        .recover_assets_for_new_deaths(
            &mut staged_roster,
            &mut staged_inventory,
            &mut staged_fragments,
        )
        .unwrap_err();

    assert!(matches!(error, GameError::InvalidAction));
    assert_eq!(serde_json::to_value(&staged_roster).unwrap(), roster_before);
    assert_eq!(
        serde_json::to_value(&staged_inventory).unwrap(),
        inventory_before
    );
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
        .combat_eligible_employee_ids()
        .first()
        .copied()
        .expect("starter employee");

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                copy_uuid: Uuid::from_u128(0xFFFF),
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[0],
            copy_uuid,
        },
    )
    .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee_ids[1],
                copy_uuid,
            },
        )
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[1],
            copy_uuid,
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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
    let first_copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee_ids[0],
            copy_uuid: first_copy_uuid,
        },
    )
    .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee_ids[1],
                copy_uuid,
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        sword,
        Uuid::from_u128(0xE002_0001),
    );
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                copy_uuid,
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
        .find(|entry| entry["copy_uuid"] == json!(copy_uuid))
        .unwrap();
    assert_eq!(compatibility["is_compatible"], false);
    assert_eq!(
        compatibility["failure_codes"],
        json!(["weapon_archetype_mismatch"])
    );
}

#[test]
fn equip_item_moves_equipped_item_between_employees_in_one_result() {
    let unit_meta = abnormality_meta(1);
    let moving = equipment_meta(20, "moving_weapon", EquipmentType::Weapon);
    let displaced = equipment_meta(21, "displaced_weapon", EquipmentType::Weapon);
    let game_data = game_data_with_equipment(
        Arc::clone(&unit_meta),
        vec![moving.clone(), displaced.clone()],
    );
    let mut core = GameCore::new(game_data, 123);
    start_new_game_with_default_starters(&mut core, Uuid::from_u128(1));
    let employees = core.roster().unwrap().combat_eligible_employee_ids();
    let source_employee = employees[0];
    let target_employee = employees[1];
    let moving_uuid = Uuid::from_u128(220);
    let displaced_uuid = Uuid::from_u128(221);
    {
        let inventory = core.inventory_mut().unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(moving_uuid, Arc::new(moving)))
            .unwrap();
        inventory
            .equipments
            .add_item(OwnedEquipment::new(displaced_uuid, Arc::new(displaced)))
            .unwrap();
    }
    core.handle_equip_item(moving_uuid, source_employee)
        .unwrap();
    core.handle_equip_item(displaced_uuid, target_employee)
        .unwrap();

    let before_same_target = serialized_equipment_ledgers(&core);
    let same_target_error = core
        .handle_equip_item(moving_uuid, source_employee)
        .unwrap_err();
    assert!(matches!(same_target_error, GameError::InvalidAction));
    assert_eq!(serialized_equipment_ledgers(&core), before_same_target);
    assert_equipment_loadout_graph_valid(&core);

    let result = core
        .handle_equip_item(moving_uuid, target_employee)
        .unwrap();

    let BehaviorResult::EquipItem { result } = result else {
        panic!("expected equip item result");
    };
    assert_eq!(
        result
            .inventory_diff
            .updated
            .iter()
            .map(InventoryItemDto::uuid)
            .collect::<Vec<_>>(),
        vec![moving_uuid, displaced_uuid]
    );
    assert!(result.inventory_diff.added.is_empty());
    assert!(result.inventory_diff.removed.is_empty());
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipments
            .get_item(&moving_uuid)
            .unwrap()
            .equipped_to,
        Some(target_employee)
    );
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipments
            .get_item(&displaced_uuid)
            .unwrap()
            .equipped_to,
        None
    );
    assert!(!core
        .roster()
        .unwrap()
        .get(&source_employee)
        .unwrap()
        .loadout
        .item_slot
        .contains_instance(moving_uuid));
    assert!(core
        .roster()
        .unwrap()
        .get(&target_employee)
        .unwrap()
        .loadout
        .item_slot
        .contains_instance(moving_uuid));
    assert_equipment_loadout_graph_valid(&core);
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    grant_and_equip_weapon(&mut core, employee_uuid, gun, Uuid::from_u128(0xE003_0001));
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                copy_uuid,
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentLoadoutUpdated { .. }
    ));
}

#[test]
fn equipment_replacement_rejects_weapon_that_invalidates_active_fragment_atomically() {
    let mut fragment = active_skill_fragment("gun_locked_fragment", 0xF015, "gun_locked_skill");
    fragment.compatibility = SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
        ..SkillFragmentCompatibilityRequirements::default()
    };
    let gun = weapon_equipment(0xE008, "equipped_gun", WeaponArchetype::Gun);
    let sword = weapon_equipment(0xE00A, "replacement_sword", WeaponArchetype::Sword);
    let game_data = game_data_with_equipment_and_skill_fragments(
        vec![gun.clone(), sword.clone()],
        vec![fragment.clone()],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    grant_and_equip_weapon(&mut core, employee_uuid, gun, Uuid::from_u128(0xE008_0001));
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
        },
    )
    .unwrap();
    let sword_owned_uuid = Uuid::from_u128(0xE00A_0001);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(sword_owned_uuid, Arc::new(sword)))
        .unwrap();

    let before = serialized_equipment_ledgers(&core);

    let err = core
        .handle_equip_item(sword_owned_uuid, employee_uuid)
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
    assert_eq!(serialized_equipment_ledgers(&core), before);
    assert_equipment_loadout_graph_valid(&core);
    assert_eq!(
        core.inventory()
            .unwrap()
            .equipments
            .get_item(&sword_owned_uuid)
            .unwrap()
            .equipped_to,
        None
    );
    assert!(core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .any(|equipped| equipped.instance_uuid == Uuid::from_u128(0xE008_0001)));
}

#[test]
fn skill_fragment_form_selection_requires_maintenance_and_preserves_dust() {
    let target = active_skill_fragment("upgrade_active_fragment", 0xF00F, "upgrade_skill");
    let game_data = game_data_with_skill_fragments(vec![target.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &target,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid,
                form_id: "base".into(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::SelectSkillFragmentForm,
            reason: ActionDisabledReasonCode::ContextIrrelevant,
        }
    ));

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
    core.state.skill_fragments.add_fragment_dust(4).unwrap();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid,
                form_id: "base".into(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentFormSelected { changed: false, .. }
    ));
    assert_eq!(core.state.skill_fragments.count(&target.id), 1);
    assert_eq!(core.state.skill_fragments.fragment_dust(), 4);
}

#[test]
fn skill_fragment_dismantle_is_maintenance_only_and_grants_explicit_recipe() {
    let fragment = active_skill_fragment("dismantle_fragment", 0xF011, "dismantle_skill");
    let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let other_copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid,
                confirmed: true,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::DismantleSkillFragment,
            reason: ActionDisabledReasonCode::ContextIrrelevant,
        }
    ));

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
                copy_uuid,
                confirmed: true,
            },
        )
        .unwrap();

    let BehaviorResult::SkillFragmentDismantled {
        definition_id,
        enkephalin_gained,
        ..
    } = result
    else {
        panic!("expected dismantle");
    };
    assert_eq!(definition_id, fragment.id);
    assert_eq!(enkephalin_gained, 2);
    assert_eq!(core.state.skill_fragments.count(&fragment.id), 1);
    assert!(core.state.skill_fragments.get(&other_copy_uuid).is_some());
    assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE004_0001),
    );
    let bag_copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
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
                copy_uuid: bag_copy_uuid,
                confirmed: false,
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentDismantled { .. }
    ));
    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(employee.skill_fragments.active_copy_uuid(), Some(copy_uuid));
    assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
}

#[test]
fn skill_fragment_dismantle_allows_last_copy() {
    let fragment = active_skill_fragment("last_copy_dismantle_fragment", 0xF013, "last_copy_skill");
    let game_data = game_data_with_skill_fragments(vec![fragment.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
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
                copy_uuid,
                confirmed: true,
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SkillFragmentDismantled { .. }
    ));
    assert_eq!(core.state.skill_fragments.count(&fragment.id), 0);
}
