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

        for fragment_id in self.state.skill_fragments.owned_ids() {
            items.push(self.maintenance_skill_fragment_preview(
                fragment_id,
                MaintenanceSourceDto {
                    source_type: MaintenanceSourceKind::Bag,
                    employee_uuid: None,
                    slot_kind: None,
                },
                false,
            ));
        }

        for employee in self.state.roster.iter() {
            if let Some(fragment_id) = employee.skill_fragments.active_fragment_id() {
                items.push(self.maintenance_skill_fragment_preview(
                    fragment_id,
                    MaintenanceSourceDto {
                        source_type: MaintenanceSourceKind::Equipped,
                        employee_uuid: Some(employee.uuid),
                        slot_kind: Some(MaintenanceSlotKind::SkillFragment),
                    },
                    true,
                ));
            }
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
                (ActionKind::UpgradeSkillFragment, MaintenanceTargetKind::SkillFragment) => {
                    item.operations.enhance.can_execute
                }
                (ActionKind::AwakenSkillFragment, MaintenanceTargetKind::SkillFragment) => {
                    item.operations.awaken.can_execute
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
        let costs = self
            .game_data
            .equipment_data
            .get_enhancement_recipe_by_equipment_id(&equipment.meta.id)
            .map(|recipe| {
                recipe
                    .costs_per_level
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
                .get_enhancement_recipe_by_equipment_id(&equipment.meta.id)
                .is_some_and(|recipe| {
                    equipment.enhancement_level < recipe.max_level
                        && recipe.costs_per_level.iter().all(|cost| {
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
        fragment_id: &crate::game::data::skill_fragment_data::SkillFragmentId,
        source: MaintenanceSourceDto,
        will_unequip: bool,
    ) -> MaintenanceItemPreviewDto {
        let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
        let before = self.skill_fragment_state_preview(fragment_id);
        MaintenanceItemPreviewDto {
            target_id: fragment_id.to_string(),
            target_kind: MaintenanceTargetKind::SkillFragment,
            equipment_type: None,
            display_name: metadata
                .map(|fragment| fragment.name.clone())
                .unwrap_or_else(|| fragment_id.to_string()),
            source,
            operations: MaintenanceOperationsDto {
                dismantle: self.skill_fragment_dismantle_preview(
                    fragment_id,
                    before.clone(),
                    will_unequip,
                ),
                enhance: self.skill_fragment_enhance_preview(fragment_id, before.clone()),
                awaken: self.skill_fragment_awaken_preview(fragment_id, before),
            },
        }
    }

    fn skill_fragment_state_preview(
        &self,
        fragment_id: &crate::game::data::skill_fragment_data::SkillFragmentId,
    ) -> MaintenanceTargetStatePreviewDto {
        let progress = self.state.skill_fragments.progress(fragment_id);
        MaintenanceTargetStatePreviewDto {
            stack_count: Some(self.state.skill_fragments.count(fragment_id)),
            enhancement_level: None,
            upgrade_level: Some(progress.upgrade_level),
            awakening_progress: Some(progress.awakening_progress),
            awakening_available: Some(progress.awakening_available),
            awakened: Some(progress.awakened),
        }
    }

    fn skill_fragment_dismantle_preview(
        &self,
        fragment_id: &crate::game::data::skill_fragment_data::SkillFragmentId,
        before: MaintenanceTargetStatePreviewDto,
        will_unequip: bool,
    ) -> MaintenanceOperationPreviewDto {
        let gains = self
            .state
            .skill_fragment_policy
            .dismantle
            .validate_dismantle(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .map(|amount| {
                vec![MaintenanceMaterialAmountDto {
                    material_id: "fragment_dust".to_string(),
                    amount,
                }]
            })
            .unwrap_or_default();
        let can_execute = !gains.is_empty();
        let after_count = self
            .state
            .skill_fragments
            .count(fragment_id)
            .saturating_sub(1);
        MaintenanceOperationPreviewDto {
            can_execute,
            disabled_reason: (!can_execute).then(|| "cannot_dismantle_skill_fragment".to_string()),
            costs: vec![],
            gains,
            before: before.clone(),
            after: Some(MaintenanceTargetStatePreviewDto {
                stack_count: Some(after_count),
                ..before
            }),
            requires_confirm: true,
            will_unequip,
            warnings: will_unequip
                .then(|| MaintenanceWarning::EquippedItemWillBeUnequipped)
                .into_iter()
                .collect(),
        }
    }

    fn skill_fragment_enhance_preview(
        &self,
        fragment_id: &crate::game::data::skill_fragment_data::SkillFragmentId,
        before: MaintenanceTargetStatePreviewDto,
    ) -> MaintenanceOperationPreviewDto {
        let cost = self
            .state
            .skill_fragment_policy
            .composition
            .dust_upgrade_cost(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .ok();
        let can_execute =
            cost.is_some_and(|amount| self.state.skill_fragments.fragment_dust() >= amount);
        let progress = self.state.skill_fragments.progress(fragment_id);
        let next_awakening_progress = progress.awakening_progress.saturating_add(
            self.state
                .skill_fragment_policy
                .composition
                .awakening_progress_per_upgrade,
        );
        MaintenanceOperationPreviewDto {
            can_execute,
            disabled_reason: (!can_execute).then(|| "not_enough_fragment_dust".to_string()),
            costs: cost
                .map(|amount| {
                    vec![MaintenanceMaterialAmountDto {
                        material_id: "fragment_dust".to_string(),
                        amount,
                    }]
                })
                .unwrap_or_default(),
            gains: vec![],
            before: before.clone(),
            after: Some(MaintenanceTargetStatePreviewDto {
                upgrade_level: Some(progress.upgrade_level.saturating_add(1)),
                awakening_progress: Some(next_awakening_progress),
                awakening_available: Some(
                    progress.awakening_available
                        || next_awakening_progress
                            >= self
                                .state
                                .skill_fragment_policy
                                .composition
                                .awakening_threshold,
                ),
                ..before
            }),
            requires_confirm: true,
            will_unequip: false,
            warnings: vec![],
        }
    }

    fn skill_fragment_awaken_preview(
        &self,
        fragment_id: &crate::game::data::skill_fragment_data::SkillFragmentId,
        before: MaintenanceTargetStatePreviewDto,
    ) -> MaintenanceOperationPreviewDto {
        let cost = self
            .state
            .skill_fragment_policy
            .composition
            .dust_awakening_cost(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .ok();
        let can_execute =
            cost.is_some_and(|amount| self.state.skill_fragments.fragment_dust() >= amount);
        MaintenanceOperationPreviewDto {
            can_execute,
            disabled_reason: (!can_execute).then(|| "cannot_awaken_skill_fragment".to_string()),
            costs: cost
                .map(|amount| {
                    vec![MaintenanceMaterialAmountDto {
                        material_id: "fragment_dust".to_string(),
                        amount,
                    }]
                })
                .unwrap_or_default(),
            gains: vec![],
            before: before.clone(),
            after: Some(MaintenanceTargetStatePreviewDto {
                awakening_available: Some(true),
                awakened: Some(true),
                ..before
            }),
            requires_confirm: true,
            will_unequip: false,
            warnings: vec![],
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
