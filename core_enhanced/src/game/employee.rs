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
    employee_psychology::EmployeePsychology,
    growth::GrowthStack,
    skill_fragment::{SkillFragmentInventory, SkillFragmentLoadout},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeInitialLoadout {
    pub equipment_ids: Vec<String>,
    pub baseline_skill_fragment_ids: Vec<SkillFragmentId>,
}

impl Default for EmployeeInitialLoadout {
    fn default() -> Self {
        Self {
            equipment_ids: Vec::new(),
            baseline_skill_fragment_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeRiskDisposition {
    Brave,
    Cautious,
    SurvivalInstinct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeSocialDisposition {
    Obedient,
    Defiant,
    Comradely,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatAptitude {
    Balanced,
    Defensive,
    Offensive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventAptitude {
    Medical,
    Security,
    Analysis,
    Reconnaissance,
    Containment,
    Maintenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeePersonality {
    pub risk: EmployeeRiskDisposition,
    pub social: EmployeeSocialDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeBaseCombatStats {
    pub max_hp: u32,
    pub attack: u32,
    pub defense: i32,
    pub magic_resist: i32,
}

impl Default for EmployeeBaseCombatStats {
    fn default() -> Self {
        Self {
            max_hp: 100,
            attack: 10,
            defense: 0,
            magic_resist: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeProfile {
    pub role: String,
    pub background: String,
    pub personality: EmployeePersonality,
    pub combat_aptitude: CombatAptitude,
    pub event_aptitude: EventAptitude,
    pub base_combat_stats: EmployeeBaseCombatStats,
    pub basic_attack_speed_percent: u32,
}

impl Default for EmployeeProfile {
    fn default() -> Self {
        Self {
            role: "Employee".to_string(),
            background: "Runtime employee".to_string(),
            personality: EmployeePersonality {
                risk: EmployeeRiskDisposition::Cautious,
                social: EmployeeSocialDisposition::Obedient,
            },
            combat_aptitude: CombatAptitude::Balanced,
            event_aptitude: EventAptitude::Maintenance,
            base_combat_stats: EmployeeBaseCombatStats::default(),
            basic_attack_speed_percent: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeDefinition {
    pub employee_definition_id: String,
    pub name: String,
    pub starter_eligible: bool,
    pub initial_loadout: EmployeeInitialLoadout,
    pub profile: EmployeeProfile,
}

pub const MAX_PHYSICAL_DAMAGE: u8 = 6;
pub const MAX_TRAUMA: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalDamageState {
    Normal,
    Minor,
    Major,
    Critical,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentalConditionState {
    Stable,
    Breakdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeActivity {
    Combat,
    Event,
    HighRisk,
    Consumable,
    Loadout,
    PhysicalTreatment,
    TraumaRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmployeeIneligibilityReason {
    Dead,
    CriticalPhysicalDamage,
    Breakdown,
    NoTrauma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmployeeEligibility {
    pub eligible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<EmployeeIneligibilityReason>,
}

impl EmployeeEligibility {
    pub const fn enabled() -> Self {
        Self {
            eligible: true,
            disabled_reason: None,
        }
    }

    pub const fn disabled(reason: EmployeeIneligibilityReason) -> Self {
        Self {
            eligible: false,
            disabled_reason: Some(reason),
        }
    }
}

impl EmployeeIneligibilityReason {
    pub const fn action_disabled_reason(self) -> crate::game::behavior::ActionDisabledReasonCode {
        match self {
            Self::Dead => crate::game::behavior::ActionDisabledReasonCode::EmployeeDead,
            Self::CriticalPhysicalDamage => {
                crate::game::behavior::ActionDisabledReasonCode::CriticalPhysicalDamage
            }
            Self::Breakdown => crate::game::behavior::ActionDisabledReasonCode::Breakdown,
            Self::NoTrauma => crate::game::behavior::ActionDisabledReasonCode::NoTrauma,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeCondition {
    pub physical_damage: u8,
    pub trauma: u32,
}

impl EmployeeCondition {
    pub fn new(physical_damage: u8, trauma: u32) -> Self {
        Self {
            physical_damage: physical_damage.min(MAX_PHYSICAL_DAMAGE),
            trauma: trauma.min(MAX_TRAUMA),
        }
    }

    pub const fn physical_state(self) -> PhysicalDamageState {
        match self.physical_damage {
            0 => PhysicalDamageState::Normal,
            1..=3 => PhysicalDamageState::Minor,
            4 => PhysicalDamageState::Major,
            5 => PhysicalDamageState::Critical,
            _ => PhysicalDamageState::Dead,
        }
    }

    pub const fn mental_state(self) -> MentalConditionState {
        if self.trauma >= MAX_TRAUMA {
            MentalConditionState::Breakdown
        } else {
            MentalConditionState::Stable
        }
    }

    pub const fn is_dead(self) -> bool {
        self.physical_damage >= MAX_PHYSICAL_DAMAGE
    }

    pub const fn is_breakdown(self) -> bool {
        self.trauma >= MAX_TRAUMA
    }

    pub const fn eligibility(self, activity: EmployeeActivity) -> EmployeeEligibility {
        if self.is_dead() {
            return EmployeeEligibility::disabled(EmployeeIneligibilityReason::Dead);
        }
        match activity {
            EmployeeActivity::Combat | EmployeeActivity::Event | EmployeeActivity::HighRisk => {
                if self.physical_damage == MAX_PHYSICAL_DAMAGE - 1 {
                    EmployeeEligibility::disabled(
                        EmployeeIneligibilityReason::CriticalPhysicalDamage,
                    )
                } else if self.is_breakdown() {
                    EmployeeEligibility::disabled(EmployeeIneligibilityReason::Breakdown)
                } else {
                    EmployeeEligibility::enabled()
                }
            }
            EmployeeActivity::PhysicalTreatment => EmployeeEligibility::enabled(),
            EmployeeActivity::TraumaRecovery => {
                if self.trauma == 0 {
                    EmployeeEligibility::disabled(EmployeeIneligibilityReason::NoTrauma)
                } else {
                    EmployeeEligibility::enabled()
                }
            }
            EmployeeActivity::Consumable | EmployeeActivity::Loadout => {
                EmployeeEligibility::enabled()
            }
        }
    }

    pub fn add_trauma(&mut self, amount: u32) {
        self.trauma = self.trauma.saturating_add(amount).min(MAX_TRAUMA);
    }

    pub fn reduce_trauma(&mut self, amount: u32) {
        self.trauma = self.trauma.saturating_sub(amount);
    }

    pub fn reduce_trauma_percent(&mut self, percent: u32) {
        let reduction = self.trauma.saturating_mul(percent) / 100;
        self.reduce_trauma(reduction);
    }

    pub fn reduce_physical_damage(&mut self, amount: u8) {
        if !self.is_dead() {
            self.physical_damage = self.physical_damage.saturating_sub(amount);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncapacitationOutcome {
    pub became_dead: bool,
    pub death_prevented: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeeLoadout {
    pub item_slot: ItemSlot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub uuid: Uuid,
    pub employee_definition_id: String,
    pub name: String,
    pub profile: EmployeeProfile,
    pub level: u32,
    pub experience: u32,
    pub condition: EmployeeCondition,
    pub loadout: EmployeeLoadout,
    pub skill_fragments: SkillFragmentLoadout,
    pub active_consumable_modifier: Option<ActiveConsumableModifier>,
    pub combat_profile: EmployeeCombatProfile,
    pub psychology: EmployeePsychology,
}

impl Employee {
    pub fn new(
        uuid: Uuid,
        employee_definition_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let profile = EmployeeProfile::default();
        let combat_profile = EmployeeCombatProfile {
            battle_profile: UnitCombatProfile::for_employee(
                profile.base_combat_stats.max_hp,
                profile.base_combat_stats.attack,
                profile.base_combat_stats.defense,
                profile.base_combat_stats.magic_resist,
                profile.basic_attack_speed_percent,
            ),
            growth_stacks: GrowthStack::new(),
        };
        Self::with_profile_and_skill_fragments(
            uuid,
            employee_definition_id,
            name,
            profile,
            combat_profile,
            SkillFragmentLoadout::default(),
        )
    }

    pub fn from_definition(uuid: Uuid, definition: &EmployeeDefinition) -> Self {
        let stats = definition.profile.base_combat_stats;
        let combat_profile = EmployeeCombatProfile {
            battle_profile: UnitCombatProfile::for_employee(
                stats.max_hp,
                stats.attack,
                stats.defense,
                stats.magic_resist,
                definition.profile.basic_attack_speed_percent,
            ),
            growth_stacks: GrowthStack::new(),
        };
        Self::with_profile_and_skill_fragments(
            uuid,
            definition.employee_definition_id.clone(),
            definition.name.clone(),
            definition.profile.clone(),
            combat_profile,
            SkillFragmentLoadout::with_baseline_ids(
                definition
                    .initial_loadout
                    .baseline_skill_fragment_ids
                    .clone(),
            ),
        )
    }

    fn with_profile_and_skill_fragments(
        uuid: Uuid,
        employee_definition_id: impl Into<String>,
        name: impl Into<String>,
        profile: EmployeeProfile,
        combat_profile: EmployeeCombatProfile,
        skill_fragments: SkillFragmentLoadout,
    ) -> Self {
        Self {
            uuid,
            employee_definition_id: employee_definition_id.into(),
            name: name.into(),
            profile,
            level: 1,
            experience: 0,
            condition: EmployeeCondition::default(),
            loadout: EmployeeLoadout::default(),
            skill_fragments,
            active_consumable_modifier: None,
            combat_profile,
            psychology: EmployeePsychology::default(),
        }
    }

    pub fn is_combat_eligible(&self) -> bool {
        self.eligibility(EmployeeActivity::Combat).eligible
    }

    pub fn can_receive_consumable_modifier(&self) -> bool {
        self.eligibility(EmployeeActivity::Consumable).eligible
    }

    pub fn eligibility(&self, activity: EmployeeActivity) -> EmployeeEligibility {
        self.condition.eligibility(activity)
    }

    pub fn is_alive(&self) -> bool {
        !self.condition.is_dead()
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
        if !self.is_combat_eligible() {
            return 0;
        }
        let trauma_remaining = MAX_TRAUMA.saturating_sub(self.condition.trauma);
        let trauma_scaled_hp =
            u64::from(battle_max_hp) * u64::from(trauma_remaining) / u64::from(MAX_TRAUMA);
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

    pub fn apply_incapacitation(&mut self, trauma_amount: u32) -> IncapacitationOutcome {
        let was_dead = self.condition.is_dead();
        let trauma_amount = self.mitigated_trauma_amount(trauma_amount);
        self.condition.add_trauma(trauma_amount);
        self.condition.physical_damage = self
            .condition
            .physical_damage
            .saturating_add(1)
            .min(MAX_PHYSICAL_DAMAGE);
        let mut death_prevented = false;
        if self.condition.is_dead() && !was_dead && self.consume_death_prevent_if_available() {
            self.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
            death_prevented = true;
        }
        IncapacitationOutcome {
            became_dead: !was_dead && self.condition.is_dead(),
            death_prevented,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    pub fn combat_eligible_employee_ids(&self) -> Vec<Uuid> {
        let mut ids = self
            .employees
            .values()
            .filter(|employee| employee.is_combat_eligible())
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
        if !employee.eligibility(EmployeeActivity::Loadout).eligible {
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
        if !employee.eligibility(EmployeeActivity::Loadout).eligible {
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
    fn battle_start_hp_uses_only_trauma_remaining_ratio() {
        let mut employee = Employee::new(Uuid::from_u128(1), "trauma_test", "Trauma Test");
        employee.condition.trauma = 25;

        assert_eq!(employee.battle_start_hp_for_max(100), 75);
        assert_eq!(employee.battle_start_hp_for_max(200), 150);
    }

    #[test]
    fn experience_crosses_multiple_levels_and_derives_battle_tier_from_policy() {
        let policy = RunPolicyData::builtin();
        let mut employee = Employee::new(Uuid::from_u128(1), "growth_policy", "Growth Policy");

        employee.add_experience_with_policy(250, &policy);

        assert_eq!(employee.level, 3);
        assert_eq!(employee.experience, 50);
        assert_eq!(employee.battle_tier(&policy), crate::game::enums::Tier::II);
    }

    #[test]
    fn battle_start_hp_keeps_alive_employee_at_minimum_one_hp() {
        let mut employee = Employee::new(Uuid::from_u128(1), "near_breakdown", "Near Breakdown");
        employee.condition.trauma = MAX_TRAUMA - 1;

        assert_eq!(employee.battle_start_hp_for_max(100), 1);
    }

    #[test]
    fn breakdown_employee_has_no_battle_start_hp() {
        let mut employee = Employee::new(Uuid::from_u128(1), "breakdown", "Breakdown");
        employee.condition.trauma = MAX_TRAUMA;

        assert_eq!(employee.battle_start_hp_for_max(100), 0);
    }

    #[test]
    fn consumable_battle_hp_setup_increases_starting_battle_hp_with_cap() {
        let mut employee = Employee::new(Uuid::from_u128(1), "battle_hp_buff", "Battle HP Buff");
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

        // Starting HP is resolved once, after all draft max-HP modifiers.
        assert_eq!(profile.stats.current_health, 100);
    }

    #[test]
    fn rare_consumable_offense_boost_increases_attack() {
        let mut employee = Employee::new(Uuid::from_u128(1), "attack_buff", "Attack Buff");
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
        let mut employee =
            Employee::new(Uuid::from_u128(1), "skill_charge_buff", "Skill Charge Buff");
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
        let mut employee =
            Employee::new(Uuid::from_u128(1), "skill_charge_cap", "Skill Charge Cap");
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
        let mut employee = Employee::new(Uuid::from_u128(1), "defense_buff", "Defense Buff");
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
        let mut employee = Employee::new(Uuid::from_u128(1), "trauma_buff", "Trauma Buff");
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "trauma",
                ConsumableTier::Common,
                ConsumableEffect::TraumaMitigation { percent: 25 },
            ),
        );

        employee.apply_incapacitation(40);

        assert_eq!(employee.condition.trauma, 30);
        assert_eq!(employee.condition.physical_damage, 1);
        assert!(employee.is_alive());
    }

    #[test]
    fn death_prevent_consumes_buff_and_keeps_employee_alive_once() {
        let mut employee = Employee::new(Uuid::from_u128(1), "death_prevent", "Death Prevent");
        employee.condition = EmployeeCondition::new(MAX_PHYSICAL_DAMAGE - 1, MAX_TRAUMA - 5);
        employee.apply_consumable_modifier(
            Uuid::from_u128(2),
            &test_consumable(
                "death",
                ConsumableTier::Critical,
                ConsumableEffect::DeathPrevent,
            ),
        );

        let outcome = employee.apply_incapacitation(40);

        assert!(outcome.death_prevented);
        assert!(!outcome.became_dead);
        assert!(employee.is_alive());
        assert_eq!(employee.condition.physical_damage, MAX_PHYSICAL_DAMAGE - 1);
        assert_eq!(employee.condition.trauma, MAX_TRAUMA);
        assert!(employee.active_consumable_modifier.is_none());

        let second = employee.apply_incapacitation(0);
        assert!(second.became_dead);
        assert!(!second.death_prevented);
        assert_eq!(employee.condition.physical_damage, MAX_PHYSICAL_DAMAGE);
    }

    #[test]
    fn condition_bands_and_activity_eligibility_are_derived() {
        let major = EmployeeCondition::new(4, 99);
        assert_eq!(major.physical_state(), PhysicalDamageState::Major);
        assert_eq!(major.mental_state(), MentalConditionState::Stable);
        assert!(major.eligibility(EmployeeActivity::Combat).eligible);

        let critical = EmployeeCondition::new(5, 0);
        assert_eq!(critical.physical_state(), PhysicalDamageState::Critical);
        assert_eq!(
            critical
                .eligibility(EmployeeActivity::Combat)
                .disabled_reason,
            Some(EmployeeIneligibilityReason::CriticalPhysicalDamage)
        );
        assert!(critical.eligibility(EmployeeActivity::Loadout).eligible);
        assert!(
            critical
                .eligibility(EmployeeActivity::PhysicalTreatment)
                .eligible
        );

        let breakdown = EmployeeCondition::new(0, MAX_TRAUMA);
        assert_eq!(breakdown.mental_state(), MentalConditionState::Breakdown);
        assert_eq!(
            breakdown
                .eligibility(EmployeeActivity::HighRisk)
                .disabled_reason,
            Some(EmployeeIneligibilityReason::Breakdown)
        );
        assert!(breakdown.eligibility(EmployeeActivity::Loadout).eligible);

        let dead = EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, 0);
        assert_eq!(dead.physical_state(), PhysicalDamageState::Dead);
        assert_eq!(
            dead.eligibility(EmployeeActivity::PhysicalTreatment)
                .disabled_reason,
            Some(EmployeeIneligibilityReason::Dead)
        );

        assert_eq!(
            EmployeeCondition::new(5, MAX_TRAUMA)
                .eligibility(EmployeeActivity::Combat)
                .disabled_reason,
            Some(EmployeeIneligibilityReason::CriticalPhysicalDamage)
        );
        assert_eq!(
            EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, MAX_TRAUMA)
                .eligibility(EmployeeActivity::Combat)
                .disabled_reason,
            Some(EmployeeIneligibilityReason::Dead)
        );
    }

    #[test]
    fn repeated_incapacitation_increments_physical_damage_once_each_time() {
        let mut employee = Employee::new(Uuid::from_u128(1), "repeat", "Repeat");

        employee.apply_incapacitation(10);
        employee.apply_incapacitation(10);

        assert_eq!(employee.condition, EmployeeCondition::new(2, 20));
    }

    #[test]
    fn dead_employee_cannot_be_revived_by_physical_treatment() {
        let mut condition = EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, 40);

        condition.reduce_physical_damage(MAX_PHYSICAL_DAMAGE);

        assert_eq!(condition.physical_damage, MAX_PHYSICAL_DAMAGE);
    }

    #[test]
    fn employee_from_definition_preserves_authored_profile_and_combat_values() {
        let definition = EmployeeDefinition {
            employee_definition_id: "authored_employee".to_string(),
            name: "Authored Employee".to_string(),
            starter_eligible: false,
            initial_loadout: EmployeeInitialLoadout::default(),
            profile: EmployeeProfile {
                role: "Defender".to_string(),
                background: "Authored background".to_string(),
                personality: EmployeePersonality {
                    risk: EmployeeRiskDisposition::Brave,
                    social: EmployeeSocialDisposition::Defiant,
                },
                combat_aptitude: CombatAptitude::Defensive,
                event_aptitude: EventAptitude::Security,
                base_combat_stats: EmployeeBaseCombatStats {
                    max_hp: 135,
                    attack: 17,
                    defense: 8,
                    magic_resist: 11,
                },
                basic_attack_speed_percent: 125,
            },
        };

        let employee = Employee::from_definition(Uuid::from_u128(42), &definition);

        assert_eq!(employee.employee_definition_id, "authored_employee");
        assert_eq!(employee.profile, definition.profile);
        assert_eq!(employee.combat_profile.battle_profile.stats.max_health, 135);
        assert_eq!(employee.combat_profile.battle_profile.stats.attack, 17);
        assert_eq!(employee.combat_profile.battle_profile.stats.defense, 8);
        assert_eq!(
            employee.combat_profile.battle_profile.stats.magic_resist,
            11
        );
        assert_eq!(
            employee
                .combat_profile
                .battle_profile
                .basic_attack
                .interval_ms,
            1200
        );
    }

    #[test]
    fn employee_profile_requires_authored_event_aptitude() {
        let error = ron::de::from_str::<EmployeeProfile>(
            r#"(
                role: "Employee",
                background: "Background",
                personality: (risk: Cautious, social: Obedient),
                combat_aptitude: Balanced,
                base_combat_stats: (max_hp: 100, attack: 10, defense: 0, magic_resist: 0),
                basic_attack_speed_percent: 100,
            )"#,
        )
        .expect_err("event_aptitude must be explicitly authored");

        assert!(error.to_string().contains("event_aptitude"));
    }
}
