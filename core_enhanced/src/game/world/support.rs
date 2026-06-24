use super::GameCore;
use crate::game::behavior::{
    BehaviorResult, GameError, MaintenanceItemPreviewDto, MaintenanceMaterialAmountDto,
    MaintenanceOperationPreviewDto, MaintenanceOperationsDto, MaintenanceOptionsDto,
    MaintenanceSlotKind, MaintenanceSourceDto, MaintenanceSourceKind, MaintenanceTargetKind,
    MaintenanceTargetStatePreviewDto, MaintenanceWarning,
};
use crate::game::data::equipment_data::EquipmentType;
use crate::game::employee::EmployeeLifeState;
use crate::game::employee_trust::{EmployeeTrustResolver, TrustEvent, TrustEventKind};
use crate::game::map::{MapProgression, RunMap, SupportNodeType};
use crate::game::resources::{MaintenanceSessionState, SupportSessionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StagedSupportEffect {
    None,
    SavePoint,
    Rest,
}

impl GameCore {
    pub(super) fn handle_choose_support(
        &mut self,
        support_type: SupportNodeType,
    ) -> Result<BehaviorResult, GameError> {
        {
            let support = self
                .state
                .active_node_content
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.select(support_type)?;
        }
        let support = {
            let support = self
                .state
                .active_node_content
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.clone()
        };
        let result = self.support_state_result(&support);
        self.refresh_allowed_actions();

        Ok(result)
    }

    pub(super) fn support_state_result(&self, support: &SupportSessionState) -> BehaviorResult {
        self.support_state_result_with_deliveries(support, vec![])
    }

    pub(super) fn support_state_result_with_deliveries(
        &self,
        support: &SupportSessionState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> BehaviorResult {
        BehaviorResult::SupportState {
            node_id: support.node_id,
            support_mode: support.support_mode,
            support_type: support.support_type,
            choices: support.choices.clone(),
            selected_support_type: support.selected_support_type,
            research_deliveries,
        }
    }

    pub(super) fn maintenance_state_result_with_deliveries(
        &self,
        maintenance: &MaintenanceSessionState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> BehaviorResult {
        BehaviorResult::MaintenanceState {
            node_id: maintenance.node_id,
            maintenance_options: self.maintenance_options(),
            research_deliveries,
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

    pub(super) fn default_support_choices() -> Vec<SupportNodeType> {
        vec![SupportNodeType::SavePoint, SupportNodeType::Rest]
    }

    pub(super) fn is_in_maintenance_node(&self) -> bool {
        self.state
            .active_node_content
            .as_ref()
            .is_some_and(|selected| selected.as_maintenance().is_ok())
    }

    pub(super) fn plan_current_support_node_effect(
        &self,
        map: &RunMap,
        progression: &MapProgression,
    ) -> Result<StagedSupportEffect, GameError> {
        let Some(node_id) = progression.current_node_id else {
            return Ok(StagedSupportEffect::None);
        };
        let Some(node) = map.node(node_id) else {
            return Ok(StagedSupportEffect::None);
        };
        let support_session = self
            .state
            .active_node_content
            .as_ref()
            .and_then(|selected| selected.as_support().ok())
            .filter(|support| support.node_id == node_id);

        let Some(support_session) = support_session else {
            if matches!(
                node.payload,
                crate::game::map::MapNodePayload::Support { .. }
            ) {
                return Err(GameError::InvalidAction);
            }
            return Ok(StagedSupportEffect::None);
        };
        let support_type = support_session.resolved_support_type()?;
        Ok(match support_type {
            SupportNodeType::SavePoint => StagedSupportEffect::SavePoint,
            SupportNodeType::Rest => StagedSupportEffect::Rest,
        })
    }

    pub(super) fn apply_staged_support_effect(
        &mut self,
        effect: StagedSupportEffect,
    ) -> Result<(), GameError> {
        match effect {
            StagedSupportEffect::None => Ok(()),
            StagedSupportEffect::SavePoint => self.support_save_point(),
            StagedSupportEffect::Rest => self.support_rest(),
        }
    }

    fn support_save_point(&mut self) -> Result<(), GameError> {
        let reduction_percent = self
            .run_policy()
            .support
            .save_point_trauma_reduction_percent;
        let roster = self.roster_mut()?;
        for employee in roster
            .iter_mut()
            .filter(|employee| employee.life_state == EmployeeLifeState::Alive)
        {
            let reduction = employee.trauma.saturating_mul(reduction_percent) / 100;
            employee.trauma = employee.trauma.saturating_sub(reduction);
        }
        Ok(())
    }

    fn support_rest(&mut self) -> Result<(), GameError> {
        let trust_policy = self.state.employee_trust_policy.clone();
        let rest_trauma_heal = self.run_policy().support.rest_trauma_heal;
        let roster = self.roster_mut()?;
        for employee in roster
            .iter_mut()
            .filter(|employee| employee.life_state == EmployeeLifeState::Alive)
        {
            employee.trauma = employee.trauma.saturating_sub(rest_trauma_heal);
            let reaction = EmployeeTrustResolver::apply_event(
                TrustEvent::new(employee.uuid, TrustEventKind::RestedAtSupportRest),
                &trust_policy,
            );
            employee.trust.apply_reaction(&reaction);
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
