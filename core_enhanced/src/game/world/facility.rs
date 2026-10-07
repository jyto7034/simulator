use super::GameCore;
// SavePoint recovery and non-combat facility-node operations.

use crate::game::behavior::{
    ActionKind, BehaviorResult, GameError, MaintenanceItemPreviewDto, MaintenanceMaterialAmountDto,
    MaintenanceOperationPreviewDto, MaintenanceOperationsDto, MaintenanceOptionsDto,
    MaintenanceSlotKind, MaintenanceSourceDto, MaintenanceSourceKind, MaintenanceTargetKind,
    MaintenanceTargetStatePreviewDto, MaintenanceWarning, TraumaRecoveryEmployeePreviewDto,
    TraumaRecoveryPreviewDto,
};
use crate::game::data::equipment_data::EquipmentType;
use crate::game::employee::EmployeeActivity;
use crate::game::map::{MapNodeCategory, MapProgression, RunMap};
use crate::game::resources::{MaintenanceSessionState, SavePointSessionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StagedRecoveryEffect {
    None,
    SavePoint,
    GateTransition,
}

impl GameCore {
    pub(super) fn save_point_state_result(
        &self,
        save_point: &SavePointSessionState,
    ) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::SavePointState {
            node_id: save_point.node_id,
            recovery_preview: self
                .trauma_recovery_preview_for_category(MapNodeCategory::SavePoint)?
                .ok_or(GameError::InvalidAction)?,
        })
    }

    pub(super) fn trauma_recovery_preview_for_category(
        &self,
        category: MapNodeCategory,
    ) -> Result<Option<TraumaRecoveryPreviewDto>, GameError> {
        let (recovery, replaces_run_checkpoint) = match category {
            MapNodeCategory::SavePoint => (self.run_policy().save_point_trauma_recovery, true),
            MapNodeCategory::Gate => (self.run_policy().gate_trauma_recovery, false),
            _ => return Ok(None),
        };
        let employees = self
            .roster()?
            .iter()
            .map(|employee| {
                let eligibility = employee.eligibility(EmployeeActivity::TraumaRecovery);
                let trauma_after = if employee.is_alive() {
                    recovery.apply(employee.condition.trauma)
                } else {
                    employee.condition.trauma
                };
                let mut after_condition = employee.condition;
                after_condition.trauma = trauma_after;
                TraumaRecoveryEmployeePreviewDto {
                    employee_uuid: employee.uuid,
                    employee_name: employee.name.clone(),
                    eligibility,
                    physical_damage: employee.condition.physical_damage,
                    trauma_before: employee.condition.trauma,
                    trauma_after,
                    mental_state_before: employee.condition.mental_state(),
                    mental_state_after: after_condition.mental_state(),
                }
            })
            .collect();
        Ok(Some(TraumaRecoveryPreviewDto {
            recovery,
            employees,
            physical_damage_unchanged: true,
            replaces_run_checkpoint,
        }))
    }

    pub(super) fn maintenance_state_result(
        &self,
        maintenance: &MaintenanceSessionState,
    ) -> BehaviorResult {
        BehaviorResult::MaintenanceState {
            node_id: maintenance.node_id,
            maintenance_options: self.maintenance_options(),
        }
    }

    pub(super) fn maintenance_options(&self) -> MaintenanceOptionsDto {
        let Ok(inventory) = self.inventory() else {
            return MaintenanceOptionsDto {
                items: Vec::new(),
                materials: self.maintenance_materials_snapshot(),
            };
        };
        let mut items = Vec::new();
        for equipment in inventory.equipments.iter() {
            items.push(self.maintenance_equipment_preview(equipment));
        }

        for copy in self.state.skill_fragments.copies() {
            items.push(self.maintenance_skill_fragment_preview(copy));
        }

        items.sort_by(|left, right| {
            left.target_kind
                .cmp(&right.target_kind)
                .then_with(|| left.target_id.cmp(&right.target_id))
                .then_with(|| left.display_name.cmp(&right.display_name))
        });

        MaintenanceOptionsDto {
            items,
            materials: self.maintenance_materials_snapshot(),
        }
    }

    pub(super) fn maintenance_action_has_executable_payload(
        options: &MaintenanceOptionsDto,
        action: ActionKind,
    ) -> bool {
        options
            .items
            .iter()
            .any(|item| match (action, item.target_kind) {
                (ActionKind::SelectSkillFragmentForm, MaintenanceTargetKind::SkillFragment) => {
                    item.fragment_forms.iter().any(|form| form.can_select)
                }
                (ActionKind::AwakenSkillFragment, MaintenanceTargetKind::SkillFragment) => {
                    item.fragment_forms.iter().any(|form| form.can_unlock)
                }
                (ActionKind::DismantleSkillFragment, MaintenanceTargetKind::SkillFragment) => {
                    item.operations.dismantle.can_execute
                }
                (ActionKind::DismantleEquipment, MaintenanceTargetKind::Equipment) => {
                    item.operations.dismantle.can_execute
                }
                (ActionKind::EnhanceEquipment, MaintenanceTargetKind::Equipment) => {
                    item.operations.enhance.can_execute
                }
                _ => false,
            })
    }

    fn maintenance_materials_snapshot(&self) -> Vec<MaintenanceMaterialAmountDto> {
        let mut materials = vec![
            MaintenanceMaterialAmountDto {
                material_id: "fragment_dust".to_string(),
                amount: self.state.skill_fragments.fragment_dust(),
            },
            MaintenanceMaterialAmountDto {
                material_id: "equipment_dust".to_string(),
                amount: self
                    .state
                    .inventory
                    .equipment_materials
                    .amount("equipment_dust"),
            },
        ];
        for (material_id, amount) in self.state.inventory.equipment_materials.iter() {
            if material_id == "equipment_dust" {
                continue;
            }
            materials.push(MaintenanceMaterialAmountDto {
                material_id: material_id.clone(),
                amount: *amount,
            });
        }
        materials.sort_by(|left, right| left.material_id.cmp(&right.material_id));
        materials
    }

    fn maintenance_equipment_preview(
        &self,
        equipment: &crate::game::resources::OwnedEquipment,
    ) -> MaintenanceItemPreviewDto {
        let will_unequip = equipment.equipped_to.is_some();
        let source = MaintenanceSourceDto {
            source_type: if will_unequip {
                MaintenanceSourceKind::Equipped
            } else {
                MaintenanceSourceKind::Bag
            },
            employee_uuid: equipment.equipped_to,
            slot_kind: will_unequip.then(|| equipment_slot_kind(equipment.meta.equipment_type)),
        };
        let before = MaintenanceTargetStatePreviewDto {
            enhancement_level: Some(equipment.enhancement_level),
            ..MaintenanceTargetStatePreviewDto::default()
        };

        MaintenanceItemPreviewDto {
            target_id: equipment.instance_uuid.to_string(),
            target_kind: MaintenanceTargetKind::Equipment,
            fragment_forms: vec![],
            equipment_type: Some(equipment.meta.equipment_type),
            display_name: equipment.meta.name.clone(),
            source,
            operations: MaintenanceOperationsDto {
                dismantle: self.equipment_dismantle_preview(
                    equipment,
                    before.clone(),
                    will_unequip,
                ),
                enhance: self.equipment_enhance_preview(equipment, before.clone()),
                awaken: MaintenanceOperationPreviewDto {
                    can_execute: false,
                    disabled_reason: Some("equipment_awaken_unsupported".to_string()),
                    costs: vec![],
                    gains: vec![],
                    before,
                    after: None,
                    requires_confirm: false,
                    will_unequip: false,
                    warnings: vec![],
                },
            },
        }
    }

    fn equipment_dismantle_preview(
        &self,
        equipment: &crate::game::resources::OwnedEquipment,
        before: MaintenanceTargetStatePreviewDto,
        will_unequip: bool,
    ) -> MaintenanceOperationPreviewDto {
        let gains = self
            .game_data
            .equipment_data
            .get_dismantle_recipe_by_equipment_id(&equipment.meta.id)
            .map(|recipe| {
                recipe
                    .yields
                    .iter()
                    .map(|material| MaintenanceMaterialAmountDto {
                        material_id: material.material_id.clone(),
                        amount: material.amount,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let can_execute = !equipment.meta.bound
            && self
                .game_data
                .equipment_data
                .get_dismantle_recipe_by_equipment_id(&equipment.meta.id)
                .is_some_and(|recipe| {
                    recipe.yields.iter().all(|material| {
                        let current = self
                            .state
                            .inventory
                            .equipment_materials
                            .amount(&material.material_id);
                        current <= u32::MAX.saturating_sub(material.amount)
                    })
                });
        MaintenanceOperationPreviewDto {
            can_execute,
            disabled_reason: (!can_execute).then(|| "cannot_dismantle_equipment".to_string()),
            costs: vec![],
            gains,
            before,
            after: None,
            requires_confirm: true,
            will_unequip,
            warnings: will_unequip
                .then(|| MaintenanceWarning::EquippedItemWillBeUnequipped)
                .into_iter()
                .collect(),
        }
    }

    fn equipment_enhance_preview(
        &self,
        equipment: &crate::game::resources::OwnedEquipment,
        before: MaintenanceTargetStatePreviewDto,
    ) -> MaintenanceOperationPreviewDto {
        let recipe = self
            .game_data
            .equipment_data
            .get_enhancement_recipe_by_equipment_id(&equipment.meta.id);
        let aggregated_costs = recipe.and_then(|recipe| recipe.aggregated_costs());
        let costs = aggregated_costs
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|material| MaintenanceMaterialAmountDto {
                material_id: material.material_id.clone(),
                amount: material.amount,
            })
            .collect::<Vec<_>>();
        let can_execute = !equipment.meta.bound
            && recipe
                .zip(aggregated_costs.as_ref())
                .is_some_and(|(recipe, costs)| {
                    equipment.enhancement_level < recipe.max_level
                        && costs.iter().all(|cost| {
                            self.state
                                .inventory
                                .equipment_materials
                                .amount(&cost.material_id)
                                >= cost.amount
                        })
                });
        MaintenanceOperationPreviewDto {
            can_execute,
            disabled_reason: (!can_execute).then(|| "cannot_enhance_equipment".to_string()),
            costs,
            gains: vec![],
            before: before.clone(),
            after: Some(MaintenanceTargetStatePreviewDto {
                enhancement_level: Some(equipment.enhancement_level.saturating_add(1)),
                ..before
            }),
            requires_confirm: true,
            will_unequip: false,
            warnings: vec![],
        }
    }

    fn maintenance_skill_fragment_preview(
        &self,
        copy: &crate::game::skill_fragment::OwnedFragment,
    ) -> MaintenanceItemPreviewDto {
        let metadata = self
            .game_data
            .skill_fragment_data
            .get_by_id(&copy.definition_id);
        let before = MaintenanceTargetStatePreviewDto {
            stack_count: Some(self.state.skill_fragments.count(&copy.definition_id)),
            enhancement_level: None,
            fragment_copy_uuid: Some(copy.copy_uuid),
            selected_form: Some(copy.selected_form.clone()),
            unlocked_forms: copy.unlocked_forms.iter().cloned().collect(),
            owner: copy.equipped_to,
        };
        let unavailable = |reason: &str| MaintenanceOperationPreviewDto {
            can_execute: false,
            disabled_reason: Some(reason.into()),
            costs: vec![],
            gains: vec![],
            before: before.clone(),
            after: None,
            requires_confirm: false,
            will_unequip: false,
            warnings: vec![],
        };
        let effects = self
            .fragment_dismantle_effects(copy.copy_uuid, true)
            .and_then(|(_, effects)| {
                self.preview_grant_effects(&effects)?;
                Ok(effects)
            });
        let mut dismantle = unavailable("fragment_recipe_or_grant_unavailable");
        if let Ok(effects) = effects {
            dismantle.can_execute = true;
            dismantle.disabled_reason = None;
            dismantle.gains = effects
                .iter()
                .filter_map(|effect| match effect {
                    crate::game::reward::RewardEffect::GrantEquipmentMaterial {
                        material_id,
                        amount,
                    } => Some(MaintenanceMaterialAmountDto {
                        material_id: material_id.clone(),
                        amount: *amount,
                    }),
                    crate::game::reward::RewardEffect::GrantEnkephalin { amount } => {
                        Some(MaintenanceMaterialAmountDto {
                            material_id: "enkephalin".into(),
                            amount: *amount,
                        })
                    }
                    _ => None,
                })
                .collect();
            dismantle.requires_confirm = copy.equipped_to.is_some();
            dismantle.will_unequip = copy.equipped_to.is_some();
            dismantle.warnings = copy
                .equipped_to
                .map(|_| MaintenanceWarning::EquippedItemWillBeUnequipped)
                .into_iter()
                .collect();
        }
        MaintenanceItemPreviewDto {
            target_id: copy.copy_uuid.to_string(),
            target_kind: MaintenanceTargetKind::SkillFragment,
            equipment_type: None,
            display_name: metadata
                .map(|m| m.name.clone())
                .unwrap_or_else(|| copy.definition_id.to_string()),
            source: MaintenanceSourceDto {
                source_type: if copy.equipped_to.is_some() {
                    MaintenanceSourceKind::Equipped
                } else {
                    MaintenanceSourceKind::Bag
                },
                employee_uuid: copy.equipped_to,
                slot_kind: copy.equipped_to.map(|_| MaintenanceSlotKind::SkillFragment),
            },
            operations: MaintenanceOperationsDto {
                dismantle,
                enhance: unavailable("fragment_upgrade_removed"),
                awaken: unavailable("select_explicit_fragment_form"),
            },
            fragment_forms: self.fragment_forms_snapshot(copy),
        }
    }

    pub(super) fn is_in_maintenance_node(&self) -> bool {
        self.state
            .active_node_content
            .as_ref()
            .is_some_and(|selected| selected.as_maintenance().is_ok())
    }

    pub(super) fn plan_current_recovery_effect(
        &self,
        map: &RunMap,
        progression: &MapProgression,
    ) -> Result<StagedRecoveryEffect, GameError> {
        let Some(node_id) = progression.current_node_id else {
            return Ok(StagedRecoveryEffect::None);
        };
        let Some(node) = map.node(node_id) else {
            return Ok(StagedRecoveryEffect::None);
        };
        let save_point_session = self
            .state
            .active_node_content
            .as_ref()
            .and_then(|selected| selected.as_save_point().ok())
            .filter(|save_point| save_point.node_id == node_id);

        let Some(_) = save_point_session else {
            if matches!(node.payload, crate::game::map::MapNodePayload::SavePoint) {
                return Err(GameError::InvalidAction);
            }
            return Ok(StagedRecoveryEffect::None);
        };
        Ok(StagedRecoveryEffect::SavePoint)
    }

    pub(super) fn apply_staged_recovery_effect(
        &mut self,
        effect: StagedRecoveryEffect,
    ) -> Result<(), GameError> {
        match effect {
            StagedRecoveryEffect::None => Ok(()),
            StagedRecoveryEffect::SavePoint => self.apply_save_point_recovery(),
            StagedRecoveryEffect::GateTransition => self.apply_gate_recovery(),
        }
    }

    fn apply_save_point_recovery(&mut self) -> Result<(), GameError> {
        let recovery = self.run_policy().save_point_trauma_recovery;
        let roster = self.roster_mut()?;
        for employee in roster.iter_mut().filter(|employee| employee.is_alive()) {
            employee.condition.trauma = recovery.apply(employee.condition.trauma);
        }
        Ok(())
    }

    fn apply_gate_recovery(&mut self) -> Result<(), GameError> {
        let recovery = self.run_policy().gate_trauma_recovery;
        let roster = self.roster_mut()?;
        for employee in roster.iter_mut().filter(|employee| employee.is_alive()) {
            employee.condition.trauma = recovery.apply(employee.condition.trauma);
        }
        Ok(())
    }
}

fn equipment_slot_kind(equipment_type: EquipmentType) -> MaintenanceSlotKind {
    match equipment_type {
        EquipmentType::Weapon => MaintenanceSlotKind::Weapon,
        EquipmentType::Armor => MaintenanceSlotKind::Armor,
        EquipmentType::Accessory => MaintenanceSlotKind::Accessory,
    }
}
