use serde_json::json;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    employee::EmployeeLifeState,
    reward::{GrantExecutionContext, GrantExecutionResult, GrantExecutor, RewardEffect},
};

use super::{AdminCommandOutput, GameCore};

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
        let effects = (0..count)
            .map(|_| RewardEffect::GrantConsumable {
                consumable_id: metadata.id.clone(),
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
            "AdminGrantedConsumable",
            json!({ "definition_id": definition_id, "item_uuids": item_uuids }),
        ))
    }

    pub(super) fn admin_grant_artifact(
        &mut self,
        definition_id: &str,
    ) -> Result<AdminCommandOutput, GameError> {
        let metadata = self
            .game_data
            .artifact_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing artifact '{definition_id}'"
                ))
            })?
            .clone();
        let artifact_uuid = metadata.uuid;
        self.admin_apply_grant_effects(&[RewardEffect::GrantArtifact {
            artifact_id: metadata.id.clone(),
        }])?;
        Ok(self.admin_output(
            "AdminGrantedArtifact",
            json!({ "definition_id": definition_id, "artifact_uuid": artifact_uuid }),
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

    fn admin_apply_grant_effects(
        &mut self,
        effects: &[RewardEffect],
    ) -> Result<GrantExecutionResult, GameError> {
        let mut next_inventory = self.state.inventory.clone();
        let mut next_skill_fragments = self.state.skill_fragments.clone();
        let mut next_roster = self.state.roster.clone();
        let mut next_uuid_manager = self.state.uuid_manager.clone();
        let mut next_enkephalin = self.state.enkephalin.clone();
        let granted = GrantExecutor::grant_effects_with_state(
            &mut next_inventory,
            &mut next_skill_fragments,
            &mut next_roster,
            &mut next_uuid_manager,
            &mut next_enkephalin,
            &self.game_data,
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            effects,
            self.run_seed ^ 0x4144_4d49_4e47_524e,
        )?;

        self.state.inventory = next_inventory;
        self.state.skill_fragments = next_skill_fragments;
        self.state.roster = next_roster;
        self.state.uuid_manager = next_uuid_manager;
        self.state.enkephalin = next_enkephalin;

        Ok(granted)
    }

    pub(super) fn admin_set_employee_hp(
        &mut self,
        employee_uuid: Uuid,
        current_hp: u32,
        max_hp: Option<u32>,
    ) -> Result<AdminCommandOutput, GameError> {
        let (current_hp, max_hp) = {
            let employee = self
                .state
                .roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if employee.life_state != EmployeeLifeState::Alive {
                return Err(GameError::InvalidAction);
            }
            if let Some(max_hp) = max_hp {
                if max_hp == 0 {
                    return Err(GameError::InvalidAction);
                }
                employee.health.max_hp = max_hp;
            }
            employee.health.current_hp = current_hp.min(employee.health.max_hp);
            (employee.health.current_hp, employee.health.max_hp)
        };
        Ok(self.admin_output(
            "AdminSetEmployeeHp",
            json!({
                "employee_uuid": employee_uuid,
                "current_hp": current_hp,
                "max_hp": max_hp,
            }),
        ))
    }

    pub(super) fn admin_set_employee_trauma(
        &mut self,
        employee_uuid: Uuid,
        trauma: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        let trauma = {
            let employee = self
                .state
                .roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if employee.life_state != EmployeeLifeState::Alive {
                return Err(GameError::InvalidAction);
            }
            employee.trauma = trauma;
            employee.trauma
        };
        Ok(self.admin_output(
            "AdminSetEmployeeTrauma",
            json!({ "employee_uuid": employee_uuid, "trauma": trauma }),
        ))
    }

    pub(super) fn admin_set_enkephalin(&mut self, amount: u32) -> AdminCommandOutput {
        self.state.enkephalin.amount = amount;
        self.admin_output("AdminSetEnkephalin", json!({ "amount": amount }))
    }
}
