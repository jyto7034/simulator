use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{
        reward_data::{RewardGrantKind, RewardMetadata},
        skill_fragment_data::SkillFragmentId,
        GameDataBase, Item,
    },
    employee::{EmployeeLifeState, EmployeeRoster},
    managers::uuid_manager::UuidManager,
    resources::{Enkephalin, Inventory, InventoryDiffDto, InventoryItemDto},
    skill_fragment::{SkillFragmentInventory, SkillFragmentPolicy},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RewardEffect {
    GrantEnkephalin {
        amount: u32,
    },
    GrantExperience {
        amount: u32,
        target: ExperienceTargetPolicy,
    },
    GrantEquipment {
        equipment_id: String,
    },
    GrantEquipmentFromPool {
        pool_id: String,
    },
    GrantEquipmentMaterial {
        material_id: String,
        amount: u32,
    },
    GrantArtifact {
        artifact_id: String,
    },
    GrantConsumable {
        consumable_id: String,
    },
    GrantSkillFragment {
        fragment_id: SkillFragmentId,
    },
    GrantSkillFragmentResearch {
        fragment_id: SkillFragmentId,
        amount: u32,
    },
    GrantFragmentDust {
        amount: u32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperienceTargetPolicy {
    CombatParticipants,
    AliveRoster,
    SelectedEmployee,
    AllRoster,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RewardOption {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub effects: Vec<RewardEffect>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrantExecutionResult {
    pub inventory_diff: InventoryDiffDto,
    pub skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
    pub skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
    pub employee_experience_diffs: Vec<EmployeeExperienceDiffDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeExperienceDiffDto {
    pub employee_uuid: Uuid,
    pub amount: u32,
    pub level_before: u32,
    pub level_after: u32,
    pub experience_before: u32,
    pub experience_after: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillFragmentGrantDiffDto {
    pub fragment_id: SkillFragmentId,
    pub count_before: u32,
    pub count_after: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillFragmentResearchDiffDto {
    pub fragment_id: SkillFragmentId,
    pub research_progress_before: u32,
    pub research_progress_after: u32,
    pub research_completion_count_before: u32,
    pub research_completion_count_after: u32,
    pub newly_pending_deliveries: u32,
}

impl RewardOption {
    pub fn from_metadata(value: &RewardMetadata) -> Self {
        Self {
            id: value.id.clone(),
            uuid: value.uuid,
            name: value.name.clone(),
            description: value.description.clone(),
            icon: value.icon.clone(),
            effects: value.effects.clone(),
        }
    }

    pub fn grant_kinds(&self) -> Vec<RewardGrantKind> {
        RewardGrantKind::from_effects(&self.effects)
    }
}

impl From<&RewardMetadata> for RewardOption {
    fn from(value: &RewardMetadata) -> Self {
        Self::from_metadata(value)
    }
}

pub struct GrantExecutor;

#[derive(Debug, Clone, Default)]
pub struct GrantExecutionContext {
    pub combat_participant_employee_ids: Option<Vec<Uuid>>,
    pub selected_employee_id: Option<Uuid>,
}

impl GrantExecutor {
    pub fn grant_reward_with_state(
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        roster: &mut EmployeeRoster,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        context: &GrantExecutionContext,
        reward: &RewardOption,
        seed: u64,
    ) -> Result<GrantExecutionResult, GameError> {
        Self::grant_effects_with_state(
            inventory,
            skill_fragments,
            roster,
            uuid_manager,
            enkephalin,
            game_data,
            skill_fragment_policy,
            context,
            &reward.effects,
            seed,
        )
    }

    pub fn grant_effects_with_state(
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        roster: &mut EmployeeRoster,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        context: &GrantExecutionContext,
        effects: &[RewardEffect],
        seed: u64,
    ) -> Result<GrantExecutionResult, GameError> {
        let mut result = GrantExecutionResult::default();

        for (effect_index, effect) in effects.iter().enumerate() {
            let effect_seed = seed ^ ((effect_index as u64) << 32);
            let granted = Self::grant_effect_with_state(
                inventory,
                skill_fragments,
                roster,
                uuid_manager,
                enkephalin,
                game_data,
                skill_fragment_policy,
                context,
                effect,
                effect_seed,
            )?;
            result
                .inventory_diff
                .added
                .extend(granted.inventory_diff.added);
            result
                .inventory_diff
                .updated
                .extend(granted.inventory_diff.updated);
            result
                .inventory_diff
                .removed
                .extend(granted.inventory_diff.removed);
            result
                .inventory_diff
                .material_stacks
                .extend(granted.inventory_diff.material_stacks);
            result
                .skill_fragment_diffs
                .extend(granted.skill_fragment_diffs);
            result
                .skill_fragment_research_diffs
                .extend(granted.skill_fragment_research_diffs);
            result
                .employee_experience_diffs
                .extend(granted.employee_experience_diffs);
        }

        Ok(result)
    }

    fn grant_effect_with_state(
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        roster: &mut EmployeeRoster,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        context: &GrantExecutionContext,
        effect: &RewardEffect,
        seed: u64,
    ) -> Result<GrantExecutionResult, GameError> {
        let mut result = GrantExecutionResult::default();

        match effect {
            RewardEffect::GrantEnkephalin { amount } => {
                enkephalin.checked_add(*amount)?;
                info!(
                    "Granted enkephalin reward: amount={}, new_total={}",
                    amount, enkephalin.amount
                );
            }
            RewardEffect::GrantExperience { amount, target } => {
                result
                    .employee_experience_diffs
                    .extend(Self::grant_experience(
                        roster, context, game_data, *amount, *target,
                    )?);
                info!(
                    "Granted experience reward: amount={}, target={:?}",
                    amount, target
                );
            }
            RewardEffect::GrantEquipment { equipment_id } => {
                let item = Item::Equipment(std::sync::Arc::new(
                    game_data
                        .equipment_data
                        .get_by_id(equipment_id)
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(format!(
                                "reward references missing equipment id '{equipment_id}'"
                            ))
                        })?
                        .clone(),
                ));
                Self::grant_inventory_item(
                    inventory,
                    uuid_manager,
                    item,
                    &mut result.inventory_diff,
                )?;
            }
            RewardEffect::GrantEquipmentFromPool { pool_id } => {
                let pool = game_data
                    .reward_data
                    .equipment_pool_by_id(pool_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "reward references missing equipment reward pool '{pool_id}'"
                        ))
                    })?;
                let candidates = pool.candidates(&game_data.equipment_data);
                let total_weight = candidates
                    .iter()
                    .fold(0_u32, |total, (_, weight)| total.saturating_add(*weight));
                if total_weight == 0 {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment reward pool '{pool_id}' has no weighted candidates"
                    )));
                }

                let mut rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0xB0B0_5001);
                let mut roll = rng.gen_range(0..total_weight);
                let selected = candidates
                    .into_iter()
                    .find_map(|(equipment, weight)| {
                        if roll < weight {
                            Some(equipment)
                        } else {
                            roll -= weight;
                            None
                        }
                    })
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "equipment reward pool '{pool_id}' selection failed"
                        ))
                    })?;
                let item = Item::Equipment(std::sync::Arc::new(selected.clone()));
                Self::grant_inventory_item(
                    inventory,
                    uuid_manager,
                    item,
                    &mut result.inventory_diff,
                )?;
            }
            RewardEffect::GrantEquipmentMaterial {
                material_id,
                amount,
            } => {
                let metadata = game_data
                    .equipment_data
                    .get_material_by_id(material_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "reward references missing equipment material id '{material_id}'"
                        ))
                    })?;

                let new_amount = inventory.equipment_materials.add(&metadata.id, *amount)?;
                result.inventory_diff.material_stacks.push(
                    crate::game::resources::EquipmentMaterialStackDto::from_metadata(
                        metadata, new_amount,
                    ),
                );
                info!(
                    "Granted equipment material reward: material_id={}, amount={}, new_total={}",
                    metadata.id, amount, new_amount
                );
            }
            RewardEffect::GrantArtifact { artifact_id } => {
                let item = Item::Artifact(std::sync::Arc::new(
                    game_data
                        .artifact_data
                        .get_by_id(artifact_id)
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(format!(
                                "reward references missing artifact id '{artifact_id}'"
                            ))
                        })?
                        .clone(),
                ));
                Self::grant_inventory_item(
                    inventory,
                    uuid_manager,
                    item,
                    &mut result.inventory_diff,
                )?;
            }
            RewardEffect::GrantConsumable { consumable_id } => {
                let item = Item::Consumable(std::sync::Arc::new(
                    game_data
                        .consumable_data
                        .get_by_id(consumable_id)
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(format!(
                                "reward references missing consumable id '{consumable_id}'"
                            ))
                        })?
                        .clone(),
                ));
                Self::grant_inventory_item(
                    inventory,
                    uuid_manager,
                    item,
                    &mut result.inventory_diff,
                )?;
            }
            RewardEffect::GrantSkillFragment { fragment_id } => {
                let metadata = game_data
                    .skill_fragment_data
                    .get_by_id(fragment_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "reward references missing skill fragment id '{fragment_id}'"
                        ))
                    })?;

                let count_before = skill_fragments.count(&metadata.id);
                skill_fragments.add_with_policy(metadata, skill_fragment_policy)?;
                let count_after = skill_fragments.count(&metadata.id);
                result.skill_fragment_diffs.push(SkillFragmentGrantDiffDto {
                    fragment_id: metadata.id.clone(),
                    count_before,
                    count_after,
                });
                info!("Granted skill fragment reward: fragment_id={}", metadata.id);
            }
            RewardEffect::GrantSkillFragmentResearch {
                fragment_id,
                amount,
            } => {
                let metadata = game_data
                    .skill_fragment_data
                    .get_by_id(fragment_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "reward references missing skill fragment id '{fragment_id}'"
                        ))
                    })?;

                let progress_before = skill_fragments.progress(&metadata.id);
                let progress_after = skill_fragments.add_research_progress_with_policy(
                    &metadata.id,
                    *amount,
                    skill_fragment_policy,
                )?;
                result
                    .skill_fragment_research_diffs
                    .push(SkillFragmentResearchDiffDto {
                        fragment_id: metadata.id.clone(),
                        research_progress_before: progress_before.research_progress,
                        research_progress_after: progress_after.progress.research_progress,
                        research_completion_count_before: progress_before.research_completion_count,
                        research_completion_count_after: progress_after
                            .progress
                            .research_completion_count,
                        newly_pending_deliveries: progress_after.newly_pending_deliveries,
                    });
                info!(
                    "Granted skill fragment research progress: fragment_id={}, amount={}",
                    metadata.id, amount
                );
            }
            RewardEffect::GrantFragmentDust { amount } => {
                let total = skill_fragments.add_fragment_dust(*amount)?;
                info!(
                    "Granted skill fragment dust: amount={}, new_total={}",
                    amount, total
                );
            }
        }

        Ok(result)
    }

    fn grant_inventory_item(
        inventory: &mut Inventory,
        uuid_manager: &mut UuidManager,
        item: Item,
        inventory_diff: &mut InventoryDiffDto,
    ) -> Result<(), GameError> {
        if !inventory.can_add_item(&item) {
            return Err(GameError::InventoryFull);
        }

        let owned_uuid = match &item {
            Item::Consumable(_) => uuid_manager.next_owned_consumable(),
            _ => uuid_manager.next_owned_equipment(),
        };
        inventory.add_item_owned(owned_uuid, item.clone_arc())?;
        inventory_diff
            .added
            .push(InventoryItemDto::from_item_with_uuid(&item, owned_uuid)?);

        Ok(())
    }

    fn grant_experience(
        roster: &mut EmployeeRoster,
        context: &GrantExecutionContext,
        game_data: &GameDataBase,
        amount: u32,
        target: ExperienceTargetPolicy,
    ) -> Result<Vec<EmployeeExperienceDiffDto>, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }

        let mut employee_ids = match target {
            ExperienceTargetPolicy::CombatParticipants => context
                .combat_participant_employee_ids
                .clone()
                .ok_or(GameError::InvalidAction)?,
            ExperienceTargetPolicy::AliveRoster => roster
                .iter()
                .filter(|employee| employee.life_state == EmployeeLifeState::Alive)
                .map(|employee| employee.uuid)
                .collect(),
            ExperienceTargetPolicy::SelectedEmployee => {
                vec![context
                    .selected_employee_id
                    .ok_or(GameError::InvalidAction)?]
            }
            ExperienceTargetPolicy::AllRoster => {
                roster.iter().map(|employee| employee.uuid).collect()
            }
        };

        employee_ids.sort_by_key(|uuid| uuid.as_u128());
        employee_ids.dedup();
        employee_ids.retain(|employee_id| roster.contains(employee_id));
        if employee_ids.is_empty() {
            return Ok(Vec::new());
        }

        let eligible_count =
            u32::try_from(employee_ids.len()).map_err(|_| GameError::InvalidAction)?;
        let base_amount = amount / eligible_count;
        let mut remainder = amount % eligible_count;
        let mut diffs = Vec::with_capacity(employee_ids.len());

        for employee_id in employee_ids {
            let grant_amount = base_amount + u32::from(remainder > 0);
            remainder = remainder.saturating_sub(1);
            if grant_amount == 0 {
                continue;
            }

            let employee = roster
                .get_mut(&employee_id)
                .ok_or(GameError::UnitNotFound)?;
            let level_before = employee.level;
            let experience_before = employee.experience;
            employee.add_experience_with_policy(grant_amount, game_data.run_policy.as_ref());
            diffs.push(EmployeeExperienceDiffDto {
                employee_uuid: employee_id,
                amount: grant_amount,
                level_before,
                level_after: employee.level,
                experience_before,
                experience_after: employee.experience,
            });
        }

        Ok(diffs)
    }
}
