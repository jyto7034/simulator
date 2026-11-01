//! Actual paid investment and explicit carried/warehouse material allocation.
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::game::{behavior::GameError, data::equipment_data::EquipmentDatabase, resources::Inventory, relic::RunCraftingResources};

/// The typed run balance is the sole owner of the reserved equipment dust id.
pub(crate) fn carried_amount(inventory: &Inventory, resources: &RunCraftingResources, id: &str) -> u32 {
    if id == "equipment_dust" { resources.equipment_dust } else { inventory.equipment_materials.amount(id) }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSources {
    pub carried: u32,
    pub warehouse: u32,
}
impl MaterialSources {
    pub fn total(self) -> Result<u32, GameError> {
        self.carried.checked_add(self.warehouse).ok_or(GameError::InvalidAction)
    }
}
crate::authoring::clone_shape::copy_shape!(MaterialSources);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialInvestment {
    pub materials: BTreeMap<String, MaterialSources>,
}
crate::authoring::clone_shape::struct_shape!(MaterialInvestment { materials });
impl MaterialInvestment {
    /// Call on a transaction candidate, before publication.
    pub fn record(&mut self, paid: &Self) -> Result<(), GameError> {
        let mut next = self.materials.clone();
        for (id, amount) in &paid.materials {
            let old = next.entry(id.clone()).or_default();
            old.carried = old.carried.checked_add(amount.carried).ok_or(GameError::InvalidAction)?;
            old.warehouse = old.warehouse.checked_add(amount.warehouse).ok_or(GameError::InvalidAction)?;
            old.total()?;
        }
        self.materials = next;
        Ok(())
    }
    pub(crate) fn validate_additional(&self, costs: &BTreeMap<String, u32>) -> Result<(), GameError> {
        for (id, amount) in costs {
            self.materials.get(id).copied().unwrap_or_default().total()?
                .checked_add(*amount).ok_or(GameError::InvalidAction)?;
        }
        Ok(())
    }
    pub fn refunds(&self) -> Result<BTreeMap<String, u32>, GameError> {
        let mut result = BTreeMap::new();
        for (id, paid) in &self.materials {
            let amount = paid.total()? / 2;
            if amount > 0 { result.insert(id.clone(), amount); }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialPayment {
    pub material_id: String,
    pub carried: u32,
    pub warehouse: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialRecovery {
    pub material_id: String,
    pub amount: u32,
}

/// Read-only source allocation; never changes balances or consumes a UUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialPaymentPreview {
    pub requirements: BTreeMap<String, u32>,
    pub selected: Vec<MaterialPayment>,
    pub actual: Option<MaterialInvestment>,
}

fn plan_materials(
    inventory: &Inventory, resources: &RunCraftingResources, warehouse: &BTreeMap<String, u32>, debits: &BTreeMap<String, u32>,
    requirements: &BTreeMap<String, u32>, selected: Option<&[MaterialPayment]>,
    warehouse_allowed: bool, data: &EquipmentDatabase,
) -> Result<MaterialInvestment, GameError> {
    let mut paid = MaterialInvestment::default();
    if let Some(selected) = selected {
        let mut seen = BTreeSet::new();
        for row in selected {
            let sources = MaterialSources { carried: row.carried, warehouse: row.warehouse };
            if !seen.insert(row.material_id.as_str()) || sources.total()? == 0
                || requirements.get(&row.material_id) != Some(&sources.total()?) {
                return Err(GameError::InvalidAction);
            }
            paid.materials.insert(row.material_id.clone(), sources);
        }
        if paid.materials.len() != requirements.len() { return Err(GameError::InvalidAction); }
    } else {
        paid.materials = requirements.iter().map(|(id, amount)| (id.clone(), MaterialSources { carried: *amount, warehouse: 0 })).collect();
    }
    for (id, sources) in &paid.materials {
        let metadata = data.get_material_by_id(id).ok_or(GameError::InvalidAction)?;
        if sources.total()? == 0 || carried_amount(inventory, resources, id) < sources.carried
            || (sources.warehouse > 0 && (!warehouse_allowed || !metadata.material_type.is_storable()))
            || warehouse.get(id).copied().unwrap_or(0) < sources.warehouse {
            return Err(GameError::InvalidAction);
        }
        debits.get(id).copied().unwrap_or(0).checked_add(sources.warehouse).ok_or(GameError::InvalidAction)?;
    }
    Ok(paid)
}

pub(crate) fn preview_material_payment(
    inventory: &Inventory, resources: &RunCraftingResources, warehouse: &BTreeMap<String, u32>, debits: &BTreeMap<String, u32>,
    requirements: &BTreeMap<String, u32>, selected: &[MaterialPayment],
    warehouse_allowed: bool, data: &EquipmentDatabase,
) -> MaterialPaymentPreview {
    MaterialPaymentPreview {
        requirements: requirements.clone(), selected: selected.to_vec(),
        actual: plan_materials(inventory, resources, warehouse, debits, requirements, Some(selected), warehouse_allowed, data).ok(),
    }
}

/// Returns the selected actual investment. All arguments are transaction candidates.
/// Explicit allocations must match the entire recipe; no source fallback is performed.
pub(crate) fn pay_materials(
    inventory: &mut Inventory,
    resources: &mut RunCraftingResources,
    warehouse: &mut BTreeMap<String, u32>,
    debits: &mut BTreeMap<String, u32>,
    requirements: &BTreeMap<String, u32>,
    selected: Option<&[MaterialPayment]>,
    warehouse_allowed: bool,
    data: &EquipmentDatabase,
) -> Result<MaterialInvestment, GameError> {
    let paid = plan_materials(inventory, resources, warehouse, debits, requirements, selected, warehouse_allowed, data)?;
    for (id, sources) in &paid.materials {
        if sources.carried > 0 {
            if id == "equipment_dust" { resources.equipment_dust -= sources.carried; }
            else { inventory.equipment_materials.consume(id, sources.carried)?; }
        }
        if sources.warehouse > 0 {
            let remaining = warehouse[id] - sources.warehouse;
            if remaining == 0 { warehouse.remove(id); } else { warehouse.insert(id.clone(), remaining); }
            *debits.entry(id.clone()).or_default() += sources.warehouse;
        }
    }
    Ok(paid)
}

/// Adjust only this expedition's consumption; unrelated warehouse deposits survive.
pub(crate) fn reconcile_warehouse(
    warehouse: &BTreeMap<String, u32>, current: &BTreeMap<String, u32>, target: &BTreeMap<String, u32>,
) -> Result<BTreeMap<String, u32>, GameError> {
    let mut next = warehouse.clone();
    for id in current.keys().chain(target.keys()).collect::<BTreeSet<_>>() {
        let balance = u64::from(warehouse.get(id).copied().unwrap_or(0))
            + u64::from(current.get(id).copied().unwrap_or(0));
        let restored = balance.checked_sub(u64::from(target.get(id).copied().unwrap_or(0)))
            .and_then(|n| u32::try_from(n).ok()).ok_or(GameError::InvalidAction)?;
        if restored == 0 { next.remove(id); } else { next.insert(id.clone(), restored); }
    }
    Ok(next)
}
