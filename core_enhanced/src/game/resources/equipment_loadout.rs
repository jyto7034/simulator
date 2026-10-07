use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::equipment_data::EquipmentType,
    employee::{EmployeeActivity, EmployeeRoster},
    resources::{
        inventory::{Inventory, OwnedEquipment},
        item_slot::{EquippedRef, ItemSlot},
    },
};

#[derive(Debug, Clone)]
pub(crate) struct EquipmentLoadoutEquipPlan {
    item_uuid: Uuid,
    target_unit: Uuid,
    source_unit: Option<Uuid>,
    replaced_item_uuid: Option<Uuid>,
    incoming_ref: EquippedRef,
    allow_duplicate: bool,
    projected_target: ItemSlot,
}

impl EquipmentLoadoutEquipPlan {
    pub(crate) fn projected_target(&self) -> &ItemSlot {
        &self.projected_target
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EquipmentLoadoutEquipOutcome {
    pub(crate) updated_item_uuids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StarterEquipmentAssignment {
    employee_uuid: Uuid,
    instance_uuid: Uuid,
    base_uuid: Uuid,
    equipment_type: EquipmentType,
    allow_duplicate: bool,
}

impl StarterEquipmentAssignment {
    pub(crate) fn new(
        employee_uuid: Uuid,
        instance_uuid: Uuid,
        base_uuid: Uuid,
        equipment_type: EquipmentType,
        allow_duplicate: bool,
    ) -> Self {
        Self {
            employee_uuid,
            instance_uuid,
            base_uuid,
            equipment_type,
            allow_duplicate,
        }
    }
}

/// Owns paired mutation of employee item slots and inventory reverse bindings.
///
/// Public inventory and slot APIs remain available for compatibility, but production lifecycle
/// transitions use this aggregate so one operation plans and commits both serialized ledgers.
pub(crate) struct EquipmentLoadoutTransition<'a> {
    inventory: &'a mut Inventory,
    roster: &'a mut EmployeeRoster,
}

impl<'a> EquipmentLoadoutTransition<'a> {
    pub(crate) fn new(inventory: &'a mut Inventory, roster: &'a mut EmployeeRoster) -> Self {
        Self { inventory, roster }
    }

    pub(crate) fn plan_equip(
        inventory: &Inventory,
        roster: &EmployeeRoster,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<EquipmentLoadoutEquipPlan, GameError> {
        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if owned_equipment.meta.bound {
            return Err(GameError::InvalidAction);
        }

        let target = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        if !target.eligibility(EmployeeActivity::Loadout).eligible {
            return Err(GameError::InvalidAction);
        }

        let source_unit = owned_equipment.equipped_to;
        if source_unit == Some(target_unit) {
            return Err(GameError::InvalidAction);
        }
        if let Some(source_unit) = source_unit {
            let source = roster.get(&source_unit).ok_or(GameError::UnitNotFound)?;
            if !source.eligibility(EmployeeActivity::Loadout).eligible
                || !source.loadout.item_slot.contains_instance(item_uuid)
            {
                return Err(GameError::InvalidAction);
            }
        }

        let incoming_ref = EquippedRef {
            instance_uuid: item_uuid,
            base_uuid: owned_equipment.meta.uuid,
            equipment_type: owned_equipment.meta.equipment_type,
        };
        let replaced = target
            .loadout
            .item_slot
            .iter()
            .find(|equipped| equipped.equipment_type == owned_equipment.meta.equipment_type)
            .copied();
        let mut projected_target = target.loadout.item_slot.clone();
        if let Some(replaced) = replaced {
            let replaced_item = inventory
                .equipments
                .get_item(&replaced.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if replaced_item.equipped_to != Some(target_unit) || replaced_item.meta.bound {
                return Err(GameError::InvalidAction);
            }
            projected_target
                .remove_by_instance(replaced.instance_uuid)
                .ok_or(GameError::InvalidAction)?;
        }
        projected_target
            .equip(incoming_ref, owned_equipment.meta.allow_duplicate_equip)
            .map_err(|_| GameError::InvalidAction)?;

        Ok(EquipmentLoadoutEquipPlan {
            item_uuid,
            target_unit,
            source_unit,
            replaced_item_uuid: replaced.map(|item| item.instance_uuid),
            incoming_ref,
            allow_duplicate: owned_equipment.meta.allow_duplicate_equip,
            projected_target,
        })
    }

    pub(crate) fn equip(
        &mut self,
        plan: EquipmentLoadoutEquipPlan,
    ) -> Result<EquipmentLoadoutEquipOutcome, GameError> {
        let mut next_roster = self.roster.clone();
        let mut next_inventory = self.inventory.clone();

        if let Some(source_unit) = plan.source_unit {
            next_roster
                .get_mut(&source_unit)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .remove_by_instance(plan.item_uuid)
                .ok_or(GameError::InvalidAction)?;
        }

        if let Some(replaced_item_uuid) = plan.replaced_item_uuid {
            next_roster
                .get_mut(&plan.target_unit)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .remove_by_instance(replaced_item_uuid)
                .ok_or(GameError::InvalidAction)?;
            next_inventory
                .equipments
                .get_item_mut(&replaced_item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = None;
        }

        next_roster
            .get_mut(&plan.target_unit)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .equip(plan.incoming_ref, plan.allow_duplicate)
            .map_err(|_| GameError::InvalidAction)?;
        next_inventory
            .equipments
            .get_item_mut(&plan.item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .equipped_to = Some(plan.target_unit);

        let mut updated_item_uuids = vec![plan.item_uuid];
        if let Some(replaced_item_uuid) = plan.replaced_item_uuid {
            updated_item_uuids.push(replaced_item_uuid);
        }
        *self.roster = next_roster;
        *self.inventory = next_inventory;
        Ok(EquipmentLoadoutEquipOutcome { updated_item_uuids })
    }

    pub(crate) fn validate_unequip(
        inventory: &Inventory,
        roster: &EmployeeRoster,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<(), GameError> {
        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if owned_equipment.equipped_to != Some(target_unit) || owned_equipment.meta.bound {
            return Err(GameError::InvalidAction);
        }
        let employee = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        if !employee.eligibility(EmployeeActivity::Loadout).eligible
            || !employee.loadout.item_slot.contains_instance(item_uuid)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(crate) fn unequip(&mut self, item_uuid: Uuid, target_unit: Uuid) -> Result<(), GameError> {
        Self::validate_unequip(self.inventory, self.roster, item_uuid, target_unit)?;
        let mut next_roster = self.roster.clone();
        let mut next_inventory = self.inventory.clone();
        next_roster
            .get_mut(&target_unit)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .remove_by_instance(item_uuid)
            .ok_or(GameError::InvalidAction)?;
        next_inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .equipped_to = None;
        *self.roster = next_roster;
        *self.inventory = next_inventory;
        Ok(())
    }

    pub(crate) fn assign_starter_equipment(
        &mut self,
        assignments: &[StarterEquipmentAssignment],
    ) -> Result<(), GameError> {
        let mut projected_roster = self.roster.clone();
        let mut seen_instances = HashSet::new();
        for assignment in assignments {
            if !seen_instances.insert(assignment.instance_uuid) {
                return Err(GameError::InvalidAction);
            }
            let owned = self
                .inventory
                .equipments
                .get_item(&assignment.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if owned.equipped_to.is_some()
                || owned.meta.uuid != assignment.base_uuid
                || owned.meta.equipment_type != assignment.equipment_type
            {
                return Err(GameError::InvalidAction);
            }
            projected_roster
                .get_mut(&assignment.employee_uuid)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .equip(
                    EquippedRef {
                        instance_uuid: assignment.instance_uuid,
                        base_uuid: assignment.base_uuid,
                        equipment_type: assignment.equipment_type,
                    },
                    assignment.allow_duplicate,
                )
                .map_err(|_| GameError::InvalidAction)?;
        }

        let mut next_inventory = self.inventory.clone();
        for assignment in assignments {
            next_inventory
                .equipments
                .get_item_mut(&assignment.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = Some(assignment.employee_uuid);
        }
        *self.roster = projected_roster;
        *self.inventory = next_inventory;
        Ok(())
    }

    pub(crate) fn clear_employee_equipment(
        &mut self,
        employee_uuid: Uuid,
    ) -> Result<Vec<Uuid>, GameError> {
        let equipped = self
            .roster
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .iter()
            .copied()
            .collect::<Vec<_>>();
        for equipped_ref in &equipped {
            let owned = self
                .inventory
                .equipments
                .get_item(&equipped_ref.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if !binding_matches(owned, equipped_ref, employee_uuid) {
                return Err(GameError::InvalidAction);
            }
        }

        let mut next_roster = self.roster.clone();
        let mut next_inventory = self.inventory.clone();
        next_roster
            .get_mut(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .unequip_all();
        let mut updated_item_uuids = Vec::with_capacity(equipped.len());
        for equipped_ref in equipped {
            next_inventory
                .equipments
                .get_item_mut(&equipped_ref.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = None;
            updated_item_uuids.push(equipped_ref.instance_uuid);
        }
        *self.roster = next_roster;
        *self.inventory = next_inventory;
        Ok(updated_item_uuids)
    }

    pub(crate) fn remove_for_dismantle(
        &mut self,
        item_uuid: Uuid,
    ) -> Result<OwnedEquipment, GameError> {
        let owned = self
            .inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if owned.meta.bound {
            return Err(GameError::InvalidAction);
        }
        let equipped_to = owned.equipped_to;
        if let Some(employee_uuid) = equipped_to {
            let equipped_ref = self
                .roster
                .get(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .iter()
                .find(|equipped| equipped.instance_uuid == item_uuid)
                .copied()
                .ok_or(GameError::InvalidAction)?;
            if !binding_matches(owned, &equipped_ref, employee_uuid) {
                return Err(GameError::InvalidAction);
            }
        } else if self
            .roster
            .iter()
            .any(|employee| employee.loadout.item_slot.contains_instance(item_uuid))
        {
            return Err(GameError::InvalidAction);
        }

        let mut next_roster = self.roster.clone();
        let mut next_inventory = self.inventory.clone();
        if let Some(employee_uuid) = equipped_to {
            next_roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .remove_by_instance(item_uuid)
                .ok_or(GameError::InvalidAction)?;
        }
        let removed = next_inventory
            .equipments
            .remove_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        *self.roster = next_roster;
        *self.inventory = next_inventory;
        Ok(removed)
    }
}

fn binding_matches(owned: &OwnedEquipment, equipped: &EquippedRef, employee_uuid: Uuid) -> bool {
    owned.instance_uuid == equipped.instance_uuid
        && owned.meta.uuid == equipped.base_uuid
        && owned.meta.equipment_type == equipped.equipment_type
        && owned.equipped_to == Some(employee_uuid)
}
