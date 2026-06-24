use std::collections::BTreeMap;

use crate::game::{
    ability::SkillActivationMode,
    battle::types::UnitCombatProfile,
    behavior::GameError,
    data::skill_fragment_data::{
        SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentId, SkillFragmentMetadata,
        SkillFragmentRarity,
    },
    map::MapNodeCategory,
};
use serde::{Deserialize, Serialize};

pub fn starter_basic_attack_fragment_id() -> SkillFragmentId {
    SkillFragmentId::from("starter_basic_attack_enhancement")
}

#[derive(Debug, Clone, Default)]
pub struct SkillFragmentInventory {
    stacks: BTreeMap<SkillFragmentId, SkillFragmentStack>,
    progress: BTreeMap<SkillFragmentId, SkillFragmentProgress>,
    pending_research_deliveries: BTreeMap<SkillFragmentId, u32>,
    fragment_dust: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillFragmentStack {
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SkillFragmentProgress {
    pub research_progress: u32,
    pub research_completion_count: u32,
    pub upgrade_level: u8,
    pub awakening_progress: u32,
    pub awakening_available: bool,
    pub awakened: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFragmentGrantResult {
    AddedNew { count: u32 },
    StackedAdditionalCopy { count: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentUpgradeResult {
    pub dust_spent: u32,
    pub remaining_dust: u32,
    pub progress: SkillFragmentProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentAwakenResult {
    pub dust_spent: u32,
    pub remaining_dust: u32,
    pub progress: SkillFragmentProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentDismantleResult {
    pub remaining_count: u32,
    pub dust_gained: u32,
    pub total_dust: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentResearchResult {
    pub progress: SkillFragmentProgress,
    pub newly_pending_deliveries: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillFragmentResearchDelivery {
    pub fragment_id: SkillFragmentId,
    pub count: u32,
    pub total_count: u32,
}

impl SkillFragmentInventory {
    pub fn new() -> Self {
        Self {
            stacks: BTreeMap::new(),
            progress: BTreeMap::new(),
            pending_research_deliveries: BTreeMap::new(),
            fragment_dust: 0,
        }
    }

    pub fn owned_ids(&self) -> impl Iterator<Item = &SkillFragmentId> {
        self.stacks.keys()
    }

    pub fn stacks(&self) -> impl Iterator<Item = (&SkillFragmentId, &SkillFragmentStack)> {
        self.stacks.iter()
    }

    pub fn contains(&self, fragment_id: &SkillFragmentId) -> bool {
        self.count(fragment_id) > 0
    }

    pub fn count(&self, fragment_id: &SkillFragmentId) -> u32 {
        self.stacks.get(fragment_id).map_or(0, |stack| stack.count)
    }

    pub fn progress(&self, fragment_id: &SkillFragmentId) -> SkillFragmentProgress {
        self.progress.get(fragment_id).copied().unwrap_or_default()
    }

    pub fn progress_entries(
        &self,
    ) -> impl Iterator<Item = (&SkillFragmentId, SkillFragmentProgress)> {
        self.progress
            .iter()
            .map(|(fragment_id, progress)| (fragment_id, *progress))
    }

    pub fn fragment_dust(&self) -> u32 {
        self.fragment_dust
    }

    pub fn add_fragment_dust(&mut self, amount: u32) -> Result<u32, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        self.fragment_dust = self
            .fragment_dust
            .checked_add(amount)
            .ok_or(GameError::InvalidAction)?;
        Ok(self.fragment_dust)
    }

    fn consume_fragment_dust(&mut self, amount: u32) -> Result<u32, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        if self.fragment_dust < amount {
            return Err(GameError::InvalidAction);
        }
        self.fragment_dust -= amount;
        Ok(self.fragment_dust)
    }

    pub fn pending_research_deliveries(&self) -> impl Iterator<Item = (&SkillFragmentId, u32)> {
        self.pending_research_deliveries
            .iter()
            .map(|(fragment_id, count)| (fragment_id, *count))
    }

    pub fn add(
        &mut self,
        fragment: &SkillFragmentMetadata,
    ) -> Result<SkillFragmentGrantResult, GameError> {
        self.add_with_policy(fragment, &SkillFragmentPolicy::default())
    }

    pub fn add_with_policy(
        &mut self,
        fragment: &SkillFragmentMetadata,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentGrantResult, GameError> {
        self.add_id_with_policy(&fragment.id, policy)
    }

    pub fn add_id(
        &mut self,
        fragment_id: &SkillFragmentId,
    ) -> Result<SkillFragmentGrantResult, GameError> {
        self.add_id_with_policy(fragment_id, &SkillFragmentPolicy::default())
    }

    pub fn add_id_with_policy(
        &mut self,
        fragment_id: &SkillFragmentId,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentGrantResult, GameError> {
        let existing_count = self.count(fragment_id);
        if existing_count > 0
            && matches!(
                policy.stacking,
                SkillFragmentStackingPolicy::RejectAdditionalCopies
            )
        {
            return Err(GameError::InvalidAction);
        }

        let stack = self
            .stacks
            .entry(fragment_id.clone())
            .or_insert(SkillFragmentStack { count: 0 });
        stack.count = stack.count.checked_add(1).ok_or(GameError::InvalidAction)?;
        self.progress.entry(fragment_id.clone()).or_default();

        Ok(if existing_count == 0 {
            SkillFragmentGrantResult::AddedNew { count: stack.count }
        } else {
            SkillFragmentGrantResult::StackedAdditionalCopy { count: stack.count }
        })
    }

    pub fn add_research_progress(
        &mut self,
        fragment_id: &SkillFragmentId,
        amount: u32,
    ) -> Result<SkillFragmentResearchResult, GameError> {
        self.add_research_progress_with_policy(fragment_id, amount, &SkillFragmentPolicy::default())
    }

    pub fn add_research_progress_with_policy(
        &mut self,
        fragment_id: &SkillFragmentId,
        amount: u32,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentResearchResult, GameError> {
        let progress = self.progress.entry(fragment_id.clone()).or_default();
        progress.research_progress = progress.research_progress.saturating_add(amount);
        let newly_pending_deliveries = policy.research.queue_completed_deliveries(
            fragment_id,
            progress,
            &mut self.pending_research_deliveries,
        )?;

        Ok(SkillFragmentResearchResult {
            progress: *progress,
            newly_pending_deliveries,
        })
    }

    pub fn deliver_pending_research(
        &mut self,
        database: &SkillFragmentDatabase,
    ) -> Result<Vec<SkillFragmentResearchDelivery>, GameError> {
        self.deliver_pending_research_with_policy(database, &SkillFragmentPolicy::default())
    }

    pub fn deliver_pending_research_with_policy(
        &mut self,
        database: &SkillFragmentDatabase,
        policy: &SkillFragmentPolicy,
    ) -> Result<Vec<SkillFragmentResearchDelivery>, GameError> {
        let pending = self
            .pending_research_deliveries
            .iter()
            .map(|(fragment_id, count)| (fragment_id.clone(), *count))
            .collect::<Vec<_>>();
        for (fragment_id, _) in &pending {
            if database.get_by_id(fragment_id).is_none() {
                return Err(GameError::InvalidStaticData(format!(
                    "pending research delivery references missing skill fragment '{fragment_id}'"
                )));
            }
        }

        let mut deliveries = Vec::new();
        for (fragment_id, count) in pending {
            for _ in 0..count {
                self.add_id_with_policy(&fragment_id, policy)?;
            }
            self.pending_research_deliveries.remove(&fragment_id);
            deliveries.push(SkillFragmentResearchDelivery {
                fragment_id: fragment_id.clone(),
                count,
                total_count: self.count(&fragment_id),
            });
        }
        Ok(deliveries)
    }

    pub fn upgrade_with_dust(
        &mut self,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<SkillFragmentUpgradeResult, GameError> {
        self.upgrade_with_dust_policy(
            target_fragment_id,
            database,
            &SkillFragmentPolicy::default(),
        )
    }

    pub fn upgrade_with_dust_policy(
        &mut self,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentUpgradeResult, GameError> {
        let cost = policy
            .composition
            .validate_dust_upgrade(self, target_fragment_id, database)?;
        let remaining_dust = self.consume_fragment_dust(cost)?;

        let progress = self.progress.entry(target_fragment_id.clone()).or_default();
        progress.upgrade_level = progress.upgrade_level.saturating_add(1);
        progress.awakening_progress = progress
            .awakening_progress
            .saturating_add(policy.composition.awakening_progress_per_upgrade);
        if policy
            .composition
            .target_can_awaken(database, target_fragment_id)?
            && progress.awakening_progress >= policy.composition.awakening_threshold
        {
            progress.awakening_available = true;
        }

        Ok(SkillFragmentUpgradeResult {
            dust_spent: cost,
            remaining_dust,
            progress: *progress,
        })
    }

    pub fn awaken_with_dust(
        &mut self,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<SkillFragmentAwakenResult, GameError> {
        self.awaken_with_dust_policy(
            target_fragment_id,
            database,
            &SkillFragmentPolicy::default(),
        )
    }

    pub fn awaken_with_dust_policy(
        &mut self,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentAwakenResult, GameError> {
        let cost =
            policy
                .composition
                .validate_dust_awakening(self, target_fragment_id, database)?;
        let remaining_dust = if cost == 0 {
            self.fragment_dust
        } else {
            self.consume_fragment_dust(cost)?
        };

        let progress = self.progress.entry(target_fragment_id.clone()).or_default();
        progress.awakening_available = true;
        progress.awakened = true;

        Ok(SkillFragmentAwakenResult {
            dust_spent: cost,
            remaining_dust,
            progress: *progress,
        })
    }

    pub fn remove_copy_for_dismantle(
        &mut self,
        fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<SkillFragmentDismantleResult, GameError> {
        self.remove_copy_for_dismantle_with_policy(
            fragment_id,
            database,
            &SkillFragmentPolicy::default(),
        )
    }

    pub fn remove_copy_for_dismantle_with_policy(
        &mut self,
        fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
        policy: &SkillFragmentPolicy,
    ) -> Result<SkillFragmentDismantleResult, GameError> {
        let dust_gained = policy
            .dismantle
            .validate_dismantle(self, fragment_id, database)?;
        let remaining_count = {
            let stack = self
                .stacks
                .get_mut(fragment_id)
                .ok_or(GameError::InvalidAction)?;
            stack.count = stack.count.checked_sub(1).ok_or(GameError::InvalidAction)?;
            stack.count
        };
        if remaining_count == 0 {
            self.stacks.remove(fragment_id);
        }

        Ok(SkillFragmentDismantleResult {
            remaining_count,
            dust_gained,
            total_dust: self.fragment_dust,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFragmentActiveReplacementPolicy {
    EmptySlotOnly,
    FreeSwap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFragmentStackingPolicy {
    RejectAdditionalCopies,
    StackCopies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFragmentDismantlePolicy {
    Disabled,
    MaintenanceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentEquipPolicy {
    pub active_replacement: SkillFragmentActiveReplacementPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentCompositionPolicy {
    pub dust_per_upgrade: u32,
    pub awakening_progress_per_upgrade: u32,
    pub awakening_threshold: u32,
    pub post_awakening_dust_per_upgrade: u32,
    pub max_upgrade_level: u8,
    pub early_awakening_dust_per_missing_progress: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentResearchPolicy {
    pub completion_threshold: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchDeliveryPolicy {
    pub deliver_on_node_categories: Vec<MapNodeCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentPolicy {
    pub equip: SkillFragmentEquipPolicy,
    pub stacking: SkillFragmentStackingPolicy,
    pub dismantle: SkillFragmentDismantlePolicy,
    pub composition: SkillFragmentCompositionPolicy,
    pub research: SkillFragmentResearchPolicy,
}

impl SkillFragmentPolicy {
    pub fn default_run_policy() -> Self {
        Self {
            equip: SkillFragmentEquipPolicy {
                active_replacement: SkillFragmentActiveReplacementPolicy::EmptySlotOnly,
            },
            stacking: SkillFragmentStackingPolicy::StackCopies,
            dismantle: SkillFragmentDismantlePolicy::MaintenanceOnly,
            composition: SkillFragmentCompositionPolicy {
                dust_per_upgrade: 4,
                awakening_progress_per_upgrade: 1,
                awakening_threshold: 7,
                post_awakening_dust_per_upgrade: 8,
                max_upgrade_level: 10,
                early_awakening_dust_per_missing_progress: 4,
            },
            research: SkillFragmentResearchPolicy {
                completion_threshold: 100,
            },
        }
    }
}

impl Default for ResearchDeliveryPolicy {
    fn default() -> Self {
        Self {
            deliver_on_node_categories: vec![
                MapNodeCategory::Support,
                MapNodeCategory::HeadquartersContact,
                MapNodeCategory::Shop,
                MapNodeCategory::Reward,
            ],
        }
    }
}

impl ResearchDeliveryPolicy {
    pub fn can_deliver_on(&self, category: MapNodeCategory) -> bool {
        self.deliver_on_node_categories.contains(&category)
    }
}

impl SkillFragmentResearchPolicy {
    fn queue_completed_deliveries(
        &self,
        fragment_id: &SkillFragmentId,
        progress: &mut SkillFragmentProgress,
        pending_research_deliveries: &mut BTreeMap<SkillFragmentId, u32>,
    ) -> Result<u32, GameError> {
        if self.completion_threshold == 0 {
            return Err(GameError::InvalidStaticData(
                "skill fragment research completion_threshold must be > 0".to_string(),
            ));
        }

        let completed_count = progress.research_progress / self.completion_threshold;
        let newly_completed = completed_count.saturating_sub(progress.research_completion_count);
        if newly_completed == 0 {
            return Ok(0);
        }

        progress.research_completion_count = progress
            .research_completion_count
            .checked_add(newly_completed)
            .ok_or(GameError::InvalidAction)?;
        let pending = pending_research_deliveries
            .entry(fragment_id.clone())
            .or_insert(0);
        *pending = pending
            .checked_add(newly_completed)
            .ok_or(GameError::InvalidAction)?;
        Ok(newly_completed)
    }
}

impl SkillFragmentDismantlePolicy {
    pub fn validate_dismantle(
        &self,
        inventory: &SkillFragmentInventory,
        fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<u32, GameError> {
        if matches!(self, SkillFragmentDismantlePolicy::Disabled) {
            return Err(GameError::InvalidAction);
        }
        let count = inventory.count(fragment_id);
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let fragment = database.get_by_id(fragment_id).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "skill fragment '{}' is owned by run but missing from static data",
                fragment_id
            ))
        })?;
        Ok(match fragment.rarity {
            SkillFragmentRarity::Common => 1,
            SkillFragmentRarity::Rare => 2,
            SkillFragmentRarity::Exceptional => 4,
        })
    }
}

impl SkillFragmentEquipPolicy {
    fn validate_active_equip(
        &self,
        loadout: &SkillFragmentLoadout,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        match self.active_replacement {
            SkillFragmentActiveReplacementPolicy::EmptySlotOnly => {
                if loadout.active_fragment_id.is_none()
                    || loadout.active_fragment_id.as_ref() == Some(fragment_id)
                {
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            SkillFragmentActiveReplacementPolicy::FreeSwap => Ok(()),
        }
    }
}

impl SkillFragmentCompositionPolicy {
    pub fn dust_upgrade_cost(
        &self,
        inventory: &SkillFragmentInventory,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<u32, GameError> {
        if self.dust_per_upgrade == 0 {
            return Err(GameError::InvalidStaticData(
                "fragment upgrade dust_per_upgrade must be > 0".to_string(),
            ));
        }
        if self.post_awakening_dust_per_upgrade == 0 {
            return Err(GameError::InvalidStaticData(
                "fragment upgrade post_awakening_dust_per_upgrade must be > 0".to_string(),
            ));
        }
        if !inventory.contains(target_fragment_id) {
            return Err(GameError::InvalidAction);
        }

        database.get_by_id(target_fragment_id).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "skill fragment '{}' is owned by run but missing from static data",
                target_fragment_id
            ))
        })?;

        let progress = inventory.progress(target_fragment_id);
        if progress.upgrade_level >= self.max_upgrade_level {
            return Err(GameError::InvalidAction);
        }

        Ok(if progress.awakened {
            self.post_awakening_dust_per_upgrade
        } else {
            self.dust_per_upgrade
        })
    }

    pub fn dust_awakening_cost(
        &self,
        inventory: &SkillFragmentInventory,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<u32, GameError> {
        if !self.target_can_awaken(database, target_fragment_id)? {
            return Err(GameError::InvalidAction);
        }
        if !inventory.contains(target_fragment_id) {
            return Err(GameError::InvalidAction);
        }

        let progress = inventory.progress(target_fragment_id);
        if progress.awakened {
            return Err(GameError::InvalidAction);
        }

        let missing_progress = self
            .awakening_threshold
            .saturating_sub(progress.awakening_progress);
        missing_progress
            .checked_mul(self.early_awakening_dust_per_missing_progress)
            .ok_or(GameError::InvalidAction)
    }

    pub fn validate_dust_upgrade(
        &self,
        inventory: &SkillFragmentInventory,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<u32, GameError> {
        let cost = self.dust_upgrade_cost(inventory, target_fragment_id, database)?;
        if inventory.fragment_dust() < cost {
            return Err(GameError::InvalidAction);
        }
        Ok(cost)
    }

    pub fn validate_dust_awakening(
        &self,
        inventory: &SkillFragmentInventory,
        target_fragment_id: &SkillFragmentId,
        database: &SkillFragmentDatabase,
    ) -> Result<u32, GameError> {
        let required_dust = self.dust_awakening_cost(inventory, target_fragment_id, database)?;

        if inventory.fragment_dust() < required_dust {
            Err(GameError::InvalidAction)
        } else {
            Ok(required_dust)
        }
    }

    fn target_can_awaken(
        &self,
        database: &SkillFragmentDatabase,
        target_fragment_id: &SkillFragmentId,
    ) -> Result<bool, GameError> {
        let target = database.get_by_id(target_fragment_id).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "skill fragment '{}' is owned by run but missing from static data",
                target_fragment_id
            ))
        })?;
        Ok(matches!(
            &target.effect,
            SkillFragmentEffectDef::ActiveSkill {
                awakened_skill_id: Some(_),
                ..
            }
        ))
    }
}

impl Default for SkillFragmentPolicy {
    fn default() -> Self {
        Self::default_run_policy()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SkillFragmentLoadout {
    baseline_ids: Vec<SkillFragmentId>,
    active_fragment_id: Option<SkillFragmentId>,
}

impl SkillFragmentLoadout {
    #[cfg(test)]
    pub fn starter() -> Self {
        Self::with_baseline_ids(vec![starter_basic_attack_fragment_id()])
    }

    pub fn with_baseline_ids(baseline_ids: Vec<SkillFragmentId>) -> Self {
        Self {
            baseline_ids,
            active_fragment_id: None,
        }
    }

    pub fn baseline_ids(&self) -> &[SkillFragmentId] {
        &self.baseline_ids
    }

    pub fn active_fragment_id(&self) -> Option<&SkillFragmentId> {
        self.active_fragment_id.as_ref()
    }

    pub fn equipped_ids(&self) -> Vec<SkillFragmentId> {
        let mut ids = self.baseline_ids.clone();
        ids.extend(self.active_fragment_id.iter().cloned());
        ids
    }

    pub fn equip(
        &mut self,
        inventory: &SkillFragmentInventory,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        self.equip_with_policy(
            inventory,
            database,
            fragment_id,
            &SkillFragmentPolicy::default(),
        )
    }

    pub fn equip_with_policy(
        &mut self,
        inventory: &SkillFragmentInventory,
        database: &SkillFragmentDatabase,
        fragment_id: &SkillFragmentId,
        policy: &SkillFragmentPolicy,
    ) -> Result<(), GameError> {
        if self.baseline_ids.iter().any(|id| id == fragment_id) {
            return Ok(());
        }

        if !inventory.contains(fragment_id) {
            return Err(GameError::InvalidAction);
        }

        let metadata = database.get_by_id(fragment_id).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "employee loadout references missing skill fragment '{fragment_id}'"
            ))
        })?;

        if !matches!(metadata.effect, SkillFragmentEffectDef::ActiveSkill { .. }) {
            return Err(GameError::InvalidAction);
        }

        if self
            .active_fragment_id
            .as_ref()
            .is_some_and(|active_id| active_id == fragment_id)
        {
            return Ok(());
        }

        policy.equip.validate_active_equip(self, fragment_id)?;
        self.active_fragment_id = Some(fragment_id.clone());
        Ok(())
    }

    pub fn unequip(&mut self, fragment_id: &SkillFragmentId) -> Result<(), GameError> {
        if self.baseline_ids.iter().any(|id| id == fragment_id) {
            return Err(GameError::InvalidAction);
        }

        if self.active_fragment_id.as_ref() != Some(fragment_id) {
            return Err(GameError::InvalidAction);
        }

        self.active_fragment_id = None;
        Ok(())
    }

    pub fn apply_to_profile(
        &self,
        database: &SkillFragmentDatabase,
        inventory: &SkillFragmentInventory,
        base_profile: &UnitCombatProfile,
    ) -> Result<UnitCombatProfile, GameError> {
        let mut profile = base_profile.clone();

        for fragment_id in &self.baseline_ids {
            let metadata = database.get_by_id(fragment_id).ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "employee loadout references missing skill fragment '{fragment_id}'"
                ))
            })?;

            match &metadata.effect {
                SkillFragmentEffectDef::BasicAttackModifier {
                    attack_bonus,
                    attack_interval_ms_reduction,
                } => {
                    profile
                        .stats
                        .add_attack((*attack_bonus).min(i32::MAX as u32) as i32);
                    if *attack_interval_ms_reduction > 0 {
                        let reduction = i64::from(*attack_interval_ms_reduction);
                        profile.stats.add_attack_interval_ms(-reduction);
                        profile.basic_attack.interval_ms = profile
                            .basic_attack
                            .interval_ms
                            .saturating_sub(*attack_interval_ms_reduction as u64)
                            .max(1);
                    }
                }
                SkillFragmentEffectDef::ActiveSkill {
                    imitation_skill_id, ..
                } => {
                    return Err(GameError::InvalidStaticData(format!(
                        "baseline skill fragment '{fragment_id}' cannot grant active skill '{}'",
                        imitation_skill_id
                    )));
                }
            }
        }

        if let Some(fragment_id) = &self.active_fragment_id {
            let metadata = database.get_by_id(fragment_id).ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "employee loadout references missing skill fragment '{fragment_id}'"
                ))
            })?;

            let SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id,
                upgrade_skill_ids,
                awakened_skill_id,
            } = &metadata.effect
            else {
                return Err(GameError::InvalidStaticData(format!(
                    "active skill fragment slot references non-active fragment '{fragment_id}'"
                )));
            };

            if profile.skill_id.is_some() {
                return Err(GameError::InvalidAction);
            }
            let progress = inventory.progress(fragment_id);
            let selected_skill_id = if progress.awakened {
                awakened_skill_id.as_ref().ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "awakened skill fragment '{fragment_id}' has no awakened_skill_id"
                    ))
                })?
            } else {
                upgrade_skill_ids
                    .range(..=progress.upgrade_level)
                    .next_back()
                    .map(|(_, skill_id)| skill_id)
                    .unwrap_or(imitation_skill_id)
            };
            profile.skill_id = Some(selected_skill_id.clone());
            profile.skill_activation_mode = SkillActivationMode::Manual;
        }

        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::types::UnitCombatProfile;
    use crate::game::data::skill_fragment_data::SkillFragmentDatabase;

    #[test]
    fn starter_fragment_enhances_basic_attack_without_granting_active_skill() {
        let loadout = SkillFragmentLoadout::starter();
        let database = SkillFragmentDatabase::with_builtin_starter(vec![]);
        let profile = loadout
            .apply_to_profile(
                &database,
                &SkillFragmentInventory::new(),
                &UnitCombatProfile::employee_default(),
            )
            .unwrap();

        assert_eq!(profile.stats.attack, 12);
        assert_eq!(profile.skill_id, None);
    }

    #[test]
    fn active_skill_fragment_sets_employee_skill_when_equipped() {
        let mut loadout = SkillFragmentLoadout::starter();
        let mut inventory = SkillFragmentInventory::new();
        let active = crate::game::data::skill_fragment_data::SkillFragmentMetadata {
            id: crate::game::data::skill_fragment_data::SkillFragmentId::from(
                "fragment_test_active",
            ),
            uuid: uuid::Uuid::from_u128(1),
            name: "Active".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("test_active_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![active.clone()]);
        inventory.add(&active).unwrap();
        loadout
            .equip(
                &inventory,
                &database,
                &crate::game::data::skill_fragment_data::SkillFragmentId::from(
                    "fragment_test_active",
                ),
            )
            .unwrap();

        let profile = loadout
            .apply_to_profile(
                &database,
                &inventory,
                &UnitCombatProfile::employee_default(),
            )
            .unwrap();
        assert_eq!(profile.skill_id.as_deref(), Some("test_active_skill"));
        assert_eq!(profile.skill_activation_mode, SkillActivationMode::Manual);
    }

    #[test]
    fn active_skill_slot_rejects_second_active_fragment() {
        fn active_fragment(id: &str, skill_id: &str, uuid: u128) -> SkillFragmentMetadata {
            SkillFragmentMetadata {
                id: SkillFragmentId::from(id),
                uuid: uuid::Uuid::from_u128(uuid),
                name: id.to_string(),
                description: "test".to_string(),
                rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
                equip_limit: crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
                origin: None,
                sources: vec![
                    crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
                ],
                dependencies: vec![],
                compatibility: Default::default(),
                effect: SkillFragmentEffectDef::ActiveSkill {
                    imitation_skill_id: crate::game::ability::SkillId::from(skill_id),
                    upgrade_skill_ids: Default::default(),
                    awakened_skill_id: None,
                },
            }
        }

        let first = active_fragment("fragment_first_active", "first_active_skill", 1);
        let second = active_fragment("fragment_second_active", "second_active_skill", 2);
        let database =
            SkillFragmentDatabase::with_builtin_starter(vec![first.clone(), second.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&first).unwrap();
        inventory.add(&second).unwrap();
        let mut loadout = SkillFragmentLoadout::starter();

        loadout.equip(&inventory, &database, &first.id).unwrap();
        let err = loadout
            .equip(&inventory, &database, &second.id)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert_eq!(loadout.active_fragment_id(), Some(&first.id));
    }

    #[test]
    fn additional_fragment_copies_stack_until_dismantled_for_dust() {
        let fragment = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_stack_test"),
            uuid: uuid::Uuid::from_u128(10),
            name: "Duplicate".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("stack_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
        };
        let mut inventory = SkillFragmentInventory::new();

        assert_eq!(
            inventory.add(&fragment).unwrap(),
            SkillFragmentGrantResult::AddedNew { count: 1 }
        );
        assert_eq!(
            inventory.add(&fragment).unwrap(),
            SkillFragmentGrantResult::StackedAdditionalCopy { count: 2 }
        );
        assert_eq!(inventory.count(&fragment.id), 2);
        assert_eq!(
            inventory.progress(&fragment.id),
            SkillFragmentProgress::default()
        );
    }

    #[test]
    fn dust_upgrade_consumes_fragment_dust_and_increases_awakening_progress() {
        let target = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_upgrade_target"),
            uuid: uuid::Uuid::from_u128(11),
            name: "Upgrade Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("upgrade_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: Some(crate::game::ability::SkillId::from("awakened_skill")),
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![target.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&target).unwrap();
        inventory.add_fragment_dust(4).unwrap();

        let result = inventory.upgrade_with_dust(&target.id, &database).unwrap();

        assert_eq!(result.dust_spent, 4);
        assert_eq!(result.remaining_dust, 0);
        assert_eq!(result.progress.upgrade_level, 1);
        assert_eq!(result.progress.awakening_progress, 1);
        assert!(!result.progress.awakening_available);
    }

    #[test]
    fn research_progress_is_tracked_separately_from_awakening_progress() {
        let mut inventory = SkillFragmentInventory::new();
        let fragment_id = SkillFragmentId::from("fragment_research_target");

        let result = inventory
            .add_research_progress(&fragment_id, 7)
            .expect("research progress should be recordable before owning a fragment");

        assert_eq!(result.progress.research_progress, 7);
        assert_eq!(result.progress.awakening_progress, 0);
        assert!(!inventory.contains(&fragment_id));
    }

    #[test]
    fn research_completion_queues_pending_delivery_without_granting_fragment() {
        let mut inventory = SkillFragmentInventory::new();
        let fragment_id = SkillFragmentId::from("fragment_research_target");

        let result = inventory
            .add_research_progress(&fragment_id, 100)
            .expect("research completion should queue delivery");

        assert_eq!(result.progress.research_progress, 100);
        assert_eq!(result.progress.research_completion_count, 1);
        assert_eq!(result.newly_pending_deliveries, 1);
        assert!(!inventory.contains(&fragment_id));
        assert_eq!(
            inventory.pending_research_deliveries().collect::<Vec<_>>(),
            vec![(&fragment_id, 1)]
        );
    }

    #[test]
    fn pending_research_delivery_grants_fragment_when_delivered() {
        let fragment = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_delivery_target"),
            uuid: uuid::Uuid::from_u128(15),
            name: "Delivery Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("delivery_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![fragment.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory
            .add_research_progress(&fragment.id, 200)
            .expect("two completions should queue deliveries");

        let deliveries = inventory
            .deliver_pending_research(&database)
            .expect("pending research should deliver owned fragments");

        assert_eq!(
            deliveries,
            vec![SkillFragmentResearchDelivery {
                fragment_id: fragment.id.clone(),
                count: 2,
                total_count: 2,
            }]
        );
        assert_eq!(inventory.count(&fragment.id), 2);
        assert!(inventory.pending_research_deliveries().next().is_none());
    }

    #[test]
    fn research_delivery_policy_excludes_combat_and_boss_by_default() {
        let policy = ResearchDeliveryPolicy::default();

        assert!(policy.can_deliver_on(MapNodeCategory::Support));
        assert!(policy.can_deliver_on(MapNodeCategory::HeadquartersContact));
        assert!(policy.can_deliver_on(MapNodeCategory::Shop));
        assert!(policy.can_deliver_on(MapNodeCategory::Reward));
        assert!(!policy.can_deliver_on(MapNodeCategory::Start));
        assert!(!policy.can_deliver_on(MapNodeCategory::Combat));
        assert!(!policy.can_deliver_on(MapNodeCategory::Boss));
    }

    #[test]
    fn dust_upgrade_rejects_insufficient_fragment_dust() {
        let target_id = SkillFragmentId::from("fragment_target");
        let target = SkillFragmentMetadata {
            id: target_id.clone(),
            uuid: uuid::Uuid::from_u128(13),
            name: "Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("target_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![target.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&target).unwrap();

        let err = inventory
            .upgrade_with_dust(&target_id, &database)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
        assert_eq!(inventory.fragment_dust(), 0);
    }

    #[test]
    fn awakening_requires_awakened_skill_and_consumes_early_dust_cost() {
        let target = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_awakening_target"),
            uuid: uuid::Uuid::from_u128(15),
            name: "Awakening Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("target_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: Some(crate::game::ability::SkillId::from("awakened_skill")),
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![target.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&target).unwrap();
        inventory.add_fragment_dust(28).unwrap();

        let result = inventory.awaken_with_dust(&target.id, &database).unwrap();

        assert_eq!(result.dust_spent, 28);
        assert_eq!(result.remaining_dust, 0);
        assert!(result.progress.awakening_available);
        assert!(result.progress.awakened);
    }

    #[test]
    fn awakening_rejects_fragments_without_awakened_skill_data() {
        let target = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_no_awakening_target"),
            uuid: uuid::Uuid::from_u128(17),
            name: "No Awakening Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("target_skill"),
                upgrade_skill_ids: Default::default(),
                awakened_skill_id: None,
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![target.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&target).unwrap();

        let err = inventory
            .awaken_with_dust(&target.id, &database)
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidAction));
    }

    #[test]
    fn active_fragment_uses_upgrade_variant_then_awakened_skill() {
        let mut upgrade_skill_ids = std::collections::BTreeMap::new();
        upgrade_skill_ids.insert(1, crate::game::ability::SkillId::from("variant_skill"));
        let target = SkillFragmentMetadata {
            id: SkillFragmentId::from("fragment_variant_target"),
            uuid: uuid::Uuid::from_u128(18),
            name: "Variant Target".to_string(),
            description: "test".to_string(),
            rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity::Rare,
            equip_limit:
                crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            sources: vec![
                crate::game::data::skill_fragment_data::SkillFragmentAcquisitionSource::RareReward,
            ],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: crate::game::ability::SkillId::from("base_skill"),
                upgrade_skill_ids,
                awakened_skill_id: Some(crate::game::ability::SkillId::from("awakened_skill")),
            },
        };
        let database = SkillFragmentDatabase::with_builtin_starter(vec![target.clone()]);
        let mut inventory = SkillFragmentInventory::new();
        inventory.add(&target).unwrap();
        inventory.add_fragment_dust(28).unwrap();
        inventory.upgrade_with_dust(&target.id, &database).unwrap();

        let mut loadout = SkillFragmentLoadout::starter();
        loadout.equip(&inventory, &database, &target.id).unwrap();
        let upgraded_profile = loadout
            .apply_to_profile(
                &database,
                &inventory,
                &UnitCombatProfile::employee_default(),
            )
            .unwrap();
        assert_eq!(upgraded_profile.skill_id.as_deref(), Some("variant_skill"));

        inventory.awaken_with_dust(&target.id, &database).unwrap();
        let awakened_profile = loadout
            .apply_to_profile(
                &database,
                &inventory,
                &UnitCombatProfile::employee_default(),
            )
            .unwrap();
        assert_eq!(awakened_profile.skill_id.as_deref(), Some("awakened_skill"));
    }
}
