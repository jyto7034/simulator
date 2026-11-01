//! Canonical run-owned relic copies and their independent three-slot loadouts.
pub(crate) mod allocation;
pub mod numeric;
pub mod plan;
pub mod structural;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::relic_data::{RelicDatabase, RelicDefinition},
    managers::uuid_manager::UuidManager,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicVariant {
    #[default]
    Normal,
    Corrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedRelic {
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub variant: RelicVariant,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicInventory {
    #[serde(
        deserialize_with = "crate::game::skill_fragment::bounded::map::<_, _, _, { usize::MAX }>"
    )]
    copies: BTreeMap<Uuid, OwnedRelic>,
    /// The single placement authority. Copies have no independently mutable back-reference.
    #[serde(
        deserialize_with = "crate::game::skill_fragment::bounded::map::<_, _, _, { usize::MAX }>"
    )]
    slots: BTreeMap<Uuid, RelicSlots>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct RelicSlots([Option<Uuid>; 3]);
crate::authoring::clone_shape::copy_shape!(RelicSlots);

/// Reserved legacy fragment_dust remains with SkillFragmentInventory. This is
/// the one run equipment-dust owner for subsequent crafting consumers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCraftingResources {
    pub equipment_dust: u32,
    pub relic_corruption_catalysts: u32,
}
crate::authoring::clone_shape::copy_shape!(RelicVariant);
crate::authoring::clone_shape::struct_shape!(OwnedRelic {
    copy_uuid,
    definition_id,
    variant
});
impl crate::authoring::clone_shape::CloneShape for RunCraftingResources {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        if self.is_empty() {
            return Ok(());
        }
        meter.scan_storage(Some(1))
    }
}
impl crate::authoring::clone_shape::CloneShape for RelicInventory {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        if self.is_empty() {
            return Ok(());
        }
        meter.scan_storage(Some(1))?;
        self.copies.visit_clone_shape(meter)?;
        self.slots.visit_clone_shape(meter)
    }
}

fn invalid() -> GameError {
    GameError::InvalidAction
}
fn charge_scan(units: Option<usize>, site: &'static str) -> Result<(), GameError> {
    let units = units
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| GameError::InvalidStaticData("relic scan accounting overflow".into()))?;
    crate::authoring::control::charge(units, site)
}

impl RunCraftingResources {
    pub fn is_empty(&self) -> bool {
        self.equipment_dust == 0 && self.relic_corruption_catalysts == 0
    }
    pub fn grant_equipment_dust(&mut self, amount: u32) -> Result<(), GameError> {
        if amount == 0 {
            return Err(invalid());
        }
        self.equipment_dust = self
            .equipment_dust
            .checked_add(amount)
            .ok_or_else(invalid)?;
        Ok(())
    }
    pub fn grant_relic_catalysts(&mut self, amount: u32) -> Result<(), GameError> {
        if amount == 0 {
            return Err(invalid());
        }
        self.relic_corruption_catalysts = self
            .relic_corruption_catalysts
            .checked_add(amount)
            .ok_or_else(invalid)?;
        Ok(())
    }
    pub(crate) fn consume_relic_catalyst(&mut self) -> Result<(), GameError> {
        self.relic_corruption_catalysts = self
            .relic_corruption_catalysts
            .checked_sub(1)
            .ok_or_else(invalid)?;
        Ok(())
    }
}

