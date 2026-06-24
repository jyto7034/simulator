use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::game::data::consumable_data::{
    ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata, ConsumableTier,
};
use crate::game::resources::item_slot::ItemSlot;
use crate::game::{
    battle::types::UnitCombatProfile,
    behavior::GameError,
    data::{
        run_policy_data::RunPolicyData,
        skill_fragment_data::{SkillFragmentDatabase, SkillFragmentId},
    },
    employee_trust::EmployeeTrustState,
    growth::GrowthStack,
    skill_fragment::{SkillFragmentInventory, SkillFragmentLoadout},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarterEmployeeLoadout {
    #[serde(default)]
    pub equipment_ids: Vec<String>,
    #[serde(default)]
    pub baseline_skill_fragment_ids: Vec<SkillFragmentId>,
}

impl Default for StarterEmployeeLoadout {
    fn default() -> Self {
        Self {
            equipment_ids: Vec::new(),
            baseline_skill_fragment_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarterEmployeeCandidate {
    pub id: String,
    pub name: String,
    pub role: String,
    pub background: String,
    #[serde(default)]
    pub starter_loadout: StarterEmployeeLoadout,
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
    pub battle_profile: UnitCombatProfile,
    pub growth_stacks: GrowthStack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveConsumableModifier {
    pub source_item_uuid: Uuid,
    pub definition_id: String,
    pub name: String,
    pub tier: ConsumableTier,
    pub duration_policy: ConsumableDurationPolicy,
    pub remaining_combat_nodes: u32,
    pub effect: ConsumableEffect,
}

impl ActiveConsumableModifier {
    pub fn from_consumable(source_item_uuid: Uuid, meta: &ConsumableMetadata) -> Self {
        Self {
            source_item_uuid,
            definition_id: meta.id.clone(),
            name: meta.name.clone(),
            tier: meta.tier,
            duration_policy: meta.duration_policy,
            remaining_combat_nodes: meta.duration_policy.initial_remaining_combat_nodes(),
            effect: meta.effect.clone(),
        }
    }

    pub fn decrement_after_combat_node(&mut self) -> bool {
        self.remaining_combat_nodes = self.remaining_combat_nodes.saturating_sub(1);
        self.remaining_combat_nodes == 0
    }
}

impl Default for EmployeeCombatProfile {
    fn default() -> Self {
        Self {
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
    pub active_consumable_modifier: Option<ActiveConsumableModifier>,
    pub combat_profile: EmployeeCombatProfile,
    pub trust: EmployeeTrustState,
}

impl Employee {
    pub const TRAUMA_DEATH_THRESHOLD: u32 = 100;
    const BATTLE_START_BASE_CONDITION_PERCENT: u32 = 60;
    const BATTLE_START_RUN_HP_WEIGHT_PERCENT: u32 = 40;

    pub fn new(uuid: Uuid, name: impl Into<String>) -> Self {
        let combat_profile = EmployeeCombatProfile::default();
        Self::with_combat_profile(uuid, name, combat_profile)
    }

    pub fn from_starter_candidate(uuid: Uuid, candidate: &StarterEmployeeCandidate) -> Self {
        Self::with_combat_profile_and_skill_fragments(
            uuid,
            candidate.name.clone(),
            EmployeeCombatProfile::default(),
            SkillFragmentLoadout::with_baseline_ids(
                candidate
                    .starter_loadout
                    .baseline_skill_fragment_ids
                    .clone(),
            ),
        )
    }

    fn with_combat_profile(
        uuid: Uuid,
        name: impl Into<String>,
        combat_profile: EmployeeCombatProfile,
    ) -> Self {
        Self::with_combat_profile_and_skill_fragments(
            uuid,
            name,
            combat_profile,
            SkillFragmentLoadout::default(),
        )
    }

    fn with_combat_profile_and_skill_fragments(
        uuid: Uuid,
        name: impl Into<String>,
        combat_profile: EmployeeCombatProfile,
        skill_fragments: SkillFragmentLoadout,
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
            skill_fragments,
            active_consumable_modifier: None,
            combat_profile,
            trust: EmployeeTrustState::new_for_employee(uuid),
        }
    }

    pub fn is_available_for_combat(&self) -> bool {
        self.life_state == EmployeeLifeState::Alive
            && self.availability == EmployeeAvailability::Available
            && !self.health.is_depleted()
    }

    pub fn can_receive_consumable_modifier(&self) -> bool {
        self.life_state == EmployeeLifeState::Alive
    }

    pub fn battle_tier(&self, policy: &RunPolicyData) -> crate::game::enums::Tier {
        policy.battle_tier_for_level(self.level)
    }

    pub fn combat_profile_for_battle(
        &self,
        skill_fragments: &SkillFragmentDatabase,
        fragment_inventory: &SkillFragmentInventory,
    ) -> Result<UnitCombatProfile, GameError> {
        crate::game::battle::stat_pipeline::employee_combat_profile_for_battle(
            self,
            skill_fragments,
            fragment_inventory,
        )
    }

    pub fn battle_start_hp_for_max(&self, battle_max_hp: u32) -> u32 {
        let battle_max_hp = battle_max_hp.max(1);
        let run_max_hp = self.health.max_hp.max(1);
        let run_current_hp = self.health.current_hp.min(run_max_hp);
        if run_current_hp == 0 || self.trauma >= Self::TRAUMA_DEATH_THRESHOLD {
            return 0;
        }

        let run_hp_percent = u64::from(run_current_hp) * 100 / u64::from(run_max_hp);
        let battle_condition_percent = u64::from(Self::BATTLE_START_BASE_CONDITION_PERCENT)
            + run_hp_percent * u64::from(Self::BATTLE_START_RUN_HP_WEIGHT_PERCENT) / 100;
        let condition_scaled_hp = u64::from(battle_max_hp) * battle_condition_percent / 100;
        let trauma_remaining = Self::TRAUMA_DEATH_THRESHOLD.saturating_sub(self.trauma);
        let trauma_scaled_hp = condition_scaled_hp * u64::from(trauma_remaining)
            / u64::from(Self::TRAUMA_DEATH_THRESHOLD);

        (trauma_scaled_hp as u32).max(1).min(battle_max_hp)
    }

    pub fn apply_consumable_modifier(
        &mut self,
        item_uuid: Uuid,
        meta: &ConsumableMetadata,
    ) -> Option<ActiveConsumableModifier> {
        let next = ActiveConsumableModifier::from_consumable(item_uuid, meta);
        self.active_consumable_modifier.replace(next)
    }

    pub fn decrement_consumable_after_combat_node(&mut self) {
        let should_remove = self
            .active_consumable_modifier
            .as_mut()
            .is_some_and(ActiveConsumableModifier::decrement_after_combat_node);
        if should_remove {
            self.active_consumable_modifier = None;
        }
    }

    pub fn deployment_cost_after_consumable(&self, base_cost: u32) -> u32 {
        let Some(modifier) = self.active_consumable_modifier.as_ref() else {
            return base_cost;
        };
        match modifier.effect {
            ConsumableEffect::DeployCostReduction { percent } => {
                base_cost.saturating_sub(percent_amount_ceil(base_cost, percent))
            }
            _ => base_cost,
        }
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

    pub fn add_experience_with_policy(&mut self, amount: u32, policy: &RunPolicyData) {
        self.experience = self.experience.saturating_add(amount);
        while self.experience >= policy.xp_required_for_next_level(self.level) {
            self.experience -= policy.xp_required_for_next_level(self.level);
            self.level = self.level.saturating_add(1);
        }
    }

    pub fn apply_incapacitation(
        &mut self,
        trauma_amount: u32,
        run_hp_loss_percent: u32,
        injury: EmployeeInjury,
    ) {
        let trauma_amount = self.mitigated_trauma_amount(trauma_amount);
        let run_hp_loss_percent = self.mitigated_run_hp_loss_percent(run_hp_loss_percent);
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
            if self.consume_death_prevent_if_available() {
                self.trauma = self.trauma.min(Self::TRAUMA_DEATH_THRESHOLD - 1);
                if self.health.is_depleted() {
                    self.health.set_current_hp(1);
                }
                self.life_state = EmployeeLifeState::Alive;
                self.availability = EmployeeAvailability::Available;
                return;
            }
            self.life_state = EmployeeLifeState::Dead;
            self.availability = EmployeeAvailability::Unavailable;
        }
    }

    fn mitigated_trauma_amount(&self, amount: u32) -> u32 {
        let Some(modifier) = self.active_consumable_modifier.as_ref() else {
            return amount;
        };
        match modifier.effect {
            ConsumableEffect::TraumaMitigation { percent } => {
                amount.saturating_sub(percent_amount_ceil(amount, percent))
            }
            _ => amount,
        }
    }

    fn mitigated_run_hp_loss_percent(&self, percent: u32) -> u32 {
        let Some(modifier) = self.active_consumable_modifier.as_ref() else {
            return percent;
        };
        match modifier.effect {
            ConsumableEffect::RunHpLossMitigation {
                percent: mitigation_percent,
            } => percent.saturating_sub(percent_amount_ceil(percent, mitigation_percent)),
            _ => percent,
        }
    }

    fn consume_death_prevent_if_available(&mut self) -> bool {
        if self
            .active_consumable_modifier
            .as_ref()
            .is_some_and(|modifier| matches!(modifier.effect, ConsumableEffect::DeathPrevent))
        {
            self.active_consumable_modifier = None;
            true
        } else {
            false
        }
    }
}

pub(crate) fn percent_amount_ceil(value: u32, percent: u32) -> u32 {
    if value == 0 || percent == 0 {
        return 0;
    }
    let numerator = u64::from(value) * u64::from(percent);
    numerator.div_ceil(100) as u32
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        data::consumable_data::{
            ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata, ConsumableTargetPolicy,
            ConsumableTier,
        },
        data::skill_fragment_data::SkillFragmentDatabase,
        skill_fragment::SkillFragmentInventory,
    };

    fn test_consumable(
        id: &str,
        tier: ConsumableTier,
        effect: ConsumableEffect,
    ) -> ConsumableMetadata {
        ConsumableMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(0xCAFE),
            name: id.to_string(),
            description: "test consumable".to_string(),
            tier,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 1,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy: ConsumableDurationPolicy::NextCombatNode,
            effect,
            live_pool: true,
        }
    }

    #[test]
    fn battle_start_hp_scales_run_hp_by_trauma_remaining_ratio() {
        let mut employee = Employee::new(Uuid::from_u128(1), "trauma_test");
        employee.health.set_current_hp(80);
        employee.trauma = 25;

        assert_eq!(employee.battle_start_hp_for_max(100), 69);
        assert_eq!(employee.battle_start_hp_for_max(200), 138);
    }

    #[test]
    fn battle_start_hp_keeps_run_hp_damage_as_soft_condition_loss() {
        let mut employee = Employee::new(Uuid::from_u128(1), "run_hp_damage");
        employee.health.set_current_hp(20);
        employee.trauma = 0;

        assert_eq!(employee.battle_start_hp_for_max(100), 68);
    }

    #[test]
    fn battle_start_hp_keeps_alive_employee_at_minimum_one_hp() {
        let mut employee = Employee::new(Uuid::from_u128(1), "near_breakdown");
        employee.health.set_current_hp(1);
        employee.trauma = Employee::TRAUMA_DEATH_THRESHOLD - 1;

        assert_eq!(employee.battle_start_hp_for_max(100), 1);
    }

    #[test]
    fn combat_profile_for_battle_uses_trauma_adjusted_battle_start_hp() {
        let mut employee = Employee::new(Uuid::from_u128(1), "combat_profile");
        employee.health.set_current_hp(80);
        employee.trauma = 50;

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(profile.stats.max_health, 100);
        assert_eq!(profile.stats.current_health, 46);
    }

    #[test]
    fn consumable_battle_hp_setup_increases_starting_battle_hp_with_cap() {
        let mut employee = Employee::new(Uuid::from_u128(1), "battle_hp_buff");
        employee.health.set_current_hp(20);
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "battle_hp",
                ConsumableTier::Common,
                ConsumableEffect::BattleHpSetup { bonus_percent: 20 },
            ),
        );

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(profile.stats.current_health, 88);
    }

    #[test]
    fn rare_consumable_offense_boost_increases_attack() {
        let mut employee = Employee::new(Uuid::from_u128(1), "attack_buff");
        let baseline = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "attack",
                ConsumableTier::Rare,
                ConsumableEffect::OffenseBoost {
                    attack_bonus_percent: 20,
                },
            ),
        );

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(
            profile.stats.attack,
            baseline
                .stats
                .attack
                .saturating_add(percent_amount_ceil(baseline.stats.attack, 20))
        );
    }

    #[test]
    fn consumable_initial_skill_charge_increases_starting_resonance() {
        let mut employee = Employee::new(Uuid::from_u128(1), "skill_charge_buff");
        employee.combat_profile.battle_profile.resonance.start = 10;
        employee.combat_profile.battle_profile.resonance.max = 100;
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "skill_charge",
                ConsumableTier::Rare,
                ConsumableEffect::InitialSkillCharge { percent: 40 },
            ),
        );

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(profile.resonance.start, 50);
    }

    #[test]
    fn consumable_initial_skill_charge_caps_at_resonance_max() {
        let mut employee = Employee::new(Uuid::from_u128(1), "skill_charge_cap");
        employee.combat_profile.battle_profile.resonance.start = 80;
        employee.combat_profile.battle_profile.resonance.max = 100;
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "skill_charge_cap",
                ConsumableTier::Critical,
                ConsumableEffect::InitialSkillCharge { percent: 100 },
            ),
        );

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(profile.resonance.start, 100);
    }

    #[test]
    fn consumable_defense_mitigation_reduces_incoming_battle_damage() {
        let mut employee = Employee::new(Uuid::from_u128(1), "defense_buff");
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "defense",
                ConsumableTier::Uncommon,
                ConsumableEffect::DefenseMitigation { percent: 30 },
            ),
        );

        let profile = employee
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new(),
            )
            .unwrap();

        assert_eq!(
            profile.incoming_damage_modifiers.damage_reduction_percent,
            30
        );
    }

    #[test]
    fn consumable_trauma_mitigation_reduces_incapacitation_trauma() {
        let mut employee = Employee::new(Uuid::from_u128(1), "trauma_buff");
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "trauma",
                ConsumableTier::Common,
                ConsumableEffect::TraumaMitigation { percent: 25 },
            ),
        );

        employee.apply_incapacitation(
            40,
            25,
            EmployeeInjury {
                id: "test".to_string(),
                severity: 1,
            },
        );

        assert_eq!(employee.trauma, 30);
        assert_eq!(employee.life_state, EmployeeLifeState::Alive);
    }

    #[test]
    fn death_prevent_consumes_buff_and_keeps_employee_alive_once() {
        let mut employee = Employee::new(Uuid::from_u128(1), "death_prevent");
        employee.trauma = Employee::TRAUMA_DEATH_THRESHOLD - 5;
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "death",
                ConsumableTier::Critical,
                ConsumableEffect::DeathPrevent,
            ),
        );

        employee.apply_incapacitation(
            40,
            25,
            EmployeeInjury {
                id: "test".to_string(),
                severity: 1,
            },
        );

        assert_eq!(employee.life_state, EmployeeLifeState::Alive);
        assert_eq!(employee.trauma, Employee::TRAUMA_DEATH_THRESHOLD - 1);
        assert!(employee.active_consumable_modifier.is_none());
    }
}
