use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{
        reward_data::{RewardMetadata, RewardTag},
        skill_fragment_data::SkillFragmentId,
        GameDataBase, Item,
    },
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
    },
    GrantEquipment {
        equipment_id: Option<String>,
    },
    GrantEquipmentMaterial {
        material_id: String,
        amount: u32,
    },
    GrantArtifact {
        artifact_id: String,
    },
    GrantSkillFragment {
        fragment_id: SkillFragmentId,
    },
    GrantSkillFragmentResearch {
        fragment_id: SkillFragmentId,
        amount: u32,
    },
    ForbiddenAbnormalityGrant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RewardOption {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub tags: Vec<RewardTag>,
    pub effects: Vec<RewardEffect>,
}

impl RewardOption {
    pub fn from_metadata(value: &RewardMetadata) -> Self {
        Self {
            id: value.id.clone(),
            uuid: value.uuid,
            name: value.name.clone(),
            description: value.description.clone(),
            icon: value.icon.clone(),
            tags: value.resolved_tags(),
            effects: value.effects.clone(),
        }
    }
}

impl From<&RewardMetadata> for RewardOption {
    fn from(value: &RewardMetadata) -> Self {
        Self::from_metadata(value)
    }
}

pub struct RewardExecutor;

impl RewardExecutor {
    pub fn grant_reward_with_state(
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        reward: &RewardOption,
        seed: u64,
    ) -> Result<InventoryDiffDto, GameError> {
        let mut inventory_diff = InventoryDiffDto::default();

        for (effect_index, effect) in reward.effects.iter().enumerate() {
            let effect_seed = seed ^ ((effect_index as u64) << 32);
            let granted = Self::grant_effect_with_state(
                inventory,
                skill_fragments,
                uuid_manager,
                enkephalin,
                game_data,
                skill_fragment_policy,
                effect,
                effect_seed,
            )?;
            inventory_diff.added.extend(granted.added);
            inventory_diff.updated.extend(granted.updated);
            inventory_diff.removed.extend(granted.removed);
            inventory_diff
                .material_stacks
                .extend(granted.material_stacks);
        }

        Ok(inventory_diff)
    }

    fn grant_effect_with_state(
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        effect: &RewardEffect,
        seed: u64,
    ) -> Result<InventoryDiffDto, GameError> {
        let mut inventory_diff = InventoryDiffDto::default();

        match effect {
            RewardEffect::GrantEnkephalin { amount } => {
                enkephalin.amount = enkephalin
                    .amount
                    .checked_add(*amount)
                    .ok_or(GameError::InvalidAction)?;
                info!(
                    "Granted enkephalin reward: amount={}, new_total={}",
                    amount, enkephalin.amount
                );
            }
            RewardEffect::GrantExperience { amount } => {
                info!(
                    "Queued experience reward for caller-specific employee targets: amount={}",
                    amount
                );
            }
            RewardEffect::GrantEquipment { equipment_id } => {
                let item = match equipment_id.as_deref() {
                    Some(id) => Item::Equipment(std::sync::Arc::new(
                        game_data
                            .equipment_data
                            .get_by_id(id)
                            .ok_or_else(|| {
                                GameError::InvalidStaticData(format!(
                                    "reward references missing equipment id '{id}'"
                                ))
                            })?
                            .clone(),
                    )),
                    None => {
                        if game_data.equipment_data.items.is_empty() {
                            warn!("Equipment reward requested but equipment database is empty");
                            return Err(GameError::InvalidAction);
                        }
                        let mut rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0xB0B0_5001);
                        let idx = rng.gen_range(0..game_data.equipment_data.items.len());
                        Item::Equipment(std::sync::Arc::new(
                            game_data.equipment_data.items[idx].clone(),
                        ))
                    }
                };
                Self::grant_inventory_item(inventory, uuid_manager, item, &mut inventory_diff)?;
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
                inventory_diff.material_stacks.push(
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
                Self::grant_inventory_item(inventory, uuid_manager, item, &mut inventory_diff)?;
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

                skill_fragments.add_with_policy(metadata, skill_fragment_policy)?;
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

                skill_fragments.add_research_progress_with_policy(
                    &metadata.id,
                    *amount,
                    skill_fragment_policy,
                )?;
                info!(
                    "Granted skill fragment research progress: fragment_id={}, amount={}",
                    metadata.id, amount
                );
            }
            RewardEffect::ForbiddenAbnormalityGrant => {
                warn!("Rejected abnormality materialization into player inventory");
                return Err(GameError::InvalidAction);
            }
        }

        Ok(inventory_diff)
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

        let owned_uuid = uuid_manager.next_owned_equipment();
        inventory.add_item_owned(owned_uuid, item.clone_arc())?;
        inventory_diff
            .added
            .push(InventoryItemDto::from_item_with_uuid(&item, owned_uuid)?);

        Ok(())
    }
}