impl RelicInventory {
    pub fn is_empty(&self) -> bool {
        self.copies.is_empty() && self.slots.is_empty()
    }
    /// Heap owned by these two canonical trees. Empty inline roots add no
    /// retained allocation. The node bound matches growth admission: up to
    /// eleven key/value slots plus the fixed B-tree node/header allowance.
    pub(crate) fn retained_extent(&self) -> Result<usize, GameError> {
        charge_scan(
            self.copies.len().checked_add(self.slots.len()),
            "relic_owner_extent",
        )?;
        let copy_nodes = self.copies.len().checked_mul(
            std::mem::size_of::<(Uuid, OwnedRelic)>()
                .checked_mul(11)
                .and_then(|n| n.checked_add(224))
                .ok_or_else(invalid)?,
        );
        let slot_nodes = self.slots.len().checked_mul(
            std::mem::size_of::<(Uuid, RelicSlots)>()
                .checked_mul(11)
                .and_then(|n| n.checked_add(224))
                .ok_or_else(invalid)?,
        );
        let mut bytes = copy_nodes
            .and_then(|n| n.checked_add(slot_nodes?))
            .ok_or_else(invalid)?;
        for copy in self.copies.values() {
            bytes = bytes
                .checked_add(copy.definition_id.capacity())
                .ok_or_else(invalid)?;
        }
        Ok(bytes)
    }
    pub(crate) fn preflight_clone(&self) -> Result<(), GameError> {
        if !self.is_empty() {
            let bytes = self.retained_extent()?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(bytes),
                "relic_owner_clone",
            )?;
            if crate::game::battle::runtime_control::is_active()
                && !crate::authoring::control::is_active()
            {
                crate::game::resource::store::preflight_resource_clone(self)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            }
        }
        Ok(())
    }
    pub fn copies(&self) -> impl Iterator<Item = &OwnedRelic> {
        self.copies.values()
    }
    pub(crate) fn copy_count(&self) -> usize {
        self.copies.len()
    }
    pub fn get(&self, id: Uuid) -> Option<&OwnedRelic> {
        self.copies.get(&id)
    }
    pub(crate) fn has_equipped(&self) -> bool {
        !self.slots.is_empty()
    }
    pub fn employee_slots(&self, employee: Uuid) -> [Option<Uuid>; 3] {
        self.slots
            .get(&employee)
            .map(|slots| slots.0)
            .unwrap_or([None; 3])
    }
    pub fn equipped_to(&self, copy: Uuid) -> Result<Option<(Uuid, usize)>, GameError> {
        charge_scan(self.slots.len().checked_mul(3), "relic_slot_lookup")?;
        Ok(self.slots.iter().find_map(|(employee, slots)| {
            slots
                .0
                .iter()
                .position(|id| *id == Some(copy))
                .map(|slot| (*employee, slot))
        }))
    }
    pub fn grant(
        &mut self,
        definition: &RelicDefinition,
        uuids: &mut UuidManager,
    ) -> Result<Uuid, GameError> {
        let id = uuids.peek(UuidManager::NS_OWNED_RELIC);
        if id.is_nil() || self.copies.contains_key(&id) {
            return Err(invalid());
        }
        let bytes = std::mem::size_of::<(Uuid, OwnedRelic)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(definition.id.len()))
            .and_then(|n| n.checked_add(224));
        crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "relic_copy_grant")?;
        crate::authoring::control::charge(definition.id.len() as u64 + 1, "relic_copy_grant")?;
        let id = uuids.next_owned_relic()?;
        self.copies.insert(
            id,
            OwnedRelic {
                copy_uuid: id,
                definition_id: definition.id.clone(),
                variant: RelicVariant::Normal,
            },
        );
        Ok(id)
    }
    pub(crate) fn validate_employee_slots(
        &self,
        slots: &[Option<Uuid>; 3],
        database: &RelicDatabase,
    ) -> Result<(), GameError> {
        charge_scan(
            database
                .relics
                .len()
                .checked_mul(3)
                .and_then(|n| n.checked_add(9)),
            "relic_loadout_validation",
        )?;
        let mut definitions: [Option<&RelicDefinition>; 3] = [None; 3];
        for (index, id) in slots.iter().enumerate() {
            if let Some(id) = id {
                if slots[..index].contains(&Some(*id)) {
                    return Err(invalid());
                }
                let copy = self.get(*id).ok_or_else(invalid)?;
                let definition = database
                    .get_by_id(&copy.definition_id)
                    .ok_or_else(invalid)?;
                for previous in definitions[..index].iter().flatten() {
                    let previous: &RelicDefinition = previous;
                    if (previous.id == definition.id && !definition.allow_duplicates)
                        || definition.incompatible_with.contains(&previous.id)
                        || previous.incompatible_with.contains(&definition.id)
                    {
                        return Err(invalid());
                    }
                }
                definitions[index] = Some(definition);
            }
        }
        Ok(())
    }
    /// Private publication input. These deterministic copy IDs model a bounded
    /// three-slot ledger; no World, generation or execution authority is minted.
    pub(crate) fn publication_input(
        employee: Uuid,
        definitions: &[(&RelicDefinition, RelicVariant)],
        database: &RelicDatabase,
    ) -> Result<Option<Self>, GameError> {
        if definitions.len() > 3 || employee.is_nil() {
            return Err(invalid());
        }
        let bytes = definitions
            .iter()
            .try_fold(std::mem::size_of::<Self>() + 128, |n, (definition, _)| {
                n.checked_add(std::mem::size_of::<(Uuid, OwnedRelic)>() + 64 + definition.id.len())
            })
            .and_then(|n| n.checked_add(std::mem::size_of::<(Uuid, RelicSlots)>() + 64));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            bytes,
            "relic_publication_input",
        )?;
        let mut result = Self::default();
        let mut slots = [None; 3];
        for (index, (definition, variant)) in definitions.iter().enumerate() {
            crate::authoring::control::charge(
                definition.id.len() as u64 + 1,
                "relic_publication_copy",
            )?;
            let copy_uuid = Uuid::from_u128(index as u128 + 1);
            result.copies.insert(
                copy_uuid,
                OwnedRelic {
                    copy_uuid,
                    definition_id: definition.id.clone(),
                    variant: *variant,
                },
            );
            slots[index] = Some(copy_uuid);
        }
        match result.validate_employee_slots(&slots, database) {
            Ok(()) => {
                result.slots.insert(employee, RelicSlots(slots));
                Ok(Some(result))
            }
            Err(GameError::InvalidAction) => Ok(None),
            Err(error) => Err(error),
        }
    }
    /// All validation precedes mutation, including both affected employees.
    pub fn equip(
        &mut self,
        copy: Uuid,
        employee: Uuid,
        slot: usize,
        database: &RelicDatabase,
    ) -> Result<(), GameError> {
        if employee.is_nil() || slot >= 3 || self.get(copy).is_none() {
            return Err(invalid());
        }
        let source = self.equipped_to(copy)?;
        let mut target_slots = self.employee_slots(employee);
        if let Some((owner, old_slot)) = source {
            if owner == employee {
                target_slots[old_slot] = None;
            }
        }
        target_slots[slot] = Some(copy);
        self.validate_employee_slots(&target_slots, database)?;
        let source_slots =
            source
                .filter(|(owner, _)| *owner != employee)
                .map(|(owner, old_slot)| {
                    let mut slots = self.employee_slots(owner);
                    slots[old_slot] = None;
                    (owner, slots)
                });
        if let Some((_, slots)) = &source_slots {
            self.validate_employee_slots(slots, database)?;
        }
        if !self.slots.contains_key(&employee) {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                std::mem::size_of::<(Uuid, [Option<Uuid>; 3])>()
                    .checked_mul(11)
                    .and_then(|n| n.checked_add(224)),
                "relic_slot_owner",
            )?;
        }
        crate::authoring::control::poll("relic_equip_commit")?;
        if let Some((owner, slots)) = source_slots {
            self.write_slots(owner, slots);
        }
        self.write_slots(employee, target_slots);
        Ok(())
    }
    fn write_slots(&mut self, employee: Uuid, slots: [Option<Uuid>; 3]) {
        if slots.iter().all(Option::is_none) {
            self.slots.remove(&employee);
        } else {
            self.slots.insert(employee, RelicSlots(slots));
        }
    }
    pub fn unequip(&mut self, copy: Uuid) -> Result<(), GameError> {
        if self.get(copy).is_none() {
            return Err(invalid());
        }
        if let Some((employee, slot)) = self.equipped_to(copy)? {
            let mut slots = self.employee_slots(employee);
            slots[slot] = None;
            self.write_slots(employee, slots);
        }
        Ok(())
    }
    pub(crate) fn recover_employee(&mut self, employee: Uuid) {
        self.slots.remove(&employee);
    }
    pub(crate) fn validate_corruption_target(
        &self,
        copy: Uuid,
        database: &RelicDatabase,
    ) -> Result<&OwnedRelic, GameError> {
        charge_scan(Some(database.relics.len()), "relic_corruption_target")?;
        let copy = self.get(copy).ok_or_else(invalid)?;
        if copy.variant != RelicVariant::Normal
            || database
                .get_by_id(&copy.definition_id)
                .and_then(|d| d.corruption.as_ref())
                .is_none()
        {
            return Err(invalid());
        }
        Ok(copy)
    }
    pub(crate) fn corrupt(
        &mut self,
        copy: Uuid,
        database: &RelicDatabase,
    ) -> Result<(), GameError> {
        self.validate_corruption_target(copy, database)?;
        self.copies.get_mut(&copy).ok_or_else(invalid)?.variant = RelicVariant::Corrupt;
        Ok(())
    }
    pub(crate) fn return_dealer_escrow(&mut self, copy: OwnedRelic) -> Result<(), GameError> {
        if copy.copy_uuid.is_nil() || self.copies.contains_key(&copy.copy_uuid) { return Err(invalid()); }
        let bytes = std::mem::size_of::<(Uuid, OwnedRelic)>().checked_mul(11).and_then(|n| n.checked_add(copy.definition_id.len())).and_then(|n| n.checked_add(224));
        crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "dealer_escrow_return")?;
        self.copies.insert(copy.copy_uuid, copy); Ok(())
    }
    pub(crate) fn remove(&mut self, copy: Uuid) -> Result<OwnedRelic, GameError> {
        self.unequip(copy)?;
        self.copies.remove(&copy).ok_or_else(invalid)
    }
    pub(crate) fn validate(
        &self,
        database: &RelicDatabase,
        has_employee: impl Fn(Uuid) -> bool,
    ) -> Result<(), GameError> {
        charge_scan(
            self.copies
                .len()
                .checked_mul(database.relics.len())
                .and_then(|n| {
                    self.slots
                        .len()
                        .checked_mul(self.slots.len())
                        .and_then(|m| m.checked_mul(9))
                        .and_then(|m| n.checked_add(m))
                }),
            "relic_inventory_validation",
        )?;
        for (id, copy) in &self.copies {
            if id.is_nil() || id != &copy.copy_uuid {
                return Err(invalid());
            }
            let definition = database
                .get_by_id(&copy.definition_id)
                .ok_or_else(invalid)?;
            if copy.variant == RelicVariant::Corrupt && definition.corruption.is_none() {
                return Err(invalid());
            }
        }
        for (employee, slots) in &self.slots {
            if !has_employee(*employee) || slots.0.iter().all(Option::is_none) {
                return Err(invalid());
            }
            self.validate_employee_slots(&slots.0, database)?;
            for copy in slots.0.iter().flatten() {
                if self
                    .slots
                    .values()
                    .flat_map(|s| s.0.iter())
                    .filter(|id| **id == Some(*copy))
                    .count()
                    != 1
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicLoadoutDto {
    pub employee_uuid: Uuid,
    pub slots: [Option<Uuid>; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicStateDto {
    pub copies: Vec<OwnedRelic>,
    pub loadouts: Vec<RelicLoadoutDto>,
    pub resources: RunCraftingResources,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_sellable_copy_uuids: Vec<Uuid>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelicOperationKind {
    Corrupt,
    Dismantle,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicOperationPreviewDto {
    pub copy: OwnedRelic,
    pub operation: RelicOperationKind,
    pub effects: Vec<crate::game::data::relic_data::RelicDamageEffect>,
    pub penalties: Vec<crate::game::data::relic_data::RelicPenalty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub support_conversions: Vec<SupportConversionPreviewDto>,
    pub catalyst_cost: u32,
    pub equipment_dust_gained: u32,
    pub irreversible: bool,
    pub equipped_to: Option<(Uuid, usize)>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportConversionPreviewDto {
    pub skill_id: crate::game::ability::SkillId,
    pub step_id: String,
    pub effect_id: String,
    pub conversion: structural::support::SupportConversionDef,
}
impl RelicInventory {
    pub(crate) fn snapshot(
        &self,
        resources: RunCraftingResources,
    ) -> Result<RelicStateDto, GameError> {
        let extent = self.retained_extent()?;
        if extent != 0 {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(extent),
                "relic_query_snapshot",
            )?;
        }
        if !self.copies.is_empty() {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.copies.len().checked_mul(std::mem::size_of::<Uuid>()),
                "relic_not_sellable_projection",
            )?;
            crate::authoring::control::admit(
                crate::authoring::control::Resource::CloneSlots,
                0,
                self.copies.len(),
                "relic_not_sellable_copy",
            )?;
            crate::game::battle::runtime_control::charge_clone(
                self.copies.len(),
                "relic_not_sellable_copy",
            )?;
            crate::authoring::control::charge(self.copies.len() as u64, "relic_not_sellable_copy")?;
        }
        Ok(RelicStateDto {
            not_sellable_copy_uuids: self.copies.keys().copied().collect(),
            copies: self.copies.values().cloned().collect(),
            loadouts: self
                .slots
                .iter()
                .map(|(employee, slots)| RelicLoadoutDto {
                    employee_uuid: *employee,
                    slots: slots.0,
                })
                .collect(),
            resources,
        })
    }
}
