use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::game::enums::Tier;
use crate::game::resources::item_slot::ItemSlot;
use crate::game::{
    battle::types::UnitCombatProfile,
    behavior::GameError,
    data::skill_fragment_data::{SkillFragmentDatabase, SkillFragmentId},
    employee_trust::EmployeeTrustState,
    growth::{GrowthId, GrowthStack},
    skill_fragment::{SkillFragmentInventory, SkillFragmentLoadout},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeGrade {
    Junior,
    Regular,
    Senior,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarterEmployeeCandidate {
    pub id: String,
    pub name: String,
    pub grade: EmployeeGrade,
    pub role: String,
    pub background: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeLifeState {
    Alive,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmployeeAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeInjury {
    pub id: String,
    pub severity: u8,
}

#[derive(Debug, Clone, Default)]
pub struct EmployeeLoadout {
    pub item_slot: ItemSlot,
}

#[derive(Debug, Clone)]
pub struct EmployeeCombatProfile {
    pub grade: EmployeeGrade,
    pub battle_profile: UnitCombatProfile,
    pub growth_stacks: GrowthStack,
}

impl Default for EmployeeCombatProfile {
    fn default() -> Self {
        Self {
            grade: EmployeeGrade::Junior,
            battle_profile: UnitCombatProfile::employee_default(),
            growth_stacks: GrowthStack::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeHealthState {
    pub current_hp: u32,
    pub max_hp: u32,
}

impl EmployeeHealthState {
    pub fn new(max_hp: u32) -> Self {
        let max_hp = max_hp.max(1);
        Self {
            current_hp: max_hp,
            max_hp,
        }
    }

    pub fn restore_hp_percent(&mut self, percent: u32) {
        let amount = self.max_hp.saturating_mul(percent) / 100;
        self.current_hp = self.current_hp.saturating_add(amount).min(self.max_hp);
    }

    pub fn set_current_hp(&mut self, current_hp: u32) {
        self.current_hp = current_hp.min(self.max_hp);
    }

    pub fn is_depleted(&self) -> bool {
        self.current_hp == 0
    }
}

#[derive(Debug, Clone)]
pub struct Employee {
    pub uuid: Uuid,
    pub name: String,
    pub level: u32,
    pub experience: u32,
    pub life_state: EmployeeLifeState,
    pub availability: EmployeeAvailability,
    pub trauma: u32,
    pub injuries: Vec<EmployeeInjury>,
    pub health: EmployeeHealthState,
    pub loadout: EmployeeLoadout,
    pub skill_fragments: SkillFragmentLoadout,
    pub combat_profile: EmployeeCombatProfile,
    pub trust: EmployeeTrustState,
}

impl Employee {
    pub const TRAUMA_DEATH_THRESHOLD: u32 = 100;

    pub fn new(uuid: Uuid, name: impl Into<String>) -> Self {
        let combat_profile = EmployeeCombatProfile::default();
        Self::with_combat_profile(uuid, name, combat_profile)
    }

    pub fn from_starter_candidate(uuid: Uuid, candidate: &StarterEmployeeCandidate) -> Self {
        let combat_profile = EmployeeCombatProfile {
            grade: candidate.grade.clone(),
            ..EmployeeCombatProfile::default()
        };
        Self::with_combat_profile(uuid, candidate.name.clone(), combat_profile)
    }

    fn with_combat_profile(
        uuid: Uuid,
        name: impl Into<String>,
        combat_profile: EmployeeCombatProfile,
    ) -> Self {
        let health = EmployeeHealthState::new(combat_profile.battle_profile.stats.max_health);
        Self {
            uuid,
            name: name.into(),
            level: 1,
            experience: 0,
            life_state: EmployeeLifeState::Alive,
            availability: EmployeeAvailability::Available,
            trauma: 0,
            injuries: Vec::new(),
            health,
            loadout: EmployeeLoadout::default(),
            skill_fragments: SkillFragmentLoadout::starter(),
            combat_profile,
            trust: EmployeeTrustState::new_for_employee(uuid),
        }
    }

    pub fn is_available_for_combat(&self) -> bool {
        self.life_state == EmployeeLifeState::Alive
            && self.availability == EmployeeAvailability::Available
            && !self.health.is_depleted()
    }

    pub fn battle_tier(&self) -> Tier {
        match self.combat_profile.grade {
            EmployeeGrade::Junior => Tier::I,
            EmployeeGrade::Regular => Tier::II,
            EmployeeGrade::Senior => Tier::III,
        }
    }

    pub fn combat_profile_for_battle(
        &self,
        skill_fragments: &SkillFragmentDatabase,
        fragment_inventory: &SkillFragmentInventory,
    ) -> Result<UnitCombatProfile, GameError> {
        let mut profile = self.skill_fragments.apply_to_profile(
            skill_fragments,
            fragment_inventory,
            &self.combat_profile.battle_profile,
        )?;
        profile.stats.current_health = self.health.current_hp.min(profile.stats.max_health);
        Ok(profile)
    }

    pub fn equip_skill_fragment(
        &mut self,
        inventory: &SkillFragmentInventory,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        let mut next_loadout = self.skill_fragments.clone();
        next_loadout.equip(inventory, database, fragment_id)?;
        next_loadout.apply_to_profile(database, inventory, &self.combat_profile.battle_profile)?;
        self.skill_fragments = next_loadout;
        Ok(())
    }

    pub fn unequip_skill_fragment(
        &mut self,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        let mut next_loadout = self.skill_fragments.clone();
        next_loadout.unequip(fragment_id)?;
        next_loadout.apply_to_profile(
            database,
            &SkillFragmentInventory::new(),
            &self.combat_profile.battle_profile,
        )?;
        self.skill_fragments = next_loadout;
        Ok(())
    }

    pub fn add_experience(&mut self, amount: u32) {
        self.experience = self.experience.saturating_add(amount);
        while self.experience >= self.experience_required_for_next_level() {
            self.experience -= self.experience_required_for_next_level();
            self.level = self.level.saturating_add(1);
            self.combat_profile
                .growth_stacks
                .add(GrowthId::PveWinStack, 1);
            self.combat_profile.grade = match self.level {
                0..=2 => EmployeeGrade::Junior,
                3..=5 => EmployeeGrade::Regular,
                _ => EmployeeGrade::Senior,
            };
        }
    }

    pub fn apply_incapacitation(
        &mut self,
        trauma_amount: u32,
        run_hp_loss_percent: u32,
        injury: EmployeeInjury,
    ) {
        let loss = self
            .health
            .max_hp
            .saturating_mul(run_hp_loss_percent)
            .saturating_div(100)
            .max(1);
        self.health
            .set_current_hp(self.health.current_hp.saturating_sub(loss));
        self.trauma = self.trauma.saturating_add(trauma_amount);
        self.injuries.push(injury);
        if self.trauma >= Self::TRAUMA_DEATH_THRESHOLD || self.health.is_depleted() {
            self.life_state = EmployeeLifeState::Dead;
            self.availability = EmployeeAvailability::Unavailable;
        }
    }

    fn experience_required_for_next_level(&self) -> u32 {
        100
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmployeeRoster {
    employees: HashMap<Uuid, Employee>,
}

impl EmployeeRoster {
    pub fn new() -> Self {
        Self {
            employees: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.employees.clear();
    }

    pub fn add(&mut self, employee: Employee) {
        self.employees.insert(employee.uuid, employee);
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Employee> {
        self.employees.get(uuid)
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut Employee> {
        self.employees.get_mut(uuid)
    }

    pub fn contains(&self, uuid: &Uuid) -> bool {
        self.employees.contains_key(uuid)
    }

    pub fn len(&self) -> usize {
        self.employees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.employees.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Employee> {
        self.employees.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Employee> {
        self.employees.values_mut()
    }

    pub fn available_employee_ids(&self) -> Vec<Uuid> {
        let mut ids = self
            .employees
            .values()
            .filter(|employee| employee.is_available_for_combat())
            .map(|employee| employee.uuid)
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn equip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        inventory: &SkillFragmentInventory,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
    ) -> Result<&Employee, GameError> {
        let employee = self
            .employees
            .get_mut(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee.is_available_for_combat() {
            return Err(GameError::InvalidAction);
        }
        employee.equip_skill_fragment(inventory, database, fragment_id)?;
        Ok(employee)
    }

    pub fn unequip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
    ) -> Result<&Employee, GameError> {
        let employee = self
            .employees
            .get_mut(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee.is_available_for_combat() {
            return Err(GameError::InvalidAction);
        }
        employee.unequip_skill_fragment(database, fragment_id)?;
        Ok(employee)
    }
}
