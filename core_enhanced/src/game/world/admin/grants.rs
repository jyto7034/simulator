use serde_json::json;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    employee::EmployeeCondition,
    reward::{GrantExecutionContext, GrantExecutionResult, RewardEffect},
};

use super::{AdminCommandOutput, GameCore};
use crate::game::world::grant_state::EconomyGrantState;

impl GameCore {
    pub(super) fn admin_grant_equipment(
        &mut self,
        definition_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let metadata = self
            .game_data
            .equipment_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing equipment '{definition_id}'"
                ))
            })?
            .clone();
        let effects = (0..count)
            .map(|_| RewardEffect::GrantEquipment {
                equipment_id: metadata.id.clone(),
            })
            .collect::<Vec<_>>();
        let granted = self.admin_apply_grant_effects(&effects)?;
        let item_uuids = granted
            .inventory_diff
            .added
            .iter()
            .map(|item| item.uuid())
            .collect::<Vec<_>>();
        Ok(self.admin_output(
            "AdminGrantedEquipment",
            json!({ "definition_id": definition_id, "item_uuids": item_uuids }),
        ))
    }

    pub(super) fn admin_grant_consumable(
        &mut self,
        definition_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let metadata = self
            .game_data
            .consumable_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing consumable '{definition_id}'"
                ))
            })?
            .clone();
        self.admin_apply_grant_effects(&[RewardEffect::GrantConsumable {
            consumable_id: metadata.id.clone(),
            quantity: count,
        }])?;
        let quantity = self
            .state
            .inventory
            .consumables
            .get_by_definition_id(definition_id)
            .ok_or(GameError::InventoryItemNotFound)?
            .quantity;
        Ok(self.admin_output(
            "AdminGrantedConsumable",
            json!({
                "definition_id": definition_id,
                "item_uuid": metadata.uuid,
                "quantity": quantity,
            }),
        ))
    }

    pub(super) fn admin_grant_skill_fragment(
        &mut self,
        fragment_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let fragment_id =
            crate::game::data::skill_fragment_data::SkillFragmentId::from(fragment_id);
        if self
            .game_data
            .skill_fragment_data
            .get_by_id(&fragment_id)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(format!(
                "admin grant references missing skill fragment '{fragment_id}'"
            )));
        }
        let effects = (0..count)
            .map(|_| RewardEffect::GrantSkillFragment {
                fragment_id: fragment_id.clone(),
            })
            .collect::<Vec<_>>();
        self.admin_apply_grant_effects(&effects)?;
        Ok(self.admin_output(
            "AdminGrantedSkillFragment",
            json!({
                "fragment_id": fragment_id,
                "count": self.state.skill_fragments.count(&fragment_id),
            }),
        ))
    }

    pub(super) fn admin_grant_equipment_material(
        &mut self,
        material_id: &str,
        amount: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        if self
            .game_data
            .equipment_data
            .get_material_by_id(material_id)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(format!(
                "admin grant references missing equipment material '{material_id}'"
            )));
        }
        self.admin_apply_grant_effects(&[RewardEffect::GrantEquipmentMaterial {
            material_id: material_id.to_string(),
            amount,
        }])?;
        let amount = self.state.inventory.equipment_materials.amount(material_id);
        Ok(self.admin_output(
            "AdminGrantedEquipmentMaterial",
            json!({ "material_id": material_id, "amount": amount }),
        ))
    }

    pub(super) fn admin_grant_fragment_dust(
        &mut self,
        amount: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        self.admin_apply_grant_effects(&[RewardEffect::GrantFragmentDust { amount }])?;
        let amount = self.state.skill_fragments.fragment_dust();
        Ok(self.admin_output("AdminGrantedFragmentDust", json!({ "amount": amount })))
    }

    pub(super) fn admin_grant_relic(
        &mut self,
        definition_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        crate::authoring::control::charge(
            self.game_data.relic_data.relics.len() as u64,
            "relic_admin_definition",
        )?;
        if self.game_data.relic_data.get_by_id(definition_id).is_none() {
            return Err(GameError::InvalidAction);
        }
        let mut copies = Vec::new();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            (count as usize).checked_mul(
                std::mem::size_of::<crate::game::relic::OwnedRelic>() + definition_id.len(),
            ),
            "relic_admin_grant_projection",
        )?;
        for _ in 0..count {
            crate::authoring::control::poll("relic_admin_grant_loop")?;
            let granted = self.admin_apply_grant_effects(&[RewardEffect::GrantRelic {
                relic_id: definition_id.to_owned(),
            }])?;
            copies.extend(granted.relic_diffs);
        }
        Ok(self.admin_output(
            "AdminGrantedRelic",
            json!({ "definition_id": definition_id, "copies": copies }),
        ))
    }
    pub(super) fn admin_grant_relic_resources(
        &mut self,
        effect: RewardEffect,
    ) -> Result<AdminCommandOutput, GameError> {
        self.ensure_relic_operation()?;
        self.admin_apply_grant_effects(&[effect])?;
        Ok(self.admin_output(
            "AdminGrantedCraftingResources",
            json!({ "resources": self.state.crafting_resources }),
        ))
    }
    fn admin_apply_grant_effects(
        &mut self,
        effects: &[RewardEffect],
    ) -> Result<GrantExecutionResult, GameError> {
        self.ensure_relic_grant(effects)?;
        let mut grant_state = EconomyGrantState::capture(&self.state)?;
        let granted = grant_state.grant_effects(
            &self.game_data,
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            effects,
            self.run_seed ^ 0x4144_4d49_4e47_524e,
        )?;

        grant_state.commit_into(&mut self.state);

        Ok(granted)
    }

    pub(super) fn admin_set_employee_condition(
        &mut self,
        employee_uuid: Uuid,
        physical_damage: u8,
        trauma: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        let condition = {
            let employee = self
                .state
                .roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if physical_damage > crate::game::employee::MAX_PHYSICAL_DAMAGE
                || trauma > crate::game::employee::MAX_TRAUMA
            {
                return Err(GameError::InvalidAction);
            }
            employee.condition = EmployeeCondition::new(physical_damage, trauma);
            employee.condition
        };
        Ok(self.admin_output(
            "AdminSetEmployeeCondition",
            json!({
                "employee_uuid": employee_uuid,
                "physical_damage": condition.physical_damage,
                "trauma": condition.trauma,
            }),
        ))
    }

    pub(super) fn admin_set_enkephalin(&mut self, amount: u32) -> AdminCommandOutput {
        self.state.enkephalin.amount = amount;
        self.admin_output("AdminSetEnkephalin", json!({ "amount": amount }))
    }
}
