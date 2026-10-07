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
    employee::EmployeeRoster,
    managers::uuid_manager::UuidManager,
    resources::{Enkephalin, Inventory, InventoryDiffDto, InventoryItemDto},
    skill_fragment::{SkillFragmentInventory, SkillFragmentPolicy},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
    GrantConsumable {
        consumable_id: String,
        quantity: u32,
    },
    GrantSkillFragment {
        fragment_id: SkillFragmentId,
    },
    GrantFragmentDust {
        amount: u32,
    },
    GrantRelic {
        relic_id: String,
    },
    GrantRelicCorruptionCatalyst {
        amount: u32,
    },
    GrantEquipmentDust {
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
    pub employee_experience_diffs: Vec<EmployeeExperienceDiffDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relic_diffs: Vec<crate::game::relic::OwnedRelic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crafting_resources: Option<crate::game::relic::RunCraftingResources>,
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
    pub copy_uuid: Uuid,
    pub fragment_id: SkillFragmentId,
    pub count_before: u32,
    pub count_after: u32,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantExecutionContext {
    pub combat_participant_employee_ids: Option<Vec<Uuid>>,
    pub selected_employee_id: Option<Uuid>,
}

crate::authoring::clone_shape::struct_shape!(GrantExecutionContext {
    combat_participant_employee_ids,
    selected_employee_id
});

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
        Self::grant_effects_internal(
            inventory,
            skill_fragments,
            roster,
            uuid_manager,
            enkephalin,
            game_data,
            skill_fragment_policy,
            context,
            effects,
            seed,
            None,
        )
    }

    /// Ordered effects on caller-owned transaction roots, including run relics.
    /// The Core's EconomyGrantState supplies the atomic capture/commit boundary.
    pub fn grant_effects_with_run_state(
        relics: &mut crate::game::relic::RelicInventory,
        resources: &mut crate::game::relic::RunCraftingResources,
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
        let uses_relics = !relics.is_empty()
            || !resources.is_empty()
            || effects.iter().any(RewardEffect::uses_relic_owner);
        let initial_extent = if uses_relics {
            relics.retained_extent()?
        } else {
            0
        };
        let mut operation = || {
            Self::grant_effects_internal(
                inventory,
                skill_fragments,
                roster,
                uuid_manager,
                enkephalin,
                game_data,
                skill_fragment_policy,
                context,
                effects,
                seed,
                Some((relics, resources)),
            )
        };
        if uses_relics {
            let settings = game_data.runtime_safety();
            crate::game::battle::runtime_control::run_with_profile(
                settings.native_runtime(),
                settings,
                || {
                    crate::game::battle::runtime_control::seed_retained_extent(initial_extent)?;
                    operation()
                },
            )
        } else {
            operation()
        }
    }

    pub fn grant_reward_with_run_state(
        relics: &mut crate::game::relic::RelicInventory,
        resources: &mut crate::game::relic::RunCraftingResources,
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
        Self::grant_effects_with_run_state(
            relics,
            resources,
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

    fn grant_effects_internal(
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
        mut run_state: Option<(
            &mut crate::game::relic::RelicInventory,
            &mut crate::game::relic::RunCraftingResources,
        )>,
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
                run_state
                    .as_mut()
                    .map(|(relics, resources)| (&mut **relics, &mut **resources)),
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
                .employee_experience_diffs
                .extend(granted.employee_experience_diffs);
            result.relic_diffs.extend(granted.relic_diffs);
            if granted.crafting_resources.is_some() {
                result.crafting_resources = granted.crafting_resources;
            }
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
        run_state: Option<(
            &mut crate::game::relic::RelicInventory,
            &mut crate::game::relic::RunCraftingResources,
        )>,
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
                    1,
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
                    1,
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
            RewardEffect::GrantConsumable {
                consumable_id,
                quantity,
            } => {
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
                    *quantity,
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
                let copy_uuid =
                    skill_fragments.grant(metadata, uuid_manager, skill_fragment_policy)?;
                let count_after = skill_fragments.count(&metadata.id);
                result.skill_fragment_diffs.push(SkillFragmentGrantDiffDto {
                    copy_uuid,
                    fragment_id: metadata.id.clone(),
                    count_before,
                    count_after,
                });
                info!("Granted skill fragment reward: fragment_id={}", metadata.id);
            }
            RewardEffect::GrantFragmentDust { amount } => {
                let total = skill_fragments.add_fragment_dust(*amount)?;
                info!(
                    "Granted skill fragment dust: amount={}, new_total={}",
                    amount, total
                );
            }
            RewardEffect::GrantRelic { relic_id } => {
                let (relics, _) = run_state.ok_or(GameError::InvalidAction)?;
                crate::authoring::control::charge(
                    game_data.relic_data.relics.len() as u64,
                    "relic_grant_definition",
                )?;
                let definition = game_data.relic_data.get_by_id(relic_id).ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "reward references missing relic '{relic_id}'"
                    ))
                })?;
                let copy = relics.grant(definition, uuid_manager)?;
                result
                    .relic_diffs
                    .push(relics.get(copy).ok_or(GameError::InvalidAction)?.clone());
            }
            RewardEffect::GrantRelicCorruptionCatalyst { amount } => {
                let (_, resources) = run_state.ok_or(GameError::InvalidAction)?;
                resources.grant_relic_catalysts(*amount)?;
                result.crafting_resources = Some(*resources);
            }
            RewardEffect::GrantEquipmentDust { amount } => {
                let (_, resources) = run_state.ok_or(GameError::InvalidAction)?;
                resources.grant_equipment_dust(*amount)?;
                result.crafting_resources = Some(*resources);
            }
        }

        Ok(result)
    }

    fn grant_inventory_item(
        inventory: &mut Inventory,
        uuid_manager: &mut UuidManager,
        item: Item,
        quantity: u32,
        inventory_diff: &mut InventoryDiffDto,
    ) -> Result<(), GameError> {
        let owned_uuid = match &item {
            Item::Equipment(_) => uuid_manager.next_owned_equipment(),
            Item::Consumable(meta) => meta.uuid,
        };
        let outcome = match &item {
            Item::Consumable(meta) => {
                inventory.add_consumable(std::sync::Arc::clone(meta), quantity)?
            }
            _ if quantity == 1 => inventory.add_item_owned(owned_uuid, item.clone_arc())?,
            _ => return Err(GameError::InvalidAction),
        };
        let dto = match &item {
            Item::Consumable(meta) => {
                let stack = inventory
                    .consumables
                    .get_by_definition_id(&meta.id)
                    .ok_or(GameError::InventoryItemNotFound)?;
                InventoryItemDto::Consumable(
                    crate::game::resources::ConsumableItemDto::from_consumable_stack(stack),
                )
            }
            _ => InventoryItemDto::from_item_with_uuid(&item, owned_uuid)?,
        };
        match outcome {
            crate::game::resources::InventoryAddOutcome::Added => inventory_diff.added.push(dto),
            crate::game::resources::InventoryAddOutcome::Updated => {
                inventory_diff.updated.push(dto)
            }
        }

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
                .filter(|employee| employee.is_alive())
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

        let mut diffs = Vec::with_capacity(employee_ids.len());

        for employee_id in employee_ids {
            let employee = roster
                .get_mut(&employee_id)
                .ok_or(GameError::UnitNotFound)?;
            let level_before = employee.level;
            let experience_before = employee.experience;
            employee.add_experience_with_policy(amount, game_data.run_policy.as_ref());
            diffs.push(EmployeeExperienceDiffDto {
                employee_uuid: employee_id,
                amount,
                level_before,
                level_after: employee.level,
                experience_before,
                experience_after: employee.experience,
            });
        }

        Ok(diffs)
    }
}

