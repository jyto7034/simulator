//! Exact-copy loadout transitions. All fallible checks precede the paired writes.
use super::*;
use crate::game::data::skill_fragment_data::SkillFragmentEquipLimit;
use crate::game::employee::{EmployeeActivity, EmployeeRoster};

impl SkillFragmentInventory {
    pub(crate) fn validate_bindings(
        &self,
        roster: &EmployeeRoster,
        database: &SkillFragmentDatabase,
    ) -> Result<(), GameError> {
        let mut exclusive = BTreeSet::new();
        for copy in self.copies.values() {
            if let Some(owner) = copy.equipped_to {
                let employee = roster.get(&owner).ok_or(GameError::InvalidAction)?;
                if employee.skill_fragments.active_copy_uuid() != Some(copy.copy_uuid) {
                    return Err(GameError::InvalidAction);
                }
                let (_, meta, _) = self.resolve(&copy.copy_uuid, database)?;
                if meta.equip_limit == SkillFragmentEquipLimit::GlobalExclusive
                    && !exclusive.insert(&copy.definition_id)
                {
                    return Err(GameError::InvalidAction);
                }
            }
        }
        for employee in roster.iter() {
            if let Some(id) = employee.skill_fragments.active_copy_uuid() {
                if self.get(&id).ok_or(GameError::InvalidAction)?.equipped_to != Some(employee.uuid)
                {
                    return Err(GameError::InvalidAction);
                }
                self.resolve(&id, database)?;
            }
        }
        Ok(())
    }
    pub(crate) fn equip_copy(
        &mut self,
        roster: &mut EmployeeRoster,
        database: &SkillFragmentDatabase,
        copy_uuid: Uuid,
        owner: Uuid,
        transfer: bool,
    ) -> Result<bool, GameError> {
        self.validate_bindings(roster, database)?;
        let (copy, meta, _) = self.resolve(&copy_uuid, database)?;
        let previous = copy.equipped_to;
        let target = roster.get(&owner).ok_or(GameError::UnitNotFound)?;
        if !target.eligibility(EmployeeActivity::Loadout).eligible {
            return Err(GameError::InvalidAction);
        }
        if target.skill_fragments.active_copy_uuid() == Some(copy_uuid) {
            return Ok(false);
        }
        if target.skill_fragments.active_copy_uuid().is_some()
            || (!transfer && previous.is_some())
            || (transfer && previous.is_none())
        {
            return Err(GameError::InvalidAction);
        }
        if meta.equip_limit == SkillFragmentEquipLimit::GlobalExclusive
            && self.copies.values().any(|c| {
                c.copy_uuid != copy_uuid
                    && c.definition_id == copy.definition_id
                    && c.equipped_to.is_some()
            })
        {
            return Err(GameError::InvalidAction);
        }
        if let Some(previous) = previous {
            let employee = roster.get(&previous).ok_or(GameError::InvalidAction)?;
            if !employee.eligibility(EmployeeActivity::Loadout).eligible {
                return Err(GameError::InvalidAction);
            }
        }
        if let Some(previous) = previous {
            roster
                .get_mut(&previous)
                .expect("validated owner")
                .skill_fragments
                .set_copy(None);
        }
        roster
            .get_mut(&owner)
            .expect("validated owner")
            .skill_fragments
            .set_copy(Some(copy_uuid));
        self.copies
            .get_mut(&copy_uuid)
            .expect("validated copy")
            .equipped_to = Some(owner);
        Ok(true)
    }
    pub(crate) fn unequip_copy(
        &mut self,
        roster: &mut EmployeeRoster,
        database: &SkillFragmentDatabase,
        copy_uuid: Uuid,
        owner: Uuid,
    ) -> Result<(), GameError> {
        self.validate_bindings(roster, database)?;
        let copy = self.get(&copy_uuid).ok_or(GameError::InvalidAction)?;
        if copy.equipped_to != Some(owner) {
            return Err(GameError::InvalidAction);
        }
        roster
            .get_mut(&owner)
            .expect("validated reverse binding")
            .skill_fragments
            .set_copy(None);
        self.copies
            .get_mut(&copy_uuid)
            .expect("validated copy")
            .equipped_to = None;
        Ok(())
    }
    pub fn any_form_unlocked(&self, definition: &SkillFragmentId, form: &str) -> bool {
        self.copies
            .values()
            .any(|c| &c.definition_id == definition && c.unlocked_forms.contains(form))
    }
}
