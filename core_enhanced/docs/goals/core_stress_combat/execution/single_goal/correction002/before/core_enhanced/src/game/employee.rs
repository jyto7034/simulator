use crate::owned_collections::OwnedHashMap;
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
    growth::GrowthStack,
    skill_fragment::{SkillFragmentInventory, SkillFragmentLoadout},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeInitialLoadout {
    pub equipment_ids: Vec<String>,
    pub baseline_skill_fragment_ids: Vec<SkillFragmentId>,
    #[serde(default)]
    pub active_skill_fragment_id: Option<SkillFragmentId>,
}

impl Default for EmployeeInitialLoadout {
    fn default() -> Self {
        Self {
            equipment_ids: Vec::new(),
            baseline_skill_fragment_ids: Vec::new(),
            active_skill_fragment_id: None,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EmployeeRole {
    Dealer,
    Tank,
    Healer,
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
    pub role: EmployeeRole,
    pub employee_definition_id: String,
    pub name: String,
    pub starter_eligible: bool,
    pub initial_loadout: EmployeeInitialLoadout,
    pub profile: EmployeeProfile,
}

pub const MAX_PHYSICAL_DAMAGE: u8 = 6;
pub const MAX_EMPLOYEE_ROSTER_SIZE: usize = 10;
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
#[serde(deny_unknown_fields)]
pub struct EmployeeCombatProfile {
    #[serde(with = "employee_profile_authoring")]
    pub battle_profile: UnitCombatProfile,
    pub growth_stacks: GrowthStack,
}

// The autonomous resonance authoring contract belongs to opponents. Persistent
// employee profiles contain no resource overrides; the equipped form and entry
// modifiers produce those values when entering battle.
mod employee_profile_authoring {
    use super::*;
    use crate::game::{
        ability::{SkillActivationMode, SkillId},
        battle::{
            damage::DamageModifiers,
            types::{DeploymentAffinity, MobilityKind, UnitTargetTrait},
        },
        data::{
            abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
            equipment_data::WeaponCombatProfile,
        },
        skill_fragment::FragmentSourceSnapshot,
        stats::UnitStats,
    };

    fn base_resonance() -> ResonanceDef {
        UnitCombatProfile::employee_base_resonance()
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "UnitCombatProfile", deny_unknown_fields)]
    struct Definition {
        stats: UnitStats,
        basic_attack: BasicAttackDef,
        basic_attack_speed_percent: u32,
        weapon_profile: Option<WeaponCombatProfile>,
        movement: MovementDef,
        #[serde(skip, default = "base_resonance")]
        resonance: ResonanceDef,
        skill_id: Option<SkillId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fragment_source: Option<FragmentSourceSnapshot>,
        skill_activation_mode: SkillActivationMode,
        deployment_affinity: DeploymentAffinity,
        block_capacity: u32,
        block_radius_units: f32,
        blockable: bool,
        mobility_kind: MobilityKind,
        target_traits: Vec<UnitTargetTrait>,
        incoming_damage_modifiers: DamageModifiers,
    }

    pub fn serialize<S: serde::Serializer>(
        value: &UnitCombatProfile,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if !value.has_employee_base_resonance() {
            return Err(serde::ser::Error::custom(
                "employee resonance overrides are not authorable",
            ));
        }
        Definition::serialize(value, serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<UnitCombatProfile, D::Error> {
        Definition::deserialize(deserializer)
    }
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BreakdownCounter {
    #[default]
    Inactive,
    Active {
        remaining_cost: u32,
        entered_node: Option<crate::game::map::MapNodeId>,
    },
}
crate::authoring::clone_shape::copy_shape!(BreakdownCounter);
impl BreakdownCounter {
    pub fn remaining_cost(self) -> Option<u32> {
        match self {
            Self::Inactive => None,
            Self::Active {
                remaining_cost, ..
            } => Some(remaining_cost),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Employee {
    pub role: EmployeeRole,
    pub uuid: Uuid,
    pub employee_definition_id: String,
    pub name: String,
    pub profile: EmployeeProfile,
    pub level: u32,
    pub experience: u32,
    pub condition: EmployeeCondition,
    pub breakdown: BreakdownCounter,
    pub loadout: EmployeeLoadout,
    pub skill_fragments: SkillFragmentLoadout,
    pub active_consumable_modifier: Option<ActiveConsumableModifier>,
    pub combat_profile: EmployeeCombatProfile,
}

crate::authoring::clone_shape::copy_shape!(EmployeeRole, EmployeePersonality, EmployeeRiskDisposition, EmployeeSocialDisposition, CombatAptitude, EventAptitude, EmployeeBaseCombatStats);
crate::authoring::clone_shape::struct_shape!(EmployeeInitialLoadout { equipment_ids, baseline_skill_fragment_ids, active_skill_fragment_id });
crate::authoring::clone_shape::struct_shape!(Employee {
    role,
    uuid,
    employee_definition_id,
    name,
    profile,
    level,
    experience,
    condition,
    breakdown,
    loadout,
    skill_fragments,
    active_consumable_modifier,
    combat_profile,
});
crate::authoring::clone_shape::struct_shape!(EmployeeLoadout { item_slot });
crate::authoring::clone_shape::struct_shape!(EmployeeCombatProfile {
    battle_profile,
    growth_stacks
});
crate::authoring::clone_shape::struct_shape!(EmployeeRoster { employees });

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
        let mut employee = Self::with_profile_and_skill_fragments(
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
        );
        employee.role = definition.role;
        employee
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
            role: EmployeeRole::Dealer,
            employee_definition_id: employee_definition_id.into(),
            name: name.into(),
            profile,
            level: 1,
            experience: 0,
            condition: EmployeeCondition::default(),
            breakdown: BreakdownCounter::Inactive,
            loadout: EmployeeLoadout::default(),
            skill_fragments,
            active_consumable_modifier: None,
            combat_profile,
        }
    }

    pub fn reconcile_breakdown(&mut self, limit: u32, node: Option<crate::game::map::MapNodeId>) {
        if !self.is_alive() || !self.condition.is_breakdown() {
            self.breakdown = BreakdownCounter::Inactive;
        } else if matches!(self.breakdown, BreakdownCounter::Inactive) {
            self.breakdown = BreakdownCounter::Active {
                remaining_cost: limit,
                entered_node: node,
            };
        }
    }
    /// Consume a new node's regulated entry cost, even when deterrence is already zero.
    /// The expedition receipt owner calls this once; completing or reentering a node
    /// never advances the grace period.
    pub fn consume_breakdown_entry_cost(
        &mut self,
        limit: u32,
        cost: u32,
        node: Option<crate::game::map::MapNodeId>,
    ) {
        self.reconcile_breakdown(limit, node);
        if let BreakdownCounter::Active { remaining_cost, .. } = &mut self.breakdown {
            *remaining_cost = remaining_cost.saturating_sub(cost);
            if *remaining_cost == 0 {
                // Noncombat collapse: no incapacitation trauma, injury roll or DeathPrevent.
                self.condition.physical_damage = MAX_PHYSICAL_DAMAGE;
                self.breakdown = BreakdownCounter::Inactive;
            }
        }
    }

    pub fn is_combat_eligible(&self) -> bool {
        self.eligibility(EmployeeActivity::Combat).eligible
    }

    pub fn has_equipped_weapon(&self) -> bool {
        self.loadout
            .item_slot
            .contains_equipment_type(crate::game::data::equipment_data::EquipmentType::Weapon)
    }

    pub fn battle_deployment_disabled_reason(
        &self,
    ) -> Option<crate::game::behavior::ActionDisabledReasonCode> {
        if let Some(reason) = self.eligibility(EmployeeActivity::Combat).disabled_reason {
            return Some(reason.action_disabled_reason());
        }
        if !self.has_equipped_weapon() {
            return Some(crate::game::behavior::ActionDisabledReasonCode::WeaponRequired);
        }
        None
    }

    pub fn is_battle_deployable(&self) -> bool {
        self.battle_deployment_disabled_reason().is_none()
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
    let numerator = u128::from(value) * u128::from(percent);
    numerator.div_ceil(100).min(u128::from(u32::MAX)) as u32
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeeRoster {
    employees: OwnedHashMap<Uuid, Employee>,
}

impl EmployeeRoster {
    pub fn new() -> Self {
        Self {
            employees: OwnedHashMap::new(),
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

    pub(crate) fn validate_checkpoint_identities(&self) -> bool {
        self.employees
            .iter()
            .all(|(employee_uuid, employee)| *employee_uuid == employee.uuid)
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

    #[test]
    fn percent_amount_ceil_preserves_the_full_u32_domain() {
        for (value, percent, expected) in [
            (0, u32::MAX, 0),
            (1, 0, 0),
            (1, 1, 1),
            (u32::MAX, 1, 42_949_673),
            (u32::MAX, 100, u32::MAX),
            (u32::MAX, u32::MAX, u32::MAX),
        ] {
            assert_eq!(
                percent_amount_ceil(value, percent),
                expected,
                "value={value} percent={percent}"
            );
        }
    }

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
    fn u05_employee_authoring_rejects_removed_resource_overrides() {
        let employee = Employee::new(Uuid::from_u128(1), "base", "Base");
        let original = serde_json::to_value(&employee).unwrap();
        assert!(original["combat_profile"]["battle_profile"]
            .get("resonance")
            .is_none());
        let restored: Employee = serde_json::from_value(original.clone()).unwrap();
        assert!(restored
            .combat_profile
            .battle_profile
            .has_employee_base_resonance());
        for value in [
            serde_json::json!(null),
            serde_json::json!({"start":20,"max":100,"gain_lock_ms":0}),
        ] {
            let mut invalid = original.clone();
            invalid["combat_profile"]["battle_profile"]["resonance"] = value;
            assert!(serde_json::from_value::<Employee>(invalid).is_err());
        }
        let mut invalid = employee;
        invalid.combat_profile.battle_profile.resonance.start = 99;
        assert!(serde_json::to_value(&invalid).is_err());
        assert!(invalid
            .combat_profile_for_battle(
                &SkillFragmentDatabase::with_builtin_starter(vec![]),
                &SkillFragmentInventory::new()
            )
            .is_err());
        let rest = "(hp_recovery_per_tick:3)";
        assert!(
            ron::de::from_str::<crate::game::data::equipment_data::PlatformRestProfile>(rest)
                .is_ok()
        );
        assert!(
            ron::de::from_str::<crate::game::data::equipment_data::PlatformRestProfile>(
                &rest.replace(")", ",resonance_recovery_per_tick:5)")
            )
            .is_err()
        );
    }

    #[test]
    fn consumable_initial_skill_charge_increases_starting_resonance() {
        let mut employee =
            Employee::new(Uuid::from_u128(1), "skill_charge_buff", "Skill Charge Buff");
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

        assert_eq!(profile.resonance.start, 60);
    }

    #[test]
    fn consumable_initial_skill_charge_caps_at_resonance_max() {
        let mut employee =
            Employee::new(Uuid::from_u128(1), "skill_charge_cap", "Skill Charge Cap");
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
            role: EmployeeRole::Tank,
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
        assert_eq!(employee.role, EmployeeRole::Tank);
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

#[cfg(test)]
mod breakdown_tests {
    use super::*;
    #[test]
    fn u14_breakdown_regulated_cost_recovery_reentry_and_serialization() {
        let mut employee = Employee::new(Uuid::from_u128(9), "counter", "Counter");
        let node = Some(crate::game::map::MapNodeId::new(Uuid::from_u128(91)));
        employee.condition.trauma = 100;
        employee.reconcile_breakdown(4, node);
        employee.consume_breakdown_entry_cost(4, 0, node);
        assert_eq!(employee.breakdown.remaining_cost(), Some(4));
        employee.consume_breakdown_entry_cost(4, 2, node);
        assert_eq!(employee.breakdown.remaining_cost(), Some(2));
        employee.condition.reduce_trauma(1);
        employee.reconcile_breakdown(4, None);
        assert_eq!(employee.breakdown, BreakdownCounter::Inactive);
        employee.condition.add_trauma(1);
        employee.reconcile_breakdown(4, node);
        assert_eq!(employee.breakdown.remaining_cost(), Some(4));
        employee.consume_breakdown_entry_cost(4, 3, node);
        let encoded = serde_json::to_vec(&employee).unwrap();
        let mut restored: Employee = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(restored.breakdown.remaining_cost(), Some(1));
        restored.consume_breakdown_entry_cost(4, 2, None);
        assert!(!restored.is_alive());
        assert_eq!(restored.condition.trauma, 100);
        assert_eq!(restored.breakdown, BreakdownCounter::Inactive);
        let mut sequence=Employee::new(Uuid::from_u128(10),"sequence","Sequence");
        sequence.condition.trauma=100;
        for (cost,remaining) in [(1,3),(2,1),(1,0)] {
            sequence.consume_breakdown_entry_cost(4,cost,None);
            assert_eq!(sequence.is_alive(),remaining>0);
            assert_eq!(sequence.breakdown.remaining_cost(),(remaining>0).then_some(remaining));
        }
    }
}