#[cfg(test)]
mod tests {
    use super::RewardEffect;

    #[test]
    fn consumable_grant_requires_explicit_canonical_quantity() {
        let missing =
            ron::de::from_str::<RewardEffect>(r#"GrantConsumable(consumable_id: "field_tonic")"#)
                .expect_err("missing quantity must reject");
        assert!(missing.to_string().contains("quantity"));

        let legacy = ron::de::from_str::<RewardEffect>(
            r#"GrantConsumable(consumable_id: "field_tonic", quantity: 2, count: 2)"#,
        )
        .expect_err("legacy count must reject");
        assert!(legacy.to_string().contains("count"));

        let parsed = ron::de::from_str::<RewardEffect>(
            r#"GrantConsumable(consumable_id: "field_tonic", quantity: 2)"#,
        )
        .expect("canonical quantity should parse");
        assert!(matches!(
            parsed,
            RewardEffect::GrantConsumable {
                consumable_id,
                quantity: 2,
            } if consumable_id == "field_tonic"
        ));
    }

    #[test]
    fn removed_artifact_reward_vocabulary_is_rejected() {
        let error =
            ron::de::from_str::<RewardEffect>(r#"GrantArtifact(artifact_id: "legacy_artifact")"#)
                .expect_err("removed Artifact rewards must fail strict decoding");
        assert!(error.to_string().contains("GrantArtifact"));
    }
}

impl RewardEffect {
    pub(crate) fn uses_relic_owner(&self) -> bool {
        matches!(
            self,
            Self::GrantRelic { .. }
                | Self::GrantRelicCorruptionCatalyst { .. }
                | Self::GrantEquipmentDust { .. }
        )
    }
}
